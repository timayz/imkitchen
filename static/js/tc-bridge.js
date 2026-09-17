// Vanilla bridge for what the topcoat runtime (0.8.x) cannot express:
// visibility triggers, toasts, and recovery from stale runtime endpoints.
// Loaded synchronously in <head>, before the runtime module hydrates the page.
(function () {
  "use strict";

  var imk = (window.imk = window.imk || {});

  // ── Timezone ──────────────────────────────────────────────────────────────
  // Replaces the `TS-Timezone` request header the patched twinspark.js sent.
  // A cookie also reaches plain form posts and sendBeacon, which a header can't.
  try {
    var tz = Intl.DateTimeFormat().resolvedOptions().timeZone;
    if (tz && document.cookie.indexOf("tz=" + tz) === -1) {
      document.cookie = "tz=" + tz + ";path=/;max-age=31536000;samesite=lax";
    }
  } catch (e) {}

  // ── Toasts ────────────────────────────────────────────────────────────────
  // Same markup and 5s auto-dismiss as the askama partials/toast-*.html.
  var ICONS = {
    error:
      "M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z",
    success: "M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z",
  };
  var COLORS = { error: "#c3321a", success: "#2f7d4f" };

  // The container is created here and attached to <html>, *outside* <body>: a
  // topcoat page re-run morphs every child of <body> back to what the server
  // rendered, which would delete toasts living in a server-rendered container.
  var toastContainer = function () {
    var container = document.getElementById("toast-container");
    if (!container) {
      container = document.createElement("div");
      container.id = "toast-container";
      container.className = "fixed bottom-4 right-4 z-50 space-y-2 max-w-md";
      document.documentElement.appendChild(container);
    }
    return container;
  };

  imk.toast = function (kind, message) {
    if (!message) return;
    var container = toastContainer();

    var color = COLORS[kind] || COLORS.error;
    var toast = document.createElement("div");
    toast.setAttribute("role", "status");
    toast.className =
      "bg-paper border-l-4 rounded-xl shadow-md p-4 transform transition-all duration-300";
    toast.style.borderLeftColor = color;
    toast.innerHTML =
      '<div class="flex items-center gap-3">' +
      '<svg class="w-5 h-5 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">' +
      '<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"></path></svg>' +
      '<div class="flex-1 text-sm font-semibold text-ink"></div>' +
      '<button type="button" class="text-ink-3 hover:text-ink-2" aria-label="Close">' +
      '<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">' +
      '<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>' +
      "</svg></button></div>";
    toast.querySelector("svg").style.color = color;
    toast.querySelector("path").setAttribute("d", ICONS[kind] || ICONS.error);
    // textContent, never innerHTML: the message may echo user input.
    toast.querySelector(".flex-1").textContent = message;
    toast.querySelector("button").addEventListener("click", function () {
      toast.remove();
    });

    container.appendChild(toast);
    setTimeout(function () {
      toast.remove();
    }, 5000);
  };

  // ── Visibility trigger ────────────────────────────────────────────────────
  // `<div data-tc-visible @tc-visible=$(...)>` fires once each time the element
  // scrolls into view. A MutationObserver picks up sentinels that shards morph in.
  if ("IntersectionObserver" in window) {
    var io = new IntersectionObserver(
      function (entries) {
        entries.forEach(function (entry) {
          if (!entry.isIntersecting) return;
          var el = entry.target;
          el.dispatchEvent(new CustomEvent("tc-visible"));
          // A shard re-render morphs the sentinel in place. If it is still in
          // view afterwards (short page), the observer would never fire again
          // because the intersection state did not change: re-arm it.
          io.unobserve(el);
          setTimeout(function () {
            if (el.isConnected) io.observe(el);
          }, 400);
        });
      },
      { rootMargin: "200px" },
    );

    var observe = function (root) {
      if (root.nodeType !== 1) return;
      if (root.hasAttribute("data-tc-visible")) io.observe(root);
      root.querySelectorAll("[data-tc-visible]").forEach(function (el) {
        io.observe(el);
      });
    };

    document.addEventListener("DOMContentLoaded", function () {
      observe(document.documentElement);
      new MutationObserver(function (mutations) {
        mutations.forEach(function (mutation) {
          mutation.addedNodes.forEach(observe);
        });
      }).observe(document.body, { childList: true, subtree: true });
    });
  }

  // ── Stale runtime endpoints ───────────────────────────────────────────────
  // Mitigation for tokio-rs/topcoat#252: shard/procedure endpoint ids are random
  // per build, so a tab that outlives a deploy calls endpoints the new binary no
  // longer serves (404/405). Reload once, throttled, and never while offline.
  // Remove once upstream ships deterministic ids.
  var RELOAD_KEY = "imk:tc-reloaded-at";
  var nativeFetch = window.fetch;

  window.fetch = function (input, init) {
    return nativeFetch.call(this, input, init).then(function (response) {
      var url = typeof input === "string" ? input : (input && input.url) || "";
      var stale =
        url.indexOf("/_topcoat/runtime/") !== -1 &&
        (response.status === 404 || response.status === 405);

      if (stale && navigator.onLine !== false) {
        var last = 0;
        try {
          last = Number(sessionStorage.getItem(RELOAD_KEY)) || 0;
        } catch (e) {}

        if (Date.now() - last > 30000) {
          try {
            sessionStorage.setItem(RELOAD_KEY, String(Date.now()));
          } catch (e) {}
          location.reload();
        }
      }

      return response;
    });
  };

  // A failed procedure call rejects without a value; tell the user something.
  window.addEventListener("unhandledrejection", function () {
    imk.toast("error", document.documentElement.dataset.genericError || "Something went wrong");
  });
})();

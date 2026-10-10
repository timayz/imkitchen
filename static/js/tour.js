/*
 * Guided spotlight tours.
 *
 * The signed-in shell (templates/partials/tour.html) emits the catalog —
 * tours, steps, anchors, copy — and the step card markup. This script asks
 * /tours where the user is, opens the tour that belongs to the page, dims
 * everything but the current anchor and posts progress back as the user
 * moves. Steps whose anchors are not on the page (hidden at this viewport,
 * no data yet) are skipped. `?tour=<id>` replays a tour from its first step.
 */
(function () {
  "use strict";

  var catalogEl = document.getElementById("tour-catalog");
  var cardTpl = document.getElementById("tour-card");
  if (!catalogEl || !cardTpl || !window.fetch) return;

  var catalog;
  try {
    catalog = JSON.parse(catalogEl.textContent);
  } catch (e) {
    return;
  }

  var PAD = 8;
  var GAP = 12;
  var MARGIN = 12;
  var MODAL = 'body > [id*="modal"]';
  var active = null; // { id, steps, pos, root, spotlight, card, prevFocus }
  var frame = 0;

  function visible(el) {
    if (!el || !el.getClientRects().length) return false;
    var style = getComputedStyle(el);
    if (style.visibility === "hidden" || style.display === "none") return false;
    var rect = el.getBoundingClientRect();
    return rect.width > 0 && rect.height > 0;
  }

  function resolve(targets) {
    for (var i = 0; i < targets.length; i++) {
      var found = document.querySelectorAll(targets[i]);
      for (var j = 0; j < found.length; j++)
        if (visible(found[j])) return found[j];
    }
    return null;
  }

  function requirementsMet(tour) {
    return (tour.requires || []).every(function (selector) {
      return !!resolve([selector]);
    });
  }

  function pagePath() {
    return location.pathname.replace(/\/+$/, "") || "/";
  }

  function post(path, step) {
    var body = step == null ? "" : "step=" + encodeURIComponent(step);
    try {
      fetch(path, {
        method: "POST",
        credentials: "same-origin",
        keepalive: true,
        headers: { "Content-Type": "application/x-www-form-urlencoded" },
        body: body,
      }).catch(function () {});
    } catch (e) {
      /* progress is best effort */
    }
  }

  /* Which tour opens on this page: the forced one, else the first catalog
     tour for this path that is pending or in progress with its requirements
     on screen. */
  function pick(progress, force) {
    var path = pagePath();
    var byId = {};
    progress.forEach(function (p) {
      byId[p.id] = p;
    });

    if (force) {
      var forced = catalog.tours[force];
      return forced && requirementsMet(forced) ? { id: force, start: 0 } : null;
    }

    var ids = Object.keys(catalog.tours);
    for (var i = 0; i < ids.length; i++) {
      var tour = catalog.tours[ids[i]];
      var p = byId[ids[i]];
      if (tour.page !== path || !p) continue;
      if (p.status !== "pending" && p.status !== "in_progress") continue;
      if (!requirementsMet(tour)) continue;
      return { id: ids[i], start: p.status === "in_progress" ? p.step : 0 };
    }
    return null;
  }

  function open(id, start) {
    var tour = catalog.tours[id];
    var steps = [];
    tour.steps.forEach(function (step, index) {
      if (resolve(step.targets)) steps.push({ index: index, def: step });
    });
    if (!steps.length) return false;

    var pos = 0;
    for (var i = 0; i < steps.length; i++) {
      if (steps[i].index >= start) {
        pos = i;
        break;
      }
    }

    var root = document.createElement("div");
    root.id = "tour-root";
    root.style.cssText = "position:fixed;inset:0;z-index:70;";

    var spotlight = document.createElement("div");
    spotlight.id = "tour-spotlight";
    spotlight.style.cssText =
      "position:absolute;border-radius:14px;pointer-events:none;" +
      "box-shadow:0 0 0 9999px rgba(27,20,12,.62);transition:all .25s ease;";
    root.appendChild(spotlight);

    var card = cardTpl.content.firstElementChild.cloneNode(true);
    card.style.cssText = "position:absolute;left:0;top:0;";
    root.appendChild(card);

    card.querySelector("[data-tour-back]").addEventListener("click", back);
    card.querySelector("[data-tour-next]").addEventListener("click", next);
    card.querySelector("[data-tour-skip]").addEventListener("click", skip);

    active = {
      id: id,
      steps: steps,
      pos: pos,
      root: root,
      spotlight: spotlight,
      card: card,
      prevFocus: document.activeElement,
    };
    document.body.appendChild(root);
    show();
    card.focus({ preventScroll: true });
    return true;
  }

  function current() {
    return active.steps[active.pos];
  }

  function show() {
    var step = current();
    var count = active.card.querySelector("[data-tour-count]");
    count.textContent =
      catalog.labels.step +
      " " +
      (active.pos + 1) +
      " " +
      catalog.labels.of +
      " " +
      active.steps.length;
    active.card.querySelector("[data-tour-title]").textContent = step.def.title;
    active.card.querySelector("[data-tour-body]").textContent = step.def.body;
    active.card.querySelector("[data-tour-back]").hidden = active.pos === 0;
    active.card.querySelector("[data-tour-next]").textContent =
      active.pos === active.steps.length - 1
        ? catalog.labels.done
        : catalog.labels.next;

    var dots = active.card.querySelector("[data-tour-dots]");
    var dot = dots.querySelector("[data-tour-dot]");
    var dotActive = dots.querySelector("[data-tour-dot-active]");
    dots.textContent = "";
    for (var i = 0; i < active.steps.length; i++) {
      var clone = (i === active.pos ? dotActive : dot).cloneNode(true);
      dots.appendChild(clone);
    }

    post("/tours/" + active.id + "/advance", step.index);

    var el = resolve(step.def.targets);
    if (el) el.scrollIntoView({ block: "center", inline: "nearest" });
    schedule();
    requestAnimationFrame(schedule);
  }

  function layout() {
    frame = 0;
    if (!active) return;

    if (!document.body.contains(active.root))
      document.body.appendChild(active.root);

    var paused = !!document.querySelector(MODAL);
    active.root.style.visibility = paused ? "hidden" : "";
    if (paused) return;

    var el = resolve(current().def.targets);
    if (!el) {
      // The anchor vanished (list swapped, viewport changed): move on.
      if (active.pos < active.steps.length - 1) {
        active.pos += 1;
        show();
      } else {
        close();
      }
      return;
    }

    var r = el.getBoundingClientRect();
    var s = active.spotlight.style;
    s.left = r.left - PAD + "px";
    s.top = r.top - PAD + "px";
    s.width = r.width + PAD * 2 + "px";
    s.height = r.height + PAD * 2 + "px";

    var vw = document.documentElement.clientWidth;
    var vh = document.documentElement.clientHeight;
    var cw = active.card.offsetWidth;
    var ch = active.card.offsetHeight;
    var left = r.left + r.width / 2 - cw / 2;
    var top;
    if (r.bottom + PAD + GAP + ch <= vh - MARGIN) {
      top = r.bottom + PAD + GAP;
    } else if (r.top - PAD - GAP - ch >= MARGIN) {
      top = r.top - PAD - GAP - ch;
    } else if (r.right + PAD + GAP + cw <= vw - MARGIN) {
      top = Math.max(MARGIN, Math.min(r.top, vh - ch - MARGIN));
      left = r.right + PAD + GAP;
    } else if (r.left - PAD - GAP - cw >= MARGIN) {
      top = Math.max(MARGIN, Math.min(r.top, vh - ch - MARGIN));
      left = r.left - PAD - GAP - cw;
    } else {
      top = vh - ch - MARGIN;
    }
    left = Math.max(MARGIN, Math.min(left, vw - cw - MARGIN));
    active.card.style.left = left + "px";
    active.card.style.top = top + "px";
  }

  function schedule() {
    if (!frame) frame = requestAnimationFrame(layout);
  }

  function next() {
    if (!active) return;
    if (active.pos >= active.steps.length - 1) {
      post("/tours/" + active.id + "/complete");
      close();
      return;
    }
    active.pos += 1;
    show();
  }

  function back() {
    if (!active || active.pos === 0) return;
    active.pos -= 1;
    show();
  }

  function skip() {
    if (!active) return;
    post("/tours/" + active.id + "/skip", current().index);
    close();
  }

  function close() {
    if (!active) return;
    var prev = active.prevFocus;
    active.root.remove();
    active = null;
    if (prev && prev.focus) {
      try {
        prev.focus({ preventScroll: true });
      } catch (e) {
        /* the element may be gone */
      }
    }
    if (location.search.indexOf("tour=") !== -1) {
      var url = new URL(location.href);
      url.searchParams.delete("tour");
      history.replaceState(history.state, "", url);
    }
  }

  function onKey(event) {
    if (!active || active.root.style.visibility === "hidden") return;
    if (event.key === "Escape") {
      event.preventDefault();
      skip();
    } else if (event.key === "ArrowRight" || event.key === "Enter") {
      if (
        event.target &&
        event.target.closest &&
        event.target.closest("[data-tour-back],[data-tour-skip]")
      )
        return;
      event.preventDefault();
      next();
    } else if (event.key === "ArrowLeft") {
      event.preventDefault();
      back();
    }
  }

  function boot() {
    if (active) return;
    var force = new URLSearchParams(location.search).get("tour");
    fetch("/tours", {
      credentials: "same-origin",
      headers: { Accept: "application/json" },
    })
      .then(function (res) {
        if (
          !res.ok ||
          (res.headers.get("content-type") || "").indexOf("json") === -1
        )
          return null;
        return res.json();
      })
      .then(function (state) {
        if (!state || active) return;
        var choice = pick(state.tours || [], force);
        if (choice) open(choice.id, choice.start);
      })
      .catch(function () {});
  }

  window.addEventListener("resize", schedule);
  window.addEventListener("scroll", schedule, true);
  document.addEventListener("keydown", onKey);
  new MutationObserver(schedule).observe(document.documentElement, {
    childList: true,
    subtree: true,
  });
  window.addEventListener("popstate", boot);

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", boot);
  } else {
    boot();
  }
})();

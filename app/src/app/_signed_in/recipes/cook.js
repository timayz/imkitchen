// The cooking screen (`_cook.rs`): one step at a time, the screen kept
// awake, and the timers of the steps.
//
// The step being read and the timers that were started are kept in this
// browser, under the id of the recipe, until the cooking is finished or
// stopped. Every sentence shown is the page's: nothing is worded here.
(() => {
  const root = document.querySelector("[data-cook]");
  if (!root) return;

  const key = `imkitchen:cook:${root.dataset.cook}`;
  const steps = [...root.querySelectorAll("[data-cook-step]")];
  const controls = root.querySelector("[data-cook-controls]");
  const previous = root.querySelector("[data-cook-previous]");
  const next = root.querySelector("[data-cook-next]");
  const finish = root.querySelector("[data-cook-finish]");
  const timers = root.querySelector("[data-cook-timers]");
  const row = root.querySelector("[data-cook-timer-row]");
  const last = steps.length - 1;

  // What is kept: the step being read, and when each timer that was
  // started ends (`at`, on the clock of this device) by the place of its
  // step. `rung` once it was said to be over.
  const state = { step: 0, timers: {} };
  try {
    const kept = JSON.parse(localStorage.getItem(key) || "null");
    if (kept && Number.isInteger(kept.step)) state.step = kept.step;
    if (kept && kept.timers && typeof kept.timers === "object") {
      for (const [place, timer] of Object.entries(kept.timers)) {
        if (timer && Number.isFinite(timer.at)) {
          state.timers[place] = { at: timer.at, rung: timer.rung === true };
        }
      }
    }
  } catch {
    // Nothing is kept where the browser keeps nothing.
  }
  // The recipe may have been rewritten since.
  state.step = Math.min(Math.max(state.step, 0), Math.max(last, 0));
  for (const place of Object.keys(state.timers)) {
    if (!steps[place]) delete state.timers[place];
  }
  // A timer that ended while the page was closed is over, and rings no
  // more.
  for (const timer of Object.values(state.timers)) {
    if (timer.at <= Date.now()) timer.rung = true;
  }

  const keep = () => {
    try {
      localStorage.setItem(key, JSON.stringify(state));
    } catch {
      // The cooking goes on, kept by the page alone.
    }
  };
  const forget = () => {
    try {
      localStorage.removeItem(key);
    } catch {
      // Nothing was kept.
    }
  };

  const show = (step) => {
    state.step = Math.min(Math.max(step, 0), Math.max(last, 0));
    steps.forEach((li, place) => {
      li.hidden = place !== state.step;
    });
    previous.disabled = state.step === 0;
    next.hidden = state.step >= last;
    finish.hidden = state.step < last;
    root.dataset.cookAt = String(state.step);
    keep();
  };

  // The sound of a timer that is over: three notes, where the browser lets
  // a page be heard (after who cooks touched it).
  let audio = null;
  const hear = () => {
    const Audio = window.AudioContext || window.webkitAudioContext;
    if (!Audio) return;
    try {
      audio = audio || new Audio();
      if (audio.state === "suspended") audio.resume();
    } catch {
      audio = null;
    }
  };
  const ring = (step) => {
    if (audio && audio.state === "running") {
      for (const nth of [0, 1, 2]) {
        const note = audio.createOscillator();
        const volume = audio.createGain();
        const from = audio.currentTime + nth * 0.5;
        note.frequency.value = 880;
        volume.gain.setValueAtTime(0.0001, from);
        volume.gain.exponentialRampToValueAtTime(0.4, from + 0.02);
        volume.gain.exponentialRampToValueAtTime(0.0001, from + 0.35);
        note.connect(volume).connect(audio.destination);
        note.start(from);
        note.stop(from + 0.4);
      }
    }
    if ("Notification" in window && Notification.permission === "granted") {
      try {
        new Notification(root.dataset.title, {
          body: step.dataset.over,
          tag: `${key}:${step.dataset.cookStep}`,
        });
      } catch {
        // The page says it.
      }
    }
  };

  const left = (seconds) => {
    const two = (n) => String(n).padStart(2, "0");
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    const rest = `${two(minutes)}:${two(seconds % 60)}`;
    return hours > 0 ? `${hours}:${rest}` : rest;
  };

  // The rows of the timers, by the place of their step.
  const rows = new Map();
  const tick = () => {
    const now = Date.now();
    for (const [place, shown] of rows) {
      if (!state.timers[place]) {
        shown.remove();
        rows.delete(place);
      }
    }
    for (const [place, timer] of Object.entries(state.timers)) {
      const step = steps[place];
      let shown = rows.get(place);
      if (!shown) {
        shown = row.content.firstElementChild.cloneNode(true);
        shown.dataset.cookRunning = place;
        const label = shown.querySelector("[data-cook-show]");
        label.textContent = step.dataset.label;
        label.addEventListener("click", () => show(Number(place)));
        shown.querySelector("[data-cook-cancel]").addEventListener("click", () => {
          delete state.timers[place];
          keep();
          tick();
        });
        timers.append(shown);
        rows.set(place, shown);
      }
      const seconds = Math.max(Math.ceil((timer.at - now) / 1000), 0);
      const over = seconds === 0;
      shown.querySelector("[data-cook-left]").textContent = left(seconds);
      shown.querySelector("[data-cook-left]").hidden = over;
      shown.querySelector("[data-cook-over]").hidden = !over;
      shown.dataset.state = over ? "over" : "running";
      if (over && !timer.rung) {
        timer.rung = true;
        keep();
        ring(step);
      }
    }
    // A step whose timer runs is not started twice.
    for (const [place, step] of steps.entries()) {
      const start = step.querySelector("[data-cook-start]");
      if (start) start.disabled = Boolean(state.timers[place]);
    }
  };

  steps.forEach((step, place) => {
    const start = step.querySelector("[data-cook-start]");
    const minutes = Number(step.dataset.minutes);
    if (!start || !(minutes > 0)) return;
    start.hidden = false;
    start.addEventListener("click", () => {
      state.timers[place] = { at: Date.now() + minutes * 60000, rung: false };
      keep();
      hear();
      if ("Notification" in window && Notification.permission === "default") {
        try {
          Notification.requestPermission();
        } catch {
          // The page says when it is over.
        }
      }
      tick();
    });
  });

  previous.addEventListener("click", () => show(state.step - 1));
  next.addEventListener("click", () => show(state.step + 1));
  document.addEventListener("keydown", (event) => {
    if (event.target instanceof HTMLElement && event.target.closest("input, textarea, select")) {
      return;
    }
    if (event.key === "ArrowRight") show(state.step + 1);
    if (event.key === "ArrowLeft") show(state.step - 1);
  });
  // Finished, or stopped: nothing of the cooking is kept.
  for (const way of root.querySelectorAll("[data-cook-finish], [data-cook-leave]")) {
    way.addEventListener("click", forget);
  }

  // The screen stays awake while the page is shown, where the browser has
  // the wake lock: it gives it back when the page is hidden.
  const asleep = root.querySelector("[data-cook-asleep]");
  const awake = async () => {
    if (!("wakeLock" in navigator)) {
      asleep.hidden = false;
      return;
    }
    try {
      const lock = await navigator.wakeLock.request("screen");
      root.dataset.cookAwake = "on";
      asleep.hidden = true;
      lock.addEventListener("release", () => {
        root.dataset.cookAwake = "off";
      });
    } catch {
      asleep.hidden = false;
    }
  };
  document.addEventListener("visibilitychange", () => {
    if (document.visibilityState === "visible") awake();
  });

  if (steps.length > 0) controls.hidden = false;
  show(state.step);
  tick();
  setInterval(tick, 250);
  awake();
})();

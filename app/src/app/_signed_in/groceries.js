// The groceries (`groceries.rs`): an item is checked, and unchecked, without
// the page being read again.
//
// Each item is a form that posts to its page. Here the item is shown as it
// was just said at once, and its form sent behind the page: one that was
// not done is shown as it was again, and the page says so. Every sentence
// shown is the page's: nothing is worded here.
(() => {
  const root = document.querySelector("[data-groceries]");
  if (!root) return;

  const count = root.querySelector("[data-groceries-checked]");
  const failed = root.querySelector("[data-groceries-failed]");

  const show = (item) => {
    item.li.dataset.checked = String(item.wanted);
    item.button.setAttribute("aria-checked", String(item.wanted));
    // What the form does next, were it posted without this script.
    item.form.action = item.wanted ? item.form.dataset.uncheck : item.form.dataset.check;
    if (count) {
      count.textContent = String(root.querySelectorAll('[data-item][data-checked="true"]').length);
    }
  };

  // Says `checked` of `item` to its page, and answers whether it was done.
  // The page answers by sending the browser back to the groceries: anywhere
  // else is for the browser itself to go.
  const say = async (item, checked) => {
    const action = checked ? item.form.dataset.check : item.form.dataset.uncheck;
    try {
      const response = await fetch(action, {
        method: "POST",
        credentials: "same-origin",
        // Sent to its end, were the page left meanwhile.
        keepalive: true,
      });
      const at = new URL(response.url);
      if (at.pathname !== location.pathname) {
        // The session ended.
        location.reload();
        return true;
      }
      if (response.ok && at.search) {
        // The list changed: its groceries as they now stand.
        location.replace(at.href);
        return true;
      }
      return response.ok;
    } catch {
      return false;
    }
  };

  // Sends what is wanted of `item` until its page knows it: one command at
  // a time, and none for what was said and unsaid meanwhile.
  const settle = async (item) => {
    if (item.busy) return;
    item.busy = true;
    while (item.said !== item.wanted) {
      const wanted = item.wanted;
      if (await say(item, wanted)) {
        item.said = wanted;
        failed.hidden = true;
      } else {
        item.wanted = item.said;
        show(item);
        failed.hidden = false;
      }
    }
    item.busy = false;
  };

  for (const li of root.querySelectorAll("[data-item]")) {
    const form = li.querySelector("form");
    const button = form.querySelector("[role=checkbox]");
    const checked = li.dataset.checked === "true";
    const item = { li, form, button, said: checked, wanted: checked, busy: false };
    form.addEventListener("submit", (event) => {
      event.preventDefault();
      item.wanted = !item.wanted;
      show(item);
      settle(item);
    });
  }
})();

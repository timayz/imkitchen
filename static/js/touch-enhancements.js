/**
 * Touch Enhancement Module (Story 5.5)
 * Provides haptic feedback, long-press menus, and touch gesture support
 */

// Haptic Feedback Module
const Haptic = {
  // Check if Vibration API is supported
  isSupported: () => 'vibrate' in navigator,

  // User preference for haptic feedback (default: enabled)
  enabled: true,

  // Load user preference from localStorage
  loadPreference() {
    const pref = localStorage.getItem('haptic_enabled');
    this.enabled = pref === null ? true : pref === 'true';
  },

  // Save user preference to localStorage
  savePreference(enabled) {
    this.enabled = enabled;
    localStorage.setItem('haptic_enabled', enabled.toString());
  },

  // Trigger haptic feedback (50ms subtle vibration)
  tap() {
    if (this.enabled && this.isSupported()) {
      navigator.vibrate(50);
    }
  },

  // Trigger haptic feedback for critical actions (100ms)
  action() {
    if (this.enabled && this.isSupported()) {
      navigator.vibrate(100);
    }
  },
};

// Long-Press Detection Module
const LongPress = {
  threshold: 300, // 300ms threshold for long-press
  timers: new Map(),

  // Attach long-press handler to element
  attach(element, callback) {
    let timer = null;
    let touchStartPos = null;

    const handleStart = (e) => {
      const touch = e.touches ? e.touches[0] : e;
      touchStartPos = { x: touch.clientX, y: touch.clientY };

      timer = setTimeout(() => {
        Haptic.action(); // Haptic feedback on long-press
        callback(element, e);
        timer = null;
      }, this.threshold);
    };

    const handleMove = (e) => {
      if (timer && touchStartPos) {
        const touch = e.touches ? e.touches[0] : e;
        const dx = Math.abs(touch.clientX - touchStartPos.x);
        const dy = Math.abs(touch.clientY - touchStartPos.y);

        // Cancel long-press if user moves finger > 10px (likely scrolling)
        if (dx > 10 || dy > 10) {
          clearTimeout(timer);
          timer = null;
        }
      }
    };

    const handleEnd = () => {
      if (timer) {
        clearTimeout(timer);
        timer = null;
      }
      touchStartPos = null;
    };

    element.addEventListener('touchstart', handleStart, { passive: true });
    element.addEventListener('touchmove', handleMove, { passive: true });
    element.addEventListener('touchend', handleEnd);
    element.addEventListener('touchcancel', handleEnd);

    // Store cleanup function
    this.timers.set(element, () => {
      element.removeEventListener('touchstart', handleStart);
      element.removeEventListener('touchmove', handleMove);
      element.removeEventListener('touchend', handleEnd);
      element.removeEventListener('touchcancel', handleEnd);
    });
  },

  // Detach long-press handler from element
  detach(element) {
    const cleanup = this.timers.get(element);
    if (cleanup) {
      cleanup();
      this.timers.delete(element);
    }
  },
};

// Contextual Menu Module
const ContextMenu = {
  activeMenu: null,

  // Show contextual menu at position
  show(menu, x, y) {
    this.hide(); // Hide any existing menu

    menu.style.position = 'fixed';
    menu.style.left = `${x}px`;
    menu.style.top = `${y}px`;
    menu.style.display = 'block';
    menu.classList.remove('hidden');
    this.activeMenu = menu;

    // Close menu on tap outside or scroll
    const handleOutsideClick = (e) => {
      if (!menu.contains(e.target)) {
        this.hide();
      }
    };

    const handleScroll = () => {
      this.hide();
    };

    setTimeout(() => {
      document.addEventListener('click', handleOutsideClick, { once: true });
      document.addEventListener('touchstart', handleOutsideClick, { once: true, passive: true });
      document.addEventListener('scroll', handleScroll, { once: true, passive: true });
    }, 100);
  },

  // Hide active contextual menu
  hide() {
    if (this.activeMenu) {
      this.activeMenu.style.display = 'none';
      this.activeMenu.classList.add('hidden');
      this.activeMenu = null;
    }
  },
};

// Initialize touch enhancements on page load
document.addEventListener('DOMContentLoaded', () => {
  // Load haptic preference
  Haptic.loadPreference();

  // Add haptic feedback to all buttons
  document.querySelectorAll('button, a[role="button"], .btn-primary, .btn-secondary').forEach((btn) => {
    btn.addEventListener('click', () => Haptic.tap(), { passive: true });
  });

  // Add haptic feedback to form submissions
  document.querySelectorAll('form').forEach((form) => {
    form.addEventListener('submit', () => Haptic.action(), { passive: true });
  });

  // Add haptic feedback to checkbox/radio changes
  document.querySelectorAll('input[type="checkbox"], input[type="radio"]').forEach((input) => {
    input.addEventListener('change', () => Haptic.tap(), { passive: true });
  });

  // Setup long-press menus on recipe cards (if present)
  const recipeCards = document.querySelectorAll('.recipe-card');
  if (recipeCards.length > 0) {
    recipeCards.forEach((card) => {
      const recipeId = card.dataset.recipeId || card.querySelector('a[href^="/recipes/"]')?.href.split('/').pop();
      if (!recipeId) return;

    // Create contextual menu (hidden by default)
    const menu = document.createElement('div');
    menu.className = 'hidden bg-white shadow-lg rounded-lg border border-gray-200 z-50';
    menu.innerHTML = `
      <div class="py-2 min-w-[160px]">
        <a href="/recipes/${recipeId}" class="block px-4 py-2 text-gray-700 hover:bg-gray-100">
          View Details
        </a>
        <a href="/recipes/${recipeId}/edit" class="block px-4 py-2 text-gray-700 hover:bg-gray-100">
          Edit Recipe
        </a>
        <button onclick="if(confirm('Delete this recipe?')) location.href='/recipes/${recipeId}/delete'"
                class="block w-full text-left px-4 py-2 text-red-600 hover:bg-gray-100">
          Delete
        </button>
      </div>
    `;
    document.body.appendChild(menu);

      // Attach long-press handler
      LongPress.attach(card, (element, e) => {
        e.preventDefault();
        const touch = e.touches ? e.touches[0] : e;
        ContextMenu.show(menu, touch.clientX, touch.clientY);
      });
    });
  }

  // Setup passive scroll event listeners for performance (Story 5.5 - Task 7)
  document.querySelectorAll('[data-scroll-handler]').forEach((el) => {
    el.addEventListener('scroll', (e) => {
      // Custom scroll handler logic here
    }, { passive: true });
  });
});

// Export for use in other modules
if (typeof module !== 'undefined' && module.exports) {
  module.exports = { Haptic, LongPress, ContextMenu };
}

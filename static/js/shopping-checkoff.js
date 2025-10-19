/**
 * Shopping list checkoff persistence for offline support
 * Story 5.3 AC-5, AC-6 - Offline shopping list with checkoff functionality
 *
 * Uses LocalStorage for immediate checkbox persistence
 * Queues mutations in IndexedDB for background sync
 */

// CSS Selectors
const SELECTORS = {
  CHECKBOX: '.shopping-item-checkbox',
  ITEM_ROW: '.shopping-item'
};

/**
 * Initialize shopping list checkoff handlers
 */
function initShoppingCheckoff() {
  // Restore checkbox states from LocalStorage on page load
  restoreCheckboxStates();

  // Add event listeners to all shopping list checkboxes
  const checkboxes = document.querySelectorAll(SELECTORS.CHECKBOX);
  checkboxes.forEach(checkbox => {
    checkbox.addEventListener('change', handleCheckboxChange);
  });

  console.log('Shopping checkoff initialized');
}

/**
 * Restore checkbox states from LocalStorage
 */
function restoreCheckboxStates() {
  const checkboxes = document.querySelectorAll(SELECTORS.CHECKBOX);

  checkboxes.forEach(checkbox => {
    const itemId = checkbox.dataset.itemId;
    const storageKey = `shopping-checkoff-${itemId}`;
    const isChecked = localStorage.getItem(storageKey) === 'true';

    if (isChecked !== checkbox.checked) {
      checkbox.checked = isChecked;
      updateCheckboxUI(checkbox, isChecked);
    }
  });

  console.log('Checkbox states restored from LocalStorage');
}

/**
 * Handle checkbox change event
 * @param {Event} event
 */
async function handleCheckboxChange(event) {
  const checkbox = event.target;
  const itemId = checkbox.dataset.itemId;
  const isChecked = checkbox.checked;

  // Persist to LocalStorage immediately (optimistic UI)
  const storageKey = `shopping-checkoff-${itemId}`;
  localStorage.setItem(storageKey, isChecked.toString());

  // Update UI
  updateCheckboxUI(checkbox, isChecked);

  console.debug(`Shopping item ${itemId} ${isChecked ? 'checked' : 'unchecked'}`);

  // Try to sync with server
  try {
    await syncCheckoffToServer(itemId, isChecked);
  } catch (error) {
    console.warn('Failed to sync checkoff (offline), queuing for background sync:', error.message);

    // Queue for background sync if online sync fails
    await queueCheckoffMutation(itemId, isChecked);
  }
}

/**
 * Update checkbox UI styling
 * @param {HTMLInputElement} checkbox
 * @param {boolean} isChecked
 */
function updateCheckboxUI(checkbox, isChecked) {
  const itemRow = checkbox.closest(SELECTORS.ITEM_ROW);
  if (itemRow) {
    if (isChecked) {
      itemRow.classList.add('checked');
      itemRow.classList.add('opacity-50');
    } else {
      itemRow.classList.remove('checked');
      itemRow.classList.remove('opacity-50');
    }
  }
}

/**
 * Sync checkoff state to server
 * @param {string} itemId
 * @param {boolean} isChecked
 * @returns {Promise<Response>}
 */
async function syncCheckoffToServer(itemId, isChecked) {
  const url = `/shopping/item/${itemId}/checkoff`;
  const method = 'POST';

  const response = await fetch(url, {
    method,
    headers: {
      'Content-Type': 'application/json',
      'Accept': 'application/json'
    },
    body: JSON.stringify({ is_collected: isChecked })
  });

  if (!response.ok) {
    throw new Error(`Server returned ${response.status}`);
  }

  console.debug('Checkoff synced to server:', itemId, isChecked);
  return response;
}

/**
 * Queue checkoff mutation for background sync
 * @param {string} itemId
 * @param {boolean} isChecked
 */
async function queueCheckoffMutation(itemId, isChecked) {
  try {
    const requestData = {
      url: `/shopping/item/${itemId}/checkoff`,
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Accept': 'application/json'
      },
      body: { is_collected: isChecked }
    };

    // Use offlineDB to queue the request
    if (window.offlineDB) {
      await window.offlineDB.queueRequest(requestData);
      console.debug('Checkoff mutation queued for sync:', itemId);

      // Register sync event with service worker
      if ('serviceWorker' in navigator && 'sync' in ServiceWorkerRegistration.prototype) {
        const registration = await navigator.serviceWorker.ready;
        await registration.sync.register('sync-offline-actions');
        console.debug('Background sync registered');
      }
    } else {
      console.error('offlineDB not available, mutation not queued');
    }
  } catch (error) {
    console.error('Failed to queue checkoff mutation:', error);
  }
}

/**
 * Clear all checkoff states from LocalStorage
 * Used when starting a new shopping week
 */
function clearCheckoffStates() {
  const keys = Object.keys(localStorage);
  const checkoffKeys = keys.filter(key => key.startsWith('shopping-checkoff-'));

  checkoffKeys.forEach(key => {
    localStorage.removeItem(key);
  });

  console.debug(`Cleared ${checkoffKeys.length} checkoff states from LocalStorage`);
}

/**
 * Initialize on DOM ready
 */
if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', initShoppingCheckoff);
} else {
  initShoppingCheckoff();
}

// Export for global access if needed
if (typeof window !== 'undefined') {
  window.shoppingCheckoff = {
    initShoppingCheckoff,
    clearCheckoffStates
  };
}

/**
 * Sync UI - Story 5.8: Real-Time Sync When Connectivity Restored
 *
 * Handles:
 * - iOS Safari detection and fallback (Subtask 1.5)
 * - Sync progress indicator display (Task 4)
 * - Success/failure toast notifications
 * - Manual sync button for iOS
 */

// ============================================================
// iOS Safari Detection - Story 5.8 Subtask 1.5
// ============================================================

/**
 * Detect if running on iOS Safari (which doesn't support Background Sync API)
 * @returns {boolean}
 */
function isIOSSafari() {
  const ua = navigator.userAgent;
  const iOS = /iPad|iPhone|iPod/.test(ua) && !window.MSStream;

  // Check if Safari (not Chrome on iOS)
  const safari = /^((?!chrome|android).)*safari/i.test(ua);

  return iOS && safari;
}

/**
 * Check if Background Sync API is supported
 * @returns {boolean}
 */
function isBackgroundSyncSupported() {
  return 'serviceWorker' in navigator &&
         'SyncManager' in window;
}

/**
 * Initialize iOS fallback if needed
 */
function initializeIOSFallback() {
  if (isIOSSafari() && !isBackgroundSyncSupported()) {
    console.log('iOS Safari detected - Background Sync API not supported, showing fallback');

    // Show warning message
    showIOSWarning();

    // Show manual sync button
    showManualSyncButton();
  }
}

/**
 * Show warning for iOS users about Background Sync limitation
 */
function showIOSWarning() {
  const warningId = 'ios-sync-warning';

  // Don't show if already dismissed
  if (localStorage.getItem(warningId + '-dismissed') === 'true') {
    return;
  }

  // Don't duplicate warning
  if (document.getElementById(warningId)) {
    return;
  }

  const warning = document.createElement('div');
  warning.id = warningId;
  warning.className = 'fixed top-4 right-4 bg-yellow-50 border border-yellow-200 text-yellow-900 p-4 rounded-lg shadow-lg max-w-sm z-50';
  warning.innerHTML = `
    <div class="flex items-start">
      <svg class="w-5 h-5 mr-2 text-yellow-600 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"></path>
      </svg>
      <div class="flex-1">
        <h3 class="font-medium text-sm">Background sync not available on iOS</h3>
        <p class="text-xs mt-1">Stay online when submitting changes, or use the manual sync button if you go offline.</p>
        <button onclick="dismissIOSWarning()" class="mt-2 text-xs font-medium text-yellow-700 underline">Dismiss</button>
      </div>
    </div>
  `;

  document.body.appendChild(warning);

  // Auto-dismiss after 15 seconds
  setTimeout(() => {
    dismissIOSWarning();
  }, 15000);
}

/**
 * Dismiss iOS warning and remember preference
 */
function dismissIOSWarning() {
  const warning = document.getElementById('ios-sync-warning');
  if (warning) {
    warning.remove();
    localStorage.setItem('ios-sync-warning-dismissed', 'true');
  }
}

/**
 * Show manual sync button for iOS users
 */
function showManualSyncButton() {
  const buttonId = 'manual-sync-button';

  // Don't duplicate button
  if (document.getElementById(buttonId)) {
    return;
  }

  // Find a good location to insert the button (e.g., in nav or top bar)
  // For now, create a floating button
  const button = document.createElement('button');
  button.id = buttonId;
  button.className = 'fixed bottom-20 right-4 bg-blue-500 hover:bg-blue-600 text-white p-3 rounded-full shadow-lg z-40 transition-transform transform hover:scale-105';
  button.setAttribute('aria-label', 'Sync Now');
  button.innerHTML = `
    <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"></path>
    </svg>
  `;
  button.onclick = triggerManualSync;

  document.body.appendChild(button);

  // Add tooltip
  button.title = 'Sync offline changes now';
}

/**
 * Manually trigger sync for iOS users
 */
async function triggerManualSync() {
  console.log('Manual sync triggered');

  const button = document.getElementById('manual-sync-button');
  if (button) {
    // Disable button and show loading state
    button.disabled = true;
    button.innerHTML = `
      <svg class="w-6 h-6 animate-spin" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"></path>
      </svg>
    `;
  }

  try {
    // Send message to service worker to trigger sync
    if ('serviceWorker' in navigator && navigator.serviceWorker.controller) {
      navigator.serviceWorker.controller.postMessage({
        type: 'MANUAL_SYNC'
      });

      // Show toast
      showToast('Syncing changes...', 'info');
    } else {
      showToast('Service worker not available. Please reload the page.', 'error');
    }
  } catch (error) {
    console.error('Manual sync failed:', error);
    showToast('Sync failed. Please try again.', 'error');
  } finally {
    // Re-enable button
    if (button) {
      button.disabled = false;
      button.innerHTML = `
        <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"></path>
        </svg>
      `;
    }
  }
}

// ============================================================
// Sync Progress UI - Story 5.8 Task 4
// ============================================================

/**
 * Show sync progress indicator
 * @param {number} current - Current request being synced
 * @param {number} total - Total requests to sync
 */
function showSyncProgress(current, total) {
  const toastId = 'sync-progress-toast';
  let toast = document.getElementById(toastId);

  if (!toast) {
    toast = document.createElement('div');
    toast.id = toastId;
    toast.className = 'fixed bottom-4 right-4 bg-blue-50 border border-blue-200 text-blue-900 p-4 rounded-lg shadow-lg max-w-sm z-50 transition-opacity';
    document.body.appendChild(toast);
  }

  // Update content - Story 5.8 Subtask 4.1
  toast.innerHTML = `
    <div class="flex items-center">
      <svg class="w-5 h-5 mr-2 text-blue-600 animate-spin" fill="none" viewBox="0 0 24 24">
        <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
        <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
      </svg>
      <div class="flex-1">
        <p class="font-medium text-sm">Syncing changes... (${current} of ${total})</p>
        <div class="w-full bg-blue-200 rounded-full h-1.5 mt-2">
          <div class="bg-blue-600 h-1.5 rounded-full transition-all" style="width: ${(current / total) * 100}%"></div>
        </div>
      </div>
    </div>
  `;

  toast.style.opacity = '1';
}

/**
 * Hide sync progress indicator
 */
function hideSyncProgress() {
  const toast = document.getElementById('sync-progress-toast');
  if (toast) {
    toast.style.opacity = '0';
    setTimeout(() => {
      toast.remove();
    }, 300);
  }
}

/**
 * Show generic toast notification
 * @param {string} message - Toast message
 * @param {string} type - Toast type: 'success', 'error', 'info', 'warning'
 * @param {number} duration - Auto-dismiss duration in ms (0 = no auto-dismiss)
 */
function showToast(message, type = 'info', duration = 3000) {
  const toastId = `toast-${Date.now()}`;
  const toast = document.createElement('div');
  toast.id = toastId;

  // Style based on type
  const styles = {
    success: 'bg-green-50 border-green-200 text-green-900',
    error: 'bg-red-50 border-red-200 text-red-900',
    info: 'bg-blue-50 border-blue-200 text-blue-900',
    warning: 'bg-yellow-50 border-yellow-200 text-yellow-900'
  };

  const icons = {
    success: `<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"></path>`,
    error: `<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z"></path>`,
    info: `<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path>`,
    warning: `<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"></path>`
  };

  toast.className = `fixed bottom-4 right-4 ${styles[type]} border p-4 rounded-lg shadow-lg max-w-sm z-50 transition-opacity`;
  toast.style.opacity = '0';

  toast.innerHTML = `
    <div class="flex items-start">
      <svg class="w-5 h-5 mr-2 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        ${icons[type]}
      </svg>
      <div class="flex-1">
        <p class="text-sm">${message}</p>
      </div>
      <button onclick="this.closest('.fixed').remove()" class="ml-2 text-gray-400 hover:text-gray-600">
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
        </svg>
      </button>
    </div>
  `;

  document.body.appendChild(toast);

  // Fade in
  setTimeout(() => {
    toast.style.opacity = '1';
  }, 10);

  // Auto-dismiss if duration > 0
  if (duration > 0) {
    setTimeout(() => {
      toast.style.opacity = '0';
      setTimeout(() => {
        toast.remove();
      }, 300);
    }, duration);
  }
}

// ============================================================
// Service Worker Message Listener - Story 5.8 Subtask 4.2, 4.3
// ============================================================

/**
 * Listen for messages from service worker
 */
function initializeServiceWorkerMessageListener() {
  if ('serviceWorker' in navigator) {
    navigator.serviceWorker.addEventListener('message', (event) => {
      const { type, data } = event.data || {};

      console.log('Message from service worker:', type, data);

      switch (type) {
        case 'SYNC_PROGRESS':
          // Story 5.8 Subtask 4.3 - Update UI with progress
          showSyncProgress(data.current, data.total);
          break;

        case 'SYNC_COMPLETE':
          // Story 5.8 Subtask 4.4 - Show success toast
          hideSyncProgress();
          showToast('Your changes have been synced!', 'success', 3000);
          break;

        case 'SYNC_ERROR':
          // Show error toast
          hideSyncProgress();
          showToast(data.message || 'Sync failed. Will retry automatically.', 'error', 5000);
          break;

        case 'SYNC_CONFLICT':
          // Story 5.8 Subtask 3.2 - Notify user of conflict
          hideSyncProgress();
          showToast(`Conflict detected for ${data.itemName}. Server version restored. Please review changes.`, 'warning', 8000);
          break;

        default:
          console.log('Unknown message type from service worker:', type);
      }
    });

    console.log('Service worker message listener initialized');
  }
}

// ============================================================
// Initialization
// ============================================================

// Initialize on page load
if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', () => {
    initializeIOSFallback();
    initializeServiceWorkerMessageListener();
  });
} else {
  initializeIOSFallback();
  initializeServiceWorkerMessageListener();
}

// Export functions for global access
if (typeof window !== 'undefined') {
  window.syncUI = {
    isIOSSafari,
    isBackgroundSyncSupported,
    showSyncProgress,
    hideSyncProgress,
    showToast,
    triggerManualSync,
    dismissIOSWarning
  };
}

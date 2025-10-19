// Offline Indicator - Story 5.2
// Displays a banner when the user goes offline and dismisses it when back online

(function() {
    'use strict';

    let offlineBanner = null;

    // Listen for offline event
    window.addEventListener('offline', function() {
        console.log('Connection lost - showing offline indicator');
        showOfflineBanner();
    });

    // Listen for online event
    window.addEventListener('online', function() {
        console.log('Connection restored - dismissing offline indicator');
        showOnlineBanner();

        // Auto-dismiss online banner after 3 seconds
        setTimeout(function() {
            dismissBanner();
        }, 3000);
    });

    /**
     * Show offline banner at top of page
     */
    function showOfflineBanner() {
        // Remove existing banner if present
        dismissBanner();

        // Create offline banner
        offlineBanner = document.createElement('div');
        offlineBanner.id = 'offline-indicator';
        offlineBanner.className = 'fixed top-0 left-0 right-0 bg-gray-600 text-white p-3 flex items-center justify-center z-50 shadow-lg';
        offlineBanner.innerHTML = `
            <svg class="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M18.364 5.636a9 9 0 010 12.728m0 0l-2.829-2.829m2.829 2.829L21 21M15.536 8.464a5 5 0 010 7.072m0 0l-2.829-2.829m-4.243 2.829a4.978 4.978 0 01-1.414-2.83m-1.414 5.658a9 9 0 01-2.167-9.238m7.824 2.167a1 1 0 111.414 1.414m-1.414-1.414L3 3m8.293 8.293l1.414 1.414"></path>
            </svg>
            <span class="font-medium">You're offline. Some features may be unavailable.</span>
        `;

        document.body.insertBefore(offlineBanner, document.body.firstChild);

        // Push content down to avoid overlapping
        pushContentDown(true);
    }

    /**
     * Show online banner (connection restored)
     */
    function showOnlineBanner() {
        // Remove existing banner if present
        dismissBanner();

        // Create online banner
        offlineBanner = document.createElement('div');
        offlineBanner.id = 'offline-indicator';
        offlineBanner.className = 'fixed top-0 left-0 right-0 bg-green-600 text-white p-3 flex items-center justify-center z-50 shadow-lg';
        offlineBanner.innerHTML = `
            <svg class="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7"></path>
            </svg>
            <span class="font-medium">You're back online!</span>
        `;

        document.body.insertBefore(offlineBanner, document.body.firstChild);

        // Push content down to avoid overlapping
        pushContentDown(true);
    }

    /**
     * Dismiss banner
     */
    function dismissBanner() {
        if (offlineBanner && offlineBanner.parentNode) {
            offlineBanner.parentNode.removeChild(offlineBanner);
            offlineBanner = null;

            // Restore content position
            pushContentDown(false);
        }
    }

    /**
     * Push page content down when banner is shown
     * @param {boolean} push - true to push down, false to restore
     */
    function pushContentDown(push) {
        const mainContent = document.getElementById('main-content');
        if (mainContent) {
            if (push) {
                mainContent.style.paddingTop = '3rem';
            } else {
                mainContent.style.paddingTop = '';
            }
        }
    }

    // Check initial connection status on load
    if (!navigator.onLine) {
        showOfflineBanner();
    }
})();

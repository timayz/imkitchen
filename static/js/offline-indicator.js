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
     * Story 5.3 AC-8, AC-9 - Neutral styling with reassuring messaging
     */
    function showOfflineBanner() {
        // Remove existing banner if present
        dismissBanner();

        // Create offline banner with neutral styling (AC-9: doesn't alarm user)
        offlineBanner = document.createElement('div');
        offlineBanner.id = 'offline-indicator';
        offlineBanner.className = 'fixed top-0 left-0 right-0 bg-blue-50 text-blue-900 border-b border-blue-200 p-3 flex items-center justify-center z-50 shadow-sm';
        offlineBanner.innerHTML = `
            <svg class="w-5 h-5 mr-2 text-blue-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path>
            </svg>
            <span class="font-medium">Viewing cached content</span>
            <span class="ml-2 text-sm text-blue-700">— Your changes will sync when you're back online</span>
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

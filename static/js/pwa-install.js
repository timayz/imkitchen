/**
 * PWA Installation Prompt Logic
 * Story 5.1: PWA Manifest and Installation
 *
 * Handles browser install prompt and provides manual installation trigger
 */

(function() {
    'use strict';

    let deferredPrompt = null;
    let installButton = null;

    /**
     * Initialize PWA installation logic
     */
    function init() {
        // Find install button if it exists on the page
        installButton = document.getElementById('pwa-install-button');

        // Listen for beforeinstallprompt event
        window.addEventListener('beforeinstallprompt', handleBeforeInstallPrompt);

        // Listen for app installed event
        window.addEventListener('appinstalled', handleAppInstalled);

        // Check if app is already installed
        if (window.matchMedia('(display-mode: standalone)').matches) {
            handleAlreadyInstalled();
        }

        // Hide install button if iOS Safari (uses different installation method)
        if (isIOSSafari() && installButton) {
            // iOS uses "Add to Home Screen" from share menu
            installButton.style.display = 'none';
        }
    }

    /**
     * Handle beforeinstallprompt event
     * Fired when browser determines app is installable
     */
    function handleBeforeInstallPrompt(event) {
        console.log('[PWA] beforeinstallprompt event fired');

        // Prevent default browser install prompt
        event.preventDefault();

        // Store the event for later use
        deferredPrompt = event;

        // Show custom install button
        if (installButton) {
            installButton.style.display = 'block';
            installButton.addEventListener('click', showInstallPrompt);
        }
    }

    /**
     * Show install prompt when user clicks install button
     */
    function showInstallPrompt() {
        if (!deferredPrompt) {
            console.log('[PWA] No deferred prompt available');
            return;
        }

        console.log('[PWA] Showing install prompt');

        // Show the browser install prompt
        deferredPrompt.prompt();

        // Wait for user choice
        deferredPrompt.userChoice.then(function(choiceResult) {
            console.log('[PWA] User choice:', choiceResult.outcome);

            if (choiceResult.outcome === 'accepted') {
                console.log('[PWA] User accepted installation');
                // Track analytics if configured
                trackInstallAccepted();
            } else {
                console.log('[PWA] User dismissed installation');
                // Track analytics if configured
                trackInstallDismissed();
            }

            // Clear the deferred prompt
            deferredPrompt = null;
        });
    }

    /**
     * Handle app installed event
     */
    function handleAppInstalled(event) {
        console.log('[PWA] App installed successfully');

        // Hide install button
        if (installButton) {
            installButton.style.display = 'none';
        }

        // Track analytics if configured
        trackInstallCompleted();
    }

    /**
     * Handle case where app is already installed
     */
    function handleAlreadyInstalled() {
        console.log('[PWA] App is already installed (running in standalone mode)');

        // Hide install button
        if (installButton) {
            installButton.style.display = 'none';
        }
    }

    /**
     * Check if running on iOS Safari
     */
    function isIOSSafari() {
        const userAgent = window.navigator.userAgent.toLowerCase();
        const isIOS = /iphone|ipad|ipod/.test(userAgent);
        const isSafari = /safari/.test(userAgent) && !/chrome|crios|fxios/.test(userAgent);
        return isIOS && isSafari;
    }

    /**
     * Track install accepted (placeholder for analytics)
     */
    function trackInstallAccepted() {
        // TODO: Integrate with analytics service if needed
        console.log('[PWA] Analytics: Install accepted');
    }

    /**
     * Track install dismissed (placeholder for analytics)
     */
    function trackInstallDismissed() {
        // TODO: Integrate with analytics service if needed
        console.log('[PWA] Analytics: Install dismissed');
    }

    /**
     * Track install completed (placeholder for analytics)
     */
    function trackInstallCompleted() {
        // TODO: Integrate with analytics service if needed
        console.log('[PWA] Analytics: Install completed');
    }

    // Initialize when DOM is ready
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', init);
    } else {
        init();
    }
})();

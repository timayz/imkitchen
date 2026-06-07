/**
 * PWA Installation Prompt Logic
 * Story 5.1: PWA Manifest and Installation
 *
 * - Captures the browser's `beforeinstallprompt` event when available
 * - Opens a footer-hosted modal (`#pwa-install-modal`) with platform-specific
 *   install instructions when the user clicks `#pwa-install-trigger`
 */

(function () {
    'use strict';

    let deferredPrompt = null;

    function $(sel, root) { return (root || document).querySelector(sel); }

    function init() {
        window.addEventListener('beforeinstallprompt', function (event) {
            event.preventDefault();
            deferredPrompt = event;
        });

        window.addEventListener('appinstalled', function () {
            deferredPrompt = null;
            const modal = $('#pwa-install-modal');
            if (modal && !modal.hidden) {
                showSection('installed');
            }
        });

        const trigger = $('#pwa-install-trigger');
        const modal = $('#pwa-install-modal');
        if (!trigger || !modal) return;

        trigger.addEventListener('click', openModal);

        modal.querySelectorAll('[data-pwa-install-close]').forEach(function (el) {
            el.addEventListener('click', closeModal);
        });

        document.addEventListener('keydown', function (e) {
            if (e.key === 'Escape' && !modal.hidden) closeModal();
        });

        const installBtn = $('#pwa-install-button');
        if (installBtn) installBtn.addEventListener('click', triggerInstallPrompt);
    }

    function openModal() {
        const modal = $('#pwa-install-modal');
        if (!modal) return;
        showSection(detectPlatform());
        modal.hidden = false;
        document.body.style.overflow = 'hidden';
    }

    function closeModal() {
        const modal = $('#pwa-install-modal');
        if (!modal) return;
        modal.hidden = true;
        document.body.style.overflow = '';
    }

    function showSection(name) {
        const modal = $('#pwa-install-modal');
        if (!modal) return;
        modal.querySelectorAll('[data-pwa-section]').forEach(function (section) {
            section.hidden = section.getAttribute('data-pwa-section') !== name;
        });
    }

    function detectPlatform() {
        if (window.matchMedia('(display-mode: standalone)').matches ||
            window.navigator.standalone === true) {
            return 'installed';
        }

        if (deferredPrompt) return 'prompt';

        const ua = window.navigator.userAgent;
        const isIOS = /iphone|ipad|ipod/i.test(ua);
        if (isIOS) return 'ios';

        const isAndroid = /android/i.test(ua);
        if (isAndroid) return 'android-manual';

        return 'desktop-manual';
    }

    function triggerInstallPrompt() {
        if (!deferredPrompt) return;
        deferredPrompt.prompt();
        deferredPrompt.userChoice.then(function () {
            deferredPrompt = null;
            closeModal();
        });
    }

    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', init);
    } else {
        init();
    }
})();

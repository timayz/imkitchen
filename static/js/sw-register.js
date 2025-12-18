// Service Worker Registration - Story 5.2
// Registers service worker with feature detection and error handling

(function() {
  'use strict';

  // Feature detection: Check if service worker API is available
  if (!('serviceWorker' in navigator)) {
    console.warn('Service Worker not supported in this browser');
    return;
  }

  // Register service worker on DOMContentLoaded
  document.addEventListener('DOMContentLoaded', function() {
    registerServiceWorker();
  });

  /**
   * Register the service worker at /sw.js with root scope
   */
  async function registerServiceWorker() {
    try {
      const registration = await navigator.serviceWorker.register('/sw.js?v=v0.1', {
        scope: '/'
      });

      console.log('Service Worker registered successfully:', registration);
      console.log('Service Worker scope:', registration.scope);

      // Handle first-time activation - reload when SW takes control
      if (!navigator.serviceWorker.controller) {
        navigator.serviceWorker.addEventListener('controllerchange', () => {
          console.log('Service Worker now controlling page, reloading...');
          window.location.reload();
        });
      }

      // Handle service worker updates
      registration.addEventListener('updatefound', () => {
        const newWorker = registration.installing;
        console.log('Service Worker update found');

        newWorker.addEventListener('statechange', () => {
          console.log('Service Worker state changed:', newWorker.state);

          if (newWorker.state === 'installed' && navigator.serviceWorker.controller) {
            // New service worker available, show update notification
            showUpdateNotification();
          }
        });
      });

      // Check for updates every 5 minutes (reduced from 60s per review feedback)
      setInterval(() => {
        registration.update();
      }, 300000);

    } catch (error) {
      console.error('Service Worker registration failed:', error);
    }
  }

  /**
   * Show update notification when new service worker is available
   */
  function showUpdateNotification() {
    console.log('New version available. Refresh to update.');

    // Create update banner
    const banner = document.createElement('div');
    banner.className = 'fixed top-0 left-0 right-0 bg-primary-500 text-white p-4 flex items-center justify-between z-50 shadow-lg';
    banner.innerHTML = `
            <span class="font-medium">New version available. Refresh to update.</span>
            <button id="sw-refresh-btn" class="bg-white text-primary-500 px-4 py-2 rounded font-medium hover:bg-gray-100 transition-colors">
                Refresh
            </button>
        `;

    document.body.insertBefore(banner, document.body.firstChild);

    // Handle refresh button click
    document.getElementById('sw-refresh-btn').addEventListener('click', () => {
      window.location.reload();
    });
  }
})();

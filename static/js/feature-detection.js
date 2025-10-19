/**
 * Feature Detection for PWA APIs
 *
 * Story 5.7: Cross-Browser Compatibility
 * AC-3: Feature detection for PWA APIs (service worker, Web Push, Wake Lock)
 *
 * Provides centralized feature detection with graceful degradation.
 * Displays user-friendly warnings when features are unavailable.
 */

const FeatureDetection = {
  /**
   * Detect Service Worker support
   * Supported: Chrome 40+, Firefox 44+, Safari 11.1+, Edge 17+
   */
  hasServiceWorker() {
    return 'serviceWorker' in navigator;
  },

  /**
   * Detect Background Sync API support
   * Supported: Chrome 49+, Edge 79+
   * NOT supported: iOS Safari, Firefox (as of 2024)
   */
  hasBackgroundSync() {
    if (!this.hasServiceWorker()) {
      return false;
    }

    return 'sync' in ServiceWorkerRegistration.prototype;
  },

  /**
   * Detect Wake Lock API support (screen keep-awake for kitchen mode)
   * Supported: Chrome 84+, Edge 84+, Safari 16.4+
   * NOT supported: Firefox (as of 2024)
   */
  hasWakeLock() {
    return 'wakeLock' in navigator;
  },

  /**
   * Detect Web Push API support
   * Supported: Chrome 42+, Firefox 44+, Edge 17+
   * Partial support: iOS Safari 16.4+ (requires user gesture)
   */
  hasWebPush() {
    return 'PushManager' in window && this.hasServiceWorker();
  },

  /**
   * Detect if running as installed PWA
   * Checks display-mode media query
   */
  isInstalledPWA() {
    return window.matchMedia('(display-mode: standalone)').matches ||
           window.matchMedia('(display-mode: fullscreen)').matches ||
           window.navigator.standalone === true; // iOS Safari
  },

  /**
   * Detect beforeinstallprompt support (PWA install prompt)
   * Supported: Chrome, Edge
   * NOT supported: iOS Safari, Firefox
   */
  hasInstallPrompt() {
    return 'onbeforeinstallprompt' in window;
  },

  /**
   * Get browser name and version (simplified detection)
   */
  getBrowserInfo() {
    const ua = navigator.userAgent;
    let browser = 'Unknown';
    let version = 'Unknown';

    if (ua.includes('Firefox/')) {
      browser = 'Firefox';
      version = ua.match(/Firefox\/(\d+)/)?.[1] || 'Unknown';
    } else if (ua.includes('Edg/')) {
      browser = 'Edge';
      version = ua.match(/Edg\/(\d+)/)?.[1] || 'Unknown';
    } else if (ua.includes('Chrome/')) {
      browser = 'Chrome';
      version = ua.match(/Chrome\/(\d+)/)?.[1] || 'Unknown';
    } else if (ua.includes('Safari/') && !ua.includes('Chrome')) {
      browser = 'Safari';
      version = ua.match(/Version\/(\d+)/)?.[1] || 'Unknown';
    }

    return { browser, version };
  },

  /**
   * Check if browser meets minimum version requirements
   * iOS Safari >= 14, Chrome >= 90, Firefox >= 88
   */
  meetsMinimumRequirements() {
    const { browser, version } = this.getBrowserInfo();
    const versionNum = parseInt(version, 10);

    if (isNaN(versionNum)) {
      return true; // Unknown browser, assume supported
    }

    switch (browser) {
      case 'Safari':
        return versionNum >= 14;
      case 'Chrome':
        return versionNum >= 90;
      case 'Firefox':
        return versionNum >= 88;
      case 'Edge':
        return versionNum >= 90;
      default:
        return true; // Unknown browser, assume supported
    }
  },

  /**
   * Display warning for missing features
   */
  showFeatureWarning(featureName, message) {
    console.warn(`Feature not supported: ${featureName}`, message);

    // Create warning toast if container exists
    const toastContainer = document.getElementById('feature-warnings');
    if (toastContainer) {
      const warning = document.createElement('div');
      warning.className = 'feature-warning bg-yellow-100 border-l-4 border-yellow-500 text-yellow-700 p-4 mb-2';
      warning.setAttribute('role', 'alert');
      warning.innerHTML = `
        <p class="font-bold">${featureName} not supported</p>
        <p class="text-sm">${message}</p>
      `;
      toastContainer.appendChild(warning);

      // Auto-dismiss after 10 seconds
      setTimeout(() => {
        warning.remove();
      }, 10000);
    }
  },

  /**
   * Display browser upgrade warning
   */
  showBrowserUpgradeWarning() {
    const { browser, version } = this.getBrowserInfo();
    const upgradeContainer = document.getElementById('browser-upgrade-warning');

    if (!upgradeContainer) {
      return;
    }

    upgradeContainer.className = 'bg-red-100 border-l-4 border-red-500 text-red-700 p-4 mb-4';
    upgradeContainer.setAttribute('role', 'alert');
    upgradeContainer.innerHTML = `
      <p class="font-bold">Browser Update Required</p>
      <p>You're using ${browser} ${version}. For the best experience, please update to:</p>
      <ul class="list-disc ml-6 mt-2">
        <li>iOS Safari 14+ or Android Chrome 90+</li>
        <li>Desktop: Chrome 90+, Firefox 88+, Safari 14+</li>
      </ul>
      <p class="mt-2">
        <a href="/browser-support" class="underline">Learn more about browser support</a>
      </p>
    `;
    upgradeContainer.style.display = 'block';
  },

  /**
   * Initialize feature detection and display warnings
   */
  init() {
    // Check minimum browser requirements
    if (!this.meetsMinimumRequirements()) {
      this.showBrowserUpgradeWarning();
      return;
    }

    // Service Worker detection
    if (!this.hasServiceWorker()) {
      this.showFeatureWarning(
        'Service Worker',
        'Offline functionality and background sync will not be available. Please upgrade your browser.'
      );
    }

    // Background Sync detection (iOS Safari limitation)
    if (this.hasServiceWorker() && !this.hasBackgroundSync()) {
      const { browser } = this.getBrowserInfo();
      if (browser === 'Safari') {
        this.showFeatureWarning(
          'Background Sync',
          'iOS Safari does not support background sync. Your changes will sync when you open the app.'
        );
      }
    }

    // Wake Lock detection (kitchen mode feature)
    if (!this.hasWakeLock()) {
      console.info('Wake Lock API not supported. Kitchen mode "keep screen awake" feature unavailable.');
    }

    // Web Push detection
    if (!this.hasWebPush()) {
      console.info('Web Push not supported. Push notifications will not be available.');
    }

    // Log feature support summary
    console.info('Feature Support Summary:', {
      serviceWorker: this.hasServiceWorker(),
      backgroundSync: this.hasBackgroundSync(),
      wakeLock: this.hasWakeLock(),
      webPush: this.hasWebPush(),
      installedPWA: this.isInstalledPWA(),
      installPrompt: this.hasInstallPrompt(),
      browserInfo: this.getBrowserInfo(),
    });
  },
};

// Auto-initialize on DOM ready
if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', () => FeatureDetection.init());
} else {
  FeatureDetection.init();
}

// Export for use in other scripts
if (typeof window !== 'undefined') {
  window.FeatureDetection = FeatureDetection;
}

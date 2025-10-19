/**
 * Push Notification Subscription Manager
 * Story 4.10: Push Notification Permission Flow
 *
 * AC #1-8: Handles notification permission request, service worker registration,
 * push subscription creation, and grace period enforcement.
 */

const PushSubscription = {
    vapidPublicKey: null,
    serviceWorkerPath: '/sw.js',

    /**
     * Initialize push subscription manager with VAPID public key
     * @param {string} vapidPublicKey - VAPID public key for push subscription
     * @param {string} serviceWorkerPath - Path to service worker file (default: '/sw.js')
     */
    init(vapidPublicKey, serviceWorkerPath = '/sw.js') {
        this.vapidPublicKey = vapidPublicKey;
        this.serviceWorkerPath = serviceWorkerPath;
    },

    /**
     * Check if push notifications are supported in the current browser
     * Returns: { supported: boolean, reason: string }
     */
    checkBrowserSupport() {
        // Check for Notification API
        if (!('Notification' in window)) {
            return {
                supported: false,
                reason: 'Notifications not supported',
                userMessage: 'Push notifications are not supported in your browser. Please use Chrome, Firefox, Edge, or Safari 16+.'
            };
        }

        // Check for Service Worker API
        if (!('serviceWorker' in navigator)) {
            return {
                supported: false,
                reason: 'Service Workers not supported',
                userMessage: 'Your browser does not support service workers. Please update to the latest version or use a modern browser.'
            };
        }

        // Check for Push API
        if (!('PushManager' in window)) {
            return {
                supported: false,
                reason: 'Push API not supported',
                userMessage: 'Push notifications are not supported in your browser. Safari users: please update to Safari 16+. Otherwise, use Chrome, Firefox, or Edge.'
            };
        }

        // Detect Safari < 16 (no push support)
        const isSafari = /^((?!chrome|android).)*safari/i.test(navigator.userAgent);
        if (isSafari) {
            // Safari 16+ supports web push, earlier versions do not
            const safariVersion = navigator.userAgent.match(/version\/(\d+)/i);
            if (safariVersion && parseInt(safariVersion[1]) < 16) {
                return {
                    supported: false,
                    reason: 'Safari version too old',
                    userMessage: 'Push notifications require Safari 16 or later. Please update Safari or use Chrome, Firefox, or Edge.'
                };
            }
        }

        return {
            supported: true,
            reason: 'All APIs supported',
            userMessage: null
        };
    },

    /**
     * Convert base64 VAPID key to Uint8Array for PushManager API
     */
    urlBase64ToUint8Array(base64String) {
        const padding = '='.repeat((4 - base64String.length % 4) % 4);
        const base64 = (base64String + padding)
            .replace(/\-/g, '+')
            .replace(/_/g, '/');

        const rawData = window.atob(base64);
        const outputArray = new Uint8Array(rawData.length);

        for (let i = 0; i < rawData.length; ++i) {
            outputArray[i] = rawData.charCodeAt(i);
        }
        return outputArray;
    },

    /**
     * Request notification permission from browser
     * AC #1, #2, #3: Ask user for permission with benefits explanation
     *
     * Returns: Promise<NotificationPermission> - 'granted', 'denied', or 'default'
     */
    async requestPermission() {
        if (!('Notification' in window)) {
            console.error('This browser does not support notifications');
            alert('Push notifications are not supported in your browser. Please try using a modern browser like Chrome, Firefox, or Edge.');
            return 'denied';
        }

        try {
            const permission = await Notification.requestPermission();
            return permission;
        } catch (error) {
            console.error('Error requesting notification permission:', error);
            return 'denied';
        }
    },

    /**
     * Register service worker for push notifications
     * AC #4: If allowed, register service worker
     */
    async registerServiceWorker() {
        if (!('serviceWorker' in navigator)) {
            console.error('Service workers are not supported');
            alert('Your browser does not support service workers, which are required for push notifications. Please update to the latest version of your browser or use Chrome, Firefox, or Edge.');
            return null;
        }

        try {
            const registration = await navigator.serviceWorker.register(this.serviceWorkerPath);
            console.log('Service worker registered:', registration);

            // Wait for service worker to be ready
            await navigator.serviceWorker.ready;

            return registration;
        } catch (error) {
            console.error('Service worker registration failed:', error);
            alert('Failed to register service worker. This may be due to browser security settings or using an insecure connection (HTTP). Push notifications require HTTPS.');
            return null;
        }
    },

    /**
     * Create push subscription using PushManager API
     * AC #4: Create subscription with VAPID public key
     */
    async createPushSubscription(registration) {
        if (!registration) {
            console.error('No service worker registration');
            return null;
        }

        // Check for PushManager support
        if (!('pushManager' in registration)) {
            console.error('Push messaging is not supported');
            alert('Push messaging is not supported in your browser. Safari users: please update to Safari 16+ or use Chrome/Firefox/Edge for push notifications.');
            return null;
        }

        try {
            const applicationServerKey = this.urlBase64ToUint8Array(this.vapidPublicKey);

            const subscription = await registration.pushManager.subscribe({
                userVisibleOnly: true,
                applicationServerKey: applicationServerKey
            });

            console.log('Push subscription created:', subscription);
            return subscription;
        } catch (error) {
            console.error('Failed to create push subscription:', error);

            // Provide helpful error messages based on common failure scenarios
            if (error.name === 'NotAllowedError') {
                alert('Push notification permission was denied. Please allow notifications in your browser settings to enable this feature.');
            } else if (error.name === 'NotSupportedError') {
                alert('Push notifications are not supported on this device or browser version. Please try using a desktop browser or update your mobile browser.');
            } else {
                alert('Failed to enable push notifications. Please try again or contact support if the problem persists.');
            }

            return null;
        }
    },

    /**
     * Send subscription data to server
     * AC #4: Store subscription endpoint, keys, and user ID
     */
    async sendSubscriptionToServer(subscription) {
        const p256dh = subscription.getKey('p256dh');
        const auth = subscription.getKey('auth');

        const subscriptionData = {
            endpoint: subscription.endpoint,
            p256dh_key: btoa(String.fromCharCode.apply(null, new Uint8Array(p256dh))),
            auth_key: btoa(String.fromCharCode.apply(null, new Uint8Array(auth)))
        };

        try {
            const response = await fetch('/api/notifications/subscribe', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify(subscriptionData)
            });

            if (!response.ok) {
                throw new Error(`Server responded with ${response.status}`);
            }

            const result = await response.json();
            console.log('Subscription sent to server:', result);
            return result;
        } catch (error) {
            console.error('Failed to send subscription to server:', error);
            throw error;
        }
    },

    /**
     * Record permission change on server for grace period tracking
     * AC #5, #8: Track denial timestamp for 30-day grace period
     */
    async recordPermissionChange(permissionStatus) {
        try {
            const response = await fetch('/api/notifications/permission', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({ permission_status: permissionStatus })
            });

            if (!response.ok) {
                throw new Error(`Server responded with ${response.status}`);
            }

            const result = await response.json();
            console.log('Permission change recorded:', result);
            return result;
        } catch (error) {
            console.error('Failed to record permission change:', error);
            throw error;
        }
    },

    /**
     * Complete flow: Request permission, register SW, create subscription
     * AC #3: User can allow, deny, or skip
     */
    async enablePushNotifications() {
        // Check browser support first
        const browserSupport = this.checkBrowserSupport();
        if (!browserSupport.supported) {
            console.error('Browser not supported:', browserSupport.reason);
            alert(browserSupport.userMessage);
            return { success: false, reason: 'unsupported_browser', message: browserSupport.reason };
        }

        // Request permission
        const permission = await this.requestPermission();

        if (permission === 'granted') {
            // Record "granted" status
            await this.recordPermissionChange('granted');

            // Register service worker
            const registration = await this.registerServiceWorker();
            if (!registration) {
                console.error('Failed to register service worker');
                return { success: false, reason: 'service_worker_failed' };
            }

            // Create push subscription
            const subscription = await this.createPushSubscription(registration);
            if (!subscription) {
                console.error('Failed to create push subscription');
                return { success: false, reason: 'subscription_failed' };
            }

            // Send subscription to server
            try {
                await this.sendSubscriptionToServer(subscription);
                return { success: true, permission: 'granted' };
            } catch (error) {
                return { success: false, reason: 'server_error', error };
            }
        } else if (permission === 'denied') {
            // AC #5, #8: Record denial with timestamp for grace period
            await this.recordPermissionChange('denied');
            return { success: false, permission: 'denied' };
        } else {
            // User dismissed or skipped
            return { success: false, permission: 'default' };
        }
    },

    /**
     * Handle skip action in onboarding
     * AC #3: User can skip permission request
     */
    async skipPermissionRequest() {
        await this.recordPermissionChange('skipped');
        return { success: true, skipped: true };
    },

    /**
     * Check notification status
     * AC #7: Check if notifications are enabled
     */
    async getNotificationStatus() {
        try {
            const response = await fetch('/api/notifications/status');
            if (!response.ok) {
                throw new Error(`Server responded with ${response.status}`);
            }

            const status = await response.json();
            return status; // { enabled, subscription_count, can_prompt }
        } catch (error) {
            console.error('Failed to get notification status:', error);
            throw error;
        }
    }
};

// Export for use in other modules
if (typeof module !== 'undefined' && module.exports) {
    module.exports = PushSubscription;
}

// Service Worker for imkitchen - Story 4.6: Push Notifications
// Handles push notification events and notification clicks

const CACHE_NAME = 'imkitchen-v1';

// Install event - basic cache setup
self.addEventListener('install', (event) => {
    console.log('Service Worker installing...');
    self.skipWaiting();
});

// Activate event
self.addEventListener('activate', (event) => {
    console.log('Service Worker activating...');
    event.waitUntil(clients.claim());
});

// Push event - Story 4.6 AC-5: Display push notifications
self.addEventListener('push', (event) => {
    console.log('Push notification received:', event);

    let data = {
        title: 'Prep Reminder',
        body: 'Time to start preparing your meal',
        icon: '/static/icons/icon-192.png',
        badge: '/static/icons/badge-72.png',
        data: {
            url: '/notifications'
        }
    };

    // Parse push payload if available
    if (event.data) {
        try {
            data = event.data.json();
        } catch (e) {
            console.error('Failed to parse push data:', e);
        }
    }

    const options = {
        body: data.body,
        icon: data.icon || '/static/icons/icon-192.png',
        badge: data.badge || '/static/icons/badge-72.png',
        data: data.data || { url: '/notifications' },
        actions: data.actions || [
            { action: 'view', title: 'View Recipe' },
            { action: 'dismiss', title: 'Dismiss' }
        ],
        vibrate: [200, 100, 200],
        requireInteraction: false,
        tag: 'prep-reminder'
    };

    event.waitUntil(
        self.registration.showNotification(data.title, options)
    );
});

// Notification click event - Story 4.6 AC-7: Deep link to recipe
self.addEventListener('notificationclick', (event) => {
    console.log('Notification clicked:', event);

    event.notification.close();

    const urlToOpen = event.notification.data?.url || '/notifications';

    event.waitUntil(
        clients.matchAll({ type: 'window', includeUncontrolled: true })
            .then((clientList) => {
                // Check if app is already open
                for (const client of clientList) {
                    if (client.url.includes(self.location.origin) && 'focus' in client) {
                        client.focus();
                        client.navigate(urlToOpen);
                        return;
                    }
                }
                // Open new window if app not open
                if (clients.openWindow) {
                    return clients.openWindow(urlToOpen);
                }
            })
    );
});

// Handle notification close
self.addEventListener('notificationclose', (event) => {
    console.log('Notification closed:', event);
});

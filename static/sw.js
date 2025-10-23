// Service Worker Source - Story 5.2
// Combined service worker with Workbox caching + Push notifications (Story 4.6)
// This file is processed by Workbox CLI to generate static/sw.js

// Note: SRI (Subresource Integrity) is not supported for importScripts() in service workers
// as of the current Service Worker spec. The Google CDN used here is highly trusted.
// For additional security, consider migrating to npm-installed Workbox modules in future.
// Reference: https://developers.google.com/web/tools/workbox/guides/using-bundlers
importScripts('https://storage.googleapis.com/workbox-cdn/releases/7.1.0/workbox-sw.js');

// Initialize Workbox
if (workbox) {
  console.log('Workbox loaded successfully');

  // Configure cache ID and update behavior
  workbox.core.setCacheNameDetails({
    prefix: 'imkitchen',
    suffix: 'v1'
  });

  // Skip waiting and claim clients immediately on update
  workbox.core.skipWaiting();
  workbox.core.clientsClaim();

  // Precache static assets (manifest injected by Workbox CLI)
  workbox.precaching.precacheAndRoute([{"revision":"2092e500a50382606ad4d87cd0c5c85a","url":"/static/css/main.css"},{"revision":"c5e0998b75e998fb544fa2da4b80757b","url":"/static/css/tailwind.css"},{"revision":"9c0bf863e65d6c208a57bd2b63896df7","url":"/static/icons/apple-touch-icon.png"},{"revision":"92b24651b31dd4e8c233330512d4fdf3","url":"/static/icons/icon-192-maskable.png"},{"revision":"14aae3946b6c97d6b80693c6edef76a6","url":"/static/icons/icon-192.png"},{"revision":"e7e8f5a5500290d14ac25e8ae14c4e1b","url":"/static/icons/icon-512-maskable.png"},{"revision":"da087846670ad8ca9784a40e7febfd69","url":"/static/icons/icon-512.png"},{"revision":"2c0121d0b07c2d3c9a3a22b207441dcd","url":"/static/icons/icon-maskable.svg"},{"revision":"18e1cf596913c6fcced548f0ee5852b6","url":"/static/icons/icon.svg"},{"revision":"a8feda6f7d2cbc7d651e6cd627dbb496","url":"/static/icons/shortcut-dashboard.png"},{"revision":"a8feda6f7d2cbc7d651e6cd627dbb496","url":"/static/icons/shortcut-recipes.png"},{"revision":"057f2441d71fb0c051c91277eb5e9273","url":"/static/js/feature-detection.js"},{"revision":"8529e0b0fae9c0c1eb9e68a51460fde4","url":"/static/js/kitchen-mode.js"},{"revision":"0a5885b62d77ac74960a072c82ed0652","url":"/static/js/meal-regeneration.js"},{"revision":"e9074b9047b7aa91ecd3805736d7220b","url":"/static/js/meal-replacement.js"},{"revision":"807bedac4a42f00e05efa3e221844bdb","url":"/static/js/offline-db.js"},{"revision":"419306bf5ded7db1c82e91e241c5bdaa","url":"/static/js/offline-indicator.js"},{"revision":"d7cc6c4feef0c898b6979516b46d40b8","url":"/static/js/polyfills.js"},{"revision":"f4d1c1a9b83d2ac3e3c1ced4254729ab","url":"/static/js/push-subscription.js"},{"revision":"9faae8de3ae27a94a5aae5d090fc8cc0","url":"/static/js/pwa-install.js"},{"revision":"a48cbf7788b0637e2e214b71aa8151bb","url":"/static/js/shopping-checkoff.js"},{"revision":"555498175f096ea96e544b52db944d59","url":"/static/js/sw-register.js"},{"revision":"0403b1a58b3b4d78c389d40f821f9dcb","url":"/static/js/sync-ui.js"},{"revision":"bf08a792573cd249c76ee88a0de0f1ff","url":"/static/js/touch-enhancements.js"},{"revision":"c10a1b3bf2335c6c61b5c4b35ef2eef5","url":"/static/js/twinspark.js"},{"revision":"b29ea81655ba2a8d0a6972cc44982773","url":"/static/js/twinspark.min.js"},{"revision":"17a371c907aa569828a81bdd2bde76bd","url":"/static/screenshots/dashboard-mobile.png"},{"revision":"140e8a6cfb51ad453f54b23769b106d9","url":"/static/screenshots/dashboard-mobile.svg"},{"revision":"758bb95bf894e397a88dbd982dec8f71","url":"/static/screenshots/meal-calendar-desktop.png"},{"revision":"345c0e789c18b393c64560e9463cf1f9","url":"/static/screenshots/meal-calendar-desktop.svg"},{"revision":"f7ea03ffd8722e7664e4d523995c68e6","url":"/static/screenshots/recipe-detail-mobile.png"},{"revision":"36515dc1e40312ed29b96e7d9a1117e7","url":"/static/screenshots/recipe-detail-mobile.svg"}] || []);

  // Runtime caching strategies

  // HTML pages: Network-first with cache fallback
  workbox.routing.registerRoute(
    ({request}) => request.mode === 'navigate',
    new workbox.strategies.NetworkFirst({
      cacheName: 'pages-v1',
      plugins: [
        new workbox.expiration.ExpirationPlugin({
          maxEntries: 50,
          maxAgeSeconds: 7 * 24 * 60 * 60 // 7 days
        }),
        new workbox.cacheableResponse.CacheableResponsePlugin({
          statuses: [0, 200]
        })
      ]
    })
  );

  // Images: Cache-first for fast offline access
  workbox.routing.registerRoute(
    ({request}) => request.destination === 'image',
    new workbox.strategies.CacheFirst({
      cacheName: 'images-v1',
      plugins: [
        new workbox.expiration.ExpirationPlugin({
          maxEntries: 100,
          maxAgeSeconds: 30 * 24 * 60 * 60 // 30 days
        })
      ]
    })
  );

  // API/Data endpoints: Network-first with cache fallback
  workbox.routing.registerRoute(
    ({url}) => url.pathname.startsWith('/api') ||
               url.pathname.startsWith('/recipes') ||
               url.pathname.startsWith('/plan') ||
               url.pathname.startsWith('/shopping'),
    new workbox.strategies.NetworkFirst({
      cacheName: 'api-v1',
      plugins: [
        new workbox.expiration.ExpirationPlugin({
          maxEntries: 100,
          maxAgeSeconds: 24 * 60 * 60 // 1 day
        })
      ]
    })
  );

  // Static assets: Cache-first with long expiration
  workbox.routing.registerRoute(
    ({request}) =>
      request.destination === 'style' ||
      request.destination === 'script' ||
      request.destination === 'font',
    new workbox.strategies.CacheFirst({
      cacheName: 'static-v1',
      plugins: [
        new workbox.expiration.ExpirationPlugin({
          maxEntries: 60,
          maxAgeSeconds: 365 * 24 * 60 * 60 // 1 year
        })
      ]
    })
  );

  // Offline fallback for navigation requests
  const OFFLINE_FALLBACK_URL = '/offline';

  // Precache the offline page and monitor storage quota
  self.addEventListener('install', (event) => {
    event.waitUntil(
      Promise.all([
        caches.open('pages-v1').then((cache) => {
          return cache.add(OFFLINE_FALLBACK_URL);
        }),
        checkStorageQuota()
      ])
    );
  });

  // Serve offline fallback when navigation fails
  workbox.routing.setCatchHandler(({event}) => {
    if (event.request.mode === 'navigate') {
      return caches.match(OFFLINE_FALLBACK_URL);
    }
    return Response.error();
  });

  /**
   * Check storage quota and log warnings when approaching limits
   * Helps prevent unbounded cache growth on low-storage devices
   */
  async function checkStorageQuota() {
    if ('storage' in navigator && 'estimate' in navigator.storage) {
      try {
        const estimate = await navigator.storage.estimate();
        const usage = estimate.usage || 0;
        const quota = estimate.quota || 0;
        const percentUsed = quota > 0 ? (usage / quota) * 100 : 0;

        console.log(`Storage: ${(usage / 1024 / 1024).toFixed(2)} MB used of ${(quota / 1024 / 1024).toFixed(2)} MB (${percentUsed.toFixed(1)}%)`);

        // Warn when approaching quota limits
        if (percentUsed > 90) {
          console.warn('Storage quota critical: ' + percentUsed.toFixed(1) + '% used. Consider clearing old caches.');
        } else if (percentUsed > 75) {
          console.warn('Storage quota high: ' + percentUsed.toFixed(1) + '% used.');
        }

        return estimate;
      } catch (error) {
        console.error('Failed to check storage quota:', error);
      }
    } else {
      console.log('Storage estimation API not available');
    }
  }

} else {
  console.error('Workbox failed to load');
}

// ============================================================
// Push Notifications - Story 4.6 (preserved functionality)
// ============================================================

// Push event - Display push notifications
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

// Notification click event - Deep link to recipe
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

// ============================================================
// Background Sync - Story 5.2 AC-7
// ============================================================

// Background Sync for offline mutations
self.addEventListener('sync', (event) => {
  console.log('Background sync triggered:', event.tag);

  if (event.tag === 'sync-offline-actions') {
    event.waitUntil(syncOfflineActions());
  }
});

/**
 * Sync queued offline actions with server
 * Retrieves queued requests from IndexedDB and replays them
 */
async function syncOfflineActions() {
  try {
    // Open IndexedDB to retrieve queued requests
    const db = await openSyncDatabase();
    const requests = await getAllQueuedRequests(db);

    console.log(`Syncing ${requests.length} queued requests`);

    // Replay each request with exponential backoff
    for (const queuedRequest of requests) {
      try {
        await replayRequest(queuedRequest);
        await removeQueuedRequest(db, queuedRequest.id);
        console.log('Successfully synced request:', queuedRequest.id);
      } catch (error) {
        console.error('Failed to sync request:', queuedRequest.id, error);
        // Keep in queue for next sync attempt
        await incrementRetryCount(db, queuedRequest.id);
      }
    }

    console.log('Background sync completed');
  } catch (error) {
    console.error('Background sync failed:', error);
    throw error; // Retry sync on next trigger
  }
}

/**
 * Open IndexedDB for sync queue
 */
function openSyncDatabase() {
  return new Promise((resolve, reject) => {
    const request = indexedDB.open('imkitchen-sync', 1);

    request.onerror = () => reject(request.error);
    request.onsuccess = () => resolve(request.result);

    request.onupgradeneeded = (event) => {
      const db = event.target.result;
      if (!db.objectStoreNames.contains('queue')) {
        const store = db.createObjectStore('queue', { keyPath: 'id', autoIncrement: true });
        store.createIndex('timestamp', 'timestamp', { unique: false });
        store.createIndex('retryCount', 'retryCount', { unique: false });
      }
    };
  });
}

/**
 * Get all queued requests from IndexedDB
 */
function getAllQueuedRequests(db) {
  return new Promise((resolve, reject) => {
    const transaction = db.transaction(['queue'], 'readonly');
    const store = transaction.objectStore('queue');
    const request = store.getAll();

    request.onsuccess = () => resolve(request.result || []);
    request.onerror = () => reject(request.error);
  });
}

/**
 * Replay a queued request to the server
 */
async function replayRequest(queuedRequest) {
  const { url, method, body, headers } = queuedRequest;

  const response = await fetch(url, {
    method,
    headers: headers || {},
    body: body ? JSON.stringify(body) : undefined
  });

  if (!response.ok) {
    throw new Error(`Request failed with status ${response.status}`);
  }

  return response;
}

/**
 * Remove successfully synced request from queue
 */
function removeQueuedRequest(db, id) {
  return new Promise((resolve, reject) => {
    const transaction = db.transaction(['queue'], 'readwrite');
    const store = transaction.objectStore('queue');
    const request = store.delete(id);

    request.onsuccess = () => resolve();
    request.onerror = () => reject(request.error);
  });
}

/**
 * Increment retry count for failed request
 * Remove if max retries (3) exceeded
 */
function incrementRetryCount(db, id) {
  return new Promise((resolve, reject) => {
    const transaction = db.transaction(['queue'], 'readwrite');
    const store = transaction.objectStore('queue');
    const getRequest = store.get(id);

    getRequest.onsuccess = () => {
      const record = getRequest.result;
      if (record) {
        record.retryCount = (record.retryCount || 0) + 1;

        // Remove if max retries exceeded
        if (record.retryCount >= 3) {
          store.delete(id);
          console.warn('Max retries exceeded for request:', id);
        } else {
          store.put(record);
        }
      }
      resolve();
    };

    getRequest.onerror = () => reject(getRequest.error);
  });
}

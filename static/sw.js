// Service Worker Source - Story 5.2 + 5.3
// Combined service worker with Workbox caching + Push notifications (Story 4.6)
// + IndexedDB offline data persistence (Story 5.3)
// This file is processed by Workbox CLI to generate static/sw.js

// Note: SRI (Subresource Integrity) is not supported for importScripts() in service workers
// as of the current Service Worker spec. The Google CDN used here is highly trusted.
// For additional security, consider migrating to npm-installed Workbox modules in future.
// Reference: https://developers.google.com/web/tools/workbox/guides/using-bundlers
importScripts('https://storage.googleapis.com/workbox-cdn/releases/7.1.0/workbox-sw.js');

/**
 * @typedef {Object} OfflineDB
 * @property {function(string, *): Promise<void>} put - Store data in IndexedDB
 * @property {function(string, string|number): Promise<*>} get - Retrieve data from IndexedDB
 * @property {function(string, string|number): Promise<void>} remove - Delete data from IndexedDB
 * @property {function(string): Promise<Array>} getAll - Get all records from a store
 * @property {function(*): Promise<void>} queueRequest - Queue an offline request for sync
 * @property {function(): Promise<Array>} getQueuedRequests - Get all queued sync requests
 * @property {function(number): Promise<void>} removeQueuedRequest - Remove a request from sync queue
 */

/** @type {OfflineDB} */
// eslint-disable-next-line no-undef
importScripts('/static/js/offline-db.js');

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
  workbox.precaching.precacheAndRoute([{"revision":"fc3d16bc9f8ab09142869ec6cd0b813d","url":"css/main.css"},{"revision":"a68300dc81da25933868483887859aa3","url":"css/tailwind.css"},{"revision":"64bec4cade0f438e7f0eb3a99147e7ad","url":"icons/apple-touch-icon.png"},{"revision":"e82608c5290db4953645ac9309b2115b","url":"icons/icon-192-maskable.png"},{"revision":"9c987d8a93f80dff28d25a0179b9045f","url":"icons/icon-192.png"},{"revision":"484c4146cad4fac958d65f42dbf3d2a9","url":"icons/icon-512-maskable.png"},{"revision":"392977e6d2c32330ff38d80269674655","url":"icons/icon-512.png"},{"revision":"5a73e111de0b068cdbcb9631e5536bb0","url":"icons/icon-maskable.svg"},{"revision":"846125c7aa7e0bd8db280ed4132266c2","url":"icons/icon.svg"},{"revision":"a8feda6f7d2cbc7d651e6cd627dbb496","url":"icons/shortcut-dashboard.png"},{"revision":"a8feda6f7d2cbc7d651e6cd627dbb496","url":"icons/shortcut-recipes.png"},{"revision":"58c77c2aa0014b1e01c8237143aa2d2a","url":"js/kitchen-mode.js"},{"revision":"0a5885b62d77ac74960a072c82ed0652","url":"js/meal-regeneration.js"},{"revision":"e9074b9047b7aa91ecd3805736d7220b","url":"js/meal-replacement.js"},{"revision":"7873b153974a9236eb72e9f1e5e7fefa","url":"js/offline-indicator.js"},{"revision":"757fb6bd855fe5e27769427b5d2d4ea1","url":"js/push-subscription.js"},{"revision":"9faae8de3ae27a94a5aae5d090fc8cc0","url":"js/pwa-install.js"},{"revision":"555498175f096ea96e544b52db944d59","url":"js/sw-register.js"},{"revision":"c10a1b3bf2335c6c61b5c4b35ef2eef5","url":"js/twinspark.js"},{"revision":"b29ea81655ba2a8d0a6972cc44982773","url":"js/twinspark.min.js"},{"revision":"17a371c907aa569828a81bdd2bde76bd","url":"screenshots/dashboard-mobile.png"},{"revision":"140e8a6cfb51ad453f54b23769b106d9","url":"screenshots/dashboard-mobile.svg"},{"revision":"758bb95bf894e397a88dbd982dec8f71","url":"screenshots/meal-calendar-desktop.png"},{"revision":"345c0e789c18b393c64560e9463cf1f9","url":"screenshots/meal-calendar-desktop.svg"},{"revision":"f7ea03ffd8722e7664e4d523995c68e6","url":"screenshots/recipe-detail-mobile.png"},{"revision":"36515dc1e40312ed29b96e7d9a1117e7","url":"screenshots/recipe-detail-mobile.svg"}] || []);

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

  // Recipe images: Cache-first for fast offline access (Story 5.3 AC-2)
  workbox.routing.registerRoute(
    ({request, url}) => {
      // Match recipe images specifically
      return request.destination === 'image' &&
             (url.pathname.includes('/recipes/') || url.pathname.includes('/recipe-images/'));
    },
    new workbox.strategies.CacheFirst({
      cacheName: 'recipe-images-cache',
      plugins: [
        new workbox.expiration.ExpirationPlugin({
          maxEntries: 100,
          maxAgeSeconds: 30 * 24 * 60 * 60, // 30 days
          purgeOnQuotaError: true // Auto-purge on storage quota errors
        }),
        new workbox.cacheableResponse.CacheableResponsePlugin({
          statuses: [0, 200] // Cache opaque and successful responses
        })
      ]
    })
  );

  // Other images: Cache-first with shorter expiration
  workbox.routing.registerRoute(
    ({request}) => request.destination === 'image',
    new workbox.strategies.CacheFirst({
      cacheName: 'images-v1',
      plugins: [
        new workbox.expiration.ExpirationPlugin({
          maxEntries: 60,
          maxAgeSeconds: 14 * 24 * 60 * 60 // 14 days
        })
      ]
    })
  );

  // Recipe pages: Stale-while-revalidate with IndexedDB caching (Story 5.3)
  workbox.routing.registerRoute(
    ({url}) => url.pathname.match(/^\/recipes\/[^/]+$/),
    new workbox.strategies.StaleWhileRevalidate({
      cacheName: 'pages-cache',
      plugins: [
        new workbox.expiration.ExpirationPlugin({
          maxEntries: 50,
          maxAgeSeconds: 7 * 24 * 60 * 60 // 7 days
        }),
        {
          // Cache recipe data in IndexedDB after successful fetch
          fetchDidSucceed: async ({request, response}) => {
            try {
              // Clone response to read body without consuming original
              const responseClone = response.clone();
              const html = await responseClone.text();

              // Extract recipe ID from URL
              const recipeId = request.url.match(/\/recipes\/([^/]+)$/)?.[1];

              if (recipeId && html) {
                // Parse recipe data from HTML (server-rendered)
                // This is a simplified extraction - in production, consider embedding JSON-LD
                const titleMatch = html.match(/<h1[^>]*>(.*?)<\/h1>/);
                const title = titleMatch ? titleMatch[1] : 'Unknown Recipe';

                // Store in IndexedDB for offline access
                await offlineDB.cacheRecipe({
                  id: recipeId,
                  title: title,
                  html: html,
                  url: request.url
                });

                console.log('Recipe cached in IndexedDB:', recipeId);
              }
            } catch (error) {
              console.error('Failed to cache recipe in IndexedDB:', error);
            }

            return response;
          }
        }
      ]
    })
  );

  // Meal plan page: Network-first with IndexedDB caching (Story 5.3 AC-4)
  workbox.routing.registerRoute(
    ({url}) => url.pathname === '/plan' || url.pathname.startsWith('/plan/'),
    new workbox.strategies.NetworkFirst({
      cacheName: 'pages-cache',
      plugins: [
        new workbox.expiration.ExpirationPlugin({
          maxEntries: 20,
          maxAgeSeconds: 24 * 60 * 60 // 1 day
        }),
        {
          // Cache meal plan data in IndexedDB after successful fetch
          fetchDidSucceed: async ({request, response}) => {
            try {
              const responseClone = response.clone();
              const html = await responseClone.text();

              // Extract meal plan data from HTML or embedded JSON
              // Look for data-meal-plan attribute or script tag with JSON-LD
              const mealPlanMatch = html.match(/data-meal-plan='([^']+)'/);
              if (mealPlanMatch) {
                const mealPlanData = JSON.parse(mealPlanMatch[1].replace(/&quot;/g, '"'));

                // Store in IndexedDB
                await offlineDB.cacheMealPlan(mealPlanData);
                console.log('Meal plan cached in IndexedDB:', mealPlanData.id);

                // Pre-cache all assigned recipes
                if (mealPlanData.meals) {
                  for (const meal of mealPlanData.meals) {
                    if (meal.recipe_id) {
                      // Trigger recipe caching in background
                      fetch(`/recipes/${meal.recipe_id}`).catch(err => {
                        console.log('Failed to pre-cache recipe:', meal.recipe_id, err);
                      });
                    }
                  }
                }
              }
            } catch (error) {
              console.error('Failed to cache meal plan in IndexedDB:', error);
            }

            return response;
          }
        }
      ]
    })
  );

  // Shopping list page: Network-first with IndexedDB caching (Story 5.3 AC-5)
  workbox.routing.registerRoute(
    ({url}) => url.pathname === '/shopping' || url.pathname.startsWith('/shopping/'),
    new workbox.strategies.NetworkFirst({
      cacheName: 'pages-cache',
      plugins: [
        new workbox.expiration.ExpirationPlugin({
          maxEntries: 10,
          maxAgeSeconds: 7 * 24 * 60 * 60 // 7 days
        }),
        {
          // Cache shopping list data in IndexedDB after successful fetch
          fetchDidSucceed: async ({request, response}) => {
            try {
              const responseClone = response.clone();
              const html = await responseClone.text();

              // Extract shopping list data from HTML
              const shoppingListMatch = html.match(/data-shopping-list='([^']+)'/);
              if (shoppingListMatch) {
                const shoppingListData = JSON.parse(shoppingListMatch[1].replace(/&quot;/g, '"'));

                // Store in IndexedDB
                await offlineDB.cacheShoppingList(shoppingListData);
                console.log('Shopping list cached in IndexedDB:', shoppingListData.id);
              }
            } catch (error) {
              console.error('Failed to cache shopping list in IndexedDB:', error);
            }

            return response;
          }
        }
      ]
    })
  );

  // API/Data endpoints: Network-first with cache fallback
  workbox.routing.registerRoute(
    ({url}) => url.pathname.startsWith('/api'),
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
// Background Sync - Story 5.2 AC-7 + Story 5.8
// ============================================================

// Configuration constants
const BATCH_SIZE = 10; // Max requests per sync batch
const RETRY_DELAYS_MS = [0, 60000, 300000, 900000]; // 0min, 1min, 5min, 15min
const MAX_RETRIES = 3; // Maximum retry attempts before giving up

// Detect network restoration and trigger sync - Story 5.8 Subtask 1.1
self.addEventListener('online', (event) => {
  console.log('Network connectivity restored, triggering background sync');

  // Register background sync when coming back online - Story 5.8 Subtask 1.2
  self.registration.sync.register('sync-changes').then(() => {
    console.log('Background sync registered: sync-changes');
  }).catch((error) => {
    console.error('Failed to register background sync:', error);
    // Fallback: trigger sync immediately if registration fails
    syncOfflineActions();
  });
});

// Background Sync for offline mutations - Story 5.8 Subtask 1.3
self.addEventListener('sync', (event) => {
  console.log('Background sync triggered:', event.tag);

  // Handle both legacy tag and new tag - Story 5.8
  if (event.tag === 'sync-offline-actions' || event.tag === 'sync-changes') {
    event.waitUntil(syncOfflineActions());
  }
});

/**
 * Sync queued offline actions with server
 * Retrieves queued requests from IndexedDB and replays them
 * Story 5.3 AC-7 - Enhanced with sync status notifications
 * Story 5.8 Task 2 - Process in order with batching
 * Story 5.8 Task 3 - Conflict resolution and retry logic
 */
async function syncOfflineActions() {
  try {
    // Use the new offlineDB utility
    const requests = await offlineDB.getQueuedRequests();

    console.log(`Syncing ${requests.length} queued requests`);

    if (requests.length === 0) {
      console.log('No queued requests to sync');
      return;
    }

    // Story 5.8 Subtask 2.3 - Batch processing (use configured batch size)
    const batch = requests.slice(0, BATCH_SIZE);
    const total = batch.length;

    let successCount = 0;
    let failureCount = 0;
    let conflictCount = 0;

    // Story 5.8 Subtask 2.2 - Replay requests in FIFO order
    for (let i = 0; i < batch.length; i++) {
      const queuedRequest = batch[i];
      const current = i + 1;

      // Story 5.8 Subtask 4.2 - Post progress to main thread
      postMessageToClients({
        type: 'SYNC_PROGRESS',
        data: {
          current,
          total
        }
      });

      try {
        // Story 5.8 Subtask 3.1 - Parse server response
        const response = await replayRequest(queuedRequest);

        if (response.status === 409) {
          // Story 5.8 Subtask 3.2 - Handle 409 Conflict
          console.warn('Conflict detected for request:', queuedRequest.request_id);
          conflictCount++;

          // Remove from queue (server wins)
          await offlineDB.removeQueuedRequest(queuedRequest.request_id);

          // Notify user of conflict
          const itemName = extractItemName(queuedRequest);
          postMessageToClients({
            type: 'SYNC_CONFLICT',
            data: {
              itemName,
              requestId: queuedRequest.request_id
            }
          });
        } else {
          // Story 5.8 Subtask 2.5 - Remove successfully synced request
          await offlineDB.removeQueuedRequest(queuedRequest.request_id);
          console.log('Successfully synced request:', queuedRequest.request_id);
          successCount++;
        }
      } catch (error) {
        console.error('Failed to sync request:', queuedRequest.request_id, error);
        failureCount++;

        // Story 5.8 Subtask 3.3 - Exponential backoff retry strategy
        const retryCount = (queuedRequest.retry_count || 0) + 1;
        const now = Date.now();

        // Calculate next retry time with exponential backoff
        const nextRetryDelay = RETRY_DELAYS_MS[Math.min(retryCount, RETRY_DELAYS_MS.length - 1)];
        const nextRetryTime = now + nextRetryDelay;

        if (retryCount >= MAX_RETRIES) {
          // Story 5.8 Subtask 3.4 - After 3 failed retries
          await offlineDB.removeQueuedRequest(queuedRequest.request_id);
          console.warn('Max retries exceeded, removing request:', queuedRequest.request_id);

          // Show persistent error notification
          postMessageToClients({
            type: 'SYNC_ERROR',
            data: {
              message: `Sync failed for ${extractItemName(queuedRequest)}. Please retry manually.`,
              requestId: queuedRequest.request_id,
              persistent: true
            }
          });

          // Story 5.8 Subtask 3.5 - Log to OpenTelemetry (placeholder for future)
          console.error('[OpenTelemetry] Sync failure:', {
            requestId: queuedRequest.request_id,
            url: queuedRequest.url,
            method: queuedRequest.method,
            retryCount,
            error: error.message
          });
        } else {
          // Update retry count and next retry time
          await offlineDB.put('sync_queue', {
            ...queuedRequest,
            retry_count: retryCount,
            next_retry_time: nextRetryTime
          });

          console.log(`Request ${queuedRequest.request_id} will retry in ${nextRetryDelay / 1000}s (attempt ${retryCount + 1}/3)`);
        }
      }
    }

    console.log(`Background sync completed: ${successCount} succeeded, ${failureCount} failed, ${conflictCount} conflicts`);

    // Story 5.8 Subtask 2.3 - Re-register sync if more requests remain
    if (requests.length > BATCH_SIZE) {
      const remaining = requests.length - BATCH_SIZE;
      console.log(`${remaining} requests remaining, re-registering sync`);
      await self.registration.sync.register('sync-changes');
    }

    // Notify user of sync completion
    if (successCount > 0 && failureCount === 0) {
      // All succeeded - Story 5.8 Subtask 4.4
      postMessageToClients({
        type: 'SYNC_COMPLETE',
        data: {
          count: successCount
        }
      });
    } else if (successCount > 0 && failureCount > 0) {
      // Partial success
      postMessageToClients({
        type: 'SYNC_COMPLETE',
        data: {
          count: successCount,
          partialFailure: true
        }
      });
    }
  } catch (error) {
    console.error('Background sync failed:', error);

    // Notify main thread of error
    postMessageToClients({
      type: 'SYNC_ERROR',
      data: {
        message: 'Background sync failed. Will retry automatically.',
        error: error.message
      }
    });

    throw error; // Retry sync on next trigger
  }
}

/**
 * Show notification to user about sync completion
 * @param {number} syncedCount - Number of requests successfully synced
 */
async function showSyncNotification(syncedCount) {
  try {
    const title = 'Changes Synced';
    const body = `${syncedCount} ${syncedCount === 1 ? 'change' : 'changes'} synced to server`;

    await self.registration.showNotification(title, {
      body,
      icon: '/static/icons/icon-192.png',
      badge: '/static/icons/badge-72.png',
      tag: 'sync-complete',
      requireInteraction: false,
      vibrate: [100],
      data: {
        type: 'sync-complete',
        count: syncedCount
      }
    });

    console.log('Sync notification shown');
  } catch (error) {
    console.error('Failed to show sync notification:', error);
  }
}

/**
 * Replay a queued request to the server
 * Story 5.8 Subtask 3.1 - Returns response for status code checking
 */
async function replayRequest(queuedRequest) {
  const { url, method, body, headers } = queuedRequest;

  const response = await fetch(url, {
    method,
    headers: headers || { 'Content-Type': 'application/json' },
    body: body ? JSON.stringify(body) : undefined
  });

  // Story 5.8 Subtask 3.1 - Check response status
  // 200/201/204 = Success
  // 409 = Conflict (handle in caller)
  // 4xx = Client error (don't retry)
  // 5xx = Server error (retry)

  if (response.status >= 500) {
    throw new Error(`Server error: ${response.status}`);
  }

  if (response.status >= 400 && response.status !== 409) {
    // 4xx errors (except 409) - client errors, don't retry
    console.warn(`Client error ${response.status}, not retrying`);
    throw new Error(`Client error: ${response.status}`);
  }

  return response;
}

/**
 * Post message to all active clients (main thread)
 * Story 5.8 Subtask 4.2 - Communication with main thread
 */
async function postMessageToClients(message) {
  const clients = await self.clients.matchAll({ includeUncontrolled: true, type: 'window' });

  clients.forEach((client) => {
    client.postMessage(message);
  });
}

/**
 * Extract item name from request for user-friendly error messages
 * @param {object} queuedRequest
 * @returns {string}
 */
function extractItemName(queuedRequest) {
  try {
    // Try to extract recipe title or other item name from request body
    if (queuedRequest.body) {
      const body = typeof queuedRequest.body === 'string'
        ? JSON.parse(queuedRequest.body)
        : queuedRequest.body;

      if (body.title) return body.title;
      if (body.name) return body.name;
    }

    // Fallback: extract from URL path
    const urlMatch = queuedRequest.url.match(/\/(recipes|meal-plans|shopping)\/([^/]+)/);
    if (urlMatch) {
      return `${urlMatch[1]} item`;
    }

    // Default fallback
    return queuedRequest.method + ' request';
  } catch (error) {
    console.error('Failed to extract item name:', error);
    return 'Unknown item';
  }
}

// ============================================================
// Manual Sync Handler - Story 5.8 Subtask 1.5 (iOS fallback)
// ============================================================

// Listen for manual sync trigger from main thread (iOS fallback)
self.addEventListener('message', (event) => {
  if (event.data && event.data.type === 'MANUAL_SYNC') {
    console.log('Manual sync requested (iOS fallback)');

    // Trigger sync immediately
    event.waitUntil(syncOfflineActions());
  }
});

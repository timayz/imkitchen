// IMKitchen PWA Service Worker
// Handles offline functionality, caching, and background sync

const CACHE_NAME = 'imkitchen-v1';
const OFFLINE_URL = '/offline';

// Resources to cache immediately upon installation
const CORE_CACHE_RESOURCES = [
  '/',
  '/static/css/tailwind.css',
  '/static/js/twinspark.js',
  '/static/manifest.json',
  '/offline',
  '/auth/login',
  '/static/images/icon-192x192.png',
  '/static/images/icon-512x512.png'
];

// Network-first resources (dynamic content)
const NETWORK_FIRST_PATTERNS = [
  /^\/api\//,
  /^\/meal-plans\//,
  /^\/recipes\//,
  /^\/shopping-lists\//,
  /^\/profile\//
];

// Cache-first resources (static assets)
const CACHE_FIRST_PATTERNS = [
  /^\/static\//,
  /\.(?:png|jpg|jpeg|svg|gif|webp|ico)$/,
  /\.(?:css|js)$/
];

// Install event - cache core resources
self.addEventListener('install', event => {
  console.log('[Service Worker] Installing...');
  
  event.waitUntil(
    caches.open(CACHE_NAME)
      .then(cache => {
        console.log('[Service Worker] Caching core resources');
        return cache.addAll(CORE_CACHE_RESOURCES);
      })
      .then(() => {
        console.log('[Service Worker] Core resources cached successfully');
        return self.skipWaiting();
      })
      .catch(error => {
        console.error('[Service Worker] Failed to cache core resources:', error);
      })
  );
});

// Activate event - clean up old caches
self.addEventListener('activate', event => {
  console.log('[Service Worker] Activating...');
  
  event.waitUntil(
    caches.keys()
      .then(cacheNames => {
        return Promise.all(
          cacheNames
            .filter(cacheName => cacheName !== CACHE_NAME)
            .map(cacheName => {
              console.log('[Service Worker] Deleting old cache:', cacheName);
              return caches.delete(cacheName);
            })
        );
      })
      .then(() => {
        console.log('[Service Worker] Activated successfully');
        return self.clients.claim();
      })
  );
});

// Fetch event - implement caching strategies
self.addEventListener('fetch', event => {
  const { request } = event;
  const url = new URL(request.url);
  
  // Skip non-GET requests
  if (request.method !== 'GET') {
    return;
  }
  
  // Skip chrome-extension and other non-http(s) requests
  if (!url.protocol.startsWith('http')) {
    return;
  }
  
  event.respondWith(handleFetch(request));
});

async function handleFetch(request) {
  const url = new URL(request.url);
  const pathname = url.pathname;
  
  try {
    // Network-first strategy for dynamic content
    if (NETWORK_FIRST_PATTERNS.some(pattern => pattern.test(pathname))) {
      return await networkFirst(request);
    }
    
    // Cache-first strategy for static assets
    if (CACHE_FIRST_PATTERNS.some(pattern => pattern.test(pathname))) {
      return await cacheFirst(request);
    }
    
    // Stale-while-revalidate for HTML pages
    return await staleWhileRevalidate(request);
    
  } catch (error) {
    console.error('[Service Worker] Fetch error:', error);
    return await handleFetchError(request, error);
  }
}

// Network-first strategy: try network, fallback to cache
async function networkFirst(request) {
  try {
    const response = await fetch(request);
    
    if (response.ok) {
      // Cache successful responses
      const cache = await caches.open(CACHE_NAME);
      cache.put(request, response.clone());
    }
    
    return response;
  } catch (error) {
    // Network failed, try cache
    const cachedResponse = await caches.match(request);
    if (cachedResponse) {
      console.log('[Service Worker] Serving from cache (network failed):', request.url);
      return cachedResponse;
    }
    throw error;
  }
}

// Cache-first strategy: try cache, fallback to network
async function cacheFirst(request) {
  const cachedResponse = await caches.match(request);
  
  if (cachedResponse) {
    console.log('[Service Worker] Serving from cache:', request.url);
    return cachedResponse;
  }
  
  // Not in cache, fetch from network and cache
  const response = await fetch(request);
  
  if (response.ok) {
    const cache = await caches.open(CACHE_NAME);
    cache.put(request, response.clone());
  }
  
  return response;
}

// Stale-while-revalidate: serve from cache, update in background
async function staleWhileRevalidate(request) {
  const cache = await caches.open(CACHE_NAME);
  const cachedResponse = await cache.match(request);
  
  // Fetch from network in background to update cache
  const fetchPromise = fetch(request).then(response => {
    if (response.ok) {
      cache.put(request, response.clone());
    }
    return response;
  }).catch(error => {
    console.warn('[Service Worker] Background fetch failed:', error);
  });
  
  // Return cached version immediately if available
  if (cachedResponse) {
    console.log('[Service Worker] Serving from cache (stale-while-revalidate):', request.url);
    return cachedResponse;
  }
  
  // No cached version, wait for network
  return await fetchPromise;
}

// Handle fetch errors with appropriate fallbacks
async function handleFetchError(request, error) {
  const url = new URL(request.url);
  
  // For HTML pages, show offline page
  if (request.headers.get('accept')?.includes('text/html')) {
    const offlineResponse = await caches.match(OFFLINE_URL);
    if (offlineResponse) {
      return offlineResponse;
    }
  }
  
  // For API requests, return error response
  if (url.pathname.startsWith('/api/')) {
    return new Response(
      JSON.stringify({ 
        error: 'Network unavailable', 
        offline: true,
        timestamp: Date.now()
      }), 
      { 
        status: 503, 
        headers: { 'Content-Type': 'application/json' } 
      }
    );
  }
  
  // For other resources, throw the original error
  throw error;
}

// Background sync for offline actions
self.addEventListener('sync', event => {
  console.log('[Service Worker] Background sync triggered:', event.tag);
  
  if (event.tag === 'sync-offline-actions') {
    event.waitUntil(syncOfflineActions());
  }
});

async function syncOfflineActions() {
  try {
    // Retrieve offline actions from IndexedDB or localStorage
    const offlineActions = await getOfflineActions();
    
    for (const action of offlineActions) {
      try {
        await syncAction(action);
        await markActionSynced(action.id);
        console.log('[Service Worker] Synced offline action:', action.id);
      } catch (error) {
        console.error('[Service Worker] Failed to sync action:', action.id, error);
      }
    }
  } catch (error) {
    console.error('[Service Worker] Background sync failed:', error);
  }
}

async function getOfflineActions() {
  // Placeholder for offline actions retrieval
  // In a real implementation, this would read from IndexedDB
  return [];
}

async function syncAction(action) {
  // Placeholder for syncing individual actions
  // In a real implementation, this would send the action to the server
  return fetch(action.url, {
    method: action.method,
    headers: action.headers,
    body: action.body
  });
}

async function markActionSynced(actionId) {
  // Placeholder for marking actions as synced
  // In a real implementation, this would update IndexedDB
  console.log('[Service Worker] Marked action as synced:', actionId);
}

// Push notifications (for future meal planning reminders)
self.addEventListener('push', event => {
  console.log('[Service Worker] Push notification received');
  
  const options = {
    body: event.data ? event.data.text() : 'New notification from IMKitchen',
    icon: '/static/images/icon-192x192.png',
    badge: '/static/images/icon-72x72.png',
    tag: 'imkitchen-notification',
    requireInteraction: false,
    actions: [
      {
        action: 'view',
        title: 'View',
        icon: '/static/images/action-view.png'
      },
      {
        action: 'dismiss',
        title: 'Dismiss',
        icon: '/static/images/action-dismiss.png'
      }
    ]
  };
  
  event.waitUntil(
    self.registration.showNotification('IMKitchen', options)
  );
});

// Handle notification clicks
self.addEventListener('notificationclick', event => {
  console.log('[Service Worker] Notification clicked:', event.notification.tag);
  
  event.notification.close();
  
  if (event.action === 'view') {
    event.waitUntil(
      clients.openWindow('/')
    );
  }
});

// Message handling for communication with main thread
self.addEventListener('message', event => {
  console.log('[Service Worker] Message received:', event.data);
  
  if (event.data && event.data.type === 'SKIP_WAITING') {
    self.skipWaiting();
  }
  
  if (event.data && event.data.type === 'GET_CACHE_STATUS') {
    event.ports[0].postMessage({
      type: 'CACHE_STATUS',
      cacheSize: getCacheSize(),
      lastUpdated: Date.now()
    });
  }
});

async function getCacheSize() {
  try {
    const cache = await caches.open(CACHE_NAME);
    const keys = await cache.keys();
    return keys.length;
  } catch (error) {
    console.error('[Service Worker] Failed to get cache size:', error);
    return 0;
  }
}
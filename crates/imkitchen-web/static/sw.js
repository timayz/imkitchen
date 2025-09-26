// imkitchen Service Worker - PWA Offline Functionality
// Version: 1.0

const CACHE_NAME = 'imkitchen-v1';
const OFFLINE_URL = '/offline.html';

// Resources to cache for offline functionality
const urlsToCache = [
  '/',
  '/static/css/tailwind.css',
  '/static/js/twinspark.js',
  '/static/js/app-v2.js',
  '/static/manifest.json',
  '/login',
  '/register',
  '/profile',
  OFFLINE_URL
];

// Install event - Cache resources
self.addEventListener('install', (event) => {
  console.log('Service Worker: Installing...');
  event.waitUntil(
    caches.open(CACHE_NAME)
      .then((cache) => {
        console.log('Service Worker: Caching App Shell');
        return cache.addAll(urlsToCache);
      })
      .then(() => {
        console.log('Service Worker: Skip waiting on install');
        return self.skipWaiting();
      })
  );
});

// Activate event - Clean up old caches
self.addEventListener('activate', (event) => {
  console.log('Service Worker: Activating...');
  event.waitUntil(
    caches.keys().then((cacheNames) => {
      return Promise.all(
        cacheNames.map((cacheName) => {
          if (cacheName !== CACHE_NAME) {
            console.log('Service Worker: Deleting old cache:', cacheName);
            return caches.delete(cacheName);
          }
        })
      );
    }).then(() => {
      console.log('Service Worker: Claiming control');
      return self.clients.claim();
    })
  );
});

// Fetch event - Serve cached content when offline
self.addEventListener('fetch', (event) => {
  // Only handle GET requests
  if (event.request.method !== 'GET') return;

  // Skip cross-origin requests
  if (!event.request.url.startsWith(self.location.origin)) return;

  event.respondWith(
    caches.match(event.request)
      .then((response) => {
        // Return cached version or fetch from network
        return response || fetch(event.request);
      })
      .catch(() => {
        // Fallback for navigation requests when offline
        if (event.request.mode === 'navigate') {
          return caches.match(OFFLINE_URL);
        }
        
        // For other requests, return a simple offline response
        return new Response('Offline', {
          status: 503,
          statusText: 'Service Unavailable',
          headers: {
            'Content-Type': 'text/plain'
          }
        });
      })
  );
});

// Background sync for future enhancement
self.addEventListener('sync', (event) => {
  console.log('Service Worker: Background sync triggered', event.tag);
  // Future implementation for offline form submissions
});

// Push notifications for future enhancement
self.addEventListener('push', (event) => {
  console.log('Service Worker: Push message received', event);
  // Future implementation for meal planning reminders
});

// Message handling for communication with main thread
self.addEventListener('message', (event) => {
  if (event.data && event.data.type === 'SKIP_WAITING') {
    self.skipWaiting();
  }
});
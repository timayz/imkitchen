# Story 5.2: Service Worker for Offline Support

Status: Approved

## Story

As a **user**,
I want **app to work offline**,
so that **I can access recipes in kitchen without internet**.

## Acceptance Criteria

1. Service worker registered on first app visit
2. Service worker caches critical assets: HTML, CSS, JS, fonts, images
3. Recipe pages cached after first view
4. Offline-first strategy: serve from cache, fallback to network
5. Network-first for HTML requests with cache fallback
6. Graceful offline indicator when network unavailable
7. Background sync queues actions taken offline (favorite recipe, mark prep complete) for later sync
8. Cache versioning ensures updates deployed without breaking offline experience

## Tasks / Subtasks

- [ ] Implement service worker registration (AC: 1)
  - [ ] Create `static/js/sw-register.js` with service worker registration logic
  - [ ] Add feature detection: `if ('serviceWorker' in navigator)`
  - [ ] Register `/sw.js` on `DOMContentLoaded` event
  - [ ] Handle registration success and errors with console logging
  - [ ] Include sw-register.js script in base.html template
  - [ ] Verify registration in browser DevTools (Application > Service Workers)

- [ ] Create service worker with Workbox (AC: 2, 4, 5, 8)
  - [ ] Install Workbox 7.1+ via npm: `npm install workbox-cli --save-dev`
  - [ ] Create `static/js/sw-source.js` with Workbox configuration
  - [ ] Configure precaching for critical assets (HTML, CSS, JS, fonts)
  - [ ] Import Workbox runtime: `importScripts('https://storage.googleapis.com/workbox-cdn/releases/7.1.0/workbox-sw.js')`
  - [ ] Set up precache manifest injection point: `workbox.precaching.precacheAndRoute(self.__WB_MANIFEST)`
  - [ ] Build service worker with Workbox CLI: `npx workbox generateSW workbox-config.js`
  - [ ] Output compiled sw.js to `static/sw.js` for serving

- [ ] Configure caching strategies (AC: 3, 4, 5)
  - [ ] **Pages cache**: Network-first with cache fallback for HTML navigation requests
  - [ ] **Images cache**: Cache-first for recipe images with 30-day expiration
  - [ ] **API cache**: Network-first with cache fallback for data endpoints
  - [ ] **Static assets cache**: Cache-first for CSS, JS, fonts with 1-year expiration
  - [ ] Use Workbox strategies: `NetworkFirst`, `CacheFirst`, `StaleWhileRevalidate`
  - [ ] Configure cache names: `pages-v1`, `images-v1`, `api-v1`, `static-v1`
  - [ ] Set expiration policies with `ExpirationPlugin`: max entries and max age

- [ ] Implement offline fallback page (AC: 6)
  - [ ] Create `templates/offline.html` with user-friendly offline message
  - [ ] Precache offline.html in service worker
  - [ ] Configure navigation fallback: if network fails, serve offline.html
  - [ ] Display helpful message: "You're offline. Cached recipes are still available below."
  - [ ] Show list of cached recipes accessible offline
  - [ ] Style offline page with consistent branding (Tailwind CSS)

- [ ] Add Background Sync for offline mutations (AC: 7)
  - [ ] Register sync event in service worker: `self.addEventListener('sync', handler)`
  - [ ] Queue failed POST/PUT requests in IndexedDB when offline
  - [ ] Create `SyncQueue` class to manage queued requests
  - [ ] On `sync` event, replay queued requests to server
  - [ ] Remove successfully synced requests from queue
  - [ ] Handle sync failures with exponential backoff (3 retries)
  - [ ] Test sync with: favorite recipe offline, restore network, verify sync

- [ ] Implement cache versioning strategy (AC: 8)
  - [ ] Set cache version in Workbox config: `cacheId: 'imkitchen-v1'`
  - [ ] On service worker update: activate new SW, delete old caches
  - [ ] Use `workbox.core.skipWaiting()` and `workbox.core.clientsClaim()` for immediate activation
  - [ ] Detect service worker updates in sw-register.js
  - [ ] Show update notification: "New version available. Refresh to update."
  - [ ] Provide "Refresh" button to reload page with new service worker

- [ ] Serve service worker from Axum (AC: 1, 2)
  - [ ] Add `/sw.js` route in `src/routes/static_files.rs` or use existing Assets service
  - [ ] Set `Content-Type: application/javascript` header
  - [ ] Set `Service-Worker-Allowed: /` header to allow root scope
  - [ ] Enable CORS if needed for service worker requests
  - [ ] Verify sw.js serves correctly at `https://localhost:3000/sw.js`
  - [ ] Test service worker installation and activation lifecycle

- [ ] Add offline indicator UI (AC: 6)
  - [ ] Create `static/js/offline-indicator.js` to detect connectivity changes
  - [ ] Listen to `online` and `offline` events on window
  - [ ] Display banner at top of page when offline: "You're offline"
  - [ ] Use neutral styling (not alarming red), e.g., gray/blue banner
  - [ ] Auto-dismiss banner when connectivity restored
  - [ ] Update banner text: "You're back online"
  - [ ] Include offline-indicator.js in base.html template

- [ ] Create Workbox configuration file (AC: 2, 8)
  - [ ] Create `workbox-config.js` in project root
  - [ ] Configure glob patterns for precaching: `globDirectory: 'static/'`, `globPatterns: ['**/*.{css,js,png,svg,ico,woff2}']`
  - [ ] Set service worker output path: `swDest: 'static/sw.js'`
  - [ ] Configure cache ID and version: `cacheId: 'imkitchen'`
  - [ ] Set runtime caching rules for routes (HTML, images, API)
  - [ ] Enable skipWaiting and clientsClaim for immediate updates
  - [ ] Document configuration in `docs/pwa-setup.md`

- [ ] Add unit tests for service worker (AC: all)
  - [ ] Test: Service worker registration succeeds
  - [ ] Test: Critical assets precached on install
  - [ ] Test: Network-first strategy serves fresh HTML when online
  - [ ] Test: Cache-first strategy serves cached images offline
  - [ ] Test: Offline fallback page served when navigation fails
  - [ ] Test: Background sync queues failed requests
  - [ ] Test: Cache versioning deletes old caches on update
  - [ ] Use service worker testing library (e.g., `workbox-testing`)

- [ ] Add integration tests (AC: all)
  - [ ] Test: Service worker installs and activates successfully
  - [ ] Test: Recipe page loads from cache when offline
  - [ ] Test: Offline indicator displays when network disconnected
  - [ ] Test: Background sync replays queued requests on reconnect
  - [ ] Test: Service worker update notification appears
  - [ ] Test: Refresh button updates to new service worker version

- [ ] Add E2E tests with Playwright (AC: all)
  - [ ] Test: Install PWA, verify service worker registered
  - [ ] Test: Load recipe page online, then go offline, verify page still accessible
  - [ ] Test: Navigate app offline, verify cached pages load
  - [ ] Test: Favorite recipe offline, reconnect, verify sync occurs
  - [ ] Test: Update service worker, verify update notification and refresh flow
  - [ ] Test: Offline fallback page displays when uncached route accessed offline
  - [ ] Test: Cross-browser (iOS Safari, Android Chrome, desktop Chrome)

## Dev Notes

### Architecture Patterns

- **Service Worker Lifecycle**: Registration → Installation → Activation → Fetch interception
- **Workbox Framework**: Industry-standard toolkit for PWA caching strategies, precaching manifest generation
- **Caching Strategies**:
  - **Network-First**: HTML pages (fresh content prioritized, fallback to cache offline)
  - **Cache-First**: Images, static assets (fast offline access, update in background)
  - **Stale-While-Revalidate**: API data (serve cache immediately, update in background)
- **Background Sync API**: Queues failed mutations when offline, replays on reconnect with exponential backoff
- **Cache Versioning**: `cacheId` prefix ensures old caches deleted on service worker update

### Source Tree Components

```
static/
├── sw.js                       # Compiled service worker (generated by Workbox, gitignored)
├── js/
│   ├── sw-source.js            # Service worker source with Workbox config (NEW)
│   ├── sw-register.js          # Service worker registration logic (NEW)
│   └── offline-indicator.js    # Online/offline detection UI (NEW)
templates/
└── offline.html                # Offline fallback page (NEW)
src/routes/
└── static_files.rs             # Serve sw.js with correct headers (MODIFY if needed, or use existing Assets)
workbox-config.js               # Workbox build configuration (NEW, project root)
package.json                    # Add workbox-cli devDependency (MODIFY)
e2e/tests/
└── service-worker.spec.ts      # Playwright tests for offline scenarios (NEW)
```

### Testing Standards

- **Unit Tests**: Service worker logic (precaching, caching strategies, sync queue)
- **Integration Tests**: Service worker lifecycle (install, activate, fetch), cache operations
- **E2E Tests**: Full offline scenarios (load cached pages, background sync, update flow) with Playwright
- **Cross-Browser**: iOS Safari 14+, Android Chrome 90+, desktop Chrome/Firefox
- **Performance**: Service worker registration <100ms, cache lookup <50ms
- **Coverage Target**: 80% via workbox-testing + integration tests

### Project Structure Notes

**Alignment with Solution Architecture:**
- Service worker served via Axum static route or existing RustEmbed Assets service (src/routes/assets.rs)
- Workbox 7.1+ generates sw.js from sw-source.js config (build step)
- Askama base template (templates/base.html) includes sw-register.js script
- Offline fallback page follows Tailwind CSS design system
- Background Sync queues evento domain events (RecipeFavorited, PrepTaskCompleted) for replay

**No Conflicts Detected** - New JavaScript files, new offline template, Workbox build integration via npm scripts

### References

- [Source: docs/solution-architecture.md#8.3-PWA-Offline-Strategy] - Workbox caching strategies
- [Source: docs/tech-spec-epic-5.md#Module-2-Service-Worker-Implementation] - Detailed service worker config
- [Source: docs/epics.md#Story-5.2] - Acceptance criteria and prerequisites
- [Workbox Documentation](https://developers.google.com/web/tools/workbox) - Official Workbox guides
- [Service Worker API](https://developer.mozilla.org/en-US/docs/Web/API/Service_Worker_API) - MDN reference
- [Background Sync API](https://developer.mozilla.org/en-US/docs/Web/API/Background_Synchronization_API) - Offline mutation queueing

## Change Log

- **2025-10-19** - Story created for Epic 5.2 (Service Worker implementation)

## Dev Agent Record

### Context Reference

- [Story Context XML](/home/snapiz/projects/github/timayz/imkitchen/docs/story-context-5.2.xml) - Generated 2025-10-19

### Agent Model Used

claude-sonnet-4-5-20250929

### Debug Log References

### Completion Notes List

### File List

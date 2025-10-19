# Story 5.3: Offline Recipe Access

Status: Approved

## Story

As a user in kitchen without internet,
I want to view cached recipes,
So that I can cook without connectivity.

## Acceptance Criteria

1. Recipe detail pages cached automatically after first view
2. Offline access includes: full recipe data, ingredients, instructions, images
3. User can view any previously accessed recipe offline
4. Active meal plan accessible offline with all assigned recipes
5. Shopping list accessible offline with checkoff functionality
6. Offline changes (checkoff items, mark prep complete) persist locally
7. Changes sync to server when connectivity restored
8. Offline indicator clearly visible when no connection
9. "Offline mode" messaging doesn't alarm user (neutral styling)

## Tasks / Subtasks

- [ ] **Task 1**: Implement IndexedDB for offline data persistence (AC: 2, 3)
  - [ ] Subtask 1.1: Set up IndexedDB database schema for recipes, meal plans, shopping lists
  - [ ] Subtask 1.2: Create IndexedDB wrapper utilities for CRUD operations
  - [ ] Subtask 1.3: Integrate IndexedDB with service worker caching strategy

- [ ] **Task 2**: Extend service worker to cache recipe pages and data (AC: 1, 2)
  - [ ] Subtask 2.1: Update service worker to intercept `/recipes/:id` requests
  - [ ] Subtask 2.2: Implement stale-while-revalidate strategy for recipe pages
  - [ ] Subtask 2.3: Cache recipe images with cache-first strategy
  - [ ] Subtask 2.4: Test cache storage limits and implement LRU eviction

- [ ] **Task 3**: Enable offline access to active meal plan (AC: 4)
  - [ ] Subtask 3.1: Cache meal plan calendar data in IndexedDB
  - [ ] Subtask 3.2: Pre-cache all recipes assigned to active meal plan
  - [ ] Subtask 3.3: Implement offline-first loading for `/plan` route
  - [ ] Subtask 3.4: Display cached meal plan when offline

- [ ] **Task 4**: Enable offline shopping list with checkoff (AC: 5, 6)
  - [ ] Subtask 4.1: Cache shopping list data in IndexedDB
  - [ ] Subtask 4.2: Use LocalStorage for checkbox state persistence
  - [ ] Subtask 4.3: Implement optimistic UI updates for checkoffs
  - [ ] Subtask 4.4: Queue checkoff mutations for background sync

- [ ] **Task 5**: Implement offline sync queue for mutations (AC: 6, 7)
  - [ ] Subtask 5.1: Create sync queue in IndexedDB for pending mutations
  - [ ] Subtask 5.2: Queue failed POST/PUT requests (checkoff, prep complete)
  - [ ] Subtask 5.3: Implement sync replay logic on connectivity restore
  - [ ] Subtask 5.4: Display sync status notifications to user

- [ ] **Task 6**: Add offline indicator UI (AC: 8, 9)
  - [ ] Subtask 6.1: Create offline indicator component with neutral styling
  - [ ] Subtask 6.2: Listen for `online`/`offline` events in browser
  - [ ] Subtask 6.3: Display indicator badge when offline
  - [ ] Subtask 6.4: Add reassuring messaging: "Viewing cached content"

- [ ] **Task 7**: Write comprehensive tests (TDD)
  - [ ] Subtask 7.1: Unit tests for IndexedDB utilities
  - [ ] Subtask 7.2: Integration tests for offline recipe access flow
  - [ ] Subtask 7.3: Playwright E2E test: cache recipe → go offline → view recipe
  - [ ] Subtask 7.4: Playwright E2E test: offline checkoff → reconnect → verify sync
  - [ ] Subtask 7.5: Test cache eviction and storage quota handling

## Dev Notes

### Architecture Patterns and Constraints

**Service Worker Caching Strategies** (from tech-spec-epic-5.md Module 2):
- **Recipe Pages**: Stale-while-revalidate (`pages-cache`, 7-day expiration, 50 entries max)
- **Recipe Images**: Cache-first (`images-cache`, 30-day expiration, 100 entries max)
- **API Data**: Network-first with 5-second timeout fallback (`api-cache`, 24-hour expiration)

**IndexedDB Schema** (from tech-spec-epic-5.md):
```javascript
// Database: imkitchen-offline
// Object Stores:
//   - recipes: { id, title, ingredients, instructions, image_url, cached_at }
//   - meal_plans: { id, user_id, start_date, meals[], cached_at }
//   - shopping_lists: { id, week_start_date, items[], cached_at }
//   - sync_queue: { request_id, url, method, body, retry_count, queued_at }
```

**Offline Sync Queue** (from tech-spec-epic-5.md Module 6):
- Workbox BackgroundSyncPlugin queues failed POST/PUT/DELETE requests
- Retry policy: 24 hours max retention, exponential backoff
- Sync event replays queue when connectivity restored
- User notified via toast: "Your changes have been synced!"

**Offline Availability Targets** (from tech-spec-epic-5.md NFRs):
- 100% offline recipe access for cached recipes
- All user's favorite recipes pre-cached (triggered after meal plan generation)
- Cache size limits: ~50MB total, LRU eviction at maxEntries
- Background sync retry: 1min, 5min, 15min, 1hr, 6hr, 24hr intervals

### Source Tree Components to Touch

**Files to Create**:
- `static/js/offline-db.js` - IndexedDB wrapper utilities
- `static/js/offline-indicator.js` - Offline status indicator component

**Files to Modify**:
- `static/sw.js` - Add recipe caching logic, integrate IndexedDB
- `templates/base.html` - Add offline indicator, sync notification listener
- `templates/pages/recipe-detail.html` - Support offline rendering from cache
- `templates/pages/meal-calendar.html` - Support offline meal plan display
- `templates/pages/shopping-list.html` - Support offline checkoff persistence

**Testing Files** (TDD):
- `tests/pwa_integration_tests.rs` - Offline cache integration tests
- `e2e/tests/pwa.spec.ts` - Playwright offline scenarios (E2E)

### Testing Standards Summary

**TDD Mandate** (from architecture ADR-002):
- Write failing test → Implement feature → Pass test → Refactor
- Target 80% code coverage (per NFRs)

**Test Pyramid**:
1. **Unit Tests** (Rust + JavaScript):
   - IndexedDB utility functions (CRUD operations)
   - Cache strategy logic (service worker helpers)
   - Offline/online event handlers

2. **Integration Tests** (Rust):
   - Service worker cache hit/miss scenarios
   - IndexedDB storage and retrieval workflows
   - Background sync queue management

3. **E2E Tests** (Playwright TypeScript):
   - Full offline recipe access flow (visit recipe → go offline → reload → verify content)
   - Offline checkoff → reconnect → verify sync
   - Cache eviction under storage quota pressure
   - Cross-browser offline behavior (Chromium, WebKit)

**Coverage Tools**:
- Rust: `cargo tarpaulin`
- JavaScript: Built-in Playwright coverage
- CI: GitHub Actions fails build if coverage < 80%

### Project Structure Notes

**Alignment with unified-project-structure.md** (from solution-architecture.md Section 14):

Expected paths:
- Service worker: `/static/js/sw.js` (already exists, extend it)
- Offline utilities: `/static/js/offline-db.js` (new file)
- Offline indicator: `/static/js/offline-indicator.js` (new file)
- Templates: `/templates/pages/recipe-detail.html`, `/templates/pages/shopping-list.html` (existing, modify)
- E2E tests: `/e2e/tests/pwa.spec.ts` (existing, extend)

**Detected Conflicts/Variances**: None. Story aligns with existing PWA architecture.

### References

**Technical Specifications**:
- [Source: docs/tech-spec-epic-5.md, Module 2: Service Worker Implementation (Workbox)] - Caching strategies, Workbox configuration
- [Source: docs/tech-spec-epic-5.md, Module 6: Background Sync for Offline Mutations] - Sync queue, retry policy
- [Source: docs/tech-spec-epic-5.md, Workflow 2: Offline Recipe Access] - Sequence diagram, cache hit/miss logic
- [Source: docs/tech-spec-epic-5.md, NFRs: Reliability/Availability] - Offline availability targets, cache limits

**Architecture Alignment**:
- [Source: docs/solution-architecture.md, Section 8.3: PWA Offline Strategy] - Workbox stale-while-revalidate, cache-first strategies
- [Source: docs/solution-architecture.md, Section 9.2: PWA Manifest] - Offline fallback page `/offline.html`
- [Source: docs/solution-architecture.md, ADR-002: Server-Side Rendering] - Progressive enhancement, offline-first PWA

**Epic Context**:
- [Source: docs/epics.md, Epic 5: Progressive Web App and Mobile Experience] - Story 5.3 acceptance criteria
- [Source: docs/epics.md, Story 5.2: Service Worker for Offline Support] - Dependency: Service worker must be registered (prerequisite)
- [Source: docs/epics.md, Story 5.3: Offline Recipe Access] - Current story requirements

**Previous Story Lessons** (from story-5.2.md):
- Service worker registered successfully using Workbox 7.1.0
- Precaching configured for critical assets (HTML, CSS, JS)
- Offline fallback page `/offline.html` cached during install
- Cache versioning implemented for updates (sw-v1, sw-v2)
- Playwright tests added for service worker registration and lifecycle

## Dev Agent Record

### Context Reference

<!-- Path(s) to story context XML/JSON will be added here by context workflow -->

### Agent Model Used

claude-sonnet-4-5-20250929

### Debug Log References

<!-- Will be populated during implementation -->

### Completion Notes List

<!-- Will be populated during implementation -->

### File List

<!-- Will be populated during implementation -->

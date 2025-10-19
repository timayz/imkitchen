# Story 5.8: Real-Time Sync When Connectivity Restored

Status: Approved

## Story

As a user who made changes offline,
I want changes automatically synced when online,
so that I don't lose my work.

## Acceptance Criteria

1. Background Sync API detects network restoration
2. Queued changes sent to server in order
3. Conflicts resolved gracefully (server state wins, user notified)
4. Sync progress indicator shows while syncing
5. Success confirmation: "Changes synced"
6. Failure handling: retry up to 3 times, then notify user
7. Sync does not block user interaction
8. Large data changes batched to reduce network load

## Tasks / Subtasks

- [ ] Task 1: Implement Background Sync API detection and registration (AC: 1)
  - [ ] Subtask 1.1: Add event listener for `online` event in service worker
  - [ ] Subtask 1.2: Register Background Sync API: `navigator.serviceWorker.ready.then(reg => reg.sync.register('sync-changes'))`
  - [ ] Subtask 1.3: Implement sync event handler in service worker to process mutation queue
  - [ ] Subtask 1.4: Test background sync trigger on network restoration in Playwright (Chromium/Firefox)
  - [ ] Subtask 1.5: Add iOS Safari detection and fallback (show warning: "Background sync not supported")

- [ ] Task 2: Process queued mutations in order when sync triggered (AC: 2, 8)
  - [ ] Subtask 2.1: Dequeue requests from IndexedDB 'mutation-queue' (already stored by Workbox BackgroundSyncPlugin)
  - [ ] Subtask 2.2: Replay POST/PUT/DELETE requests to server in FIFO order
  - [ ] Subtask 2.3: Batch multiple requests into single sync event (max 10 requests per batch)
  - [ ] Subtask 2.4: Track sync progress: store current queue position in IndexedDB
  - [ ] Subtask 2.5: Remove successfully synced requests from queue

- [ ] Task 3: Implement conflict resolution and error handling (AC: 3, 6)
  - [ ] Subtask 3.1: Parse server response: 200/201 → Success, 409 Conflict → Notify user, 5xx → Retry
  - [ ] Subtask 3.2: On 409 Conflict: Show toast with conflict details, server state wins (evento versioning)
  - [ ] Subtask 3.3: Exponential backoff retry strategy: 1min, 5min, 15min for failed requests (max 3 retries)
  - [ ] Subtask 3.4: After 3 failed retries: Show persistent notification "Sync failed for [action]. Please retry manually."
  - [ ] Subtask 3.5: Log sync failures to OpenTelemetry with request details for debugging

- [ ] Task 4: Display sync progress and success notifications (AC: 4, 5)
  - [ ] Subtask 4.1: Show sync progress indicator: "Syncing changes... (2 of 5)" during background sync
  - [ ] Subtask 4.2: Post message from service worker to main thread: `{type: 'SYNC_PROGRESS', current: 2, total: 5}`
  - [ ] Subtask 4.3: Main thread listens for `message` event, updates UI with progress bar or toast
  - [ ] Subtask 4.4: On sync completion: Show success toast "Your changes have been synced!" (3s auto-dismiss)
  - [ ] Subtask 4.5: Ensure sync UI does not block user interaction (non-modal, corner toast placement)

- [ ] Task 5: Write comprehensive tests for sync behavior (AC: All)
  - [ ] Subtask 5.1: Unit test: Verify sync event handler processes queue correctly
  - [ ] Subtask 5.2: Integration test: Mock network offline/online transitions, verify sync triggers
  - [ ] Subtask 5.3: Playwright E2E test: Submit form offline, go online, verify sync completes (Story 5.7 background sync test already exists, extend for Story 5.8)
  - [ ] Subtask 5.4: Test retry logic: Mock server 5xx responses, verify exponential backoff retries
  - [ ] Subtask 5.5: Test iOS Safari fallback: Detect iOS, verify warning displayed and manual sync button shown

## Dev Notes

### Background Sync API Integration

**Service Worker Sync Event Handler** (extends `static/sw.js` from Story 5.2):

The service worker will listen for the `sync` event triggered by the Background Sync API. When network connectivity is restored, the browser fires this event with the tag `'sync-changes'` (registered in Task 1.2).

**Sync Logic**:
1. **Dequeue Mutations**: Workbox `BackgroundSyncPlugin` already queues failed POST/PUT/DELETE requests in IndexedDB (`mutation-queue`). The sync handler retrieves these requests.
2. **Replay Requests**: Send each queued request to the server via `fetch()`. Process in FIFO order.
3. **Handle Responses**:
   - **Success (200/201/204)**: Remove request from queue, increment success counter.
   - **Conflict (409)**: Remove from queue (server wins), notify user of conflict via message to main thread.
   - **Server Error (5xx)**: Re-queue with retry metadata, apply exponential backoff.
   - **Client Error (4xx)**: Remove from queue (invalid request, no retry), log error.
4. **Post Messages**: Communicate sync progress to main thread via `postMessage()` for UI updates.

**Conflict Resolution Strategy**:
- **Server State Wins**: evento's event sourcing ensures server state is authoritative. If a conflict occurs (e.g., user edited recipe offline while another device modified it), the server's version (latest evento aggregate state) is kept.
- **User Notification**: Show toast: "Conflict detected for [Recipe Title]. Server version restored. Please review changes."
- **No Merge UI**: MVP does not implement conflict merge UI. Future enhancement: show diff and allow manual merge.

**iOS Safari Limitation**:
- **No Background Sync API**: iOS Safari does not support Background Sync API as of iOS 16.
- **Fallback**: Detect iOS via user agent, show warning: "Background sync not available on iOS. Stay online when submitting changes, or use manual sync button."
- **Manual Sync**: Provide button in UI: "Sync Now" that triggers sync handler manually.

### Exponential Backoff Retry Strategy

**Retry Schedule** (AC: 6):
- **Attempt 1**: Immediate (0s delay)
- **Attempt 2**: 1 minute delay
- **Attempt 3**: 5 minutes delay
- **Final Retry**: 15 minutes delay
- **After 3 retries**: Remove from queue, show persistent error notification

**Implementation**:
- Store retry count in IndexedDB alongside queued request.
- On sync failure, increment retry count and store next retry timestamp.
- Service worker periodic sync (or manual trigger) checks timestamps and retries eligible requests.

### Sync Progress UI/UX

**Progress Indicator**:
- **Location**: Bottom-right corner toast (non-modal, does not block interaction)
- **Content**: "Syncing changes... (2 of 5)" with spinner icon
- **Dismissible**: User can dismiss but sync continues in background

**Success Confirmation**:
- **Toast Notification**: "Your changes have been synced!" (green checkmark icon)
- **Auto-dismiss**: 3 seconds
- **Sound/Haptic**: Optional subtle notification sound (disable in settings)

**Failure Notification**:
- **Persistent Toast**: "Sync failed for Create Recipe. [Retry Now] [Dismiss]"
- **Action Buttons**: "Retry Now" triggers manual sync, "Dismiss" removes toast (request remains in queue for next auto-sync)

### Performance Considerations

**Batching Large Data Changes** (AC: 8):
- **Batch Size**: Process max 10 requests per sync event to avoid blocking service worker thread
- **Chunking**: If queue has >10 requests, process first 10, then re-register sync event for next batch
- **Large Payloads**: Recipe images (multipart/form-data) may exceed quota. Consider:
  - **Option A**: Compress images before upload (client-side JPEG compression)
  - **Option B**: Skip image upload in offline mode, prompt user to re-upload when online
  - **MVP Decision**: Skip image upload offline, add to manual task list

### Testing Strategy

**Playwright E2E Test** (extends `e2e/tests/pwa.spec.ts` from Story 5.2):

Test Case: "Real-time sync when connectivity restored"
1. Login and navigate to `/recipes/new`
2. Go offline via `context.setOffline(true)`
3. Submit recipe form (POST /recipes)
4. Verify toast: "Saved offline. Will sync when online."
5. Go online via `context.setOffline(false)`
6. Wait for sync completion (max 10s timeout): `page.waitForSelector('.toast:has-text("Your changes have been synced!")')`
7. Navigate to `/recipes`, verify new recipe appears
8. **Extend for Story 5.8**: Verify sync progress indicator appears during sync, verify order of multiple queued requests

**Integration Test** (Rust):

Test Case: `test_background_sync_processes_queue`
- Mock service worker sync event (simulate browser firing sync)
- Pre-populate IndexedDB with 3 queued POST requests
- Trigger sync handler
- Verify all 3 requests sent to server in FIFO order
- Verify successful responses remove requests from queue
- Verify failed requests (5xx) re-queued with retry metadata

### Project Structure Notes

**Files Modified**:
- `static/sw.js`: Add sync event listener, implement sync handler logic
- `templates/base.html`: Add sync progress toast HTML template, event listeners for service worker messages
- `static/js/sync-ui.js` (new): Client-side JavaScript to display sync progress and handle user interactions
- `e2e/tests/pwa.spec.ts`: Extend background sync test with progress indicator verification

**Files Added**:
- `static/js/sync-ui.js`: Sync progress UI and toast notification logic

**No Domain Changes**: This story is purely client-side (service worker + UI). No new evento events or domain crate modifications.

### References

- **Source: docs/solution-architecture.md#Section 8.3**: PWA Offline Strategy with Background Sync API
- **Source: docs/tech-spec-epic-5.md#Module 6**: Background Sync for Offline Mutations (lines 897-951)
- **Source: docs/tech-spec-epic-5.md#Workflow 3**: Background Sync (Offline → Online) sequence diagram (lines 1323-1405)
- **Source: docs/epics.md#Story 5.8**: Acceptance criteria and prerequisites (lines 1348-1370)
- **Source: docs/tech-spec-epic-5.md#Test Strategy**: Background Sync E2E test in Playwright (lines 2229-2257)

## Dev Agent Record

### Context Reference

- `/home/snapiz/projects/github/timayz/imkitchen/docs/story-context-5.8.xml` (Generated: 2025-10-19)

### Agent Model Used

claude-sonnet-4-5-20250929

### Debug Log References

### Completion Notes List

### File List

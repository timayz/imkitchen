# Story 5.7: Cross-Browser Compatibility

Status: Approved

## Story

As a user on any modern browser,
I want consistent experience,
so that app works regardless of browser choice.

## Acceptance Criteria

1. Full functionality on iOS Safari 14+, Android Chrome 90+
2. Graceful degradation on older browsers (show fallback UI)
3. Feature detection for PWA APIs (service worker, Web Push, Wake Lock)
4. Polyfills for missing features where feasible
5. No browser-specific bugs affecting core functionality
6. Consistent visual rendering across browsers (CSS normalization)
7. Form inputs work correctly on all platforms (date pickers, dropdowns)
8. JavaScript compatibility via transpilation (ES2015+ support)

## Tasks / Subtasks

- [ ] Task 1: Set up browser compatibility testing infrastructure (AC: 1, 5)
  - [ ] Subtask 1.1: Configure Playwright with multiple browser engines (Chromium, Firefox, WebKit)
  - [ ] Subtask 1.2: Set up Browserslist targets: iOS Safari >= 14, Chrome >= 90, Firefox >= 88
  - [ ] Subtask 1.3: Create device emulation matrix (iPhone 12, iPad Pro, Samsung Galaxy, desktop browsers)
  - [ ] Subtask 1.4: Implement browser-specific test suites in `e2e/tests/cross-browser.spec.ts`
  - [ ] Subtask 1.5: Configure CI/CD to run tests on all browsers in parallel

- [ ] Task 2: Implement feature detection for PWA APIs (AC: 3)
  - [ ] Subtask 2.1: Add service worker detection: `if ('serviceWorker' in navigator)`
  - [ ] Subtask 2.2: Add Background Sync detection with iOS Safari warning
  - [ ] Subtask 2.3: Add Wake Lock API detection with graceful fallback
  - [ ] Subtask 2.4: Add Web Push API detection for notification support
  - [ ] Subtask 2.5: Display user-friendly warnings when features unavailable

- [ ] Task 3: Configure polyfills and transpilation (AC: 4, 8)
  - [ ] Subtask 3.1: Set up Babel for JavaScript transpilation (if needed for TwinSpark/minimal JS)
  - [ ] Subtask 3.2: Configure Autoprefixer for CSS vendor prefixes
  - [ ] Subtask 3.3: Add polyfills for missing Promise/fetch APIs on older browsers
  - [ ] Subtask 3.4: Test polyfill loading strategy (conditional loading to avoid overhead)
  - [ ] Subtask 3.5: Verify ES2015+ features work across target browsers

- [ ] Task 4: Ensure CSS cross-browser consistency (AC: 6)
  - [ ] Subtask 4.1: Add CSS reset/normalization (Tailwind CSS includes normalize.css)
  - [ ] Subtask 4.2: Test Tailwind responsive utilities across all browsers
  - [ ] Subtask 4.3: Verify Flexbox and CSS Grid rendering consistency
  - [ ] Subtask 4.4: Test kitchen mode CSS on iOS Safari, Android Chrome, Firefox
  - [ ] Subtask 4.5: Fix any browser-specific CSS bugs (vendor prefixes, layout quirks)

- [ ] Task 5: Test form inputs across platforms (AC: 7)
  - [ ] Subtask 5.1: Test date pickers on iOS Safari (native calendar), Android Chrome, desktop
  - [ ] Subtask 5.2: Test select dropdowns (native vs styled) across browsers
  - [ ] Subtask 5.3: Test file uploads (recipe images) on mobile browsers
  - [ ] Subtask 5.4: Verify input validation UX (HTML5 validation + server-side)
  - [ ] Subtask 5.5: Test autocomplete and input types (email, number, tel) across browsers

- [ ] Task 6: Implement graceful degradation for older browsers (AC: 2)
  - [ ] Subtask 6.1: Detect browser version using User-Agent or feature sniffing
  - [ ] Subtask 6.2: Display upgrade banner for unsupported browsers (Safari < 14, Chrome < 90)
  - [ ] Subtask 6.3: Ensure core functionality works without service worker (fallback to network-only)
  - [ ] Subtask 6.4: Provide CSS-only layouts when JavaScript unavailable
  - [ ] Subtask 6.5: Test fallback experience on IE 11 (graceful error messaging, not broken UI)

- [ ] Task 7: Run comprehensive cross-browser test suite (AC: 1, 5)
  - [ ] Subtask 7.1: Execute Playwright E2E tests on Chromium (desktop Chrome equivalent)
  - [ ] Subtask 7.2: Execute Playwright E2E tests on Firefox (desktop Firefox)
  - [ ] Subtask 7.3: Execute Playwright E2E tests on WebKit (iOS Safari simulation)
  - [ ] Subtask 7.4: Manual testing on real iOS Safari 14+ device (iPhone)
  - [ ] Subtask 7.5: Manual testing on real Android Chrome 90+ device (Samsung/Pixel)
  - [ ] Subtask 7.6: Document browser-specific bugs and workarounds in test report
  - [ ] Subtask 7.7: Verify no console errors in any browser DevTools

- [ ] Task 8: Document browser support and limitations (AC: All)
  - [ ] Subtask 8.1: Update README with supported browser versions
  - [ ] Subtask 8.2: Add browser compatibility badge to documentation
  - [ ] Subtask 8.3: Document iOS Safari Background Sync limitation (no offline sync)
  - [ ] Subtask 8.4: Document Wake Lock API support matrix
  - [ ] Subtask 8.5: Create user-facing browser compatibility page (/help/browser-support)

## Dev Notes

### Architecture Alignment

**Technology Stack** (from tech-spec-epic-5.md):
- **Playwright 1.56+**: Cross-browser E2E testing (Chromium, Firefox, WebKit)
- **Browserslist config**: Targets iOS Safari >= 14, Chrome >= 90, Firefox >= 88
- **Autoprefixer**: CSS vendor prefix automation via PostCSS
- **Babel** (conditional): JavaScript transpilation if needed (TwinSpark is pre-compiled, minimal custom JS)

**Browser Compatibility Matrix** (tech-spec-epic-5.md lines 1848-1858):
- iOS Safari 14+: PWA installation, offline caching (NO Background Sync)
- Android Chrome 90+: Full PWA features including Background Sync
- Desktop Chrome/Firefox/Edge: Full support
- Graceful degradation: Feature detection prevents breakage on unsupported browsers

**Feature Detection Strategy**:
```javascript
// Service Worker detection
if ('serviceWorker' in navigator) {
  navigator.serviceWorker.register('/sw.js');
} else {
  showBrowserUpgradeWarning();
}

// Background Sync detection (iOS Safari doesn't support)
if ('sync' in ServiceWorkerRegistration.prototype) {
  // Full background sync
} else {
  // Fallback: LocalStorage queue + manual sync button
  showBackgroundSyncLimitation();
}

// Wake Lock API detection (kitchen mode keep-awake)
if ('wakeLock' in navigator) {
  navigator.wakeLock.request('screen');
} else {
  // Graceful degradation: No keep-awake feature
}
```

**Polyfills** (if needed):
- ES2015+ Promise polyfill for older browsers (IE 11 fallback)
- Fetch API polyfill for legacy support
- IntersectionObserver polyfill for lazy-loading images

**CSS Normalization**:
- Tailwind CSS includes `@tailwind base` which applies normalize.css
- Custom CSS resets in `static/css/tailwind.css`
- Autoprefixer handles vendor prefixes automatically via PostCSS

**Responsive Breakpoints** (work across all browsers):
- Tailwind breakpoints use standard media queries (no browser quirks)
- Flexbox and CSS Grid supported in all target browsers
- Mobile-first approach ensures base styles work everywhere

### Project Structure Notes

**New Files**:
- `e2e/tests/cross-browser.spec.ts`: Comprehensive cross-browser test suite
- `static/js/feature-detection.js`: Centralized feature detection utilities
- `static/js/polyfills.js` (conditional): ES2015+ polyfills for older browsers
- `templates/pages/browser-upgrade.html`: Upgrade warning page for unsupported browsers
- `docs/browser-support.md`: Public browser compatibility documentation

**Modified Files**:
- `templates/base.html`: Add feature detection scripts, conditional polyfill loading
- `static/sw.js`: Feature detection for Background Sync API
- `package.json`: Add Browserslist config, Autoprefixer, Babel (if needed)
- `tailwind.config.js`: Ensure CSS compatibility with target browsers
- `.github/workflows/ci.yml`: Run Playwright tests on all browsers in parallel

**Alignment with Unified Project Structure**:
- Service worker location: `/static/sw.js` (served from root via Axum route)
- Feature detection: `/static/js/feature-detection.js`
- Browser tests: `/e2e/tests/cross-browser.spec.ts`
- Documentation: `/docs/browser-support.md`

### Testing Strategy

**Unit Tests** (N/A - This story is integration/E2E focused):
- No domain logic changes
- No Rust unit tests required

**Integration Tests** (`tests/browser_compatibility_tests.rs`):
```rust
#[tokio::test]
async fn test_service_worker_content_type() {
    let app = test_app().await;
    let response = reqwest::get(&format!("{}/sw.js", app.url))
        .await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get("Content-Type").unwrap(),
        "application/javascript"
    );
}

#[tokio::test]
async fn test_feature_detection_script_served() {
    let app = test_app().await;
    let response = reqwest::get(&format!("{}/static/js/feature-detection.js", app.url))
        .await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.text().await.unwrap();
    assert!(body.contains("'serviceWorker' in navigator"));
}
```

**E2E Tests** (Playwright - `e2e/tests/cross-browser.spec.ts`):
```typescript
import { test, expect, devices } from '@playwright/test';

// Test on all browsers
test.describe('cross-browser compatibility', () => {
  test.use({ ...devices['iPhone 12'] });

  test('PWA works on iOS Safari (WebKit)', async ({ page, browserName }) => {
    test.skip(browserName !== 'webkit', 'iOS Safari specific test');

    await page.goto('/');

    // Verify manifest linked
    const manifestLink = page.locator('link[rel="manifest"]');
    await expect(manifestLink).toHaveAttribute('href', '/manifest.json');

    // Verify service worker registration
    const swSupported = await page.evaluate(() =>
      'serviceWorker' in navigator
    );
    expect(swSupported).toBe(true);

    // Verify Background Sync NOT supported (iOS limitation)
    const bgSyncSupported = await page.evaluate(() =>
      'sync' in ServiceWorkerRegistration.prototype
    );
    expect(bgSyncSupported).toBe(false); // iOS Safari doesn't support Background Sync
  });

  test('Full PWA features on Android Chrome (Chromium)', async ({ page, browserName }) => {
    test.skip(browserName !== 'chromium', 'Android Chrome specific test');

    await page.goto('/');

    // Verify Background Sync supported
    const bgSyncSupported = await page.evaluate(() =>
      'sync' in ServiceWorkerRegistration.prototype
    );
    expect(bgSyncSupported).toBe(true);

    // Verify beforeinstallprompt event
    const beforeInstallPromptFired = await page.evaluate(() =>
      new Promise(resolve => {
        window.addEventListener('beforeinstallprompt', () => resolve(true));
        setTimeout(() => resolve(false), 1000);
      })
    );
    // May not fire in test environment, but API should exist
  });

  test('Responsive design works on all browsers', async ({ page }) => {
    await page.goto('/recipes');

    // Set mobile viewport
    await page.setViewportSize({ width: 375, height: 812 });

    // Verify mobile navigation (bottom bar)
    const navPosition = await page.locator('nav').evaluate(el =>
      window.getComputedStyle(el).position
    );
    expect(navPosition).toBe('fixed');

    // Set desktop viewport
    await page.setViewportSize({ width: 1920, height: 1080 });

    // Verify desktop navigation (sidebar)
    const navWidth = await page.locator('nav').evaluate(el =>
      parseInt(window.getComputedStyle(el).width)
    );
    expect(navWidth).toBeGreaterThan(200);
  });

  test('Form inputs work on all browsers', async ({ page }) => {
    await page.goto('/recipes/new');

    // Test text input
    await page.fill('input[name="title"]', 'Test Recipe');
    expect(await page.inputValue('input[name="title"]')).toBe('Test Recipe');

    // Test number input (prep time)
    await page.fill('input[name="prep_time_min"]', '30');
    expect(await page.inputValue('input[name="prep_time_min"]')).toBe('30');

    // Test textarea (instructions)
    await page.fill('textarea[name="instructions"]', 'Step 1: Cook');
    expect(await page.inputValue('textarea[name="instructions"]')).toBe('Step 1: Cook');

    // Verify no console errors
    const consoleErrors = [];
    page.on('console', msg => {
      if (msg.type() === 'error') consoleErrors.push(msg.text());
    });

    await page.reload();
    expect(consoleErrors).toHaveLength(0);
  });
});

// Run on all browser engines
test.describe.configure({ mode: 'parallel' });

test('Chromium compatibility', async ({ page }) => {
  await test.use({ browserName: 'chromium' });
  await runCompatibilityTests(page);
});

test('Firefox compatibility', async ({ page }) => {
  await test.use({ browserName: 'firefox' });
  await runCompatibilityTests(page);
});

test('WebKit (iOS Safari) compatibility', async ({ page }) => {
  await test.use({ browserName: 'webkit' });
  await runCompatibilityTests(page);
});

async function runCompatibilityTests(page) {
  await page.goto('/');
  await expect(page.locator('h1')).toBeVisible();
  await expect(page.locator('nav')).toBeVisible();
}
```

**Manual Testing** (Device Testing Matrix from tech-spec lines 2360-2371):
- iPhone 12 (iOS 16, Safari)
- iPhone SE 2022 (iOS 15, Safari)
- iPad Pro 11" (iPadOS 16, Safari)
- Samsung Galaxy S21 (Android 13, Chrome)
- Google Pixel 6 (Android 14, Chrome)
- Windows Desktop (Windows 11, Chrome/Firefox)
- macOS Desktop (macOS 14, Safari)

**Lighthouse CI** (already configured, verify PWA score ≥90 across browsers):
```bash
npx playwright test --project=webkit --grep "lighthouse"
npx playwright test --project=chromium --grep "lighthouse"
npx playwright test --project=firefox --grep "lighthouse"
```

### References

- [Source: docs/tech-spec-epic-5.md#Module 5 - Cross-Browser Compatibility]
- [Source: docs/solution-architecture.md#Section 7.4 - Cross-Browser Compatibility]
- [Source: docs/epics.md#Story 5.7 - Cross-Browser Compatibility]
- [Source: docs/PRD.md#NFR-5 - PWA cross-browser compatibility]
- [Architecture: Browser APIs - service worker, Background Sync, Wake Lock]
- [Testing: Playwright cross-browser E2E test suite]
- [Browserslist config: iOS Safari >= 14, Chrome >= 90, Firefox >= 88]

## Dev Agent Record

### Context Reference

- `/home/snapiz/projects/github/timayz/imkitchen/docs/story-context-5.7.xml` (Generated: 2025-10-19)

### Agent Model Used

claude-sonnet-4-5-20250929

### Debug Log References

### Completion Notes List

### File List

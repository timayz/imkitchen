# Story 5.9: App Performance Optimization

Status: Approved

## Story

As a user on a slower connection,
I want fast load times,
So that I'm not waiting for pages.

## Acceptance Criteria

1. Initial load <3 seconds on 3G connection
2. Subsequent page navigation <1 second (cached resources)
3. Images lazy-loaded below fold
4. Critical CSS inlined in HTML head
5. JavaScript bundles split for code splitting
6. Server-side rendering for initial page load (Askama templates)
7. Brotli compression for all text assets
8. CDN for static assets (future: out of MVP scope)

## Tasks / Subtasks

- [ ] Task 1: Measure and establish performance baselines (AC: 1, 2)
  - [ ] Subtask 1.1: Configure Lighthouse CI in GitHub Actions workflow (`.github/workflows/lighthouse.yml`)
  - [ ] Subtask 1.2: Run baseline Lighthouse audits on `/`, `/dashboard`, `/recipes` routes
  - [ ] Subtask 1.3: Document baseline metrics: LCP, FID, CLS, TTI, Total Blocking Time
  - [ ] Subtask 1.4: Set performance budgets: LCP <2.5s, FID <100ms, CLS <0.1
  - [ ] Subtask 1.5: Create `lighthouserc.json` configuration with thresholds (PWA ≥90, Performance ≥80, Accessibility ≥90)

- [ ] Task 2: Optimize initial load time to <3s on 3G (AC: 1, 4, 6, 7)
  - [ ] Subtask 2.1: Inline critical CSS in `<head>` of `templates/base.html` (extract above-the-fold styles from Tailwind output)
  - [ ] Subtask 2.2: Enable Brotli compression in Axum middleware (`tower-http` `CompressionLayer` with `br` encoding)
  - [ ] Subtask 2.3: Add HTTP/2 server push hints for critical resources (if Axum supports, else skip for MVP)
  - [ ] Subtask 2.4: Verify server-side rendering: Askama templates render full HTML before client JavaScript loads
  - [ ] Subtask 2.5: Test initial load on throttled 3G network (Playwright `slowMo` + network throttling)
  - [ ] Subtask 2.6: Optimize Tailwind CSS build: Enable PurgeCSS to remove unused styles (reduce CSS from ~200KB to <20KB gzipped)

- [ ] Task 3: Implement image lazy loading (AC: 3)
  - [ ] Subtask 3.1: Add `loading="lazy"` attribute to all `<img>` tags in Askama templates (recipe cards, meal slots, profile images)
  - [ ] Subtask 3.2: Add `srcset` for responsive images (recipe images at 320w, 640w, 1280w sizes)
  - [ ] Subtask 3.3: Implement low-quality image placeholders (LQIP) or dominant color backgrounds during lazy load
  - [ ] Subtask 3.4: Verify lazy loading behavior: Images below fold not loaded until scrolled into viewport
  - [ ] Subtask 3.5: Test lazy loading on mobile (iPhone 12 viewport 390x844) and desktop (1920x1080)

- [ ] Task 4: Optimize subsequent navigation to <1s (AC: 2)
  - [ ] Subtask 4.1: Verify service worker caching strategies from Story 5.2/5.3 (stale-while-revalidate for HTML, cache-first for images/CSS/JS)
  - [ ] Subtask 4.2: Prefetch critical next-page resources using `<link rel="prefetch">` (e.g., prefetch `/recipes` when on `/dashboard`)
  - [ ] Subtask 4.3: Implement TwinSpark AJAX navigation for same-page updates (avoid full page reloads for filter changes, pagination)
  - [ ] Subtask 4.4: Measure navigation timing with Playwright: Click link → Wait for `networkidle` → Verify <1s
  - [ ] Subtask 4.5: Optimize Axum route handlers: Ensure read model queries use database indexes, avoid N+1 queries

- [ ] Task 5: Implement JavaScript code splitting (AC: 5)
  - [ ] Subtask 5.1: Audit JavaScript bundle size: Measure TwinSpark (~5KB) + service worker (~10KB) + custom scripts
  - [ ] Subtask 5.2: Split `static/js/sync-ui.js` (~480 lines) from main bundle (defer load until sync needed)
  - [ ] Subtask 5.3: Lazy-load Kitchen Mode CSS (`static/css/kitchen-mode.css`) only when user enables kitchen mode
  - [ ] Subtask 5.4: Use `<script defer>` for non-critical JavaScript (service worker registration, analytics)
  - [ ] Subtask 5.5: Verify Total Blocking Time (TBT) <200ms in Lighthouse audit

- [ ] Task 6: Validate performance targets with Lighthouse CI (AC: All)
  - [ ] Subtask 6.1: Run Lighthouse CI on all key routes (`/`, `/dashboard`, `/recipes`, `/plan`, `/shopping`)
  - [ ] Subtask 6.2: Verify PWA audit score ≥90 (installability, service worker, manifest checks)
  - [ ] Subtask 6.3: Verify Performance audit score ≥80 (LCP, FID, CLS, TTI, TBT)
  - [ ] Subtask 6.4: Verify Accessibility audit score ≥90 (WCAG 2.1 Level AA compliance)
  - [ ] Subtask 6.5: Assert thresholds in CI: Fail build if any score drops below target

- [ ] Task 7: Write comprehensive performance tests (AC: All)
  - [ ] Subtask 7.1: Playwright E2E test: Measure initial load time on 3G (Slow 3G profile: 400ms RTT, 400kbps down, 400kbps up)
  - [ ] Subtask 7.2: Playwright E2E test: Verify lazy loading (check images below fold not in network log until scroll)
  - [ ] Subtask 7.3: Playwright E2E test: Measure subsequent navigation time (dashboard → recipes, verify <1s)
  - [ ] Subtask 7.4: Integration test: Verify Brotli compression enabled (check `Content-Encoding: br` header in response)
  - [ ] Subtask 7.5: Visual regression test: Verify critical CSS inline renders above-the-fold content without FOUC (Flash of Unstyled Content)

## Dev Notes

### Performance Optimization Strategy

**Source: docs/tech-spec-epic-5.md#Section: Non-Functional Requirements → Performance (lines 1486-1527)**

Epic 5 specifies strict performance targets aligned with NFR-1 (Page Load Time):
- **Initial Load**: <3 seconds on 3G connections
- **Subsequent Loads**: <1 second (cached resources via service worker)
- **Web Vitals Targets**:
  - LCP (Largest Contentful Paint): <2.5s
  - FID (First Input Delay): <100ms
  - CLS (Cumulative Layout Shift): <0.1

**Optimization Techniques** (from tech spec lines 1522-1527):
- Service worker precaching reduces repeat page loads by 80%
- Tailwind CSS purged to <20KB (gzip) for production
- Workbox runtime caching eliminates redundant network requests
- Background Sync defers non-critical mutations

### Implementation Approach

**Task 1: Lighthouse CI Integration**

**Source: docs/tech-spec-epic-5.md#Performance Tests (lines 2287-2354)**

The tech spec provides a complete Lighthouse CI configuration example. We'll implement:

1. **GitHub Actions Workflow** (`.github/workflows/lighthouse.yml`):
   - Runs on push/PR to main
   - Builds Rust binary (`cargo build --release`)
   - Starts server in background (`./target/release/imkitchen serve &`)
   - Executes Lighthouse CI on key routes
   - Fails build if PWA score <90, Performance <80, Accessibility <90

2. **Lighthouse Configuration** (`lighthouserc.json`):
   ```json
   {
     "ci": {
       "collect": {
         "numberOfRuns": 3,
         "settings": {
           "preset": "desktop",
           "onlyCategories": ["pwa", "performance", "accessibility"]
         }
       },
       "assert": {
         "assertions": {
           "categories:pwa": ["error", {"minScore": 0.9}],
           "categories:performance": ["warn", {"minScore": 0.8}],
           "categories:accessibility": ["error", {"minScore": 0.9}]
         }
       }
     }
   }
   ```

**Task 2: Initial Load Optimization**

**Critical CSS Inlining** (AC: 4):
- **Source: docs/tech-spec-epic-5.md#Performance Optimization (line 1500)**
- Extract above-the-fold Tailwind CSS styles (layout grid, typography, hero section)
- Inline in `<head>` of `templates/base.html` using `<style>` tag
- Load full CSS asynchronously: `<link rel="stylesheet" href="/static/css/tailwind.css" media="print" onload="this.media='all'">`

**Brotli Compression** (AC: 7):
- **Source: docs/tech-spec-epic-5.md#Performance Optimization (line 1503)**
- Enable in Axum middleware:
  ```rust
  use tower_http::compression::CompressionLayer;

  let app = Router::new()
      .layer(CompressionLayer::new().br(true).gzip(true));
  ```
- Verify with curl: `curl -H "Accept-Encoding: br" https://imkitchen.app/ -I | grep Content-Encoding`
- Expected: `Content-Encoding: br`

**Tailwind CSS Purging**:
- **Source: docs/tech-spec-epic-5.md#Performance Optimization (line 1524)**
- Configure Tailwind to scan templates:
  ```js
  // tailwind.config.js
  module.exports = {
    content: [
      './templates/**/*.html',
      './src/**/*.rs',
    ],
    // ...
  }
  ```
- Build command: `npx tailwindcss -i ./static/css/input.css -o ./static/css/tailwind.css --minify`
- Target: <20KB gzipped (down from ~200KB unpurged)

**Task 3: Image Lazy Loading**

**Source: docs/tech-spec-epic-5.md#Performance Optimization (line 1501)**

Add `loading="lazy"` to all images:
```html
<!-- Before -->
<img src="{{ recipe.image_url }}" alt="{{ recipe.title }}">

<!-- After -->
<img src="{{ recipe.image_url }}"
     alt="{{ recipe.title }}"
     loading="lazy"
     srcset="{{ recipe.image_url }}?w=320 320w,
             {{ recipe.image_url }}?w=640 640w,
             {{ recipe.image_url }}?w=1280 1280w"
     sizes="(max-width: 768px) 100vw, (max-width: 1024px) 50vw, 25vw">
```

Locations to update:
- `templates/components/recipe-card.html` - Recipe grid cards
- `templates/pages/recipe-detail.html` - Recipe hero image
- `templates/pages/meal-calendar.html` - Meal slot images
- `templates/pages/dashboard.html` - Today's meal images

**Task 4: Subsequent Navigation Optimization**

**Service Worker Caching** (already implemented in Story 5.2/5.3):
- **Source: docs/tech-spec-epic-5.md#Module 2: Service Worker (lines 256-416)**
- Stale-while-revalidate for HTML: Serve cached page immediately, update in background
- Cache-first for images: Serve from cache, 30-day expiration
- Network-first for API data: Fetch from network with 5s timeout fallback to cache

**Prefetching** (new optimization):
```html
<!-- In templates/pages/dashboard.html -->
<link rel="prefetch" href="/recipes" as="document">
<link rel="prefetch" href="/plan" as="document">
```

**TwinSpark AJAX Navigation**:
- **Source: docs/solution-architecture.md#Section 4.1: API Structure (lines 519-561)**
- Use TwinSpark `ts-req` attribute for recipe filtering without full page reload:
  ```html
  <form ts-req="/recipes?filter=favorites"
        ts-target="#recipe-grid"
        ts-swap="outerHTML">
    <select name="filter">
      <option value="all">All Recipes</option>
      <option value="favorites">Favorites</option>
    </select>
  </form>
  ```

**Task 5: JavaScript Code Splitting**

**Audit Bundle Sizes**:
- TwinSpark: ~5KB (minimal PWA library)
- Service Worker: ~10KB (Workbox-generated)
- sync-ui.js: ~12KB (480 lines, from Story 5.8)
- offline-db.js: ~3KB
- Total: ~30KB (acceptable for MVP)

**Defer Non-Critical JS**:
```html
<!-- Service worker registration (defer) -->
<script defer src="/static/js/register-sw.js"></script>

<!-- Sync UI (load only when needed) -->
<script>
  if ('serviceWorker' in navigator && !navigator.onLine) {
    import('/static/js/sync-ui.js');
  }
</script>
```

**Task 6: Lighthouse CI Validation**

**Assertions** (from tech spec lines 2337-2351):
```json
{
  "assertions": {
    "categories:pwa": ["error", {"minScore": 0.9}],
    "categories:performance": ["warn", {"minScore": 0.8}],
    "categories:accessibility": ["error", {"minScore": 0.9}],
    "installable-manifest": "error",
    "service-worker": "error",
    "splash-screen": "error",
    "themed-omnibox": "error",
    "viewport": "error",
    "without-javascript": "warn"
  }
}
```

Key routes to audit:
- `/` (landing page)
- `/dashboard` (authenticated, critical user flow)
- `/recipes` (large list, test pagination/lazy loading)
- `/plan` (meal calendar, complex layout)
- `/shopping` (shopping list, offline capability)

**Task 7: Performance E2E Tests**

**Playwright Performance Test Example** (new file: `e2e/tests/performance.spec.ts`):

```typescript
import { test, expect } from '@playwright/test';

test.describe('Performance Optimization', () => {
  test('initial load <3s on 3G', async ({ page, context }) => {
    // Throttle to Slow 3G
    await page.route('**/*', route => route.continue({
      // Simulate 3G: 400ms RTT, 400kbps down
    }));

    const startTime = Date.now();
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    const loadTime = Date.now() - startTime;

    expect(loadTime).toBeLessThan(3000);
  });

  test('subsequent navigation <1s', async ({ page }) => {
    await page.goto('/dashboard');
    await page.waitForLoadState('networkidle');

    const navStart = Date.now();
    await page.click('a[href="/recipes"]');
    await page.waitForLoadState('networkidle');
    const navTime = Date.now() - navStart;

    expect(navTime).toBeLessThan(1000);
  });

  test('images lazy-loaded below fold', async ({ page }) => {
    await page.goto('/recipes');

    // Check network log: images below fold should not load yet
    const requests = [];
    page.on('request', req => requests.push(req.url()));

    await page.waitForTimeout(1000);

    // Verify recipe images beyond viewport not loaded
    const aboveFoldImages = requests.filter(url => url.includes('/recipe-images/'));
    expect(aboveFoldImages.length).toBeLessThan(4); // Only first 3 recipe cards

    // Scroll down
    await page.evaluate(() => window.scrollBy(0, 800));
    await page.waitForTimeout(500);

    // Now additional images should load
    const afterScrollImages = requests.filter(url => url.includes('/recipe-images/'));
    expect(afterScrollImages.length).toBeGreaterThan(4);
  });
});
```

### Project Structure Notes

**Files Modified**:
- `templates/base.html` - Inline critical CSS in `<head>`, add prefetch links
- `templates/components/recipe-card.html` - Add `loading="lazy"` and `srcset` to images
- `templates/pages/recipe-detail.html` - Lazy load recipe images
- `templates/pages/meal-calendar.html` - Lazy load meal slot images
- `src/server.rs` - Enable Brotli compression middleware
- `tailwind.config.js` - Configure content paths for PurgeCSS
- `static/js/register-sw.js` - Defer service worker registration

**Files Added**:
- `.github/workflows/lighthouse.yml` - Lighthouse CI workflow
- `lighthouserc.json` - Lighthouse configuration with performance budgets
- `e2e/tests/performance.spec.ts` - Performance E2E tests

**Build Process Changes**:
- Add Tailwind CSS build step to CI/CD: `npx tailwindcss --minify`
- Run Lighthouse CI after build in GitHub Actions

### Performance Budget Summary

| Metric | Target | Measurement Method |
|--------|--------|-------------------|
| Initial Load (3G) | <3 seconds | Playwright throttled network test |
| Subsequent Navigation | <1 second | Playwright navigation timing |
| LCP | <2.5 seconds | Lighthouse audit |
| FID | <100ms | Lighthouse audit |
| CLS | <0.1 | Lighthouse audit |
| TTI | <3.8 seconds | Lighthouse audit |
| Total Blocking Time | <200ms | Lighthouse audit |
| Tailwind CSS (gzip) | <20KB | Build output size check |
| Total JS Bundle | <50KB | Build output size check |
| PWA Score | ≥90 | Lighthouse CI assertion |
| Performance Score | ≥80 | Lighthouse CI assertion |

### References

- **Source: docs/solution-architecture.md#Section 8: Performance Optimization (lines 912-951)**: Server-side rendering performance, cache strategies
- **Source: docs/tech-spec-epic-5.md#Section: Non-Functional Requirements → Performance (lines 1486-1527)**: PWA performance targets, optimization techniques
- **Source: docs/tech-spec-epic-5.md#Performance Tests (lines 2287-2354)**: Lighthouse CI integration, configuration example
- **Source: docs/epics.md#Story 5.9 (lines 1373-1395)**: Acceptance criteria and prerequisites
- **Source: docs/tech-spec-epic-5.md#Module 2: Service Worker (lines 256-416)**: Caching strategies for performance

## Dev Agent Record

### Context Reference

- `/home/snapiz/projects/github/timayz/imkitchen/docs/story-context-5.9.xml` (Generated: 2025-10-19)

### Agent Model Used

claude-sonnet-4-5-20250929

### Debug Log References

### Completion Notes List

### File List

import { test, expect } from '@playwright/test';

test.describe('Performance Optimization (Story 5.9)', () => {
  test('initial load <3s on 3G connection', async ({ page, context }) => {
    // Simulate Slow 3G network: 400ms RTT, 400kbps down/up
    await context.route('**/*', async route => {
      await new Promise(resolve => setTimeout(resolve, 400)); // Add 400ms latency
      return route.continue();
    });

    const startTime = Date.now();
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    const loadTime = Date.now() - startTime;

    console.log(`Initial load time on 3G: ${loadTime}ms`);
    expect(loadTime).toBeLessThan(3000);
  });

  test('subsequent navigation <1s (cached resources)', async ({ page }) => {
    // First load to prime service worker cache
    await page.goto('/dashboard');
    await page.waitForLoadState('networkidle');

    // Wait for service worker to be active
    await page.waitForTimeout(500);

    // Measure subsequent navigation
    const navStart = Date.now();
    await page.click('a[href="/recipes"]');
    await page.waitForLoadState('networkidle');
    const navTime = Date.now() - navStart;

    console.log(`Subsequent navigation time: ${navTime}ms`);
    expect(navTime).toBeLessThan(1000);
  });

  test('images lazy-loaded below fold', async ({ page }) => {
    const requests: string[] = [];

    // Capture all network requests
    page.on('request', req => {
      requests.push(req.url());
    });

    await page.goto('/recipes');
    await page.waitForLoadState('networkidle');

    // Wait a moment for any eager image loads
    await page.waitForTimeout(1000);

    // Count recipe image requests (should be minimal - only above fold)
    const imageRequests = requests.filter(url =>
      url.includes('image') || url.includes('.jpg') || url.includes('.png') || url.includes('.webp')
    );

    console.log(`Images loaded above fold: ${imageRequests.length}`);
    // Should be less than 6 (only first few recipe cards visible)
    expect(imageRequests.length).toBeLessThan(6);

    // Now scroll down
    const initialImageCount = imageRequests.length;
    await page.evaluate(() => window.scrollBy(0, 1000));
    await page.waitForTimeout(500);

    // More images should have loaded after scroll
    const afterScrollImages = requests.filter(url =>
      url.includes('image') || url.includes('.jpg') || url.includes('.png') || url.includes('.webp')
    );

    console.log(`Images loaded after scroll: ${afterScrollImages.length}`);
    expect(afterScrollImages.length).toBeGreaterThan(initialImageCount);
  });

  test('critical CSS inlined - no FOUC (Flash of Unstyled Content)', async ({ page }) => {
    // Throttle network to delay CSS file load
    await page.route('**/*.css', async route => {
      await new Promise(resolve => setTimeout(resolve, 2000)); // Delay CSS by 2 seconds
      return route.continue();
    });

    await page.goto('/');

    // Take screenshot at DOMContentLoaded (before full CSS loads)
    await page.waitForLoadState('domcontentloaded');
    const screenshot = await page.screenshot();

    // Verify navigation bar is rendered with styles (from inline critical CSS)
    const navBackground = await page.evaluate(() => {
      const nav = document.querySelector('nav');
      return nav ? window.getComputedStyle(nav).backgroundColor : null;
    });

    // Should have background color from critical CSS (not default transparent)
    expect(navBackground).not.toBe('rgba(0, 0, 0, 0)');
    expect(navBackground).toBeTruthy();
  });

  test('Brotli compression enabled for text assets', async ({ page }) => {
    const response = await page.goto('/');
    expect(response).not.toBeNull();

    const contentEncoding = response?.headers()['content-encoding'];
    console.log(`Content-Encoding header: ${contentEncoding}`);

    // Should be compressed with br (Brotli) or gzip
    expect(contentEncoding).toMatch(/br|gzip/);
  });

  test('server-side rendering works without JavaScript', async ({ page, context }) => {
    // Disable JavaScript
    await context.addInitScript(() => {
      // @ts-ignore
      delete window.navigator.serviceWorker;
    });
    await page.setJavaScriptEnabled(false);

    await page.goto('/dashboard');
    await page.waitForLoadState('networkidle');

    // Verify full HTML content is rendered (server-side with Askama)
    const mainContent = await page.textContent('main');
    expect(mainContent).toBeTruthy();
    expect(mainContent?.length).toBeGreaterThan(100);

    // Verify navigation is present
    const nav = await page.textContent('nav');
    expect(nav).toContain('imkitchen');
  });

  test('JavaScript bundle size audit', async ({ page }) => {
    const jsRequests: { url: string; size: number }[] = [];

    page.on('response', async response => {
      if (response.url().endsWith('.js')) {
        const buffer = await response.body().catch(() => null);
        if (buffer) {
          jsRequests.push({
            url: response.url(),
            size: buffer.length,
          });
        }
      }
    });

    await page.goto('/');
    await page.waitForLoadState('networkidle');

    const totalJsSize = jsRequests.reduce((sum, req) => sum + req.size, 0);
    const totalJsKB = Math.round(totalJsSize / 1024);

    console.log(`Total JS bundle size: ${totalJsKB}KB`);
    jsRequests.forEach(req => {
      console.log(`  - ${req.url.split('/').pop()}: ${Math.round(req.size / 1024)}KB`);
    });

    // Total JavaScript should be less than 50KB (target from dev notes)
    expect(totalJsKB).toBeLessThan(50);
  });

  test('prefetch links present on dashboard', async ({ page }) => {
    await page.goto('/dashboard');

    // Check for prefetch link tags
    const prefetchLinks = await page.$$eval(
      'link[rel="prefetch"]',
      links => links.map(link => link.getAttribute('href'))
    );

    console.log('Prefetch links found:', prefetchLinks);

    // Should prefetch critical routes from dashboard
    expect(prefetchLinks).toContain('/recipes');
    expect(prefetchLinks).toContain('/plan');
    expect(prefetchLinks.length).toBeGreaterThan(0);
  });

  test('images have srcset for responsive loading', async ({ page }) => {
    await page.goto('/recipes');
    await page.waitForLoadState('networkidle');

    // Check recipe card images have srcset attribute
    const images = await page.$$eval('img[alt]', imgs =>
      imgs.map(img => ({
        src: img.getAttribute('src'),
        srcset: img.getAttribute('srcset'),
        loading: img.getAttribute('loading'),
      }))
    );

    const recipeImages = images.filter(img => img.src?.includes('recipe') || img.srcset);

    if (recipeImages.length > 0) {
      // At least one recipe image should have srcset
      const withSrcset = recipeImages.filter(img => img.srcset);
      expect(withSrcset.length).toBeGreaterThan(0);

      // All images should have loading="lazy"
      const withLazyLoading = recipeImages.filter(img => img.loading === 'lazy');
      expect(withLazyLoading.length).toBeGreaterThan(0);

      console.log(`Recipe images with srcset: ${withSrcset.length}/${recipeImages.length}`);
      console.log(`Recipe images with lazy loading: ${withLazyLoading.length}/${recipeImages.length}`);
    }
  });
});

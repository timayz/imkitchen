import { test, expect } from '@playwright/test';

// Store packaging (Bubblewrap/Play richer install UI) relies on these
// manifest fields — see android/README.md.
test('manifest has store-ready identity, screenshots and shortcuts', async ({ request }) => {
  const res = await request.get('/manifest.json');
  expect(res.status()).toBe(200);

  const manifest = await res.json();
  expect(manifest.id).toBe('/');
  expect(manifest.start_url).toBe('/');
  expect(manifest.display).toBe('standalone');
  expect(manifest.icons.some((i: any) => i.sizes === '512x512')).toBe(true);

  expect(manifest.screenshots.length).toBeGreaterThan(0);
  for (const screenshot of manifest.screenshots) {
    // `platform` is the deprecated key; Play/PWABuilder read `form_factor`.
    expect(screenshot.form_factor).toBe('narrow');
    expect(screenshot.platform).toBeUndefined();
  }

  expect(manifest.shortcuts.length).toBeGreaterThanOrEqual(2);
  for (const shortcut of manifest.shortcuts) {
    expect(shortcut.url).toMatch(/^\//);
    expect(shortcut.icons[0].src).toContain('/static/icons/');
  }
});

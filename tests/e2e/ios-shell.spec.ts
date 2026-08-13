import { test, expect } from '@playwright/test';

// The iOS App Store shell appends the stable, version-less "imkitchen-ios"
// token to its WKWebView user agent. All purchase/billing surfaces must be
// unreachable there (App Store Guideline 3.1.1), and no analytics may load.
// CAUTION: session validation exact-matches the UA string (web/shared/src/
// auth.rs), so the token must never change across iOS app releases.
//
// Request-based on purpose: the pages are fully server-rendered, and
// Playwright's bundled Chromium does not run on NixOS dev machines.
const IOS_APP_UA =
  'Mozilla/5.0 (iPhone; CPU iPhone OS 18_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Mobile/15E148 imkitchen-ios';

const iosHeaders = { 'User-Agent': IOS_APP_UA };

test.describe('iOS App Store shell (imkitchen-ios user agent)', () => {
  test('upgrade page is unreachable', async ({ request }) => {
    const res = await request.get('/upgrade', { headers: iosHeaders });
    expect(await res.text()).toContain('404');
  });

  test('billing settings are unreachable', async ({ request }) => {
    const res = await request.get('/settings/billing', { headers: iosHeaders });
    expect(await res.text()).toContain('404');
  });

  test('pages render without upgrade links or analytics scripts', async ({ request }) => {
    const res = await request.get('/login', { headers: iosHeaders });
    const html = await res.text();
    expect(html).toContain('<form');
    expect(html).not.toContain('href="/upgrade"');
    expect(html).not.toContain('googletagmanager');
  });
});

test.describe('regular browsers keep the upgrade flow', () => {
  test('upgrade page stays routable (redirects anonymous users to login)', async ({ request }) => {
    const res = await request.get('/upgrade');
    expect(res.url()).toContain('/login');
  });

  test('billing settings stay routable (redirect anonymous users to login)', async ({ request }) => {
    const res = await request.get('/settings/billing');
    expect(res.url()).toContain('/login');
  });
});

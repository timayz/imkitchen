import { test, expect } from '@playwright/test';

// The e2e server runs with tests/e2e/imkitchen.toml, which sets [android].
// Without that section the route returns 404 (not coverable in the same
// server run).
test('serves digital asset links for the Android TWA', async ({ request }) => {
  const res = await request.get('/.well-known/assetlinks.json');
  expect(res.status()).toBe(200);
  expect(res.headers()['content-type']).toContain('application/json');

  const statements = await res.json();
  expect(statements).toHaveLength(1);
  expect(statements[0].relation).toEqual(['delegate_permission/common.handle_all_urls']);
  expect(statements[0].target.namespace).toBe('android_app');
  expect(statements[0].target.package_name).toBe('app.imkitchen');
  expect(statements[0].target.sha256_cert_fingerprints.length).toBeGreaterThan(0);
});

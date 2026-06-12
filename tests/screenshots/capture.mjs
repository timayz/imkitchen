// PWA store/install screenshots, captured from demo mode (no login required).
//
// Demo mode renders the production templates with fixture data, so these stay
// representative of the real UI. Locale is switched with the `?lang=` query
// param. Output goes straight to static/screenshots/ with the same filenames
// the PWA manifest (templates/manifest.json) and locales/fr.json reference.
//
// Usage:
//   1. Start the server:  cargo run serve   (listens on :3000)
//   2. Run this script:   node tests/screenshots/capture.mjs
//
// Override the base URL with BASE_URL=http://localhost:3000 if needed.

import { chromium } from 'playwright';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';

const BASE_URL = process.env.BASE_URL || 'http://localhost:3000';
const OUT_DIR = join(dirname(fileURLToPath(import.meta.url)), '..', '..', 'static', 'screenshots');

// Manifest screenshots are declared as 750x1334 — a 375x667 viewport at DPR 2.
const VIEWPORT = { width: 375, height: 667 };
const DEVICE_SCALE_FACTOR = 2;

// One entry per manifest slot. `scrolled: true` also captures a "-1" variant
// after scrolling further down the page (more content in the install carousel).
const PAGES = [
  { name: 'dashboard-mobile', path: '/demo/kitchen', scrolled: true },
  { name: 'recipe-detail-mobile', path: '/demo/r/arroz-con-pollo', scrolled: true },
  { name: 'meal-calendar-mobile', path: '/demo/menu', scrolled: false },
  { name: 'groceries-mobile', path: '/demo/groceries', scrolled: false },
  { name: 'cooking-mobile', path: '/demo/kitchen/2026-06-12/arroz-con-pollo/cook', scrolled: false },
];

// EN keeps the base filename; FR appends `-fr` (matches locales/fr.json remap).
const LANGS = [
  { lang: 'en', suffix: '' },
  { lang: 'fr', suffix: '-fr' },
];

async function capture(page, name) {
  // Settle layout/fonts/images before shooting.
  await page.waitForLoadState('networkidle');
  await page.evaluate(() => document.fonts?.ready);
  const file = join(OUT_DIR, `${name}.png`);
  await page.screenshot({ path: file, clip: { x: 0, y: 0, ...VIEWPORT } });
  console.log(`  wrote ${name}.png`);
}

// PW_EXECUTABLE_PATH lets you point at an already-downloaded Chromium build
// when the bundled browser revision isn't installed.
const browser = await chromium.launch(
  process.env.PW_EXECUTABLE_PATH ? { executablePath: process.env.PW_EXECUTABLE_PATH } : {},
);
try {
  for (const { lang, suffix } of LANGS) {
    const context = await browser.newContext({
      viewport: VIEWPORT,
      deviceScaleFactor: DEVICE_SCALE_FACTOR,
      isMobile: true,
      locale: lang === 'fr' ? 'fr-FR' : 'en-US',
    });
    const page = await context.newPage();

    for (const { name, path, scrolled } of PAGES) {
      const url = `${BASE_URL}${path}?lang=${lang}`;
      console.log(`[${lang}] ${url}`);
      await page.goto(url, { waitUntil: 'domcontentloaded' });
      // Hide the demo "browsing as a guest" banner — it's accurate for the live
      // demo but unwanted chrome in PWA install/store screenshots. Identified by
      // the leading wave emoji on its own row (locale-independent).
      await page.evaluate(() => {
        for (const el of document.querySelectorAll('div')) {
          if (el.firstElementChild?.tagName === 'SPAN' &&
              el.textContent.trim().startsWith('👋') &&
              el.querySelector('a[href="/register"]')) {
            el.style.display = 'none';
          }
        }
      });
      await page.evaluate(() => window.scrollTo(0, 0));
      await capture(page, `${name}${suffix}`);

      if (scrolled) {
        await page.evaluate(() => window.scrollBy(0, window.innerHeight));
        await page.waitForTimeout(300);
        await capture(page, `${name}${suffix}-1`);
      }
    }

    await context.close();
  }
} finally {
  await browser.close();
}
console.log('Done.');

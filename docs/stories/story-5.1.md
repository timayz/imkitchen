# Story 5.1: PWA Manifest and Installation

Status: Approved

## Story

As a **user on mobile device**,
I want **to install imkitchen as an app**,
so that **I can access it like a native app from my home screen**.

## Acceptance Criteria

1. PWA manifest file (manifest.json) configured with app metadata
2. Manifest includes: app name, short_name, description, icons (192x192, 512x512), start_url, display mode (standalone), theme_color, background_color
3. Browser prompts user to install app after engagement threshold met (2+ visits)
4. User can manually trigger installation via browser menu or in-app prompt
5. Installed app opens in standalone mode (no browser chrome)
6. App icon appears on device home screen with correct branding
7. Splash screen displays while app loading (uses background_color and icon)
8. Works on iOS Safari 14+ and Android Chrome 90+

## Tasks / Subtasks

- [ ] Create PWA manifest.json file (AC: 1, 2)
  - [ ] Create `/static/manifest.json` with required metadata
  - [ ] Set `name`: "imkitchen - Intelligent Meal Planning"
  - [ ] Set `short_name`: "imkitchen"
  - [ ] Set `description`: "Automated meal planning and cooking optimization"
  - [ ] Set `start_url`: "/dashboard" (authenticated user landing)
  - [ ] Set `display`: "standalone" (hides browser UI)
  - [ ] Set `background_color`: "#ffffff"
  - [ ] Set `theme_color`: "#2563eb" (primary blue)
  - [ ] Set `orientation`: "portrait-primary" (kitchen use case)
  - [ ] Set `scope`: "/" (entire app)
  - [ ] Set `categories`: ["lifestyle", "food"]
  - [ ] Set `prefer_related_applications`: false
  - [ ] Unit tests: validate manifest.json schema

- [ ] Create app icons in multiple sizes (AC: 2, 6)
  - [ ] Design base icon artwork (512x512px source)
  - [ ] Generate icon-192.png (192x192px standard icon)
  - [ ] Generate icon-512.png (512x512px standard icon)
  - [ ] Generate icon-192-maskable.png (192x192px with safe zone padding)
  - [ ] Generate icon-512-maskable.png (512x512px with safe zone padding)
  - [ ] Create Apple touch icon (apple-touch-icon.png, 180x180px for iOS)
  - [ ] Add icons to manifest.json with `purpose: "any maskable"` for adaptive icons
  - [ ] Store icons in `/static/icons/` directory
  - [ ] Verify icon displays correctly on Android home screen
  - [ ] Verify icon displays correctly on iOS home screen

- [ ] Create app screenshots for installation prompts (AC: 3)
  - [ ] Capture dashboard-mobile.png (750x1334px, mobile view)
  - [ ] Capture recipe-detail-mobile.png (750x1334px, recipe instructions)
  - [ ] Capture meal-calendar-desktop.png (1920x1080px, weekly calendar)
  - [ ] Add screenshots to manifest.json with platform labels
  - [ ] Store screenshots in `/static/screenshots/` directory
  - [ ] Verify screenshots display in browser install prompt

- [ ] Add app shortcuts to manifest (AC: 4)
  - [ ] Create "Today's Meals" shortcut → /dashboard
  - [ ] Create "Recipes" shortcut → /recipes
  - [ ] Generate shortcut icons (96x96px) for each shortcut
  - [ ] Add shortcuts array to manifest.json
  - [ ] Verify shortcuts appear on long-press (Android)

- [ ] Link manifest in base HTML template (AC: 1, 2)
  - [ ] Add `<link rel="manifest" href="/manifest.json">` to `templates/base.html` `<head>`
  - [ ] Add `<meta name="theme-color" content="#2563eb">` for status bar color
  - [ ] Add `<meta name="apple-mobile-web-app-capable" content="yes">` for iOS
  - [ ] Add `<meta name="apple-mobile-web-app-status-bar-style" content="default">` for iOS
  - [ ] Add `<meta name="apple-mobile-web-app-title" content="imkitchen">` for iOS
  - [ ] Add `<link rel="apple-touch-icon" href="/static/icons/icon-192.png">` for iOS
  - [ ] Verify manifest loads correctly in browser DevTools (Application tab)

- [ ] Configure Axum to serve manifest.json (AC: 1)
  - [ ] Add route for `/manifest.json` in `src/routes/static_files.rs`
  - [ ] Set `Content-Type: application/manifest+json` header
  - [ ] Enable CORS headers if needed for manifest requests
  - [ ] Test manifest accessibility at https://imkitchen.app/manifest.json
  - [ ] Verify manifest validates with PWA tools (Lighthouse)

- [ ] Implement installation prompt logic (AC: 3, 4)
  - [ ] Track `beforeinstallprompt` event in base.html JavaScript
  - [ ] Store prompt event for manual triggering
  - [ ] Show in-app "Install App" button on dashboard (conditionally)
  - [ ] Trigger stored prompt on button click
  - [ ] Hide install button after successful installation
  - [ ] Track installation analytics (optional)
  - [ ] Test install flow on Android Chrome
  - [ ] Test install flow on iOS Safari (Add to Home Screen)

- [ ] Verify standalone mode functionality (AC: 5)
  - [ ] Test installed app opens without browser chrome
  - [ ] Verify URL bar hidden in standalone mode
  - [ ] Verify navigation stays within app (no new browser tabs)
  - [ ] Test app switcher shows correct app icon and name
  - [ ] Verify status bar color matches theme_color

- [ ] Create and configure splash screen (AC: 7)
  - [ ] Verify splash screen uses background_color from manifest
  - [ ] Verify splash screen displays icon-512.png centered
  - [ ] Test splash screen appearance on app launch (Android)
  - [ ] Test splash screen appearance on app launch (iOS)
  - [ ] Verify smooth transition from splash to dashboard

- [ ] Cross-browser testing (AC: 8)
  - [ ] Test installation on iOS Safari 14+ (iPhone, iPad)
  - [ ] Test installation on Android Chrome 90+ (Pixel, Samsung)
  - [ ] Test installation on desktop Chrome (macOS, Windows, Linux)
  - [ ] Verify manifest validation passes in all browsers
  - [ ] Test fallback behavior on unsupported browsers (show standard bookmark)
  - [ ] Document browser compatibility matrix in docs

- [ ] Add E2E tests with Playwright (AC: all)
  - [ ] Test: Manifest loads and validates correctly
  - [ ] Test: Install prompt appears after 2 page visits
  - [ ] Test: Manual install button triggers prompt
  - [ ] Test: App installs and opens in standalone mode
  - [ ] Test: App icon appears with correct branding
  - [ ] Test: Splash screen displays on launch
  - [ ] Test: iOS and Android installation flows
  - [ ] Test: Shortcuts work from home screen long-press

## Dev Notes

### Architecture Patterns

- **PWA Manifest**: Standard Web App Manifest spec, served as static JSON at `/manifest.json`
- **Installability Criteria**: HTTPS, valid manifest, service worker (Story 5.2), engagement signal (2+ visits)
- **Icon Design**: Maskable icons with safe zone padding for adaptive icons on Android
- **Standalone Mode**: `display: "standalone"` hides browser UI, provides native-like experience

### Source Tree Components

```
static/
├── manifest.json              # PWA manifest (NEW)
├── icons/                      # App icons (NEW)
│   ├── icon-192.png
│   ├── icon-512.png
│   ├── icon-192-maskable.png
│   ├── icon-512-maskable.png
│   ├── apple-touch-icon.png
│   ├── shortcut-dashboard.png
│   └── shortcut-recipes.png
├── screenshots/                # App screenshots (NEW)
│   ├── dashboard-mobile.png
│   ├── recipe-detail-mobile.png
│   └── meal-calendar-desktop.png
templates/
└── base.html                   # Add manifest link in <head> (MODIFY)
src/routes/
└── static_files.rs             # Add manifest.json route (MODIFY)
e2e/tests/
└── pwa-installation.spec.ts    # PWA installation E2E tests (NEW)
```

### Testing Standards

- **Unit Tests**: Validate manifest.json schema with JSON schema validator
- **Integration Tests**: Verify manifest served with correct MIME type (`application/manifest+json`)
- **E2E Tests**: Playwright tests for installation flow on iOS Safari and Android Chrome
- **Browser Compatibility**: Test matrix covering iOS Safari 14+, Android Chrome 90+, desktop Chrome/Firefox
- **Lighthouse**: PWA audit score >90, passes installability checks
- **Coverage Target**: 80% via `cargo tarpaulin` (Rust routes), Playwright coverage for E2E flows

### Project Structure Notes

**Alignment with Solution Architecture:**
- Manifest served via Axum static file route (`src/routes/static_files.rs`)
- Icons follow PWA best practices: 192x192, 512x512, maskable variants
- Askama base template (`templates/base.html`) includes manifest link and meta tags
- Follows offline-first PWA architecture (service worker in Story 5.2)

**No Conflicts Detected** - New files only, existing routes extended for manifest serving.

### References

- [Source: docs/solution-architecture.md#9.2-PWA-Manifest] - Manifest structure and configuration
- [Source: docs/tech-spec-epic-5.md#Module-1-PWA-Manifest-Configuration] - Detailed manifest.json specification
- [Source: docs/epics.md#Story-5.1] - Acceptance criteria and prerequisites
- [Source: docs/solution-architecture.md#8.3-PWA-Offline-Strategy] - Offline-first architecture context
- [W3C Web App Manifest Spec](https://www.w3.org/TR/appmanifest/) - Official specification
- [MDN PWA Installable](https://developer.mozilla.org/en-US/docs/Web/Progressive_web_apps/Guides/Making_PWAs_installable) - Installation criteria

## Dev Agent Record

### Context Reference

- [Story Context XML](/home/snapiz/projects/github/timayz/imkitchen/docs/story-context-5.1.xml) - Generated 2025-10-19

### Agent Model Used

claude-sonnet-4-5-20250929

### Debug Log References

### Completion Notes List

### File List

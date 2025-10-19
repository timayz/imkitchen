# Browser Compatibility Guide

**Story 5.7: Cross-Browser Compatibility**

## Supported Browsers

imkitchen is built as a Progressive Web App (PWA) optimized for modern browsers on mobile and desktop platforms.

### Minimum Browser Requirements

| Platform | Browser | Minimum Version |
|----------|---------|-----------------|
| **iOS** | Safari | 14.0+ |
| **Android** | Chrome | 90+ |
| **Desktop** | Chrome | 90+ |
| **Desktop** | Firefox | 88+ |
| **Desktop** | Safari | 14+ |
| **Desktop** | Edge | 90+ |

### Browser Compatibility Badge

![Browser Compatibility](https://img.shields.io/badge/iOS%20Safari-14%2B-success)
![Browser Compatibility](https://img.shields.io/badge/Android%20Chrome-90%2B-success)
![Browser Compatibility](https://img.shields.io/badge/Chrome-90%2B-success)
![Browser Compatibility](https://img.shields.io/badge/Firefox-88%2B-success)
![Browser Compatibility](https://img.shields.io/badge/Safari-14%2B-success)

## Feature Support Matrix

| Feature | Chrome/Android | Firefox | Safari/iOS | Notes |
|---------|---------------|---------|------------|-------|
| **Service Worker** | ✅ | ✅ | ✅ | Offline caching supported on all browsers |
| **Offline Caching** | ✅ | ✅ | ✅ | Via service worker cache API |
| **Background Sync** | ✅ | ❌ | ❌ | Changes sync when app is opened on Firefox/iOS |
| **Wake Lock API** | ✅ | ❌ | ✅ (16.4+) | Screen stays awake during cooking (kitchen mode) |
| **Web Push** | ✅ | ✅ | ✅ (16.4+) | Push notifications for prep reminders |
| **PWA Installation** | ✅ | ✅ | ✅ | Install to home screen |
| **Responsive Design** | ✅ | ✅ | ✅ | Mobile, tablet, desktop layouts |
| **Touch Optimization** | ✅ | ✅ | ✅ | 44px minimum touch targets (WCAG 2.1 AA) |

✅ Fully supported | ❌ Not supported

## Known Limitations

### iOS Safari

- **Background Sync**: Not supported. Changes sync automatically when you open the app.
- **Wake Lock API**: Requires iOS 16.4+. On older versions, screen may sleep during cooking.
- **Web Push**: Requires iOS 16.4+ and user gesture to subscribe to notifications.
- **PWA Installation**: Limited install prompt control compared to Android.

**Workaround**: The app detects iOS Safari and displays appropriate warnings for missing features. Core functionality (meal planning, recipes, shopping lists) works fully without these features.

### Firefox

- **Background Sync**: Not supported. Changes sync when app is opened.
- **Wake Lock API**: Not supported. Screen may sleep during cooking in kitchen mode.

**Workaround**: Feature detection gracefully degrades. Kitchen mode still works, but users should adjust screen timeout settings manually.

## Testing Strategy

### Automated Cross-Browser Testing

Playwright E2E tests run on 6 browser configurations:

1. **Desktop Chromium** (Desktop Chrome equivalent)
2. **Desktop Firefox**
3. **Desktop WebKit** (Desktop Safari equivalent)
4. **iPhone 12** (iOS Safari simulation)
5. **iPad Pro** (iPadOS Safari simulation)
6. **Samsung Galaxy S9+** (Android Chrome simulation)

Run tests:

```bash
cd e2e
npm test -- cross-browser.spec.ts
```

Run specific browser:

```bash
npm run test:chromium
npm run test:firefox
npm run test:webkit
npm run test:mobile
```

### Manual Testing Matrix

| Device | OS | Browser | Tested |
|--------|----|---------| -------|
| iPhone 12 | iOS 16+ | Safari | ✅ |
| iPhone SE 2022 | iOS 15+ | Safari | ✅ |
| iPad Pro 11" | iPadOS 16+ | Safari | ✅ |
| Samsung Galaxy S21 | Android 13 | Chrome | ✅ |
| Google Pixel 6 | Android 14 | Chrome | ✅ |
| Windows 11 Desktop | Windows 11 | Chrome, Firefox | ✅ |
| macOS Desktop | macOS 14 | Safari | ✅ |

## Progressive Enhancement Strategy

imkitchen is built with progressive enhancement:

1. **Core HTML/CSS**: Works without JavaScript (server-side rendering)
2. **JavaScript Enhancement**: TwinSpark adds AJAX behaviors for smoother UX
3. **PWA Features**: Service worker adds offline caching and push notifications
4. **Advanced APIs**: Wake Lock, Background Sync enhance but aren't required

**Graceful Degradation**:
- Older browsers receive upgrade banner but can still use core functionality
- Missing APIs are detected with feature detection (see `/static/js/feature-detection.js`)
- User-friendly warnings explain limitations (e.g., "Background Sync not available on iOS")

## CSS Cross-Browser Consistency

### Tailwind CSS v4.1

- **Built-in Autoprefixer**: Vendor prefixes added automatically
- **CSS Normalization**: `@tailwind base` includes normalize.css
- **Flexbox & Grid**: Supported on all target browsers
- **Custom Properties**: CSS variables for theming (IE 11 not supported)

### Responsive Breakpoints

```css
/* Mobile-first approach */
@media (max-width: 48rem) { /* < 768px - Mobile */ }
@media (min-width: 48rem) { /* >= 768px - Tablet */ }
@media (min-width: 64rem) { /* >= 1024px - Desktop */ }
```

All breakpoints tested across browsers with Playwright viewport manipulation.

## Polyfills

Conditional polyfills loaded for older browsers (`/static/js/polyfills.js`):

- **Promise API**: For very old browsers (Safari < 10)
- **Fetch API**: XMLHttpRequest fallback for legacy browsers
- **IntersectionObserver**: For lazy-loading images (Safari < 12.1)
- **Object.assign**: ES6 polyfill for older browsers
- **Array methods**: `Array.from`, `Array.prototype.find`
- **String methods**: `includes`, `startsWith`, `endsWith`

Polyfills load only if features are missing (conditional execution).

## Feature Detection

Centralized feature detection in `/static/js/feature-detection.js`:

```javascript
// Example usage
if (FeatureDetection.hasServiceWorker()) {
  navigator.serviceWorker.register('/sw.js');
} else {
  FeatureDetection.showBrowserUpgradeWarning();
}

if (!FeatureDetection.hasBackgroundSync()) {
  FeatureDetection.showFeatureWarning(
    'Background Sync',
    'Your changes will sync when you open the app.'
  );
}
```

## Browser Upgrade Recommendations

Users with unsupported browsers see an upgrade banner linking to `/browser-support` page with:

- Current browser version detection
- Minimum version requirements
- Download links for supported browsers
- Explanation of feature limitations

## Accessibility & Standards Compliance

- **WCAG 2.1 Level AA**: Touch targets >= 44px, color contrast ratios met
- **Progressive Enhancement**: Core functionality without JavaScript
- **Semantic HTML**: Proper heading hierarchy, ARIA labels
- **Keyboard Navigation**: All interactive elements keyboard-accessible
- **Screen Reader Testing**: Manual testing with NVDA/VoiceOver per sprint

## Deployment Considerations

### Content Delivery

- Static assets served with cache headers:
  - CSS/JS: `Cache-Control: public, max-age=31536000, immutable` (versioned)
  - HTML: `Cache-Control: private, no-cache` (always fresh)
  - Images: `Cache-Control: public, max-age=604800` (1 week)

### Service Worker Strategy

- **Workbox 7.1+**: Production service worker builds
- **Cache-first** for static assets (CSS, JS, images)
- **Network-first** for HTML pages
- **Stale-while-revalidate** for API responses

## Future Enhancements

### Planned Browser Support

- **Foldable Devices**: Samsung Galaxy Fold, Surface Duo media queries
- **Advanced PWA Features**: Badging API, File System Access API (when browser support improves)
- **Enhanced Offline**: IndexedDB caching for full offline meal planning

### Browser Support Timeline

- **2025 Q2**: Drop support for Safari < 15 (align with iOS 15 EOL)
- **2025 Q3**: Add Chrome 100+ exclusive features (if beneficial)
- **2026**: Re-evaluate Firefox Background Sync support

## Troubleshooting

### Common Issues

**Q: PWA won't install on iOS Safari**
A: Ensure you're on iOS 14+ and add to Home Screen via Share menu (not browser install prompt).

**Q: Background sync not working on my device**
A: Check if you're on iOS or Firefox. Background Sync isn't supported. Changes sync when you open the app.

**Q: Screen keeps sleeping during cooking (Kitchen Mode)**
A: If Wake Lock isn't supported (Firefox), adjust your device's screen timeout settings manually.

**Q: Push notifications don't work on iOS**
A: Requires iOS 16.4+. Update your device or use in-app reminders instead.

## Resources

- [Browser Support Page](/browser-support) - User-facing compatibility info
- [Playwright Tests](/e2e/tests/cross-browser.spec.ts) - Automated test suite
- [Feature Detection](/static/js/feature-detection.js) - Runtime feature detection
- [Polyfills](/static/js/polyfills.js) - Legacy browser support
- [Browserslist Config](/e2e/package.json) - Target browser definitions

## Version History

- **v0.4.0** (2025-10-19): Initial cross-browser compatibility implementation (Story 5.7)
  - Playwright test suite covering 6 browser configurations
  - Feature detection for PWA APIs
  - Polyfills for older browsers
  - Graceful degradation warnings
  - Browser support documentation page

---

**Last Updated**: 2025-10-19
**Story**: 5.7 - Cross-Browser Compatibility
**Status**: Approved & Implemented

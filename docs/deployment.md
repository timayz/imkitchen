# Deployment Guide - imkitchen

## Prerequisites

### Service Worker Build

Before deployment, the service worker must be generated from source:

```bash
npm run build:sw
```

This command:
- Processes `static/js/sw-source.js` using Workbox CLI
- Injects precache manifest for static assets
- Generates `static/sw.js` (gitignored file)
- Precaches ~26 static assets (~528 kB)

**Important:** `static/sw.js` is a **generated file** and must be built before deployment.

## External Dependencies

### Workbox CDN Dependency

The service worker loads Workbox 7.1.0 from Google's CDN:

```javascript
importScripts('https://storage.googleapis.com/workbox-cdn/releases/7.1.0/workbox-sw.js');
```

**Deployment Considerations:**

1. **CDN Availability:** Service worker functionality requires access to `storage.googleapis.com`
   - Verify CDN is accessible in production environment
   - Consider firewall/proxy rules that may block Google CDN

2. **Offline-First Guarantee:** Once loaded, Workbox is cached by the browser
   - First-time visitors require CDN access
   - Subsequent visits work offline (Workbox cached)

3. **Future Migration Path:** For enhanced offline-first guarantees, consider:
   - Migrating to npm-installed Workbox modules
   - Bundling Workbox locally instead of CDN
   - Reference: https://developers.google.com/web/tools/workbox/guides/using-bundlers

4. **Version Pinning:** Current implementation uses pinned version `7.1.0`
   - Updates require manual version bump in `sw-source.js`
   - Test thoroughly before upgrading Workbox versions

## Build Process

### Production Build

1. Install Node.js dependencies:
```bash
npm install
```

2. Build service worker:
```bash
npm run build:sw
```

3. Build Rust binary:
```bash
cargo build --release
```

4. Verify service worker generation:
```bash
ls -lh static/sw.js
# Should show generated file with Workbox precache manifest
```

## Deployment Checklist

- [ ] Run `npm run build:sw` before deployment
- [ ] Verify `static/sw.js` exists and contains precache manifest
- [ ] Confirm Google CDN (`storage.googleapis.com`) is accessible from production
- [ ] Test service worker registration in production environment
- [ ] Verify offline fallback page (`/offline`) is accessible
- [ ] Test update flow (deploy new version, verify update notification appears)

## Service Worker Update Flow

When deploying a new version:

1. **Build new service worker:**
```bash
npm run build:sw
```

2. **Deploy updated `static/sw.js`** to production

3. **Automatic update detection:**
   - Existing users' browsers check for SW updates every 5 minutes
   - Browser also checks on navigation (built-in behavior)
   - Update notification appears: "New version available. Refresh to update."

4. **User action required:**
   - User clicks "Refresh" button to activate new service worker
   - `skipWaiting` + `clientsClaim` ensure immediate activation

## Cache Management

### Cache Versioning

- Cache names include version suffix: `imkitchen-v1`
- Update `cacheId` in `workbox-config.js` when making breaking cache changes
- Workbox automatically deletes old caches on service worker activation

### Cache Storage Limits

Service worker monitors storage quota during install:
- Logs current usage: `Storage: X MB used of Y MB (Z%)`
- Warns at 75% usage: `Storage quota high`
- Critical warning at 90%: `Storage quota critical`

## Security Considerations

1. **Service Worker Scope:** Restricted to root `/` for full app coverage
2. **CSP Compliance:** No inline scripts, Workbox loaded via `importScripts`
3. **HTTPS Requirement:** Service workers only work on HTTPS (except localhost)
4. **CDN Integrity:** Google CDN is trusted; SRI not supported for `importScripts()`

## Troubleshooting

### Service Worker Not Registering

Check browser DevTools → Application → Service Workers:
- Verify `/sw.js` returns 200 OK with `application/javascript` MIME type
- Check console for registration errors
- Confirm HTTPS is enabled (or localhost for dev)

### Offline Fallback Not Working

1. Verify `/offline` route returns 200 OK
2. Check service worker console for `setCatchHandler` errors
3. Confirm offline page is precached during install event

### Update Notification Not Appearing

1. Clear browser cache and unregister service worker
2. Verify `updatefound` event listener in `sw-register.js`
3. Check console for "Service Worker update found" message
4. Test in incognito window for clean state

## Monitoring

### Service Worker Metrics

Monitor in production:
- Registration success rate (should be >99%)
- Cache hit ratio (pages-v1, images-v1, api-v1, static-v1)
- Storage quota warnings (75%, 90% thresholds)
- Background sync queue length (offline mutation queue)

### Browser Support

Minimum supported versions:
- Chrome 90+ (desktop/Android)
- Safari 14+ (iOS/desktop)
- Firefox 88+
- Edge 90+

**Note:** Service worker gracefully degrades on unsupported browsers (feature detection in `sw-register.js`).

## References

- [Workbox Documentation](https://developers.google.com/web/tools/workbox)
- [Service Worker Specification](https://www.w3.org/TR/service-workers/)
- [PWA Deployment Best Practices](https://web.dev/pwa-checklist/)

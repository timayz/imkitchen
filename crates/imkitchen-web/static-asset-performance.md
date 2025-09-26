# Static Asset Performance Configuration

This document outlines the required HTTP caching headers and performance optimizations for static assets in the imkitchen PWA.

## Required HTTP Caching Headers

### CSS Files (*.css)
```
Cache-Control: public, max-age=31536000, immutable
Content-Type: text/css
Content-Encoding: gzip (if compressed)
```

### JavaScript Files (*.js)
```
Cache-Control: public, max-age=31536000, immutable
Content-Type: application/javascript
Content-Encoding: gzip (if compressed)
```

### Image Files (*.png, *.jpg, *.svg)
```
Cache-Control: public, max-age=31536000, immutable
Content-Type: image/[type]
Content-Encoding: gzip (for SVG if compressed)
```

### PWA Files (manifest.json, sw.js)
```
Cache-Control: public, max-age=86400
Content-Type: application/json (manifest.json)
Content-Type: application/javascript (sw.js)
```

### Font Files (*.woff2, *.woff, *.ttf)
```
Cache-Control: public, max-age=31536000, immutable
Content-Type: font/[type]
```

## Implementation in Rust (Axum)

Add these headers to your static file serving handler:

```rust
use axum::{
    http::{header, StatusCode},
    response::Response,
};

pub async fn serve_static_file(path: &str) -> Response {
    // Determine cache duration based on file type
    let (cache_duration, is_immutable) = match path.split('.').last() {
        Some("css") | Some("js") | Some("png") | Some("jpg") | Some("svg") | Some("woff2") => {
            (31536000, true) // 1 year, immutable
        }
        Some("json") if path.contains("manifest") => (86400, false), // 1 day
        Some("js") if path.contains("sw.js") => (86400, false), // 1 day for service worker
        _ => (3600, false), // 1 hour default
    };

    let cache_control = if is_immutable {
        format!("public, max-age={}, immutable", cache_duration)
    } else {
        format!("public, max-age={}", cache_duration)
    };

    // Build response with appropriate headers
    let mut response = Response::builder()
        .header(header::CACHE_CONTROL, cache_control)
        .header(header::VARY, "Accept-Encoding");

    // Add compression headers if file is compressed
    if should_compress(path) {
        response = response.header(header::CONTENT_ENCODING, "gzip");
    }

    response.body(file_content).unwrap()
}
```

## Compression Configuration

Enable gzip compression for:
- CSS files
- JavaScript files  
- SVG images
- JSON files (including manifest.json)
- HTML files

## Performance Targets

- Initial page load: <3 seconds on 3G
- CSS delivery: Critical CSS inline, non-critical deferred
- JavaScript: Essential scripts only, defer non-critical
- Images: WebP format where supported, with fallbacks
- Service Worker: Aggressive caching of static assets

## Monitoring

Track these metrics:
- First Contentful Paint (FCP)
- Largest Contentful Paint (LCP)
- Time to Interactive (TTI)
- Cumulative Layout Shift (CLS)

Target scores:
- FCP: <1.8s
- LCP: <2.5s
- TTI: <3.8s
- CLS: <0.1
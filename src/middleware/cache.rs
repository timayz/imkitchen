use axum::{
    body::Body,
    http::{header, Request, Response},
    middleware::Next,
};

/// Middleware to set cache control headers
/// - Static files: Allow caching (1 year for immutable assets)
/// - All other routes: No caching (prevent browser cache)
pub async fn cache_control_middleware(req: Request<Body>, next: Next) -> Response<Body> {
    // Clone the path before moving req
    let path = req.uri().path().to_string();
    let mut response = next.run(req).await;

    // Check if this is a static file request
    let is_static_file = path.starts_with("/static/")
        || path.starts_with("/icons/")
        || path.starts_with("/css/")
        || path.starts_with("/js/")
        || path.starts_with("/images/")
        || path.starts_with("/fonts/")
        || path == "/manifest.json"
        || path == "/robots.txt"
        || path == "/sitemap.xml"
        || path == "/favicon.ico"
        || path.ends_with(".png")
        || path.ends_with(".jpg")
        || path.ends_with(".jpeg")
        || path.ends_with(".gif")
        || path.ends_with(".svg")
        || path.ends_with(".webp")
        || path.ends_with(".css")
        || path.ends_with(".js")
        || path.ends_with(".woff")
        || path.ends_with(".woff2")
        || path.ends_with(".ttf")
        || path.ends_with(".eot");

    let headers = response.headers_mut();

    if is_static_file {
        // Cache static files aggressively (1 year)
        headers.insert(
            header::CACHE_CONTROL,
            "public, max-age=31536000, immutable"
                .parse()
                .unwrap(),
        );
    } else {
        // Don't cache HTML pages and API responses
        headers.insert(
            header::CACHE_CONTROL,
            "no-store, no-cache, must-revalidate, proxy-revalidate"
                .parse()
                .unwrap(),
        );
        headers.insert(header::PRAGMA, "no-cache".parse().unwrap());
        headers.insert(header::EXPIRES, "0".parse().unwrap());
    }

    response
}

use axum::{body::Body, http::header, response::Response};

/// Serve service worker at /sw.js (must be at root for scope: '/')
pub async fn sw() -> Response {
    match crate::assets::Assets::get("/sw.js") {
        Some(content) => Response::builder()
            .header(
                header::CONTENT_TYPE,
                "application/javascript; charset=utf-8",
            )
            .header(header::CACHE_CONTROL, "no-cache, no-store, must-revalidate")
            .header("Service-Worker-Allowed", "/")
            .body(Body::from(content.data))
            .unwrap(),
        None => Response::builder()
            .status(404)
            .body(Body::from("Service worker not found"))
            .unwrap(),
    }
}

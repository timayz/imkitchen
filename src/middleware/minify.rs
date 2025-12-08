use axum::{body::Body, response::Response};
cfg_if::cfg_if! {
    if #[cfg(not(debug_assertions))] {
        use axum::{body::to_bytes, http::header};
        use std::sync::LazyLock;

        /// Configuration for HTML minification
        static MINIFY_CFG: LazyLock<minify_html::Cfg> = LazyLock::new(|| minify_html::Cfg {
            keep_closing_tags: true,
            keep_html_and_head_opening_tags: true,
            minify_doctype: false,
            minify_css: true,
            minify_js: true,
            ..Default::default()
        });
    }
}

/// Middleware to minify HTML responses
///
/// This middleware intercepts responses with `text/html` content type
/// and applies HTML minification to reduce payload size.
pub async fn minify_html_middleware(response: Response<Body>) -> Response<Body> {
    cfg_if::cfg_if! {
        if #[cfg(not(debug_assertions))] {
            let content_type = response
                .headers()
                .get(header::CONTENT_TYPE)
                .map(|h| h.to_str().unwrap_or_default())
                .unwrap_or_default();

            if content_type.contains("text/html") {
                let (parts, body) = response.into_parts();
                let bytes = to_bytes(body, usize::MAX).await.unwrap_or_default();
                let minified = minify_html::minify(&bytes, &MINIFY_CFG);
                let new_response = Response::from_parts(parts, Body::from(minified));
                return new_response;
            }
        }
    }

    response
}

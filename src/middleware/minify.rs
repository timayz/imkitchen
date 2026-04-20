use axum::{body::Body, response::Response};
use axum::{body::to_bytes, http::header};
use lightningcss::printer::PrinterOptions;
use lightningcss::stylesheet::{ParserOptions, StyleSheet};
use oxc::allocator::Allocator;
use oxc::codegen::{Codegen, CodegenOptions};
use oxc::minifier::{Minifier, MinifierOptions};
use oxc::parser::Parser;
use oxc::span::SourceType;
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

/// Middleware to minify HTML responses
///
/// This middleware intercepts responses with `text/html` content type
/// and applies HTML minification to reduce payload size.
pub async fn minify_html_middleware(response: Response<Body>) -> Response<Body> {
    let content_type = response
        .headers()
        .get(header::CONTENT_TYPE)
        .map(|h| h.to_str().unwrap_or_default().to_owned())
        .unwrap_or_default();

    let needles = &["text/html", "javascript", "text/css"];
    if !needles.iter().any(|n| content_type.contains(n)) {
        return response;
    }

    let (parts, body) = response.into_parts();
    let mut bytes = to_bytes(body, usize::MAX).await.unwrap_or_default();

    let minified_bytes: Option<Vec<u8>> = if content_type.contains("text/html") {
        Some(minify_html::minify(&bytes, &MINIFY_CFG))
    } else if content_type.contains("javascript") {
        // matches application/javascript, text/javascript, application/x-javascript
        std::str::from_utf8(&bytes)
            .ok()
            .map(|s| minify_js(s).into_bytes())
    } else if content_type.contains("text/css") {
        // use lightningcss for real CSS (see note below)
        minify_css(&bytes).ok()
    } else {
        None
    };

    if let Some(minified_bytes) = minified_bytes {
        bytes = minified_bytes.into();
    }

    Response::from_parts(parts, Body::from(bytes))
}

fn minify_js(source: &str) -> String {
    let allocator = Allocator::default();
    let source_type = SourceType::mjs(); // or ::cjs() / ::default()

    let parser_ret = Parser::new(&allocator, source, source_type).parse();
    if !parser_ret.errors.is_empty() {
        // parsing failed — return original rather than corrupt JS
        return source.to_string();
    }
    let mut program = parser_ret.program;

    let minifier_ret = Minifier::new(MinifierOptions::default()).minify(&allocator, &mut program);

    Codegen::new()
        .with_options(CodegenOptions::minify())
        .with_scoping(minifier_ret.scoping) // enables identifier mangling
        .build(&program)
        .code
}

fn minify_css(bytes: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error + '_>> {
    let src = std::str::from_utf8(bytes)?;
    let ss = StyleSheet::parse(src, ParserOptions::default())?;
    let out = ss.to_css(PrinterOptions {
        minify: true,
        ..Default::default()
    })?;
    Ok(out.code.into_bytes())
}

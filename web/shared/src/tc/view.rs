use topcoat::router::{HeaderName, HeaderValue, header::CONTENT_ENCODING};
use topcoat::{
    Result,
    asset::AssetConfig,
    context::{Cx, try_app_context},
    view::{Child, View, component, view},
};

use super::context::language_iso;

/// Opts a streaming page (`live!`, `suspense`) out of response compression.
///
/// topcoat compresses through tower-http, which buffers the whole body instead
/// of flushing per emission, so a compressed `live!` page reaches the browser in
/// one piece when the stream ends. tower-http leaves a response alone when it
/// already carries `Content-Encoding`. Interpolate this first in the page view:
/// `(streamed())`.
pub fn streamed() -> (HeaderName, HeaderValue) {
    (CONTENT_ENCODING, HeaderValue::from_static("identity"))
}

/// Document shell shared by every topcoat page: head, stylesheet, the browser
/// runtime and the vanilla bridge.
#[component]
pub async fn base(
    cx: &Cx,
    #[into] title: String,
    #[default] child: Child<'_>,
) -> Result<impl View> {
    let lang = language_iso(cx).to_owned();
    let version = env!("CARGO_PKG_VERSION");
    let css = format!("/static/css/main.css?v={version}");
    let bridge = format!("/static/js/tc-bridge.js?v={version}");
    let icon = format!("/static/icons/icon-192.png?v={version}");
    let apple_icon = format!("/static/icons/apple-touch-icon.png?v={version}");
    let manifest = format!("/manifest.json?v={version}");
    // The runtime script is a bundled asset and panics when rendered without a
    // bundle; router tests run without one.
    let has_assets = try_app_context::<AssetConfig>(cx).is_some();

    Ok(view! {
        <!DOCTYPE html>
        <html lang=(lang)>
            <head>
                <meta charset="utf-8">
                <meta name="viewport" content="width=device-width, initial-scale=1">
                <title>(title) " · imkitchen"</title>
                <link rel="icon" type="image/png" href=(icon)>
                <link rel="manifest" href=(manifest)>
                <meta name="theme-color" content="#ef6c1e">
                <meta name="apple-mobile-web-app-capable" content="yes">
                <meta name="apple-mobile-web-app-status-bar-style" content="default">
                <meta name="apple-mobile-web-app-title" content="imkitchen">
                <link rel="apple-touch-icon" href=(apple_icon)>
                <link rel="stylesheet" href=(css)>
                <script src=(bridge)></script>
                if has_assets {
                    topcoat::runtime::script()
                }
            </head>
            <body class="bg-cream text-ink">
                // No toast container here: `tc-bridge.js` owns one outside <body>,
                // out of reach of the page re-run morph.
                (child)
            </body>
        </html>
    })
}

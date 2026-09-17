use topcoat::{
    Result,
    context::Cx,
    router::{
        RouterBuilder, Slot, StatusCode,
        error::{ForbiddenError, NotFoundError},
        layout, not_found,
    },
    view::{View, ViewExt, component, error_boundary, view},
};

use super::{context::i18n, view::base};

// Every URL no topcoat route serves resolves to a `NotFoundError`, so it
// bubbles through the root layout below instead of answering a bare 404.
not_found!("/");

/// Root layout: turns errors bubbling out of any page into a branded page.
/// Replaces the askama `try_page_response!` macro: pages just use `?`.
///
/// Router control errors (redirects, rewrites, bad requests) are rethrown so
/// the router still answers them itself.
#[layout("/")]
async fn root(slot: Slot<'_>) -> Result<impl View> {
    Ok(view! {
        error_boundary(
            fallback: |error| {
                let is_forbidden = error.downcast_ref::<ForbiddenError>().is_some()
                    || matches!(
                        error.downcast_ref::<imkitchen_core::Error>(),
                        Some(imkitchen_core::Error::Forbidden(_))
                    );

                if error.downcast_ref::<NotFoundError>().is_some() {
                    return Ok(view! { error_page(status: StatusCode::NOT_FOUND, title: "404") }.boxed());
                }

                if is_forbidden {
                    return Ok(view! { error_page(status: StatusCode::FORBIDDEN, title: "403") }.boxed());
                }

                if error.downcast_ref::<imkitchen_core::Error>().is_some()
                    || error.downcast_ref::<anyhow::Error>().is_some()
                {
                    tracing::error!("{error}");
                    return Ok(view! {
                        error_page(status: StatusCode::INTERNAL_SERVER_ERROR, title: "500")
                    }
                    .boxed());
                }

                Err(error)
            },
            (slot)
        )
    })
}

// SPIKE: placeholder for the ported 403.html / 404.html / 500.html.
#[component]
async fn error_page(cx: &Cx, status: StatusCode, title: &'static str) -> Result<impl View> {
    let t = i18n(cx);
    let message = match status {
        StatusCode::NOT_FOUND => t.t("Not found"),
        StatusCode::FORBIDDEN => t.t("Forbidden"),
        _ => t.t("Something went wrong, please retry later"),
    };

    Ok(view! {
        (status)
        base(
            title: title,
            <main class="container mx-auto px-4 py-24 text-center">
                <h1 class="text-6xl font-serif font-bold text-ink">(title)</h1>
                <p class="mt-4 text-ink-2">(message)</p>
                <a href="/" class="mt-8 inline-block text-primary-500 font-semibold">"imkitchen"</a>
            </main>
        )
    })
}

pub fn routes(builder: RouterBuilder) -> RouterBuilder {
    builder.layout(root).page(not_found)
}

#[cfg(test)]
mod tests {
    use topcoat::{
        cookie::RouterBuilderCookieExt,
        router::{Body, Router, page, request::Request, to_bytes},
        runtime::{RouterBuilderRuntimeExt, signal},
    };

    use super::*;

    #[page("/hello")]
    async fn hello(cx: &Cx) -> Result<impl View> {
        let open = signal(cx, || false);
        Ok(view! {
            base(title: "Hello", <button @click=$(|_e| open.toggle())>"hi"</button>)
        })
    }

    #[page("/forbidden")]
    async fn forbidden_page() -> Result<impl View> {
        Err::<topcoat::view::BoxView<'static>, _>(topcoat::router::error::forbidden().into())
    }

    // No `.assets(..)` and no `AppState`: the shape router tests run in.
    fn router() -> Router {
        routes(Router::builder().runtime().cookies())
            .page(hello)
            .page(forbidden_page)
            .build()
    }

    async fn get(path: &str, accept_language: &str) -> (StatusCode, String) {
        let request = Request::builder()
            .uri(path)
            .header("accept-language", accept_language)
            .body(Body::empty())
            .unwrap();
        let response = router().handle(request).await;
        let status = response.status();
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        (status, String::from_utf8(body.to_vec()).unwrap())
    }

    #[tokio::test]
    async fn page_renders_without_an_asset_bundle() {
        let (status, html) = get("/hello", "en").await;

        assert_eq!(status, StatusCode::OK);
        assert!(html.contains("<title>Hello · imkitchen</title>"), "{html}");
        assert!(html.contains("/static/js/tc-bridge.js"));
        // The runtime script is a bundled asset; without a bundle it is left out
        // instead of panicking the render.
        assert!(!html.contains("/_topcoat/assets/"), "{html}");
    }

    #[tokio::test]
    async fn language_follows_query_then_header() {
        let (_, html) = get("/hello", "fr-CA,fr;q=0.9").await;
        assert!(html.contains(r#"<html lang="fr""#), "{html}");

        let (_, html) = get("/hello?lang=en", "fr-CA,fr;q=0.9").await;
        assert!(html.contains(r#"<html lang="en""#), "{html}");
    }

    #[tokio::test]
    async fn unknown_url_renders_the_branded_404() {
        let (status, html) = get("/no/such/page", "en").await;

        assert_eq!(status, StatusCode::NOT_FOUND);
        assert!(html.contains("<h1") && html.contains("404"), "{html}");
        assert!(html.contains("Not found"));
    }

    #[tokio::test]
    async fn forbidden_error_renders_the_branded_403() {
        let (status, html) = get("/forbidden", "fr").await;

        assert_eq!(status, StatusCode::FORBIDDEN);
        assert!(html.contains("403"), "{html}");
    }
}

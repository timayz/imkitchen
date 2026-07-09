//! Demo mode — a browsable, no-login tour of imkitchen.
//!
//! Every page reuses the production templates (kitchen, menu, recipes,
//! groceries, cooking screen) but is fed hand-authored placeholder data from
//! [`fixtures`]. Mutating actions are swapped client-side for a sign-up modal
//! (see `templates/partials/demo-signup-modal.html`); the swap is driven by the
//! `is_demo` flag injected into every render via [`imkitchen_web_shared::template::Template::demo`].

use axum::{
    extract::Path,
    response::{IntoResponse, Redirect},
    routing::get,
};
use axum_extra::extract::Query;

use imkitchen_web_recipe::routes::cook::PageQuery as CookPageQuery;
use imkitchen_web_recipe::routes::index::PageQuery;
use imkitchen_web_shared::{
    AppState,
    template::{Template, filters},
};

pub mod fixtures;

#[derive(askama::Template)]
#[template(path = "partials/demo-signup-modal.html")]
pub struct DemoSignupModalTemplate;

pub fn routes() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/demo", get(kitchen))
        .route("/demo/kitchen", get(kitchen))
        .route("/demo/kitchen/{date}", get(kitchen))
        .route("/demo/kitchen/{date}/{recipe_id}/cook", get(cook))
        .route("/demo/menu", get(menu))
        .route("/demo/menu/{date}", get(menu_date))
        .route("/demo/recipes", get(recipes))
        .route("/demo/recipes/{id}", get(recipes_detail))
        .route("/demo/r/{slug}", get(recipes_detail))
        .route("/demo/cooks/{username}", get(cooks))
        .route("/demo/groceries", get(groceries))
        .route("/demo/signup", get(signup_modal))
}

async fn kitchen(template: Template) -> impl IntoResponse {
    template.demo().render(fixtures::kitchen())
}

async fn cook(
    template: Template,
    Path((_date, recipe_id)): Path<(String, String)>,
) -> impl IntoResponse {
    template.demo().render(fixtures::cooking(&recipe_id))
}

async fn menu(template: Template) -> impl IntoResponse {
    template.demo().render(fixtures::menu(None))
}

async fn menu_date(template: Template, Path((date,)): Path<(String,)>) -> impl IntoResponse {
    template.demo().render(fixtures::menu(Some(date)))
}

async fn recipes(template: Template, Query(query): Query<PageQuery>) -> impl IntoResponse {
    template.demo().render(fixtures::recipes(query))
}

async fn recipes_detail(template: Template, Path((slug,)): Path<(String,)>) -> impl IntoResponse {
    // Demo catalog recipes render from fixtures (their ids double as slugs). Any
    // other slug (e.g. a real recipe reached from an anonymous public page)
    // redirects to the real, now-public detail route, which renders in demo mode
    // for guests and resolves both slugs and ids.
    if fixtures::find_recipe(&slug).is_some() {
        template
            .demo()
            .render(fixtures::recipe_detail(&slug))
            .into_response()
    } else {
        Redirect::to(&format!("/r/{slug}")).into_response()
    }
}

async fn cooks(
    template: Template,
    Path((username,)): Path<(String,)>,
    Query(query): Query<CookPageQuery>,
) -> impl IntoResponse {
    template.demo().render(fixtures::cook(&username, query))
}

async fn groceries(template: Template) -> impl IntoResponse {
    template.demo().render(fixtures::groceries())
}

async fn signup_modal(template: Template) -> impl IntoResponse {
    // ts-swap="skip" so twinspark appends the modal instead of replacing the
    // element that triggered it.
    (
        [("ts-swap", "skip")],
        template.demo().render(DemoSignupModalTemplate),
    )
        .into_response()
}

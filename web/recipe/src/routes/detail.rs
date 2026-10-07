use axum::{
    extract::{Path, State},
    response::{IntoResponse, Redirect},
};
use evento::cursor::ReadResult;
use imkitchen_core::recipe::{
    favorite,
    query::{
        user::{UserView, UserViewList},
        user_stat::UserStatView,
    },
};
use imkitchen_types::recipe::{DietaryRestriction, IngredientUnitFormat, RecipeType};
use serde_json::json;

use imkitchen_web_shared::{
    AppState,
    auth::{AuthUser, RequireChef},
    services::recipe::{self, ShareError},
    template::{NotFoundTemplate, Status, Template, filters},
};

#[derive(askama::Template)]
#[template(path = "partials/set-username-modal.html")]
pub struct SetUsernameModalTemplate;

#[derive(askama::Template)]
#[template(path = "partials/recipes-delete-modal.html")]
pub struct DeleteModalTemplate {
    pub id: String,
}

#[derive(askama::Template)]
#[template(path = "partials/recipes-detail-share-button.html")]
pub struct CommunityDetailShareButtonTemplate<'a> {
    pub id: &'a str,
    pub is_shared: bool,
}

#[derive(askama::Template)]
#[template(path = "partials/recipes-delete-button.html")]
pub struct DeleteButtonTemplate<'a> {
    pub id: &'a str,
    pub status: imkitchen_web_shared::template::Status,
}

#[derive(askama::Template)]
#[template(path = "recipes-detail.html")]
pub struct DetailTemplate<'a> {
    pub current_path: String,
    pub user: AuthUser,
    pub username: &'a str,
    pub recipe: UserView,
    pub stat: UserStatView,
    pub favorite: favorite::Favorite,
    pub owner_description: String,
    /// Whether this recipe is already in the viewer's shopping list (drives the
    /// initial state of the "Add to shopping list" button).
    pub in_shopping: bool,
    /// Pre-serialized schema.org/Recipe JSON-LD for search-engine rich
    /// results. Empty string renders no `<script>` (e.g. in demo mode).
    pub json_ld: String,
}

/// Right-rail "Similar recipes" fragment, lazily loaded via twinspark
/// (`GET /r/{slug}/similar`) so the detail page returns its main content
/// without waiting on the similar-recipes queries.
#[derive(askama::Template)]
#[template(path = "partials/recipes-similar.html")]
pub struct SimilarTemplate {
    pub similar_recipes: ReadResult<UserViewList>,
}

/// Builds a schema.org/Recipe JSON-LD document for a recipe. The `<`, `>` and
/// `&` bytes are escaped to their `\uXXXX` form so the result is safe to embed
/// verbatim inside a `<script type="application/ld+json">` tag.
pub fn recipe_json_ld(recipe: &UserView, base_url: &str) -> String {
    let url = format!("{base_url}/r/{}", recipe.slug);
    let image = match &recipe.thumbnail_version {
        Some(v) => format!(
            "{base_url}/recipes/{}/thumbnail/desktop/image.webp?v={v}",
            recipe.id
        ),
        None => format!("{base_url}/static/icons/icon-512.png"),
    };

    let ingredients: Vec<String> = recipe
        .ingredients
        .iter()
        .map(|i| {
            format!("{} {}", i.unit.format(i.quantity), i.name)
                .trim()
                .to_owned()
        })
        .collect();

    let instructions: Vec<serde_json::Value> = recipe
        .instructions
        .iter()
        .enumerate()
        .map(|(idx, ins)| {
            json!({
                "@type": "HowToStep",
                "position": idx + 1,
                "text": ins.description,
            })
        })
        .collect();

    let diets: Vec<&str> = recipe
        .dietary_restrictions
        .iter()
        .filter_map(|d| match d {
            DietaryRestriction::Vegetarian => Some("https://schema.org/VegetarianDiet"),
            DietaryRestriction::Vegan => Some("https://schema.org/VeganDiet"),
            DietaryRestriction::GlutenFree => Some("https://schema.org/GlutenFreeDiet"),
            DietaryRestriction::DairyFree => Some("https://schema.org/LowLactoseDiet"),
            DietaryRestriction::NutFree => None,
        })
        .collect();

    let mut doc = json!({
        "@context": "https://schema.org",
        "@type": "Recipe",
        "name": recipe.name,
        "url": url,
        "image": [image],
        "recipeCategory": recipe.recipe_type.0.to_string(),
        "recipeYield": format!("{} servings", recipe.household_size),
        "author": {
            "@type": "Person",
            "name": recipe.owner_name.clone().unwrap_or_else(|| "imkitchen".to_owned()),
        },
        "recipeIngredient": ingredients,
        "recipeInstructions": instructions,
    });

    if !recipe.description.is_empty() {
        doc["description"] = json!(recipe.description);
    }
    if recipe.prep_time > 0 {
        doc["prepTime"] = json!(format!("PT{}M", recipe.prep_time));
    }
    if recipe.cook_time > 0 {
        doc["cookTime"] = json!(format!("PT{}M", recipe.cook_time));
    }
    if recipe.prep_time + recipe.cook_time > 0 {
        doc["totalTime"] = json!(format!("PT{}M", recipe.prep_time + recipe.cook_time));
    }
    if !diets.is_empty() {
        doc["suitableForDiet"] = json!(diets);
    }

    serde_json::to_string(&doc)
        .unwrap_or_default()
        .replace('<', "\\u003c")
        .replace('>', "\\u003e")
        .replace('&', "\\u0026")
}

impl<'a> Default for DetailTemplate<'a> {
    fn default() -> Self {
        Self {
            current_path: "recipes".to_owned(),
            user: AuthUser::default(),
            recipe: Default::default(),
            stat: UserStatView::default(),
            favorite: Default::default(),
            username: "john_doe",
            owner_description: String::new(),
            in_shopping: false,
            json_ld: String::new(),
        }
    }
}

/// Permanent redirect from the legacy `/recipes/{id}` URL to the canonical
/// slug-based `/r/{slug}` detail page, keeping old links and bookmarks working.
#[tracing::instrument(skip_all)]
pub async fn redirect_to_slug(
    template: Template,
    Path((id,)): Path<(String,)>,
    State(app): State<AppState>,
) -> impl IntoResponse {
    let recipe = imkitchen_web_shared::try_page_response!(opt: app.core.recipe.user(&id), template);

    Redirect::permanent(&format!("/r/{}", recipe.slug)).into_response()
}

#[tracing::instrument(skip_all)]
pub async fn page(
    template: Template,
    user: Option<AuthUser>,
    Path((slug,)): Path<(String,)>,
    State(app): State<AppState>,
) -> impl IntoResponse {
    let id = imkitchen_web_shared::try_page_response!(recipe::resolve_id(&app, &slug), template);

    // Public recipes are viewable by anyone. Anonymous visitors get a demo
    // "guest" identity and the page renders in demo mode — links point into
    // /demo and actions (Save, etc.) lead to sign-up.
    let is_anonymous = user.is_none();
    let user = user.unwrap_or_else(AuthUser::demo);

    let Some(detail) = imkitchen_web_shared::try_page_response!(
        recipe::detail(&app, &user.id, is_anonymous, &id),
        template
    ) else {
        return template.render(NotFoundTemplate).into_response();
    };

    let template = if is_anonymous {
        template.demo()
    } else {
        template
    };

    let username = user.username();
    // Structured data for search engines — only on the canonical public page
    // (signed-in or guest), not the demo tour.
    let json_ld = recipe_json_ld(&detail.recipe, &app.config.server.url);

    template
        .render(DetailTemplate {
            user,
            recipe: detail.recipe,
            stat: detail.stat,
            favorite: detail.favorite,
            username: username.as_str(),
            owner_description: detail.owner_description,
            in_shopping: detail.in_shopping,
            json_ld,
            ..Default::default()
        })
        .into_response()
}

/// Right-rail "Similar recipes" fragment, lazily loaded by the detail page via
/// twinspark (`ts-trigger="load"`). Kept off the page's critical path because
/// finding suggestions runs up to three fallback queries.
#[tracing::instrument(skip_all)]
pub async fn similar(
    template: Template,
    user: Option<AuthUser>,
    Path((slug,)): Path<(String,)>,
    State(app): State<AppState>,
) -> impl IntoResponse {
    let id = imkitchen_web_shared::try_page_response!(recipe::resolve_id(&app, &slug), template);
    let view = imkitchen_web_shared::try_page_response!(opt: app.core.recipe.user(&id), template);

    // Mirror the page's visibility + demo handling: only shared recipes (or the
    // owner's own) are viewable, and anonymous visitors render in demo mode.
    let is_anonymous = user.is_none();
    let user = user.unwrap_or_else(AuthUser::demo);

    if !recipe::viewable(&view, &user.id) {
        return template.render(NotFoundTemplate).into_response();
    }

    let template = if is_anonymous {
        template.demo()
    } else {
        template
    };

    let similar_recipes =
        imkitchen_web_shared::try_page_response!(recipe::similar(&app, &view), template);

    template
        .render(SimilarTemplate { similar_recipes })
        .into_response()
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn share_to_community_action(
    template: Template,
    State(app): State<AppState>,
    RequireChef(user): RequireChef,
    Path((id,)): Path<(String,)>,
) -> impl IntoResponse {
    match recipe::share(&app, &user.id, user.username.as_deref(), &id).await {
        Ok(()) => {}
        Err(ShareError::UsernameRequired) => {
            return (
                [("ts-swap", "skip")],
                template.render(SetUsernameModalTemplate),
            )
                .into_response();
        }
        Err(ShareError::Core(err)) => {
            imkitchen_web_shared::try_response!(sync: Err::<(), _>(err), template);
        }
    }

    template
        .render(CommunityDetailShareButtonTemplate {
            id: &id,
            is_shared: true,
        })
        .into_response()
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn make_private_action(
    template: Template,
    State(app): State<AppState>,
    RequireChef(user): RequireChef,
    Path((id,)): Path<(String,)>,
) -> impl IntoResponse {
    imkitchen_web_shared::try_response!(recipe::unshare(&app, &user.id, &id), template);

    template
        .render(CommunityDetailShareButtonTemplate {
            id: &id,
            is_shared: false,
        })
        .into_response()
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn delete_action(
    template: Template,
    State(app): State<AppState>,
    AuthUser(user): AuthUser,
    Path((id,)): Path<(String,)>,
) -> impl IntoResponse {
    imkitchen_web_shared::try_response!(recipe::delete(&app, &user.id, &id), template);

    template
        .render(DeleteButtonTemplate {
            id: &id,
            status: Status::Pending,
        })
        .into_response()
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn delete_status(
    template: Template,
    State(app): State<AppState>,
    user: AuthUser,
    Path((id,)): Path<(String,)>,
) -> impl IntoResponse {
    match imkitchen_web_shared::try_response!(anyhow:
        app.core.recipe.find_user(&id),
        template,
        Some(DeleteButtonTemplate {
            id: &id,
            status: Status::Idle,
        })
    ) {
        Some(_) => template
            .render(DeleteButtonTemplate {
                id: &id,
                status: Status::Checking,
            })
            .into_response(),
        _ => Redirect::to("/recipes").into_response(),
    }
}

pub async fn delete_modal(template: Template, Path((id,)): Path<(String,)>) -> impl IntoResponse {
    template.render(DeleteModalTemplate { id })
}

#[derive(askama::Template)]
#[template(path = "partials/recipes-detail-save-button.html")]
pub struct SaveButtonTemplate {
    pub id: String,
    pub saved: bool,
}

pub async fn save(
    template: Template,
    AuthUser(user): AuthUser,
    State(app): State<AppState>,
    Path((id,)): Path<(String,)>,
) -> impl IntoResponse {
    imkitchen_web_shared::try_response!(recipe::save(&app, &user.id, &id), template);

    (
        [("ts-swap", "skip")],
        template.render(SaveButtonTemplate { id, saved: true }),
    )
        .into_response()
}

pub async fn unsave(
    template: Template,
    AuthUser(user): AuthUser,
    State(app): State<AppState>,
    Path((id,)): Path<(String,)>,
) -> impl IntoResponse {
    imkitchen_web_shared::try_response!(recipe::unsave(&app, &user.id, &id), template);

    (
        [("ts-swap", "skip")],
        template.render(SaveButtonTemplate { id, saved: false }),
    )
        .into_response()
}

#[derive(askama::Template)]
#[template(path = "partials/recipes-detail-add-to-shopping-button.html")]
pub struct AddToShoppingButtonTemplate {
    pub id: String,
    pub added: bool,
}

/// Add the recipe to the viewer's shopping list. Any viewable recipe (owned or
/// shared) can be added; ingredient scaling uses the viewer's household size.
pub async fn add_to_shopping(
    template: Template,
    AuthUser(user): AuthUser,
    State(app): State<AppState>,
    Path((id,)): Path<(String,)>,
) -> impl IntoResponse {
    imkitchen_web_shared::try_response!(recipe::add_to_shopping(&app, &user.id, &id), template);

    (
        [("ts-swap", "skip")],
        template.render(AddToShoppingButtonTemplate { id, added: true }),
    )
        .into_response()
}

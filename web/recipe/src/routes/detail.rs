use axum::{
    extract::{Path, State},
    response::{IntoResponse, Redirect},
};
use evento::cursor::{Args, ReadResult};
use imkitchen_core::recipe::{
    favorite,
    query::{
        user::{RecipesQuery, SortBy, UserView, UserViewList},
        user_stat::UserStatView,
    },
};
use imkitchen_types::recipe::{DietaryRestriction, IngredientUnitFormat, RecipeType};
use serde_json::json;

use imkitchen_web_shared::{
    AppState,
    auth::{AuthUser, RequireChef, RequirePremium},
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
    // Resolve the path segment as a slug; fall back to treating it as a raw
    // recipe id so legacy/id-shaped links under `/r/` keep working.
    let id = match imkitchen_web_shared::try_page_response!(
        app.core.recipe.find_id_by_slug(&slug),
        template
    ) {
        Some(id) => id,
        None => slug.clone(),
    };
    let recipe = imkitchen_web_shared::try_page_response!(opt: app.core.recipe.user(&id), template);

    // Public recipes are viewable by anyone. Anonymous visitors get a demo
    // "guest" identity and the page renders in demo mode — links point into
    // /demo and actions (Save, etc.) lead to sign-up.
    let is_anonymous = user.is_none();
    let user = user.unwrap_or_else(AuthUser::demo);

    if recipe.owner_id != user.id && !recipe.is_shared {
        return template.render(NotFoundTemplate).into_response();
    }

    let template = if is_anonymous {
        template.demo()
    } else {
        template
    };

    let stat = imkitchen_web_shared::try_page_response!(
        app.core.recipe.find_user_stat(&recipe.owner_id),
        template
    )
    .unwrap_or_default();

    let favorite = imkitchen_web_shared::try_page_response!(
        app.core.recipe.favorite.load(&recipe.id, &user.id),
        template
    )
    .to_owned();

    let owner_profile = imkitchen_web_shared::try_page_response!(
        app.identity.user_profile.load(&recipe.owner_id),
        template
    );

    let username = user.username();
    // Structured data for search engines — only on the canonical public page
    // (signed-in or guest), not the demo tour.
    let json_ld = recipe_json_ld(&recipe, &app.config.server.url);

    template
        .render(DetailTemplate {
            user,
            recipe,
            stat,
            favorite,
            username: username.as_str(),
            owner_description: owner_profile.description,
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
    // Resolve slug → id with the same id-fallback as `page` so legacy links work.
    let id = match imkitchen_web_shared::try_page_response!(
        app.core.recipe.find_id_by_slug(&slug),
        template
    ) {
        Some(id) => id,
        None => slug.clone(),
    };
    let recipe = imkitchen_web_shared::try_page_response!(opt: app.core.recipe.user(&id), template);

    // Mirror the page's visibility + demo handling: only shared recipes (or the
    // owner's own) are viewable, and anonymous visitors render in demo mode.
    let is_anonymous = user.is_none();
    let user = user.unwrap_or_else(AuthUser::demo);

    if recipe.owner_id != user.id && !recipe.is_shared {
        return template.render(NotFoundTemplate).into_response();
    }

    let template = if is_anonymous {
        template.demo()
    } else {
        template
    };

    let exclude_ids = vec![recipe.id.to_owned()];

    let mut similar_recipes = imkitchen_web_shared::try_page_response!(
        app.core.recipe.filter_user(RecipesQuery {
            exclude_ids: Some(exclude_ids.to_vec()),
            user_id: None,
            recipe_type: Some(recipe.recipe_type.0.to_owned()),
            is_shared: Some(true),
            has_thumbnail: None,
            dietary_restrictions: recipe.dietary_restrictions.0.to_vec(),
            dietary_where_any: false,
            in_meal_plan: None,
            sort_by: SortBy::Random,
            args: Args::forward(10, None),
            search: None,
        }),
        template
    );

    if similar_recipes.edges.len() < 10 {
        let mut similar_ids = similar_recipes
            .edges
            .iter()
            .map(|n| n.node.id.to_owned())
            .collect::<Vec<_>>();
        similar_ids.extend(exclude_ids.to_vec());

        let more_recipes = imkitchen_web_shared::try_page_response!(
            app.core.recipe.filter_user(RecipesQuery {
                exclude_ids: Some(similar_ids),
                user_id: None,
                recipe_type: Some(recipe.recipe_type.0.to_owned()),
                is_shared: Some(true),
                has_thumbnail: None,
                dietary_restrictions: recipe.dietary_restrictions.0.to_vec(),
                dietary_where_any: true,
                in_meal_plan: None,
                sort_by: SortBy::Random,
                args: Args::forward(10, None),
                search: None,
            }),
            template
        );

        similar_recipes.edges.extend(more_recipes.edges);
    }

    if similar_recipes.edges.len() < 10 {
        let mut similar_ids = similar_recipes
            .edges
            .iter()
            .map(|n| n.node.id.to_owned())
            .collect::<Vec<_>>();
        similar_ids.extend(exclude_ids);

        let more_recipes = imkitchen_web_shared::try_page_response!(
            app.core.recipe.filter_user(RecipesQuery {
                exclude_ids: Some(similar_ids),
                user_id: None,
                recipe_type: Some(recipe.recipe_type.0.to_owned()),
                is_shared: Some(true),
                has_thumbnail: None,
                dietary_restrictions: vec![],
                dietary_where_any: false,
                in_meal_plan: None,
                sort_by: SortBy::Random,
                args: Args::forward(10, None),
                search: None,
            }),
            template
        );

        similar_recipes.edges.extend(more_recipes.edges);
    }

    // Fallback tiers can overshoot the target; keep at most 10 (exact matches,
    // which are collected first, take precedence).
    similar_recipes.edges.truncate(10);

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
    let Some(ref username) = user.username else {
        return (
            [("ts-swap", "skip")],
            template.render(SetUsernameModalTemplate),
        )
            .into_response();
    };

    imkitchen_web_shared::try_response!(
        app.core.recipe.share_to_community(&id, &user.id, username),
        template
    );

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
    imkitchen_web_shared::try_response!(app.core.recipe.make_private(&id, &user.id), template);

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
    RequirePremium(user): RequirePremium,
    Path((id,)): Path<(String,)>,
) -> impl IntoResponse {
    imkitchen_web_shared::try_response!(app.core.recipe.delete(&id, &user.id), template);

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
    RequirePremium(user): RequirePremium,
    State(app): State<AppState>,
    Path((id,)): Path<(String,)>,
) -> impl IntoResponse {
    let recipe =
        imkitchen_web_shared::try_response!(anyhow_opt: app.core.recipe.user(&id),template);

    if !recipe.is_shared {
        imkitchen_web_shared::try_response!(sync:
            Err(imkitchen_core::Error::NotFound("recipe".to_owned())
        ), template);
    }

    imkitchen_web_shared::try_response!(
        app.core
            .recipe
            .favorite
            .save(&id, recipe.owner_id, &user.id),
        template
    );

    (
        [("ts-swap", "skip")],
        template.render(SaveButtonTemplate { id, saved: true }),
    )
        .into_response()
}

pub async fn unsave(
    template: Template,
    RequirePremium(user): RequirePremium,
    State(app): State<AppState>,
    Path((id,)): Path<(String,)>,
) -> impl IntoResponse {
    imkitchen_web_shared::try_response!(app.core.recipe.favorite.unsave(&id, &user.id), template);

    (
        [("ts-swap", "skip")],
        template.render(SaveButtonTemplate { id, saved: false }),
    )
        .into_response()
}

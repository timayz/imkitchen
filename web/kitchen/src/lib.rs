//! The kitchen: the user's recipe list (generate, add, remove), what to cook
//! next from it, and the step-by-step cooking screens.
//!
//! The logic lives in `imkitchen_web_shared::services::kitchen` (shared with
//! the JSON API); this crate only renders its results into templates.

use axum::extract::{Path, State};
use axum::response::{IntoResponse, Redirect};
use axum_extra::extract::{CookieJar, Form};
use imkitchen_core::recipe::query::user::UserView;
use imkitchen_types::recipe::{IngredientUnitFormat, Instruction, RecipeType};
use imkitchen_types::shopping::RecipeStatus;
use serde::Deserialize;

pub use imkitchen_web_shared::config;
pub use imkitchen_web_shared::services::kitchen::{IngredientAisle, ListEntry, list_entries};

use imkitchen_web_shared::AppState;
use imkitchen_web_shared::auth::{AuthToken, AuthUser};
use imkitchen_web_shared::services::kitchen::{self, CookingScreen, Overview};
use imkitchen_web_shared::template::{NotFoundTemplate, Template, filters};

#[derive(askama::Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub show_nav: bool,
}

/// The kitchen with nothing to cook: one card, one action.
#[derive(askama::Template)]
#[template(path = "kitchen-empty.html")]
pub struct KitchenEmptyTemplate {
    pub current_path: String,
    pub user: AuthUser,
    /// Offer "Generate" (true) or "Add your first recipe" (false).
    pub has_recipes: bool,
}

#[derive(askama::Template)]
#[template(path = "kitchen.html")]
pub struct KitchenTemplate {
    pub current_path: String,
    pub user: AuthUser,
    /// Every recipe in the list, in list order (generated meals read starter →
    /// main → side → dessert → drink → sauce).
    pub entries: Vec<ListEntry>,
    /// The recipe to cook next: the first one in the list not yet cooked.
    pub slot_recipe: UserView,
    pub focused_status: RecipeStatus,
    pub completed_count: usize,
    pub total_count: usize,
    /// Uncooked recipes (other than the focused one) with an advance-prep note.
    pub prep_ahead: Vec<ListEntry>,
    pub completed_instructions: Vec<(usize, String)>,
    pub coming_instructions: Vec<(usize, String)>,
    pub current_instruction: Option<(usize, Instruction)>,
    /// When true, the "Start cooking" button links to the recipe's original URL
    /// (external) instead of the in-app cooking screen.
    pub cook_external: bool,
}

#[tracing::instrument(skip_all, fields(user = tracing::field::Empty))]
pub async fn page(
    template: Template,
    user: Option<AuthUser>,
    token: Option<AuthToken>,
    State(app): State<AppState>,
    jar: CookieJar,
) -> impl IntoResponse {
    let (Some(user), Some(token)) = (user, token) else {
        return template
            .render(IndexTemplate { show_nav: true })
            .into_response();
    };

    tracing::Span::current().record("user", &user.id);

    let overview =
        imkitchen_web_shared::try_page_response!(kitchen::overview(&app, &user.id), template);

    // Sliding session: every kitchen render re-issues the cookie.
    let auth_cookie = imkitchen_web_shared::try_page_response!(sync:
        imkitchen_web_shared::auth::build_cookie(app.config.jwt, token.sub.to_owned(), token.acc.to_owned()),
        template
    );
    let jar = jar.add(auth_cookie);

    let list = match overview {
        Overview::Empty(empty) => {
            return (
                jar,
                template.render(KitchenEmptyTemplate {
                    current_path: "kitchen".to_owned(),
                    user,
                    has_recipes: empty.has_recipes,
                }),
            )
                .into_response();
        }
        Overview::List(list) => list,
    };
    let (completed_instructions, coming_instructions, current_instruction) = list.steps;

    (
        jar,
        template.render(KitchenTemplate {
            current_path: "kitchen".to_owned(),
            user,
            entries: list.entries,
            slot_recipe: list.focused,
            focused_status: list.focused_status,
            completed_count: list.completed_count,
            total_count: list.total_count,
            prep_ahead: list.prep_ahead,
            completed_instructions,
            coming_instructions,
            current_instruction,
            cook_external: list.cook_external,
        }),
    )
        .into_response()
}

#[derive(askama::Template)]
#[template(path = "cooking.html")]
pub struct CookingTemplate {
    pub slot_recipe: UserView,
    pub completed_instructions: Vec<(usize, String)>,
    pub coming_instructions: Vec<(usize, String)>,
    pub current_instruction: Option<(usize, Instruction)>,
    pub show_iframe: bool,
    /// When true, render the ingredient list (grouped in `ingredient_aisles`)
    /// as the first screen of the cooking flow instead of a step.
    pub show_ingredients: bool,
    pub ingredient_aisles: Vec<IngredientAisle>,
}

// Fragment version of CookingTemplate — same fields, but renders only the
// #cooking-screen partial so it can be swapped in place by TwinSpark.
#[derive(askama::Template)]
#[template(path = "partials/cooking-screen.html")]
pub struct CookingScreenTemplate {
    pub slot_recipe: UserView,
    pub completed_instructions: Vec<(usize, String)>,
    pub coming_instructions: Vec<(usize, String)>,
    pub current_instruction: Option<(usize, Instruction)>,
    pub show_iframe: bool,
    pub show_ingredients: bool,
    pub ingredient_aisles: Vec<IngredientAisle>,
}

impl From<CookingScreen> for CookingTemplate {
    fn from(screen: CookingScreen) -> Self {
        let (completed_instructions, coming_instructions, current_instruction) = screen.steps;
        Self {
            slot_recipe: screen.recipe,
            completed_instructions,
            coming_instructions,
            current_instruction,
            show_iframe: screen.origin_embeddable,
            show_ingredients: screen.show_ingredients,
            ingredient_aisles: screen.ingredient_aisles,
        }
    }
}

impl From<CookingScreen> for CookingScreenTemplate {
    fn from(screen: CookingScreen) -> Self {
        let (completed_instructions, coming_instructions, current_instruction) = screen.steps;
        Self {
            slot_recipe: screen.recipe,
            completed_instructions,
            coming_instructions,
            current_instruction,
            show_iframe: screen.origin_embeddable,
            show_ingredients: screen.show_ingredients,
            ingredient_aisles: screen.ingredient_aisles,
        }
    }
}

#[tracing::instrument(skip_all, fields(user = tracing::field::Empty))]
pub async fn update_step_action(
    template: Template,
    AuthUser(user): AuthUser,
    State(app): State<AppState>,
    Path((recipe_id, direction)): Path<(String, String)>,
) -> impl IntoResponse {
    tracing::Span::current().record("user", &user.id);

    let Some(screen) = imkitchen_web_shared::try_response!(
        kitchen::step(&app, &user.id, &recipe_id, &direction),
        template
    ) else {
        return template.render(NotFoundTemplate).into_response();
    };

    template
        .render(CookingScreenTemplate::from(screen))
        .into_response()
}

#[derive(askama::Template)]
#[template(path = "partials/kitchen-dish.html")]
pub struct KitchenDishTemplate {
    pub entries: Vec<ListEntry>,
    pub slot_recipe: UserView,
    pub focused_status: RecipeStatus,
    pub completed_instructions: Vec<(usize, String)>,
    pub coming_instructions: Vec<(usize, String)>,
    pub current_instruction: Option<(usize, Instruction)>,
    pub cook_external: bool,
}

#[tracing::instrument(skip_all, fields(user = tracing::field::Empty))]
pub async fn select_dish(
    template: Template,
    AuthUser(user): AuthUser,
    State(app): State<AppState>,
    Path((recipe_id,)): Path<(String,)>,
) -> impl IntoResponse {
    tracing::Span::current().record("user", &user.id);

    let Some(dish) = imkitchen_web_shared::try_page_response!(
        kitchen::dish(&app, &user.id, &recipe_id),
        template
    ) else {
        return template.render(NotFoundTemplate).into_response();
    };

    let (completed_instructions, coming_instructions, current_instruction) = dish.steps;

    template
        .render(KitchenDishTemplate {
            entries: dish.entries,
            slot_recipe: dish.recipe,
            focused_status: dish.status,
            completed_instructions,
            coming_instructions,
            current_instruction,
            cook_external: dish.cook_external,
        })
        .into_response()
}

#[tracing::instrument(skip_all, fields(user = tracing::field::Empty))]
pub async fn cook_page(
    template: Template,
    AuthUser(user): AuthUser,
    State(app): State<AppState>,
    Path((recipe_id,)): Path<(String,)>,
) -> impl IntoResponse {
    tracing::Span::current().record("user", &user.id);

    let screen = imkitchen_web_shared::try_page_response!(opt:
        kitchen::cooking_screen(&app, &user.id, &recipe_id),
        template
    );

    // Imported recipe with no parsed steps whose origin refuses framing: there is
    // nothing to show in-app, so send the user straight to the original instead of
    // rendering a "Open Original Recipe" button they'd have to tap.
    if let Some(origin) = screen.external_only() {
        return Redirect::to(origin).into_response();
    }

    template
        .render(CookingTemplate::from(screen))
        .into_response()
}

#[derive(askama::Template)]
#[template(path = "partials/kitchen-generate-modal.html")]
pub struct GenerateModalTemplate;

pub async fn generate_modal(template: Template, _user: AuthUser) -> impl IntoResponse {
    template.render(GenerateModalTemplate)
}

#[derive(Deserialize, Debug)]
pub struct GenerateForm {
    pub count: u8,
}

/// Replace the list with freshly picked recipes, then send the browser back
/// to the kitchen. The list is read back from the aggregate, so the
/// redirected page is already up to date — no polling needed.
#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn generate_action(
    template: Template,
    State(app): State<AppState>,
    AuthUser(user): AuthUser,
    Form(input): Form<GenerateForm>,
) -> impl IntoResponse {
    imkitchen_web_shared::try_response!(kitchen::generate(&app, &user.id, input.count), template);

    Redirect::to("/").into_response()
}

/// Remove a recipe from the list and re-render the whole kitchen: the hero
/// may have to move on to the next recipe.
#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn remove_recipe_action(
    template: Template,
    user: AuthUser,
    State(app): State<AppState>,
    Path((id,)): Path<(String,)>,
) -> impl IntoResponse {
    imkitchen_web_shared::try_response!(kitchen::remove(&app, &user.id, &id), template);

    Redirect::to("/").into_response()
}

/// Calendar-era `/kitchen/{date}` bookmarks.
async fn legacy_kitchen_redirect() -> impl IntoResponse {
    Redirect::permanent("/")
}

pub fn routes() -> axum::Router<imkitchen_web_shared::AppState> {
    use axum::routing::{get, post};
    axum::Router::new()
        .route("/", get(page))
        .route(
            "/kitchen/generate",
            get(generate_modal).post(generate_action),
        )
        .route("/kitchen/recipe/{id}/remove", post(remove_recipe_action))
        .route(
            "/kitchen/{recipe_id}/step/{direction}",
            post(update_step_action),
        )
        .route("/kitchen/{recipe_id}/select-dish", post(select_dish))
        .route("/kitchen/{recipe_id}/cook", get(cook_page))
        .route("/kitchen/{legacy}", get(legacy_kitchen_redirect))
}

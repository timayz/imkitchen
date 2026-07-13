use axum::{
    extract::{Json, Path, State},
    response::{IntoResponse, Redirect},
};
use axum_extra::extract::Form;
use imkitchen_core::recipe::query::user::RecipeCard;
use imkitchen_core::shopping::{Generate, ToggleInput};
use imkitchen_types::recipe::{Ingredient, IngredientUnitFormat, RecipeType};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};

use imkitchen_web_shared::{
    auth::{AuthUser, RequirePremium},
    state::AppState,
    template::{Status as TemplateStatus, Template, filters},
};

pub fn routes() -> axum::Router<imkitchen_web_shared::AppState> {
    use axum::routing::{get, post};
    axum::Router::new()
        .route("/groceries", get(page))
        .route("/groceries/toggle", post(toggle_action))
        .route(
            "/groceries/generate",
            get(generate_modal).post(generate_action),
        )
        .route("/groceries/generate/status", get(generate_status))
        .route("/groceries/recipe/{id}/remove", post(remove_recipe_action))
}

pub struct AisleSection {
    pub name: String,
    pub items: Vec<Ingredient>,
    pub checked: usize,
    pub total: usize,
    pub done: bool,
    pub pct: usize,
}

#[derive(askama::Template)]
#[template(path = "groceries.html")]
pub struct GroceriesTemplate {
    pub current_path: String,
    pub user: AuthUser,
    pub recipes: Vec<RecipeCard>,
    pub checked: HashSet<String>,
    pub aisles: Vec<AisleSection>,
    /// Index into `aisles` where the right desktop column starts (aisles are
    /// split into two columns balanced by item count).
    pub split_at: usize,
    pub from_date: u64,
    pub to_date: u64,
    pub total_items: usize,
    pub checked_items: usize,
    pub progress_pct: usize,
}

impl Default for GroceriesTemplate {
    fn default() -> Self {
        Self {
            current_path: "groceries".to_owned(),
            user: AuthUser::default(),
            recipes: vec![],
            checked: HashSet::default(),
            aisles: vec![],
            split_at: 0,
            from_date: 0,
            to_date: 0,
            total_items: 0,
            checked_items: 0,
            progress_pct: 0,
        }
    }
}

/// Body fragment (recipes section + aisles) swapped in via twinspark when the
/// recipe set changes. Mirrors the fields the `partials/groceries-body.html`
/// include reads from `GroceriesTemplate`.
#[derive(askama::Template)]
#[template(path = "partials/groceries-body.html")]
pub struct GroceriesBodyTemplate {
    pub recipes: Vec<RecipeCard>,
    pub checked: HashSet<String>,
    pub aisles: Vec<AisleSection>,
    pub split_at: usize,
    pub total_items: usize,
    pub checked_items: usize,
    pub progress_pct: usize,
}

/// Everything the groceries body needs, derived from the persisted list.
struct ShoppingView {
    recipes: Vec<RecipeCard>,
    checked: HashSet<String>,
    aisles: Vec<AisleSection>,
    split_at: usize,
    from_date: u64,
    to_date: u64,
    total_items: usize,
    checked_items: usize,
    progress_pct: usize,
}

async fn build_view(app: &AppState, user_id: &str) -> anyhow::Result<ShoppingView> {
    // Read straight from the aggregate (immediately consistent) rather than the
    // `shopping_list` read model, whose subscription lags a command by a beat —
    // otherwise a re-render right after add/remove shows the pre-change list.
    let household_size = app
        .identity
        .meal_preferences
        .load(user_id)
        .await?
        .household_size;
    let state = app.core.shopping.state(user_id, household_size).await?;

    let ingredients: Vec<(String, Vec<Ingredient>)> = to_categories(&state.ingredients);
    let recipes = app.core.recipe.filter_by_ids(state.recipe_ids).await?;

    let (from_date, to_date) = Some((state.from_date, state.days))
        .filter(|(from, days)| *from > 0 && *days > 0)
        .and_then(|(from, days)| {
            let from = u64_to_date(from)?;
            let to = from + time::Duration::days(days as i64 - 1);
            Some((from.unix_timestamp() as u64, to.unix_timestamp() as u64))
        })
        .unwrap_or_default();

    let checked: HashSet<String> = state.checked;

    let total_items: usize = ingredients.iter().map(|(_, items)| items.len()).sum();
    let checked_items = checked.len();
    let progress_pct = (checked_items * 100).checked_div(total_items).unwrap_or(0);

    let aisles: Vec<AisleSection> = ingredients
        .into_iter()
        .map(|(name, items)| {
            let total = items.len();
            let checked_count = items.iter().filter(|i| checked.contains(&i.key())).count();
            let pct = (checked_count * 100).checked_div(total).unwrap_or(0);
            AisleSection {
                name,
                items,
                checked: checked_count,
                total,
                done: total > 0 && checked_count == total,
                pct,
            }
        })
        .collect();

    let split_at = balanced_split(&aisles);

    Ok(ShoppingView {
        recipes,
        split_at,
        checked,
        aisles,
        from_date,
        to_date,
        total_items,
        checked_items,
        progress_pct,
    })
}

/// Choose where the right desktop column starts. Aisles keep their route order;
/// the split is the contiguous point that most evenly divides the total item
/// count between the two columns. E.g. counts `[2, 54, 6, 4, 39, 2, 1]` split
/// after index 2 → `[2, 54]` (56) and `[6, 4, 39, 2, 1]` (52). Both columns are
/// always non-empty (for 2+ aisles).
fn balanced_split(aisles: &[AisleSection]) -> usize {
    let n = aisles.len();
    if n <= 1 {
        return n;
    }
    let total: usize = aisles.iter().map(|a| a.total).sum();
    let mut left = 0usize;
    let mut best_split = 1;
    let mut best_diff = usize::MAX;
    // Consider splitting after each aisle except the last, so both columns are
    // non-empty; the split index is `i + 1`.
    for (i, aisle) in aisles[..n - 1].iter().enumerate() {
        left += aisle.total;
        let diff = left.abs_diff(total - left);
        if diff < best_diff {
            best_diff = diff;
            best_split = i + 1;
        }
    }
    best_split
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn page(
    template: Template,
    user: AuthUser,
    State(app): State<AppState>,
) -> impl IntoResponse {
    let view = imkitchen_web_shared::try_page_response!(build_view(&app, &user.id), template);

    template
        .render(GroceriesTemplate {
            user,
            recipes: view.recipes,
            checked: view.checked,
            aisles: view.aisles,
            split_at: view.split_at,
            from_date: view.from_date,
            to_date: view.to_date,
            total_items: view.total_items,
            checked_items: view.checked_items,
            progress_pct: view.progress_pct,
            ..Default::default()
        })
        .into_response()
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn remove_recipe_action(
    template: Template,
    user: AuthUser,
    State(app): State<AppState>,
    Path((id,)): Path<(String,)>,
) -> impl IntoResponse {
    let preferences = imkitchen_web_shared::try_response!(anyhow:
        app.identity.meal_preferences.load(&user.id),
        template
    );
    imkitchen_web_shared::try_response!(
        app.core
            .shopping
            .remove_recipe(&id, preferences.household_size, &user.id),
        template
    );

    let view = imkitchen_web_shared::try_response!(anyhow: build_view(&app, &user.id), template);

    template
        .render(GroceriesBodyTemplate {
            recipes: view.recipes,
            checked: view.checked,
            aisles: view.aisles,
            split_at: view.split_at,
            total_items: view.total_items,
            checked_items: view.checked_items,
            progress_pct: view.progress_pct,
        })
        .into_response()
}

#[derive(Deserialize, Default, Clone)]
pub struct ToggleJson {
    pub name: String,
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn toggle_action(
    template: Template,
    user: AuthUser,
    State(app): State<AppState>,
    Json(input): Json<ToggleJson>,
) -> impl IntoResponse {
    imkitchen_web_shared::try_response!(
        app.core
            .shopping
            .toggle(ToggleInput { name: input.name }, &user.id),
        template
    );

    "<div></div>".into_response()
}

fn u64_to_date(date: u64) -> Option<time::OffsetDateTime> {
    let year = (date / 10000) as i32;
    let month = ((date % 10000) / 100) as u8;
    let day = (date % 100) as u8;
    let month = time::Month::try_from(month).ok()?;
    let d = time::Date::from_calendar_date(year, month, day).ok()?;
    Some(time::PrimitiveDateTime::new(d, time::Time::MIDNIGHT).assume_utc())
}

fn to_categories(ingredients: &[Ingredient]) -> Vec<(String, Vec<Ingredient>)> {
    let mut categories = HashMap::new();
    let mut ingredients = ingredients.to_vec();
    ingredients.sort_by_key(|i| i.name.to_owned());

    for ingredient in ingredients.iter() {
        match &ingredient.category {
            Some(c) => {
                let entry = categories.entry(format!("shopping_{c}")).or_insert(vec![]);
                entry.push(ingredient.clone());
            }
            _ => {
                let entry = categories
                    .entry("shopping_Unknown".to_owned())
                    .or_insert(vec![]);
                entry.push(ingredient.clone());
            }
        };
    }

    let mut categories = categories
        .into_iter()
        .collect::<Vec<(String, Vec<Ingredient>)>>();

    categories.sort_by_key(|(k, _)| k.to_owned());

    categories
}

#[derive(askama::Template)]
#[template(path = "partials/groceries-generate-modal.html")]
pub struct GenerateModalTemplate {
    pub from: String,
    pub to: String,
}

pub async fn generate_modal(template: Template, user: AuthUser) -> impl IntoResponse {
    let tomorrow = imkitchen_core::mealplan::now(&user.tz) + time::Duration::days(1);
    let to = tomorrow + time::Duration::days(6);
    let from = format!(
        "{}-{:02}-{:02}",
        tomorrow.year(),
        tomorrow.month() as u8,
        tomorrow.day()
    );
    let to = format!("{}-{:02}-{:02}", to.year(), to.month() as u8, to.day());
    template.render(GenerateModalTemplate { from, to })
}

#[derive(askama::Template)]
#[template(path = "partials/groceries-generate-button.html")]
pub struct GenerateButtonTemplate {
    pub status: TemplateStatus,
}

#[derive(Deserialize, Debug)]
pub struct GenerateAction {
    pub from: String,
    pub to: String,
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn generate_action(
    template: Template,
    State(app): State<AppState>,
    RequirePremium(user): RequirePremium,
    Form(input): Form<GenerateAction>,
) -> impl IntoResponse {
    let preferences = imkitchen_web_shared::try_response!(anyhow:
        app.identity.meal_preferences.load(&user.id),
        template
    );
    let from_date: u64 = imkitchen_web_shared::try_response!(sync anyhow:
        input.from.replace('-', "").parse().map_err(|e| anyhow::anyhow!("invalid from date: {e}")),
        template
    );
    let to_date: u64 = imkitchen_web_shared::try_response!(sync anyhow:
        input.to.replace('-', "").parse().map_err(|e| anyhow::anyhow!("invalid to date: {e}")),
        template
    );
    let from_dt = imkitchen_web_shared::try_response!(sync anyhow:
        u64_to_date(from_date).ok_or_else(|| anyhow::anyhow!("invalid from date")),
        template
    );
    let to_dt = imkitchen_web_shared::try_response!(sync anyhow:
        u64_to_date(to_date).ok_or_else(|| anyhow::anyhow!("invalid to date")),
        template
    );
    let days = ((to_dt - from_dt).whole_days() + 1).max(1) as u8;
    imkitchen_web_shared::try_response!(
        app.core.shopping.generate(
            Generate {
                household_size: preferences.household_size,
                date: from_date,
                days
            },
            &user.id
        ),
        template
    );

    template
        .render(GenerateButtonTemplate {
            status: TemplateStatus::Pending,
        })
        .into_response()
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn generate_status(
    template: Template,
    State(app): State<AppState>,
    user: AuthUser,
) -> impl IntoResponse {
    let q_generated_at =
        imkitchen_web_shared::try_response!(anyhow: app.core.shopping.find(&user.id),
            template,
            Some(GenerateButtonTemplate{status: TemplateStatus::Idle})
        )
        .map(|s| s.generated_at);

    let c_generated_at =
        imkitchen_web_shared::try_response!(anyhow: app.core.shopping.load(&user.id),
            template,
            Some(GenerateButtonTemplate{status: TemplateStatus::Idle})
        )
        .map(|s| s.generated_at);

    if q_generated_at == c_generated_at {
        return Redirect::to("/groceries").into_response();
    }

    template
        .render(GenerateButtonTemplate {
            status: TemplateStatus::Checking,
        })
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::{AisleSection, balanced_split};

    fn aisle(total: usize) -> AisleSection {
        AisleSection {
            name: format!("a{total}"),
            items: vec![],
            checked: 0,
            total,
            done: false,
            pct: 0,
        }
    }

    fn split(totals: Vec<usize>) -> (usize, usize, usize) {
        let aisles: Vec<AisleSection> = totals.into_iter().map(aisle).collect();
        let split_at = balanced_split(&aisles);
        let left: usize = aisles[..split_at].iter().map(|a| a.total).sum();
        let right: usize = aisles[split_at..].iter().map(|a| a.total).sum();
        (split_at, left, right)
    }

    #[test]
    fn contiguous_split_balances_item_counts() {
        // [2, 54, 6, 4, 39, 2, 1] → after index 2: [2,54]=56 and rest=52.
        let (split_at, left, right) = split(vec![2, 54, 6, 4, 39, 2, 1]);
        assert_eq!(split_at, 2);
        assert_eq!((left, right), (56, 52));
    }

    #[test]
    fn even_sizes_split_in_the_middle() {
        let (split_at, left, right) = split(vec![3, 3, 3, 3]);
        assert_eq!(split_at, 2);
        assert_eq!((left, right), (6, 6));
    }

    #[test]
    fn keeps_both_columns_non_empty() {
        let (split_at, _, _) = split(vec![10, 1]);
        assert_eq!(split_at, 1);
    }
}

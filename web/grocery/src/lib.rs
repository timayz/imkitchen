use axum::{
    extract::{Json, State},
    response::{IntoResponse, Redirect},
};
use axum_extra::extract::Form;
use imkitchen_core::shopping::{Generate, ToggleInput};
use imkitchen_types::recipe::{Ingredient, IngredientUnitFormat};
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
}

#[derive(askama::Template)]
#[template(path = "groceries.html")]
pub struct GroceriesTemplate {
    pub current_path: String,
    pub user: AuthUser,
    pub checked: HashSet<String>,
    pub ingredients: Vec<(String, Vec<Ingredient>)>,
    pub from_date: u64,
    pub to_date: u64,
}

impl Default for GroceriesTemplate {
    fn default() -> Self {
        Self {
            current_path: "groceries".to_owned(),
            user: AuthUser::default(),
            checked: HashSet::default(),
            ingredients: vec![],
            from_date: 0,
            to_date: 0,
        }
    }
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn page(
    template: Template,
    user: AuthUser,
    State(app): State<AppState>,
) -> impl IntoResponse {
    let list = imkitchen_web_shared::try_page_response!(app.core.shopping.find(&user.id), template);
    let ingredients = list
        .as_ref()
        .map(|r| to_categories(&r.ingredients.0))
        .unwrap_or_default();

    let (from_date, to_date) = list
        .as_ref()
        .filter(|r| r.from_date > 0 && r.days > 0)
        .and_then(|r| {
            let from = u64_to_date(r.from_date)?;
            let to = from + time::Duration::days(r.days as i64 - 1);
            Some((from.unix_timestamp() as u64, to.unix_timestamp() as u64))
        })
        .unwrap_or_default();

    let checked =
        imkitchen_web_shared::try_page_response!(app.core.shopping.load(&user.id), template)
            .map(|loaded| loaded.checked)
            .unwrap_or_default();

    template
        .render(GroceriesTemplate {
            user,
            checked,
            ingredients,
            from_date,
            to_date,
            ..Default::default()
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

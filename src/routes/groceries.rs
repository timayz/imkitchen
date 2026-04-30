use axum::{
    extract::{Json, State},
    response::{IntoResponse, Redirect},
};
use axum_extra::extract::Form;
use imkitchen_types::recipe::{Ingredient, IngredientUnitFormat};
use imkitchen_core::shopping::{Generate, ToggleInput};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};

use crate::{
    auth::AuthUser,
    routes::AppState,
    template::{Status as TemplateStatus, Template, filters},
};

#[derive(askama::Template)]
#[template(path = "groceries.html")]
pub struct GroceriesTemplate {
    pub current_path: String,
    pub user: AuthUser,
    pub checked: HashSet<String>,
    pub ingredients: Vec<(String, Vec<Ingredient>)>,
}

impl Default for GroceriesTemplate {
    fn default() -> Self {
        Self {
            current_path: "groceries".to_owned(),
            user: AuthUser::default(),
            checked: HashSet::default(),
            ingredients: vec![],
        }
    }
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn page(
    template: Template,
    user: AuthUser,
    State(app): State<AppState>,
) -> impl IntoResponse {
    let list = crate::try_page_response!(app.core.shopping.find(&user.id), template);
    let ingredients = list
        .as_ref()
        .map(|r| to_categories(&r.ingredients.0))
        .unwrap_or_default();

    let checked = crate::try_page_response!(app.core.shopping.load(&user.id), template)
        .map(|loaded| loaded.checked)
        .unwrap_or_default();

    template
        .render(GroceriesTemplate {
            user,
            checked,
            ingredients,
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
    crate::try_response!(
        app.core.shopping
            .toggle(ToggleInput { name: input.name }, &user.id),
        template
    );

    "<div></div>".into_response()
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
pub struct GenerateModalTemplate;

pub async fn generate_modal(template: Template) -> impl IntoResponse {
    template.render(GenerateModalTemplate)
}

#[derive(askama::Template)]
#[template(path = "partials/groceries-generate-button.html")]
pub struct GenerateButtonTemplate {
    pub status: TemplateStatus,
}

#[derive(Deserialize, Debug)]
pub struct GenerateAction {
    pub days: u8,
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn generate_action(
    template: Template,
    State(app): State<AppState>,
    user: AuthUser,
    Form(input): Form<GenerateAction>,
) -> impl IntoResponse {
    let preferences = crate::try_response!(anyhow:
        app.identity.meal_preferences.load(&user.id),
        template
    );
    let date = imkitchen_core::mealplan::date_to_u64(imkitchen_core::mealplan::now(&user.tz));
    crate::try_response!(
        app.core.shopping.generate(
            Generate {
                household_size: preferences.household_size,
                date,
                days: input.days
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
    let q_generated_at = crate::try_response!(anyhow: app.core.shopping.find(&user.id),
        template,
        Some(GenerateButtonTemplate{status: TemplateStatus::Idle})
    )
    .map(|s| s.generated_at);

    let c_generated_at = crate::try_response!(anyhow: app.core.shopping.load(&user.id),
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

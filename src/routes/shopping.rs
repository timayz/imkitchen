use std::collections::{HashMap, HashSet};

use axum::{
    extract::{Json, Path, State},
    response::IntoResponse,
};
use imkitchen_mealplan::week::{WeekListRow, WeekRow};
use imkitchen_shared::recipe::{Ingredient, IngredientUnitFormat};
use imkitchen_shopping::{ToggleInput, list::ListWeekRow};
use serde::Deserialize;

use crate::{
    auth::AuthUser,
    routes::AppState,
    template::{Template, filters},
};

#[derive(askama::Template)]
#[template(path = "shopping.html")]
pub struct ShoppingTemplate {
    pub current_path: String,
    pub user: AuthUser,
    pub weeks: Vec<WeekListRow>,
    pub current: Option<ListWeekRow>,
    pub recipes: Option<Vec<String>>,
    pub checked: Option<HashSet<String>>,
    pub ingredients: Option<Vec<(String, Vec<Ingredient>)>>,
    pub index: u8,
}

impl Default for ShoppingTemplate {
    fn default() -> Self {
        Self {
            current_path: "calendar".to_owned(),
            user: AuthUser::default(),
            weeks: Default::default(),
            current: None,
            recipes: None,
            checked: None,
            ingredients: None,
            index: 0,
        }
    }
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn page(
    template: Template,
    user: AuthUser,
    State(app): State<AppState>,
    Path((mut index,)): Path<(u8,)>,
) -> impl IntoResponse {
    let week_from_now = imkitchen_mealplan::current_and_next_four_weeks_from_now()[0];
    let weeks = crate::try_page_response!(
        imkitchen_mealplan::week::filter_last_from(&app.read_db, week_from_now.start, &user.id),
        template
    );

    if index == 0 {
        index += 1;
    }

    let current = match weeks.get((index - 1) as usize) {
        Some(week) => {
            crate::try_page_response!(
                imkitchen_shopping::list::next_from(&app.read_db, week.start, &user.id),
                template
            )
        }
        _ => None,
    };

    let ingredients = current.as_ref().map(|r| to_categories(&r.ingredients.0));

    let checked = match weeks.get((index - 1) as usize) {
        Some(week) => {
            crate::try_page_response!(imkitchen_shopping::load(&app.executor, &user.id), template)
                .and_then(|loaded| loaded.checked.get(&week.start).cloned())
        }
        _ => None,
    };

    let recipes = match weeks.get((index - 1) as usize) {
        Some(week) => crate::try_page_response!(
            imkitchen_mealplan::week::find_from_unix_timestamp(&app.read_db, week.start, &user.id),
            template
        )
        .and_then(|week| Some(to_recipes(week))),
        _ => None,
    };

    template
        .render(ShoppingTemplate {
            user,
            weeks,
            recipes,
            current,
            index,
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
    Path((week,)): Path<(u64,)>,
    Json(input): Json<ToggleJson>,
) -> impl IntoResponse {
    let current = crate::try_page_response!(
        imkitchen_shopping::list::next_from(&app.read_db, week, &user.id),
        template
    );
    let ingredients = current.as_ref().map(|r| to_categories(&r.ingredients.0));

    let checked = if current.is_some() {
        let shopping = crate::try_response!(anyhow_opt: imkitchen_shopping::load(&app.executor,&user.id), template);
        crate::try_response!(
            shopping.toggle(
                ToggleInput {
                    week,
                    name: input.name,
                },
                &user.id,
            ),
            template
        );
        crate::try_response!(anyhow: imkitchen_shopping::load(&app.executor,&user.id), template)
            .and_then(|loaded| loaded.checked.get(&week).cloned())
    } else {
        None
    };

    template
        .render(ShoppingTemplate {
            user,
            current,
            ingredients,
            checked,
            ..Default::default()
        })
        .into_response()
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn reset_all_action(
    template: Template,
    user: AuthUser,
    State(app): State<AppState>,
    Path((week,)): Path<(u64,)>,
) -> impl IntoResponse {
    let current = crate::try_page_response!(
        imkitchen_shopping::list::next_from(&app.read_db, week, &user.id),
        template
    );
    let ingredients = current.as_ref().map(|r| to_categories(&r.ingredients.0));
    let recipes = crate::try_page_response!(
        imkitchen_mealplan::week::find_from_unix_timestamp(&app.read_db, week, &user.id),
        template
    )
    .and_then(|week| Some(to_recipes(week)));

    if current.is_some() {
        let shopping = crate::try_response!(anyhow_opt: imkitchen_shopping::load(&app.executor,&user.id), template);
        crate::try_response!(shopping.reset(week, &user.id), template);
    }

    template
        .render(ShoppingTemplate {
            user,
            current,
            ingredients,
            recipes,
            ..Default::default()
        })
        .into_response()
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

fn to_recipes(week: WeekRow) -> Vec<String> {
    let mut recipes = HashSet::new();
    for slot in week.slots.iter() {
        recipes.insert(format!("🍛 {}", slot.main_course.name));

        if let Some(ref r) = slot.appetizer {
            recipes.insert(format!("🥗 {}", r.name));
        }

        if let Some(ref r) = slot.accompaniment {
            recipes.insert(format!("🍚 {}", r.name));
        }

        if let Some(ref r) = slot.dessert {
            recipes.insert(format!("🍰 {}", r.name));
        }
    }

    let mut recipes = recipes.into_iter().collect::<Vec<_>>();
    recipes.sort();

    recipes
}

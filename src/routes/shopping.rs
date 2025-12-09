use std::collections::HashSet;

use axum::{
    extract::{Json, Path, State},
    response::IntoResponse,
};
use imkitchen_mealplan::WeekListRow;
use imkitchen_recipe::IngredientUnitFormat;
use imkitchen_shared::Metadata;
use imkitchen_shopping::{ListWeekRow, ToggleInput};
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
    pub user: imkitchen_user::AuthUser,
    pub weeks: Vec<WeekListRow>,
    pub current: Option<ListWeekRow>,
    pub recipes: Option<HashSet<String>>,
    pub checked: Option<HashSet<String>>,
    pub index: u8,
}

impl Default for ShoppingTemplate {
    fn default() -> Self {
        Self {
            current_path: "calendar".to_owned(),
            user: imkitchen_user::AuthUser::default(),
            weeks: Default::default(),
            current: None,
            recipes: None,
            checked: None,
            index: 0,
        }
    }
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn page(
    template: Template,
    AuthUser(user): AuthUser,
    State(app): State<AppState>,
    Path((mut index,)): Path<(u8,)>,
) -> impl IntoResponse {
    let week_from_now = imkitchen_mealplan::current_and_next_four_weeks_from_now()[0];
    let weeks = crate::try_page_response!(
        app.mealplan_query
            .filter_last_from(week_from_now.start, &user.id),
        template
    );

    if index == 0 {
        index += 1;
    }

    let current = match weeks.get((index - 1) as usize) {
        Some(week) => {
            crate::try_page_response!(app.shopping_query.next_from(week.start, &user.id), template)
                .map(|mut r| {
                    r.ingredients.sort_by_key(|i| i.name.to_owned());
                    r
                })
        }
        _ => None,
    };

    let checked = match weeks.get((index - 1) as usize) {
        Some(week) => crate::try_page_response!(app.shopping_command.load(&user.id), template)
            .and_then(|loaded| loaded.item.checked.get(&week.start).cloned()),
        _ => None,
    };

    let recipes = match weeks.get((index - 1) as usize) {
        Some(week) => crate::try_page_response!(
            app.mealplan_query
                .find_from_unix_timestamp(week.start, &user.id),
            template
        )
        .and_then(|week| {
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

            Some(recipes)
        }),
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
    AuthUser(user): AuthUser,
    State(app): State<AppState>,
    Path((week,)): Path<(u64,)>,
    Json(input): Json<ToggleJson>,
) -> impl IntoResponse {
    let current = crate::try_page_response!(app.shopping_query.next_from(week, &user.id), template)
        .map(|mut r| {
            r.ingredients.sort_by_key(|i| i.name.to_owned());
            r
        });

    let checked = if current.is_some() {
        crate::try_response!(
            app.shopping_command.toggle(
                ToggleInput {
                    week,
                    name: input.name,
                },
                &Metadata::by(user.id.to_owned()),
            ),
            template
        );
        crate::try_page_response!(app.shopping_command.load(&user.id), template)
            .and_then(|loaded| loaded.item.checked.get(&week).cloned())
    } else {
        None
    };

    template
        .render(ShoppingTemplate {
            user,
            current,
            checked,
            ..Default::default()
        })
        .into_response()
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn reset_all_action(
    template: Template,
    AuthUser(user): AuthUser,
    State(app): State<AppState>,
    Path((week,)): Path<(u64,)>,
) -> impl IntoResponse {
    let current = crate::try_page_response!(app.shopping_query.next_from(week, &user.id), template)
        .map(|mut r| {
            r.ingredients.sort_by_key(|i| i.name.to_owned());
            r
        });

    if current.is_some() {
        crate::try_response!(
            app.shopping_command
                .reset(week, &Metadata::by(user.id.to_owned())),
            template
        );
    }

    template
        .render(ShoppingTemplate {
            user,
            current,
            ..Default::default()
        })
        .into_response()
}

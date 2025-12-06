use std::collections::HashSet;

use axum::{
    extract::{Json, Path, State},
    response::IntoResponse,
};
use imkitchen_mealplan::WeekListRow;
use imkitchen_shared::Metadata;
use imkitchen_shopping::{ListWeekRow, ToggleInput};
use serde::Deserialize;

use crate::{
    auth::AuthUser,
    routes::AppState,
    template::{ServerErrorTemplate, Template, filters},
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

pub async fn page(
    template: Template<ShoppingTemplate>,
    server_error: Template<ServerErrorTemplate>,
    AuthUser(user): AuthUser,
    State(app): State<AppState>,
    Path((mut index,)): Path<(u8,)>,
) -> impl IntoResponse {
    let week_from_now = imkitchen_mealplan::current_and_next_four_weeks_from_now()[0];
    let weeks = match app
        .mealplan_query
        .filter_last_from(week_from_now.start, &user.id)
        .await
    {
        Ok(weeks) => weeks,
        Err(err) => {
            tracing::error!(
                user = user.id,
                index = index,
                err = %err,
                "failed to get current_and_next_four_weeks_from_now on calendar page"
            );

            return server_error.render(ServerErrorTemplate).into_response();
        }
    };

    if index == 0 {
        index += 1;
    }

    let current = match weeks.get((index - 1) as usize) {
        Some(week) => match app.shopping_query.next_from(week.start, &user.id).await {
            Ok(Some(mut row)) => {
                row.ingredients.sort_by_key(|i| i.name.to_owned());
                Some(row)
            }
            Ok(_) => None,
            Err(err) => {
                tracing::error!(
                    user = user.id,
                    index = index,
                    err = %err,
                    "failed to get next_shopping_list_from on shopping list page"
                );

                return server_error.render(ServerErrorTemplate).into_response();
            }
        },
        _ => None,
    };

    let checked = match weeks.get((index - 1) as usize) {
        Some(week) => match app.shopping_command.load(&user.id).await {
            Ok(Some(loaded)) => loaded.item.checked.get(&week.start).cloned(),
            Ok(_) => None,
            Err(err) => {
                tracing::error!(
                    user = user.id,
                    index = index,
                    err = %err,
                    "failed to get next_shopping_list_from on shopping list page"
                );

                return server_error.render(ServerErrorTemplate).into_response();
            }
        },
        _ => None,
    };

    let recipes = match weeks.get((index - 1) as usize) {
        Some(week) => match app
            .mealplan_query
            .find_from_unix_timestamp(week.start, &user.id)
            .await
        {
            Ok(Some(week)) => {
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
            }
            Ok(_) => None,
            Err(err) => {
                tracing::error!(
                    user = user.id,
                    index = index,
                    err = %err,
                    "failed to get next_shopping_list_from on shopping list page"
                );

                return server_error.render(ServerErrorTemplate).into_response();
            }
        },
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

pub async fn toggle_action(
    server_error: Template<ServerErrorTemplate>,
    AuthUser(user): AuthUser,
    State(app): State<AppState>,
    Path((week,)): Path<(u64,)>,
    Json(input): Json<ToggleJson>,
) -> impl IntoResponse {
    match app
        .shopping_command
        .toggle(
            ToggleInput {
                week,
                name: input.name,
            },
            &Metadata::by(user.id.to_owned()),
        )
        .await
    {
        Ok(_) => "<div></div>".into_response(),
        Err(err) => {
            tracing::error!(
                user = user.id,
                week = week,
                err = %err,
                "failed to toggle ingredient"
            );

            server_error.render(ServerErrorTemplate).into_response()
        }
    }
}

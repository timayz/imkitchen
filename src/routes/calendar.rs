use axum::{
    extract::State,
    response::{IntoResponse, Redirect},
};
use imkitchen_mealplan::Status;
use imkitchen_shared::Metadata;

use crate::{
    auth::AuthUser,
    routes::AppState,
    template::{SERVER_ERROR_MESSAGE, Template, filters},
};

#[derive(askama::Template)]
#[template(path = "calendar-regenerate-modal.html")]
pub struct RegenerateModalTemplate;

#[derive(askama::Template)]
#[template(path = "calendar-regenerate-status.html")]
pub struct RegenerateStatusTemplate;

#[derive(askama::Template)]
#[template(path = "calendar-regenerate.html")]
pub struct RegenerateTemplate {
    pub error_message: Option<String>,
}

#[derive(askama::Template)]
#[template(path = "calendar.html")]
pub struct CalendarTemplate {
    pub current_path: String,
    pub user: imkitchen_user::AuthUser,
}

impl Default for CalendarTemplate {
    fn default() -> Self {
        Self {
            current_path: "calendar".to_owned(),
            user: imkitchen_user::AuthUser::default(),
        }
    }
}

pub async fn page(
    template: Template<CalendarTemplate>,
    AuthUser(user): AuthUser,
) -> impl IntoResponse {
    template.render(CalendarTemplate {
        user,
        ..Default::default()
    })
}

pub async fn regenerate_action(
    template: Template<RegenerateTemplate>,
    State(app): State<AppState>,
    AuthUser(user): AuthUser,
) -> impl IntoResponse {
    let status = match app.mealplan_command.load_optional(&user.id).await {
        Ok(Some(r)) => r.item.status,
        Ok(_) => Status::Idle,
        Err(err) => {
            tracing::error!(user = user.id, err = %err,"Failed to regenerate meal plan");

            return template.render(RegenerateTemplate {
                error_message: Some(SERVER_ERROR_MESSAGE.to_owned()),
            });
        }
    };

    if status == Status::Processing {
        return template.render(RegenerateTemplate {
            error_message: Some("Invalid status".to_owned()),
        });
    }

    match app
        .mealplan_command
        .generate(&Metadata::by(user.id.to_owned()))
        .await
    {
        Ok(_) => template.render(RegenerateTemplate {
            error_message: None,
        }),
        Err(err) => {
            tracing::error!(user = user.id, err = %err, "Failed to regenerate meal plan");

            template.render(RegenerateTemplate {
                error_message: Some(SERVER_ERROR_MESSAGE.to_owned()),
            })
        }
    }
}

pub async fn regenerate_status(
    template: Template<RegenerateStatusTemplate>,
    State(app): State<AppState>,
    AuthUser(user): AuthUser,
) -> impl IntoResponse {
    let meal_plan = match app.mealplan_command.load_optional(&user.id).await {
        Ok(Some(loaded)) => loaded,
        Ok(_) => return template.render(RegenerateStatusTemplate).into_response(),
        Err(err) => {
            tracing::error!(user = user.id, err = %err,"Failed to check meal plan regeneration status");

            return Redirect::to("/calendar").into_response();
        }
    };

    if meal_plan.item.status == Status::Failed {
        return Redirect::to("/calendar").into_response();
    }

    let mondays = match imkitchen_mealplan::next_four_mondays_from_now() {
        Ok(m) => m,
        Err(err) => {
            tracing::error!(user = user.id, err = %err,"Failed to next four mondays when meal plan regeneration status");

            return Redirect::to("/calendar").into_response();
        }
    };

    let week = match app.mealplan_query.find(mondays[0], &user.id).await {
        Ok(Some(week)) => week,
        Ok(_) => return template.render(RegenerateStatusTemplate).into_response(),
        Err(err) => {
            tracing::error!(user = user.id, err = %err,"Failed to check meal plan regeneration status");

            return Redirect::to("/calendar").into_response();
        }
    };

    if week.status.0 == Status::Processing {
        return template.render(RegenerateStatusTemplate).into_response();
    }

    Redirect::to("/calendar").into_response()
}

pub async fn regenerate_modal(template: Template<RegenerateModalTemplate>) -> impl IntoResponse {
    template.render(RegenerateModalTemplate)
}

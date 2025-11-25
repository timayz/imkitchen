use axum::response::IntoResponse;

use crate::{
    auth::AuthUser,
    template::{Template, filters},
};

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

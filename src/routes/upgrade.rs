use axum::response::{IntoResponse, Redirect};

use crate::{
    auth::AuthUser,
    template::{Template, filters},
};

#[derive(askama::Template)]
#[template(path = "upgrade.html")]
pub struct UpgradeTemplate {
    pub current_path: String,
    pub user: AuthUser,
}

impl Default for UpgradeTemplate {
    fn default() -> Self {
        Self {
            current_path: "profile".to_owned(),
            user: Default::default(),
        }
    }
}

pub async fn page(template: Template, user: AuthUser) -> impl IntoResponse {
    if user.is_premium() {
        return Redirect::to("/profile/subscription").into_response();
    }

    template
        .render(UpgradeTemplate {
            user,
            ..Default::default()
        })
        .into_response()
}

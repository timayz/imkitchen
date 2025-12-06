use axum::response::IntoResponse;

use crate::{
    auth::AuthUser,
    template::{Template, filters},
};

#[derive(askama::Template)]
#[template(path = "community.html")]
pub struct CommunityTemplate {
    pub current_path: String,
    pub user: imkitchen_user::AuthUser,
}

impl Default for CommunityTemplate {
    fn default() -> Self {
        Self {
            current_path: "community".to_owned(),
            user: imkitchen_user::AuthUser::default(),
        }
    }
}

pub async fn page(template: Template, AuthUser(user): AuthUser) -> impl IntoResponse {
    template.render(CommunityTemplate {
        user,
        ..Default::default()
    })
}

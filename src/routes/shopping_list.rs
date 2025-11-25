use axum::response::IntoResponse;

use crate::{
    auth::AuthUser,
    template::{Template, filters},
};

#[derive(askama::Template)]
#[template(path = "shopping-list.html")]
pub struct ShoppingListTemplate {
    pub current_path: String,
    pub user: imkitchen_user::AuthUser,
}

impl Default for ShoppingListTemplate {
    fn default() -> Self {
        Self {
            current_path: "calendar".to_owned(),
            user: imkitchen_user::AuthUser::default(),
        }
    }
}

pub async fn page(
    template: Template<ShoppingListTemplate>,
    AuthUser(user): AuthUser,
) -> impl IntoResponse {
    template.render(ShoppingListTemplate {
        user,
        ..Default::default()
    })
}

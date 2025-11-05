use axum::response::IntoResponse;

use crate::auth::AuthUser;
use crate::filters;
use crate::template::Template;

#[derive(askama::Template)]
#[template(path = "index.html")]
pub struct IndexTemplate;

pub async fn page(
    template: Template<IndexTemplate>,
    AuthUser(_user_id): AuthUser,
) -> impl IntoResponse {
    template.render(IndexTemplate)
}

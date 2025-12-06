use axum::{extract::Path, response::IntoResponse};

use crate::template::{Template, filters};

#[derive(askama::Template)]
#[template(path = "coming-soon.html")]
pub struct KitchenTemplate {
    pub _current_path: String,
}

impl Default for KitchenTemplate {
    fn default() -> Self {
        Self {
            _current_path: "dashboard".to_owned(),
        }
    }
}

pub async fn page(template: Template, Path((_id,)): Path<(String,)>) -> impl IntoResponse {
    template
        .render(KitchenTemplate {
            ..Default::default()
        })
        .into_response()
}

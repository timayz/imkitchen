use askama::Template;
use axum::response::Html;

#[derive(Template)]
#[template(path = "pages/home.html")]
pub struct HomeTemplate {
    pub title: String,
}

pub async fn home_page() -> Html<String> {
    let template = HomeTemplate {
        title: "Welcome to ImKitchen".to_string(),
    };

    Html(template.render().unwrap())
}

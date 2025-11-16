use axum::{
    RequestPartsExt,
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
    response::{Html, IntoResponse, Response},
};
use std::{collections::HashMap, convert::Infallible};

use crate::language::UserLanguage;

pub const SERVER_ERROR_MESSAGE: &str = "Something went wrong, please retry later";
pub const NOT_FOUND: &str = "Not found";
pub const FORBIDDEN: &str = "Forbidden";

pub(crate) mod filters {
    pub fn t(value: &str, values: &dyn askama::Values) -> askama::Result<String> {
        let preferred_language = askama::get_value::<String>(values, "preferred_language")
            .expect("Unable to get preferred_language from askama::get_value");

        Ok(rust_i18n::t!(value, locale = preferred_language).to_string())
    }

    // pub fn assets(value: &str, values: &dyn askama::Values) -> askama::Result<String> {
    //     let config = askama::get_value::<crate::axum_extra::TemplateConfig>(values, "config")
    //         .expect("Unable to get config from askama::get_value");
    //
    //     Ok(format!("{}/{value}", config.assets_base_url))
    // }
}

pub struct Template<T> {
    template: Option<T>,
    preferred_language: String,
    preferred_language_iso: String,
    config: crate::config::Config,
}

impl<T> Template<T> {
    pub fn render(mut self, t: T) -> Self {
        self.template = Some(t);

        self
    }
}

impl<T> FromRequestParts<crate::routes::AppState> for Template<T> {
    type Rejection = Infallible;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &crate::routes::AppState,
    ) -> Result<Self, Self::Rejection> {
        let user_language = parts
            .extract::<UserLanguage>()
            .await
            .expect("Unable to extract user languages");

        let preferred_language = user_language
            .preferred_languages()
            .first()
            .cloned()
            .unwrap_or_else(|| "en".to_owned());

        let preferred_language_iso = preferred_language
            .split_once("-")
            .unwrap_or((preferred_language.as_str(), ""))
            .0
            .to_owned();

        Ok(Template {
            template: None,
            preferred_language,
            preferred_language_iso,
            config: state.config.clone(),
        })
    }
}

impl<T> IntoResponse for Template<T>
where
    T: askama::Template,
{
    fn into_response(self) -> Response {
        let mut values: HashMap<&str, Box<dyn std::any::Any>> = HashMap::new();
        values.insert("preferred_language", Box::new(self.preferred_language));
        values.insert(
            "preferred_language_iso",
            Box::new(self.preferred_language_iso),
        );
        values.insert("config", Box::new(self.config));

        #[cfg(debug_assertions)]
        {
            values.insert("is_dev", Box::new(true));
        }
        #[cfg(not(debug_assertions))]
        {
            values.insert("is_dev", Box::new(false));
        }

        match self
            .template
            .expect("template must be define using template.template(..)")
            .render_with_values(&values)
        {
            Ok(html) => Html(html).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template. Error: {err}"),
            )
                .into_response(),
        }
    }
}

#[derive(askama::Template)]
#[template(path = "404.html")]
pub struct NotFoundTemplate;

#[derive(askama::Template)]
#[template(path = "403.html")]
pub struct ForbiddenTemplate;

#[derive(askama::Template)]
#[template(path = "500.html")]
pub struct ServerErrorTemplate;

use axum::{
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
    response::{Html, IntoResponse, Response},
};
use std::{collections::HashMap, convert::Infallible};

pub struct Template<T> {
    template: Option<T>,
    // preferred_language: String,
    // preferred_language_iso: String,
    config: crate::config::Config,
}

impl<T> Template<T> {
    pub fn render(mut self, t: T) -> Self {
        self.template = Some(t);

        self
    }
}

impl<T> FromRequestParts<crate::server::AppState> for Template<T> {
    type Rejection = Infallible;

    async fn from_request_parts(
        _parts: &mut Parts,
        state: &crate::server::AppState,
    ) -> Result<Self, Self::Rejection> {
        // let user_language = parts
        //     .extract::<UserLanguage>()
        //     .await
        //     .expect("Unable to extract user languages");
        //
        // let preferred_language = user_language
        //     .preferred_languages()
        //     .first()
        //     .cloned()
        //     .unwrap_or_else(|| "en".to_owned());
        //
        // let preferred_language_iso = preferred_language
        //     .split_once("-")
        //     .unwrap_or((preferred_language.as_str(), ""))
        //     .0
        //     .to_owned();

        Ok(Template {
            template: None,
            // preferred_language,
            // preferred_language_iso,
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
        // values.insert("preferred_language", Box::new(self.preferred_language));
        // values.insert(
        //     "preferred_language_iso",
        //     Box::new(self.preferred_language_iso),
        // );
        values.insert("config", Box::new(self.config));

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

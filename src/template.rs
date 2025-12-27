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

pub enum Status {
    Idle,
    Pending,
    Checking,
}

pub(crate) mod filters {
    use time::OffsetDateTime;

    #[askama::filter_fn]
    pub fn t(value: &str, values: &dyn askama::Values) -> askama::Result<String> {
        let preferred_language = askama::get_value::<String>(values, "preferred_language")
            .expect("Unable to get preferred_language from askama::get_value");

        Ok(rust_i18n::t!(value, locale = preferred_language).to_string())
    }

    #[askama::filter_fn]
    pub fn date(value: &u64, values: &dyn askama::Values) -> askama::Result<String> {
        let preferred_language = askama::get_value::<String>(values, "preferred_language")
            .expect("Unable to get preferred_language from askama::get_value");

        let date = OffsetDateTime::from_unix_timestamp(*value as i64)
            .map_err(|e| askama::Error::Custom(Box::new(e)))?;

        let month = rust_i18n::t!(format!("{}_sm", date.month()), locale = preferred_language);
        let weekday = rust_i18n::t!(date.weekday().to_string(), locale = preferred_language);

        Ok(rust_i18n::t!(
            "date_format",
            locale = preferred_language,
            month = month,
            weekday = weekday,
            day = date.day()
        )
        .to_string())
    }

    #[askama::filter_fn]
    pub fn date_year(value: &u64, values: &dyn askama::Values) -> askama::Result<String> {
        let preferred_language = askama::get_value::<String>(values, "preferred_language")
            .expect("Unable to get preferred_language from askama::get_value");

        let date = OffsetDateTime::from_unix_timestamp(*value as i64)
            .map_err(|e| askama::Error::Custom(Box::new(e)))?;

        let month = rust_i18n::t!(format!("{}", date.month()), locale = preferred_language);
        let weekday = rust_i18n::t!(date.weekday().to_string(), locale = preferred_language);

        Ok(rust_i18n::t!(
            "date_year_format",
            locale = preferred_language,
            month = month,
            weekday = weekday,
            day = date.day(),
            year = date.year()
        )
        .to_string())
    }

    #[askama::filter_fn]
    pub fn month_year(a: &u64, values: &dyn askama::Values, b: &u64) -> askama::Result<String> {
        let preferred_language = askama::get_value::<String>(values, "preferred_language")
            .expect("Unable to get preferred_language from askama::get_value");

        let date_a = OffsetDateTime::from_unix_timestamp(*a as i64)
            .map_err(|e| askama::Error::Custom(Box::new(e)))?;

        let date_b = OffsetDateTime::from_unix_timestamp(*b as i64)
            .map_err(|e| askama::Error::Custom(Box::new(e)))?;

        let month_a = rust_i18n::t!(format!("{}", date_a.month()), locale = preferred_language);

        if date_a.month() == date_b.month() {
            return Ok(format!("{month_a} {}", date_a.year()));
        }

        let month_b = rust_i18n::t!(format!("{}", date_b.month()), locale = preferred_language);

        Ok(format!(
            "{month_a} {} - {month_b} {}",
            date_a.year(),
            date_b.year()
        ))
    }

    #[askama::filter_fn]
    pub fn minutes(minutes: &u16, values: &dyn askama::Values) -> askama::Result<String> {
        let preferred_language = askama::get_value::<String>(values, "preferred_language")
            .expect("Unable to get preferred_language from askama::get_value");

        let hours = minutes / 60;
        let remaining_minutes = minutes % 60;

        let value = match (hours, remaining_minutes) {
            (0, m) => format!("{} min", m),
            (1, 0) => format!("1 {}", rust_i18n::t!("hour", locale = preferred_language)),
            (h, 0) => format!(
                "{} {}",
                h,
                rust_i18n::t!("hours", locale = preferred_language)
            ),
            (h, m) => format!("{} h {} min", h, m),
        };

        Ok(value)
    }

    #[askama::filter_fn]
    pub fn weekday(value: &u64, values: &dyn askama::Values) -> askama::Result<String> {
        let preferred_language = askama::get_value::<String>(values, "preferred_language")
            .expect("Unable to get preferred_language from askama::get_value");

        let date = OffsetDateTime::from_unix_timestamp(*value as i64)
            .map_err(|e| askama::Error::Custom(Box::new(e)))?;

        Ok(rust_i18n::t!(date.weekday().to_string(), locale = preferred_language).to_string())
    }

    #[askama::filter_fn]
    pub fn relative_time(timestamp: &u64, values: &dyn askama::Values) -> askama::Result<String> {
        let preferred_language = askama::get_value::<String>(values, "preferred_language")
            .expect("Unable to get preferred_language from askama::get_value");

        let now = OffsetDateTime::now_utc().unix_timestamp() as u64;

        if *timestamp > now {
            return Ok(rust_i18n::t!("just now", locale = preferred_language).to_string());
        }

        let diff = now - timestamp;
        let minutes = diff / 60;
        let hours = diff / 3600;
        let days = diff / 86400;

        let value = match diff {
            s if s < 60 => rust_i18n::t!("just now", locale = preferred_language).to_string(),
            s if s < 3600 => {
                if minutes == 1 {
                    rust_i18n::t!("1 minute ago", locale = preferred_language).to_string()
                } else {
                    rust_i18n::t!("minutes ago", locale = preferred_language, count = minutes)
                        .to_string()
                }
            }
            s if s < 86400 => {
                if hours == 1 {
                    rust_i18n::t!("1 hour ago", locale = preferred_language).to_string()
                } else {
                    rust_i18n::t!("hours ago", locale = preferred_language, count = hours)
                        .to_string()
                }
            }
            s if s < 172800 => rust_i18n::t!("yesterday", locale = preferred_language).to_string(),
            s if s < 604800 => {
                rust_i18n::t!("days ago", locale = preferred_language, count = days).to_string()
            }
            s if s < 2592000 => {
                let weeks = days / 7;
                if weeks == 1 {
                    rust_i18n::t!("1 week ago", locale = preferred_language).to_string()
                } else {
                    rust_i18n::t!("weeks ago", locale = preferred_language, count = weeks)
                        .to_string()
                }
            }
            s if s < 31536000 => {
                let months = days / 30;
                if months == 1 {
                    rust_i18n::t!("1 month ago", locale = preferred_language).to_string()
                } else {
                    rust_i18n::t!("months ago", locale = preferred_language, count = months)
                        .to_string()
                }
            }
            _ => {
                let years = days / 365;
                if years == 1 {
                    rust_i18n::t!("1 year ago", locale = preferred_language).to_string()
                } else {
                    rust_i18n::t!("years ago", locale = preferred_language, count = years)
                        .to_string()
                }
            }
        };

        Ok(value)
    }

    #[askama::filter_fn]
    pub fn views(count: &u64, _values: &dyn askama::Values) -> askama::Result<String> {
        let value = match *count {
            n if n >= 1_000_000_000 => {
                let billions = n as f64 / 1_000_000_000.0;
                if billions >= 10.0 {
                    format!("{}B", (billions.round()) as u64)
                } else {
                    format!("{:.1}B", billions).replace(".0B", "B")
                }
            }
            n if n >= 1_000_000 => {
                let millions = n as f64 / 1_000_000.0;
                if millions >= 10.0 {
                    format!("{}M", (millions.round()) as u64)
                } else {
                    format!("{:.1}M", millions).replace(".0M", "M")
                }
            }
            n if n >= 1_000 => {
                let thousands = n as f64 / 1_000.0;
                if thousands >= 10.0 {
                    format!("{}K", (thousands.round()) as u64)
                } else {
                    format!("{:.1}K", thousands).replace(".0K", "K")
                }
            }
            n => n.to_string(),
        };

        Ok(value)
    }

    // #[askama::filter_fn]
    // pub fn assets(value: &str, values: &dyn askama::Values) -> askama::Result<String> {
    //     let config = askama::get_value::<crate::axum_extra::TemplateConfig>(values, "config")
    //         .expect("Unable to get config from askama::get_value");
    //
    //     Ok(format!("{}/{value}", config.assets_base_url))
    // }
}

pub struct Template {
    preferred_language: String,
    pub preferred_language_iso: String,
    pub timezone: String,
    config: crate::config::Config,
}

impl Template {
    fn render_with_values<T: askama::Template>(
        &self,
        template: T,
    ) -> Result<String, askama::Error> {
        let mut values: HashMap<&str, Box<dyn std::any::Any>> = HashMap::new();
        values.insert(
            "preferred_language",
            Box::new(self.preferred_language.to_owned()),
        );
        values.insert(
            "preferred_language_iso",
            Box::new(self.preferred_language_iso.to_owned()),
        );
        values.insert("config", Box::new(self.config.clone()));

        #[cfg(debug_assertions)]
        {
            values.insert("is_dev", Box::new(true));
        }
        #[cfg(not(debug_assertions))]
        {
            values.insert("is_dev", Box::new(false));
        }

        template.render_with_values(&values)
    }

    pub fn to_string<T: askama::Template>(&self, template: T) -> String {
        match self.render_with_values(template) {
            Ok(html) => html,
            Err(err) => format!("Failed to render template. Error: {err}"),
        }
    }

    pub fn render<T: askama::Template>(&self, template: T) -> Response {
        match self.render_with_values(template) {
            Ok(html) => Html(html).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template. Error: {err}"),
            )
                .into_response(),
        }
    }
}

impl FromRequestParts<crate::routes::AppState> for Template {
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

        let timezone = parts
            .headers
            .get("TS-Timezone")
            .and_then(|v| v.to_str().ok())
            .map(String::from)
            .unwrap_or_else(|| "UTC".to_string());

        Ok(Template {
            preferred_language,
            preferred_language_iso,
            timezone,
            config: state.config.clone(),
        })
    }
}

#[derive(askama::Template)]
#[template(path = "partials/upgrade-modal.html")]
pub struct UpgradeModalTemplate;

#[derive(askama::Template)]
#[template(path = "404.html")]
pub struct NotFoundTemplate;

#[derive(askama::Template)]
#[template(path = "403.html")]
pub struct ForbiddenTemplate;

#[derive(askama::Template)]
#[template(path = "500.html")]
pub struct ServerTemplate;

#[macro_export]
macro_rules! try_page_response {
    ($result:expr, $template:expr) => {
        match $result.await {
            Ok(r) => r,
            Err(err) => {
                tracing::error!("{err}");

                return $template
                    .render($crate::template::ServerTemplate)
                    .into_response();
            }
        }
    };

    (sync: $result:expr, $template:expr) => {
        match $result {
            Ok(r) => r,
            Err(err) => {
                tracing::error!("{err}");

                return $template
                    .render($crate::template::ServerTemplate)
                    .into_response();
            }
        }
    };

    (opt: $result:expr, $template:expr) => {
        match $result.await {
            Ok(Some(r)) => r,
            Ok(_) => {
                return $template
                    .render($crate::template::NotFoundTemplate)
                    .into_response()
            }
            Err(err) => {
                tracing::error!("{err}");

                return $template
                    .render($crate::template::ServerTemplate)
                    .into_response();
            }
        }
    };
}

#[derive(askama::Template)]
#[template(path = "partials/toast-success.html")]
pub struct ToastSuccessTemplate<'a> {
    pub original: Option<&'a str>,
    pub message: &'a str,
    pub description: Option<&'a str>,
}

#[derive(askama::Template)]
#[template(path = "partials/toast-error.html")]
pub struct ToastErrorTemplate<'a> {
    pub original: Option<&'a str>,
    pub message: &'a str,
    pub description: Option<&'a str>,
}

#[macro_export]
macro_rules! try_response {
    // Internal helper for rendering error responses
    (@render $template:expr, $fallback:expr, $message:expr) => {
        match $fallback {
            Some(t) => {
                return $template
                    .render($crate::template::ToastErrorTemplate {
                        original: Some(&$template.to_string(t)),
                        message: $message,
                        description: None,
                    })
                    .into_response();
            }
            _ => {
                return (
                    [("ts-swap", "skip")],
                    $template.render($crate::template::ToastErrorTemplate {
                        original: None,
                        message: $message,
                        description: None,
                    }),
                )
                    .into_response();
            }
        }
    };

    // Result<T, Error> with Unknown variant handling
    ($result:expr, $template:expr, $fallback:expr) => {
        $crate::try_response!(sync: $result.await, $template, $fallback)
    };

    // Result<Option<T>, Error> with Unknown variant handling
    (opt: $result:expr, $template:expr, $fallback:expr) => {
        $crate::try_response!(sync opt: $result.await, $template, $fallback)
    };

    // Result<T, anyhow::Error> - all errors treated as server errors
    (anyhow: $result:expr, $template:expr, $fallback:expr) => {
        $crate::try_response!(sync anyhow: $result.await, $template, $fallback)
    };

    // Result<Option<T>, anyhow::Error> - all errors treated as server errors
    (anyhow_opt: $result:expr, $template:expr, $fallback:expr) => {
        $crate::try_response!(sync anyhow_opt: $result.await, $template, $fallback)
    };

    // Result<T, Error> with Unknown variant handling
    ($result:expr, $template:expr) => {
        $crate::try_response!(sync: $result.await, $template, None::<$crate::template::NotFoundTemplate>)
    };

    // Result<Option<T>, Error> with Unknown variant handling
    (opt: $result:expr, $template:expr) => {
        $crate::try_response!(sync opt: $result.await, $template, None::<$crate::template::NotFoundTemplate>)
    };

    // Result<T, anyhow::Error> - all errors treated as server errors
    (anyhow: $result:expr, $template:expr) => {
        $crate::try_response!(sync anyhow: $result.await, $template, None::<$crate::template::NotFoundTemplate>)
    };

    // Result<Option<T>, anyhow::Error> - all errors treated as server errors
    (anyhow_opt: $result:expr, $template:expr) => {
        $crate::try_response!(sync anyhow_opt: $result.await, $template, None::<$crate::template::NotFoundTemplate>)
    };

    // Result<T, Error> with Unknown variant handling
    (sync: $result:expr, $template:expr) => {
        $crate::try_response!(sync: $result, $template, None::<$crate::template::NotFoundTemplate>)
    };

    // Result<Option<T>, Error> with Unknown variant handling
    (sync opt: $result:expr, $template:expr) => {
        $crate::try_response!(sync opt: $result, $template, None::<$crate::template::NotFoundTemplate>)
    };

    // Result<T, anyhow::Error> - all errors treated as server errors
    (sync anyhow: $result:expr, $template:expr) => {
        $crate::try_response!(sync anyhow: $result, $template, None::<$crate::template::NotFoundTemplate>)
    };

    // Result<Option<T>, anyhow::Error> - all errors treated as server errors
    (sync anyhow_opt: $result:expr, $template:expr) => {
        $crate::try_response!(sync anyhow_opt: $result, $template, None::<$crate::template::NotFoundTemplate>)
    };

    // Result<T, Error> with Unknown variant handling
    (sync: $result:expr, $template:expr, $fallback:expr) => {
        match $result {
            Ok(r) => r,
            Err(imkitchen_shared::Error::Server(err)) => {
                tracing::error!("{err}");
                $crate::try_response!(@render $template, $fallback, $crate::template::SERVER_ERROR_MESSAGE)
            }
            Err(imkitchen_shared::Error::Forbidden) => {
                $crate::try_response!(@render $template, $fallback, $crate::template::FORBIDDEN)
            }
            Err(err) => {
                $crate::try_response!(@render $template, $fallback, err.to_string().as_str())
            }
        }
    };

    // Result<Option<T>, Error> with Unknown variant handling
    (sync opt: $result:expr, $template:expr, $fallback:expr) => {
        match $result {
            Ok(Some(r)) => r,
            Ok(_) => {
                $crate::try_response!(@render $template, $fallback, $crate::template::NOT_FOUND)
            }
            Err(imkitchen_shared::Error::Server(err)) => {
                tracing::error!("{err}");
                $crate::try_response!(@render $template, $fallback, $crate::template::SERVER_ERROR_MESSAGE)
            }
            Err(imkitchen_shared::Error::Forbidden) => {
                $crate::try_response!(@render $template, $fallback, $crate::template::FORBIDDEN)
            }
            Err(err) => {
                $crate::try_response!(@render $template, $fallback, err.to_string().as_str())
            }
        }
    };

    // Result<T, anyhow::Error> - all errors treated as server errors
    (sync anyhow: $result:expr, $template:expr, $fallback:expr) => {
        match $result {
            Ok(r) => r,
            Err(err) => {
                tracing::error!("{err}");
                $crate::try_response!(@render $template, $fallback, $crate::template::SERVER_ERROR_MESSAGE)
            }
        }
    };

    // Result<Option<T>, anyhow::Error> - all errors treated as server errors
    (sync anyhow_opt: $result:expr, $template:expr, $fallback:expr) => {
        match $result {
            Ok(Some(r)) => r,
            Ok(_) => {
                $crate::try_response!(@render $template, $fallback, $crate::template::NOT_FOUND)
            }
            Err(err) => {
                tracing::error!("{err}");
                $crate::try_response!(@render $template, $fallback, $crate::template::SERVER_ERROR_MESSAGE)
            }
        }
    };
}

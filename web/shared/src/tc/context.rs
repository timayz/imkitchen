use topcoat::{
    context::{Cx, app_context, memoize},
    cookie::{Cookies, cookies},
    router::request::{headers, uri},
};

use crate::{AppState, config::Config, language::parse_quality_values};

pub fn state(cx: &Cx) -> &AppState {
    app_context(cx)
}

pub fn config(cx: &Cx) -> &Config {
    &state(cx).config
}

/// Preferred language of the request: `?lang=` first, then `Accept-Language`,
/// then `en`. Same order as [`UserLanguage`](crate::language::UserLanguage).
///
/// Shard and procedure requests hit their own `/_topcoat/runtime/...` URL, so
/// `?lang=` of the page is not visible there; only the header applies.
#[memoize]
pub fn language(cx: &Cx) -> String {
    let from_query = uri(cx).query().and_then(|query| {
        query
            .split('&')
            .filter_map(|pair| pair.split_once('='))
            .find(|(name, _)| *name == "lang")
            .map(|(_, value)| value.to_owned())
    });

    let from_header = || {
        let accept_language = headers(cx).get("accept-language")?.to_str().ok()?;
        parse_quality_values(accept_language)
            .first()
            .map(|(lang, _)| (*lang).to_owned())
    };

    from_query
        .filter(|lang| !lang.is_empty())
        .or_else(from_header)
        .unwrap_or_else(|| "en".to_owned())
}

/// `fr-CA` → `fr`.
pub fn language_iso(cx: &Cx) -> &str {
    let language = language(cx);
    language.split_once('-').map_or(language, |(iso, _)| iso)
}

/// IANA timezone of the browser, from the `tz` cookie `tc-bridge.js` sets.
/// Replaces the `TS-Timezone` header of the patched twinspark.js. Client
/// input: shape-checked here, and `UTC` when absent or malformed.
#[memoize]
pub fn timezone(cx: &Cx) -> String {
    cookies(cx)
        .get("tz")
        .map(|cookie| cookie.value().to_owned())
        .filter(|tz| {
            !tz.is_empty()
                && tz.len() <= 64
                && tz
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, '/' | '_' | '-' | '+'))
        })
        .unwrap_or_else(|| "UTC".to_owned())
}

/// Translator bound to the request language. `Copy` because views are
/// `async move` blocks and would otherwise consume it on first use.
#[derive(Clone, Copy)]
pub struct I18n<'a>(&'a str);

impl I18n<'_> {
    pub fn t(&self, key: &str) -> String {
        rust_i18n::t!(key, locale = self.0).to_string()
    }
}

pub fn i18n(cx: &Cx) -> I18n<'_> {
    I18n(language(cx))
}

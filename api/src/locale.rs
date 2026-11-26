//! The language a refusal is worded in.
//!
//! A client says which it reads with `Accept-Language`; the api speaks the
//! ones of [`Locale`] and answers in the first of them the header ranks
//! highest, in English when it names none. The sentences live in
//! `api/locales`, one file per language, under the refusals' codes.
//!
//! Handlers know nothing of it: a refusal leaves them worded in English and
//! [`localize`], laid over every route, words it again for who asked.

use std::borrow::Cow;
use std::collections::BTreeMap;

use axum::extract::Request;
use axum::http::HeaderMap;
use axum::http::header::{ACCEPT_LANGUAGE, CACHE_CONTROL, CONTENT_LANGUAGE};
use axum::middleware::Next;
use axum::response::Response;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

use crate::error::Refusal;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum Locale {
    #[default]
    En,
    Fr,
}

impl Locale {
    pub const ALL: [Self; 2] = [Self::En, Self::Fr];

    /// The name of its file in `api/locales`, and what `Content-Language`
    /// says.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::En => "en",
            Self::Fr => "fr",
        }
    }

    /// From a language tag, by its primary subtag: `fr-CA` is French.
    pub fn from_tag(tag: &str) -> Option<Self> {
        let primary = tag.split(['-', '_']).next()?;
        Self::ALL
            .into_iter()
            .find(|locale| locale.as_str().eq_ignore_ascii_case(primary))
    }

    /// The language an `Accept-Language` value asks for first among those
    /// spoken: the highest `q`, the earliest of equals. One refused with
    /// `q=0` is never picked.
    pub fn from_accept_language(value: &str) -> Self {
        let mut best: Option<(Self, f32)> = None;
        for entry in value.split(',') {
            let mut parts = entry.split(';');
            let Some(locale) = parts.next().and_then(|tag| Self::from_tag(tag.trim())) else {
                continue;
            };
            let quality = parts
                .find_map(|param| param.trim().strip_prefix("q="))
                .map_or(1.0, |q| q.trim().parse().unwrap_or(0.0));
            if quality > 0.0 && best.is_none_or(|(_, held)| quality > held) {
                best = Some((locale, quality));
            }
        }
        best.map(|(locale, _)| locale).unwrap_or_default()
    }

    pub fn from_headers(headers: &HeaderMap) -> Self {
        headers
            .get(ACCEPT_LANGUAGE)
            .and_then(|value| value.to_str().ok())
            .map(Self::from_accept_language)
            .unwrap_or_default()
    }

    /// The sentence under `key` in this language, `None` when the files
    /// hold none.
    pub fn sentence(self, key: &str) -> Option<Cow<'static, str>> {
        // What `t!` itself calls: it takes a key only known at run time and
        // says when there is no sentence instead of answering the key.
        crate::_rust_i18n_try_translate(self.as_str(), key)
    }

    /// The sentence under `key`, each `%{name}` replaced by its parameter.
    pub fn sentence_with(self, key: &str, params: &BTreeMap<String, Value>) -> Option<String> {
        let mut sentence = self.sentence(key)?.into_owned();
        for (name, value) in params {
            let value = match value {
                Value::String(text) => text.clone(),
                other => other.to_string(),
            };
            sentence = sentence.replace(&format!("%{{{name}}}"), &value);
        }
        Some(sentence)
    }
}

/// Words the refusals in the language the request asks for.
pub async fn localize(request: Request, next: Next) -> Response {
    let locale = Locale::from_headers(request.headers());
    let mut response = next.run(request).await;
    let Some(refusal) = response.extensions_mut().remove::<Refusal>() else {
        return response;
    };
    if locale != Locale::default() {
        // What the refusal says to the caches is said in every language.
        let cached = response.headers_mut().remove(CACHE_CONTROL);
        response = refusal.render(locale);
        if let Some(cached) = cached {
            response.headers_mut().insert(CACHE_CONTROL, cached);
        }
    }
    response
        .headers_mut()
        .insert(CONTENT_LANGUAGE, locale.as_str().parse().expect("a token"));
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_language_is_the_one_the_header_ranks_highest() {
        for (header, locale) in [
            ("fr", Locale::Fr),
            ("fr-CA,fr;q=0.9,en;q=0.8", Locale::Fr),
            ("en-US,en;q=0.9,fr;q=0.8", Locale::En),
            ("de, fr;q=0.5, en;q=0.4", Locale::Fr),
            ("en;q=0.2, FR;q=0.7", Locale::Fr),
            ("fr;q=0, en;q=0.1", Locale::En),
            ("fr;q=0", Locale::En),
            ("de,es", Locale::En),
            ("*", Locale::En),
            ("", Locale::En),
        ] {
            assert_eq!(Locale::from_accept_language(header), locale, "{header}");
        }
    }

    #[test]
    fn every_language_holds_the_same_sentences() {
        let keys = |locale: Locale| {
            let mut keys: Vec<String> = crate::_rust_i18n_backend()
                .messages_for_locale(locale.as_str())
                .unwrap_or_else(|| panic!("no sentences in {}", locale.as_str()))
                .into_iter()
                .map(|(key, _)| key.into_owned())
                .collect();
            keys.sort();
            keys
        };
        let english = keys(Locale::En);
        assert!(!english.is_empty());
        for locale in Locale::ALL {
            assert_eq!(keys(locale), english, "{}", locale.as_str());
        }
    }

    #[test]
    fn a_sentence_holds_its_parameters() {
        let params = BTreeMap::from([("min".to_owned(), 8.into()), ("max".to_owned(), 128.into())]);
        assert_eq!(
            Locale::Fr.sentence_with("fields.length", &params).unwrap(),
            "Ce champ doit contenir entre 8 et 128 caractères."
        );
        assert_eq!(Locale::En.sentence("errors.nope"), None);
    }
}

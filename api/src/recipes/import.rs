//! Importing a recipe from a page of the web.
//!
//! The page is fetched ([`page`]) and the recipe is read out of it: the one
//! it describes to machines when it does ([`schema_org`]), the one it shows
//! to people otherwise ([`article`]). That becomes a recipe of the user
//! like one they wrote: the recipe says nothing of where it came from, and
//! the same page imported twice, by one user or by two, is two recipes.
//!
//! Each try is noted apart ([`note`]), whether it gave a recipe or not: what
//! admin tells the sites that are imported from, and those the api fails to
//! read, by.
//!
//! The picture the page gives its recipe is fetched too and comes with the
//! recipe ([`image`]). Two recipes imported from the same page show the
//! same picture, which is stored once (`crate::images`).
//!
//! What is read is held to the bounds a recipe written by hand is held to
//! (`recipes::RecipeBody`), cut short rather than refused.

mod article;
mod html;
mod ingredient;
mod page;
mod schema_org;

use anyhow::Context as _;
use axum::http::StatusCode;
use imkitchen_aggregates::image::RecipeImage;
use imkitchen_aggregates::recipe_import::{Failed, Imported};
use reqwest::Url;
use validator::ValidationError;

pub use self::page::{PageError, Pages, Web};
use super::model::Content;
use crate::error::{ApiError, ApiResult, INTERNAL, VALIDATION_FAILED};
use crate::images;
use crate::server::AppState;

/// The page was not read: nothing answers at its address, not in time, not
/// with a page, or the address is not one the api goes to.
pub const PAGE_UNREACHABLE: &str = "page_unreachable";
/// The page was read and holds no recipe the api can read.
pub const NO_RECIPE_ON_PAGE: &str = "no_recipe_on_page";

const TITLE_MAX: usize = 200;
const DESCRIPTION_MAX: usize = 2000;
const INGREDIENTS_MAX: usize = 100;
const INSTRUCTIONS_MAX: usize = 100;
const INSTRUCTION_MAX: usize = 4000;
const MINUTES_MAX: u32 = 10080;
const SERVINGS_MAX: u32 = 1000;
/// How many people a recipe that does not say is taken to feed.
const SERVINGS: u32 = 4;

/// What a reader found of a recipe on a page, as the page writes it.
#[derive(Debug, Default, PartialEq)]
struct Found {
    title: String,
    description: String,
    /// One line each, to be taken apart ([`ingredient::parse`]).
    ingredients: Vec<String>,
    /// The steps, in the order they are made.
    instructions: Vec<String>,
    prep_minutes: Option<u32>,
    cook_minutes: Option<u32>,
    /// How long it all takes, for the pages that say nothing else.
    total_minutes: Option<u32>,
    servings: Option<u32>,
    /// Where the page says the picture of the recipe is, as it writes it.
    image: Option<String>,
}

/// What a page holds: its recipe, and where its picture is.
#[derive(Debug, PartialEq)]
pub struct Page {
    pub content: Content,
    pub image: Option<Url>,
}

impl Found {
    /// What was found, as a recipe of ours and the address of its picture,
    /// which the page at `url` may write from where it is. `None` without a
    /// title: there is nothing to call it.
    fn into_page(self, url: &Url) -> Option<Page> {
        let image = self.image.as_deref().and_then(|image| {
            let image = url.join(image.trim()).ok()?;
            Some(image).filter(|image| page_url(image.as_str()).is_some())
        });
        Some(Page {
            content: self.into_content()?,
            image,
        })
    }

    /// What was found, as a recipe of ours. `None` without a title: there
    /// is nothing to call it.
    fn into_content(self) -> Option<Content> {
        let title = ingredient::clip(&self.title, TITLE_MAX);
        if title.is_empty() {
            return None;
        }
        let prep_minutes = self.prep_minutes.unwrap_or(0).min(MINUTES_MAX);
        // A page that only says how long it all takes: what is not
        // preparing is cooking.
        let cook_minutes = self
            .cook_minutes
            .or_else(|| Some(self.total_minutes?.saturating_sub(prep_minutes)))
            .unwrap_or(0)
            .min(MINUTES_MAX);
        Some(Content {
            title,
            description: ingredient::clip(&self.description, DESCRIPTION_MAX),
            ingredients: self
                .ingredients
                .iter()
                .take(INGREDIENTS_MAX)
                .map(|line| ingredient::parse(line))
                .collect(),
            instructions: self
                .instructions
                .iter()
                .take(INSTRUCTIONS_MAX)
                .map(|step| ingredient::clip(step, INSTRUCTION_MAX))
                .collect(),
            prep_minutes,
            cook_minutes,
            servings: self
                .servings
                .filter(|servings| (1..=SERVINGS_MAX).contains(servings))
                .unwrap_or(SERVINGS),
        })
    }
}

/// The first number `text` writes: "Pour 4 personnes" says 4.
fn first_number(text: &str) -> Option<u32> {
    let number = text.trim_start_matches(|c: char| !c.is_ascii_digit());
    let end = number
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(number.len());
    number[..end].parse().ok()
}

/// The recipe the page `html` at `url` holds, when it holds one that is
/// read within the budget: the one it describes to machines rather than the
/// one it shows, which is only guessed at.
fn read(html: &str, url: &Url) -> Option<Page> {
    let mut budget = html::Budget::new();
    let document = html::parse(html, &budget)?;
    schema_org::recipe(&document, &mut budget)
        .or_else(|| article::recipe(&document, &mut budget))?
        .into_page(url)
}

/// The address of a page of the web, if `url` is one.
fn page_url(url: &str) -> Option<Url> {
    Url::parse(url)
        .ok()
        .filter(|url| matches!(url.scheme(), "http" | "https") && url.has_host())
}

/// Checks a field that holds the address of a page (`#[validate(custom)]`).
pub fn valid_page_url(url: &str) -> Result<(), ValidationError> {
    page_url(url)
        .map(drop)
        .ok_or(ValidationError::new("invalid_url"))
}

/// A try at importing the page at `url` for `user_id`: `page_sent` when the
/// client sent the page in place of the api fetching it.
pub struct Try<'a> {
    pub user_id: &'a str,
    pub url: &'a str,
    pub page_sent: bool,
}

/// Notes how a try went: the recipe it gave, or why it gave none. Whatever
/// happens here, the import went as it went: a try that could not be noted
/// is logged and no more.
pub async fn note(state: &AppState, tried: Try<'_>, outcome: Result<&str, &ApiError>) {
    let (user_id, url) = (tried.user_id.to_owned(), tried.url.to_owned());
    let mut write = evento::create();
    match outcome {
        Ok(recipe_id) => write.event(&Imported {
            user_id,
            url,
            page_sent: tried.page_sent,
            recipe_id: recipe_id.to_owned(),
        }),
        Err(err) => write.event(&Failed {
            user_id,
            url,
            page_sent: tried.page_sent,
            reason: match err {
                ApiError::Refused(_, code) => code,
                ApiError::Invalid(_) => VALIDATION_FAILED,
                ApiError::Internal(_) => INTERNAL,
            }
            .to_owned(),
        }),
    };
    if let Err(err) = write.commit(&state.executor).await {
        tracing::warn!(%err, "an import was not noted");
    }
}

/// The most a page may weigh, fetched or sent, in bytes.
pub const PAGE_MAX: usize = 4 * 1024 * 1024;

/// The recipe of the page at `url`: read on `html` when the client sends
/// the page as its browser got it, which a site that only answers browsers
/// leaves no other way to, fetched otherwise.
pub async fn page(pages: &dyn Pages, url: &str, html: Option<String>) -> ApiResult<Page> {
    let refused = |code| ApiError::refused(StatusCode::UNPROCESSABLE_ENTITY, code);
    let url = page_url(url).ok_or(refused(PAGE_UNREACHABLE))?;
    let html = match html {
        Some(html) => html,
        None => pages.fetch(&url).await.map_err(|err| match err {
            PageError::Unreachable(reason) => {
                tracing::info!(reason, "a page to import was not read");
                refused(PAGE_UNREACHABLE)
            }
            PageError::NotHtml => refused(NO_RECIPE_ON_PAGE),
        })?,
    };
    // A page may be megabytes: read off the threads that answer requests.
    tokio::task::spawn_blocking(move || read(&html, &url))
        .await
        .context("reading a page to import failed")?
        .ok_or(refused(NO_RECIPE_ON_PAGE))
}

/// The picture at `url`, used once more for the recipe being imported.
/// `None` when it cannot be had (nothing answers, or not with a picture
/// that is taken): a recipe is imported without its picture rather than
/// not at all.
pub async fn image(state: &AppState, url: &Url) -> Option<RecipeImage> {
    let bytes = match state.pages.image(url).await {
        Ok(bytes) => bytes,
        Err(err) => {
            tracing::info!(%url, ?err, "the picture of a recipe to import was not fetched");
            return None;
        }
    };
    match images::attach(&state.executor, &state.images, bytes).await {
        Ok(image) => Some(image),
        Err(err) => {
            tracing::info!(%url, ?err, "the picture of a recipe to import was not kept");
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_address_of_a_picture_is_read_from_where_the_page_is() {
        let url = Url::parse("https://example.com/recipes/toast").unwrap();
        let found = |image: &str| Found {
            title: "Toast".to_owned(),
            image: Some(image.to_owned()),
            ..Found::default()
        };
        let image = |image: &str| {
            let page = found(image).into_page(&url).unwrap();
            page.image.map(String::from)
        };

        for (written, read) in [
            (
                "https://cdn.example.com/toast.jpg",
                "https://cdn.example.com/toast.jpg",
            ),
            (
                "//cdn.example.com/toast.jpg",
                "https://cdn.example.com/toast.jpg",
            ),
            ("/img/toast.jpg", "https://example.com/img/toast.jpg"),
            ("toast.jpg", "https://example.com/recipes/toast.jpg"),
        ] {
            assert_eq!(image(written).as_deref(), Some(read), "{written}");
        }
        // Not somewhere a picture is fetched from: the recipe has none.
        for written in [
            "data:image/png;base64,AAAA",
            "file:///etc/passwd",
            "ftp://x/y",
        ] {
            assert_eq!(image(written), None, "{written}");
        }
        // No recipe, no picture.
        assert_eq!(Found::default().into_page(&url), None);
    }
}

//! Importing a recipe from a page of the web.
//!
//! The page is fetched ([`page`]) and the recipe is read out of it: the one
//! it describes to machines when it does ([`schema_org`]), the one it shows
//! to people otherwise ([`article`]). That becomes a recipe of the user
//! like one they wrote: nothing is kept of where it came from, and the same
//! page imported twice, by one user or by two, is two recipes.
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
use reqwest::Url;
use validator::ValidationError;

pub use self::page::{PageError, Pages, Web};
use super::model::Content;
use crate::error::{ApiError, ApiResult};

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
}

impl Found {
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

/// The recipe the page `html` holds, when it holds one that is read within
/// the budget: the one it describes to machines rather than the one it
/// shows, which is only guessed at.
fn read(html: &str) -> Option<Content> {
    let mut budget = html::Budget::new();
    let document = html::parse(html, &budget)?;
    schema_org::recipe(&document, &mut budget)
        .or_else(|| article::recipe(&document, &mut budget))?
        .into_content()
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

/// What the page at `url` says of the recipe it describes.
pub async fn content(pages: &dyn Pages, url: &str) -> ApiResult<Content> {
    let refused = |code| ApiError::refused(StatusCode::UNPROCESSABLE_ENTITY, code);
    let url = page_url(url).ok_or(refused(PAGE_UNREACHABLE))?;
    let html = pages.fetch(&url).await.map_err(|err| match err {
        PageError::Unreachable(reason) => {
            tracing::info!(reason, "a page to import was not read");
            refused(PAGE_UNREACHABLE)
        }
        PageError::NotHtml => refused(NO_RECIPE_ON_PAGE),
    })?;
    // A page may be megabytes: read off the threads that answer requests.
    tokio::task::spawn_blocking(move || read(&html))
        .await
        .context("reading a page to import failed")?
        .ok_or(refused(NO_RECIPE_ON_PAGE))
}

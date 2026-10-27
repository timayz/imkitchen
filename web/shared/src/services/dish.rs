//! Dishes: the user's composed meals. Listing resolves each dish's recipes
//! against the recipe projection (a deleted recipe is skipped), and says
//! whether the dish is already in the list.
//!
//! The dish read model is fed by a subscription, so the write services wait
//! for it to catch up before returning: the page or client that follows
//! reads the result immediately, without polling of its own.

use std::time::Duration;

use imkitchen_core::dish::query::{Candidate, CandidatePage, CandidateQuery, DishRow};
use imkitchen_core::recipe::query::user::RecipeCard;
use imkitchen_types::recipe::RecipeType;

use crate::AppState;

pub use imkitchen_core::dish::ComposeInput;

#[derive(Clone, Debug)]
pub struct DishRecipe {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub recipe_type: RecipeType,
    pub prep_time: u16,
    pub cook_time: u16,
    pub thumbnail_version: Option<String>,
    pub blur_placeholder: Option<String>,
}

impl DishRecipe {
    pub fn total_time(&self) -> u16 {
        self.prep_time + self.cook_time
    }
}

#[derive(Clone, Debug)]
pub struct DishView {
    pub id: String,
    pub name: String,
    /// In meal order; recipes deleted since the dish was composed are absent.
    pub recipes: Vec<DishRecipe>,
    /// Every remaining recipe of the dish is in the user's list.
    pub in_list: bool,
    pub created_at: u64,
    pub updated_at: u64,
}

impl DishView {
    pub fn main(&self) -> Option<&DishRecipe> {
        self.recipes
            .iter()
            .find(|r| r.recipe_type == RecipeType::MainCourse)
    }

    /// Total time of the longest recipe: the courses cook in parallel.
    pub fn total_time(&self) -> u16 {
        self.recipes
            .iter()
            .map(DishRecipe::total_time)
            .max()
            .unwrap_or(0)
    }
}

fn resolve(row: DishRow, cards: &[RecipeCard], list: &[String]) -> DishView {
    let recipes: Vec<DishRecipe> = row
        .recipe_ids
        .0
        .iter()
        .filter_map(|id| cards.iter().find(|c| &c.id == id))
        .map(|card| DishRecipe {
            id: card.id.to_owned(),
            name: card.name.to_owned(),
            slug: card.slug.to_owned(),
            recipe_type: card.recipe_type.0.clone(),
            prep_time: card.prep_time,
            cook_time: card.cook_time,
            thumbnail_version: card.thumbnail_version.to_owned(),
            blur_placeholder: card.blur_placeholder.to_owned(),
        })
        .collect();
    let in_list = !recipes.is_empty() && recipes.iter().all(|r| list.contains(&r.id));

    DishView {
        id: row.id,
        name: row.name,
        recipes,
        in_list,
        created_at: row.created_at,
        updated_at: row.updated_at,
    }
}

async fn list_recipe_ids(app: &AppState, user_id: &str) -> anyhow::Result<Vec<String>> {
    Ok(app
        .core
        .shopping
        .load(user_id)
        .await?
        .map(|s| s.recipes)
        .unwrap_or_default())
}

/// The user's dishes, by name.
pub async fn list(app: &AppState, user_id: &str) -> anyhow::Result<Vec<DishView>> {
    let rows = app.core.dish.list(user_id).await?;
    if rows.is_empty() {
        return Ok(vec![]);
    }

    let ids: Vec<String> = rows
        .iter()
        .flat_map(|r| r.recipe_ids.0.iter().cloned())
        .collect();
    let cards = app.core.recipe.filter_by_ids(ids).await?;
    let list = list_recipe_ids(app, user_id).await?;

    Ok(rows
        .into_iter()
        .map(|row| resolve(row, &cards, &list))
        .collect())
}

/// One of the user's dishes. `None` when it does not exist or belongs to
/// someone else.
pub async fn find(app: &AppState, user_id: &str, id: &str) -> anyhow::Result<Option<DishView>> {
    let Some(row) = app.core.dish.find(id).await? else {
        return Ok(None);
    };
    if row.user_id != user_id {
        return Ok(None);
    }

    let cards = app
        .core
        .recipe
        .filter_by_ids(row.recipe_ids.0.clone())
        .await?;
    let list = list_recipe_ids(app, user_id).await?;

    Ok(Some(resolve(row, &cards, &list)))
}

/// Page size of a candidate search; libraries of hundreds of recipes are
/// browsed by name rather than listed.
pub const CANDIDATE_PAGE: u16 = 20;

/// A page of what the user can compose a dish from.
pub async fn candidates(
    app: &AppState,
    user_id: &str,
    query: CandidateQuery,
) -> anyhow::Result<CandidatePage> {
    app.core.dish.candidates(user_id, query).await
}

/// The candidates among `ids`, in the order given (unknown ids dropped).
pub async fn candidates_by_ids(
    app: &AppState,
    user_id: &str,
    ids: &[String],
) -> anyhow::Result<Vec<Candidate>> {
    let found = app
        .core
        .dish
        .candidates_by_ids(user_id, ids.to_vec())
        .await?;
    Ok(ids
        .iter()
        .filter_map(|id| found.iter().find(|c| &c.id == id).cloned())
        .collect())
}

/// Composing needs at least one main course in the pool.
pub async fn can_compose(app: &AppState, user_id: &str) -> anyhow::Result<bool> {
    Ok(!app
        .core
        .shopping
        .sample_recipes(user_id, RecipeType::MainCourse)
        .await?
        .is_empty())
}

/// Polls the read model until `ready` holds, for about 1.5 seconds. Returns
/// quietly on timeout: the caller's next read may then be a step behind,
/// which is the same situation as before the wait.
async fn settle<F>(app: &AppState, id: &str, ready: F) -> anyhow::Result<()>
where
    F: Fn(Option<&DishRow>) -> bool,
{
    for _ in 0..30 {
        let row = app.core.dish.find(id).await?;
        if ready(row.as_ref()) {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    tracing::warn!(dish = id, "dish read model did not settle in time");
    Ok(())
}

/// Compose a dish and wait for it to be listed. Returns its id.
pub async fn create(
    app: &AppState,
    user_id: &str,
    input: ComposeInput,
) -> imkitchen_core::Result<String> {
    let id = app.core.dish.create(input, user_id).await?;
    settle(app, &id, |row| row.is_some()).await?;
    Ok(id)
}

/// Recompose a dish and wait for the listing to reflect it.
pub async fn update(
    app: &AppState,
    user_id: &str,
    id: &str,
    input: ComposeInput,
) -> imkitchen_core::Result<()> {
    app.core.dish.update(id, input, user_id).await?;
    let Some(dish) = app.core.dish.load(id).await? else {
        return Ok(());
    };
    settle(app, id, |row| {
        row.is_some_and(|r| r.name == dish.name && r.recipe_ids.0 == dish.recipe_ids)
    })
    .await?;
    Ok(())
}

/// Delete a dish and wait for it to leave the listing.
pub async fn delete(app: &AppState, user_id: &str, id: &str) -> imkitchen_core::Result<()> {
    app.core.dish.delete(id, user_id).await?;
    settle(app, id, |row| row.is_none()).await?;
    Ok(())
}

/// Add every recipe of the dish to the user's list, scaled to their
/// household size. The list is read from the aggregate, so the kitchen is
/// up to date right away.
pub async fn add_to_list(app: &AppState, user_id: &str, id: &str) -> imkitchen_core::Result<()> {
    let preferences = app.identity.meal_preferences.load(user_id).await?;
    app.core
        .shopping
        .add_dish(id, preferences.household_size, user_id)
        .await
}

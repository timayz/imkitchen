//! What admin knows of the recipes: all each one says, whose it is and the
//! picture it shows.
//!
//! admin keeps one row per recipe that is not deleted. The handlers leave
//! the table as it should be when an event is delivered twice (the
//! subscription is at-least-once): a recipe is written once however often
//! it is created, rewritten as its last event says, and gone however often
//! it is deleted.

use evento::metadata::Event;
use evento::subscription::{Context, SubscriptionBuilder};
use imkitchen_aggregates::recipe::{self, Created, Deleted, Edited, ImageRemoved, ImageSet};
use serde::{Deserialize, Serialize};
use timada_core::db::{FromRow, Json, search_key};
use timada_core::like_contains;

use crate::db::Db;

/// The cursor identity of the subscription, in the store.
pub const SUBSCRIPTION: &str = "imkitchen-admin/recipes";

/// Follows the recipes into `db`.
pub fn subscription<E: evento::Executor>(db: Db) -> SubscriptionBuilder<E> {
    SubscriptionBuilder::new(SUBSCRIPTION)
        .data(db)
        .handler(on_created())
        .handler(on_edited())
        .handler(on_deleted())
        .handler(on_image_set())
        .handler(on_image_removed())
}

/// What a recipe takes of one thing, as the table keeps it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Ingredient {
    pub name: String,
    pub quantity: Option<f64>,
    pub unit: Option<String>,
}

fn ingredients(ingredients: &[recipe::Ingredient]) -> anyhow::Result<String> {
    let ingredients: Vec<Ingredient> = ingredients
        .iter()
        .map(|one| Ingredient {
            name: one.name.clone(),
            quantity: one.quantity,
            unit: one.unit.clone(),
        })
        .collect();
    Json(ingredients).to_text()
}

/// What a search looks in: the title and the name of each ingredient, a
/// line each, without case or accents.
fn searched(title: &str, ingredients: &[recipe::Ingredient]) -> String {
    let lines: Vec<&str> = std::iter::once(title)
        .chain(ingredients.iter().map(|one| one.name.as_str()))
        .collect();
    search_key(&lines.join("\n"))
}

#[evento::subscription]
async fn on_created<E: evento::Executor>(
    context: &Context<'_, E>,
    event: Event<Created>,
) -> anyhow::Result<()> {
    let db: Db = context.extract();
    let data = &event.data;
    db.execute(
        "INSERT INTO recipes (id, owner_id, title, description, ingredients, instructions,
            prep_minutes, cook_minutes, servings, search, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?11)
         ON CONFLICT (id) DO NOTHING",
        (
            event.aggregate_id.as_str(),
            data.owner_id.as_str(),
            data.title.as_str(),
            data.description.as_str(),
            ingredients(&data.ingredients)?,
            Json(&data.instructions).to_text()?,
            data.prep_minutes,
            data.cook_minutes,
            data.servings,
            searched(&data.title, &data.ingredients),
            event.timestamp as i64,
        ),
    )
    .await?;
    Ok(())
}

#[evento::subscription]
async fn on_edited<E: evento::Executor>(
    context: &Context<'_, E>,
    event: Event<Edited>,
) -> anyhow::Result<()> {
    let db: Db = context.extract();
    let data = &event.data;
    db.execute(
        "UPDATE recipes SET title = ?2, description = ?3, ingredients = ?4, instructions = ?5,
            prep_minutes = ?6, cook_minutes = ?7, servings = ?8, search = ?9, updated_at = ?10
         WHERE id = ?1",
        (
            event.aggregate_id.as_str(),
            data.title.as_str(),
            data.description.as_str(),
            ingredients(&data.ingredients)?,
            Json(&data.instructions).to_text()?,
            data.prep_minutes,
            data.cook_minutes,
            data.servings,
            searched(&data.title, &data.ingredients),
            event.timestamp as i64,
        ),
    )
    .await?;
    Ok(())
}

#[evento::subscription]
async fn on_deleted<E: evento::Executor>(
    context: &Context<'_, E>,
    event: Event<Deleted>,
) -> anyhow::Result<()> {
    let db: Db = context.extract();
    db.execute(
        "DELETE FROM recipes WHERE id = ?1",
        (event.aggregate_id.as_str(),),
    )
    .await?;
    Ok(())
}

#[evento::subscription]
async fn on_image_set<E: evento::Executor>(
    context: &Context<'_, E>,
    event: Event<ImageSet>,
) -> anyhow::Result<()> {
    let db: Db = context.extract();
    let image = &event.data.image;
    db.execute(
        "UPDATE recipes SET image_id = ?2, file_id = ?3, updated_at = ?4 WHERE id = ?1",
        (
            event.aggregate_id.as_str(),
            image.image_id.as_str(),
            image.file_id.as_str(),
            event.timestamp as i64,
        ),
    )
    .await?;
    Ok(())
}

#[evento::subscription]
async fn on_image_removed<E: evento::Executor>(
    context: &Context<'_, E>,
    event: Event<ImageRemoved>,
) -> anyhow::Result<()> {
    let db: Db = context.extract();
    db.execute(
        "UPDATE recipes SET image_id = NULL, file_id = NULL, updated_at = ?2 WHERE id = ?1",
        (event.aggregate_id.as_str(), event.timestamp as i64),
    )
    .await?;
    Ok(())
}

/// What a list shows of a recipe.
#[derive(Debug, Clone, FromRow, PartialEq)]
pub struct RecipeRow {
    pub id: String,
    pub owner_id: String,
    pub title: String,
    /// The files of the picture it shows, when it shows one.
    pub file_id: Option<String>,
    /// Unix seconds.
    pub created_at: i64,
    /// Unix seconds.
    pub updated_at: i64,
}

/// A recipe, with all it says. Times are minutes, 0 when it says none.
#[derive(Debug, Clone, FromRow, PartialEq)]
pub struct Recipe {
    pub id: String,
    pub owner_id: String,
    pub title: String,
    pub description: String,
    pub ingredients: Json<Vec<Ingredient>>,
    /// The steps, in the order they are made.
    pub instructions: Json<Vec<String>>,
    pub prep_minutes: i64,
    pub cook_minutes: i64,
    pub servings: i64,
    /// The picture it shows, when it shows one.
    pub image_id: Option<String>,
    pub file_id: Option<String>,
    /// Unix seconds.
    pub created_at: i64,
    /// Unix seconds.
    pub updated_at: i64,
}

fn pattern(q: Option<&str>) -> Option<String> {
    q.map(|q| like_contains(&search_key(q)))
}

/// How many recipes `q` is found in, as far as admin has followed: all of
/// them without one.
pub async fn count(db: &Db, q: Option<&str>) -> anyhow::Result<u64> {
    let count: i64 = db
        .fetch_scalar(
            "SELECT count(*) FROM recipes WHERE ?1 IS NULL OR search LIKE ?1 ESCAPE '\\'",
            (pattern(q),),
        )
        .await?;
    Ok(count.max(0) as u64)
}

/// The recipes `q` is found in (their title or an ingredient, whatever the
/// case and the accents), the most recent first: `limit` of them from the
/// `offset`th.
pub async fn list(
    db: &Db,
    q: Option<&str>,
    offset: i64,
    limit: i64,
) -> anyhow::Result<Vec<RecipeRow>> {
    db.fetch_all(
        "SELECT id, owner_id, title, file_id, created_at, updated_at FROM recipes
         WHERE ?1 IS NULL OR search LIKE ?1 ESCAPE '\\'
         ORDER BY created_at DESC, id DESC LIMIT ?2 OFFSET ?3",
        (pattern(q), limit, offset),
    )
    .await
}

/// The recipe `id`, if there is one.
pub async fn get(db: &Db, id: &str) -> anyhow::Result<Option<Recipe>> {
    db.fetch_optional(
        "SELECT id, owner_id, title, description, ingredients, instructions, prep_minutes,
            cook_minutes, servings, image_id, file_id, created_at, updated_at
         FROM recipes WHERE id = ?1",
        (id,),
    )
    .await
}

/// How many recipes the user `owner_id` has.
pub async fn count_of(db: &Db, owner_id: &str) -> anyhow::Result<u64> {
    let count: i64 = db
        .fetch_scalar(
            "SELECT count(*) FROM recipes WHERE owner_id = ?1",
            (owner_id,),
        )
        .await?;
    Ok(count.max(0) as u64)
}

/// The recipes of the user `owner_id`, the most recent first, `limit` at
/// most.
pub async fn list_of(db: &Db, owner_id: &str, limit: i64) -> anyhow::Result<Vec<RecipeRow>> {
    db.fetch_all(
        "SELECT id, owner_id, title, file_id, created_at, updated_at FROM recipes
         WHERE owner_id = ?1 ORDER BY created_at DESC, id DESC LIMIT ?2",
        (owner_id, limit),
    )
    .await
}

/// The recipes that show the picture `image_id`, the most recent first,
/// `limit` at most.
pub async fn showing(db: &Db, image_id: &str, limit: i64) -> anyhow::Result<Vec<RecipeRow>> {
    db.fetch_all(
        "SELECT id, owner_id, title, file_id, created_at, updated_at FROM recipes
         WHERE image_id = ?1 ORDER BY created_at DESC, id DESC LIMIT ?2",
        (image_id, limit),
    )
    .await
}

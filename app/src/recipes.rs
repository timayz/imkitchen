//! What app knows of the recipes: all each one says, whose it is and the
//! picture it shows.
//!
//! A recipe is its owner's alone: but for how many there are, nothing is
//! read from the table without saying whose is asked for.
//!
//! app keeps one row per recipe that is not deleted. The handlers leave
//! the table as it should be when an event is delivered twice (the
//! subscription is at-least-once): a recipe is written once however often
//! it is created, rewritten as its last event says, and gone however often
//! it is deleted.

use evento::metadata::Event;
use evento::subscription::{Context, SubscriptionBuilder};
use imkitchen_aggregates::recipe::{self, Created, Deleted, Edited, ImageRemoved, ImageSet};
use serde::{Deserialize, Serialize};
use timada_core::db::{FromRow, Json, search_key};
use timada_core::page::Cursor;
use timada_core::{Rendition, like_contains};

use crate::db::Db;

/// The cursor identity of the subscription, in the store.
pub const SUBSCRIPTION: &str = "imkitchen-app/recipes";

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

/// What a recipe takes of one thing, as the table keeps it and as the api
/// tells of it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Ingredient {
    pub name: String,
    #[serde(default)]
    pub quantity: Option<f64>,
    #[serde(default)]
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

/// One step of a recipe, as the table keeps it and as the api tells of it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Instruction {
    pub text: String,
    /// The minutes to wait once it is done, when there is something to
    /// wait for.
    #[serde(default)]
    pub timer_minutes: Option<u32>,
}

fn instructions(instructions: &[recipe::Instruction]) -> anyhow::Result<String> {
    let instructions: Vec<Instruction> = instructions
        .iter()
        .map(|one| Instruction {
            text: one.text.clone(),
            timer_minutes: one.timer_minutes,
        })
        .collect();
    Json(instructions).to_text()
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
            instructions(&data.instructions)?,
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
            instructions(&data.instructions)?,
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
    let large = image.picture.rendition(Rendition::Page);
    db.execute(
        "UPDATE recipes SET file_id = ?2, image_width = ?3, image_height = ?4, updated_at = ?5
         WHERE id = ?1",
        (
            event.aggregate_id.as_str(),
            image.file_id.as_str(),
            large.width,
            large.height,
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
        "UPDATE recipes SET file_id = NULL, image_width = NULL, image_height = NULL,
            updated_at = ?2
         WHERE id = ?1",
        (event.aggregate_id.as_str(), event.timestamp as i64),
    )
    .await?;
    Ok(())
}

/// What a list shows of a recipe. Times are minutes, 0 when it says none.
#[derive(Debug, Clone, FromRow, PartialEq)]
pub struct RecipeRow {
    pub id: String,
    pub title: String,
    pub prep_minutes: i64,
    pub cook_minutes: i64,
    pub servings: i64,
    /// The files of the picture it shows, when it shows one.
    pub file_id: Option<String>,
    /// Unix seconds.
    pub created_at: i64,
}

impl RecipeRow {
    /// Where the row stands in its owner's list.
    pub fn cursor(&self) -> Cursor {
        Cursor {
            at: self.created_at,
            id: self.id.clone(),
        }
    }
}

/// A recipe, with all it says. Times are minutes, 0 when it says none.
#[derive(Debug, Clone, FromRow, PartialEq)]
pub struct Recipe {
    pub id: String,
    pub title: String,
    pub description: String,
    pub ingredients: Json<Vec<Ingredient>>,
    /// The steps, in the order they are made.
    pub instructions: Json<Vec<Instruction>>,
    pub prep_minutes: i64,
    pub cook_minutes: i64,
    pub servings: i64,
    /// The files of the picture it shows, when it shows one, and the size
    /// of the larger of the two.
    pub file_id: Option<String>,
    pub image_width: Option<i64>,
    pub image_height: Option<i64>,
    /// Unix seconds.
    pub updated_at: i64,
}

/// How many recipes there are, whoever's they are, as far as app has
/// followed.
pub async fn count(db: &Db) -> anyhow::Result<u64> {
    let count: i64 = db.fetch_scalar("SELECT count(*) FROM recipes", ()).await?;
    Ok(count.max(0) as u64)
}

/// The recipes of `owner_id`, the most recent first, `limit` at most: those
/// after `after` in the list, and only those `q` is found in (their title
/// or an ingredient, whatever the case and the accents) when there is one.
pub async fn list_of(
    db: &Db,
    owner_id: &str,
    q: Option<&str>,
    after: Option<&Cursor>,
    limit: i64,
) -> anyhow::Result<Vec<RecipeRow>> {
    let (at, id) = after.map_or((i64::MAX, ""), |cursor| (cursor.at, cursor.id.as_str()));
    db.fetch_all(
        "SELECT id, title, prep_minutes, cook_minutes, servings, file_id, created_at
         FROM recipes
         WHERE owner_id = ?1
            AND created_at <= ?2 AND (created_at < ?2 OR id < ?3)
            AND (?4 IS NULL OR search LIKE ?4 ESCAPE '\\')
         ORDER BY created_at DESC, id DESC LIMIT ?5",
        (
            owner_id,
            at,
            id,
            q.map(|q| like_contains(&search_key(q))),
            limit,
        ),
    )
    .await
}

/// The recipe `id`, when it is one of `owner_id`'s: someone else's is one
/// that does not exist.
pub async fn get_own(db: &Db, owner_id: &str, id: &str) -> anyhow::Result<Option<Recipe>> {
    db.fetch_optional(
        "SELECT id, title, description, ingredients, instructions, prep_minutes, cook_minutes,
            servings, file_id, image_width, image_height, updated_at
         FROM recipes WHERE id = ?1 AND owner_id = ?2",
        (id, owner_id),
    )
    .await
}

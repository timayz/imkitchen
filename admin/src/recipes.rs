//! What admin knows of the recipes: all each one says, whose it is and the
//! picture it shows.
//!
//! A recipe is a projection of its stream ([`RecipeState`]) whose snapshot
//! is its row of the `recipes` table: one row per recipe that is not
//! deleted, kept by a projection subscription ([`subscription`]). One
//! recipe is loaded from its row and the events written after it ([`get`]),
//! so its page shows it as the command an administrator just handed the api
//! left it. The lists are read from the table, a moment behind.

use evento::Executor;
use evento::metadata::Event;
use evento::projection::{Context, Projection, ProjectionSubscription, Snapshot};
use imkitchen_aggregates::recipe::{
    self, Created, Deleted, Edited, ImageRemoved, ImageSet, Recipe as RecipeStream,
};
use serde::{Deserialize, Serialize};
use timada_core::db::{FromRow, Json, search_key};
use timada_core::like_contains;

use crate::db::Db;

/// The cursor identity of the subscription, in the store.
pub const SUBSCRIPTION: &str = "imkitchen-admin/recipes";

/// What a recipe takes of one thing, as the table keeps it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Ingredient {
    pub name: String,
    pub quantity: Option<f64>,
    pub unit: Option<String>,
}

fn ingredients(ingredients: &[recipe::Ingredient]) -> Vec<Ingredient> {
    ingredients
        .iter()
        .map(|one| Ingredient {
            name: one.name.clone(),
            quantity: one.quantity,
            unit: one.unit.clone(),
        })
        .collect()
}

/// One step of a recipe, as the table keeps it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Instruction {
    pub text: String,
    /// The minutes to wait once it is done, when there is something to
    /// wait for.
    pub timer_minutes: Option<u32>,
}

fn instructions(instructions: &[recipe::Instruction]) -> Vec<Instruction> {
    instructions
        .iter()
        .map(|one| Instruction {
            text: one.text.clone(),
            timer_minutes: one.timer_minutes,
        })
        .collect()
}

/// What a search looks in: the title and the name of each ingredient, a
/// line each, without case or accents.
fn searched(title: &str, ingredients: &[Ingredient]) -> String {
    let lines: Vec<&str> = std::iter::once(title)
        .chain(ingredients.iter().map(|one| one.name.as_str()))
        .collect();
    search_key(&lines.join("\n"))
}

/// A recipe as its stream leaves it.
#[evento::projection]
#[derive(Debug)]
pub struct RecipeState {
    pub id: String,
    pub owner_id: String,
    pub title: String,
    pub description: String,
    pub ingredients: Vec<Ingredient>,
    /// The steps, in the order they are made.
    pub instructions: Vec<Instruction>,
    pub prep_minutes: u32,
    pub cook_minutes: u32,
    pub servings: u32,
    /// The picture it shows, when it shows one, and what its files are
    /// named after.
    pub image_id: Option<String>,
    pub file_id: Option<String>,
    /// Unix seconds of `Created`.
    pub created_at: u64,
    /// Unix seconds of the last event.
    pub updated_at: u64,
    pub deleted: bool,
}

fn projection<E: Executor>(db: Db) -> Projection<E, RecipeState> {
    Projection::new::<RecipeStream>()
        .handler(on_created())
        .handler(on_edited())
        .handler(on_deleted())
        .handler(on_image_set())
        .handler(on_image_removed())
        .data(db)
}

/// Keeps the row of each recipe in `db` as its stream leaves it.
pub fn subscription<E: Executor>(db: Db) -> ProjectionSubscription<E, RecipeState> {
    projection(db).subscription(SUBSCRIPTION)
}

#[evento::handler]
async fn on_created(event: Event<Created>, recipe: &mut RecipeState) -> anyhow::Result<()> {
    let data = &event.data;
    recipe.id = event.aggregate_id.to_owned();
    recipe.owner_id = data.owner_id.to_owned();
    recipe.title = data.title.to_owned();
    recipe.description = data.description.to_owned();
    recipe.ingredients = ingredients(&data.ingredients);
    recipe.instructions = instructions(&data.instructions);
    recipe.prep_minutes = data.prep_minutes;
    recipe.cook_minutes = data.cook_minutes;
    recipe.servings = data.servings;
    recipe.created_at = event.timestamp;
    recipe.updated_at = event.timestamp;
    Ok(())
}

#[evento::handler]
async fn on_edited(event: Event<Edited>, recipe: &mut RecipeState) -> anyhow::Result<()> {
    let data = &event.data;
    recipe.title = data.title.to_owned();
    recipe.description = data.description.to_owned();
    recipe.ingredients = ingredients(&data.ingredients);
    recipe.instructions = instructions(&data.instructions);
    recipe.prep_minutes = data.prep_minutes;
    recipe.cook_minutes = data.cook_minutes;
    recipe.servings = data.servings;
    recipe.updated_at = event.timestamp;
    Ok(())
}

#[evento::handler]
async fn on_deleted(event: Event<Deleted>, recipe: &mut RecipeState) -> anyhow::Result<()> {
    recipe.deleted = true;
    recipe.updated_at = event.timestamp;
    Ok(())
}

#[evento::handler]
async fn on_image_set(event: Event<ImageSet>, recipe: &mut RecipeState) -> anyhow::Result<()> {
    let image = &event.data.image;
    recipe.image_id = Some(image.image_id.to_owned());
    recipe.file_id = Some(image.file_id.to_owned());
    recipe.updated_at = event.timestamp;
    Ok(())
}

#[evento::handler]
async fn on_image_removed(
    event: Event<ImageRemoved>,
    recipe: &mut RecipeState,
) -> anyhow::Result<()> {
    recipe.image_id = None;
    recipe.file_id = None;
    recipe.updated_at = event.timestamp;
    Ok(())
}

/// The row of a recipe, and how far its stream was folded into it.
#[derive(FromRow)]
struct Row {
    id: String,
    owner_id: String,
    title: String,
    description: String,
    ingredients: Json<Vec<Ingredient>>,
    instructions: Json<Vec<Instruction>>,
    prep_minutes: i64,
    cook_minutes: i64,
    servings: i64,
    image_id: Option<String>,
    file_id: Option<String>,
    created_at: i64,
    updated_at: i64,
    cursor: String,
    aggregate_version: i64,
}

impl<E: Executor> Snapshot<E> for RecipeState {
    async fn restore(context: &Context<'_, E>) -> anyhow::Result<Option<Self>> {
        let db: Db = context.extract();
        let row: Option<Row> = db
            .fetch_optional(
                "SELECT id, owner_id, title, description, ingredients, instructions,
                    prep_minutes, cook_minutes, servings, image_id, file_id, created_at,
                    updated_at, cursor, aggregate_version
                 FROM recipes WHERE id = ?1",
                (context.id.as_str(),),
            )
            .await?;
        Ok(row.map(|row| RecipeState {
            id: row.id,
            owner_id: row.owner_id,
            title: row.title,
            description: row.description,
            ingredients: row.ingredients.0,
            instructions: row.instructions.0,
            prep_minutes: row.prep_minutes as u32,
            cook_minutes: row.cook_minutes as u32,
            servings: row.servings as u32,
            image_id: row.image_id,
            file_id: row.file_id,
            created_at: row.created_at as u64,
            updated_at: row.updated_at as u64,
            deleted: false,
            cursor: row.cursor,
            aggregate_version: row.aggregate_version as u16,
            ..Default::default()
        }))
    }

    /// A deleted recipe loses its row: loading it again folds its stream
    /// from the start, which says it is deleted.
    async fn take_snapshot(&self, context: &Context<'_, E>) -> anyhow::Result<()> {
        let db: Db = context.extract();
        if self.deleted {
            db.execute("DELETE FROM recipes WHERE id = ?1", (context.id.as_str(),))
                .await?;
            return Ok(());
        }
        db.execute(
            "INSERT INTO recipes (id, owner_id, title, description, ingredients, instructions,
                prep_minutes, cook_minutes, servings, image_id, file_id, search, created_at,
                updated_at, cursor, aggregate_version)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)
             ON CONFLICT (id) DO UPDATE SET
                title = excluded.title, description = excluded.description,
                ingredients = excluded.ingredients, instructions = excluded.instructions,
                prep_minutes = excluded.prep_minutes, cook_minutes = excluded.cook_minutes,
                servings = excluded.servings, image_id = excluded.image_id,
                file_id = excluded.file_id, search = excluded.search,
                updated_at = excluded.updated_at, cursor = excluded.cursor,
                aggregate_version = excluded.aggregate_version",
            (
                context.id.as_str(),
                self.owner_id.as_str(),
                self.title.as_str(),
                self.description.as_str(),
                Json(&self.ingredients).to_text()?,
                Json(&self.instructions).to_text()?,
                self.prep_minutes,
                self.cook_minutes,
                self.servings,
                self.image_id.as_deref(),
                self.file_id.as_deref(),
                searched(&self.title, &self.ingredients),
                self.created_at as i64,
                self.updated_at as i64,
                self.cursor.as_str(),
                i64::from(self.aggregate_version),
            ),
        )
        .await?;
        Ok(())
    }
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
    pub instructions: Json<Vec<Instruction>>,
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

/// The recipe `id`, if there is one, as its row holds it: a moment behind
/// the commands.
pub async fn row(db: &Db, id: &str) -> anyhow::Result<Option<Recipe>> {
    db.fetch_optional(
        "SELECT id, owner_id, title, description, ingredients, instructions, prep_minutes,
            cook_minutes, servings, image_id, file_id, created_at, updated_at
         FROM recipes WHERE id = ?1",
        (id,),
    )
    .await
}

/// The recipe `id`, if there is one, as its stream leaves it: its row and
/// the events written after it, read from the api.
pub async fn get<E: Executor>(executor: &E, db: &Db, id: &str) -> anyhow::Result<Option<Recipe>> {
    let recipe = projection(db.clone()).load(id).execute(executor).await?;
    Ok(recipe.filter(|recipe| !recipe.deleted).map(Recipe::from))
}

impl From<RecipeState> for Recipe {
    fn from(recipe: RecipeState) -> Self {
        Self {
            id: recipe.id,
            owner_id: recipe.owner_id,
            title: recipe.title,
            description: recipe.description,
            ingredients: Json(recipe.ingredients),
            instructions: Json(recipe.instructions),
            prep_minutes: i64::from(recipe.prep_minutes),
            cook_minutes: i64::from(recipe.cook_minutes),
            servings: i64::from(recipe.servings),
            image_id: recipe.image_id,
            file_id: recipe.file_id,
            created_at: recipe.created_at as i64,
            updated_at: recipe.updated_at as i64,
        }
    }
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

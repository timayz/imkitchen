//! What app knows of the recipes: all each one says, whose it is and the
//! picture it shows.
//!
//! A recipe is its owner's alone: but for how many there are, nothing is
//! read without saying whose is asked for.
//!
//! A recipe is a projection of its stream ([`RecipeState`]) whose snapshot
//! is its row of the `recipes` table: one row per recipe that is not
//! deleted, kept by a projection subscription ([`subscription`]). One recipe
//! is loaded from its row and the events written after it ([`get_own`]), so
//! a page shows it as the command its user just handed the api left it. The
//! lists are read from the table, a moment behind the commands.

use evento::Executor;
use evento::metadata::Event;
use evento::projection::{Context, Projection, ProjectionSubscription, Snapshot};
use imkitchen_aggregates::recipe::{
    self, Created, Deleted, Edited, ImageRemoved, ImageSet, Recipe as RecipeStream,
};
use serde::{Deserialize, Serialize};
use timada_core::db::{FromRow, Json, Value, params_from_iter, placeholders, search_key};
use timada_core::page::Cursor;
use timada_core::{Rendition, like_contains};

use crate::db::Db;

/// The cursor identity of the subscription, in the store.
pub const SUBSCRIPTION: &str = "imkitchen-app/recipes";

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

/// One step of a recipe, as the table keeps it and as the api tells of it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Instruction {
    pub text: String,
    /// The minutes to wait once it is done, when there is something to
    /// wait for.
    #[serde(default)]
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
    /// The files of the picture it shows, when it shows one, and the size
    /// of the larger of the two.
    pub file_id: Option<String>,
    pub image_width: Option<u32>,
    pub image_height: Option<u32>,
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
    let large = image.picture.rendition(Rendition::Page);
    recipe.file_id = Some(image.file_id.to_owned());
    recipe.image_width = Some(large.width);
    recipe.image_height = Some(large.height);
    recipe.updated_at = event.timestamp;
    Ok(())
}

#[evento::handler]
async fn on_image_removed(
    event: Event<ImageRemoved>,
    recipe: &mut RecipeState,
) -> anyhow::Result<()> {
    recipe.file_id = None;
    recipe.image_width = None;
    recipe.image_height = None;
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
    file_id: Option<String>,
    image_width: Option<i64>,
    image_height: Option<i64>,
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
                    prep_minutes, cook_minutes, servings, file_id, image_width, image_height,
                    created_at, updated_at, cursor, aggregate_version
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
            file_id: row.file_id,
            image_width: row.image_width.map(|width| width as u32),
            image_height: row.image_height.map(|height| height as u32),
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
                prep_minutes, cook_minutes, servings, file_id, image_width, image_height, search,
                created_at, updated_at, cursor, aggregate_version)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)
             ON CONFLICT (id) DO UPDATE SET
                title = excluded.title, description = excluded.description,
                ingredients = excluded.ingredients, instructions = excluded.instructions,
                prep_minutes = excluded.prep_minutes, cook_minutes = excluded.cook_minutes,
                servings = excluded.servings, file_id = excluded.file_id,
                image_width = excluded.image_width, image_height = excluded.image_height,
                search = excluded.search, updated_at = excluded.updated_at,
                cursor = excluded.cursor, aggregate_version = excluded.aggregate_version",
            // More values than a tuple of parameters takes.
            params_from_iter([
                Value::from(context.id.as_str()),
                Value::from(self.owner_id.as_str()),
                Value::from(self.title.as_str()),
                Value::from(self.description.as_str()),
                Value::from(Json(&self.ingredients).to_text()?),
                Value::from(Json(&self.instructions).to_text()?),
                Value::from(self.prep_minutes),
                Value::from(self.cook_minutes),
                Value::from(self.servings),
                nullable(self.file_id.as_deref()),
                nullable(self.image_width),
                nullable(self.image_height),
                Value::from(searched(&self.title, &self.ingredients)),
                Value::from(self.created_at as i64),
                Value::from(self.updated_at as i64),
                Value::from(self.cursor.as_str()),
                Value::from(i64::from(self.aggregate_version)),
            ]),
        )
        .await?;
        Ok(())
    }
}

/// `value` as a parameter: NULL when there is none.
fn nullable<T: Into<Value>>(value: Option<T>) -> Value {
    value.map_or(Value::Null, Into::into)
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

impl From<RecipeState> for Recipe {
    fn from(recipe: RecipeState) -> Self {
        Self {
            id: recipe.id,
            title: recipe.title,
            description: recipe.description,
            ingredients: Json(recipe.ingredients),
            instructions: Json(recipe.instructions),
            prep_minutes: i64::from(recipe.prep_minutes),
            cook_minutes: i64::from(recipe.cook_minutes),
            servings: i64::from(recipe.servings),
            file_id: recipe.file_id,
            image_width: recipe.image_width.map(i64::from),
            image_height: recipe.image_height.map(i64::from),
            updated_at: recipe.updated_at as i64,
        }
    }
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

/// The recipe `id`, when it is one of `owner_id`'s, as its row holds it: a
/// moment behind the commands. Someone else's is one that does not exist.
pub async fn row_own(db: &Db, owner_id: &str, id: &str) -> anyhow::Result<Option<Recipe>> {
    db.fetch_optional(
        "SELECT id, title, description, ingredients, instructions, prep_minutes, cook_minutes,
            servings, file_id, image_width, image_height, updated_at
         FROM recipes WHERE id = ?1 AND owner_id = ?2",
        (id, owner_id),
    )
    .await
}

/// The recipe `id`, when it is one of `owner_id`'s, as its stream leaves
/// it: its row and the events written after it, read from the api. Someone
/// else's is one that does not exist.
pub async fn get_own<E: Executor>(
    executor: &E,
    db: &Db,
    owner_id: &str,
    id: &str,
) -> anyhow::Result<Option<Recipe>> {
    let recipe = projection(db.clone()).load(id).execute(executor).await?;
    Ok(recipe
        .filter(|recipe| !recipe.deleted && recipe.owner_id == owner_id)
        .map(Recipe::from))
}

/// What the lists show of the recipes `ids`, in no order: those among them
/// that are `owner_id`'s.
pub async fn listed_own(db: &Db, owner_id: &str, ids: &[String]) -> anyhow::Result<Vec<RecipeRow>> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    let mut params: Vec<&str> = ids.iter().map(String::as_str).collect();
    params.push(owner_id);
    db.fetch_all(
        &format!(
            "SELECT id, title, prep_minutes, cook_minutes, servings, file_id, created_at
             FROM recipes WHERE id IN ({}) AND owner_id = ?{}",
            placeholders(ids.len()),
            params.len()
        ),
        params_from_iter(params),
    )
    .await
}

/// What the groceries take of a recipe: its ingredients, and how many it
/// is written for.
#[derive(Debug, Clone, FromRow, PartialEq)]
pub struct Taken {
    pub id: String,
    pub ingredients: Json<Vec<Ingredient>>,
    pub servings: i64,
}

/// What the groceries take of the recipes `ids`, in no order: those among
/// them that are `owner_id`'s.
pub async fn taken_own(db: &Db, owner_id: &str, ids: &[String]) -> anyhow::Result<Vec<Taken>> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    let mut params: Vec<&str> = ids.iter().map(String::as_str).collect();
    params.push(owner_id);
    db.fetch_all(
        &format!(
            "SELECT id, ingredients, servings
             FROM recipes WHERE id IN ({}) AND owner_id = ?{}",
            placeholders(ids.len()),
            params.len()
        ),
        params_from_iter(params),
    )
    .await
}

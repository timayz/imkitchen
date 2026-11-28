//! The read side of the recipes: the `recipes` table a user's list is read
//! from, each row the snapshot of a projection of the recipe's stream
//! ([`Listed`]), kept by a projection subscription ([`subscription`]).
//!
//! A deleted recipe loses its row. Every node runs the subscription over
//! its own file, with a cursor of its own: a list is a moment behind the
//! command that changed it, on whichever node answers. One recipe is read
//! from its stream ([`super::model`]), never from the table.

use evento::Executor;
use evento::metadata::Event;
use evento::projection::{Context, Projection, ProjectionSubscription, Snapshot};
use imkitchen_aggregates::recipe::{Created, Deleted, Edited, ImageRemoved, ImageSet, Recipe};
use timada_core::db::{FromRow, Json, params_from_iter, placeholders, search_key};
use timada_core::like_contains;
use timada_core::page::Cursor;

use super::Image;
use crate::db::Db;

/// The cursor identity of the subscription, in the store.
pub const SUBSCRIPTION: &str = "imkitchen-api/recipes";

/// A recipe as its row of `recipes` holds it: what the lists read of it.
#[evento::projection]
#[derive(Debug)]
pub struct Listed {
    pub id: String,
    pub owner_id: String,
    pub title: String,
    pub description: String,
    /// What a search looks in: the title and the name of each ingredient,
    /// a line each, without case or accents.
    pub search: String,
    pub prep_minutes: u32,
    pub cook_minutes: u32,
    pub servings: u32,
    /// The picture it shows, as the list answers it, and the stream it is
    /// known under.
    pub image: Option<Image>,
    pub image_id: Option<String>,
    /// Unix seconds of `Created`.
    pub created_at: u64,
    /// Unix seconds of the last event.
    pub updated_at: u64,
    pub deleted: bool,
}

fn projection<E: Executor>(db: Db) -> Projection<E, Listed> {
    Projection::new::<Recipe>()
        .handler(on_created())
        .handler(on_edited())
        .handler(on_deleted())
        .handler(on_image_set())
        .handler(on_image_removed())
        .data(db)
}

/// Keeps the row of each recipe in `db` as its stream leaves it. Start it
/// over a handle that reads this node's file as it stands: what it loads is
/// behind the event it handles.
pub fn subscription<E: Executor>(db: Db) -> ProjectionSubscription<E, Listed> {
    projection(db).subscription(SUBSCRIPTION)
}

/// What a search looks in: the title and the name of each ingredient, a
/// line each, without case or accents.
fn searched(title: &str, ingredients: &[imkitchen_aggregates::recipe::Ingredient]) -> String {
    let lines: Vec<&str> = std::iter::once(title)
        .chain(ingredients.iter().map(|one| one.name.as_str()))
        .collect();
    search_key(&lines.join("\n"))
}

#[evento::handler]
async fn on_created(event: Event<Created>, recipe: &mut Listed) -> anyhow::Result<()> {
    let data = &event.data;
    recipe.id = event.aggregate_id.to_owned();
    recipe.owner_id = data.owner_id.to_owned();
    recipe.title = data.title.to_owned();
    recipe.description = data.description.to_owned();
    recipe.search = searched(&data.title, &data.ingredients);
    recipe.prep_minutes = data.prep_minutes;
    recipe.cook_minutes = data.cook_minutes;
    recipe.servings = data.servings;
    recipe.created_at = event.timestamp;
    recipe.updated_at = event.timestamp;
    Ok(())
}

#[evento::handler]
async fn on_edited(event: Event<Edited>, recipe: &mut Listed) -> anyhow::Result<()> {
    let data = &event.data;
    recipe.title = data.title.to_owned();
    recipe.description = data.description.to_owned();
    recipe.search = searched(&data.title, &data.ingredients);
    recipe.prep_minutes = data.prep_minutes;
    recipe.cook_minutes = data.cook_minutes;
    recipe.servings = data.servings;
    recipe.updated_at = event.timestamp;
    Ok(())
}

#[evento::handler]
async fn on_deleted(event: Event<Deleted>, recipe: &mut Listed) -> anyhow::Result<()> {
    recipe.deleted = true;
    recipe.updated_at = event.timestamp;
    Ok(())
}

#[evento::handler]
async fn on_image_set(event: Event<ImageSet>, recipe: &mut Listed) -> anyhow::Result<()> {
    let image = &event.data.image;
    recipe.image = Some(Image::from(image));
    recipe.image_id = Some(image.image_id.to_owned());
    recipe.updated_at = event.timestamp;
    Ok(())
}

#[evento::handler]
async fn on_image_removed(event: Event<ImageRemoved>, recipe: &mut Listed) -> anyhow::Result<()> {
    recipe.image = None;
    recipe.image_id = None;
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
    search: String,
    prep_minutes: i64,
    cook_minutes: i64,
    servings: i64,
    image: Option<Json<Image>>,
    image_id: Option<String>,
    created_at: i64,
    updated_at: i64,
    cursor: String,
    aggregate_version: i64,
}

impl<E: Executor> Snapshot<E> for Listed {
    async fn restore(context: &Context<'_, E>) -> anyhow::Result<Option<Self>> {
        let db: Db = context.extract();
        let row: Option<Row> = db
            .fetch_optional(
                "SELECT id, owner_id, title, description, search, prep_minutes, cook_minutes,
                    servings, image, image_id, created_at, updated_at, cursor, aggregate_version
                 FROM recipes WHERE id = ?1",
                (context.id.as_str(),),
            )
            .await?;
        Ok(row.map(|row| Listed {
            id: row.id,
            owner_id: row.owner_id,
            title: row.title,
            description: row.description,
            search: row.search,
            prep_minutes: row.prep_minutes as u32,
            cook_minutes: row.cook_minutes as u32,
            servings: row.servings as u32,
            image: row.image.map(|image| image.0),
            image_id: row.image_id,
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
        let image = self
            .image
            .as_ref()
            .map(Json)
            .map(|image| image.to_text())
            .transpose()?;
        db.execute(
            "INSERT INTO recipes (id, owner_id, title, description, prep_minutes, cook_minutes,
                servings, search, created_at, updated_at, image, image_id, cursor,
                aggregate_version)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
             ON CONFLICT (id) DO UPDATE SET
                title = excluded.title, description = excluded.description,
                prep_minutes = excluded.prep_minutes, cook_minutes = excluded.cook_minutes,
                servings = excluded.servings, search = excluded.search,
                updated_at = excluded.updated_at, image = excluded.image,
                image_id = excluded.image_id, cursor = excluded.cursor,
                aggregate_version = excluded.aggregate_version",
            (
                context.id.as_str(),
                self.owner_id.as_str(),
                self.title.as_str(),
                self.description.as_str(),
                self.prep_minutes,
                self.cook_minutes,
                self.servings,
                self.search.as_str(),
                self.created_at as i64,
                self.updated_at as i64,
                image,
                self.image_id.as_deref(),
                self.cursor.as_str(),
                i64::from(self.aggregate_version),
            ),
        )
        .await?;
        Ok(())
    }
}

/// One row of `recipes`: what a list shows of a recipe.
#[derive(Debug, Clone, FromRow, PartialEq)]
pub struct RecipeRow {
    pub id: String,
    pub title: String,
    pub description: String,
    pub prep_minutes: i64,
    pub cook_minutes: i64,
    pub servings: i64,
    /// Unix seconds.
    pub created_at: i64,
    /// Unix seconds.
    pub updated_at: i64,
    /// The picture it shows, as the list answers it.
    pub image: Option<Json<Image>>,
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

/// The recipes of `owner_id`, the most recent first, `limit` at most: those
/// after `after` in the list, and only those `q` is found in (their title or
/// an ingredient, whatever the case and the accents) when there is one.
pub async fn recipes_of(
    db: &Db,
    owner_id: &str,
    q: Option<&str>,
    after: Option<&Cursor>,
    limit: i64,
) -> anyhow::Result<Vec<RecipeRow>> {
    let (at, id) = after.map_or((i64::MAX, ""), |cursor| (cursor.at, cursor.id.as_str()));
    db.fetch_all(
        "SELECT id, title, description, prep_minutes, cook_minutes, servings, created_at,
            updated_at, image
         FROM recipes INDEXED BY recipes_listed
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

/// The rows of the recipes `ids`, in no order: those that have one.
pub async fn recipes_in(db: &Db, ids: &[String]) -> anyhow::Result<Vec<RecipeRow>> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    db.fetch_all(
        &format!(
            "SELECT id, title, description, prep_minutes, cook_minutes, servings, created_at,
                updated_at, image
             FROM recipes WHERE id IN ({})",
            placeholders(ids.len())
        ),
        params_from_iter(ids.iter().map(String::as_str)),
    )
    .await
}

#[derive(FromRow)]
struct Id {
    id: String,
}

/// The ids of every recipe of `owner_id` the table knows, in no order.
pub async fn ids_of(db: &Db, owner_id: &str) -> anyhow::Result<Vec<String>> {
    let ids: Vec<Id> = db
        .fetch_all("SELECT id FROM recipes WHERE owner_id = ?1", (owner_id,))
        .await?;
    Ok(ids.into_iter().map(|row| row.id).collect())
}

/// The ids of every recipe the table knows to show the picture `image_id`,
/// whoever's they are, in no order.
pub async fn showing(db: &Db, image_id: &str) -> anyhow::Result<Vec<String>> {
    let ids: Vec<Id> = db
        .fetch_all("SELECT id FROM recipes WHERE image_id = ?1", (image_id,))
        .await?;
    Ok(ids.into_iter().map(|row| row.id).collect())
}

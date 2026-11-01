use std::collections::BTreeMap;

use evento::{Aggregate, AggregateEvent, metadata::Metadata};
use imkitchen_types::favorite::{Favorite, Saved, Unsaved};
use sqlx_migrator::vec_box;

pub struct Migration;

// `evento::hash_ids` gained a per-id length prefix in 2.0.0-alpha.28
// (imkitchen 1.9.1), so favorites saved before then live in streams keyed by
// the old hash of `[recipe_id, user_id]`, where the detail page no longer
// looks. Move every Favorite event onto the stream id the current `hash_ids`
// gives it.
sqlx_migrator::sqlite_migration!(
    Migration,
    "imkitchen",
    "m0018",
    vec_box![super::m0017::Migration],
    vec_box![RekeyFavorites]
);

/// Rewrites `aggregator_id` of every Favorite event to
/// `hash_ids([recipe_id, requested_by])`. A recipe re-saved while its old
/// stream was unreachable has events on both the old and the current id:
/// they merge into one stream, renumbered in cursor order. Favorite snapshots
/// are dropped so the merged streams are rebuilt from their events.
pub struct RekeyFavorites;

/// One Favorite event, as stored.
#[derive(sqlx::FromRow)]
struct Row {
    id: String,
    aggregator_id: String,
    name: String,
    version: i64,
    data: Vec<u8>,
    metadata: Vec<u8>,
}

impl Row {
    /// The stream id the event belongs to under the current `hash_ids`.
    fn current_id(&self) -> Result<String, sqlx_migrator::Error> {
        let recipe_id = if self.name == Saved::event_name() {
            decode::<Saved>(&self.data)?.recipe_id
        } else if self.name == Unsaved::event_name() {
            decode::<Unsaved>(&self.data)?.recipe_id
        } else {
            return Err(boxed(format!(
                "unknown Favorite event {} ({})",
                self.name, self.id
            )));
        };
        let user_id = decode::<Metadata>(&self.metadata)?
            .requested_by()
            .map_err(|err| boxed(format!("event {}: {err}", self.id)))?;

        Ok(evento::hash_ids(vec![recipe_id, user_id]))
    }
}

fn decode<T: for<'a> bitcode::Decode<'a>>(bytes: &[u8]) -> Result<T, sqlx_migrator::Error> {
    bitcode::decode(bytes).map_err(|err| boxed(err.to_string()))
}

fn boxed(message: String) -> sqlx_migrator::Error {
    sqlx_migrator::Error::Box(message.into())
}

#[async_trait::async_trait]
impl sqlx_migrator::Operation<sqlx::Sqlite> for RekeyFavorites {
    async fn up(
        &self,
        connection: &mut sqlx::SqliteConnection,
    ) -> Result<(), sqlx_migrator::Error> {
        let aggregate_type = Favorite::aggregate_type();
        let rows: Vec<Row> = sqlx::query_as(
            "SELECT id, aggregator_id, name, version, data, metadata FROM event \
             WHERE aggregator_type = ? AND tenant = '' \
             ORDER BY timestamp, timestamp_subsec, version, id",
        )
        .bind(aggregate_type)
        .fetch_all(&mut *connection)
        .await?;

        // Events per current stream id, in cursor order.
        let mut streams: BTreeMap<String, Vec<Row>> = BTreeMap::new();
        for row in rows {
            streams.entry(row.current_id()?).or_default().push(row);
        }

        let mut rekeyed = false;
        for (current_id, events) in streams {
            let in_place = events.iter().enumerate().all(|(i, event)| {
                event.aggregator_id == current_id && event.version == i as i64 + 1
            });
            if in_place {
                continue;
            }
            rekeyed = true;

            // Two passes, since the unique `(type, id, tenant, version)` index
            // is checked per row: park the events under a temporary id with
            // their final versions, then move them all to the current id.
            let parking = format!("m0018-{}", &current_id[6..]);
            for (i, event) in events.iter().enumerate() {
                sqlx::query("UPDATE event SET aggregator_id = ?, version = ? WHERE id = ?")
                    .bind(&parking)
                    .bind(i as i64 + 1)
                    .bind(&event.id)
                    .execute(&mut *connection)
                    .await?;
            }
            sqlx::query("UPDATE event SET aggregator_id = ? WHERE aggregator_id = ?")
                .bind(&current_id)
                .bind(&parking)
                .execute(&mut *connection)
                .await?;
        }

        if rekeyed {
            sqlx::query("DELETE FROM snapshot WHERE type = ?")
                .bind(aggregate_type)
                .execute(&mut *connection)
                .await?;
        }

        Ok(())
    }

    /// The old stream ids are gone and merged streams cannot be split again;
    /// the rewritten streams are what every evento version since alpha.28
    /// expects, so there is nothing to undo.
    async fn down(
        &self,
        _connection: &mut sqlx::SqliteConnection,
    ) -> Result<(), sqlx_migrator::Error> {
        Ok(())
    }
}

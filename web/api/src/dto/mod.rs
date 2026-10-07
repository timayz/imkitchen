//! Wire types. Read models derive `Deserialize` only and wrap columns in
//! `sqlx::types::Text` / `Bitcode`, so the API maps them into these explicit
//! `Serialize` structs. Enums travel as their PascalCase variant names.

pub mod auth;
pub mod grocery;
pub mod kitchen;
pub mod recipe;
pub mod settings;

use serde::Serialize;
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

/// RFC 3339 timestamps (`2026-10-19T18:03:12Z`); what every JS `Date` parses.
pub fn rfc3339<S: serde::Serializer>(
    value: &OffsetDateTime,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let text = value.format(&Rfc3339).map_err(serde::ser::Error::custom)?;
    serializer.serialize_str(&text)
}

/// `{"id": "..."}` for creation endpoints.
#[derive(Serialize)]
pub struct Created {
    pub id: String,
}

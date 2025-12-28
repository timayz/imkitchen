use sqlx::prelude::FromRow;

#[derive(Default, FromRow)]
pub struct UserStatRow {
    pub total: u32,
    pub premium: u32,
    pub suspended: u32,
}

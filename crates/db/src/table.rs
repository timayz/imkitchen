use sea_query::Iden;

#[derive(Iden, Clone)]
pub enum User {
    Table,
    Id,
    Email,
    Role,
    CreatedAt,
}

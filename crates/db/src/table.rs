use sea_query::Iden;

#[derive(Iden, Clone)]
pub enum User {
    Table,
    Id,
    Email,
    Role,
    CreatedAt,
}

#[derive(Iden, Clone)]
pub enum GlobalStatPjt {
    Table,
    Key,
    Value,
}

#[derive(Iden, Clone)]
pub enum AdminUserPjt {
    Table,
    Id,
    Email,
    FullName,
    Username,
    Status,
    AccountType,
    TotalRecipesCount,
    SharedRecipesCount,
    CreatedAt,
}

use sea_query::Iden;

#[derive(Iden, Clone)]
pub enum User {
    Table,
    Id,
    Email,
    Role,
    State,
    SubscriptionEndAt,
    CreatedAt,
}

#[derive(Iden, Clone)]
pub enum GlobalStatPjt {
    Table,
    Key,
    Value,
}

#[derive(Iden, Clone)]
pub enum ContactPjt {
    Table,
    Id,
    Email,
    Name,
    Status,
    Subject,
    Message,
    CreatedAt,
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
    TotalActiveCount,
    CreatedAt,
}

#[derive(Iden, Clone)]
pub enum RecipeList {
    Table,
    Id,
    UserId,
    RecipeType,
    CuisineType,
    Name,
    Description,
    PrepTime,
    CookTime,
    Ingredients,
    Instructions,
    DietaryRestrictions,
    AcceptsAccompaniment,
    PreferredAccompanimentTypes,
    AdvancePrep,
    IsShared,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden, Clone)]
pub enum RecipeUserStat {
    Table,
    UserId,
    Total,
    Shared,
    Favorite,
    FromCommunity,
    CreatedAt,
}

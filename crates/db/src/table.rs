use sea_query::Iden;

#[derive(Iden, Clone)]
pub enum User {
    Table,
    Id,
    Email,
    Password,
    Username,
    Role,
    State,
    CreatedAt,
}

#[derive(Iden, Clone)]
pub enum UserLogin {
    Table,
    Id,
    Cursor,
    Username,
    Role,
    State,
    SubscriptionExpireAt,
    Logins,
}

#[derive(Iden, Clone)]
pub enum UserGlobalStat {
    Table,
    Month,
    Total,
    Premium,
    Suspended,
    CreatedAt,
}

#[derive(Iden, Clone)]
pub enum UserAdmin {
    Table,
    Id,
    Cursor,
    Email,
    FullName,
    Username,
    State,
    Role,
    SubscriptionExpireAt,
    TotalRecipesCount,
    SharedRecipesCount,
    TotalActiveCount,
    CreatedAt,
}

#[derive(Iden, Clone)]
pub enum UserAdminFts {
    Table,
    Id,
    Email,
    Username,
    Rank,
}

#[derive(Iden, Clone)]
pub enum ContactAdmin {
    Table,
    Id,
    Cursor,
    Email,
    Name,
    Status,
    Subject,
    Message,
    CreatedAt,
}

#[derive(Iden, Clone)]
pub enum ContactAdminFts {
    Table,
    Id,
    Email,
    Name,
    Message,
    Rank,
}

#[derive(Iden, Clone)]
pub enum ContactGlobalStat {
    Table,
    Day,
    Total,
    Today,
    Unread,
    AvgResponseTime,
    CreatedAt,
}

#[derive(Iden, Clone)]
pub enum RecipeUser {
    Table,
    Id,
    Cursor,
    OwnerId,
    OwnerName,
    RecipeType,
    CuisineType,
    Name,
    Description,
    HouseholdSize,
    PrepTime,
    CookTime,
    Ingredients,
    Instructions,
    DietaryRestrictions,
    AcceptsAccompaniment,
    AdvancePrep,
    IsShared,
    TotalViews,
    TotalLikes,
    TotalComments,
    CreatedAt,
    UpdatedAt,
    ThumbnailVersion,
}

#[derive(Iden, Clone)]
pub enum RecipeUserFts {
    Table,
    Id,
    Name,
    Description,
    Ingredients,
    Rank,
}

#[derive(Iden, Clone)]
pub enum RecipeThumbnail {
    Table,
    Id,
    Device,
    Data,
}

#[derive(Iden, Clone)]
pub enum RecipeComment {
    Table,
    Id,
    RecipeId,
    Cursor,
    OwnerId,
    ReplyTo,
    OwnerName,
    Body,
    TotalLikes,
    TotalReplies,
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

#[derive(Iden, Clone)]
pub enum MealPlanRecipe {
    Table,
    Id,
    UserId,
    RecipeType,
    Name,
    AdvancePrep,
    PrepTime,
    CookTime,
    AcceptsAccompaniment,
    DietaryRestrictions,
}

#[derive(Iden, Clone)]
pub enum MealPlanWeek {
    Table,
    UserId,
    Start,
    End,
    Slots,
}

#[derive(Iden, Clone)]
pub enum MealPlanSlot {
    Table,
    UserId,
    Day,
    Appetizer,
    MainCourse,
    Accompaniment,
    Dessert,
}

#[derive(Iden, Clone)]
pub enum ShoppingRecipe {
    Table,
    Id,
    UserId,
    HouseholdSize,
    Ingredients,
}

#[derive(Iden, Clone)]
pub enum ShoppingList {
    Table,
    UserId,
    Week,
    Ingredients,
}

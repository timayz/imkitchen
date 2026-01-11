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
pub enum ContactAdmin {
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
pub enum RecipeCommand {
    Table,
    Id,
    OwnerId,
    RecipeType,
    CuisineType,
    IsShared,
    BasicInformationHash,
    IngredientsHash,
    InstructionsHash,
    DietaryRestrictionsHash,
    AdvancePrepHash,
    AcceptsAccompaniment,
    IsDeleted,
    Version,
    RoutingKey,
}

#[derive(Iden, Clone)]
pub enum RecipeUser {
    Table,
    Id,
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
pub enum RecipeRatingCommand {
    Table,
    RecipeId,
    UserId,
    Viewed,
    Liked,
    Unliked,
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
pub enum MealPlanLastWeek {
    Table,
    UserId,
    Start,
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

use sea_query::Iden;

#[derive(Iden, Clone)]
pub enum User {
    Table,
    Id,
    Email,
    Username,
    Role,
    State,
    SubscriptionExpireAt,
    CreatedAt,
}

#[derive(Iden, Clone)]
pub enum UserLogin {
    Table,
    Id,
    UserId,
    Revision,
    UserAgent,
    CreatedAt,
}

#[derive(Iden, Clone)]
pub enum UserStat {
    Table,
    Day,
    Total,
    Premium,
    Suspended,
}

#[derive(Iden, Clone)]
pub enum UserList {
    Table,
    Id,
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
pub enum ContactList {
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
pub enum ContactStat {
    Table,
    Day,
    Total,
    Unread,
    AvgResponseTime,
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
    HouseholdSize,
    PrepTime,
    CookTime,
    Ingredients,
    Instructions,
    DietaryRestrictions,
    AcceptsAccompaniment,
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

#[derive(Iden, Clone)]
pub enum MealPlanRecipe {
    Table,
    Id,
    UserId,
    RecipeType,
    CuisineType,
    Name,
    HouseholdSize,
    PrepTime,
    CookTime,
    Ingredients,
    Instructions,
    DietaryRestrictions,
    AcceptsAccompaniment,
    AdvancePrep,
}

#[derive(Iden, Clone)]
pub enum MealPlanWeek {
    Table,
    UserId,
    Start,
    End,
    Status,
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
pub enum ShoppingList {
    Table,
    UserId,
    Week,
    Ingredients,
}

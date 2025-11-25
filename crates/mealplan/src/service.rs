use imkitchen_db::table::MealPlanRecipe;
use imkitchen_recipe::RecipeType;
use rand::seq::SliceRandom;
use sea_query::{
    Expr, ExprTrait, Func, IntoColumnRef, Order, Query, SimpleExpr, SqliteQueryBuilder,
};
use sea_query_sqlx::SqlxBinder;
use time::{Duration, OffsetDateTime, Weekday};

pub async fn has(
    pool: &sqlx::SqlitePool,
    id: impl Into<String>,
    recipe_type: RecipeType,
) -> imkitchen_shared::Result<bool> {
    let id = id.into();
    let statement = Query::select()
        .columns([MealPlanRecipe::Id, MealPlanRecipe::Name])
        .from(MealPlanRecipe::Table)
        .and_where(Expr::col(MealPlanRecipe::UserId).eq(id))
        .and_where(Expr::col(MealPlanRecipe::RecipeType).eq(recipe_type.to_string()))
        .and_where(Expr::col(MealPlanRecipe::Name).is_not(""))
        .limit(1)
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);

    let recipe = sqlx::query_as_with::<_, (String,), _>(&sql, values)
        .fetch_optional(pool)
        .await?;

    Ok(recipe.is_some())
}

pub async fn random(
    pool: &sqlx::SqlitePool,
    id: impl Into<String>,
    recipe_type: RecipeType,
) -> imkitchen_shared::Result<Vec<String>> {
    let id = id.into();
    let statement = Query::select()
        .columns([MealPlanRecipe::Id, MealPlanRecipe::Name])
        .from(MealPlanRecipe::Table)
        .and_where(
            MealPlanRecipe::Id.into_column_ref().in_subquery(
                Query::select()
                    .columns([MealPlanRecipe::Id])
                    .from(MealPlanRecipe::Table)
                    .and_where(Expr::col(MealPlanRecipe::UserId).eq(id))
                    .and_where(Expr::col(MealPlanRecipe::RecipeType).eq(recipe_type.to_string()))
                    .and_where(Expr::col(MealPlanRecipe::Name).is_not(""))
                    .order_by_expr(SimpleExpr::FunctionCall(Func::random()), Order::Asc)
                    .limit(7 * 4)
                    .take(),
            ),
        )
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);

    let mut recipes = sqlx::query_as_with::<_, (String,), _>(&sql, values)
        .fetch_all(pool)
        .await?
        .iter()
        .map(|(id,)| id.to_owned())
        .collect::<Vec<_>>();

    let mut rng = rand::rng();
    recipes.shuffle(&mut rng);

    Ok(recipes)
}

/// Returns the timestamps of the next 4 Mondays from now
/// All timestamps are set to 00:00:00 (midnight)
pub fn next_four_mondays_from_now() -> anyhow::Result<[u64; 4]> {
    next_four_mondays(OffsetDateTime::now_utc().unix_timestamp() as u64)
}

/// Returns the timestamps of the next 4 Mondays from the given timestamp
/// All timestamps are set to 00:00:00 (midnight)
pub fn next_four_mondays(from_timestamp: u64) -> anyhow::Result<[u64; 4]> {
    let from_date = OffsetDateTime::from_unix_timestamp(from_timestamp as i64)?;
    let mut mondays = [0u64; 4];

    // Get the current weekday
    let current_weekday = from_date.weekday();

    // Calculate days until next Monday
    let days_until_monday = match current_weekday {
        Weekday::Monday => 7, // If today is Monday, get next Monday
        Weekday::Tuesday => 6,
        Weekday::Wednesday => 5,
        Weekday::Thursday => 4,
        Weekday::Friday => 3,
        Weekday::Saturday => 2,
        Weekday::Sunday => 1,
    };

    // Calculate the first Monday and reset time to 00:00:00
    let first_monday = from_date + Duration::days(days_until_monday);
    let first_monday = first_monday.replace_time(time::Time::MIDNIGHT);

    // Calculate all 4 Mondays
    for i in 0..4 {
        let monday = first_monday + Duration::weeks(i as i64);
        mondays[i as usize] = monday.unix_timestamp() as u64;
    }

    Ok(mondays)
}

/// Returns the timestamps of the next 4 Mondays from the given timestamp
/// All timestamps are set to 00:00:00 (midnight)
pub fn week_monday_of(from_timestamp: u64) -> anyhow::Result<u64> {
    let from_date = OffsetDateTime::from_unix_timestamp(from_timestamp as i64)?;

    // Get the current weekday
    let current_weekday = from_date.weekday();

    // Calculate days until next Monday
    let days_until_monday = match current_weekday {
        Weekday::Monday => 7, // If today is Monday, get next Monday
        Weekday::Tuesday => 6,
        Weekday::Wednesday => 5,
        Weekday::Thursday => 4,
        Weekday::Friday => 3,
        Weekday::Saturday => 2,
        Weekday::Sunday => 1,
    };

    // Calculate the first Monday and reset time to 00:00:00
    let first_monday = from_date + Duration::days(days_until_monday);
    let first_monday = first_monday.replace_time(time::Time::MIDNIGHT);

    Ok(first_monday.unix_timestamp() as u64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::datetime;

    #[test]
    fn test_next_four_mondays_from_monday() {
        // Start from Monday, 2025-01-20
        let monday = datetime!(2025-01-20 12:00:00 UTC);
        let timestamp = monday.unix_timestamp();

        let result = next_four_mondays(timestamp as u64).unwrap();

        // Should get the next 4 Mondays at 00:00:00: Jan 27, Feb 3, Feb 10, Feb 17
        let expected = [
            datetime!(2025-01-27 00:00:00 UTC).unix_timestamp() as u64,
            datetime!(2025-02-03 00:00:00 UTC).unix_timestamp() as u64,
            datetime!(2025-02-10 00:00:00 UTC).unix_timestamp() as u64,
            datetime!(2025-02-17 00:00:00 UTC).unix_timestamp() as u64,
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_next_four_mondays_from_wednesday() {
        // Start from Wednesday, 2025-01-22
        let wednesday = datetime!(2025-01-22 12:00:00 UTC);
        let timestamp = wednesday.unix_timestamp();

        let result = next_four_mondays(timestamp as u64).unwrap();

        // Should get the next 4 Mondays at 00:00:00: Jan 27, Feb 3, Feb 10, Feb 17
        let expected = [
            datetime!(2025-01-27 00:00:00 UTC).unix_timestamp() as u64,
            datetime!(2025-02-03 00:00:00 UTC).unix_timestamp() as u64,
            datetime!(2025-02-10 00:00:00 UTC).unix_timestamp() as u64,
            datetime!(2025-02-17 00:00:00 UTC).unix_timestamp() as u64,
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_next_four_mondays_from_sunday() {
        // Start from Sunday, 2025-01-26
        let sunday = datetime!(2025-01-26 12:00:00 UTC);
        let timestamp = sunday.unix_timestamp();

        let result = next_four_mondays(timestamp as u64).unwrap();

        // Should get the next 4 Mondays at 00:00:00: Jan 27, Feb 3, Feb 10, Feb 17
        let expected = [
            datetime!(2025-01-27 00:00:00 UTC).unix_timestamp() as u64,
            datetime!(2025-02-03 00:00:00 UTC).unix_timestamp() as u64,
            datetime!(2025-02-10 00:00:00 UTC).unix_timestamp() as u64,
            datetime!(2025-02-17 00:00:00 UTC).unix_timestamp() as u64,
        ];

        assert_eq!(result, expected);
    }
}

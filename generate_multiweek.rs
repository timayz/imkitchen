// Temporary script to generate multi-week meal plan for john.doe@imkitchen.localhost
use sqlx::SqlitePool;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to database
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:imkitchen.db".to_string());

    let pool = SqlitePool::connect(&database_url).await?;
    let executor: evento::Sqlite = pool.clone().into();

    let user_id = "01K8M5WQ7DBKK1XJN3HS0H03TN"; // john.doe@imkitchen.localhost

    // Load favorite recipes
    let recipes: Vec<meal_planning::RecipeForPlanning> = sqlx::query_as(
        "SELECT id, user_id, title, recipe_type, complexity,
         prep_time_min, cook_time_min, advance_prep_hours, cuisine, dietary_tags,
         accepts_accompaniment, preferred_accompaniments, accompaniment_category
         FROM recipes
         WHERE user_id = ? AND is_favorite = 1 AND deleted_at IS NULL"
    )
    .bind(user_id)
    .fetch_all(&pool)
    .await?;

    println!("Found {} favorite recipes", recipes.len());

    // Load user preferences
    let prefs = meal_planning::UserPreferences {
        dietary_restrictions: vec![],
        max_prep_time_weeknight: 30,
        max_prep_time_weekend: 90,
        avoid_consecutive_complex: true,
        cuisine_variety_weight: 0.7,
    };

    // Generate multi-week plan
    println!("Generating multi-week meal plan...");
    let cmd = meal_planning::GenerateMultiWeekMealPlanCommand {
        user_id: user_id.to_string(),
        week_count: 5,
    };

    meal_planning::generate_multi_week_meal_plan(
        cmd,
        &executor,
        recipes,
        prefs,
    )
    .await?;

    println!("✅ Multi-week meal plan generated successfully with 5 weeks!");

    Ok(())
}

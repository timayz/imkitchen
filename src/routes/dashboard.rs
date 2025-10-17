use askama::Template;
use axum::{extract::State, response::Html, Extension};
use meal_planning::read_model::{MealAssignmentWithRecipe, MealPlanQueries};
use recipe::read_model::query_recipe_count;
use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::middleware::auth::Auth;
use crate::routes::AppState;

/// Today's meal slot data for template rendering (Story 3.9)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodayMealSlotData {
    pub assignment_id: String,
    pub recipe_id: String,
    pub recipe_title: String,
    pub prep_time_min: Option<i32>,
    pub cook_time_min: Option<i32>,
    pub total_time_min: u32,
    pub advance_prep_required: bool,
    pub complexity: Option<String>,
    pub assignment_reasoning: Option<String>,
}

/// Today's meals data for dashboard template (Story 3.9)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodaysMealsData {
    pub breakfast: Option<TodayMealSlotData>,
    pub lunch: Option<TodayMealSlotData>,
    pub dinner: Option<TodayMealSlotData>,
    pub has_meal_plan: bool,
}

#[derive(Template)]
#[template(path = "pages/dashboard.html")]
pub struct DashboardTemplate {
    pub user: Option<()>,
    pub todays_meals: Option<TodaysMealsData>,
    pub has_meal_plan: bool,
    pub recipe_count: usize,
    pub favorite_count: usize,
}

/// GET /dashboard - Display dashboard with today's meals (Story 3.9)
///
/// AC-1: Home dashboard prominently displays "Today's Meals" section at top
/// AC-2: Shows breakfast, lunch, and dinner assigned for today
/// AC-3: Each meal displays: recipe title, image placeholder, prep time
/// AC-4: Advance prep indicator if preparation required today for future meal
/// AC-5: "View Full Calendar" link to navigate to week view
/// AC-6: If no meal plan active, displays "Generate Meal Plan" call-to-action
/// AC-7: Today's meals update automatically at midnight (new day)
/// AC-8: Click recipe navigates to full recipe detail
pub async fn dashboard_handler(
    Extension(auth): Extension<Auth>,
    State(state): State<AppState>,
) -> Result<Html<String>, AppError> {
    // Query today's meals with recipe details
    let todays_meal_assignments =
        MealPlanQueries::get_todays_meals(&auth.user_id, &state.db_pool).await?;

    // Query recipe stats for dashboard cards
    let (recipe_count, favorite_count) = query_recipe_count(&auth.user_id, &state.db_pool).await?;

    // Map assignments to TodaysMealsData structure
    let todays_meals = if todays_meal_assignments.is_empty() {
        // No meal plan - template will show CTA
        None
    } else {
        Some(map_to_todays_meals(&todays_meal_assignments))
    };

    let has_meal_plan = todays_meals.is_some();

    let template = DashboardTemplate {
        user: Some(()),
        todays_meals,
        has_meal_plan,
        recipe_count,
        favorite_count,
    };

    template.render().map(Html).map_err(|e| {
        tracing::error!("Failed to render dashboard template: {:?}", e);
        AppError::InternalError("Failed to render page".to_string())
    })
}

/// Helper: Map meal assignments to TodaysMealsData
pub fn map_to_todays_meals(assignments: &[MealAssignmentWithRecipe]) -> TodaysMealsData {
    let mut data = TodaysMealsData {
        breakfast: None,
        lunch: None,
        dinner: None,
        has_meal_plan: true,
    };

    for assignment in assignments {
        let total_time_min = assignment.prep_time_min.unwrap_or(0) as u32
            + assignment.cook_time_min.unwrap_or(0) as u32;

        let advance_prep_required = assignment
            .advance_prep_hours
            .map(|hours| hours > 0)
            .unwrap_or(false);

        let slot_data = TodayMealSlotData {
            assignment_id: assignment.id.clone(),
            recipe_id: assignment.recipe_id.clone(),
            recipe_title: assignment.recipe_title.clone(),
            prep_time_min: assignment.prep_time_min,
            cook_time_min: assignment.cook_time_min,
            total_time_min,
            advance_prep_required,
            complexity: assignment.complexity.clone(),
            assignment_reasoning: assignment.assignment_reasoning.clone(),
        };

        match assignment.meal_type.as_str() {
            "breakfast" => data.breakfast = Some(slot_data),
            "lunch" => data.lunch = Some(slot_data),
            "dinner" => data.dinner = Some(slot_data),
            _ => {}
        }
    }

    data
}

#[cfg(test)]
mod tests {
    use super::*;
    use meal_planning::read_model::MealAssignmentWithRecipe;

    /// Test: map_to_todays_meals() correctly organizes meals by type
    #[test]
    fn test_map_to_todays_meals_organizes_by_meal_type() {
        let assignments = vec![
            MealAssignmentWithRecipe {
                id: "assignment_breakfast".to_string(),
                meal_plan_id: "plan1".to_string(),
                date: "2025-01-15".to_string(),
                meal_type: "breakfast".to_string(),
                recipe_id: "recipe1".to_string(),
                prep_required: false,
                assignment_reasoning: None,
                recipe_title: "Pancakes".to_string(),
                prep_time_min: Some(10),
                cook_time_min: Some(15),
                advance_prep_hours: None,
                complexity: Some("simple".to_string()),
            },
            MealAssignmentWithRecipe {
                id: "assignment_lunch".to_string(),
                meal_plan_id: "plan1".to_string(),
                date: "2025-01-15".to_string(),
                meal_type: "lunch".to_string(),
                recipe_id: "recipe2".to_string(),
                prep_required: true,
                assignment_reasoning: Some("Marinated chicken".to_string()),
                recipe_title: "Chicken Salad".to_string(),
                prep_time_min: Some(20),
                cook_time_min: Some(0),
                advance_prep_hours: Some(4),
                complexity: Some("moderate".to_string()),
            },
            MealAssignmentWithRecipe {
                id: "assignment_dinner".to_string(),
                meal_plan_id: "plan1".to_string(),
                date: "2025-01-15".to_string(),
                meal_type: "dinner".to_string(),
                recipe_id: "recipe3".to_string(),
                prep_required: false,
                assignment_reasoning: None,
                recipe_title: "Pasta Carbonara".to_string(),
                prep_time_min: Some(15),
                cook_time_min: Some(20),
                advance_prep_hours: None,
                complexity: Some("moderate".to_string()),
            },
        ];

        let result = map_to_todays_meals(&assignments);

        assert!(result.has_meal_plan);
        assert!(result.breakfast.is_some());
        assert!(result.lunch.is_some());
        assert!(result.dinner.is_some());

        // Verify breakfast
        let breakfast = result.breakfast.unwrap();
        assert_eq!(breakfast.recipe_title, "Pancakes");
        assert_eq!(breakfast.total_time_min, 25);
        assert!(!breakfast.advance_prep_required);

        // Verify lunch
        let lunch = result.lunch.unwrap();
        assert_eq!(lunch.recipe_title, "Chicken Salad");
        assert_eq!(lunch.total_time_min, 20);
        assert!(lunch.advance_prep_required);

        // Verify dinner
        let dinner = result.dinner.unwrap();
        assert_eq!(dinner.recipe_title, "Pasta Carbonara");
        assert_eq!(dinner.total_time_min, 35);
        assert!(!dinner.advance_prep_required);
    }

    /// Test: map_to_todays_meals() handles missing meal slots
    #[test]
    fn test_map_to_todays_meals_handles_missing_slots() {
        let assignments = vec![MealAssignmentWithRecipe {
            id: "assignment_breakfast".to_string(),
            meal_plan_id: "plan1".to_string(),
            date: "2025-01-15".to_string(),
            meal_type: "breakfast".to_string(),
            recipe_id: "recipe1".to_string(),
            prep_required: false,
            assignment_reasoning: None,
            recipe_title: "Pancakes".to_string(),
            prep_time_min: Some(10),
            cook_time_min: Some(15),
            advance_prep_hours: None,
            complexity: Some("simple".to_string()),
        }];

        let result = map_to_todays_meals(&assignments);

        assert!(result.has_meal_plan);
        assert!(result.breakfast.is_some());
        assert!(result.lunch.is_none());
        assert!(result.dinner.is_none());
    }

    /// Test: map_to_todays_meals() handles zero times gracefully
    #[test]
    fn test_map_to_todays_meals_handles_zero_times() {
        let assignments = vec![MealAssignmentWithRecipe {
            id: "assignment_breakfast".to_string(),
            meal_plan_id: "plan1".to_string(),
            date: "2025-01-15".to_string(),
            meal_type: "breakfast".to_string(),
            recipe_id: "recipe1".to_string(),
            prep_required: false,
            assignment_reasoning: None,
            recipe_title: "Cereal".to_string(),
            prep_time_min: None,
            cook_time_min: None,
            advance_prep_hours: None,
            complexity: None,
        }];

        let result = map_to_todays_meals(&assignments);

        let breakfast = result.breakfast.unwrap();
        assert_eq!(breakfast.total_time_min, 0);
        assert!(!breakfast.advance_prep_required);
    }
}

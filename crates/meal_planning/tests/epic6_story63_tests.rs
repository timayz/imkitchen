//! Story 6.3: Update MealPlan Domain Model - Unit Tests
//!
//! This test module validates all domain model changes for multi-week meal planning:
//! - WeekStatus enum serialization
//! - WeekMealPlan struct creation and serialization
//! - RotationState multi-week tracking fields and helper methods
//! - MultiWeekMealPlan struct serialization
//! - MealAssignment with accompaniment_recipe_id field
//! - Three new events: MultiWeekMealPlanGenerated, SingleWeekRegenerated, AllFutureWeeksRegenerated
//! - Aggregate event handlers for all three events
//! - Backwards compatibility with old events

use chrono::Utc;
use evento::{
    migrator::{Migrate, Plan},
    Sqlite,
};
use meal_planning::{
    AllFutureWeeksRegenerated, MealPlanAggregate, MultiWeekMealPlan, MultiWeekMealPlanGenerated,
    RotationState, SingleWeekRegenerated, WeekMealPlan, WeekMealPlanData, WeekStatus,
};
use sqlx::sqlite::SqlitePoolOptions;

/// Setup in-memory test executor for evento
async fn setup_test_executor() -> (Sqlite, sqlx::SqlitePool) {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(":memory:")
        .await
        .unwrap();

    // Run evento migrations for event store
    let mut conn = pool.acquire().await.unwrap();
    evento::sql_migrator::new_migrator::<sqlx::Sqlite>()
        .unwrap()
        .run(&mut conn, &Plan::apply_all())
        .await
        .unwrap();
    drop(conn);

    let executor: Sqlite = pool.clone().into();
    (executor, pool)
}

// ============================================================================
// AC-3: WeekStatus Enum Tests
// ============================================================================

#[test]
fn test_week_status_serialization_round_trip() {
    // Test serde JSON serialization for all variants
    let statuses = vec![
        WeekStatus::Future,
        WeekStatus::Current,
        WeekStatus::Past,
        WeekStatus::Archived,
    ];

    for status in statuses {
        let json = serde_json::to_string(&status).unwrap();
        let deserialized: WeekStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(status, deserialized);
    }

    // Verify JSON format (snake_case)
    assert_eq!(
        serde_json::to_string(&WeekStatus::Future).unwrap(),
        "\"future\""
    );
    assert_eq!(
        serde_json::to_string(&WeekStatus::Current).unwrap(),
        "\"current\""
    );
    assert_eq!(
        serde_json::to_string(&WeekStatus::Past).unwrap(),
        "\"past\""
    );
    assert_eq!(
        serde_json::to_string(&WeekStatus::Archived).unwrap(),
        "\"archived\""
    );
}

#[test]
fn test_week_status_bincode_serialization() {
    // Test bincode serialization for evento event storage
    let statuses = vec![
        WeekStatus::Future,
        WeekStatus::Current,
        WeekStatus::Past,
        WeekStatus::Archived,
    ];

    for status in statuses {
        let encoded = bincode::encode_to_vec(status, bincode::config::standard()).unwrap();
        let (decoded, _): (WeekStatus, usize) =
            bincode::decode_from_slice(&encoded, bincode::config::standard()).unwrap();
        assert_eq!(status, decoded);
    }
}

// ============================================================================
// AC-1: WeekMealPlan Struct Tests
// ============================================================================

#[test]
fn test_week_meal_plan_creation() {
    let week = WeekMealPlan {
        id: "week-1".to_string(),
        user_id: "user-1".to_string(),
        start_date: "2025-10-27".to_string(), // Monday
        end_date: "2025-11-02".to_string(),   // Sunday
        status: WeekStatus::Future,
        is_locked: false,
        generation_batch_id: "batch-123".to_string(),
        meal_assignments: vec![],
        shopping_list_id: "shopping-1".to_string(),
        created_at: Utc::now().to_rfc3339(),
    };

    // Verify all 10 fields exist
    assert_eq!(week.id, "week-1");
    assert_eq!(week.user_id, "user-1");
    assert_eq!(week.start_date, "2025-10-27");
    assert_eq!(week.end_date, "2025-11-02");
    assert_eq!(week.status, WeekStatus::Future);
    assert!(!week.is_locked);
    assert_eq!(week.generation_batch_id, "batch-123");
    assert_eq!(week.meal_assignments.len(), 0);
    assert_eq!(week.shopping_list_id, "shopping-1");
    assert!(!week.created_at.is_empty());
}

#[test]
fn test_week_meal_plan_serialization() {
    let week = WeekMealPlan {
        id: "week-1".to_string(),
        user_id: "user-1".to_string(),
        start_date: "2025-10-27".to_string(),
        end_date: "2025-11-02".to_string(),
        status: WeekStatus::Current,
        is_locked: true,
        generation_batch_id: "batch-123".to_string(),
        meal_assignments: vec![],
        shopping_list_id: "shopping-1".to_string(),
        created_at: "2025-10-25T12:00:00Z".to_string(),
    };

    // Test serde JSON serialization
    let json = serde_json::to_string(&week).unwrap();
    let deserialized: WeekMealPlan = serde_json::from_str(&json).unwrap();
    assert_eq!(week.id, deserialized.id);
    assert_eq!(week.status, deserialized.status);
    assert_eq!(week.is_locked, deserialized.is_locked);

    // Test bincode serialization (for evento events)
    let encoded = bincode::encode_to_vec(&week, bincode::config::standard()).unwrap();
    let (decoded, _): (WeekMealPlan, usize) =
        bincode::decode_from_slice(&encoded, bincode::config::standard()).unwrap();
    assert_eq!(week.id, decoded.id);
    assert_eq!(week.status, decoded.status);
}

// ============================================================================
// AC-4: RotationState Multi-Week Tracking Tests
// ============================================================================

#[test]
fn test_rotation_state_new_constructor() {
    let state = RotationState::new();

    // Verify Epic 6 fields initialized to empty
    assert_eq!(state.used_main_course_ids.len(), 0);
    assert_eq!(state.used_appetizer_ids.len(), 0);
    assert_eq!(state.used_dessert_ids.len(), 0);
    assert_eq!(state.cuisine_usage_count.len(), 0);
    assert_eq!(state.last_complex_meal_date, None);
}

#[test]
fn test_rotation_state_mark_used_main_course() {
    let mut state = RotationState::new();

    // Mark main courses as used
    state.mark_used_main_course("main-1");
    state.mark_used_main_course("main-2");

    // Verify uniqueness - marking same recipe twice doesn't duplicate
    state.mark_used_main_course("main-1");

    assert_eq!(state.used_main_course_ids.len(), 2);
    assert!(state.used_main_course_ids.contains(&"main-1".to_string()));
    assert!(state.used_main_course_ids.contains(&"main-2".to_string()));
}

#[test]
fn test_rotation_state_is_main_course_used() {
    let mut state = RotationState::new();

    assert!(!state.is_main_course_used("main-1"));

    state.mark_used_main_course("main-1");

    assert!(state.is_main_course_used("main-1"));
    assert!(!state.is_main_course_used("main-2"));
}

#[test]
fn test_rotation_state_mark_used_appetizer_and_dessert() {
    let mut state = RotationState::new();

    // Appetizers and desserts CAN repeat (no uniqueness check)
    state.mark_used_appetizer("app-1");
    state.mark_used_appetizer("app-1"); // Allowed to repeat

    state.mark_used_dessert("dessert-1");
    state.mark_used_dessert("dessert-1"); // Allowed to repeat

    assert_eq!(state.used_appetizer_ids.len(), 2); // Both occurrences tracked
    assert_eq!(state.used_dessert_ids.len(), 2);
}

#[test]
fn test_rotation_state_cuisine_usage_tracking() {
    use recipe::Cuisine;

    let mut state = RotationState::new();

    // Track cuisine usage
    state.increment_cuisine_usage(&Cuisine::Italian);
    state.increment_cuisine_usage(&Cuisine::Italian);
    state.increment_cuisine_usage(&Cuisine::Indian);

    assert_eq!(state.get_cuisine_usage(&Cuisine::Italian), 2);
    assert_eq!(state.get_cuisine_usage(&Cuisine::Indian), 1);
    assert_eq!(state.get_cuisine_usage(&Cuisine::Mexican), 0); // Never used
}

#[test]
fn test_rotation_state_last_complex_meal_date() {
    let mut state = RotationState::new();

    assert_eq!(state.last_complex_meal_date, None);

    state.update_last_complex_meal_date("2025-10-27");

    assert_eq!(state.last_complex_meal_date, Some("2025-10-27".to_string()));
}

#[test]
fn test_rotation_state_serialization_with_epic6_fields() {
    use recipe::Cuisine;

    let mut state = RotationState::new();
    state.mark_used_main_course("main-1");
    state.mark_used_appetizer("app-1");
    state.mark_used_dessert("dessert-1");
    state.increment_cuisine_usage(&Cuisine::Italian);
    state.update_last_complex_meal_date("2025-10-27");

    // Test JSON serialization
    let json = state.to_json().unwrap();
    let deserialized = RotationState::from_json(&json).unwrap();

    assert_eq!(deserialized.used_main_course_ids.len(), 1);
    assert_eq!(deserialized.used_appetizer_ids.len(), 1);
    assert_eq!(deserialized.used_dessert_ids.len(), 1);
    assert_eq!(deserialized.get_cuisine_usage(&Cuisine::Italian), 1);
    assert_eq!(
        deserialized.last_complex_meal_date,
        Some("2025-10-27".to_string())
    );
}

// ============================================================================
// AC-2: MultiWeekMealPlan Struct Tests
// ============================================================================

#[test]
fn test_multi_week_meal_plan_serialization() {
    let week1 = WeekMealPlan {
        id: "week-1".to_string(),
        user_id: "user-1".to_string(),
        start_date: "2025-10-27".to_string(),
        end_date: "2025-11-02".to_string(),
        status: WeekStatus::Current,
        is_locked: true,
        generation_batch_id: "batch-123".to_string(),
        meal_assignments: vec![],
        shopping_list_id: "shopping-1".to_string(),
        created_at: "2025-10-25T12:00:00Z".to_string(),
    };

    let week2 = WeekMealPlan {
        id: "week-2".to_string(),
        user_id: "user-1".to_string(),
        start_date: "2025-11-03".to_string(),
        end_date: "2025-11-09".to_string(),
        status: WeekStatus::Future,
        is_locked: false,
        generation_batch_id: "batch-123".to_string(),
        meal_assignments: vec![],
        shopping_list_id: "shopping-2".to_string(),
        created_at: "2025-10-25T12:00:00Z".to_string(),
    };

    let multi_week = MultiWeekMealPlan {
        user_id: "user-1".to_string(),
        generation_batch_id: "batch-123".to_string(),
        generated_weeks: vec![week1, week2],
        rotation_state: RotationState::new(),
    };

    // Test serde JSON serialization
    let json = serde_json::to_string(&multi_week).unwrap();
    let deserialized: MultiWeekMealPlan = serde_json::from_str(&json).unwrap();

    assert_eq!(multi_week.user_id, deserialized.user_id);
    assert_eq!(
        multi_week.generated_weeks.len(),
        deserialized.generated_weeks.len()
    );
    assert_eq!(deserialized.generated_weeks.len(), 2);

    // Test bincode serialization
    let encoded = bincode::encode_to_vec(&multi_week, bincode::config::standard()).unwrap();
    let (decoded, _): (MultiWeekMealPlan, usize) =
        bincode::decode_from_slice(&encoded, bincode::config::standard()).unwrap();
    assert_eq!(multi_week.user_id, decoded.user_id);
    assert_eq!(decoded.generated_weeks.len(), 2);
}

// ============================================================================
// AC-8: MealAssignment with accompaniment_recipe_id
// ============================================================================

#[test]
fn test_meal_assignment_with_accompaniment_id() {
    use meal_planning::events::MealAssignment;

    let assignment = MealAssignment {
        date: "2025-10-27".to_string(),
        course_type: "main_course".to_string(),
        recipe_id: "tikka-masala".to_string(),
        prep_required: false,
        assignment_reasoning: Some("Weekend: more prep time".to_string()),
        accompaniment_recipe_id: Some("basmati-rice".to_string()),
    };

    assert_eq!(
        assignment.accompaniment_recipe_id,
        Some("basmati-rice".to_string())
    );

    // Test serde serialization
    let json = serde_json::to_string(&assignment).unwrap();
    let deserialized: MealAssignment = serde_json::from_str(&json).unwrap();
    assert_eq!(
        deserialized.accompaniment_recipe_id,
        Some("basmati-rice".to_string())
    );
}

#[test]
fn test_meal_assignment_without_accompaniment_id() {
    use meal_planning::events::MealAssignment;

    let assignment = MealAssignment {
        date: "2025-10-27".to_string(),
        course_type: "appetizer".to_string(),
        recipe_id: "soup".to_string(),
        prep_required: false,
        assignment_reasoning: None,
        accompaniment_recipe_id: None,
    };

    assert_eq!(assignment.accompaniment_recipe_id, None);
}

#[test]
fn test_meal_assignment_backwards_compatibility() {
    use meal_planning::events::MealAssignment;

    // Old MealAssignment JSON (without accompaniment_recipe_id field)
    let old_json = r#"{
        "date": "2025-10-27",
        "course_type": "main_course",
        "recipe_id": "pasta",
        "prep_required": false,
        "assignment_reasoning": "Quick weeknight meal"
    }"#;

    // Should deserialize with #[serde(default)] - accompaniment_recipe_id becomes None
    let assignment: MealAssignment = serde_json::from_str(old_json).unwrap();
    assert_eq!(assignment.accompaniment_recipe_id, None);
    assert_eq!(assignment.recipe_id, "pasta");
}

// ============================================================================
// AC-5, AC-6, AC-7: Event Serialization Tests
// ============================================================================

#[test]
fn test_multi_week_meal_plan_generated_event_serialization() {
    let week_data = WeekMealPlanData {
        id: "week-1".to_string(),
        start_date: "2025-10-27".to_string(),
        end_date: "2025-11-02".to_string(),
        status: WeekStatus::Future,
        is_locked: false,
        meal_assignments: vec![],
        shopping_list_id: "shopping-1".to_string(),
    };

    let event = MultiWeekMealPlanGenerated {
        generation_batch_id: "batch-123".to_string(),
        user_id: "user-1".to_string(),
        weeks: vec![week_data],
        rotation_state: RotationState::new(),
        generated_at: "2025-10-25T12:00:00Z".to_string(),
    };

    // Test bincode serialization (evento events use bincode)
    let encoded = bincode::encode_to_vec(&event, bincode::config::standard()).unwrap();
    let (decoded, _): (MultiWeekMealPlanGenerated, usize) =
        bincode::decode_from_slice(&encoded, bincode::config::standard()).unwrap();

    assert_eq!(event.generation_batch_id, decoded.generation_batch_id);
    assert_eq!(event.user_id, decoded.user_id);
    assert_eq!(event.weeks.len(), decoded.weeks.len());
}

#[test]
fn test_single_week_regenerated_event_serialization() {
    use meal_planning::events::MealAssignment;

    let assignment = MealAssignment {
        date: "2025-10-27".to_string(),
        course_type: "main_course".to_string(),
        recipe_id: "pasta".to_string(),
        prep_required: false,
        assignment_reasoning: None,
        accompaniment_recipe_id: None,
    };

    let event = SingleWeekRegenerated {
        week_id: "week-1".to_string(),
        week_start_date: "2025-10-27".to_string(),
        meal_assignments: vec![assignment],
        updated_rotation_state: RotationState::new(),
        regenerated_at: "2025-10-25T12:00:00Z".to_string(),
    };

    // Test bincode serialization
    let encoded = bincode::encode_to_vec(&event, bincode::config::standard()).unwrap();
    let (decoded, _): (SingleWeekRegenerated, usize) =
        bincode::decode_from_slice(&encoded, bincode::config::standard()).unwrap();

    assert_eq!(event.week_id, decoded.week_id);
    assert_eq!(event.meal_assignments.len(), decoded.meal_assignments.len());
}

#[test]
fn test_all_future_weeks_regenerated_event_serialization() {
    let week_data = WeekMealPlanData {
        id: "week-2".to_string(),
        start_date: "2025-11-03".to_string(),
        end_date: "2025-11-09".to_string(),
        status: WeekStatus::Future,
        is_locked: false,
        meal_assignments: vec![],
        shopping_list_id: "shopping-2".to_string(),
    };

    let event = AllFutureWeeksRegenerated {
        generation_batch_id: "batch-456".to_string(),
        user_id: "user-1".to_string(),
        weeks: vec![week_data],
        preserved_current_week_id: Some("week-1".to_string()),
        regenerated_at: "2025-10-25T12:00:00Z".to_string(),
    };

    // Test bincode serialization
    let encoded = bincode::encode_to_vec(&event, bincode::config::standard()).unwrap();
    let (decoded, _): (AllFutureWeeksRegenerated, usize) =
        bincode::decode_from_slice(&encoded, bincode::config::standard()).unwrap();

    assert_eq!(event.generation_batch_id, decoded.generation_batch_id);
    assert_eq!(
        event.preserved_current_week_id,
        decoded.preserved_current_week_id
    );
}

// ============================================================================
// AC-5, AC-6, AC-7: Aggregate Event Handler Tests
// ============================================================================

#[tokio::test]
async fn test_aggregate_handles_multi_week_meal_plan_generated() {
    let (executor, _pool) = setup_test_executor().await;

    let week_data = WeekMealPlanData {
        id: "week-1".to_string(),
        start_date: "2025-10-27".to_string(),
        end_date: "2025-11-02".to_string(),
        status: WeekStatus::Future,
        is_locked: false,
        meal_assignments: vec![],
        shopping_list_id: "shopping-1".to_string(),
    };

    let event_data = MultiWeekMealPlanGenerated {
        generation_batch_id: "batch-123".to_string(),
        user_id: "user-1".to_string(),
        weeks: vec![week_data],
        rotation_state: RotationState::new(),
        generated_at: Utc::now().to_rfc3339(),
    };

    // Emit event
    let meal_plan_id = evento::create::<MealPlanAggregate>()
        .data(&event_data)
        .unwrap()
        .metadata(&true)
        .unwrap()
        .commit(&executor)
        .await
        .unwrap();

    // Load aggregate and verify state
    let loaded = evento::load::<MealPlanAggregate, _>(&executor, &meal_plan_id)
        .await
        .unwrap();
    let aggregate = loaded.item;

    assert_eq!(aggregate.user_id, "user-1");
    assert_eq!(aggregate.start_date, "2025-10-27"); // First week's start_date
    assert_eq!(aggregate.status, "active");
}

#[tokio::test]
async fn test_aggregate_handles_single_week_regenerated() {
    use meal_planning::events::MealAssignment;

    let (executor, _pool) = setup_test_executor().await;

    // First, create initial multi-week meal plan
    let week_data = WeekMealPlanData {
        id: "week-1".to_string(),
        start_date: "2025-10-27".to_string(),
        end_date: "2025-11-02".to_string(),
        status: WeekStatus::Future,
        is_locked: false,
        meal_assignments: vec![],
        shopping_list_id: "shopping-1".to_string(),
    };

    let initial_event = MultiWeekMealPlanGenerated {
        generation_batch_id: "batch-123".to_string(),
        user_id: "user-1".to_string(),
        weeks: vec![week_data],
        rotation_state: RotationState::new(),
        generated_at: Utc::now().to_rfc3339(),
    };

    let meal_plan_id = evento::create::<MealPlanAggregate>()
        .data(&initial_event)
        .unwrap()
        .metadata(&true)
        .unwrap()
        .commit(&executor)
        .await
        .unwrap();

    // Now regenerate the first week
    let new_assignment = MealAssignment {
        date: "2025-10-27".to_string(),
        course_type: "main_course".to_string(),
        recipe_id: "new-recipe".to_string(),
        prep_required: false,
        assignment_reasoning: None,
        accompaniment_recipe_id: None,
    };

    let regen_event = SingleWeekRegenerated {
        week_id: "week-1".to_string(),
        week_start_date: "2025-10-27".to_string(),
        meal_assignments: vec![new_assignment],
        updated_rotation_state: RotationState::new(),
        regenerated_at: Utc::now().to_rfc3339(),
    };

    // Emit regeneration event
    evento::save::<MealPlanAggregate>(&meal_plan_id)
        .data(&regen_event)
        .unwrap()
        .metadata(&true)
        .unwrap()
        .commit(&executor)
        .await
        .unwrap();

    // Load aggregate and verify meal assignments updated
    let loaded = evento::load::<MealPlanAggregate, _>(&executor, &meal_plan_id)
        .await
        .unwrap();
    let aggregate = loaded.item;

    assert_eq!(aggregate.meal_assignments.len(), 1);
    assert_eq!(aggregate.meal_assignments[0].recipe_id, "new-recipe");
}

#[tokio::test]
async fn test_aggregate_handles_all_future_weeks_regenerated() {
    let (executor, _pool) = setup_test_executor().await;

    // Create initial multi-week plan
    let week_data = WeekMealPlanData {
        id: "week-1".to_string(),
        start_date: "2025-10-27".to_string(),
        end_date: "2025-11-02".to_string(),
        status: WeekStatus::Current,
        is_locked: true,
        meal_assignments: vec![],
        shopping_list_id: "shopping-1".to_string(),
    };

    let initial_event = MultiWeekMealPlanGenerated {
        generation_batch_id: "batch-123".to_string(),
        user_id: "user-1".to_string(),
        weeks: vec![week_data],
        rotation_state: RotationState::new(),
        generated_at: Utc::now().to_rfc3339(),
    };

    let meal_plan_id = evento::create::<MealPlanAggregate>()
        .data(&initial_event)
        .unwrap()
        .metadata(&true)
        .unwrap()
        .commit(&executor)
        .await
        .unwrap();

    // Regenerate all future weeks (current week preserved)
    let future_week_data = WeekMealPlanData {
        id: "week-2".to_string(),
        start_date: "2025-11-03".to_string(),
        end_date: "2025-11-09".to_string(),
        status: WeekStatus::Future,
        is_locked: false,
        meal_assignments: vec![],
        shopping_list_id: "shopping-2".to_string(),
    };

    let regen_event = AllFutureWeeksRegenerated {
        generation_batch_id: "batch-456".to_string(),
        user_id: "user-1".to_string(),
        weeks: vec![future_week_data],
        preserved_current_week_id: Some("week-1".to_string()),
        regenerated_at: Utc::now().to_rfc3339(),
    };

    // Emit regeneration event
    evento::save::<MealPlanAggregate>(&meal_plan_id)
        .data(&regen_event)
        .unwrap()
        .metadata(&true)
        .unwrap()
        .commit(&executor)
        .await
        .unwrap();

    // Load aggregate and verify first future week data updated
    let loaded = evento::load::<MealPlanAggregate, _>(&executor, &meal_plan_id)
        .await
        .unwrap();
    let aggregate = loaded.item;

    // Aggregate root should now show first future week
    assert_eq!(aggregate.start_date, "2025-11-03");
}

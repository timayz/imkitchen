use bincode::{Decode, Encode};
use meal_planning::events::{MealAssignment, MealPlanGenerated};

#[test]
fn test_meal_plan_generated_bincode() {
    let assignment = MealAssignment {
        date: "2025-10-17".to_string(),
        meal_type: "breakfast".to_string(),
        recipe_id: "recipe-123".to_string(),
        prep_required: false,
    };
    
    let event = MealPlanGenerated {
        user_id: "user-123".to_string(),
        start_date: "2025-10-20".to_string(),
        meal_assignments: vec![assignment],
        rotation_state_json: "{}".to_string(),
        generated_at: "2025-10-17T00:00:00Z".to_string(),
    };
    
    // Test encode with bincode::config::standard() (what evento uses)
    let encoded = bincode::encode_to_vec(&event, bincode::config::standard())
        .expect("Failed to encode MealPlanGenerated");
    
    println!("Encoded {} bytes", encoded.len());
    
    // Test decode
    let (decoded, _): (MealPlanGenerated, _) = bincode::decode_from_slice(&encoded, bincode::config::standard())
        .expect("Failed to decode MealPlanGenerated");
    
    assert_eq!(decoded.user_id, "user-123");
    assert_eq!(decoded.meal_assignments.len(), 1);
}

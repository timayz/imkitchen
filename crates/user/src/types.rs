/// Domain types for user crate
///
/// This module contains shared types used across the user domain
/// including subscription tiers, user identifiers, and other domain-specific types.
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Subscription tier enum with type-safe tier values
///
/// This enum ensures tier values are restricted to "free" or "premium"
/// preventing typos and invalid tier assignments.
///
/// ## Usage
/// ```
/// use user::types::SubscriptionTier;
///
/// let tier = SubscriptionTier::Free;
/// assert_eq!(tier.as_str(), "free");
///
/// let tier: SubscriptionTier = "premium".parse().unwrap();
/// assert_eq!(tier, SubscriptionTier::Premium);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Encode, Decode, Default)]
#[serde(rename_all = "lowercase")]
pub enum SubscriptionTier {
    #[default]
    Free,
    Premium,
}

impl SubscriptionTier {
    /// Convert tier to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            SubscriptionTier::Free => "free",
            SubscriptionTier::Premium => "premium",
        }
    }

    /// Check if tier is premium
    pub fn is_premium(&self) -> bool {
        matches!(self, SubscriptionTier::Premium)
    }

    /// Check if tier is free
    pub fn is_free(&self) -> bool {
        matches!(self, SubscriptionTier::Free)
    }
}

impl fmt::Display for SubscriptionTier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for SubscriptionTier {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "free" => Ok(SubscriptionTier::Free),
            "premium" => Ok(SubscriptionTier::Premium),
            _ => Err(format!(
                "Invalid subscription tier: {}. Must be 'free' or 'premium'",
                s
            )),
        }
    }
}

impl From<SubscriptionTier> for String {
    fn from(tier: SubscriptionTier) -> Self {
        tier.as_str().to_string()
    }
}

impl From<&str> for SubscriptionTier {
    fn from(s: &str) -> Self {
        s.parse().unwrap_or_default()
    }
}

/// Dietary restriction enum for meal planning preferences
///
/// Represents common dietary restrictions and allows custom user-defined restrictions.
/// Used in UserPreferences to filter recipe selection in meal planning algorithm.
///
/// ## Variants
/// - Standard restrictions: Vegetarian, Vegan, GlutenFree, DairyFree, NutFree, Halal, Kosher
/// - Custom: User-defined restriction (e.g., "shellfish", "soy")
///
/// ## Design Note
/// DietaryRestriction is distinct from DietaryTag (on recipes):
/// - Restrictions are stored on User (constraints)
/// - Tags are stored on Recipe (characteristics)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
#[serde(tag = "type", content = "value")]
pub enum DietaryRestriction {
    Vegetarian,
    Vegan,
    GlutenFree,
    DairyFree,
    NutFree,
    Halal,
    Kosher,
    Custom(String),
}

/// Skill level enum for user cooking proficiency
///
/// Affects complexity filtering in meal planning:
/// - Beginner: Only simple recipes
/// - Intermediate: Simple + moderate recipes
/// - Advanced: All complexity levels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
pub enum SkillLevel {
    Beginner,
    Intermediate,
    Advanced,
}

/// Time range for weeknight availability
///
/// Represents when a user is typically available to cook on weeknights.
/// Used in meal planning algorithm to schedule meals within user's time constraints.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
pub struct TimeRange {
    /// Start time in 24-hour format (e.g., "18:00")
    pub start: String,
    /// Duration in minutes
    pub duration_minutes: u32,
}

/// User preferences for meal planning personalization
///
/// Contains all user constraints and preferences used by the meal planning algorithm
/// to generate personalized meal plans.
///
/// ## Default Values
/// Per architecture design decisions (docs/architecture-update-meal-planning-enhancements.md#3.4):
/// - max_prep_time_weeknight: 30 minutes (typical weeknight cooking window)
/// - max_prep_time_weekend: 90 minutes (more flexible weekend schedule)
/// - cuisine_variety_weight: 0.7 (70% variety preference, balanced)
/// - avoid_consecutive_complex: true (prevent burnout)
///
/// ## Design Notes
/// - Cuisine preferences are INFERRED from favorite recipe selection (NOT stored)
/// - Advance prep timing is stored on Recipe aggregate, NOT User
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct UserPreferences {
    /// User ID this preferences belong to
    pub user_id: String,
    /// Dietary restrictions (e.g., Vegetarian, Custom("shellfish"))
    pub dietary_restrictions: Vec<DietaryRestriction>,
    /// Number of people in household (affects serving size calculations)
    pub household_size: u32,
    /// User's cooking skill level (affects complexity filtering)
    pub skill_level: SkillLevel,
    /// When user is typically available to cook on weeknights
    pub weeknight_availability: TimeRange,
    /// Maximum prep time allowed for weeknight meals (minutes, default: 30)
    pub max_prep_time_weeknight: u32,
    /// Maximum prep time allowed for weekend meals (minutes, default: 90)
    pub max_prep_time_weekend: u32,
    /// Avoid scheduling complex meals back-to-back (default: true)
    pub avoid_consecutive_complex: bool,
    /// Cuisine variety weight: 0.0 (prefer favorites) to 1.0 (maximize variety), default: 0.7
    pub cuisine_variety_weight: f32,
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            user_id: String::new(),
            dietary_restrictions: Vec::new(),
            household_size: 2,
            skill_level: SkillLevel::Intermediate,
            weeknight_availability: TimeRange {
                start: "18:00".to_string(),
                duration_minutes: 45,
            },
            max_prep_time_weeknight: 30,
            max_prep_time_weekend: 90,
            avoid_consecutive_complex: true,
            cuisine_variety_weight: 0.7,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subscription_tier_as_str() {
        assert_eq!(SubscriptionTier::Free.as_str(), "free");
        assert_eq!(SubscriptionTier::Premium.as_str(), "premium");
    }

    #[test]
    fn test_subscription_tier_display() {
        assert_eq!(format!("{}", SubscriptionTier::Free), "free");
        assert_eq!(format!("{}", SubscriptionTier::Premium), "premium");
    }

    #[test]
    fn test_subscription_tier_from_str() {
        assert_eq!(
            "free".parse::<SubscriptionTier>().unwrap(),
            SubscriptionTier::Free
        );
        assert_eq!(
            "premium".parse::<SubscriptionTier>().unwrap(),
            SubscriptionTier::Premium
        );
        assert_eq!(
            "FREE".parse::<SubscriptionTier>().unwrap(),
            SubscriptionTier::Free
        ); // Case insensitive
        assert_eq!(
            "Premium".parse::<SubscriptionTier>().unwrap(),
            SubscriptionTier::Premium
        );
    }

    #[test]
    fn test_subscription_tier_from_str_invalid() {
        let result = "invalid".parse::<SubscriptionTier>();
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Invalid subscription tier: invalid. Must be 'free' or 'premium'"
        );
    }

    #[test]
    fn test_subscription_tier_default() {
        assert_eq!(SubscriptionTier::default(), SubscriptionTier::Free);
    }

    #[test]
    fn test_subscription_tier_is_premium() {
        assert!(!SubscriptionTier::Free.is_premium());
        assert!(SubscriptionTier::Premium.is_premium());
    }

    #[test]
    fn test_subscription_tier_is_free() {
        assert!(SubscriptionTier::Free.is_free());
        assert!(!SubscriptionTier::Premium.is_free());
    }

    #[test]
    fn test_subscription_tier_to_string() {
        let tier = SubscriptionTier::Premium;
        let tier_string: String = tier.into();
        assert_eq!(tier_string, "premium");
    }

    #[test]
    fn test_subscription_tier_serde() {
        // Test serialization
        let tier = SubscriptionTier::Premium;
        let json = serde_json::to_string(&tier).unwrap();
        assert_eq!(json, "\"premium\"");

        // Test deserialization
        let tier: SubscriptionTier = serde_json::from_str("\"free\"").unwrap();
        assert_eq!(tier, SubscriptionTier::Free);
    }

    #[test]
    fn test_dietary_restriction_serde() {
        // Test standard variant serialization
        let restriction = DietaryRestriction::Vegetarian;
        let json = serde_json::to_string(&restriction).unwrap();
        assert_eq!(json, r#"{"type":"Vegetarian"}"#);

        // Test custom variant serialization
        let restriction = DietaryRestriction::Custom("shellfish".to_string());
        let json = serde_json::to_string(&restriction).unwrap();
        assert_eq!(json, r#"{"type":"Custom","value":"shellfish"}"#);

        // Test deserialization
        let restriction: DietaryRestriction = serde_json::from_str(r#"{"type":"Vegan"}"#).unwrap();
        assert_eq!(restriction, DietaryRestriction::Vegan);

        let restriction: DietaryRestriction =
            serde_json::from_str(r#"{"type":"Custom","value":"soy"}"#).unwrap();
        assert_eq!(restriction, DietaryRestriction::Custom("soy".to_string()));
    }

    #[test]
    fn test_dietary_restriction_bincode() {
        // Test bincode round-trip for standard variant
        let restriction = DietaryRestriction::GlutenFree;
        let encoded = bincode::encode_to_vec(&restriction, bincode::config::standard()).unwrap();
        let (decoded, _): (DietaryRestriction, _) =
            bincode::decode_from_slice(&encoded, bincode::config::standard()).unwrap();
        assert_eq!(decoded, restriction);

        // Test bincode round-trip for custom variant
        let restriction = DietaryRestriction::Custom("peanuts".to_string());
        let encoded = bincode::encode_to_vec(&restriction, bincode::config::standard()).unwrap();
        let (decoded, _): (DietaryRestriction, _) =
            bincode::decode_from_slice(&encoded, bincode::config::standard()).unwrap();
        assert_eq!(decoded, restriction);
    }

    #[test]
    fn test_skill_level_serde() {
        // Test serialization
        let skill = SkillLevel::Intermediate;
        let json = serde_json::to_string(&skill).unwrap();
        assert_eq!(json, r#""Intermediate""#);

        // Test deserialization
        let skill: SkillLevel = serde_json::from_str(r#""Beginner""#).unwrap();
        assert_eq!(skill, SkillLevel::Beginner);

        let skill: SkillLevel = serde_json::from_str(r#""Advanced""#).unwrap();
        assert_eq!(skill, SkillLevel::Advanced);
    }

    #[test]
    fn test_skill_level_bincode() {
        // Test bincode round-trip
        let skill = SkillLevel::Advanced;
        let encoded = bincode::encode_to_vec(&skill, bincode::config::standard()).unwrap();
        let (decoded, _): (SkillLevel, _) =
            bincode::decode_from_slice(&encoded, bincode::config::standard()).unwrap();
        assert_eq!(decoded, skill);
    }

    #[test]
    fn test_user_preferences_default() {
        let prefs = UserPreferences::default();

        // AC #8: Test default values match specification
        assert_eq!(prefs.max_prep_time_weeknight, 30);
        assert_eq!(prefs.max_prep_time_weekend, 90);
        assert_eq!(prefs.cuisine_variety_weight, 0.7);
        assert!(prefs.avoid_consecutive_complex);

        // Test other defaults
        assert_eq!(prefs.household_size, 2);
        assert_eq!(prefs.skill_level, SkillLevel::Intermediate);
        assert_eq!(prefs.weeknight_availability.start, "18:00");
        assert_eq!(prefs.weeknight_availability.duration_minutes, 45);
        assert!(prefs.dietary_restrictions.is_empty());
    }

    #[test]
    fn test_user_preferences_serde() {
        let prefs = UserPreferences {
            user_id: "user123".to_string(),
            dietary_restrictions: vec![
                DietaryRestriction::Vegetarian,
                DietaryRestriction::Custom("shellfish".to_string()),
            ],
            household_size: 4,
            skill_level: SkillLevel::Advanced,
            weeknight_availability: TimeRange {
                start: "19:00".to_string(),
                duration_minutes: 60,
            },
            max_prep_time_weeknight: 45,
            max_prep_time_weekend: 120,
            avoid_consecutive_complex: false,
            cuisine_variety_weight: 0.5,
        };

        // Test JSON round-trip
        let json = serde_json::to_string(&prefs).unwrap();
        let decoded: UserPreferences = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.user_id, prefs.user_id);
        assert_eq!(decoded.household_size, 4);
        assert_eq!(decoded.max_prep_time_weeknight, 45);
    }

    #[test]
    fn test_user_preferences_bincode() {
        let prefs = UserPreferences::default();

        // Test bincode round-trip
        let encoded = bincode::encode_to_vec(&prefs, bincode::config::standard()).unwrap();
        let (decoded, _): (UserPreferences, _) =
            bincode::decode_from_slice(&encoded, bincode::config::standard()).unwrap();

        assert_eq!(
            decoded.max_prep_time_weeknight,
            prefs.max_prep_time_weeknight
        );
        assert_eq!(decoded.cuisine_variety_weight, prefs.cuisine_variety_weight);
    }
}

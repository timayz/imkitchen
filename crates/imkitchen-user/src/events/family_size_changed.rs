// FamilySizeChanged event for tracking family size modifications

use chrono::{DateTime, Utc};
use imkitchen_shared::FamilySize;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Event fired when a user changes their family size
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FamilySizeChanged {
    pub user_id: Uuid,
    pub previous_size: FamilySize,
    pub new_size: FamilySize,
    pub changed_at: DateTime<Utc>,
    pub reason: Option<String>, // Optional reason for the change
}

impl FamilySizeChanged {
    /// Create a new FamilySizeChanged event
    pub fn new(user_id: Uuid, previous_size: FamilySize, new_size: FamilySize) -> Self {
        Self {
            user_id,
            previous_size,
            new_size,
            changed_at: Utc::now(),
            reason: None,
        }
    }

    /// Create a new FamilySizeChanged event with a reason
    pub fn with_reason(
        user_id: Uuid,
        previous_size: FamilySize,
        new_size: FamilySize,
        reason: String,
    ) -> Self {
        Self {
            user_id,
            previous_size,
            new_size,
            changed_at: Utc::now(),
            reason: Some(reason),
        }
    }

    /// Get the size change (positive for increase, negative for decrease)
    pub fn size_change(&self) -> i8 {
        self.new_size.value as i8 - self.previous_size.value as i8
    }

    /// Check if the family size increased
    pub fn is_size_increase(&self) -> bool {
        self.new_size.value > self.previous_size.value
    }

    /// Check if the family size decreased
    pub fn is_size_decrease(&self) -> bool {
        self.new_size.value < self.previous_size.value
    }

    /// Check if any size change actually occurred
    pub fn has_changes(&self) -> bool {
        self.previous_size.value != self.new_size.value
    }

    /// Get the percentage change in family size
    pub fn percentage_change(&self) -> f64 {
        if self.previous_size.value == 0 {
            return 0.0;
        }

        let change = self.size_change() as f64;
        let previous = self.previous_size.value as f64;
        (change / previous) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use imkitchen_shared::FamilySize;

    #[test]
    fn test_family_size_changed_creation() {
        let user_id = Uuid::new_v4();
        let previous = FamilySize::new(2).unwrap();
        let new = FamilySize::new(4).unwrap();

        let event = FamilySizeChanged::new(user_id, previous, new);

        assert_eq!(event.user_id, user_id);
        assert_eq!(event.previous_size, previous);
        assert_eq!(event.new_size, new);
        assert!(event.has_changes());
        assert!(event.reason.is_none());
    }

    #[test]
    fn test_family_size_changed_with_reason() {
        let user_id = Uuid::new_v4();
        let previous = FamilySize::new(2).unwrap();
        let new = FamilySize::new(3).unwrap();
        let reason = "New baby born".to_string();

        let event = FamilySizeChanged::with_reason(user_id, previous, new, reason.clone());

        assert_eq!(event.user_id, user_id);
        assert_eq!(event.previous_size, previous);
        assert_eq!(event.new_size, new);
        assert_eq!(event.reason, Some(reason));
        assert!(event.has_changes());
    }

    #[test]
    fn test_size_change_calculations() {
        let user_id = Uuid::new_v4();

        // Test increase
        let increase_event = FamilySizeChanged::new(
            user_id,
            FamilySize::new(2).unwrap(),
            FamilySize::new(5).unwrap(),
        );
        assert_eq!(increase_event.size_change(), 3);
        assert!(increase_event.is_size_increase());
        assert!(!increase_event.is_size_decrease());

        // Test decrease
        let decrease_event = FamilySizeChanged::new(
            user_id,
            FamilySize::new(6).unwrap(),
            FamilySize::new(3).unwrap(),
        );
        assert_eq!(decrease_event.size_change(), -3);
        assert!(!decrease_event.is_size_increase());
        assert!(decrease_event.is_size_decrease());

        // Test no change
        let no_change_event = FamilySizeChanged::new(
            user_id,
            FamilySize::new(4).unwrap(),
            FamilySize::new(4).unwrap(),
        );
        assert_eq!(no_change_event.size_change(), 0);
        assert!(!no_change_event.is_size_increase());
        assert!(!no_change_event.is_size_decrease());
        assert!(!no_change_event.has_changes());
    }

    #[test]
    fn test_percentage_change() {
        let user_id = Uuid::new_v4();

        // Test 100% increase (double)
        let double_event = FamilySizeChanged::new(
            user_id,
            FamilySize::new(2).unwrap(),
            FamilySize::new(4).unwrap(),
        );
        assert_eq!(double_event.percentage_change(), 100.0);

        // Test 50% increase
        let increase_event = FamilySizeChanged::new(
            user_id,
            FamilySize::new(4).unwrap(),
            FamilySize::new(6).unwrap(),
        );
        assert_eq!(increase_event.percentage_change(), 50.0);

        // Test 25% decrease
        let decrease_event = FamilySizeChanged::new(
            user_id,
            FamilySize::new(4).unwrap(),
            FamilySize::new(3).unwrap(),
        );
        assert_eq!(decrease_event.percentage_change(), -25.0);
    }

    #[test]
    fn test_serialization() {
        let user_id = Uuid::new_v4();
        let previous = FamilySize::new(2).unwrap();
        let new = FamilySize::new(4).unwrap();

        let event =
            FamilySizeChanged::with_reason(user_id, previous, new, "Growing family".to_string());

        // Test serialization
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: FamilySizeChanged = serde_json::from_str(&json).unwrap();

        assert_eq!(event, deserialized);
    }
}

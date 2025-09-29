// DietaryRestrictionsChanged event for tracking dietary preference modifications

use chrono::{DateTime, Utc};
use imkitchen_shared::DietaryRestriction;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Event fired when a user changes their dietary restrictions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DietaryRestrictionsChanged {
    pub user_id: Uuid,
    pub previous_restrictions: Vec<DietaryRestriction>,
    pub new_restrictions: Vec<DietaryRestriction>,
    pub changed_at: DateTime<Utc>,
}

impl DietaryRestrictionsChanged {
    /// Create a new DietaryRestrictionsChanged event
    pub fn new(
        user_id: Uuid,
        previous_restrictions: Vec<DietaryRestriction>,
        new_restrictions: Vec<DietaryRestriction>,
    ) -> Self {
        Self {
            user_id,
            previous_restrictions,
            new_restrictions,
            changed_at: Utc::now(),
        }
    }

    /// Get the added restrictions (new restrictions not in previous)
    pub fn added_restrictions(&self) -> Vec<DietaryRestriction> {
        self.new_restrictions
            .iter()
            .filter(|r| !self.previous_restrictions.contains(r))
            .cloned()
            .collect()
    }

    /// Get the removed restrictions (previous restrictions not in new)
    pub fn removed_restrictions(&self) -> Vec<DietaryRestriction> {
        self.previous_restrictions
            .iter()
            .filter(|r| !self.new_restrictions.contains(r))
            .cloned()
            .collect()
    }

    /// Check if any restrictions were actually changed
    pub fn has_changes(&self) -> bool {
        self.previous_restrictions != self.new_restrictions
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use imkitchen_shared::DietaryRestriction;

    #[test]
    fn test_dietary_restrictions_changed_creation() {
        let user_id = Uuid::new_v4();
        let previous = vec![DietaryRestriction::Vegetarian];
        let new = vec![DietaryRestriction::Vegetarian, DietaryRestriction::GlutenFree];

        let event = DietaryRestrictionsChanged::new(user_id, previous.clone(), new.clone());

        assert_eq!(event.user_id, user_id);
        assert_eq!(event.previous_restrictions, previous);
        assert_eq!(event.new_restrictions, new);
        assert!(event.has_changes());
    }

    #[test]
    fn test_added_restrictions() {
        let user_id = Uuid::new_v4();
        let previous = vec![DietaryRestriction::Vegetarian];
        let new = vec![
            DietaryRestriction::Vegetarian,
            DietaryRestriction::GlutenFree,
            DietaryRestriction::DairyFree,
        ];

        let event = DietaryRestrictionsChanged::new(user_id, previous, new);
        let added = event.added_restrictions();

        assert_eq!(added.len(), 2);
        assert!(added.contains(&DietaryRestriction::GlutenFree));
        assert!(added.contains(&DietaryRestriction::DairyFree));
    }

    #[test]
    fn test_removed_restrictions() {
        let user_id = Uuid::new_v4();
        let previous = vec![
            DietaryRestriction::Vegetarian,
            DietaryRestriction::GlutenFree,
            DietaryRestriction::DairyFree,
        ];
        let new = vec![DietaryRestriction::Vegetarian];

        let event = DietaryRestrictionsChanged::new(user_id, previous, new);
        let removed = event.removed_restrictions();

        assert_eq!(removed.len(), 2);
        assert!(removed.contains(&DietaryRestriction::GlutenFree));
        assert!(removed.contains(&DietaryRestriction::DairyFree));
    }

    #[test]
    fn test_no_changes() {
        let user_id = Uuid::new_v4();
        let restrictions = vec![DietaryRestriction::Vegetarian, DietaryRestriction::GlutenFree];

        let event = DietaryRestrictionsChanged::new(user_id, restrictions.clone(), restrictions);

        assert!(!event.has_changes());
        assert!(event.added_restrictions().is_empty());
        assert!(event.removed_restrictions().is_empty());
    }

    #[test]
    fn test_serialization() {
        let user_id = Uuid::new_v4();
        let previous = vec![DietaryRestriction::Vegetarian];
        let new = vec![DietaryRestriction::Vegan];

        let event = DietaryRestrictionsChanged::new(user_id, previous, new);
        
        // Test serialization
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: DietaryRestrictionsChanged = serde_json::from_str(&json).unwrap();
        
        assert_eq!(event, deserialized);
    }
}
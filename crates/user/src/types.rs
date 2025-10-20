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
}

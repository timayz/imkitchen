//! Centralized access control service for freemium restrictions
//!
//! This module implements the access control logic for freemium features.
//! Priority order: global bypass > per-user bypass > premium subscription > free tier

use crate::queries::user::{count_user_favorites, get_user_profile};
use crate::Config;
use sqlx::SqlitePool;
use tracing::{debug, info};

/// Centralized access control service
///
/// Handles all freemium access control decisions with consistent priority:
/// 1. Global bypass (config) - for dev/staging environments
/// 2. Per-user bypass (database) - for demo/test accounts
/// 3. Premium subscription (database) - for paying users
/// 4. Free tier restrictions - default behavior
#[derive(Clone)]
pub struct AccessControlService {
    config: Config,
    pool: SqlitePool,
}

impl AccessControlService {
    /// Create a new AccessControlService
    pub fn new(config: Config, pool: SqlitePool) -> Self {
        Self { config, pool }
    }

    /// Check if user can view a specific week
    ///
    /// Free tier: Week 1 only
    /// Premium/Bypass: All weeks (1-5)
    pub async fn can_view_week(&self, user_id: &str, week_number: u8) -> anyhow::Result<bool> {
        // Check 1: Global bypass
        if self.config.access_control.global_premium_bypass {
            debug!(
                user_id,
                week_number, "Global bypass enabled - granting week access"
            );
            return Ok(true);
        }

        // Check 2: Load user profile
        let profile = get_user_profile(&self.pool, user_id).await?;

        // Check 3: Per-user bypass OR premium active OR week 1 (always free)
        let can_access = profile.premium_bypass || profile.is_premium_active || week_number == 1;

        info!(
            user_id,
            week_number,
            premium_bypass = profile.premium_bypass,
            is_premium = profile.is_premium_active,
            can_access,
            "Week access check"
        );

        Ok(can_access)
    }

    /// Check if user can add another favorite recipe
    ///
    /// Free tier: Maximum 10 favorites
    /// Premium/Bypass: Unlimited favorites
    pub async fn can_add_favorite(&self, user_id: &str) -> anyhow::Result<bool> {
        // Global bypass check
        if self.config.access_control.global_premium_bypass {
            debug!(user_id, "Global bypass enabled - granting favorite access");
            return Ok(true);
        }

        // Load profile
        let profile = get_user_profile(&self.pool, user_id).await?;

        // Per-user bypass OR premium - unlimited
        if profile.premium_bypass || profile.is_premium_active {
            info!(
                user_id,
                premium_bypass = profile.premium_bypass,
                is_premium = profile.is_premium_active,
                "Premium/bypass user - unlimited favorites"
            );
            return Ok(true);
        }

        // Free tier - check 10 favorite limit
        let count = count_user_favorites(&self.pool, user_id).await?;
        let can_add = count < 10;

        info!(
            user_id,
            current_count = count,
            limit = 10,
            can_add,
            "Free tier favorite limit check"
        );

        Ok(can_add)
    }

    /// Check if user can access shopping list for a specific week
    ///
    /// Free tier: Week 1 only
    /// Premium/Bypass: All weeks (1-5)
    pub async fn can_access_shopping_list(
        &self,
        user_id: &str,
        week_number: u8,
    ) -> anyhow::Result<bool> {
        // Shopping list access follows same rules as calendar week access
        self.can_view_week(user_id, week_number).await
    }
}

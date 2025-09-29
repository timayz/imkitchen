// Basic projection maintenance and consistency management

use chrono::{DateTime, Duration, Utc};

use super::{
    UserPreferencesProjectionBuilder, UserPreferencesView, UserProfileProjectionBuilder,
    UserProfileView,
};

/// Configuration for projection maintenance
#[derive(Debug, Clone)]
pub struct MaintenanceConfig {
    /// Maximum age of projections before rebuild (in hours)
    pub max_projection_age_hours: i64,
    /// Maximum version count before rebuild
    pub max_version_before_rebuild: u64,
    /// Batch size for processing events
    pub batch_size: usize,
}

impl Default for MaintenanceConfig {
    fn default() -> Self {
        Self {
            max_projection_age_hours: 24,   // Rebuild after 24 hours
            max_version_before_rebuild: 50, // Rebuild after 50 versions
            batch_size: 100,                // Process 100 events at a time
        }
    }
}

/// Statistics about projection maintenance
#[derive(Debug, Clone, Default)]
pub struct MaintenanceStats {
    pub total_maintenances: u64,
    pub projections_rebuilt: u64,
    pub projections_cleaned: u64,
    pub errors_encountered: u64,
    pub last_maintenance_duration_ms: u64,
}

/// Projection maintenance manager
#[derive(Debug)]
pub struct ProjectionMaintenanceManager {
    profile_builder: UserProfileProjectionBuilder,
    preferences_builder: UserPreferencesProjectionBuilder,
    config: MaintenanceConfig,
    last_maintenance: Option<DateTime<Utc>>,
    maintenance_stats: MaintenanceStats,
}

/// Information about projection caches
#[derive(Debug)]
pub struct ProjectionCacheInfo {
    pub profile_projections_count: usize,
    pub preferences_projections_count: usize,
    pub total_users_cached: usize,
    pub last_maintenance: Option<DateTime<Utc>>,
}

impl ProjectionMaintenanceManager {
    pub fn new(config: MaintenanceConfig) -> Self {
        Self {
            profile_builder: UserProfileProjectionBuilder::new(),
            preferences_builder: UserPreferencesProjectionBuilder::new(),
            config,
            last_maintenance: None,
            maintenance_stats: MaintenanceStats::default(),
        }
    }

    /// Check if a profile projection needs maintenance
    pub fn needs_maintenance_profile(&self, projection: &UserProfileView) -> bool {
        let now = Utc::now();
        let age = now - projection.last_profile_update;

        // Check age
        if age > Duration::hours(self.config.max_projection_age_hours) {
            return true;
        }

        // Check version count
        if projection.version > self.config.max_version_before_rebuild {
            return true;
        }

        false
    }

    /// Check if a preferences projection needs maintenance
    pub fn needs_maintenance_preferences(&self, projection: &UserPreferencesView) -> bool {
        let now = Utc::now();
        let age = now - projection.last_updated;

        // Check age
        if age > Duration::hours(self.config.max_projection_age_hours) {
            return true;
        }

        // Check version count
        if projection.version > self.config.max_version_before_rebuild {
            return true;
        }

        false
    }

    /// Get maintenance statistics
    pub fn get_stats(&self) -> &MaintenanceStats {
        &self.maintenance_stats
    }

    /// Get configuration
    pub fn get_config(&self) -> &MaintenanceConfig {
        &self.config
    }

    /// Update configuration
    pub fn update_config(&mut self, config: MaintenanceConfig) {
        self.config = config;
    }

    /// Clear all projection caches
    pub fn clear_all_caches(&mut self) {
        self.profile_builder.clear_cache();
        self.preferences_builder.clear_cache();
        println!("Cleared all projection caches");
    }

    /// Get projection cache info
    pub fn get_cache_info(&self) -> ProjectionCacheInfo {
        let profile_stats = self.profile_builder.cache_stats();
        let prefs_info = self.preferences_builder.maintenance_info();

        ProjectionCacheInfo {
            profile_projections_count: profile_stats.0,
            preferences_projections_count: prefs_info.total_projections,
            total_users_cached: profile_stats.1.len() + prefs_info.user_ids.len(),
            last_maintenance: self.last_maintenance,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_maintenance_config_default() {
        let config = MaintenanceConfig::default();

        assert_eq!(config.max_projection_age_hours, 24);
        assert_eq!(config.max_version_before_rebuild, 50);
        assert_eq!(config.batch_size, 100);
    }

    #[test]
    fn test_maintenance_stats_default() {
        let stats = MaintenanceStats::default();

        assert_eq!(stats.total_maintenances, 0);
        assert_eq!(stats.projections_rebuilt, 0);
        assert_eq!(stats.projections_cleaned, 0);
        assert_eq!(stats.errors_encountered, 0);
        assert_eq!(stats.last_maintenance_duration_ms, 0);
    }

    #[test]
    fn test_projection_maintenance_manager_creation() {
        let config = MaintenanceConfig::default();
        let manager = ProjectionMaintenanceManager::new(config.clone());

        assert!(manager.last_maintenance.is_none());
        assert_eq!(
            manager.get_config().max_projection_age_hours,
            config.max_projection_age_hours
        );
    }
}

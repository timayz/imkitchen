# Story 1.5: Premium Bypass Configuration

Status: drafted

## Story

As a developer,
I want to configure premium bypass globally or per-user,
So that development, staging, and demo accounts can bypass premium restrictions.

## Acceptance Criteria

1. Global premium bypass setting in config/default.toml (boolean)
2. Per-user premium_bypass flag in user profile (boolean)
3. Access control logic checks: global config OR user flag OR active premium subscription
4. Tests verify bypass behavior in both global and per-user scenarios
5. Documentation added to CLAUDE.md explaining bypass configuration

## Tasks / Subtasks

- [ ] Task 1: Add global bypass configuration (AC: 1)
  - [ ] Add [access_control] section to config/default.toml
  - [ ] Add global_premium_bypass setting (default: false)
  - [ ] Load config in server.rs startup
  - [ ] Make config accessible via Axum State

- [ ] Task 2: Implement AccessControlService (AC: 3)
  - [ ] Create src/access_control.rs module
  - [ ] Define AccessControlService struct with config and pool
  - [ ] Implement can_view_week(user_id, week_number) method
  - [ ] Implement can_add_favorite(user_id) method
  - [ ] Implement can_access_shopping_list(user_id, week_number) method
  - [ ] Each method checks: global bypass OR user premium_bypass OR is_premium_active

- [ ] Task 3: Premium bypass flag already in user_profiles (AC: 2)
  - [ ] Verify premium_bypass column exists from Story 1.3
  - [ ] Ensure UserPremiumBypassToggled event from Story 1.4 updates this flag
  - [ ] No additional migration needed

- [ ] Task 4: Integrate AccessControlService with existing routes (AC: 3)
  - [ ] Add AccessControlService to AppState
  - [ ] Update calendar routes to check can_view_week()
  - [ ] Update shopping list routes to check can_access_shopping_list()
  - [ ] Return upgrade prompt templates when access denied
  - [ ] Document access control points for future stories

- [ ] Task 5: Write comprehensive tests (AC: 4)
  - [ ] Create tests/access_control_test.rs
  - [ ] Test: Global bypass allows access to all features
  - [ ] Test: Per-user bypass allows access regardless of premium status
  - [ ] Test: Premium user has full access
  - [ ] Test: Free tier user restricted to Week 1 only
  - [ ] Test: Free tier user can add favorites up to 10
  - [ ] Test: Multiple bypass mechanisms work independently

- [ ] Task 6: Document bypass configuration (AC: 5)
  - [ ] Update CLAUDE.md with bypass configuration section
  - [ ] Explain global bypass use case (dev/staging environments)
  - [ ] Explain per-user bypass use case (demo accounts, testers)
  - [ ] Provide example config/dev.toml with bypass enabled
  - [ ] Warn about security: never enable global bypass in production

## Dev Notes

### Architecture Patterns

From [architecture.md](../architecture.md#adr-004-centralized-access-control-service):

**Centralized Access Control Service:**
```rust
pub struct AccessControlService {
    config: Config,
    pool: SqlitePool,
}

impl AccessControlService {
    pub async fn can_view_week(
        &self,
        user_id: &str,
        week_number: u8,
    ) -> anyhow::Result<bool> {
        // Check 1: Global bypass
        if self.config.access_control.global_premium_bypass {
            return Ok(true);
        }

        // Check 2: Load user profile
        let profile = get_user_profile(&self.pool, user_id).await?;

        // Check 3: Per-user bypass OR premium active
        Ok(profile.premium_bypass || profile.is_premium_active || week_number == 1)
    }

    pub async fn can_add_favorite(
        &self,
        user_id: &str,
    ) -> anyhow::Result<bool> {
        // Global bypass check
        if self.config.access_control.global_premium_bypass {
            return Ok(true);
        }

        // Load profile
        let profile = get_user_profile(&self.pool, user_id).await?;

        // Per-user bypass OR premium OR under limit
        if profile.premium_bypass || profile.is_premium_active {
            return Ok(true);
        }

        // Count current favorites
        let count = count_user_favorites(&self.pool, user_id).await?;
        Ok(count < 10)
    }
}
```

**Configuration File Structure:**

config/default.toml:
```toml
[access_control]
global_premium_bypass = false  # NEVER true in production
```

config/dev.toml (.gitignored):
```toml
[access_control]
global_premium_bypass = true  # Enable for local development
```

### Bypass Use Cases

From [PRD: FR051, FR116-FR119](../PRD.md#functional-requirements):

**Global Bypass:**
- Development environments (laptop, CI/CD)
- Staging environments for QA testing
- Never enabled in production

**Per-User Bypass:**
- Demo accounts for sales presentations
- Test accounts for automated E2E tests
- Beta testers needing full access temporarily
- Admin users managing community (optional)

**Priority Order:**
1. Global bypass (highest) - config file
2. Per-user bypass - database flag
3. Active premium subscription - database flag
4. Free tier restrictions (default)

### Freemium Tier Restrictions Summary

From [PRD: FR052-FR053](../PRD.md#functional-requirements):

**Free Tier Limits:**
- Calendar: Week 1 visible only (Weeks 2-5 locked)
- Dashboard: Nearest day only if in Week 1
- Favorites: Maximum 10 recipes
- Shopping Lists: Week 1 only
- Regenerations: Unlimited (no restriction)
- Community features: Full access (sharing, rating)

**Premium Tier (or Bypass):**
- Calendar: All weeks visible
- Dashboard: Nearest day from any week
- Favorites: Unlimited
- Shopping Lists: All weeks
- All other features unrestricted

### Story Dependencies

This story provides the access control foundation for:
- Story 2.3 (Recipe Favorites System) - 10 favorite limit
- Story 4.4 (Freemium Calendar Restrictions) - Week 1 visibility
- Story 4.5 (Dashboard Freemium Restrictions) - Nearest day access
- Story 4.7 (Shopping List Access) - Week-based access
- Story 5.4 (Freemium Favorite Limit) - Upgrade modal
- Story 5.6 (Premium Access Control Logic) - Centralized enforcement

### References

- [PRD: Freemium Access Control](../PRD.md#freemium-access-control) - FR051-FR053
- [Architecture: ADR-004](../architecture.md#adr-004-centralized-access-control-service) - Access control decision
- [Architecture: Premium Bypass Configuration](../architecture.md#premium-bypass) - Config structure

## Dev Agent Record

### Context Reference

<!-- Path(s) to story context XML will be added here by context workflow -->

### Agent Model Used

<!-- Will be filled by Dev agent -->

### Debug Log References

<!-- Dev agent logs will be added here -->

### Completion Notes List

<!-- Dev agent completion notes will be added here -->

### File List

<!-- List of files created/modified will be added here -->

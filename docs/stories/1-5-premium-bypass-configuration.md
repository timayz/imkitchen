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
  - [ ] Add global_premium_bypass = false (default for production)
  - [ ] Override to true in config/dev.toml for local development
  - [ ] Load configuration in main.rs and make available to server
  - [ ] Store global bypass config in AppState for route handlers

- [ ] Task 2: Per-user bypass flag (implemented in Story 1.4) (AC: 2)
  - [ ] Verify premium_bypass column exists in user_profiles table
  - [ ] Verify UserPremiumBypassToggled event exists in User aggregate
  - [ ] Verify toggle_premium_bypass command exists
  - [ ] Verify admin panel can toggle bypass flag per user
  - [ ] Note: Implementation done in Story 1.4, this story focuses on access control logic

- [ ] Task 3: Create Access Control Service (AC: 3)
  - [ ] Create src/access_control.rs module
  - [ ] Define AccessControlService struct with global_bypass config
  - [ ] Implement can_access_premium(&self, user: &User) -> bool
  - [ ] Logic: return true if global_bypass OR user.premium_bypass OR user.is_premium_active
  - [ ] Add AccessControlService to AppState
  - [ ] Inject service into route handlers that check premium features

- [ ] Task 4: Implement fine-grained access methods (AC: 3)
  - [ ] can_view_week(&self, user: &User, week_number: u32) -> bool
  - [ ] Logic: week 1 always accessible, weeks 2+ require premium access
  - [ ] can_add_favorite(&self, user: &User, current_count: u32) -> bool
  - [ ] Logic: free tier max 10, premium unlimited (or bypass enabled)
  - [ ] can_generate_shopping_list(&self, user: &User, week_number: u32) -> bool
  - [ ] Logic: week 1 for free, all weeks for premium
  - [ ] Note: These methods will be called in Epic 4-5 stories

- [ ] Task 5: Testing (AC: 4)
  - [ ] Create tests/access_control_test.rs
  - [ ] Test: Global bypass=true grants premium access to all users
  - [ ] Test: Global bypass=false, user bypass=true grants access
  - [ ] Test: Global bypass=false, user bypass=false, is_premium_active=true grants access
  - [ ] Test: All false denies access
  - [ ] Test: can_view_week allows week 1 for free tier, blocks week 2+
  - [ ] Test: can_add_favorite enforces 10 favorite limit for free tier
  - [ ] Test: Premium bypass overrides favorite limits

- [ ] Task 6: Documentation (AC: 5)
  - [ ] Update CLAUDE.md with Premium Bypass Configuration section
  - [ ] Explain global bypass (config/default.toml setting)
  - [ ] Explain per-user bypass (admin panel toggle)
  - [ ] Document access control methods and usage
  - [ ] Provide examples for dev/staging/demo environments
  - [ ] Note: Set global_premium_bypass=true in dev.toml for local testing

- [ ] Task 7: Code quality validation
  - [ ] Run cargo clippy and fix all warnings
  - [ ] Run cargo fmt --all
  - [ ] Verify all tests pass: cargo test
  - [ ] Manual test: Toggle global bypass, verify premium features accessible

## Dev Notes

### Architecture Patterns

**Access Control Service (Centralized):**
- Single source of truth for premium access decisions
- Used throughout application for freemium gating
- Injected via AppState to all route handlers
- Fine-grained methods for specific feature checks

**Bypass Hierarchy:**
1. **Global bypass** (config) - Entire environment bypasses (dev/staging)
2. **Per-user bypass** (DB flag) - Specific users bypass (demo accounts)
3. **Active premium subscription** (DB flag) - Paid users (Story 5.6)

**Configuration Strategy:**
```toml
# config/default.toml (committed)
[access_control]
global_premium_bypass = false  # Production default

# config/dev.toml (.gitignored)
[access_control]
global_premium_bypass = true   # Local development override
```

**Access Control Methods:**
- `can_access_premium(&self, user: &User) -> bool` - General premium check
- `can_view_week(&self, user: &User, week_number: u32) -> bool` - Calendar weeks
- `can_add_favorite(&self, user: &User, current_count: u32) -> bool` - Recipe favorites
- `can_generate_shopping_list(&self, user: &User, week_number: u32) -> bool` - Shopping lists

**Usage in Routes:**
```rust
let access_control = &state.access_control;
if !access_control.can_view_week(&user, week_number) {
    return UpgradePromptTemplate { feature: "full calendar" }.into_response();
}
```

### Project Structure Notes

New files added:
- `src/access_control.rs` - AccessControlService implementation
- `tests/access_control_test.rs` - Access control logic tests

**Configuration updates:**
- Add [access_control] section to config/default.toml
- Document override pattern in config/dev.toml.example

**AppState updates:**
- Add AccessControlService to AppState struct
- Initialize service in server.rs with loaded config

### References

- [Source: docs/epics.md#Story 1.5] - Complete acceptance criteria
- [Source: docs/architecture.md#ADR-004] - Centralized Access Control Service decision
- [Source: docs/PRD.md#Requirements FR051-FR053] - Freemium access control requirements
- [Source: docs/architecture.md#Deployment Architecture] - Configuration management pattern
- [Source: CLAUDE.md#CLI and Configuration] - TOML configuration rules

## Dev Agent Record

### Context Reference

<!-- Path(s) to story context XML will be added here by context workflow -->

### Agent Model Used

<!-- Will be populated during implementation -->

### Debug Log References

<!-- Will be populated during implementation -->

### Completion Notes List

<!-- Will be populated during implementation -->

### File List

<!-- Will be populated during implementation -->

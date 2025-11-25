# Story 4.4: Freemium Calendar Restrictions (First Week Only)

Status: drafted

## Story

As a free tier user,
I want to view only my first generated week,
So that I can experience meal planning value while understanding premium benefits.

## Acceptance Criteria

1. Free tier users can view first generated week fully (7 days, 3 courses each)
2. Weeks 2-N display "Upgrade to unlock" placeholder cards with upgrade CTA
3. Clicking locked week triggers upgrade modal
4. Premium tier users view all generated weeks without restrictions
5. Premium bypass configuration respected (global or per-user)
6. Tests verify free tier restrictions and premium access

## Tasks / Subtasks

- [ ] Create Access Control Service (AC: #1, #4, #5)
  - [ ] Create `src/access_control.rs` module
  - [ ] Implement `AccessControlService` struct with SqlitePool
  - [ ] Add `can_view_week(user_id: &str, week_number: usize) -> Result<bool>` method
  - [ ] Check premium access: is_premium_active OR premium_bypass OR global_bypass
  - [ ] Free tier logic: week_number == 1 allowed, others denied
  - [ ] Query user_profiles for is_premium_active and premium_bypass flags
  - [ ] Load global_premium_bypass from config
- [ ] Integrate Access Control in calendar route (AC: #1, #2, #4)
  - [ ] Modify `src/routes/mealplan/calendar.rs` to use AccessControlService
  - [ ] Check access for each week before rendering
  - [ ] Mark weeks 2-N as locked for free tier users
  - [ ] Pass locked status to template for conditional rendering
  - [ ] Premium users: all weeks rendered fully
- [ ] Update calendar template with locked week UI (AC: #2, #3)
  - [ ] Modify `templates/pages/mealplan/calendar.html` for locked weeks
  - [ ] Display "Upgrade to unlock" placeholder for weeks 2-N (free tier)
  - [ ] Show lock icon and upgrade CTA button
  - [ ] Clicking locked week/CTA triggers upgrade modal
  - [ ] Use Twinspark `ts-action` to open upgrade modal
  - [ ] Keep first week fully visible with all meal cards
- [ ] Create upgrade modal component (AC: #3)
  - [ ] Create `templates/components/upgrade-modal.html`
  - [ ] Display modal title: "Unlock Full Month Visibility"
  - [ ] Show feature comparison: Free (Week 1 only) vs Premium (All weeks)
  - [ ] Display pricing: $9.99/month or $59.94/year
  - [ ] Add "Upgrade Now" CTA button linking to pricing page
  - [ ] Add close/dismiss button
  - [ ] Style with Tailwind modal utilities
  - [ ] Use Twinspark for modal open/close without JavaScript
- [ ] Add global premium bypass configuration (AC: #5)
  - [ ] Add to `config/default.toml`: `global_premium_bypass = false`
  - [ ] Document in comments: set to true for dev/staging environments
  - [ ] Load config value in AccessControlService on init
  - [ ] Test bypass behavior: true → all users have premium access
- [ ] Integrate with user profile premium fields (AC: #4, #5)
  - [ ] Ensure user_profiles table has `is_premium_active` and `premium_bypass` columns
  - [ ] Query these fields in AccessControlService.can_view_week()
  - [ ] Per-user bypass: premium_bypass=true → full access regardless of subscription
  - [ ] Active premium: is_premium_active=true → full access
- [ ] Write integration tests (AC: #6)
  - [ ] Extend `tests/calendar_test.rs` with access control tests
  - [ ] Test free tier user: first week visible, weeks 2+ locked
  - [ ] Test premium tier user: all weeks visible
  - [ ] Test global bypass: free user sees all weeks when global_premium_bypass=true
  - [ ] Test per-user bypass: free user with premium_bypass=true sees all weeks
  - [ ] Test locked week UI: verify "Upgrade to unlock" placeholder rendered
  - [ ] Test upgrade modal trigger on locked week click

## Dev Notes

### Architecture Patterns and Constraints

**Access Control Service Pattern (per ADR-004):**
- Centralized service enforces freemium logic consistently
- Single source of truth prevents scattered access checks
- Service injected into route handlers via AppState
- Fine-grained control: week-level, not just route-level

**Access Control Logic:**
```rust
// src/access_control.rs
pub struct AccessControlService {
    pool: SqlitePool,
    global_bypass: bool,
}

impl AccessControlService {
    pub async fn can_view_week(&self, user_id: &str, week_number: usize) -> anyhow::Result<bool> {
        // 1. Check global bypass (dev/staging)
        if self.global_bypass { return Ok(true); }

        // 2. Query user profile
        let profile = get_user_profile(&self.pool, user_id).await?;

        // 3. Check per-user bypass
        if profile.premium_bypass { return Ok(true); }

        // 4. Check active premium subscription
        if profile.is_premium_active { return Ok(true); }

        // 5. Free tier: only week 1 accessible
        Ok(week_number == 1)
    }
}
```

**Freemium Conversion Strategy:**
- Locked weeks show clear value proposition: "See all 4 weeks"
- Upgrade modal emphasizes time savings and planning visibility
- No unfavoriting workaround (Story 5.4 enforces favorite limits separately)
- Pricing displayed prominently: monthly and annual options

**Premium Bypass Use Cases:**
- Global bypass: Development and staging environments (no payment required)
- Per-user bypass: Demo accounts, beta testers, promotional access
- Configuration file approach per CLAUDE.md standards (no .env files)

**Template Conditional Rendering:**
```html
<!-- templates/pages/mealplan/calendar.html -->
{% for week in weeks %}
  {% if week.is_locked %}
    <!-- Locked week placeholder -->
    <div class="locked-week">
      <div class="lock-icon">🔒</div>
      <p>Upgrade to unlock Week {{ week.number }}</p>
      <button ts-action="target #upgrade-modal, class+ show">Upgrade Now</button>
    </div>
  {% else %}
    <!-- Full week display with meal cards -->
    ...
  {% endif %}
{% endfor %}
```

**Query Optimization:**
- Access control check adds single query per page load
- User profile cached in session after first load (optional optimization)
- No performance impact: <10ms for access check query

**Testing Strategy:**
- Integration tests use in-memory SQLite databases
- Set up test users with different tier configurations
- Test all bypass scenarios: global, per-user, premium active
- Verify template rendering for locked vs accessible weeks

### Project Structure Notes

**Files to Create/Modify:**
- `src/access_control.rs` - Access Control Service (NEW)
- `src/lib.rs` - Export AccessControlService module (MODIFY)
- `src/server.rs` - Add AccessControlService to AppState (MODIFY)
- `src/routes/mealplan/calendar.rs` - Integrate access control (MODIFY)
- `templates/pages/mealplan/calendar.html` - Add locked week UI (MODIFY)
- `templates/components/upgrade-modal.html` - Upgrade modal component (NEW)
- `config/default.toml` - Add global_premium_bypass setting (MODIFY)
- `tests/calendar_test.rs` - Add access control tests (MODIFY)

**AppState Integration:**
```rust
// src/lib.rs or src/server.rs
pub struct AppState {
    pub pool: SqlitePool,
    pub executor: SqliteExecutor,
    pub access_control: AccessControlService,
    // ... other services
}
```

**Configuration Example:**
```toml
# config/default.toml
[access_control]
global_premium_bypass = false  # Set to true for dev/staging

# config/dev.toml (local development)
[access_control]
global_premium_bypass = true  # All users have premium access
```

**Visual Mockup Alignment:**
- Implements `mockups/calendar-free.html` locked week placeholders
- Week 1 fully visible matches mockup first-week-only restriction
- Weeks 2-5 locked with "Upgrade to unlock" matches mockup upgrade prompts
- Upgrade modal matches mockup conversion touchpoint design

**Integration with Story 1.5:**
- Story 1.5 implemented premium bypass configuration infrastructure
- This story consumes that configuration in access control logic
- Global bypass and per-user bypass flags defined in Story 1.5

### References

- [Source: docs/epics.md#Story 4.4 - Acceptance Criteria and Prerequisites]
- [Source: docs/PRD.md#FR031 - Free tier first week visibility]
- [Source: docs/PRD.md#FR032 - Premium tier all weeks access]
- [Source: docs/PRD.md#FR051 - Premium bypass configuration]
- [Source: docs/PRD.md#FR052 - Free tier visibility restrictions]
- [Source: docs/architecture.md#ADR-004 - Centralized Access Control Service]
- [Source: docs/architecture.md#Data Architecture - user_profiles premium fields]
- [Source: CLAUDE.md#Configuration Guidelines - TOML configuration approach]

## Dev Agent Record

### Context Reference

<!-- Path(s) to story context XML will be added here by context workflow -->

### Agent Model Used

<!-- Model information will be added during implementation -->

### Debug Log References

<!-- Debug logs will be added during implementation -->

### Completion Notes List

<!-- Completion notes will be added during implementation -->

### File List

<!-- Files created/modified will be listed during implementation -->

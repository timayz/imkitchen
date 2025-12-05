# Story 5.5: Upgrade Prompts (Multiple Touchpoints)

Status: drafted

## Story

As a product owner,
I want upgrade prompts displayed at strategic touchpoints,
so that free tier users are aware of premium benefits and conversion opportunities.

## Acceptance Criteria

1. Upgrade modal shown when clicking locked weeks in calendar (references Story 4.4 implementation)
2. Upgrade prompt shown on dashboard if nearest day is outside accessible week (references Story 4.5 implementation)
3. Upgrade modal shown when attempting 11th favorite (implemented in Story 5.4)
4. All upgrade prompts include consistent messaging: feature comparison table, pricing ($9.99/month or $59.94/year), "Upgrade Now" CTA
5. Prompts are dismissible (close button) but reappear on next trigger event
6. Prompts are non-intrusive (modal overlay, not blocking entire UI)
7. "Upgrade Now" button links to pricing/upgrade page (route: /pricing)
8. Tests verify modal triggering at each touchpoint and consistent content

## Tasks / Subtasks

- [ ] Create reusable upgrade modal component (AC: #4, #5, #6)
  - [ ] Create `templates/components/upgrade-modal.html` (if not already from Story 5.4)
  - [ ] Accept parameters: trigger_type ("favorite_limit", "locked_week", "dashboard_restricted"), custom_message (optional)
  - [ ] Display feature comparison table:
    - Free Tier: 10 favorites, Week 1 visible only, unlimited regenerations
    - Premium Tier: Unlimited favorites, full month visible, all features ($9.99/mo or $59.94/yr)
  - [ ] Include "Upgrade Now" CTA button linking to /pricing route
  - [ ] Include "Close" button with Twinspark ts-action to dismiss modal
  - [ ] Modal uses overlay (semi-transparent background) to maintain context
- [ ] Integrate upgrade modal with locked week clicks (AC: #1)
  - [ ] Update calendar template `templates/pages/mealplan/calendar.html`
  - [ ] For locked weeks (weeks 2-5 for free tier), add click handler with Twinspark
  - [ ] Use ts-req="/upgrade-prompt?trigger=locked_week" on locked week cards
  - [ ] Return upgrade modal partial from GET `/upgrade-prompt` route
  - [ ] Modal replaces target div (e.g., #upgrade-modal-container)
- [ ] Integrate upgrade prompt with dashboard restriction (AC: #2)
  - [ ] Update dashboard template `templates/pages/dashboard.html`
  - [ ] For free tier users, check if nearest day falls outside Week 1
  - [ ] If outside Week 1, display inline upgrade banner (not modal):
    - Message: "Upgrade to see upcoming meals beyond Week 1"
    - "View Full Calendar" button linking to /pricing
  - [ ] Banner dismissible with Twinspark action (stores dismissed state in session)
- [ ] Confirm favorite limit integration (AC: #3)
  - [ ] Verify Story 5.4 implementation already triggers upgrade modal
  - [ ] Ensure modal content matches consistent format from this story
  - [ ] Test favorite limit → upgrade modal flow
- [ ] Create upgrade prompt route handler (AC: #1)
  - [ ] Create GET `/upgrade-prompt` route in `src/routes/upgrade.rs`
  - [ ] Accept query parameter: trigger (favorite_limit, locked_week, dashboard_restricted)
  - [ ] Render upgrade modal partial with trigger-specific messaging
  - [ ] Return partial HTML for Twinspark to inject into page
- [ ] Create pricing/upgrade landing page (AC: #7)
  - [ ] Create GET `/pricing` route in `src/routes/pricing.rs`
  - [ ] Create `templates/pages/pricing.html`
  - [ ] Display detailed feature comparison table (Free vs Premium)
  - [ ] Show pricing options: $9.99/month or $59.94/year (50% savings)
  - [ ] Include "Get Started Free" CTA (links to /auth/register)
  - [ ] Include "Upgrade to Premium" CTA (deferred to payment integration post-MVP)
  - [ ] Add FAQ section answering common questions
- [ ] Write tests (AC: #8)
  - [ ] Test upgrade modal triggered by clicking locked week in calendar
  - [ ] Test upgrade modal content includes feature comparison and pricing
  - [ ] Test upgrade modal includes "Upgrade Now" button linking to /pricing
  - [ ] Test upgrade modal dismissible with close button
  - [ ] Test dashboard upgrade prompt shown when nearest day outside Week 1
  - [ ] Test upgrade modal triggered by attempting 11th favorite (Story 5.4)
  - [ ] Test modal reappears after dismissal on next trigger event
  - [ ] Test modal content consistent across all touchpoints

## Dev Notes

### Architecture Patterns

**Upgrade Touchpoints Summary:**
1. **Locked Week Clicks** - Modal triggered by Twinspark ts-req on calendar locked week cards
2. **Dashboard Restriction** - Inline banner displayed when nearest day outside accessible week (non-modal)
3. **Favorite Limit** - Modal triggered when favorite command returns FavoriteLimitReached error

**Modal vs Inline Prompt:**
- **Modal (overlay)**: Locked week clicks, favorite limit - immediate conversion opportunity
- **Inline banner**: Dashboard restriction - less intrusive, persistent visibility

**Modal Dismissal Strategy:**
- Dismissible with close button (Twinspark action removes modal div)
- NOT persistent - modal reappears on next trigger event
- Goal: Balance conversion incentive with user experience

**Pricing Page Purpose:**
- Central destination for all "Upgrade Now" CTAs
- Detailed feature comparison beyond modal summary
- FAQ addressing objections and questions
- Payment integration deferred to post-MVP (Story links but no payment flow yet)

### Project Structure Notes

**Files to Create/Modify:**
- `templates/components/upgrade-modal.html` - Reusable modal component (may exist from Story 5.4)
- `templates/pages/mealplan/calendar.html` - Add Twinspark handlers to locked week cards
- `templates/pages/dashboard.html` - Add inline upgrade banner for free tier
- `templates/pages/pricing.html` - Pricing/upgrade landing page
- `src/routes/upgrade.rs` - Upgrade prompt route handler
- `src/routes/pricing.rs` - Pricing page route handler
- `tests/upgrade_test.rs` - Upgrade prompt tests

**Twinspark Integration:**
- Locked week click: `ts-req="/upgrade-prompt?trigger=locked_week" ts-target="#modal-container"`
- Modal close button: `ts-action="remove" ts-trigger="click"`
- Dashboard banner dismiss: `ts-action="class+ hidden" ts-trigger="click"`

**Feature Comparison Table Content:**
| Feature | Free Tier | Premium Tier |
|---------|-----------|--------------|
| Recipe Favorites | 10 max | Unlimited |
| Calendar Visibility | Week 1 only | Full month (4-5 weeks) |
| Shopping Lists | Week 1 only | All weeks |
| Regenerations | Unlimited | Unlimited |
| Community Features | Full access | Full access |

### References

- [Source: docs/epics.md#Story 5.5] - Story acceptance criteria and prerequisites
- [Source: docs/epics.md#Story 4.4] - Locked week calendar restrictions
- [Source: docs/epics.md#Story 4.5] - Dashboard freemium restrictions
- [Source: docs/epics.md#Story 5.4] - Favorite limit upgrade modal
- [Source: docs/PRD.md#User Journey 1] - Upgrade decision point in user flow
- [Source: CLAUDE.md#Twinspark API Reference] - ts-req, ts-action, ts-target usage

## Dev Agent Record

### Context Reference

<!-- Path(s) to story context XML will be added here by context workflow -->

### Agent Model Used

{{agent_model_name_version}}

### Debug Log References

### Completion Notes List

### File List

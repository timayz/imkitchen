# Story 5.5: Upgrade Prompts (Multiple Touchpoints)

Status: drafted

## Story

As a product owner,
I want upgrade prompts displayed at strategic touchpoints,
so that free tier users are aware of premium benefits and conversion opportunities.

## Acceptance Criteria

1. "Get Premium" CTA in the sidebar (expanded card on desktop, icon-only on the rail) and in the mobile bottom navigation opens the upgrade modal (references Story 5.4 implementation)
2. Ad slots rendered on content pages for free tier users carry a "Go Premium to remove them." link to the upgrade page (references Story 5.4 implementation)
3. Pricing page carries an "Upgrade to Premium" link for signed-in free users (references Story 6.2 implementation)
4. All upgrade prompts include consistent messaging: ad-free benefit, pricing (monthly or annual plan), "Get Premium" CTA
5. Modal is dismissible (close button) and reappears on the next trigger; ad slots are inline and not dismissible
6. Prompts are non-intrusive (modal overlay, not blocking entire UI; ad slots placed below page content)
7. No prompt is shown to premium users, in demo mode, or when premium is not configured (self-hosted)
8. Tests verify modal triggering at each touchpoint and consistent content

## Tasks / Subtasks

- [ ] Reuse the upgrade modal component (AC: #4, #5, #6)
  - [ ] Use `templates/partials/upgrade-modal.html` (created in Story 5.4)
  - [ ] Accept parameters: trigger_type ("sidebar_cta", "ad_slot", "pricing_page"), custom_message (optional)
  - [ ] Display tier comparison:
    - Free Tier: full features, ad-supported
    - Premium Tier: full features, no ads (monthly or annual plan)
  - [ ] Include "Get Premium" CTA button continuing to the plan/order summary
  - [ ] Include "Close" button with Twinspark ts-action to dismiss modal
  - [ ] Modal uses overlay (semi-transparent background) to maintain context
- [ ] Integrate upgrade modal with the "Get Premium" navigation CTA (AC: #1, #7)
  - [ ] Update the user layout `templates/_user.html`
  - [ ] Sidebar footer: expanded card on xl, icon-only button on the md rail
  - [ ] Mobile bottom navigation: "Premium" entry
  - [ ] Use ts-req="/upgrade/modal" ts-target="body" ts-swap="append" on each CTA
  - [ ] Wrap every CTA in `{% if !user.is_premium() && !demo %}`
- [ ] Integrate upgrade link with ad slots (AC: #2, #7)
  - [ ] Use the `ads::slot` macro from `templates/partials/ad-slot.html`
  - [ ] Render the "content-bottom" slot in `templates/_user.html` inside `{% if user.show_ads() && !demo %}`
  - [ ] Slot text: "Ads keep imkitchen free." with link "Go Premium to remove them." to `/upgrade`
- [ ] Integrate upgrade link with the pricing page (AC: #3)
  - [ ] Verify Story 6.2 pricing page shows "Upgrade to Premium" for signed-in free users and "Get Started Free" for visitors
  - [ ] Link routes to `/upgrade` (page) or opens `/upgrade/modal` when rendered inside the user layout
- [ ] Create upgrade prompt route handler (AC: #1, #7)
  - [ ] GET `/upgrade/modal` route in `web/public/src/routes/upgrade.rs`
  - [ ] Redirect to `/` when the user is premium or premium is not configured
  - [ ] Render upgrade modal partial with `ts-swap: skip` header so Twinspark appends it to body
  - [ ] GET `/upgrade` page redirects premium users to `/settings/billing`
- [ ] Write tests (AC: #8)
  - [ ] Test upgrade modal returned from `/upgrade/modal` for a free user
  - [ ] Test `/upgrade/modal` redirects for a premium user and when premium is not configured
  - [ ] Test "Get Premium" CTA present in sidebar and mobile navigation for free users, absent for premium and demo
  - [ ] Test ad slot rendered for free users with the upgrade link, absent for premium and demo
  - [ ] Test upgrade modal content includes tier comparison and pricing
  - [ ] Test upgrade modal dismissible with close button
  - [ ] Test modal reappears after dismissal on next trigger event
  - [ ] Test modal content consistent across all touchpoints

## Dev Notes

### Architecture Patterns

**Upgrade Touchpoints Summary:**
1. **Get Premium CTA** - Modal triggered by Twinspark ts-req on the sidebar card, rail icon and mobile navigation entry
2. **Ad Slots** - Inline link inside every ad slot rendered for free users (non-modal)
3. **Pricing Page** - "Upgrade to Premium" link for signed-in free users

**Modal vs Inline Prompt:**
- **Modal (overlay)**: Get Premium CTA - immediate conversion opportunity
- **Inline link**: Ad slots and pricing page - less intrusive, persistent visibility

**Modal Dismissal Strategy:**
- Dismissible with close button (Twinspark action removes modal div)
- NOT persistent - modal reappears on next trigger event
- Goal: Balance conversion incentive with user experience

**Visibility Rules:**
- `user.is_premium()` hides every CTA; `user.show_ads()` (the negation) gates ad slots
- Demo, admin and self-hosted (premium config unset) sessions are treated as premium upstream and never see prompts

**Pricing Page Purpose:**
- Central destination for visitors comparing tiers
- Tier comparison beyond modal summary
- FAQ addressing objections and questions
- Payment flow lives in the upgrade page/modal (plan, amount, country/state for tax)

### Project Structure Notes

**Files to Create/Modify:**
- `templates/partials/upgrade-modal.html` - Reusable modal component (from Story 5.4)
- `templates/partials/upgrade-order-summary.html` - Plan/price summary shown in the modal
- `templates/partials/ad-slot.html` - `ads::slot` macro with the inline upgrade link
- `templates/_user.html` - Get Premium CTAs (sidebar, rail, mobile nav) and content-bottom ad slot
- `templates/pricing.html` - Pricing page upgrade link (Story 6.2)
- `web/public/src/routes/upgrade.rs` - Upgrade page, modal and order-summary route handlers
- `tests/upgrade_test.rs` - Upgrade prompt tests

**Twinspark Integration:**
- Get Premium CTA: `ts-trigger="click" ts-req="/upgrade/modal" ts-target="body" ts-swap="append"`
- Modal close button: `ts-action="remove" ts-trigger="click"`
- Modal response sets header `ts-swap: skip` so the triggering element is not replaced

**Tier Comparison Table Content:**
| Feature | Free Tier | Premium Tier |
|---------|-----------|--------------|
| Recipe Favorites | Unlimited | Unlimited |
| Recipe List Generation | Unlimited | Unlimited |
| Cooking Mode & Groceries | Full access | Full access |
| Community Features | Full access | Full access |
| Ads | Shown | Removed |

### References

- [Source: docs/epics.md#Story 5.5] - Story acceptance criteria and prerequisites
- [Source: docs/epics.md#Story 5.4] - Ad-supported free tier, `show_ads()` and ad slots
- [Source: docs/epics.md#Story 5.6] - Premium access control logic
- [Source: docs/epics.md#Story 6.2] - Pricing page with tier comparison
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

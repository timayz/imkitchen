# Story 1.7: Premium Subscription Upgrade Flow

Status: Approved

## Story

As a free tier user,
I want to upgrade to premium,
So that I can access unlimited recipes and advanced features.

## Acceptance Criteria

1. "Upgrade to Premium" button visible on subscription page and within freemium restriction prompts
2. Subscription page displays premium benefits (unlimited recipes, advanced features) and pricing ($9.99/month)
3. Clicking "Upgrade to Premium" redirects to Stripe Checkout hosted page
4. Stripe Checkout accepts credit/debit card payment securely
5. Successful payment triggers `checkout.session.completed` webhook
6. Webhook handler upgrades user tier to "premium" via `SubscriptionUpgraded` event
7. User redirected back to `/subscription/success` after successful payment
8. Premium tier status immediately reflected in UI (badge, unlimited recipe indicator)
9. All freemium restrictions (10 recipe limit) removed for premium users
10. Failed payment displays Stripe error message and allows retry
11. User can cancel Stripe Checkout and return to app without charge

## Tasks / Subtasks

- [ ] Create subscription management UI (AC: 1, 2)
  - [ ] Create subscription page route `GET /subscription` in `src/routes/profile.rs`
  - [ ] Create `templates/pages/subscription.html` with tier status display
  - [ ] Display current tier (Free/Premium) with badge styling
  - [ ] Show premium benefits list: "Unlimited recipes", "Advanced scheduling", "Priority support"
  - [ ] Display pricing: "$9.99/month" with "Upgrade to Premium" button
  - [ ] Add "Upgrade to Premium" button in recipe limit error prompts
  - [ ] Style upgrade button with prominent CTA styling (Tailwind)

- [ ] Implement Stripe Checkout integration (AC: 3, 4)
  - [ ] Add async-stripe dependency (0.39+) to root `Cargo.toml`
  - [ ] Create `POST /subscription/upgrade` route handler in `src/routes/profile.rs`
  - [ ] Initialize Stripe client with `STRIPE_SECRET_KEY` from config
  - [ ] Create Checkout Session with:
    - `mode: Subscription`
    - `success_url: /subscription/success`
    - `cancel_url: /subscription`
    - `customer_email: user.email`
    - `line_items: [{ price: STRIPE_PRICE_ID, quantity: 1 }]`
    - `metadata: { user_id: auth.user_id }`
  - [ ] Redirect user to Checkout Session URL (302 redirect)
  - [ ] Create success page `GET /subscription/success` showing confirmation

- [ ] Implement webhook handler (AC: 5, 6, 8, 9)
  - [ ] Create `POST /webhooks/stripe` route in `src/routes/auth.rs`
  - [ ] Verify webhook signature using `stripe-signature` header and `STRIPE_WEBHOOK_SECRET`
  - [ ] Handle `checkout.session.completed` event:
    - Extract `user_id` from session metadata
    - Extract `customer_id` and `subscription_id` from session
    - Call `user::upgrade_subscription` command with Stripe IDs
  - [ ] Append `SubscriptionUpgraded` event to user aggregate
  - [ ] Update read model projection to set `tier="premium"`, store Stripe IDs
  - [ ] Return 200 OK to acknowledge webhook
  - [ ] Log webhook signature verification failures (security monitoring)

- [ ] Add SubscriptionUpgraded event handling (AC: 6, 8, 9)
  - [ ] Define `SubscriptionUpgraded` event in `crates/user/src/events.rs`
  - [ ] Add `subscription_upgraded` event handler to `UserAggregate`
  - [ ] Create projection handler in `crates/user/src/read_model.rs`
  - [ ] Update users table: `tier`, `stripe_customer_id`, `stripe_subscription_id`
  - [ ] Export `upgrade_subscription` command from `crates/user/src/lib.rs`

- [ ] Display premium status (AC: 8, 9)
  - [ ] Query user tier in all route handlers requiring tier display
  - [ ] Show "Premium Member" badge on `/profile` and `/subscription` pages
  - [ ] Display "Unlimited recipes" indicator on recipe library page
  - [ ] Update recipe count badge component to hide count for premium users
  - [ ] Remove "Upgrade" button from subscription page if already premium

- [ ] Handle errors and edge cases (AC: 10, 11)
  - [ ] Stripe Checkout displays payment errors (handled by Stripe UI)
  - [ ] User cancels checkout → redirected to `/subscription` (no charge)
  - [ ] Webhook signature verification fails → return 401, log security event
  - [ ] Duplicate webhook delivery → idempotent event handling (evento)
  - [ ] User already premium → prevent duplicate upgrade, show current status

- [ ] Test premium upgrade flow (AC: 1-11)
  - [ ] Unit test: `upgrade_subscription` command creates `SubscriptionUpgraded` event
  - [ ] Unit test: `subscription_upgraded` event handler updates aggregate `tier` field
  - [ ] Unit test: Projection handler updates users table correctly
  - [ ] Integration test: POST /subscription/upgrade creates Checkout Session, redirects
  - [ ] Integration test: Mock webhook with valid signature upgrades user tier
  - [ ] Integration test: Mock webhook with invalid signature returns 401
  - [ ] Integration test: Premium user bypasses recipe limit validation
  - [ ] E2E test: Free user → Upgrade → Mock Stripe payment → Webhook → Premium status → Create recipe #11 succeeds

## Dev Notes

### Architecture Patterns

**Stripe Integration**:
- Stripe Checkout hosted page handles PCI compliance (no card data touches imkitchen servers)
- async-stripe crate provides type-safe Rust client
- Checkout Session mode: Subscription (recurring billing managed by Stripe)
- Webhook events provide asynchronous payment confirmation

**Event Sourcing**:
- `SubscriptionUpgraded` event captures tier change with Stripe metadata
- User aggregate tracks `tier`, `stripe_customer_id`, `stripe_subscription_id`
- Read model projection updates users table for fast queries
- Idempotent event handling prevents duplicate upgrades from webhook retries

**Security**:
- Webhook signature verification prevents forged payment events
- HTTP-only secure cookies maintain user session
- Stripe Customer ID and Subscription ID stored for future management (cancellation, billing)

### Source Tree Components

**Subscription Routes** (`src/routes/profile.rs`):
- `GET /subscription` → Display subscription page with tier status and upgrade button
- `POST /subscription/upgrade` → Create Stripe Checkout Session, redirect to Stripe
- `GET /subscription/success` → Success confirmation page after payment

**Webhook Handler** (`src/routes/auth.rs`):
- `POST /webhooks/stripe` → Verify signature, handle Stripe events, upgrade tier

**User Domain** (`crates/user/`):
- `commands.rs`: `upgrade_subscription(user_id, stripe_customer_id, stripe_subscription_id)`
- `events.rs`: `SubscriptionUpgraded { new_tier, stripe_customer_id, stripe_subscription_id }`
- `aggregate.rs`: `subscription_upgraded` event handler updates tier
- `read_model.rs`: Projection handler updates users table

**Templates** (`templates/pages/`):
- `subscription.html`: Subscription management page (tier status, upgrade button)
- `subscription-success.html`: Payment confirmation page

**Configuration**:
- `STRIPE_SECRET_KEY`: Stripe API secret key (environment variable)
- `STRIPE_WEBHOOK_SECRET`: Webhook signature verification secret
- `STRIPE_PRICE_ID`: Price ID for $9.99/month premium subscription

### Testing Standards

**Unit Tests** (`crates/user/tests/`):
- Test `upgrade_subscription` command creates `SubscriptionUpgraded` event
- Test event handler updates `UserAggregate.tier` to Premium
- Test projection updates users table with Stripe IDs

**Integration Tests** (`tests/subscription_tests.rs`):
- POST /subscription/upgrade creates Checkout Session, redirects to Stripe
- Mock `checkout.session.completed` webhook upgrades user tier
- Invalid webhook signature returns 401 Unauthorized
- Premium user can create unlimited recipes (freemium validation passes)

**E2E Tests** (`e2e/tests/subscription.spec.ts`):
- Complete upgrade flow: Free user → Click upgrade → Mock Stripe payment → Webhook → Premium status → Recipe #11 creation succeeds
- Cancel Checkout flow: User cancels → returns to /subscription → no charge

### References

**PRD**:
- [Source: docs/PRD.md#FR-14] - Freemium business model with $9.99/month premium tier

**Epic Specification**:
- [Source: docs/epics.md#Story 1.7] - Original story definition for premium upgrade flow
- [Source: docs/tech-spec-epic-1.md#Story 9] - AC-9.1 to AC-9.5: Authoritative acceptance criteria for premium upgrade

**Architecture**:
- [Source: docs/solution-architecture.md#Section 1.1] - async-stripe (0.39+) in technology stack
- [Source: docs/solution-architecture.md#ADR-006] - Freemium model rationale and premium tier design
- [Source: docs/solution-architecture.md#Section 5.3] - Payment gateway integration strategy

**Technical Specification**:
- [Source: docs/tech-spec-epic-1.md#Subscription Routes] - Stripe Checkout Session creation and webhook handler implementation
- [Source: docs/tech-spec-epic-1.md#Events/SubscriptionUpgraded] - Event structure and projection logic
- [Source: docs/tech-spec-epic-1.md#Workflows/Upgrade to Premium Flow] - Complete upgrade flow sequence

### Project Structure Notes

**Alignment with solution-architecture.md**:
- Stripe integration follows external service integration pattern (Section 11.3)
- Webhook handler follows auth route conventions (Section 2.3)
- Payment processing avoids PCI compliance burden via Stripe Checkout
- No card data stored in imkitchen database (Stripe Customer ID reference only)

**Cross-Domain Integration**:
- User domain owns subscription tier enforcement
- Recipe domain queries user tier via `validate_recipe_creation` (Story 1.6)
- Subscription upgrade immediately affects recipe creation validation

**Rationale**:
- Stripe Checkout hosted page reduces implementation complexity and security risk
- Webhook-based upgrade flow supports asynchronous payment confirmation
- Subscription ID storage enables future subscription management (cancellation, updates)

## Dev Agent Record

### Context Reference

- [Story Context XML](../story-context-1.7.xml) - Generated 2025-10-13

### Agent Model Used

claude-sonnet-4-5-20250929

### Debug Log References

### Completion Notes List

### File List

# imkitchen Product Requirements Document (PRD)

**Author:** Jonathan
**Date:** 2025-10-31
**Project Level:** 3
**Target Scale:** Comprehensive product with freemium model, community features, and intelligent meal planning automation

---

## Goals and Background Context

### Goals

- Increase recipe variety by enabling users to cook 3x more unique recipes per month through intelligent multi-week rotation
- Reduce meal planning time by 60% through one-click month-based generation
- Eliminate timing complexity and advance preparation stress with automated reminders and scheduling
- Enable realistic meal composition through intelligent accompaniment pairing (85% success rate)
- Extend planning horizons from 1 week to average of 3.5 weeks for better grocery shopping and reduced waste
- Build sustainable community engagement through recipe sharing, rating, and discovery across four recipe types
- Achieve 15% freemium-to-premium conversion within 60 days by demonstrating value through first-week access

### Background Context

Home cooks face a fundamental choice between culinary variety and planning simplicity. The complexity of coordinating recipe selection, advance preparation timing, meal composition, and multi-week planning creates a self-limitation pattern where users avoid 70-80% of their saved recipes. Current meal planning apps focus on recipe storage without addressing the core problems: timing complexity, realistic meal composition (mains paired with appropriate sides), and extended planning visibility.

imkitchen solves this by automatically generating meal plans for all future weeks of the current month with intelligent accompaniment pairing, preference-aware scheduling, and comprehensive shopping lists. The freemium model (first week visible on free tier, all weeks on premium) provides immediate value demonstration while creating natural upgrade incentive for full month visibility and unlimited recipe favorites.

---

## Requirements

### Functional Requirements

**Authentication & User Management**

- FR001: System shall support user registration, login, and profile management with JWT cookie-based authentication
- FR002: System shall store user preferences including dietary restrictions, complexity preferences, cuisine variety weight (default 0.7), and household size
- FR003: System shall support admin users identified by `is_admin` flag with access to admin panel

**Recipe Management**

- FR004: System shall allow users to create, edit, and delete recipes with four types: Appetizer, Main Course, Dessert, Accompaniment
- FR005: Recipe fields (dietary restrictions, cuisine type, etc.) shall be explicitly set by users with no automatic inference
- FR006: Main courses shall default to `accepts_accompaniment=false` requiring explicit user enablement
- FR007: System shall support bulk recipe import from JSON files (max 10MB per file, 20 files per batch) via drag-and-drop
- FR008: Recipe import shall validate against HTML form schema, skip invalid recipes, and provide detailed success/failure summary
- FR009: Recipe import shall detect and block duplicate recipes (matching name or similar ingredients)
- FR010: Recipe import shall display real-time progress (imported count, failed count, remaining count)
- FR011: System shall maintain publicly accessible versioned JSON schema documentation for recipe import

**Recipe Favorites & Sharing**

- FR012: Users shall favorite both their own recipes and community-shared recipes from other users
- FR013: Free tier users shall be limited to maximum 10 favorited recipes; premium tier shall have unlimited favorites
- FR014: When free tier users attempt to exceed 10 favorites, system shall display upgrade modal with no unfavoriting option
- FR015: Users shall share recipes publicly with community (all four recipe types) in both free and premium tiers
- FR016: When recipe owner deletes their recipe, it shall be automatically removed from all users' favorites without notifications
- FR017: Users shall rate and review community-shared recipes with ratings acting as quality filter in search results

**Meal Plan Generation**

- FR018: System shall automatically generate meal plans for all future weeks of current month (from next week onwards) in single operation
- FR019: Generation shall extend into next month when executed at month-end with explicit transition messaging
- FR020: Current week (today falls within Monday-Sunday) shall be locked and preserved during regeneration
- FR021: Algorithm shall handle insufficient recipes gracefully by leaving empty meal slots (no minimum count enforced)
- FR022: Generation shall be non-deterministic, producing different arrangements each time to enable preference-based regeneration
- FR023: Regeneration shall require confirmation dialog to prevent accidental replacement of all future weeks
- FR024: Algorithm shall ensure main courses are unique across all weeks until exhausted, then leave slots empty
- FR025: Appetizers and desserts may repeat after all are used once; accompaniments may repeat freely
- FR026: Algorithm shall consider dietary restrictions, cuisine variety preferences, and complexity spacing
- FR027: System shall complete 5-week generation in <5 seconds (P95 performance)
- FR028: Main courses with accompaniment preference shall be paired with compatible accompaniment recipes
- FR029: Each week's meal plan shall create and store complete snapshots/copies of all referenced recipes at generation time

**Calendar & Visualization**

- FR030: System shall display month-based calendar view with week navigation showing 3 courses per day (appetizer, main, dessert)
- FR031: Free tier shall view only first generated week; other weeks shall display "Upgrade to unlock" placeholders
- FR032: Premium tier shall view all generated weeks with full navigation
- FR033: Current week shall display visual badge/lock icon with tooltip explaining preservation
- FR034: Empty meal slots shall display "Browse community recipes" link to community page

**Shopping Lists**

- FR035: System shall generate separate shopping lists for each week with ingredient grouping and quantity optimization
- FR036: Shopping lists shall include ingredients from both main recipes and paired accompaniments
- FR037: Shopping lists shall be accessible for current and all future weeks

**Notifications & Reminders**

- FR038: System shall send morning notifications (8 AM daily) for advance preparation tasks based on recipe requirements

**Dashboard & Landing Page**

- FR039: Home route (/) shall dynamically route authenticated users to Dashboard, unauthenticated users to SEO-optimized landing page
- FR040: Landing page shall include hero section, key features showcase, how-it-works (3-step), example screenshots, pricing preview, testimonials, and CTAs
- FR041: Dashboard shall display nearest day's recipes and preparation tasks from generated meal plan with generate/regenerate button
- FR042: Free tier dashboard shall show nearest day only if it falls within accessible first week; otherwise show upgrade prompt
- FR043: Premium dashboard shall always show nearest day from any generated week
- FR044: Empty state dashboard shall display onboarding guide explaining meal planning workflow

**Admin Panel**

- FR045: Admin panel shall provide user management (view, edit, suspend/activate accounts, manage premium bypass flags)
- FR046: Admin panel shall provide contact form inbox (view messages, mark read/resolved, search/filter)
- FR047: Suspended users shall not be able to log in; their meal plans become inaccessible and shared recipes hidden
- FR048: All suspended user data shall be preserved for potential reactivation

**Contact & Support**

- FR049: System shall provide public contact form with fields for name, email, subject, and message
- FR050: System shall send email notifications to admin(s) when new contact form messages are submitted

**Freemium Access Control**

- FR051: System shall support premium bypass configuration via global config file (entire environment) or per-user flag (selective access)
- FR052: Free tier shall allow unlimited regenerations but restrict visibility to first generated week only
- FR053: Premium tier shall provide full visibility across all generated weeks and unlimited recipe favorites

### Non-Functional Requirements

- NFR001: System shall achieve <3 second page load times on mobile devices
- NFR002: System shall complete 5-week meal plan generation in <5 seconds (P95 latency)
- NFR003: System shall maintain <0.1% error rate for meal plan generation operations
- NFR004: System shall provide offline recipe access capability
- NFR005: System shall be fully mobile-responsive with touch-optimized interface for kitchen use
- NFR006: System shall support modern mobile browsers (iOS Safari, Android Chrome) with installable PWA experience
- NFR007: System shall implement OWASP security standards for all security-related features
- NFR008: System shall encrypt user data and comply with GDPR requirements
- NFR009: System shall be SEO-optimized with proper meta tags, structured data (Schema.org), semantic HTML for organic acquisition
- NFR010: System shall avoid vendor lock-in through open standards and portable solutions
- NFR011: System shall implement comprehensive design system with consistent components, spacing, typography, and color palette
- NFR012: System shall collect only minimal anonymous analytics (aggregate, anonymized metrics) with privacy-first approach
- NFR013: System shall validate recipe import files against malicious content (script injection, oversized payloads, DoS attacks)
- NFR014: System shall use streaming parser for large recipe import files (up to 10MB)

---

## User Journeys

### Journey 1: New User Onboarding & First Meal Plan Generation

**Persona:** Sarah, busy professional with family, wants meal variety without planning stress

**Starting Point:** Discovers imkitchen through search, visits landing page

**Journey Steps:**

1. **Discovery** - Sarah lands on SEO-optimized landing page, reads value proposition about month-based meal planning with accompaniments
2. **Registration** - Creates account with email/password authentication
3. **Profile Setup** - Configures dietary restrictions (gluten-free), cuisine variety preference (0.7 default), and household size (4)
4. **Recipe Import** - Bulk imports 25 recipes from JSON file exported from previous app (drag-and-drop interface shows real-time progress)
5. **Recipe Review** - Reviews import summary: 23 successful, 2 failed (missing required fields), duplicates blocked
6. **Community Discovery** - Browses community recipes, favorites 8 additional recipes (total: 31 favorites)
7. **First Generation** - Clicks "Generate Meal Plan" button, system generates 4 weeks of meals in 3 seconds
8. **Free Tier Experience** - Views first week with 3 courses per day, sees "Upgrade to unlock" placeholders on remaining weeks
9. **Calendar Exploration** - Reviews generated meals, notices main courses paired with appropriate accompaniments (curry with rice)
10. **Shopping List** - Generates shopping list for first week, sees ingredients grouped by category
11. **Decision Point** - Satisfied with first week, wants to see all 4 weeks for better planning → upgrades to premium ($9.99/month)
12. **Premium Experience** - Now sees all 4 weeks, navigates full calendar, reviews entire month's meal plan

**Success Outcome:** Sarah has a complete month of meals planned with accompaniments in <10 minutes, upgraded to premium for full visibility

**Pain Points Addressed:**
- Recipe import eliminated manual entry of 25 recipes
- Month-based generation provided immediate planning visibility
- Accompaniment pairing removed meal composition complexity
- First-week preview demonstrated value before payment

---

### Journey 2: Weekly Meal Execution & Adaptive Regeneration

**Persona:** James, home cooking enthusiast, using imkitchen for ongoing meal planning

**Starting Point:** Currently in Week 2 of previously generated meal plan

**Journey Steps:**

1. **Daily Dashboard** - Opens app on Tuesday morning, dashboard shows today's nearest meal (appetizer, main, dessert) with prep tasks
2. **Advance Prep Reminder** - Receives 8 AM notification: "Remember to marinate chicken for tonight's dinner"
3. **Meal Execution** - Follows recipe instructions, successfully prepares meal with auto-paired rice accompaniment
4. **Mid-Week Disruption** - Thursday dinner plans change unexpectedly (family event), won't cook that night
5. **Week Lock Protection** - Current week (Week 2) is locked, preserving existing meals; future weeks remain flexible
6. **Shopping List Use** - Generates shopping list for Week 3 (upcoming week), sees all ingredients including accompaniments
7. **Grocery Shopping** - Uses shopping list at store with ingredients grouped by category, quantities optimized
8. **Week Transition** - Monday of Week 3 arrives, system automatically locks Week 3 as "current week"
9. **Month-End Scenario** - Reaches last week of October, wants more meal plans
10. **Regeneration** - Clicks "Regenerate" button, confirmation modal warns "This will replace all future weeks. Continue?"
11. **Extended Generation** - System generates 5 new weeks: remaining October days + 4 weeks into November with transition message
12. **New Rotation** - Sees fresh meal arrangements (non-deterministic), different cuisine distribution, no repeated main courses

**Success Outcome:** James maintains continuous meal planning across month boundaries with protected current week and flexible future weeks

**Pain Points Addressed:**
- Current week locking preserved in-progress meals during disruption
- Shopping lists enabled efficient bulk grocery shopping
- Seamless month transition prevented planning gaps
- Non-deterministic regeneration allowed preference-based re-planning

---

### Journey 3: Community Engagement & Recipe Contribution

**Persona:** Maria, passionate home cook, wants to share recipes and discover community favorites

**Starting Point:** Existing premium user with 45 favorited recipes

**Journey Steps:**

1. **Recipe Creation** - Creates new Thai curry recipe (Main Course type), explicitly sets dietary restrictions and cuisine type
2. **Accompaniment Configuration** - Enables `accepts_accompaniment=true`, specifies rice as compatible accompaniment
3. **Community Sharing** - Publishes recipe publicly to community, available to all users (free and premium)
4. **Recipe Discovery** - Browses community recipes filtered by "Appetizer" type for week's empty slots
5. **Rating Engagement** - Rates recently tried community recipe 5 stars, writes review: "Perfect weeknight meal!"
6. **Favorite Management** - Favorites new community recipe, adding to meal plan rotation pool (premium = unlimited favorites)
7. **Meal Generation Impact** - Next generation includes newly favorited community recipes in rotation
8. **Recipe Popularity** - Maria's shared Thai curry gains ratings from other users, rises in community search results
9. **Recipe Deletion Scenario** - Original creator of a favorited recipe deletes it → automatically removed from Maria's favorites
10. **Adaptation** - Maria notices missing favorite during next generation, browses community for replacement
11. **Quality Filter** - Low-rated recipes (< 3 stars) buried in search results due to community ratings
12. **Accompaniment Sharing** - Creates and shares specialized rice pilaf recipe (Accompaniment type) for community use

**Success Outcome:** Maria contributes to community ecosystem while discovering high-quality recipes through social rating system

**Pain Points Addressed:**
- Four recipe types (appetizer, main, dessert, accompaniment) enabled complete meal composition sharing
- Community ratings provided quality filter without pre-approval moderation
- Automatic deletion handling prevented broken favorites
- Premium unlimited favorites removed constraints for active community participants

---

## UX Design Principles

1. **Mobile-First Kitchen Optimization** - Touch-optimized interface designed for use in kitchen environments with larger tap targets, clear typography, and offline recipe access
2. **Instant Value Demonstration** - Month-based generation shows immediate results (all weeks in <5 seconds) to build trust through demonstrated time savings
3. **Progressive Disclosure** - Freemium model reveals first-week value immediately while naturally introducing premium benefits through locked week placeholders
4. **Friction Reduction** - One-click generation eliminates complex configuration; system handles timing, accompaniments, and preferences automatically
5. **Trust Through Transparency** - Explicit confirmation dialogs for destructive actions (regeneration), clear week locking indicators, and visible preference impacts
6. **Realistic Meal Composition** - Accompaniment pairing reflects how people actually eat (curry with rice, pasta with sauce), not isolated dishes
7. **Graceful Degradation** - Empty meal slots handled elegantly with community recipe suggestions rather than blocking generation
8. **Accessibility Priority** - Screen reader support, keyboard navigation, and semantic HTML for inclusive experience

---

## User Interface Design Goals

**Platform & Screens:**
- Progressive Web App (PWA) with installable experience for iOS Safari and Android Chrome
- Core screens: Landing Page, Dashboard, Meal Calendar, Recipe Management, Community Browse, Shopping Lists, User Profile, Admin Panel

**Design System:**
- Comprehensive design system with consistent components, spacing (8px grid), typography scale, and cohesive color palette
- Unified navigation patterns ensuring immersive application experience across all screens
- Reusable component library for forms, cards, modals, buttons, and data tables

**Key Interaction Patterns:**
- Week carousel navigation for mobile calendar browsing (swipe gestures)
- Drag-and-drop recipe import with real-time progress feedback
- Modal confirmations for destructive actions (regeneration, deletion)
- Inline upgrade prompts on locked weeks and favorite limits (non-intrusive)
- Responsive card layouts for recipe browsing with quick-action buttons

**Visual Feedback:**
- Loading states for generation operations (<5 second completion)
- Real-time progress indicators for bulk operations (import, generation)
- Badge/lock icons for current week with tooltips
- Empty state illustrations with clear CTAs ("Browse community recipes")
- Success/error toast notifications for user actions

**SEO & Performance:**
- Landing page optimized with meta tags, Schema.org structured data, semantic HTML
- <3 second page load times on mobile devices
- Lazy loading for recipe images and calendar weeks
- Offline-first architecture with service worker caching

---

## Visual Design References

All requirements have been prototyped in static HTML mockups located in `/mockups/`. These mockups provide visual validation of requirements and serve as reference for implementation.

### Mockup-to-Requirement Mapping

| Mockup File | Mapped Requirements | Description |
|------------|---------------------|-------------|
| **Public Pages** |
| `index.html` | FR039, FR040 | SEO-optimized landing page with hero, features showcase, how-it-works, pricing preview, testimonials |
| `login.html` | FR001 | User authentication with JWT cookie-based login, demo account quick access |
| `register.html` | FR001, FR002 | User registration with dietary restrictions, household size, cuisine variety preferences |
| `contact.html` | FR049, FR050 | Public contact form with subject categories and FAQ section |
| **Authenticated User Pages - Free Tier** |
| `dashboard-free.html` | FR041, FR042 | Free tier dashboard showing nearest day (Week 1 only), 8/10 favorites counter, premium upsell |
| `calendar-free.html` | FR030, FR031, FR033, FR034 | Month calendar with Week 1 visible, Weeks 2-5 locked with upgrade prompts, current week lock indicator |
| **Authenticated User Pages - Premium Tier** |
| `dashboard-premium.html` | FR043 | Premium dashboard with nearest day from any week, unlimited favorites (47 shown), no upgrade prompts |
| `calendar-premium.html` | FR032 | Full month calendar with all 5 weeks accessible, week navigation tabs |
| **Recipe Management** |
| `recipe-create.html` | FR004, FR005, FR006 | Recipe creation form with 4 types (appetizer, main, dessert, accompaniment), explicit field configuration, main course accepts_accompaniment toggle |
| `recipes-list.html` | FR004, FR012, FR013 | Recipe library with filters, stats cards showing favorites limit (8/10 for free tier), all 4 recipe types color-coded |
| `recipe-detail.html` | FR017 | Full recipe view with ingredients checklist, instructions, rating summary (4.8 stars, 23 reviews), suggested accompaniments sidebar |
| `import.html` | FR007, FR008, FR009, FR010, FR011 | Bulk JSON import with drag-drop, schema documentation, real-time progress (imported/failed/remaining), duplicate detection |
| **Community & Shopping** |
| `community.html` | FR015, FR017 | Community recipe browse with stats (2,547 recipes), trending section, filters (type/cuisine/dietary), rating system |
| `shopping-list.html` | FR035, FR036, FR037 | Per-week shopping lists organized by category (Proteins, Vegetables, Dairy, Bakery, Pantry), quantity aggregation |
| **Settings & Support** |
| `profile.html` | FR002, FR003, FR051 | User profile with dietary restrictions, cuisine variety slider (0.7 default), subscription management, notification toggles |
| **Admin Panel** |
| `admin-users.html` | FR045, FR047, FR048 | User management with stats cards (2,547 total, 382 premium, 12 suspended), user actions (edit, suspend, reactivate, delete) |
| `admin-contact.html` | FR046, FR050 | Contact inbox with message stats (347 total, 12 unread), quick actions (mark read, resolve) |

### Freemium Model Demonstrations

The mockups demonstrate freemium restrictions at multiple touchpoints:

- **Week Visibility**: `calendar-free.html` shows Week 1 visible with Weeks 2-5 locked vs `calendar-premium.html` showing all weeks accessible
- **Favorites Limit**: `recipes-list.html` displays "8/10 favorites" warning for free tier with upgrade prompt
- **Dashboard Restrictions**: `dashboard-free.html` shows upgrade banners and limited access vs `dashboard-premium.html` with full access
- **Shopping Lists**: `shopping-list.html` demonstrates Week 1 accessible for free tier with locked weeks requiring upgrade

### Recipe Type Demonstrations

All four recipe types are color-coded consistently across mockups:

- **Appetizer** (Blue badges) - Shown in `recipe-create.html`, `recipes-list.html`, `calendar-premium.html`
- **Main Course** (Orange badges) - Featured with accompaniment pairing in `calendar-premium.html`, `recipe-detail.html`
- **Dessert** (Pink badges) - Displayed in daily meal slots across calendar mockups
- **Accompaniment** (Purple badges) - Shown paired with main courses in `recipe-detail.html` suggested sidebar

### User Flow Demonstrations

**New User Onboarding Flow:**
`index.html` → `register.html` → `dashboard-free.html` → `calendar-free.html` → `recipes-list.html` → `community.html`

**Recipe Management Flow:**
`recipes-list.html` → `recipe-create.html` (create) → `recipe-detail.html` (view) → `import.html` (bulk import)

**Meal Planning Flow:**
`dashboard-free.html` (generate) → `calendar-free.html` (view weeks) → `shopping-list.html` (grocery shopping)

**Community Engagement Flow:**
`community.html` (discover) → `recipe-detail.html` (rate/review) → `recipes-list.html` (favorite)

**Admin Management Flow:**
`admin-users.html` (user management) → `admin-contact.html` (support inbox)

### Next Steps

During implementation:
1. Convert HTML mockups to Askama templates in `templates/pages/`
2. Extract reusable components from mockups to `templates/components/`
3. Replace static dummy data with dynamic projections
4. Implement Twinspark reactive behaviors for forms and polling
5. Validate acceptance criteria against mockup demonstrations

---

## Epic List

**Epic 1: Foundation & User Management**
- Establish project infrastructure, user authentication, and profile management with dietary restrictions and preferences
- **Estimated Stories:** 6-8 stories
- **Key Deliverables:** Project setup, JWT authentication, user registration/login, profile CRUD, admin panel foundation

**Epic 2: Recipe Management & Import System**
- Enable users to create, manage, and bulk import recipes with four types (Appetizer, Main Course, Dessert, Accompaniment)
- **Estimated Stories:** 7-9 stories
- **Key Deliverables:** Recipe CRUD operations, JSON bulk import with validation, duplicate detection, real-time progress feedback, recipe favorites

**Epic 3: Core Meal Planning Engine**
- Implement month-based meal plan generation algorithm with intelligent accompaniment pairing, dietary filtering, and recipe rotation
- **Estimated Stories:** 8-10 stories
- **Key Deliverables:** Generation algorithm, preference-aware scheduling, accompaniment pairing logic, week locking, recipe snapshot system, non-deterministic rotation

**Epic 4: Calendar Visualization & Shopping Lists**
- Build mobile-responsive calendar interface with week navigation, meal visualization, and per-week shopping list generation
- **Estimated Stories:** 6-8 stories
- **Key Deliverables:** Month calendar UI, week carousel navigation, dashboard with nearest day, shopping list generation with ingredient grouping, empty state handling

**Epic 5: Community Features & Freemium Access**
- Enable recipe sharing, rating system, freemium access controls, and premium tier restrictions with upgrade flows
- **Estimated Stories:** 7-9 stories
- **Key Deliverables:** Recipe sharing/privacy, rating and review system, freemium visibility restrictions (first-week-only), upgrade prompts, premium bypass configuration, admin user management

**Epic 6: Notifications & Landing Page**
- Implement advance preparation reminders, SEO-optimized landing page, and contact form with admin notifications
- **Estimated Stories:** 4-6 stories
- **Key Deliverables:** 8 AM prep notifications, landing page with hero/features/pricing, contact form, email notifications to admins

**Total Estimated Stories:** 38-50 stories

> **Note:** Detailed epic breakdown with full story specifications is available in [epics.md](./epics.md)

---

## Out of Scope

**Deferred to Post-MVP (Phase 2):**

- **Machine Learning Features** - ML-powered notification timing based on cooking speed patterns, predictive recipe recommendations, adaptive difficulty adjustment
- **Grocery Store Integrations** - One-tap ordering through partner services, real-time inventory checking, automatic price comparison
- **Advanced Social Features** - Community contests, chef profiles, recipe collections, public/private sharing settings beyond basic sharing/rating
- **Smart Kitchen Devices** - Connected appliance integration, automated inventory tracking, IoT temperature monitoring
- **Video Guidance** - Step-by-step video instructions, AR cooking assistance, live cooking sessions
- **Extended Planning Factors** - Weather-based suggestions, family calendar integration, energy level tracking, seasonal ingredient preferences
- **Recipe Collections & Templates** - Curated themed collections, pre-built diet-specific meal plan templates (Keto, Mediterranean, etc.)
- **Advanced Regeneration** - Partial week regeneration (specific days within locked week), constraint relaxation suggestions when insufficient recipes
- **Payment Processing** - Credit card payment gateway integration, subscription billing, auto-renewal (premium tier access control included, payment flow deferred)

**Explicitly Out of Scope:**

- **Multi-household accounts** - Shared family accounts with multiple user access
- **Meal kit delivery service** - First-party meal kit fulfillment
- **Nutritional analysis** - Detailed macro/micronutrient tracking and goals
- **Recipe scaling** - Automatic ingredient adjustment for different serving sizes
- **Meal swapping** - Drag-and-drop meal rearrangement within calendar
- **Custom meal slots** - User-defined meal types beyond appetizer/main/dessert
- **International localization** - Multi-language support, regional ingredient variations
- **Social messaging** - Direct messaging between users, community forums
- **Recipe versioning** - Historical tracking of recipe edits and changes

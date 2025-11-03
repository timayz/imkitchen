# imkitchen - Epic Breakdown

**Author:** Jonathan
**Date:** 2025-10-31
**Project Level:** 3
**Target Scale:** Comprehensive product with freemium model, community features, and intelligent meal planning automation

---

## Overview

This document provides the detailed epic breakdown for imkitchen, expanding on the high-level epic list in the [PRD](./PRD.md).

Each epic includes:

- Expanded goal and value proposition
- Complete story breakdown with user stories
- Acceptance criteria for each story
- Story sequencing and dependencies

**Epic Sequencing Principles:**

- Epic 1 establishes foundational infrastructure and initial functionality
- Subsequent epics build progressively, each delivering significant end-to-end value
- Stories within epics are vertically sliced and sequentially ordered
- No forward dependencies - each story builds only on previous work

---

## Epic 1: Foundation & User Management

**Goal:** Establish foundational project infrastructure with user authentication, profile management, and admin capabilities to enable secure user operations and preference storage for meal planning algorithm.

**Value Delivery:** Users can register, authenticate, configure dietary preferences, and admins can manage the platform—providing the essential foundation for all subsequent features.

**Visual Mockup References:**
- `mockups/login.html` - JWT cookie-based authentication
- `mockups/register.html` - Registration with dietary preferences
- `mockups/profile.html` - Profile management and settings
- `mockups/contact.html` - Public contact form
- `mockups/admin-users.html` - User management panel
- `mockups/admin-contact.html` - Contact inbox

### Stories

**Story 1.1: Project Infrastructure Setup**

As a developer,
I want a properly configured Rust workspace with evento, axum, and database setup,
So that the project foundation supports event-driven architecture and web server capabilities.

**Acceptance Criteria:**
1. Workspace Cargo.toml configured with all required dependencies (evento 1.5+, axum 0.8+, sqlx, askama, etc.)
2. CLI commands implemented: serve, migrate, reset
3. Configuration system using TOML files (config/default.toml committed, config/dev.toml in .gitignore)
4. Separate databases created: write DB (evento), read DB (queries), validation DB
5. Migration structure created: migrations/queries/ and migrations/validation/
6. Playwright configured with example E2E test (tests/e2e/ directory created)
7. Rust test helper functions created for database setup (using sqlx::migrate! and evento::sql_migrator)
8. Project compiles without errors and passes clippy/fmt checks

**Prerequisites:** None (foundational story)

---

**Story 1.2: User Registration and Authentication**

As a new user,
I want to register an account with email and password,
So that I can access the meal planning platform securely.

**Acceptance Criteria:**
1. User aggregate created with evento: UserRegistered, UserLoggedIn events
2. Registration command validates email format and password requirements
3. JWT cookie-based authentication implemented using evento metadata pattern
4. Login route returns JWT token stored in secure HTTP-only cookie
5. Protected routes verify JWT token and extract user_id
6. Registration/login forms rendered with Askama templates
7. User projection table created in queries DB with email, hashed_password, created_at
8. Tests verify registration, login, and protected route access

**Prerequisites:** Story 1.1

---

**Story 1.3: User Profile Management**

As a logged-in user,
I want to configure my dietary restrictions and preferences,
So that meal plan generation respects my dietary needs and cuisine preferences.

**Acceptance Criteria:**
1. UserProfileUpdated event stores dietary restrictions (array), complexity preferences, cuisine_variety_weight (default 0.7), household_size
2. Profile update command accepts input struct with validation
3. Profile page displays current preferences with edit form
4. Query handler projects profile data to user_profiles table
5. Profile data accessible via query function for meal planning algorithm
6. Twinspark form submission with optimistic UI update
7. Tests verify profile creation, update, and query retrieval

**Prerequisites:** Story 1.2

---

**Story 1.4: Admin User Management**

As an admin,
I want to view and manage user accounts,
So that I can suspend problematic users and manage premium bypass flags.

**Acceptance Criteria:**
1. is_admin flag added to user aggregate and projection
2. Admin panel route protected by admin-only middleware
3. Admin can view list of all users with pagination
4. Admin can suspend/activate user accounts (UserSuspended, UserActivated events)
5. Suspended users cannot log in (authentication check)
6. Suspended users' shared recipes hidden from community view
7. Admin can toggle premium_bypass flag per user (for demo/testing accounts)
8. Tests verify admin authentication, user suspension, and reactivation

**Prerequisites:** Story 1.2, Story 1.3

---

**Story 1.5: Premium Bypass Configuration**

As a developer,
I want to configure premium bypass globally or per-user,
So that development, staging, and demo accounts can bypass premium restrictions.

**Acceptance Criteria:**
1. Global premium bypass setting in config/default.toml (boolean)
2. Per-user premium_bypass flag in user profile (boolean)
3. Access control logic checks: global config OR user flag OR active premium subscription
4. Tests verify bypass behavior in both global and per-user scenarios
5. Documentation added to CLAUDE.md explaining bypass configuration

**Prerequisites:** Story 1.3, Story 1.4

---

**Story 1.6: Contact Form and Admin Notifications**

As a visitor,
I want to submit questions or feedback through a contact form,
So that I can reach the platform administrators without needing an account.

**Acceptance Criteria:**
1. Public contact form route (no authentication required) with fields: name, email, subject, message
2. ContactFormSubmitted event stores submission data with timestamp
3. Query handler projects submissions to contact_messages table
4. Admin panel displays contact form inbox with read/resolved status
5. Email notification sent to configured admin email(s) on new submission
6. Admin can mark messages as read/resolved and filter by status
7. Tests verify form submission, projection, and admin access

**Prerequisites:** Story 1.4

---

## Epic 2: Recipe Management & Import System

**Goal:** Enable users to create, manage, and bulk import recipes with four types (Appetizer, Main Course, Dessert, Accompaniment), supporting explicit field configuration and community sharing foundation.

**Value Delivery:** Users can build their recipe library through manual creation or bulk JSON import, setting dietary restrictions and accompaniment preferences explicitly for meal planning algorithm consumption.

**Visual Mockup References:**
- `mockups/recipe-create.html` - Recipe creation form with all 4 types (appetizer, main, dessert, accompaniment), explicit field configuration, main course accepts_accompaniment toggle
- `mockups/recipes-list.html` - Recipe library with filters, stats cards, favorites counter (8/10 for free tier)
- `mockups/recipe-detail.html` - Full recipe view with ingredients, instructions, rating summary
- `mockups/import.html` - Bulk JSON import with drag-drop, schema docs, real-time progress (imported/failed/remaining counts)

### Stories

**Story 2.1: Recipe Creation (Four Types)**

As a user,
I want to create recipes with explicit type selection (Appetizer, Main Course, Dessert, Accompaniment),
So that I can build my recipe library for meal planning.

**Acceptance Criteria:**
1. Recipe aggregate with RecipeCreated event including: recipe_type (enum), name, ingredients, instructions, dietary_restrictions, cuisine_type, complexity, advance_prep_text
2. Main courses include accepts_accompaniment field (defaults to false)
3. Recipe creation command validates required fields using validator crate
4. Recipe creation form with type selector and conditional fields (accompaniment field shown only for Main Course)
5. Recipe projection table stores all recipe data for querying
6. User can view their recipe list filtered by type
7. Tests verify recipe creation for all four types with validation

**Prerequisites:** Story 1.2

---

**Story 2.2: Recipe Editing and Deletion**

As a user,
I want to edit and delete my recipes,
So that I can maintain my recipe library accurately.

**Acceptance Criteria:**
1. RecipeUpdated event stores changed fields with evento::save pattern
2. RecipeDeleted event marks recipe as deleted with soft delete timestamp
3. Recipe edit form pre-populated with current data
4. Deletion requires confirmation modal
5. Deleted recipes removed from user's favorites automatically
6. Deleted shared recipes hidden from community immediately
7. Query handlers update projections for edit/delete events
8. Tests verify edit, delete, and cascade deletion of favorites

**Prerequisites:** Story 2.1

---

**Story 2.3: Recipe Favorites System**

As a user,
I want to favorite recipes (my own and community recipes),
So that I can designate which recipes should be included in meal plan generation.

**Acceptance Criteria:**
1. RecipeFavorited and RecipeUnfavorited events store user-recipe relationship
2. Free tier users limited to maximum 10 favorited recipes
3. Attempting to exceed 10 favorites shows upgrade modal (no unfavoriting option)
4. Premium tier users have unlimited favorites
5. Recipe cards show favorite button with toggle state
6. User profile displays favorited recipes list
7. When recipe owner deletes recipe, all favorites automatically removed (no notifications)
8. Query projection tracks favorite_count per recipe for community sorting
9. Tests verify favorite limits, premium bypass, and cascade deletion

**Prerequisites:** Story 2.1, Story 1.5

---

**Story 2.4: Recipe JSON Import - File Upload & Validation**

As a user,
I want to bulk import recipes from JSON files via drag-and-drop,
So that I can quickly populate my recipe library from exported data.

**Acceptance Criteria:**
1. Recipe import route accepts multiple JSON files (max 10MB per file, 20 files per batch)
2. Drag-and-drop UI with file picker fallback
3. Validation against recipe schema: all required AND optional fields must be present and valid
4. Malicious content detection (script injection, oversized payloads)
5. Streaming parser for large files to prevent memory issues
6. Invalid recipes skipped with detailed error messages collected
7. Duplicate detection blocks recipes with matching name or similar ingredients
8. Imported recipes stored as private (not shared) by default
9. Tests verify validation, malicious content rejection, and duplicate blocking

**Prerequisites:** Story 2.1

---

**Story 2.5: Recipe JSON Import - Real-Time Progress & Summary**

As a user,
I want to see real-time progress during recipe import,
So that I understand the import status and can review results.

**Acceptance Criteria:**
1. Real-time progress display: "Imported X recipes, Y failed, Z remaining..."
2. Twinspark polling updates progress without blocking UI
3. Summary report after completion: success count, failed count, duplicate count
4. Detailed error list for failed recipes (missing fields, validation errors)
5. Success message with link to recipe library when complete
6. Progress state cleared on page reload (no persistent history)
7. Tests verify progress updates and summary accuracy

**Prerequisites:** Story 2.4

---

**Story 2.6: JSON Schema Documentation**

As a third-party developer,
I want publicly accessible versioned JSON schema documentation,
So that I can build tools that export recipes compatible with imkitchen.

**Acceptance Criteria:**
1. JSON schema document created with all recipe fields and types
2. Schema versioned (v1.0) and published at public URL (/api/schema/recipe/v1.0)
3. Documentation page explains schema fields, required vs optional, and example JSON
4. Schema matches HTML form validation exactly
5. Schema includes all four recipe types with type-specific fields
6. Tests verify schema endpoint accessibility

**Prerequisites:** Story 2.4

---

**Story 2.7: Recipe Snapshot System for Meal Plans**

As a user,
I want my generated meal plans to preserve original recipe data,
So that I can access historical meal plans even if the recipe owner modifies or deletes their recipe.

**Acceptance Criteria:**
1. Meal plan generation creates complete snapshot/copy of each referenced recipe
2. Snapshots stored in meal_plan_recipes table with week_id foreign key
3. Snapshots include all recipe fields: name, ingredients, instructions, dietary_restrictions, etc.
4. Calendar displays recipe data from snapshot, not original recipe
5. Recipe modifications by owner don't affect existing meal plan snapshots
6. Recipe deletion by owner doesn't affect existing meal plan snapshots
7. Tests verify snapshot creation, isolation from original recipe changes

**Prerequisites:** Story 2.1

---

## Epic 3: Core Meal Planning Engine

**Goal:** Implement month-based meal plan generation algorithm with intelligent accompaniment pairing, dietary filtering, recipe rotation, and week locking to deliver automated meal planning that respects user preferences.

**Value Delivery:** Users generate complete month-based meal plans (all future weeks of current month) in <5 seconds with automatic accompaniment pairing, dietary restriction compliance, and intelligent cuisine variety distribution.

**Visual Mockup References:**
- `mockups/dashboard-free.html` - Generation trigger button, nearest day display (Week 1 only)
- `mockups/dashboard-premium.html` - Generation trigger with full access to all weeks
- `mockups/calendar-free.html` - Week 1 visible with meals (appetizer, main, dessert per day), current week lock indicator
- `mockups/calendar-premium.html` - All 5 weeks with meal slots, week navigation, accompaniment pairing visible

### Stories

**Story 3.1: Basic Meal Plan Generation Algorithm**

As a user,
I want to generate meal plans for all future weeks of the current month,
So that I can have my meals automatically scheduled without manual planning.

**Acceptance Criteria:**
1. MealPlanGenerated event stores week data with 7 days × 3 courses (appetizer, main, dessert)
2. Generation command calculates future weeks from next Monday until end of current month
3. Algorithm randomly selects recipes from user's favorited recipes for each meal slot
4. Main courses must be unique across all generated weeks until all favorites exhausted
5. Appetizers/desserts can repeat after all are used once
6. Empty slots left when insufficient favorited recipes available
7. Generation completes in <5 seconds for 5 weeks (P95)
8. Tests verify week calculation, recipe selection, and performance

**Prerequisites:** Story 2.3

---

**Story 3.2: Month Transition Handling**

As a user,
I want generation at month-end to extend into next month,
So that I always have continuous meal plans without manual month switching.

**Acceptance Criteria:**
1. Generation logic detects when current month has <4 future weeks remaining
2. Extends generation into next month to ensure minimum 4 weeks generated
3. Explicit transition message displayed: "Generating weeks for rest of October + first 2 weeks of November"
4. Week metadata includes month and year for accurate calendar display
5. Tests verify month transition calculation and messaging

**Prerequisites:** Story 3.1

---

**Story 3.3: Current Week Locking**

As a user,
I want my current week (today falls within Monday-Sunday) to be locked,
So that in-progress meals are preserved when I regenerate future weeks.

**Acceptance Criteria:**
1. Generation algorithm skips current week if it exists from previous generation
2. Current week determination: today's date falls within Monday-Sunday range
3. Visual lock icon/badge displayed on current week in calendar
4. Tooltip explains: "Current Week - Won't be regenerated"
5. Regeneration only affects future weeks (week start date > current Sunday)
6. Tests verify current week preservation across regenerations

**Prerequisites:** Story 3.1

---

**Story 3.4: Regeneration with Confirmation**

As a user,
I want to regenerate all future weeks with a confirmation dialog,
So that I don't accidentally replace my meal plans.

**Acceptance Criteria:**
1. "Regenerate" button displays confirmation modal: "This will replace all future weeks. Continue?"
2. Confirmation required to proceed with regeneration
3. AllFutureWeeksRegenerated event replaces all non-locked weeks
4. Non-deterministic generation produces different recipe arrangements each time
5. Query handlers delete future week projections and insert new ones
6. Tests verify confirmation requirement and regeneration behavior

**Prerequisites:** Story 3.3

---

**Story 3.5: Dietary Restriction Filtering**

As a user,
I want the algorithm to respect my dietary restrictions,
So that generated meal plans only include recipes I can actually eat.

**Acceptance Criteria:**
1. Algorithm filters favorited recipes by user's dietary restrictions from profile
2. Recipe excluded if ANY of its dietary_restrictions conflicts with user restrictions
3. 95% or higher of generated meals meet user's dietary restrictions
4. Empty slots left if insufficient restriction-compliant recipes available
5. Tests verify restriction filtering with various dietary combinations

**Prerequisites:** Story 3.1, Story 1.3

---

**Story 3.6: Cuisine Variety Scheduling**

As a user,
I want the algorithm to distribute cuisines evenly across weeks,
So that I don't eat the same cuisine type too frequently.

**Acceptance Criteria:**
1. Algorithm tracks cuisine frequency across all generated weeks
2. Cuisine variety weight (default 0.7) influences cuisine distribution
3. Higher weight (closer to 1.0) = maximum variety, lower weight = repeat cuisines more
4. Algorithm prioritizes less-recently-used cuisines when selecting recipes
5. Tests verify cuisine distribution respects variety weight configuration

**Prerequisites:** Story 3.5

---

**Story 3.7: Accompaniment Pairing System**

As a user,
I want main courses automatically paired with compatible accompaniment recipes,
So that meals are complete with realistic composition (curry with rice, pasta with sauce).

**Acceptance Criteria:**
1. Main courses with accepts_accompaniment=true trigger pairing logic
2. Algorithm matches main course with compatible accompaniment from favorited accompaniment recipes
3. Accompaniment recipes can repeat freely (not subject to uniqueness constraint)
4. Paired accompaniments stored in meal slot with main course reference
5. 85% or higher success rate for pairing when user has compatible accompaniment recipes favorited
6. Empty accompaniment slot if no compatible accompaniment available
7. Tests verify pairing logic, compatibility matching, and success rate

**Prerequisites:** Story 3.6

---

**Story 3.8: Recipe Snapshot Integration**

As a user,
I want each generated meal plan to capture complete recipe data at generation time,
So that my historical meal plans remain intact even if recipes are modified or deleted.

**Acceptance Criteria:**
1. Generation algorithm creates snapshot of each selected recipe during generation
2. Snapshots include all recipe fields plus accompaniment recipe snapshots if paired
3. MealPlanGenerated event includes recipe snapshot IDs
4. Query handler stores snapshots in meal_plan_recipes table
5. Calendar queries load recipe data from snapshots, not original recipes
6. Tests verify snapshot creation, storage, and isolation

**Prerequisites:** Story 3.7, Story 2.7

---

**Story 3.9: Empty Slot Handling**

As a user,
I want the algorithm to gracefully handle insufficient recipes,
So that I can still generate plans even with a small recipe library.

**Acceptance Criteria:**
1. Algorithm leaves meal slots empty when insufficient favorited recipes available
2. No minimum recipe count enforced (can generate with 1 favorite recipe)
3. Empty slots don't block generation or throw errors
4. Empty slot metadata stored in meal plan for UI display
5. Tests verify generation with 0, 1, 5, and 50 favorited recipes

**Prerequisites:** Story 3.1

---

**Story 3.10: Algorithm Performance Optimization**

As a developer,
I want the meal planning algorithm to complete in <5 seconds,
So that users experience instant gratification and trust the system.

**Acceptance Criteria:**
1. Algorithm completes 5-week generation in <5 seconds (P95 latency)
2. Database queries optimized with indexes on favorited recipes, dietary restrictions
3. Recipe selection logic uses efficient filtering and randomization
4. Performance tests verify P95, P99 latencies with 100+ favorited recipes
5. Algorithm scales linearly with recipe count (no exponential growth)

**Prerequisites:** Story 3.8

---

## Epic 4: Calendar Visualization & Shopping Lists

**Goal:** Build mobile-responsive meal calendar interface with week navigation, nearest-day dashboard, and per-week shopping list generation to provide users with clear visibility into generated meal plans and shopping needs.

**Value Delivery:** Users can view their month-based meal plans in a calendar interface, navigate between weeks on mobile, see today's meals on dashboard, and generate shopping lists with ingredient grouping for efficient grocery shopping.

**Visual Mockup References:**
- `mockups/calendar-free.html` - Month calendar with Week 1 visible, Weeks 2-5 locked with "Upgrade to unlock" placeholders
- `mockups/calendar-premium.html` - Full month calendar with week navigation tabs, all weeks accessible
- `mockups/dashboard-free.html` - Nearest day's meals with quick actions (only if in Week 1)
- `mockups/dashboard-premium.html` - Nearest day from any week with full access
- `mockups/shopping-list.html` - Per-week shopping lists organized by category (Proteins, Vegetables, Dairy, Bakery, Pantry), quantity aggregation

### Stories

**Story 4.1: Monthly Calendar View**

As a user,
I want to view my generated meal plans in a month-based calendar,
So that I can see all scheduled meals at a glance.

**Acceptance Criteria:**
1. Calendar displays one month at a time with week rows
2. Each day shows 3 course slots: appetizer, main, dessert
3. Recipe cards display recipe name, type icon, and preparation indicator
4. Empty slots show "Browse community recipes" link
5. Current week badge/lock icon displayed on locked week
6. Responsive layout adapts to mobile and desktop screens
7. Tests verify calendar rendering with generated meal plans

**Prerequisites:** Story 3.8

---

**Story 4.2: Week Carousel Navigation (Mobile)**

As a mobile user,
I want to swipe between weeks in the calendar,
So that I can easily navigate my meal plans on a touchscreen device.

**Acceptance Criteria:**
1. Mobile view displays one week at a time (7 days in carousel)
2. Swipe gestures navigate forward/backward between weeks
3. Week indicator shows current position (e.g., "Week 2 of 4")
4. Navigation arrows for non-touch devices
5. Smooth animations for week transitions
6. Tests verify swipe gesture handling and navigation

**Prerequisites:** Story 4.1

---

**Story 4.3: Dashboard with Nearest Day Display**

As a user,
I want to see today's (or nearest upcoming day's) meals on my dashboard,
So that I know what to prepare without navigating the calendar.

**Acceptance Criteria:**
1. Dashboard displays nearest day's meals (appetizer, main, dessert) with recipe details
2. Advance preparation tasks shown if today's meals require prep
3. "Generate Meal Plan" button shown if no meal plans exist
4. "Regenerate" button shown if meal plans exist
5. Empty state guide displayed when no meal plans generated
6. Tests verify nearest day calculation and display logic

**Prerequisites:** Story 4.1

---

**Story 4.4: Freemium Calendar Restrictions (First Week Only)**

As a free tier user,
I want to view only my first generated week,
So that I can experience meal planning value while understanding premium benefits.

**Acceptance Criteria:**
1. Free tier users can view first generated week fully (7 days, 3 courses each)
2. Weeks 2-N display "Upgrade to unlock" placeholder cards with upgrade CTA
3. Clicking locked week triggers upgrade modal
4. Premium tier users view all generated weeks without restrictions
5. Premium bypass configuration respected (global or per-user)
6. Tests verify free tier restrictions and premium access

**Prerequisites:** Story 4.3, Story 1.5

---

**Story 4.5: Dashboard Freemium Restrictions**

As a free tier user,
I want the dashboard to show nearest day only if within my accessible first week,
So that I understand the freemium limitations while seeing immediate value.

**Acceptance Criteria:**
1. Free tier dashboard shows nearest day only if it falls within first generated week
2. If nearest day is outside first week, show upgrade prompt: "Upgrade to see upcoming meals"
3. Premium tier dashboard always shows nearest day from any generated week
4. Upgrade prompt links to pricing/upgrade page
5. Tests verify dashboard display logic for free vs premium tiers

**Prerequisites:** Story 4.4

---

**Story 4.6: Shopping List Generation (Per Week)**

As a user,
I want to generate a shopping list for a specific week,
So that I can efficiently purchase all ingredients needed for that week's meals.

**Acceptance Criteria:**
1. Shopping list generation command for specified week_id
2. Aggregates ingredients from all recipes in week (appetizers, mains, desserts, accompaniments)
3. Groups ingredients by category (produce, dairy, meat, pantry, etc.)
4. Displays quantities with units (2 lbs chicken, 1 cup rice)
5. Quantity optimization combines duplicate ingredients across recipes
6. Shopping list displayed as printable/shareable format
7. Tests verify ingredient aggregation, grouping, and quantity calculation

**Prerequisites:** Story 4.1

---

**Story 4.7: Shopping List Access (Current + Future Weeks)**

As a user,
I want to generate shopping lists for current week and all future weeks,
So that I can plan my grocery shopping flexibly.

**Acceptance Criteria:**
1. Shopping list route accepts week_id parameter
2. Users can generate shopping lists for any non-past week
3. Free tier users can generate shopping lists for accessible first week only
4. Premium tier users can generate shopping lists for any generated week
5. Shopping list UI shows week selector dropdown
6. Tests verify access control for free vs premium tiers

**Prerequisites:** Story 4.6, Story 4.4

---

**Story 4.8: Empty Slot Community Recipe Links**

As a user,
I want empty meal slots to suggest browsing community recipes,
So that I can discover new recipes to fill gaps in my meal plan.

**Acceptance Criteria:**
1. Empty meal slots display "Browse community recipes" link
2. Link routes to community recipe page with appropriate filter (appetizer/main/dessert)
3. Community page shows relevant recipes based on slot type
4. Users can favorite recipes directly from community page
5. Tests verify link routing and filter application

**Prerequisites:** Story 4.1

---

## Epic 5: Community Features & Freemium Access

**Goal:** Enable recipe sharing, community rating system, freemium access controls, and admin user management to build a self-sustaining recipe ecosystem with quality filtering and premium conversion incentives.

**Value Delivery:** Users discover high-quality community recipes through ratings, share their own recipes publicly, experience freemium limitations that drive premium conversion, and admins maintain platform quality through user management.

**Visual Mockup References:**
- `mockups/community.html` - Community recipe browse with stats (2,547 recipes), trending section, filters (type/cuisine/dietary), rating system
- `mockups/recipe-detail.html` - Rating summary (4.8 stars, 23 reviews), write review form
- `mockups/recipes-list.html` - Favorites limit warning (8/10) with upgrade prompt for free tier
- `mockups/calendar-free.html` - Freemium week restrictions (Week 1 visible, others locked)
- `mockups/admin-users.html` - User management with suspend/reactivate actions
- `mockups/admin-contact.html` - Contact form inbox management

### Stories

**Story 5.1: Recipe Sharing (Public/Private)**

As a user,
I want to share my recipes publicly with the community,
So that other users can discover and favorite my recipes.

**Acceptance Criteria:**
1. Recipe aggregate includes is_shared field (boolean, defaults to false)
2. RecipeShared and RecipeUnshared events toggle public visibility
3. Recipe edit form includes "Share with community" toggle
4. Shared recipes appear in community browse page for all users (free and premium)
5. Recipe owner can unshare at any time
6. All four recipe types (appetizer, main, dessert, accompaniment) sharable
7. Tests verify sharing toggle, community visibility, and unsharing

**Prerequisites:** Story 2.1

---

**Story 5.2: Community Recipe Browse & Discovery**

As a user,
I want to browse community-shared recipes with filters,
So that I can discover new recipes to add to my favorites.

**Acceptance Criteria:**
1. Community recipe page displays all shared recipes (paginated)
2. Filters: recipe type (appetizer/main/dessert/accompaniment), cuisine type, dietary restrictions
3. Search by recipe name or ingredients
4. Sort by rating (highest first), newest, most favorited
5. Recipe cards show: name, type, rating, favorite count, owner name
6. Quick-favorite button on recipe cards
7. Tests verify filtering, search, and sorting logic

**Prerequisites:** Story 5.1, Story 2.3

---

**Story 5.3: Recipe Rating System**

As a user,
I want to rate and review community recipes I've tried,
So that I can provide feedback and help others discover quality recipes.

**Acceptance Criteria:**
1. RecipeRated event stores user_id, recipe_id, rating (1-5 stars), review text, timestamp
2. Users can rate any shared recipe (not their own)
3. Users can edit/delete their own ratings
4. Recipe detail page displays average rating and all reviews
5. Community browse page sorts by average rating
6. Low-rated recipes (< 3 stars) de-prioritized in search results
7. Tests verify rating creation, average calculation, and sorting

**Prerequisites:** Story 5.2

---

**Story 5.4: Freemium Favorite Limit (10 Max)**

As a free tier user,
I want to favorite up to 10 recipes,
So that I can try meal planning while understanding premium benefits.

**Acceptance Criteria:**
1. Free tier users can favorite maximum 10 recipes (own + community)
2. Attempting 11th favorite displays upgrade modal: "You've reached your 10 favorites limit. Upgrade to Premium for unlimited favorites"
3. Modal includes pricing info ($9.99/month or $59.94/year)
4. No unfavoriting option in modal (strong conversion incentive)
5. Premium tier users have unlimited favorites
6. Tests verify limit enforcement and premium bypass

**Prerequisites:** Story 2.3, Story 5.2

---

**Story 5.5: Upgrade Prompts (Multiple Touchpoints)**

As a product owner,
I want upgrade prompts displayed at strategic touchpoints,
So that free tier users are aware of premium benefits and conversion opportunities.

**Acceptance Criteria:**
1. Upgrade modal shown when clicking locked weeks in calendar (Story 4.4)
2. Upgrade prompt shown on dashboard if nearest day is outside accessible week (Story 4.5)
3. Upgrade modal shown when attempting 11th favorite (Story 5.4)
4. Upgrade prompts include: feature comparison, pricing, "Upgrade Now" CTA
5. Prompts are persistent but not intrusive (dismissible)
6. Tests verify modal triggering at each touchpoint

**Prerequisites:** Story 4.4, Story 4.5, Story 5.4

---

**Story 5.6: Premium Access Control Logic**

As a developer,
I want centralized premium access control logic,
So that premium features are consistently enforced across the application.

**Acceptance Criteria:**
1. Access control function checks: active premium subscription OR global bypass OR user bypass flag
2. Function used in: calendar week visibility, dashboard nearest day, shopping list access, favorite limits
3. Premium subscription status stored in user profile (is_premium_active boolean)
4. Tests verify access control in all premium-gated features
5. Documentation in CLAUDE.md explaining premium access patterns

**Prerequisites:** Story 1.5, Story 4.4, Story 4.5, Story 5.4

---

**Story 5.7: Admin User Management Panel**

As an admin,
I want to view, edit, and suspend user accounts,
So that I can maintain platform quality and manage problematic users.

**Acceptance Criteria:**
1. Admin panel displays user list with: email, registration date, is_admin, is_premium_active, is_suspended, favorite_count
2. Admin can suspend/activate users (suspension prevents login)
3. Suspended users' shared recipes hidden from community
4. Admin can toggle premium_bypass flag for demo accounts
5. Admin can view user's favorited recipes and generated meal plans
6. Search/filter users by email, status (active/suspended/premium)
7. Tests verify admin operations and authorization

**Prerequisites:** Story 1.4, Story 5.1

---

**Story 5.8: Admin Contact Form Inbox**

As an admin,
I want to view and manage contact form submissions,
So that I can respond to user inquiries and feedback.

**Acceptance Criteria:**
1. Admin panel contact inbox displays all submissions with: name, email, subject, message, timestamp, status (new/read/resolved)
2. Admin can mark messages as read or resolved
3. Search/filter by status, date range, email
4. Email notification sent to admin email on new submission
5. Tests verify inbox display, status updates, and filtering

**Prerequisites:** Story 1.6, Story 5.7

---

**Story 5.9: Recipe Deletion Impact (Favorites Cascade)**

As a recipe owner,
I want my recipe automatically removed from all users' favorites when I delete it,
So that users don't have broken favorites in their meal planning.

**Acceptance Criteria:**
1. Recipe deletion triggers cascade removal from all users' favorite lists
2. No notifications sent to users who had it favorited (silent removal)
3. No notifications sent to recipe owner about favorite count
4. Users discover missing favorite organically (empty slots in meal generation)
5. Tests verify cascade deletion and notification absence

**Prerequisites:** Story 2.2, Story 2.3, Story 5.1

---

## Epic 6: Notifications & Landing Page

**Goal:** Implement advance preparation reminders, SEO-optimized landing page with pricing information, and contact form to complete the user acquisition and meal execution experience.

**Value Delivery:** Users receive timely preparation reminders for upcoming meals, new visitors discover imkitchen through SEO-optimized landing page, and users can contact support through public form.

**Visual Mockup References:**
- `mockups/index.html` - SEO-optimized landing page with hero, features showcase, how-it-works (3 steps), pricing preview, testimonials
- `mockups/contact.html` - Public contact form with subject categories and FAQ section
- `mockups/profile.html` - Notification toggles (4 types: morning prep reminders, recipe updates, weekly summary, marketing emails)

### Stories

**Story 6.1: Advance Preparation Reminders (8 AM Daily)**

As a user,
I want to receive morning reminders for advance preparation tasks,
So that I remember to marinate, rise, or chill ingredients before cooking.

**Acceptance Criteria:**
1. Daily batch job runs at 8 AM checking all users' meal plans for today
2. Identifies recipes with advance_prep_text requiring morning reminder
3. Sends push notification to user with prep instructions
4. Notification format: "Remember to marinate chicken for tonight's dinner"
5. Notifications sent for current day's meals only
6. Tests verify reminder identification, notification sending, and timing

**Prerequisites:** Story 4.3

---

**Story 6.2: SEO-Optimized Landing Page**

As a visitor,
I want to discover imkitchen features and benefits through a clear landing page,
So that I can understand the value proposition before registering.

**Acceptance Criteria:**
1. Landing page route (/) displays for unauthenticated users (authenticated users redirected to dashboard)
2. Hero section with value proposition: "Intelligent meal planning with month-based automation and accompaniment pairing"
3. Key features showcase: meal planning, accompaniments, shopping lists (3 columns)
4. How-it-works section: 3-step process with visuals
5. Example meal plan screenshots
6. Pricing preview: Free vs Premium comparison table
7. Multiple CTAs throughout page: "Get Started Free", "Sign Up"
8. SEO optimization: meta tags, Schema.org structured data, semantic HTML, <3 second load time
9. Mobile-responsive design
10. Accessibility: Landing page passes WAVE accessibility checker, all interactive elements keyboard-navigable, semantic HTML5 elements used (nav, main, section, article)
11. Tests verify SEO meta tags, page load performance, and accessibility

**Prerequisites:** Story 1.2

---

**Story 6.3: Pricing Page with Tier Comparison**

As a visitor,
I want to see detailed pricing and feature comparison,
So that I can decide whether to use free tier or upgrade to premium.

**Acceptance Criteria:**
1. Pricing page displays Free vs Premium tier comparison table
2. Free tier: First week visibility only, 10 favorite limit, unlimited regenerations, all community features
3. Premium tier: Full month visibility, unlimited favorites, all features ($9.99/month or $59.94/year - 50% savings)
4. Feature comparison: check marks for included features, X for excluded
5. "Get Started Free" and "Upgrade to Premium" CTAs
6. FAQ section answering common questions
7. Tests verify pricing accuracy and CTA routing

**Prerequisites:** Story 6.2

---

**Story 6.4: Public Contact Form**

As a visitor or user,
I want to submit questions or feedback through a contact form,
So that I can reach the imkitchen team without requiring authentication.

**Acceptance Criteria:**
1. Public contact form route (/contact) with fields: name, email, subject, message
2. Form validation: required fields, valid email format
3. ContactFormSubmitted event stores submission
4. Success message after submission
5. Form accessible to both authenticated and unauthenticated users
6. Tests verify form submission, validation, and event creation

**Prerequisites:** Story 1.6

---

**Story 6.5: Admin Email Notifications**

As an admin,
I want to receive email notifications for new contact form submissions,
So that I can respond promptly to user inquiries.

**Acceptance Criteria:**
1. Email notification sent to configured admin email(s) on ContactFormSubmitted event
2. Email includes: submitter name, email, subject, message, timestamp
3. Email contains link to admin panel contact inbox
4. Admin email configuration in config/default.toml
5. Tests verify email sending with mock SMTP server

**Prerequisites:** Story 6.4, Story 5.8

---

**Story 6.6: Home Route Dynamic Routing**

As a product owner,
I want the home route (/) to display landing page for visitors and dashboard for authenticated users,
So that the user experience is optimized for each audience.

**Acceptance Criteria:**
1. Home route (/) checks authentication status via JWT cookie
2. Unauthenticated users → SEO-optimized landing page
3. Authenticated users → Dashboard with nearest day's meals
4. Smooth transition after login (redirect to dashboard)
5. Tests verify routing logic for both cases

**Prerequisites:** Story 6.2, Story 4.3

---

## Story Guidelines Reference

**Story Format:**

```
**Story [EPIC.N]: [Story Title]**

As a [user type],
I want [goal/desire],
So that [benefit/value].

**Acceptance Criteria:**
1. [Specific testable criterion]
2. [Another specific criterion]
3. [etc.]

**Prerequisites:** [Dependencies on previous stories, if any]
```

**Story Requirements:**

- **Vertical slices** - Complete, testable functionality delivery
- **Sequential ordering** - Logical progression within epic
- **No forward dependencies** - Only depend on previous work
- **AI-agent sized** - Completable in 2-4 hour focused session
- **Value-focused** - Integrate technical enablers into value-delivering stories

---

**For implementation:** Use the `create-story` workflow to generate individual story implementation plans from this epic breakdown.

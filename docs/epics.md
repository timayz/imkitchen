# imkitchen - Epic Breakdown

**Author:** Jonathan
**Date:** 2025-10-31
**Project Level:** 3
**Target Scale:** Comprehensive product with freemium model, community features, and automated recipe list generation

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

**Product model (no calendar):** every user has exactly one **recipe list**. It is either generated (random picks from the user's candidate pool) or filled by hand from recipe pages. The Kitchen page shows the list and which recipe to cook next, cooking mode walks through one recipe, and the Groceries page is the merged ingredient list of everything in the list. There are no weeks, days, meal slots, dates or scheduled reminders.

---

## Epic 1: Foundation & User Management

**Goal:** Establish foundational project infrastructure with user authentication, profile management, and admin capabilities to enable secure user operations and preference storage for recipe list generation.

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
So that I can access the platform securely.

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
I want to configure my household size, dietary restrictions, cuisine variety and optional courses,
So that recipe list generation respects my dietary needs and produces the courses I actually cook.

**Acceptance Criteria:**
1. UserProfileUpdated event stores household_size, dietary restrictions (array), cuisine_variety_weight (default 0.7) and the enabled optional course types (appetizer, accompaniment, dessert, beverage, condiment)
2. Profile update command accepts input struct with validation
3. Settings page displays current preferences with edit form
4. Query handler projects profile data to user_profiles table
5. Profile data accessible via query function for recipe list generation
6. Form submission with optimistic UI update
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

**Goal:** Enable users to create, manage, and bulk import recipes with an explicit course type (Main Course, Appetizer, Accompaniment, Dessert, Beverage, Condiment), supporting explicit field configuration and community sharing foundation.

**Value Delivery:** Users can build their recipe library through manual creation or bulk JSON import, setting dietary restrictions and accompaniment preferences explicitly for recipe list generation.

**Visual Mockup References:**
- `mockups/recipe-create.html` - Recipe creation form with course type selector, explicit field configuration, main course accepts_accompaniment toggle
- `mockups/recipes-list.html` - Recipe library with filters and stats cards
- `mockups/recipe-detail.html` - Full recipe view with ingredients, instructions, rating summary
- `mockups/import.html` - Bulk JSON import with drag-drop, schema docs, real-time progress (imported/failed/remaining counts)

### Stories

**Story 2.1: Recipe Creation (Course Types)**

As a user,
I want to create recipes with explicit course type selection (Main Course, Appetizer, Accompaniment, Dessert, Beverage, Condiment),
So that I can build my recipe library for list generation.

**Acceptance Criteria:**
1. Recipe aggregate with RecipeCreated event including: recipe_type (enum), name, ingredients, instructions, dietary_restrictions, cuisine_type, complexity, advance_prep_text
2. Main courses include accepts_accompaniment field (defaults to false)
3. Recipe creation command validates required fields using validator crate
4. Recipe creation form with type selector and conditional fields (accompaniment field shown only for Main Course)
5. Recipe projection table stores all recipe data for querying
6. User can view their recipe list filtered by type
7. Tests verify recipe creation for every course type with validation

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
I want to save community recipes as favorites,
So that they join my own recipes in the candidate pool used by recipe list generation.

**Acceptance Criteria:**
1. RecipeFavorited and RecipeUnfavorited events store user-recipe relationship
2. Favorites are unlimited on both free and premium tiers
3. Recipe pages and cards show a "Save" button with toggle state
4. Saved recipes are listed with the user's recipes and become generation candidates
5. When recipe owner deletes recipe, all favorites automatically removed (no notifications)
6. Query projection tracks favorite_count per recipe for community sorting
7. Tests verify save, unsave, and cascade deletion

**Prerequisites:** Story 2.1

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
2. Polling updates progress without blocking UI
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
5. Schema includes every course type with type-specific fields
6. Tests verify schema endpoint accessibility

**Prerequisites:** Story 2.4

---

**Story 2.7: Recipe Candidate Pool**

As a user,
I want my own recipes and the community recipes I saved to form one candidate pool,
So that list generation can pick from everything I am willing to cook.

**Acceptance Criteria:**
1. Candidate pool projection holds, per user, every recipe they own plus every community recipe they saved, with recipe type, dietary restrictions, cuisine and accepts_accompaniment
2. Creating or importing a recipe adds it to the owner's pool; saving a community recipe adds it to the saver's pool
3. Unsaving a recipe removes it from the saver's pool; deleting a recipe removes it from every pool
4. Recipe edits (type, dietary restrictions, cuisine, ingredients, household size) are reflected in the pool
5. Pool is queryable by recipe type and dietary restrictions for generation
6. Tests verify pool membership after create, import, save, unsave, delete and edit

**Prerequisites:** Story 2.1, Story 2.3

---

## Epic 3: Recipe List Generation

**Goal:** Implement random recipe list generation from the user's candidate pool with dietary filtering, cuisine variety weighting, optional course types and accompaniment pairing, plus manual add/remove, so users get a ready-to-cook list without planning.

**Value Delivery:** Users ask for N main courses and get a complete list (mains plus the optional courses they enabled) that respects their dietary restrictions, never repeats a recipe, and can be edited by hand at any time.

**Visual References (live templates):**
- `templates/kitchen.html` and `templates/partials/kitchen-generate-modal.html` - Kitchen page with Generate/Regenerate button and generation modal (count selector)
- `templates/partials/recipes-detail-add-to-shopping-button.html` - "Add to my list" / "In my list" button on a recipe page

### Stories

**Story 3.1: Basic Recipe List Generation**

As a user,
I want to generate a list of N main courses from my recipes,
So that I have something to cook without choosing dish by dish.

**Acceptance Criteria:**
1. Each user has exactly one recipe list (Shopping aggregate keyed by user id)
2. Generate modal on the Kitchen page asks how many main courses to pick (1 to 30, default 7)
3. Generation command picks up to N main courses at random from the user's candidate pool (own recipes plus saved community recipes)
4. A recipe never appears twice in the list
5. If the pool holds fewer main courses than requested, the list is simply shorter (no error, no empty placeholders)
6. If the pool holds no main course at all, generation is rejected with a clear message
7. ListGenerated event stores the ordered recipe ids and the merged ingredient list
8. Tests verify count validation, random selection, uniqueness and short-pool behaviour

**Prerequisites:** Story 2.7

---

**Story 3.2: Regeneration Replaces the List**

As a user,
I want Regenerate to build a fresh list,
So that I can start over whenever the current list no longer suits me.

**Acceptance Criteria:**
1. Kitchen page shows "Generate" when the list is empty and "Regenerate" when it has recipes; both open the generation modal
2. Generation replaces the whole list: manually added recipes are dropped too
3. Generation clears every checked grocery and resets the cooking progress of every recipe
4. Non-deterministic selection produces different lists across regenerations
5. Query handlers replace the projected list, groceries and cooking statuses
6. Tests verify full replacement, cleared checks and reset statuses

**Prerequisites:** Story 3.1

---

**Story 3.3: Dietary Restriction Filtering**

As a user,
I want generation to respect my dietary restrictions,
So that my list only contains recipes I can actually eat.

**Acceptance Criteria:**
1. Generation filters the candidate pool by the dietary restrictions in the user's preferences
2. A recipe is eligible only if it matches every restriction the user selected
3. Filtering applies to main courses and to every optional course type
4. A shorter list results when too few compliant recipes exist (no non-compliant recipe is ever picked)
5. Tests verify filtering with various dietary combinations

**Prerequisites:** Story 3.1, Story 1.3

---

**Story 3.4: Cuisine Variety Weight**

As a user,
I want the cuisine variety weight to control how wide generation samples,
So that I can choose between maximum variety and a more repetitive, familiar list.

**Acceptance Criteria:**
1. Cuisine variety weight (0.1 to 1.0, default 0.7) from the user's preferences sizes the main-course candidate pool before random selection
2. Higher weight (closer to 1.0) samples from the whole eligible pool; lower weight samples from a smaller subset
3. Weights below 0.1 are rejected with a validation error
4. Optional course types are sampled from their full eligible pool regardless of the weight
5. Tests verify pool sizing for several weight values

**Prerequisites:** Story 3.3

---

**Story 3.5: Optional Course Types**

As a user,
I want generation to add the optional courses I enabled in my preferences,
So that my list covers appetizers, desserts, drinks and condiments as well as mains.

**Acceptance Criteria:**
1. Optional course types are appetizer, accompaniment, dessert, beverage and condiment, each enabled or disabled in preferences
2. Each generated meal pairs its main course with one recipe of every enabled type, drawn from that type's pool without reuse; once a pool is exhausted later meals simply lack that course
3. Disabled types are skipped entirely (no query, no recipes)
4. Optional recipes are subject to the same dietary filtering and the same no-duplicate rule as mains
5. The stored list keeps meal order: starter, main, side, dessert, drink, sauce, then the next meal
6. Tests verify per-type caps, disabled types and uniqueness across types

**Prerequisites:** Story 3.4

---

**Story 3.6: Accompaniment Pairing**

As a user,
I want accompaniments only when a main course in my list accepts one,
So that side dishes show up with curry or pasta and not with a one-pot meal.

**Acceptance Criteria:**
1. Main courses carry an accepts_accompaniment flag (Story 2.1)
2. An accompaniment is paired with a main only if the accompaniment course type is enabled AND that main accepts an accompaniment
3. The side directly follows its main in the stored list order
4. Mains that do not accept an accompaniment get none, even if the type is enabled
5. Tests verify the pairing gate with and without accepting mains

**Prerequisites:** Story 3.5

---

**Story 3.7: Manual Add and Remove**

As a user,
I want to add any recipe to my list from its page and remove it from the Kitchen page,
So that I can shape the list by hand instead of, or on top of, generating it.

**Acceptance Criteria:**
1. Every recipe page (own or shared) shows an "Add to my list" button, which becomes "In my list" once the recipe is in the list
2. RecipeAdded event appends the recipe to the list and recomputes the merged ingredients
3. Adding a recipe already in the list is a no-op (no duplicates)
4. Each recipe row and the hero card on the Kitchen page has a remove control (X)
5. RecipeRemoved event removes the recipe, its cooking status and its ingredients from the list
6. Manual adds and removes keep existing grocery checks for ingredients that remain
7. Tests verify add, duplicate add, remove and ingredient recomputation

**Prerequisites:** Story 3.1

---

**Story 3.8: Household Size Scaling**

As a user,
I want ingredient quantities scaled to my household size,
So that my groceries match the number of people I cook for.

**Acceptance Criteria:**
1. Each recipe stores the household size it was written for (default 4)
2. When the list changes (generate, add, remove), every recipe's ingredients are scaled from the recipe's household size to the user's household size
3. Scaled ingredients with the same name and unit are merged into one quantity
4. Merged ingredients are stored on the list event so the Groceries page needs no recomputation
5. Tests verify scaling and merging across several recipes

**Prerequisites:** Story 3.7, Story 1.3

---

## Epic 4: Kitchen, Cooking Mode & Groceries

**Goal:** Build the mobile-first Kitchen home page, a step-by-step cooking mode with persistent progress, and a Groceries page derived from the recipe list, so users always know what to cook next and what to buy.

**Value Delivery:** Users open the app on the recipe to cook next, cook it hands-free step by step, see which recipes need advance prep, and shop from one aisle-grouped grocery list that follows their list automatically.

**Visual References (live templates):**
- `templates/kitchen.html` and `templates/partials/kitchen-dish.html` - Kitchen page: "Up next" header with cooked counter, hero card, recipe rows, "Add recipes" card, Prep ahead rail
- `templates/cooking.html` and `templates/partials/cooking-screen.html` - Cooking mode: ingredients screen, then one instruction per screen with timers
- `templates/groceries.html` and `templates/partials/groceries-body.html` - Groceries page grouped by aisle with route strip and per-aisle progress

### Stories

**Story 4.1: Kitchen Page (Home)**

As a user,
I want the home page to show my recipe list with the recipe to cook next on top,
So that I know what to cook without browsing.

**Acceptance Criteria:**
1. Kitchen page is served at `/` for authenticated users
2. Header reads "Up next" with a cooked counter (cooked recipes / recipes in list) and the Generate/Regenerate button (Story 3.2)
3. Hero card shows the recipe to cook next: the first not-yet-cooked recipe when the list is ordered by course (starter, main, side, dessert, drink, sauce) then by list order
4. Hero card offers Start cooking, See recipe and remove actions
5. Remaining recipes appear below as rows; tapping a row focuses it in the hero card, X removes it
6. Rows show a "Cooked" badge for completed recipes and a "Cooking" badge for recipes in progress
7. The rows end with an "Add recipes" card linking to the recipe library
8. Tests verify ordering, badges, counter and remove actions

**Prerequisites:** Story 3.7

---

**Story 4.2: Kitchen Onboarding States**

As a new user,
I want the Kitchen page to tell me what to do first,
So that I am not left with an empty screen.

**Acceptance Criteria:**
1. User with no recipes in their pool sees an "add your first recipe" screen linking to recipe creation and the community
2. User with recipes but an empty list sees a "generate your list" screen with the Generate button
3. Both screens replace the hero card and rows; the normal layout appears as soon as the list has a recipe
4. Tests verify the three states (no recipes, empty list, populated list)

**Prerequisites:** Story 4.1

---

**Story 4.3: Prep Ahead Rail**

As a user,
I want to see which uncooked recipes need advance preparation,
So that I remember to marinate, rise or chill before I start cooking.

**Acceptance Criteria:**
1. Kitchen page shows a "Prep ahead" side rail listing every uncooked recipe in the list that has an advance-prep note
2. Each entry shows the recipe name and its advance-prep text
3. Cooked recipes and recipes without an advance-prep note are not listed
4. Rail is hidden when no recipe qualifies
5. Tests verify rail contents across cooking statuses

**Prerequisites:** Story 4.1

---

**Story 4.4: Cooking Mode**

As a user,
I want to cook a recipe step by step with timers,
So that I can follow the instructions without scrolling a full recipe page.

**Acceptance Criteria:**
1. Cooking mode is served at `/kitchen/{recipe_id}/cook` for recipes in the user's list
2. Each recipe in the list has a cooking cursor: Idle, Cooking(step n) or Completed, persisted on the list (RecipeStatusChanged event)
3. Idle shows the ingredients screen first; starting moves to Cooking(1) and shows one instruction per screen with next/previous controls
4. Instructions with a duration offer a timer
5. Finishing the last step marks the recipe Completed; it stays in the list with a "Cooked" badge until removed or the list is regenerated
6. Leaving and reopening cooking mode resumes at the saved step
7. Tests verify cursor transitions, persistence and the completed state

**Prerequisites:** Story 4.1

---

**Story 4.5: Groceries Page**

As a user,
I want one grocery list for everything in my recipe list,
So that I can shop once for all of it.

**Acceptance Criteria:**
1. Groceries page is served at `/groceries` and shows the merged, household-size-scaled ingredients of every recipe in the list (Story 3.8)
2. Ingredients are grouped by store aisle (ingredient category) with a route strip to jump between aisles
3. Each aisle shows its progress (checked / total)
4. Checking and unchecking an ingredient is persisted (Checked / Unchecked events) and survives reloads
5. The page updates automatically when the list changes; there is no separate generate step and no date range
6. An "Edit recipes" link returns to the Kitchen page
7. Tests verify grouping, progress, check persistence and updates after list changes

**Prerequisites:** Story 3.8

---

**Story 4.6: App Navigation**

As a user,
I want a simple navigation between the four areas of the app,
So that I can move between cooking, recipes, shopping and settings on a phone.

**Acceptance Criteria:**
1. Navigation entries are Kitchen (`/`), Recipes (`/recipes`), Groceries (`/groceries`) and Settings
2. Active entry is highlighted on every page
3. Navigation is mobile-first and remains usable on desktop
4. Tests verify navigation links and active state

**Prerequisites:** Story 4.5

---

## Epic 5: Community Features & Freemium Access

**Goal:** Enable recipe sharing, community rating system, freemium access controls, and admin user management to build a self-sustaining recipe ecosystem with quality filtering and premium conversion incentives.

**Value Delivery:** Users discover high-quality community recipes through ratings, share their own recipes publicly, use every feature for free with ads that drive premium conversion, and admins maintain platform quality through user management.

**Visual Mockup References:**
- `mockups/community.html` - Community recipe browse with stats (2,547 recipes), trending section, filters (type/cuisine/dietary), rating system
- `mockups/recipe-detail.html` - Rating summary (4.8 stars, 23 reviews), write review form
- `mockups/recipes-list.html` - Ad slot and "Get Premium" sidebar CTA shown to free tier users
- `mockups/admin-users.html` - User management with suspend/reactivate actions
- `mockups/admin-contact.html` - Contact form inbox management

### Stories

**Story 5.1: Recipe Sharing (Public/Private)**

As a user,
I want to share my recipes publicly with the community,
So that other users can discover and save my recipes.

**Acceptance Criteria:**
1. Recipe aggregate includes is_shared field (boolean, defaults to false)
2. RecipeShared and RecipeUnshared events toggle public visibility
3. Recipe edit form includes "Share with community" toggle
4. Shared recipes appear in community browse page for all users (free and premium)
5. Recipe owner can unshare at any time
6. Every course type is sharable
7. Tests verify sharing toggle, community visibility, and unsharing

**Prerequisites:** Story 2.1

---

**Story 5.2: Community Recipe Browse & Discovery**

As a user,
I want to browse community-shared recipes with filters,
So that I can discover new recipes to save into my candidate pool.

**Acceptance Criteria:**
1. Community recipe page displays all shared recipes (paginated)
2. Filters: course type, cuisine type, dietary restrictions
3. Search by recipe name or ingredients
4. Sort by rating (highest first), newest, most favorited
5. Recipe cards show: name, type, rating, favorite count, owner name
6. Quick-save button on recipe cards
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

**Story 5.4: Ad-Supported Free Tier**

As a free tier user,
I want full access to every feature with ads on content pages,
So that I can use imkitchen for free while understanding that premium removes the ads.

**Acceptance Criteria:**
1. `user.show_ads()` helper returns true for every non-premium session (free users and expired subscriptions) and false for active premium
2. Ad slots are rendered with the `ads::slot` partial (`templates/partials/ad-slot.html`) inside `show_ads()` guards; each slot links to the upgrade page with "Go Premium to remove them."
3. Ads are hidden for premium users, in demo mode, and when premium is not configured (self-hosted)
4. Free tier has no other restriction: unlimited favorites, generations and community features
5. Sidebar and mobile navigation show a "Get Premium" CTA for free users that opens `/upgrade/modal`
6. Ad slots are provider-agnostic placeholders targetable via `[data-ad-slot]` and stay non-personalized by default
7. Tests verify the `show_ads()` predicate (free, premium, expired premium) and that ad slots render only for free users

**Prerequisites:** Story 1.5, Story 5.2

---

**Story 5.5: Upgrade Prompts (Multiple Touchpoints)**

As a product owner,
I want upgrade prompts displayed at strategic touchpoints,
So that free tier users are aware of premium benefits and conversion opportunities.

**Acceptance Criteria:**
1. "Get Premium" CTA in the sidebar (expanded card on desktop, icon on the rail) and in the mobile bottom navigation opens the upgrade modal from `/upgrade/modal` (Story 5.4)
2. Ad slots on content pages carry a "Go Premium to remove them." link to the upgrade page
3. Pricing page (Story 6.2) carries an "Upgrade to Premium" link for signed-in free users
4. Upgrade modal served from a single route (`/upgrade/modal`) so every touchpoint shows the same content; it redirects premium users and is unavailable when premium is not configured
5. Upgrade prompts include: ad-free benefit, pricing (monthly or annual plan), "Get Premium" CTA
6. Prompts are persistent but not intrusive (modal is dismissible, ad slots are inline)
7. Tests verify modal triggering from each touchpoint and that none of them appear for premium or demo sessions

**Prerequisites:** Story 5.4

---

**Story 5.6: Premium Access Control Logic**

As a developer,
I want centralized premium access control logic,
So that premium features are consistently enforced across the application.

**Acceptance Criteria:**
1. Access control function checks: active premium subscription OR global bypass OR user bypass flag
2. Function used in: ad display (`show_ads()`) and upgrade CTA visibility
3. Premium subscription status stored in user profile (is_premium_active boolean)
4. Tests verify access control in all premium-gated features
5. Documentation in CLAUDE.md explaining premium access patterns

**Prerequisites:** Story 1.5, Story 5.4

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
5. Admin can view user's favorited recipes and current recipe list
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
So that users don't have broken favorites in their candidate pool.

**Acceptance Criteria:**
1. Recipe deletion triggers cascade removal from all users' favorite lists and candidate pools
2. No notifications sent to users who had it favorited (silent removal)
3. No notifications sent to recipe owner about favorite count
4. Users discover the missing favorite organically (it no longer appears in their saved recipes or in generated lists)
5. Tests verify cascade deletion and notification absence

**Prerequisites:** Story 2.2, Story 2.3, Story 5.1

---

## Epic 6: Landing Page, Pricing & Contact

**Goal:** Implement an SEO-optimized landing page with pricing information and a contact form to complete the user acquisition experience.

**Value Delivery:** New visitors discover imkitchen through an SEO-optimized landing page, understand the free and premium tiers, and can contact support through a public form.

**Visual Mockup References:**
- `mockups/index.html` - SEO-optimized landing page with hero, features showcase, how-it-works (3 steps), pricing preview, testimonials
- `mockups/contact.html` - Public contact form with subject categories and FAQ section

### Stories

**Story 6.1: SEO-Optimized Landing Page**

As a visitor,
I want to discover imkitchen features and benefits through a clear landing page,
So that I can understand the value proposition before registering.

**Acceptance Criteria:**
1. Landing page route (/) displays for unauthenticated users (authenticated users see the Kitchen page)
2. Hero section with value proposition: "Generate a recipe list from your own recipes, cook it step by step, shop it from one grocery list"
3. Key features showcase: recipe list generation, cooking mode, groceries (3 columns)
4. How-it-works section: 3-step process with visuals
5. Example recipe list and groceries screenshots
6. Pricing preview: Free vs Premium comparison table
7. Multiple CTAs throughout page: "Get Started Free", "Sign Up"
8. SEO optimization: meta tags, Schema.org structured data, semantic HTML, <3 second load time
9. Mobile-responsive design
10. Accessibility: Landing page passes WAVE accessibility checker, all interactive elements keyboard-navigable, semantic HTML5 elements used (nav, main, section, article)
11. Tests verify SEO meta tags, page load performance, and accessibility

**Prerequisites:** Story 1.2

---

**Story 6.2: Pricing Page with Tier Comparison**

As a visitor,
I want to see detailed pricing and feature comparison,
So that I can decide whether to use free tier or upgrade to premium.

**Acceptance Criteria:**
1. Pricing page displays Free vs Premium tier comparison table
2. Free tier: ad-supported, otherwise full features (unlimited favorites, generations and community features)
3. Premium tier: no ads, all features ($9.99/month or $59.94/year - 50% savings)
4. Feature comparison: check marks for included features, X for excluded
5. "Get Started Free" and "Upgrade to Premium" CTAs
6. FAQ section answering common questions
7. Tests verify pricing accuracy and CTA routing

**Prerequisites:** Story 6.1

---

**Story 6.3: Public Contact Form**

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

**Story 6.4: Admin Email Notifications**

As an admin,
I want to receive email notifications for new contact form submissions,
So that I can respond promptly to user inquiries.

**Acceptance Criteria:**
1. Email notification sent to configured admin email(s) on ContactFormSubmitted event
2. Email includes: submitter name, email, subject, message, timestamp
3. Email contains link to admin panel contact inbox
4. Admin email configuration in config/default.toml
5. Tests verify email sending with mock SMTP server

**Prerequisites:** Story 6.3, Story 5.8

---

**Story 6.5: Home Route Dynamic Routing**

As a product owner,
I want the home route (/) to display the landing page for visitors and the Kitchen page for authenticated users,
So that the user experience is optimized for each audience.

**Acceptance Criteria:**
1. Home route (/) checks authentication status via JWT cookie
2. Unauthenticated users → SEO-optimized landing page
3. Authenticated users → Kitchen page with their recipe list (Story 4.1)
4. Smooth transition after login (redirect to the Kitchen page)
5. Tests verify routing logic for both cases

**Prerequisites:** Story 6.1, Story 4.1

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

# imkitchen - Epic Breakdown

**Author:** Jonathan
**Date:** 2025-10-10
**Project Level:** Level 3 (Full Product)
**Target Scale:** 12-40 stories, 2-5 epics, full PRD + architect handoff

---

## Epic Overview

This document provides the detailed epic breakdown for imkitchen, an intelligent meal planning and cooking optimization platform. The MVP is structured into 5 epics delivering incremental value:

1. **User Authentication and Profile Management** (8 stories) - Foundation for secure user access and personalized meal planning preferences
2. **Recipe Management System** (10 stories) - Core recipe CRUD, organization, and community sharing capabilities
3. **Intelligent Meal Planning Engine** (12 stories) - Automated meal scheduling with multi-factor optimization and recipe rotation
4. **Shopping and Preparation Orchestration** (11 stories) - Shopping list generation and advance preparation reminder system
5. **Progressive Web App and Mobile Experience** (9 stories) - Installable PWA with offline capabilities and kitchen-optimized interface

**Total Estimated Stories:** 50 stories
**Development Timeline:** 5-8 months to MVP launch
**Architecture Approach:** Event-sourced DDD using evento (Rust), TDD enforced, CQRS pattern

---

## Epic Details

### Epic 1: User Authentication and Profile Management

**Goal:** Enable secure user registration, authentication, and profile management with freemium tier controls

**Value Delivered:** Users can create accounts, log in securely, and manage their dietary preferences and cooking constraints that feed intelligent meal planning

**Success Criteria:**
- 95% successful registration rate (valid inputs)
- <2 second authentication response time
- Zero security vulnerabilities in auth flow
- Profile preferences correctly feed meal planning algorithm

---

#### Story 1.1: User Registration
**As a** new user
**I want to** create an account with email and password
**So that** I can access personalized meal planning features

**Prerequisites:** None

**Acceptance Criteria:**
1. Registration form displays email and password fields with clear validation rules
2. Password must be minimum 8 characters
3. Email format validation applied on client and server side
4. System prevents duplicate email registrations with clear error message
5. Successful registration creates user account and logs user in automatically
6. JWT token stored in HTTP-only secure cookie with CSRF protection
7. User redirected to onboarding/profile setup after registration
8. Failed registration displays specific validation errors (weak password, duplicate email, invalid format)

**Technical Notes:**
- Use evento aggregate for User domain entity
- Password hashing with argon2
- Email uniqueness enforced at database level
- Emit UserRegistered event for audit trail

---

#### Story 1.2: User Login
**As a** registered user
**I want to** log in with my credentials
**So that** I can access my meal plans and recipes

**Prerequisites:** User has registered account

**Acceptance Criteria:**
1. Login form accepts email and password
2. System validates credentials against stored hashed password
3. Successful login issues JWT token in HTTP-only secure cookie
4. Failed login displays generic error "Invalid credentials" (no user enumeration)
5. Login redirected to home dashboard
6. Session persists across browser restarts until token expiration
7. JWT token includes user ID and role (user/premium-user)
8. Token expiration set to 7 days with sliding expiration on activity

**Technical Notes:**
- JWT signing using RS256 algorithm
- Token includes claims: user_id, email, role, exp, iat
- Failed login attempts logged for security monitoring

---

#### Story 1.3: Password Reset Flow
**As a** user who forgot my password
**I want to** reset it via email
**So that** I can regain access to my account

**Prerequisites:** User has registered account with valid email

**Acceptance Criteria:**
1. "Forgot Password" link available on login page
2. User enters email address to request reset
3. System sends password reset email with time-limited token (valid 1 hour)
4. Reset link directs to password reset form with token validation
5. User enters new password (min 8 characters) and confirms
6. Successful reset invalidates old password and all existing sessions
7. User redirected to login page with success message
8. Expired or invalid tokens display clear error message

**Technical Notes:**
- Reset tokens stored with expiration timestamp
- Email sent via configured SMTP service
- Emit PasswordResetRequested and PasswordChanged events

---

#### Story 1.4: User Profile Creation (Onboarding)
**As a** newly registered user
**I want to** complete my profile with dietary and cooking preferences
**So that** the meal planning algorithm can personalize recommendations

**Prerequisites:** User has registered account

**Acceptance Criteria:**
1. Onboarding wizard displays after first registration
2. Step 1: Dietary restrictions (checkboxes: vegetarian, vegan, gluten-free, allergens with text input)
3. Step 2: Household size (numeric input, 1-10)
4. Step 3: Cooking skill level (radio: beginner, intermediate, expert)
5. Step 4: Typical weeknight availability (time range picker, duration slider)
6. Each step validates inputs before allowing progression
7. User can skip onboarding (optional) - defaults applied
8. Completed profile stored and accessible for editing later
9. Profile data feeds meal planning optimization algorithm

**Technical Notes:**
- UserProfile aggregate with ProfileCompleted event
- Defaults: household_size=2, skill_level=intermediate, availability=6-7pm/45min
- Validation: validator crate for input constraints

---

#### Story 1.5: Profile Editing
**As a** registered user
**I want to** update my profile preferences
**So that** meal planning reflects my current needs

**Prerequisites:** User has completed profile

**Acceptance Criteria:**
1. Profile page displays current preferences in editable form
2. User can modify dietary restrictions, household size, skill level, availability
3. Changes validated before saving
4. Successful save updates profile and shows confirmation message
5. Updated preferences immediately affect future meal plan generations
6. Active meal plans remain unchanged until regenerated
7. Profile change history tracked for audit purposes

**Technical Notes:**
- ProfileUpdated event emitted with changed fields
- Event sourcing ensures complete audit trail
- CQRS read model updated for profile display

---

#### Story 1.6: Freemium Tier Enforcement (10 Recipe Limit)
**As a** free tier user
**I want to** understand my recipe limit
**So that** I know when to upgrade to premium

**Prerequisites:** User registered on free tier

**Acceptance Criteria:**
1. Recipe count displayed on recipe management page (e.g., "7/10 recipes")
2. User can create recipes until limit reached
3. At 10th recipe, system shows "10/10 recipes - Upgrade for unlimited"
4. Attempting to create 11th recipe prevents creation, displays upgrade prompt
5. User can edit or delete existing recipes within limit
6. Deleting recipe frees up slot for new recipe
7. Recipe limit applies only to user-created recipes (not community-discovered)
8. Premium users see "Unlimited recipes" indicator

**Technical Notes:**
- RecipeCountExceeded domain event when limit reached
- Read model tracks recipe count per user
- Premium role bypasses limit check

---

#### Story 1.7: Premium Upgrade Flow
**As a** free tier user
**I want to** upgrade to premium
**So that** I can access unlimited recipes and advanced features

**Prerequisites:** User on free tier with valid payment method

**Acceptance Criteria:**
1. "Upgrade to Premium" button visible throughout app with freemium restrictions
2. Upgrade page displays premium benefits and pricing ($9.99/month)
3. Secure payment form accepts card details
4. Payment processing handled via secure payment gateway
5. Successful payment upgrades user role to premium-user
6. All freemium restrictions immediately removed
7. User receives email confirmation of upgrade
8. Failed payment displays error and retry option

**Technical Notes:**
- Payment gateway integration (Stripe recommended, avoid vendor lock-in)
- UserUpgradedToPremium event emitted
- Role change persists in JWT for subsequent requests
- PCI DSS compliance via payment gateway (no card storage)

---

#### Story 1.8: User Logout
**As a** logged-in user
**I want to** log out
**So that** my session is securely terminated

**Prerequisites:** User logged in

**Acceptance Criteria:**
1. Logout button accessible from navigation menu
2. Clicking logout clears JWT cookie
3. User redirected to login page
4. Logged-out user cannot access authenticated routes
5. Logout action logged for security audit
6. Logout confirmation message displayed

**Technical Notes:**
- Cookie cleared with secure flags
- Session invalidation tracked
- UserLoggedOut event emitted

---

**Epic 1 Technical Summary:**
- **Aggregates:** User, UserProfile
- **Events:** UserRegistered, UserLoggedIn, PasswordResetRequested, PasswordChanged, ProfileCompleted, ProfileUpdated, UserUpgradedToPremium, UserLoggedOut
- **Security:** OWASP compliance, JWT RS256, argon2 password hashing, CSRF protection
- **Testing:** TDD enforced - unit tests for domain logic, integration tests for auth flows, E2E tests for registration/login journeys

**Technical Specification:** Detailed implementation guide available in `./docs/tech-spec-epic-1.md`

---

### Epic 2: Recipe Management System

**Goal:** Provide comprehensive recipe creation, organization, and sharing capabilities with community privacy controls

**Value Delivered:** Users can build their personal recipe library, organize collections, mark favorites, and optionally share recipes with the community

**Success Criteria:**
- 90% recipe creation success rate
- Average recipe entry time <5 minutes
- 40% monthly community recipe rating participation
- Community recipe library reaches 500+ recipes within first year

---

#### Story 2.1: Create Recipe
**As a** user
**I want to** create a new recipe with all details
**So that** I can add it to my meal planning rotation

**Prerequisites:** User is authenticated

**Acceptance Criteria:**
1. Recipe creation form includes: title, ingredients (quantity/unit/name), step-by-step instructions, prep time, cook time, advance prep requirements, serving size
2. Ingredients list allows adding/removing rows dynamically
3. Instructions allow numbered step entry with reordering capability
4. Each instruction step includes optional timer field (duration in minutes)
5. Advance prep field accepts text description (e.g., "Marinate 4 hours")
6. All required fields validated before save
7. Successful save creates recipe and displays confirmation
8. User redirected to recipe detail page after creation
9. Recipe automatically owned by creating user
10. Default privacy set to "private"

**Technical Notes:**
- Recipe aggregate with RecipeCreated event
- Ingredients stored as structured data (quantity, unit, ingredient_name)
- Instructions stored as structured data (step_number, instruction_text, optional_timer_minutes)
- Validation: non-empty title, at least 1 ingredient, at least 1 instruction step
- Free tier users limited to 10 recipes total

---

#### Story 2.2: Edit Recipe
**As a** recipe owner
**I want to** modify my recipe details
**So that** I can correct errors or improve instructions

**Prerequisites:** User owns the recipe

**Acceptance Criteria:**
1. Recipe edit page pre-populated with current recipe data
2. All fields editable (title, ingredients, instructions, timing, advance prep, serving size)
3. Changes validated before saving
4. Successful save updates recipe and shows confirmation
5. Recipe version history maintained via event sourcing
6. Updated recipe immediately reflects in meal plans (if currently scheduled)
7. Only recipe owner can edit their recipes
8. Community-shared recipes remain editable by owner only

**Technical Notes:**
- RecipeUpdated event with changed fields
- Event sourcing provides complete edit history
- CQRS read model updated for display

---

#### Story 2.3: Delete Recipe
**As a** recipe owner
**I want to** delete a recipe I no longer use
**So that** I can keep my library organized

**Prerequisites:** User owns the recipe

**Acceptance Criteria:**
1. Delete button available on recipe detail page
2. Confirmation dialog displays before deletion: "Are you sure? This cannot be undone."
3. Successful deletion removes recipe from user's library
4. Deleted recipe removed from any active meal plans
5. Meal plans with deleted recipes show empty slots requiring replacement
6. Recipe count decremented (frees slot for free tier users)
7. Community ratings/reviews retained for analytics but recipe no longer discoverable
8. Soft delete maintains data integrity for audit trail

**Technical Notes:**
- RecipeDeleted event (soft delete)
- is_deleted flag on read model
- Cascade logic updates meal plans

---

#### Story 2.4: Organize Recipes into Collections
**As a** user with multiple recipes
**I want to** organize them into collections
**So that** I can find related recipes easily

**Prerequisites:** User has created at least 1 recipe

**Acceptance Criteria:**
1. Collections management page displays all user collections
2. User can create new collection with name and optional description
3. User can add/remove recipes to/from collections
4. Recipe can belong to multiple collections
5. Collections displayed in recipe library sidebar for filtering
6. Clicking collection filters recipe list to show only that collection
7. Default "All Recipes" view shows uncategorized and all collections
8. Collections deletable (removes collection but not recipes)

**Technical Notes:**
- Collection aggregate with CollectionCreated, RecipeAddedToCollection events
- Many-to-many relationship between recipes and collections
- Read model projection for fast collection filtering

---

#### Story 2.5: Automatic Recipe Tagging
**As a** user creating a recipe
**I want** the system to automatically tag my recipe
**So that** I can discover and filter recipes by attributes without manual tagging

**Prerequisites:** User creates or edits recipe

**Acceptance Criteria:**
1. System analyzes recipe data on save
2. Complexity tag assigned based on: ingredient count, instruction steps, advance prep requirements (Simple: <8 ingredients, <6 steps, no advance prep; Moderate: 8-15 ingredients or 6-10 steps; Complex: >15 ingredients or >10 steps or advance prep required)
3. Cuisine tag inferred from ingredient patterns (e.g., soy sauce + ginger = Asian, oregano + tomato = Italian)
4. Dietary tags auto-assigned: vegetarian (no meat/fish), vegan (no animal products), gluten-free (no wheat/flour)
5. Tags displayed on recipe card and detail page
6. Tags used for discovery filtering and meal planning optimization
7. Manual tag override available if auto-tagging incorrect

**Technical Notes:**
- Tagging logic in recipe domain service
- RecipeTagged event with tag list
- Machine learning opportunities for future cuisine detection improvement

---

#### Story 2.6: Mark Recipe as Favorite
**As a** user
**I want to** mark recipes as favorites
**So that** they are included in my meal plan generation

**Prerequisites:** User has access to recipe (owns or community-discovered)

**Acceptance Criteria:**
1. Favorite button (star icon) visible on recipe cards and detail pages
2. Clicking star toggles favorite status (filled = favorite, outline = not favorite)
3. Favorited recipes included in meal planning algorithm pool
4. Non-favorited recipes excluded from meal planning
5. Recipe library filterable by "Favorites Only"
6. Favorite count displayed in user profile
7. Un-favoriting recipe does not remove from existing meal plans
8. Free tier: favorites count toward 10 recipe limit
9. Premium tier: unlimited favorites

**Technical Notes:**
- RecipeFavorited/RecipeUnfavorited events
- Read model tracks favorite status per user-recipe pair
- Meal planning query filters by is_favorited=true

---

#### Story 2.7: Share Recipe to Community
**As a** recipe owner
**I want to** share my recipe with the community
**So that** others can discover and use it

**Prerequisites:** User owns recipe

**Acceptance Criteria:**
1. "Share to Community" toggle on recipe edit page
2. Toggle changes privacy from "private" to "shared"
3. Shared recipes appear in community discovery feed
4. Recipe attribution displays creator's username
5. Shared recipes remain editable only by owner
6. Owner can revert to private at any time (removes from community discovery)
7. Ratings and reviews visible only on shared recipes
8. User profile shows count of shared recipes

**Technical Notes:**
- RecipeShared/RecipeUnshared events
- is_shared boolean on Recipe aggregate
- Community discovery query filters by is_shared=true

---

#### Story 2.8: Community Recipe Discovery
**As a** user
**I want to** browse recipes shared by others
**So that** I can expand my culinary repertoire

**Prerequisites:** User is authenticated

**Acceptance Criteria:**
1. "Discover Recipes" page displays shared community recipes
2. Recipes displayed in card view with: title, image placeholder, rating, creator name, tags
3. Filters available: rating (4+ stars, 3+ stars), cuisine type, preparation time (<30min, 30-60min, >60min), dietary preferences
4. Search by recipe title or ingredient name
5. Sorting options: highest rated, most recent, most reviewed
6. Clicking recipe card opens detail view with full recipe
7. Community recipes cannot be edited by non-owners
8. "Add to My Recipes" button copies recipe to user's library

**Technical Notes:**
- Read model optimized for discovery queries with filtering
- Pagination for scalability (20 recipes per page)
- Search index on recipe title and ingredient names

---

#### Story 2.9: Rate and Review Community Recipes
**As a** user who cooked a community recipe
**I want to** rate and review it
**So that** I can help others find quality recipes

**Prerequisites:** User has access to shared recipe

**Acceptance Criteria:**
1. Rating widget (1-5 stars) visible on recipe detail page
2. User can rate recipe only once (can update rating)
3. Optional text review field (max 500 characters)
4. Ratings aggregate to show average score (e.g., 4.3/5 from 47 reviews)
5. Reviews displayed chronologically with reviewer username and date
6. User can edit or delete their own review
7. Recipe owner notified of new ratings/reviews (optional notification setting)
8. Highly rated recipes (4+ stars) featured in discovery feed

**Technical Notes:**
- RecipeRated event with rating value and optional review text
- Read model calculates average rating and review count
- Rating submissions tracked to prevent spam

---

#### Story 2.10: Copy Community Recipe to Personal Library
**As a** user browsing community recipes
**I want to** add a community recipe to my library
**So that** I can use it in my meal planning

**Prerequisites:** User viewing shared community recipe

**Acceptance Criteria:**
1. "Add to My Recipes" button visible on community recipe detail page
2. Clicking button copies recipe to user's personal library
3. Copied recipe becomes owned by user (editable)
4. Original creator attribution maintained in metadata
5. Copy counts as new recipe toward free tier limit
6. Copied recipe defaults to private (user can share separately)
7. Modifications to copy don't affect original
8. User can mark copied recipe as favorite for meal planning
9. Confirmation message: "Recipe added to your library"

**Technical Notes:**
- RecipeCopied event with original_recipe_id and new owner
- Full recipe data duplicated to new Recipe aggregate
- Attribution metadata preserved for analytics

---

#### Story 2.11: Tech Debt & Enhancements
**As a** development team
**I want to** address deferred technical improvements from Stories 2.1 and 1.7
**So that** code quality, test coverage, and documentation meet production standards

**Prerequisites:** Stories 2.1 and 1.7 completed

**Acceptance Criteria:**
1. **[Story 2.1 - HIGH]** Instruction reordering UI implemented with drag handles or up/down arrows
2. **[Story 2.1 - HIGH]** Complete test suite written: unit tests for RecipeAggregate, integration tests for HTTP routes, E2E tests for recipe creation flow with Playwright (target 80% code coverage via `cargo tarpaulin`)
3. **[Story 2.1 - MEDIUM]** Form parsing refactored to use Axum extractors (`Form<CreateRecipeForm>`) replacing manual `parse_recipe_form()` logic in `src/routes/recipes.rs:292-340`
4. **[Story 2.1 - MEDIUM]** Structured error handling implemented with AppError enum (DatabaseError, ValidationError, EventStoreError, RecipeLimitError variants) with `IntoResponse` trait for user-friendly error pages
5. **[Story 1.7 - LOW]** Stripe setup guide documented in `docs/stripe-setup.md` or README with instructions for test keys, price creation, and webhook registration
6. All tests pass in CI/CD pipeline
7. Code coverage metrics meet or exceed 80% target (NFR requirement)
8. Documentation reviewed and approved by tech lead

**Technical Notes:**
- Instruction reordering: Use TwinSpark or minimal JavaScript for client-side interaction
- Test framework: Rust `cargo test` for unit/integration, Playwright for E2E
- Axum extractors: Use `axum::Form` with custom deserializer for array fields (ingredient_name[], etc.)
- Error handling: Implement `From<DomainError>` traits for AppError conversions
- Stripe docs: Reference Story 1.7 completion notes for setup instructions

---

**Epic 2 Technical Summary:**
- **Aggregates:** Recipe, Collection
- **Events:** RecipeCreated, RecipeUpdated, RecipeDeleted, RecipeTagged, RecipeFavorited, RecipeUnfavorited, RecipeShared, RecipeUnshared, RecipeRated, RecipeCopied, CollectionCreated, RecipeAddedToCollection
- **Domain Services:** Recipe tagging service (complexity, cuisine, dietary analysis)
- **Testing:** TDD enforced - unit tests for recipe domain logic, integration tests for CRUD operations, E2E tests for community discovery and rating flows

**Technical Specification:** Detailed implementation guide available in `./docs/tech-spec-epic-2.md`

---

### Epic 3: Intelligent Meal Planning Engine

**Goal:** Deliver automated weekly meal plan generation using multi-factor optimization with recipe rotation

**Value Delivered:** Users receive intelligent meal schedules that match recipe complexity to their availability, eliminating planning mental overhead

**Success Criteria:**
- Meal plan generation completes in <5 seconds for up to 50 favorite recipes
- 85% of generated meal plans accepted without modification
- Users cook 3x more unique recipes per month compared to pre-app baseline
- 60% reduction in reported meal planning time

---

#### Story 3.1: Generate Initial Meal Plan
**As a** user with favorite recipes
**I want to** generate an automated weekly meal plan
**So that** I don't have to manually plan meals

**Prerequisites:** User has marked at least 7 favorite recipes (minimum for 1 week breakfast/lunch/dinner)

**Acceptance Criteria:**
1. Home dashboard displays "Generate Meal Plan" button prominently
2. Clicking button triggers meal planning algorithm
3. System analyzes all favorited recipes against user profile constraints
4. Algorithm generates single meal plan with recipes organized by week
5. Week-view calendar displays generated plan with breakfast/lunch/dinner slots filled
6. Generation completes within 5 seconds for up to 50 favorite recipes
7. Progress indicator shown during generation
8. Generated plan automatically becomes active
9. User redirected to calendar view after successful generation
10. If insufficient recipes (<7 favorites), display helpful error: "Add more favorite recipes to generate meal plan (need at least 7)"

**Technical Notes:**
- MealPlan aggregate with MealPlanGenerated event
- Multi-factor optimization algorithm considers: user availability, recipe complexity, advance prep requirements, ingredient freshness
- Recipe rotation ensures no duplicates until all favorites used once
- Algorithm runs synchronously (no background jobs for MVP)

---

#### Story 3.2: Multi-Factor Meal Planning Algorithm
**As a** system
**I want to** optimize meal assignments based on multiple factors
**So that** meal plans are realistic and achievable for users

**Prerequisites:** User initiates meal plan generation

**Acceptance Criteria:**
1. Algorithm analyzes user profile: weeknight availability, cooking skill level, household size
2. Recipes scored on complexity: ingredient count, instruction steps, advance prep requirements
3. Complex recipes assigned to days with more availability (weekends, evenings with >60min)
4. Simple recipes assigned to busy weeknights (<45min availability)
5. Advance prep recipes scheduled to allow proper lead time (4-hour marinade on Tue for Wed dinner)
6. Recipe dietary tags matched against user dietary restrictions (no shellfish if allergic)
7. Ingredient freshness considered (produce-heavy meals earlier in week)
8. Equipment conflicts avoided (no two oven-dependent meals back-to-back)
9. Algorithm deterministic but varied (same inputs produce different valid plans on regeneration)

**Technical Notes:**
- Scoring function: complexity_score = (ingredients * 0.3) + (steps * 0.4) + (advance_prep_multiplier * 0.3)
- Constraint satisfaction problem (CSP) solver for assignment
- Randomization seed varies per generation for variety
- Performance: O(n) complexity where n = favorite recipe count

---

#### Story 3.3: Recipe Rotation System
**As a** user
**I want** recipes to rotate without duplicates
**So that** I experience maximum variety before repeating meals

**Prerequisites:** User has generated meal plan

**Acceptance Criteria:**
1. Meal planning algorithm tracks which recipes have been used in current rotation cycle
2. Each favorite recipe used exactly once before any recipe repeats
3. After all favorites used once, rotation cycle resets and recipes become available again
4. Rotation state persists across meal plan regenerations
5. Manually replacing individual meals respects rotation (only offers unused recipes)
6. Adding new favorite mid-rotation includes it in pool immediately
7. Un-favoriting recipe removes from rotation without disrupting active plan
8. Rotation progress visible to user: "12 of 20 favorite recipes used this cycle"

**Technical Notes:**
- RecipeRotationCycleStarted, RecipeUsedInRotation events
- Read model tracks: recipe_id, last_used_date, rotation_cycle_number
- Query filters recipes by: is_favorited=true AND NOT used_in_current_cycle

---

#### Story 3.4: Visual Week-View Meal Calendar
**As a** user
**I want to** see my meal plan in calendar format
**So that** I can quickly understand my week at a glance

**Prerequisites:** User has active meal plan

**Acceptance Criteria:**
1. Calendar displays 7 days (Sunday-Saturday or Monday-Sunday based on locale)
2. Each day shows 3 meal slots: breakfast, lunch, dinner
3. Each slot displays: recipe title, recipe image placeholder, prep time indicator
4. Advance preparation indicator (clock icon) visible on recipes requiring prep
5. Complexity badge (Simple/Moderate/Complex) displayed per recipe
6. Today's date highlighted with distinct styling
7. Past dates dimmed/grayed out
8. Future dates fully interactive (clickable for details)
9. Empty slots show "No meal planned" with action to add
10. Mobile-responsive: stacks vertically on small screens, grid on tablet/desktop

**Technical Notes:**
- Read model: MealPlanCalendarView with meal_plan_id, date, meal_type (breakfast/lunch/dinner), recipe_id
- TwinSpark for interactive calendar updates without full page reload
- Tailwind responsive grid: mobile (1 column), tablet (2 columns), desktop (7 columns)

---

#### Story 3.5: View Recipe Details from Calendar
**As a** user viewing meal plan calendar
**I want to** click a meal to see full recipe details
**So that** I can review instructions before cooking

**Prerequisites:** User viewing active meal plan with assigned recipes

**Acceptance Criteria:**
1. Clicking recipe card on calendar opens recipe detail modal/page
2. Recipe detail displays: title, full ingredient list, step-by-step instructions with optional timers, prep/cook times, advance prep requirements
3. Dietary tags and complexity badge visible
4. "Replace This Meal" button available for quick substitution
5. Back/close navigation returns to calendar view
6. Recipe detail page optimized for kitchen use (large text, high contrast)
7. Instructions viewable in progressive disclosure (expand step-by-step)

**Technical Notes:**
- Recipe detail route: /recipes/{recipe_id}
- Askama template for server-rendered HTML
- TwinSpark for modal behavior on non-mobile devices

---

#### Story 3.6: Replace Individual Meal Slot
**As a** user
**I want to** replace a single meal in my plan
**So that** I can adjust for schedule changes or preferences

**Prerequisites:** User has active meal plan

**Acceptance Criteria:**
1. "Replace This Meal" button visible on each calendar slot
2. Clicking button triggers meal replacement for that specific slot
3. System offers alternative recipes matching same meal type (breakfast/lunch/dinner)
4. Alternatives respect rotation (only unused recipes offered)
5. Alternatives match or improve user constraints for that day (complexity vs availability)
6. User selects replacement from list (3-5 options)
7. Selected recipe immediately replaces meal in calendar
8. Replaced recipe returned to rotation pool (becomes available again)
9. Shopping list automatically updates with new ingredient requirements
10. Confirmation message: "Meal replaced successfully"

**Technical Notes:**
- MealSlotReplaced event with old_recipe_id, new_recipe_id, date, meal_type
- Algorithm generates candidate recipes filtered by rotation and constraints
- Shopping list recalculation triggered by event subscription

---

#### Story 3.7: Regenerate Full Meal Plan
**As a** user
**I want to** completely regenerate my meal plan
**So that** I can get fresh variety or restart after disruptions

**Prerequisites:** User has active meal plan

**Acceptance Criteria:**
1. "Regenerate Meal Plan" button visible on calendar page
2. Confirmation dialog: "This will replace your entire meal plan. Continue?"
3. Clicking confirm triggers full meal plan regeneration
4. Algorithm runs with same logic as initial generation
5. Rotation state preserved (doesn't reset cycle)
6. New plan fills all slots with different recipe assignments
7. Calendar updates to show new plan
8. Shopping list regenerated for new plan
9. Old meal plan archived for audit trail
10. Generation respects same optimization factors (availability, complexity, prep timing)

**Technical Notes:**
- MealPlanRegenerated event
- Soft delete old meal plan (keeps history)
- New MealPlan aggregate created with incremented version

---

#### Story 3.8: Algorithm Transparency (Show Reasoning)
**As a** user
**I want to** understand why meals were assigned to specific days
**So that** I trust the automated system

**Prerequisites:** User viewing meal plan calendar

**Acceptance Criteria:**
1. Hovering over (or tapping) info icon on meal slot shows reasoning tooltip
2. Reasoning displays: "Assigned to Saturday: more prep time available (Complex recipe, 75min total time)"
3. Or: "Assigned to Tuesday: Quick weeknight meal (Simple recipe, 30min total time)"
4. Or: "Prep tonight for tomorrow: Requires 4-hour marinade"
5. Reasoning adapts to actual assignment factors used by algorithm
6. Clear, human-readable language (no technical jargon)
7. Builds user trust in intelligent automation

**Technical Notes:**
- Store assignment reasoning in MealPlanSlot read model
- Reasoning generated during algorithm execution
- Template renders reasoning in tooltip/popover

---

#### Story 3.9: Home Dashboard with Today's Meals
**As a** user
**I want to** see today's meals on my dashboard
**So that** I immediately know what to cook without navigating

**Prerequisites:** User has active meal plan

**Acceptance Criteria:**
1. Home dashboard prominently displays "Today's Meals" section at top
2. Shows breakfast, lunch, and dinner assigned for today
3. Each meal displays: recipe title, image, prep time
4. Advance prep indicator if preparation required today for future meal
5. "View Full Calendar" link to navigate to week view
6. If no meal plan active, displays "Generate Meal Plan" call-to-action
7. Today's meals update automatically at midnight (new day)
8. Click recipe navigates to full recipe detail

**Technical Notes:**
- Dashboard route: /dashboard (root after login)
- Query: SELECT * FROM meal_plan_calendar WHERE user_id = ? AND date = TODAY()
- Server-rendered with Askama template

---

#### Story 3.10: Handle Insufficient Recipes for Generation
**As a** user with too few favorite recipes
**I want** clear guidance on what's needed
**So that** I can successfully generate a meal plan

**Prerequisites:** User has <7 favorite recipes

**Acceptance Criteria:**
1. "Generate Meal Plan" button visible but triggers validation
2. Error message: "You need at least 7 favorite recipes to generate a weekly meal plan. You currently have {count}."
3. Helpful guidance: "Add {7 - count} more recipes to get started!"
4. Direct link to "Add Recipe" page or "Discover Recipes" page
5. Error displayed with friendly styling (not alarming red)
6. Validation prevents wasted algorithm execution
7. Count updates in real-time as user adds/removes favorites

**Technical Notes:**
- Pre-flight validation before algorithm execution
- Read model query: COUNT(recipes WHERE is_favorited=true AND user_id=?)
- Minimum threshold configurable (default 7 for 7 days * 1 meal, flexible for MVP)

---

#### Story 3.11: Meal Plan Persistence and Activation
**As a** user
**I want** my meal plan to persist across sessions
**So that** I don't lose my schedule

**Prerequisites:** User has generated meal plan

**Acceptance Criteria:**
1. Generated meal plan stored in database
2. Exactly one meal plan active per user at a time
3. Active meal plan automatically loaded on dashboard/calendar views
4. Meal plan persists across browser sessions and device switches
5. Regeneration archives old plan and creates new active plan
6. Active plan indicated by is_active flag in database
7. Historical meal plans accessible for review (out of MVP scope, but data preserved)

**Technical Notes:**
- MealPlan aggregate with is_active boolean
- Only one MealPlan can have is_active=true per user (constraint enforced)
- Event sourcing maintains full history of all generated plans

---

#### Story 3.12: Recipe Complexity Calculation
**As a** system
**I want to** accurately calculate recipe complexity
**So that** meal assignments match user capacity

**Prerequisites:** Recipe exists with ingredients, instructions, advance prep data

**Acceptance Criteria:**
1. Complexity calculated on recipe creation/update
2. Scoring factors: ingredient count (weight 30%), instruction step count (weight 40%), advance prep requirement (weight 30%)
3. Simple: <8 ingredients, <6 steps, no advance prep (score <30)
4. Moderate: 8-15 ingredients OR 6-10 steps (score 30-60)
5. Complex: >15 ingredients OR >10 steps OR advance prep required (score >60)
6. Complexity badge stored in recipe read model for fast filtering
7. Recalculated automatically when recipe edited
8. Complexity visible on recipe cards throughout app

**Technical Notes:**
- Domain service: RecipeComplexityCalculator
- Formula: (ingredients * 0.3) + (steps * 0.4) + (advance_prep_multiplier * 0.3)
- advance_prep_multiplier: 0 if none, 50 if <4 hours, 100 if >=4 hours
- Result mapped to enum: Simple, Moderate, Complex

---

**Epic 3 Technical Summary:**
- **Aggregates:** MealPlan, MealPlanSlot
- **Events:** MealPlanGenerated, MealPlanRegenerated, MealSlotReplaced, RecipeRotationCycleStarted, RecipeUsedInRotation
- **Domain Services:** MealPlanningAlgorithm (CSP solver), RecipeComplexityCalculator
- **Algorithm Performance:** O(n) where n = favorite recipe count, <5 second target
- **Testing:** TDD enforced - unit tests for algorithm logic, integration tests for meal plan CRUD, E2E tests for generation and replacement flows, property-based testing for rotation invariants

**Technical Specification:** Detailed implementation guide available in `./docs/tech-spec-epic-3.md`

---

### Epic 4: Shopping and Preparation Orchestration

**Goal:** Automate shopping list generation and provide timely preparation reminders for advance-prep recipes

**Value Delivered:** Users get organized shopping lists and actionable reminders ensuring successful execution of complex recipes

**Success Criteria:**
- Shopping list generation completes in <2 seconds
- 80% of users regularly use shopping lists
- 90% of advance prep reminders result in completed preparation tasks
- 40% reduction in ingredient waste through aggregated shopping

---

#### Story 4.1: Generate Weekly Shopping List
**As a** user with an active meal plan
**I want** an automated shopping list for the week
**So that** I can efficiently shop for all required ingredients

**Prerequisites:** User has active meal plan with assigned recipes

**Acceptance Criteria:**
1. "Shopping List" button visible on dashboard and calendar pages
2. Clicking button generates shopping list for current week
3. System aggregates all ingredients from week's recipes
4. Duplicate ingredients combined with quantities summed (e.g., "onions 2" + "onions 3" = "onions 5")
5. Units normalized for aggregation (convert 1 cup to 240ml, combine with ml measurements)
6. Ingredients grouped by category: Produce, Dairy, Meat, Pantry, Frozen, Bakery, Other
7. Shopping list displays item count per category
8. Generation completes within 2 seconds
9. Shopping list persists and accessible for offline use
10. Confirmation: "Shopping list generated for Week of {date}"

**Technical Notes:**
- ShoppingList aggregate with ShoppingListGenerated event
- Ingredient aggregation logic in domain service
- Unit conversion table for common measurements
- Category mapping based on ingredient type

---

#### Story 4.2: Category-Based Ingredient Grouping
**As a** user shopping in a grocery store
**I want** ingredients grouped by store section
**So that** I can shop efficiently without backtracking

**Prerequisites:** Shopping list generated

**Acceptance Criteria:**
1. Shopping list displays collapsible sections per category
2. Default categories: Produce, Dairy, Meat & Seafood, Pantry, Frozen, Bakery, Other
3. Each category shows item count (e.g., "Produce (8 items)")
4. Items within category listed alphabetically
5. User can expand/collapse categories
6. All categories expanded by default on first view
7. Category order matches typical grocery store layout
8. Empty categories hidden from view

**Technical Notes:**
- Category enum: Produce, Dairy, Meat, Pantry, Frozen, Bakery, Other
- Ingredient-to-category mapping service
- Default mappings: tomato→Produce, milk→Dairy, chicken→Meat, etc.
- User category customization out of MVP scope

---

#### Story 4.3: Multi-Week Shopping List Access
**As a** user
**I want to** view shopping lists for current and future weeks
**So that** I can plan bulk shopping or shop ahead

**Prerequisites:** User has active meal plan

**Acceptance Criteria:**
1. Shopping list page displays week selector dropdown
2. Options: "This Week", "Next Week", "Week of {date}" for upcoming weeks
3. Selecting week generates shopping list for that week's meals
4. Current week highlighted as default
5. Future weeks accessible up to 4 weeks ahead
6. Each week's shopping list independent (no cross-week aggregation)
7. Past weeks not accessible (out of scope for MVP)
8. Week selection persists in URL query param for bookmarking

**Technical Notes:**
- Query param: ?week=2025-10-13 (ISO week start date)
- Shopping list generation on-demand per week
- Read model query filters meals by date range (week start to end)

---

#### Story 4.4: Shopping List Real-Time Updates
**As a** user
**I want** my shopping list to update when I change meals
**So that** it always reflects my current meal plan

**Prerequisites:** User has active meal plan and shopping list

**Acceptance Criteria:**
1. Replacing a meal slot triggers shopping list recalculation
2. Removed recipe's ingredients subtracted from list
3. New recipe's ingredients added to list
4. Quantity aggregation recalculated
5. Shopping list page auto-refreshes to show changes (if open)
6. No duplicate shopping lists created - existing list updated
7. Updates complete within 1 second of meal replacement
8. User notification: "Shopping list updated"

**Technical Notes:**
- Event subscription: on MealSlotReplaced → trigger ShoppingListRecalculated
- CQRS read model updated via projection
- TwinSpark for live updates without page reload

---

#### Story 4.5: Shopping List Item Checkoff (Optional)
**As a** user shopping at the store
**I want to** check off items as I collect them
**So that** I track progress and avoid missing items

**Prerequisites:** User has shopping list open

**Acceptance Criteria:**
1. Each shopping list item has checkbox
2. Tapping/clicking checkbox marks item as collected (strike-through styling)
3. Checked state persists across page refreshes
4. Progress indicator at top: "{checked} of {total} items collected"
5. Filter options: "Show All", "Show Remaining", "Show Collected"
6. Checked items move to bottom of category section
7. Checking all items in category collapses that section automatically
8. Reset button to uncheck all items (for next shopping trip)

**Technical Notes:**
- ShoppingListItemChecked event
- Checked state stored in read model per user, per shopping list
- LocalStorage backup for offline checkbox persistence

---

#### Story 4.6: Advance Preparation Reminder System
**As a** user with advance-prep recipes in meal plan
**I want** timely reminders for preparation tasks
**So that** I successfully execute complex recipes

**Prerequisites:** Active meal plan includes recipes with advance prep requirements

**Acceptance Criteria:**
1. System scans meal plan for recipes with advance prep (marinade, rising, chilling, etc.)
2. Reminders scheduled automatically when meal plan generated
3. Reminder timing calculated from advance prep requirement and meal schedule
4. Example: "Marinate chicken 4 hours before" for Wednesday dinner → reminder sent Tuesday evening or Wednesday morning
5. Reminders delivered via push notification (if enabled)
6. Reminder displays: recipe name, specific prep task, timing guidance
7. Tapping reminder opens recipe detail with prep instructions highlighted
8. User can snooze reminder (1 hour, 2 hours, 4 hours)

**Technical Notes:**
- PrepReminder aggregate with PrepReminderScheduled event
- Background job scheduler for reminder delivery (cron or task queue)
- Push notification service integration (Web Push API for PWA)
- Reminder calculation logic in domain service

---

#### Story 4.7: Morning Preparation Reminders
**As a** user
**I want** morning reminders for tonight's advance prep
**So that** I remember to prepare before leaving for work

**Prerequisites:** Meal plan includes advance-prep recipe requiring same-day preparation

**Acceptance Criteria:**
1. Morning reminders sent at 9:00 AM local time
2. Reminder content: "Prep reminder: {task} tonight for tomorrow's {meal}"
3. Example: "Prep reminder: Marinate chicken tonight for Thursday's dinner"
4. Reminder includes estimated prep time (e.g., "Takes 10 minutes")
5. Deep link to recipe detail page
6. Only sent if advance prep required within next 24 hours
7. User can customize reminder time in settings (future: out of MVP scope)
8. Reminder dismissed automatically after prep window passes

**Technical Notes:**
- Scheduled job runs daily at 9:00 AM per user timezone
- Query: recipes with advance_prep_hours <= 24 AND scheduled_date = tomorrow
- Notification template with recipe and task placeholders

---

#### Story 4.8: Day-of Cooking Reminders
**As a** user
**I want** reminders for today's meals
**So that** I remember to cook on schedule

**Prerequisites:** User has active meal plan with today's meals

**Acceptance Criteria:**
1. Cooking reminder sent 1 hour before typical meal time
2. Default meal times: Breakfast 8am, Lunch 12pm, Dinner 6pm
3. Reminder content: "Tonight's dinner: {recipe_name} - Ready in {total_time}"
4. Reminder displays recipe image and key info
5. Tapping opens recipe detail in cooking mode
6. User can dismiss or snooze (30 min, 1 hour)
7. Reminder respects user profile availability settings
8. No reminder sent if meal already marked as completed (out of MVP scope)

**Technical Notes:**
- Scheduled reminders based on meal type and user preferences
- Default times configurable, future: user customization
- Notification with action buttons: "View Recipe", "Dismiss"

---

#### Story 4.9: Prep Task Completion Tracking
**As a** user
**I want to** mark prep tasks as complete
**So that** I track my preparation progress

**Prerequisites:** User received advance prep reminder

**Acceptance Criteria:**
1. Advance prep reminders include "Mark Complete" button
2. Clicking marks task as completed
3. Completed tasks display checkmark on dashboard
4. Dashboard shows "Prep Tasks for Today" section with completion status
5. Completed tasks removed from active reminders
6. Completion tracked per recipe, per meal slot
7. Uncompleted tasks carried over to next reminder cycle
8. Recipe detail page shows prep completion status

**Technical Notes:**
- PrepTaskCompleted event
- Read model tracks prep_task completion per meal_plan_slot_id
- Dashboard query: prep tasks for today WHERE NOT completed

---

#### Story 4.10: Push Notification Permission Flow
**As a** user
**I want to** enable push notifications for reminders
**So that** I receive timely preparation alerts

**Prerequisites:** User logged in on PWA-capable browser

**Acceptance Criteria:**
1. Onboarding includes notification permission prompt
2. Prompt explains benefits: "Get reminders for advance prep and cooking times"
3. User can allow, deny, or skip
4. If allowed, register service worker and subscription
5. If denied, fall back to in-app notifications only
6. User can change permission in browser settings
7. Settings page shows current notification status
8. Grace period: don't re-prompt if user denied within last 30 days

**Technical Notes:**
- Web Push API for PWA notifications
- Service worker registration for background notifications
- Push subscription stored per user device
- Notification permission state tracked in user preferences

---

#### Story 4.11: Ingredient Quantity Aggregation Logic
**As a** system
**I want to** accurately aggregate ingredient quantities
**So that** shopping lists show correct totals

**Prerequisites:** Multiple recipes in meal plan use same ingredient

**Acceptance Criteria:**
1. System identifies duplicate ingredients by normalized name (case-insensitive, trim whitespace)
2. Quantities with same unit added directly (2 cups + 1 cup = 3 cups)
3. Quantities with compatible units converted then added (1 cup + 240ml = 2 cups)
4. Incompatible units kept separate (1 whole onion + 1 cup diced onion = separate line items)
5. Unit conversion table: cups↔ml, tablespoons↔teaspoons, lbs↔oz, grams↔kg
6. Fractional quantities handled: 1/2 cup + 1/4 cup = 3/4 cup
7. Aggregated quantities rounded to practical values (avoid "2.347 cups" → "2 1/3 cups")
8. Ambiguous quantities flagged for manual review (e.g., "a pinch" + "to taste")

**Technical Notes:**
- IngredientAggregationService domain service
- Unit conversion constants table
- Fraction arithmetic library for precise calculations
- Normalized ingredient name matching with fuzzy logic (optional enhancement)

---

**Epic 4 Technical Summary:**
- **Aggregates:** ShoppingList, PrepReminder
- **Events:** ShoppingListGenerated, ShoppingListRecalculated, ShoppingListItemChecked, PrepReminderScheduled, PrepTaskCompleted
- **Domain Services:** IngredientAggregationService, PrepReminderScheduler
- **External Integrations:** Web Push API for notifications, service worker for background tasks
- **Testing:** TDD enforced - unit tests for aggregation logic, integration tests for shopping list generation, E2E tests for notification flows

**Technical Specification:** Detailed implementation guide available in `./docs/tech-spec-epic-4.md`

---

### Epic 5: Progressive Web App and Mobile Experience

**Goal:** Deliver installable PWA with offline capabilities and kitchen-optimized mobile interface

**Value Delivered:** Users access recipes and meal plans in kitchen environment without connectivity concerns, with touch-optimized interface

**Success Criteria:**
- PWA installable on iOS Safari 14+ and Android Chrome 90+
- Offline recipe access works 100% of the time once cached
- <3 second load time on 3G connections
- Touch targets meet 44x44px minimum across all interactive elements
- 80% of mobile users complete tasks without usability issues

---

#### Story 5.1: PWA Manifest and Installation
**As a** user on mobile device
**I want to** install imkitchen as an app
**So that** I can access it like a native app from my home screen

**Prerequisites:** User accesses imkitchen via mobile browser

**Acceptance Criteria:**
1. PWA manifest file (manifest.json) configured with app metadata
2. Manifest includes: app name, short_name, description, icons (192x192, 512x512), start_url, display mode (standalone), theme_color, background_color
3. Browser prompts user to install app after engagement threshold met (2+ visits)
4. User can manually trigger installation via browser menu or in-app prompt
5. Installed app opens in standalone mode (no browser chrome)
6. App icon appears on device home screen with correct branding
7. Splash screen displays while app loading (uses background_color and icon)
8. Works on iOS Safari 14+ and Android Chrome 90+

**Technical Notes:**
- manifest.json served from root with correct MIME type (application/manifest+json)
- Apple touch icons for iOS support (apple-touch-icon.png)
- Service worker required for PWA installation
- Axum static file serving for manifest and assets

---

#### Story 5.2: Service Worker for Offline Support
**As a** user
**I want** app to work offline
**So that** I can access recipes in kitchen without internet

**Prerequisites:** User has visited app and service worker registered

**Acceptance Criteria:**
1. Service worker registered on first app visit
2. Service worker caches critical assets: HTML, CSS, JS, fonts, images
3. Recipe pages cached after first view
4. Offline-first strategy: serve from cache, fallback to network
5. Network-first for HTML requests with cache fallback
6. Graceful offline indicator when network unavailable
7. Background sync queues actions taken offline (favorite recipe, mark prep complete) for later sync
8. Cache versioning ensures updates deployed without breaking offline experience

**Technical Notes:**
- Service worker file: /sw.js served from root
- Cache strategies: CacheFirst for static assets, NetworkFirst for HTML pages
- Cache versioning: sw-v1, sw-v2 for cache busting on updates
- Background Sync API for offline action queueing

---

#### Story 5.3: Offline Recipe Access
**As a** user in kitchen without internet
**I want to** view cached recipes
**So that** I can cook without connectivity

**Prerequisites:** User previously viewed recipes while online

**Acceptance Criteria:**
1. Recipe detail pages cached automatically after first view
2. Offline access includes: full recipe data, ingredients, instructions, images
3. User can view any previously accessed recipe offline
4. Active meal plan accessible offline with all assigned recipes
5. Shopping list accessible offline with checkoff functionality
6. Offline changes (checkoff items, mark prep complete) persist locally
7. Changes sync to server when connectivity restored
8. Offline indicator clearly visible when no connection
9. "Offline mode" messaging doesn't alarm user (neutral styling)

**Technical Notes:**
- IndexedDB for offline data persistence
- Service worker intercepts recipe requests, serves from cache
- LocalStorage for checkbox states and prep completions
- Sync queue processes pending updates on reconnection

---

#### Story 5.4: Mobile-Responsive Design
**As a** user on mobile device
**I want** optimized interface for small screens
**So that** I can use app comfortably on phone

**Prerequisites:** User accesses app on mobile device

**Acceptance Criteria:**
1. Responsive breakpoints: mobile (<768px), tablet (768-1024px), desktop (>1024px)
2. Mobile layout: single-column stacking, full-width cards, bottom navigation
3. Tablet layout: 2-column grid for recipe cards, side navigation
4. Desktop layout: multi-column grid, persistent sidebar navigation
5. Text sizes scale appropriately (16px minimum for body text on mobile)
6. Images responsive with srcset for different screen densities
7. Form inputs sized appropriately for thumb typing
8. Navigation accessible without excessive scrolling

**Technical Notes:**
- Tailwind CSS responsive utilities (@sm, @md, @lg, @xl)
- Mobile-first CSS approach (base styles for mobile, enhance for larger screens)
- Flexible grid system with CSS Grid and Flexbox
- Viewport meta tag: width=device-width, initial-scale=1

---

#### Story 5.5: Touch-Optimized Interface
**As a** user interacting via touchscreen
**I want** touch targets large enough to tap accurately
**So that** I avoid mis-taps and frustration

**Prerequisites:** User on touch-enabled device

**Acceptance Criteria:**
1. All interactive elements (buttons, links, checkboxes) minimum 44x44px tap target
2. Adequate spacing between adjacent tap targets (8px minimum)
3. No hover-dependent interactions (avoid :hover for critical functionality)
4. Touch gestures intuitive: swipe to dismiss, pull to refresh (where appropriate)
5. Haptic feedback on button taps (where browser supports)
6. Long-press menus for contextual actions
7. Scrolling smooth and responsive (no janky scroll performance)
8. Pinch-to-zoom disabled for app UI, enabled for recipe images

**Technical Notes:**
- CSS: min-height: 44px, min-width: 44px for interactive elements
- Touch-action CSS property to control gestures
- Passive event listeners for scroll performance
- User-scalable=no in viewport meta for app chrome, allow for images

---

#### Story 5.6: Kitchen-Friendly Display Modes
**As a** user cooking in kitchen
**I want** high-contrast, large-text display option
**So that** I can read recipes in various lighting conditions

**Prerequisites:** User viewing recipe while cooking

**Acceptance Criteria:**
1. "Kitchen Mode" toggle in recipe detail view
2. Kitchen mode increases text size (20px body, 28px headings)
3. High contrast styling: dark text on light background, increased contrast ratio (7:1)
4. Simplified UI: hide non-essential elements, focus on instructions
5. Step-by-step mode: display one instruction at a time with large "Next" button
6. Keep-awake functionality prevents screen from sleeping while cooking
7. Mode persists across recipe views (stored in user preference)
8. Easy toggle to return to normal mode

**Technical Notes:**
- CSS class: .kitchen-mode applied to recipe container
- Wake Lock API to prevent screen sleep (if supported)
- LocalStorage stores kitchen_mode preference
- Alternative: URL param ?mode=kitchen for sharing

---

#### Story 5.7: Cross-Browser Compatibility
**As a** user on any modern browser
**I want** consistent experience
**So that** app works regardless of browser choice

**Prerequisites:** User accesses app from supported browser

**Acceptance Criteria:**
1. Full functionality on iOS Safari 14+, Android Chrome 90+
2. Graceful degradation on older browsers (show fallback UI)
3. Feature detection for PWA APIs (service worker, Web Push, Wake Lock)
4. Polyfills for missing features where feasible
5. No browser-specific bugs affecting core functionality
6. Consistent visual rendering across browsers (CSS normalization)
7. Form inputs work correctly on all platforms (date pickers, dropdowns)
8. JavaScript compatibility via transpilation (ES2015+ support)

**Technical Notes:**
- Browserslist config targets: iOS Safari >= 14, Chrome >= 90, Firefox >= 88
- Autoprefixer for CSS vendor prefixes
- Babel for JavaScript transpilation (if needed, Rust outputs WASM/JS)
- Feature detection: if ('serviceWorker' in navigator)

---

#### Story 5.8: Real-Time Sync When Connectivity Restored
**As a** user who made changes offline
**I want** changes automatically synced when online
**So that** I don't lose my work

**Prerequisites:** User made changes offline (favorited recipe, checked shopping list item)

**Acceptance Criteria:**
1. Background Sync API detects network restoration
2. Queued changes sent to server in order
3. Conflicts resolved gracefully (server state wins, user notified)
4. Sync progress indicator shows while syncing
5. Success confirmation: "Changes synced"
6. Failure handling: retry up to 3 times, then notify user
7. Sync does not block user interaction
8. Large data changes batched to reduce network load

**Technical Notes:**
- Background Sync API registration: navigator.serviceWorker.ready.then(reg => reg.sync.register('sync-changes'))
- Sync queue stored in IndexedDB
- Sync event in service worker processes queue
- Exponential backoff for retries

---

#### Story 5.9: App Performance Optimization
**As a** user on slower connection
**I want** fast load times
**So that** I'm not waiting for pages

**Prerequisites:** User accessing app

**Acceptance Criteria:**
1. Initial load <3 seconds on 3G connection
2. Subsequent page navigation <1 second (cached resources)
3. Images lazy-loaded below fold
4. Critical CSS inlined in HTML head
5. JavaScript bundles split for code splitting
6. Server-side rendering for initial page load (Askama templates)
7. Brotli compression for all text assets
8. CDN for static assets (future: out of MVP scope)

**Technical Notes:**
- Lighthouse performance score >90
- Web Vitals targets: LCP <2.5s, FID <100ms, CLS <0.1
- Axum Brotli compression middleware
- Lazy loading: loading="lazy" on img tags
- TwinSpark minimizes JavaScript payload

---

#### Story 5.10: Installable App Experience
**As a** user who installed the PWA
**I want** native app-like experience
**So that** it feels like a real mobile app

**Prerequisites:** User installed PWA

**Acceptance Criteria:**
1. App opens in standalone mode (no browser UI)
2. Status bar color matches app theme
3. Orientation locks to portrait for consistency (override for tablet)
4. App switcher shows app icon and name
5. Deep links open within app (not new browser tab)
6. Splash screen on app launch while loading
7. Gesture navigation feels native (swipe back, pull to refresh)
8. No web-like artifacts (address bar, browser controls)

**Technical Notes:**
- display: "standalone" in manifest
- theme-color meta tag and manifest theme_color
- Orientation: portrait-primary in manifest (optional)
- Handle app protocol links for deep linking

---

**Epic 5 Technical Summary:**
- **Technical Components:** PWA manifest, service worker, cache strategies, background sync
- **Browser APIs:** Service Worker API, Cache API, Background Sync API, Wake Lock API, Web Push API
- **Performance Targets:** <3s initial load, <1s subsequent loads, LCP <2.5s
- **Compatibility:** iOS Safari 14+, Android Chrome 90+, progressive enhancement for older browsers
- **Testing:** TDD enforced - unit tests for cache logic, integration tests for sync mechanisms, E2E Playwright tests for PWA installation and offline scenarios, browser compatibility testing matrix

**Technical Specification:** Detailed implementation guide available in `./docs/tech-spec-epic-5.md`

---

## Epic Summary

**Total Stories Across All Epics:** 50 stories
- Epic 1: 8 stories (Authentication and Profile)
- Epic 2: 10 stories (Recipe Management)
- Epic 3: 12 stories (Meal Planning Engine)
- Epic 4: 11 stories (Shopping and Preparation)
- Epic 5: 9 stories (PWA and Mobile)

**Estimated Timeline:** 5-8 months to MVP launch with solo developer or small team

**Next Steps:** This epic breakdown provides architect with detailed requirements for technical design. Proceed to architecture workflow (3-solutioning) for technical specifications, database schema, HTML endpoint design, and implementation roadmap.

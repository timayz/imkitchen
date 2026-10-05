# Project Brief: imkitchen

## Executive Summary

imkitchen is a cooking companion that eliminates the mental overhead that prevents home cooks from exploring their full recipe repertoire. Each user keeps one **recipe list**: the set of recipes they intend to cook next. The list can be generated in one tap from the user's own and saved community recipes, with intelligent accompaniment pairing and dietary-restriction filtering, or assembled by hand recipe by recipe. The Kitchen page always shows what to cook next, a guided cooking mode walks through each recipe, and a Groceries page keeps a merged, household-scaled ingredient list in sync with the list at all times—transforming cooking from a stressful daily decision into an effortless, enjoyable experience.

**Primary Problem:** Home cooks artificially limit their recipe choices to avoid preparation complexity and the daily "what's for dinner" decision, resulting in culinary monotony and underutilized recipe collections.

**Target Market:** Home cooking enthusiasts and busy families who want variety in their meals but struggle with choosing, preparing, and composing realistic meals.

**Key Value Proposition:** One generated, always-ready queue of recipes that unlocks access to complex recipes by handling selection, accompaniment pairing, prep-ahead visibility, and grocery aggregation automatically while respecting dietary restrictions.

## Problem Statement

### Current State and Pain Points

Home cooks face a fundamental choice between culinary variety and planning simplicity. Current approaches force users to manually coordinate:

- Recipe selection based on available time and energy
- Advance preparation requirements (marination, rising, chilling)
- Ingredient shopping across several recipes at once
- Realistic meal composition (main dishes paired with appropriate accompaniments)
- Equipment conflicts and kitchen workflow
- Family schedule coordination
- Dietary restrictions matching

This complexity creates a **self-limitation pattern** where users:
- Avoid recipes requiring advance preparation
- Skip recipes that need accompaniments (rice, pasta, sides) due to coordination overhead
- Decide one meal at a time, missing bulk shopping opportunities
- Ignore their actual dietary needs when selecting recipes
- Result in 70-80% of saved recipes never being cooked

Users maintain browser favorites with hundreds of recipes but repeatedly cook only 10-15 simple ones that require no accompaniments and minimal planning.

### Impact of the Problem

- **Culinary Stagnation:** Users cook 70-80% fewer recipes than they save
- **Decision Fatigue:** Daily "what's for dinner" stress compounds meal preparation burden
- **Ingredient Waste:** Poor coordination leads to unused ingredients and rushed shopping
- **Family Conflict:** Last-minute meal decisions create household tension
- **Lost Opportunities:** Complex but rewarding recipes remain untried

### Why Existing Solutions Fall Short

Current meal planning apps focus on recipe storage and calendar scheduling without addressing the core selection and meal composition problems. They require manual coordination of:

- Which recipes to cook next, and in what order
- Shopping across several recipes at once
- Accompaniment pairing (rice with curry, pasta with sauce, sides with mains)
- Dietary restrictions filtering
- Variety across the recipes chosen
- Real-time disruption handling when a planned day slips

**Critical Gap:** No existing solution replaces the calendar with a simple generated queue of recipes, with intelligent accompaniment pairing, preference-aware selection, and an always-current grocery list.

### Urgency and Importance

The home cooking market has expanded significantly post-2020, with families cooking 40% more meals at home. This creates both increased demand for cooking solutions and heightened frustration with planning complexity. The timing is optimal for an intelligent automation solution.

## Proposed Solution

### Core Concept and Approach

imkitchen replaces the meal calendar with a single per-user **recipe list** and automates everything around it:

- **One-Tap Generation:** Pick up to N main courses (1 to 30, default 7) at random from the user's candidate pool (their own recipes plus community recipes they saved), filtered by dietary restrictions and spread across cuisines according to the "cuisine variety" weight
- **Intelligent Accompaniment Pairing:** For each optional course type enabled in preferences (appetizer, accompaniment, dessert, beverage, condiment), add up to half as many recipes of that type; accompaniments are only added when a picked main course accepts one
- **No Repeats:** A recipe never appears twice in the list; if fewer recipes exist than requested, the list is simply shorter
- **Manual Control:** Any recipe page (own or shared) has an "Add to my list" button; recipes can be removed from the Kitchen page at any time
- **Cook Next, Not "Today":** The Kitchen page surfaces the next uncooked recipe, ordered by course (starter → main → side → dessert → drink → sauce) then list order, with no dates attached
- **Guided Cooking Mode:** Ingredients first, then step by step with timers; a completed recipe stays in the list marked "Cooked" until removed or the list is regenerated
- **Always-Current Groceries:** The merged, household-size-scaled ingredient list of every recipe in the list, grouped by aisle, updated automatically whenever the list changes
- **Regeneration on Demand:** Regenerating replaces the whole list, clears checked groceries and resets cooking progress

### Key Differentiators

1. **Recipe List Instead of Calendar:** No day slots to fill, no locked weeks, no plan going stale when life intervenes—users cook the next recipe whenever they are ready
2. **Intelligent Accompaniment System:** Main courses opt in to accompaniments (pasta, rice, fries, salad, bread, vegetables) and generation pairs them automatically
3. **Preference-Aware Algorithm:** Considers dietary restrictions, cuisine variety and the user's enabled course types when generating
4. **Cooking Mode:** Per-recipe cooking cursor (Idle → Cooking step n → Completed) persisted in the list, so progress survives leaving the kitchen
5. **Prep-Ahead Visibility:** A "Prep ahead" rail lists every uncooked recipe in the list that carries an advance-prep note, so marination and rising are never a surprise
6. **Community Integration:** Social recipe sharing across six course types (appetizer, main course, accompaniment, dessert, beverage, condiment) with a rating system; saving a community recipe adds it to the user's generation pool
7. **Shopping Intelligence:** One Groceries page with aisle grouping, an aisle route strip, per-aisle progress and persistent check-off—no separate "generate shopping list" step and no date range

### Why This Solution Will Succeed

- **Eliminates Core Friction:** Directly addresses recipe selection, meal composition realism and grocery aggregation that calendar-based solutions complicate
- **Trust Through Results:** Generation demonstrates immediate value—users see a full list of meals in seconds
- **Realistic Meal Composition:** Accompaniment pairing reflects how people actually eat (curry with rice, pasta with sauce), not isolated dishes
- **Respects User Constraints:** Algorithm honors dietary restrictions—lists users can actually execute
- **Network Effects:** Community features across six course types create sustainable engagement and content growth
- **Clear Value Proposition:** Measurable increase in recipe variety (2x more unique recipes cooked) and decreased planning stress (60% time reduction)

### High-Level Product Vision

A comprehensive cooking ecosystem that transforms deciding what to cook from a daily stressor into an automated background process, enabling users to focus on the joy of cooking while accessing their full culinary potential.

## Target Users

### Primary User Segment: Home Cooking Enthusiasts

**Demographics:**
- Age: 28-45
- Household income: $50,000+
- Family status: Couples and families with children
- Location: Suburban and urban areas

**Current Behaviors:**
- Save 50+ recipes but cook only 10-15 regularly
- Spend 15-30 minutes per shopping cycle deciding what to cook
- Shop for groceries 1-2 times per shopping cycle
- Avoid complex recipes due to timing uncertainty

**Specific Needs and Pain Points:**
- Want culinary variety without planning complexity
- Need advance preparation to be visible before it is too late
- Require shopping efficiency and ingredient optimization
- Desire family meal coordination

**Goals:**
- Cook interesting, varied meals without stress
- Reduce food waste and optimize grocery spending
- Teach children diverse culinary experiences
- Maintain healthy eating habits consistently

### Secondary User Segment: Busy Professional Families

**Demographics:**
- Age: 32-50
- Dual-income households
- 1-3 children
- Limited evening cooking time on workdays

**Current Behaviors:**
- Heavy reliance on meal kit services or takeout
- Batch preparation on days off when possible
- Simple workday recipes only
- Bulk shopping trips

**Specific Needs:**
- Minimal workday preparation time
- Family-friendly recipes with broad appeal
- Efficient shopping and preparation workflows
- Emergency backup meal options

**Goals:**
- Reduce reliance on processed foods and takeout
- Maintain family dinner traditions despite busy schedules
- Optimize grocery budget and reduce waste
- Create positive food experiences for children

## Goals & Success Metrics

### Business Objectives

- **User Acquisition:** 10,000 active users within 6 months of launch
- **Engagement:** 70% of registered users active in any 30-day period
- **Revenue:** $50,000 monthly recurring revenue by month 12
- **Growth:** 25% month-over-month user growth through first year
- **Market Position:** Recognized as the leading calendar-free cooking companion

### Monetization Model

**Freemium Strategy:**
- **Free Tier:** Full access to every feature—recipe list generation and manual editing, cooking mode, groceries, recipe management, import, rating, community sharing and saving—with ads displayed in the interface.
- **Premium Tier:** The same feature set without ads, plus priority support and early access to new features.
- **Upgrade Incentive:** Users who cook regularly see the ads often; an ad-free experience is the natural upgrade for engaged cooks. No feature is gated behind premium.
- **Upgrade Prompts:** A subscription entry point in Settings and around ad placements. Persistent but not intrusive.
- **Conversion Target:** 15% of free users upgrade to premium within 60 days

**Pricing Structure:**
- **Monthly Subscription:** $9.99/month with auto-renewal
- **Annual Subscription:** $59.94/year ($4.99/month equivalent) - **50% savings** compared to monthly
- **Payment:** Credit card auto-renewal, cancellable anytime by user through account settings
- **Subscription Management:** Users retain premium access until end of paid period, then immediately downgrade to free tier (no grace period)
- **After Expiration:** Ads return; the recipe list, groceries and all recipes remain fully accessible

### User Success Metrics

- **Recipe Variety Increase:** Users cook 3x more unique recipes (enabled by no-repeat generation across the whole list)
- **Planning Time Reduction:** 60% decrease in time spent deciding what to cook (one-tap generation vs manual selection)
- **Accompaniment Success:** 85% of main courses that accept an accompaniment are paired with one when the type is enabled
- **Preference Match Rate:** 95% of generated recipes meet user's dietary restrictions
- **Preparation Success:** 90% of users complete advance preparation surfaced in the "Prep ahead" rail
- **Stress Reduction:** 70% report decreased meal planning anxiety
- **Food Waste Reduction:** 40% decrease in unused ingredient waste (improved by one merged grocery list for the whole list)

### Key Performance Indicators (KPIs)

- **Daily Active Users (DAU):** Target 30% of monthly users
- **Generation Adoption:** 70% of users generate their list rather than only adding recipes by hand
- **Average List Size:** 7+ main courses per generation
- **Regeneration Rate:** <15% of generated lists regenerated before any recipe is cooked (indicating high initial satisfaction)
- **Recipe Completion Rate:** 85% of recipes in the list reach "Cooked" before removal or regeneration
- **Accompaniment Pairing Rate:** 60% of accepting main courses are served with an algorithm-selected accompaniment
- **Preference Configuration:** 80% of users configure dietary restrictions
- **Community Engagement:** 40% of users rate or review recipes monthly
- **Premium Conversion:** 15% of free users upgrade to premium within 60 days
- **Groceries Usage:** 80% of users check off groceries for each generated list
- **Recipe Import Adoption:** 45% of users import at least one recipe batch within first 30 days
- **Average Recipes Imported:** 15-25 recipes per importing user (bulk onboarding efficiency)
- **Import Success Rate:** >95% of valid JSON files successfully processed without errors
- **Algorithm Performance:** <5 seconds P95 for list generation at the maximum of 30 main courses

## MVP Scope

### Core Features (Must Have)

- **Home Route (/):** The Kitchen page for signed-in users. Signed-out visitors see an SEO-optimized landing page showcasing the main features (recipe list generation, accompaniments, cooking mode, groceries), a 3-step how-it-works section, screenshots, pricing preview and clear CTA buttons for registration/login.
- **Kitchen Page:** The home for signed-in users. Header "Up next" with a cooked counter (e.g. "1/11 cooked") and a Generate/Regenerate button. A hero card for the recipe to cook next—the first not-yet-cooked recipe, courses ordered starter → main → side → dessert → drink → sauce, then list order—with Start cooking, See recipe and remove actions. Below it the rest of the list as rows (tap to focus, X to remove, "Cooked"/"Cooking" badges, uncooked first), ending with an "Add recipes" card. A "Prep ahead" side rail lists uncooked recipes in the list that have an advance-prep note. **Onboarding:** no recipes → "add your first recipe" screen; recipes but empty list → "generate your list" screen.
- **Recipe List Generation:** Modal on the Kitchen page with a count of main courses (1 to 30, default 7). Picks up to N main courses at random from the candidate pool (own recipes plus saved community recipes), filtered by dietary restrictions and sized across cuisines by the cuisine variety weight; then, for each optional course type enabled in preferences, up to ceil(N/2) recipes of that type, accompaniments only when a picked main accepts one. A recipe never appears twice. **Generation replaces the whole list**, clears checked groceries and resets cooking progress. **Generation is non-deterministic:** each run produces a different list from the same pool, so users can regenerate until they like the result. **Insufficient recipes:** the list is simply shorter—no minimum recipe count, no warnings.
- **Manual List Editing:** "Add to my list" / "In my list" button on every recipe page (own or shared). Remove from the Kitchen page with the X on each recipe.
- **Cooking Mode:** Per-recipe cooking cursor Idle → Cooking(step n) → Completed, persisted per recipe in the list. Ingredients screen first, then step by step with timers. A completed recipe stays in the list marked "Cooked" until removed or the list is regenerated.
- **Groceries Page:** The merged, household-size-scaled ingredient list of every recipe in the list, grouped by aisle with an aisle "route" strip, per-aisle progress, check-off that persists, and an "Edit recipes" link back to the Kitchen. Updates automatically whenever the list changes—no separate generation step, no date range.
- **Recipe Management:** Users create and manage their own recipes with optional sharing to other users across six course types: Appetizer, Main Course, Accompaniment, Dessert, Beverage and Condiment. Recipe fields including dietary restrictions, cuisine type, advance-prep note and all other attributes must be **explicitly set by users**—no automatic detection or inference from recipe content. **Community Sharing:** all course types are sharable in both free and premium tiers.
- **Recipe Import:** Bulk import recipes from JSON files (max 10MB per file, 20 files per batch) via drag-and-drop or file picker interface. Imported recipes stored as private by default in user's recipe library. Supports all course types with validation against HTML form schema. Invalid recipes skipped with detailed summary report showing success/failure counts. **Progress Feedback:** Real-time progress display showing imported count, failed count, and remaining count (e.g., "Imported 45 recipes, 3 failed, 12 remaining..."). **Validation Timing:** All validation performed during processing; errors shown only in final summary report after all files are processed. **Duplicate Detection:** System detects duplicate recipes (matching name or similar ingredients) and blocks them from being imported. Duplicates shown in summary report with warning. **Validation Strictness:** Only 100% valid recipes are imported; any recipe with missing required OR optional fields is rejected entirely to ensure data quality. **Import History:** No persistent storage of import operations; summary results shown only during current import session. **JSON Schema:** Publicly accessible versioned JSON schema documentation (e.g., v1.0, v1.1) enabling third-party tools to generate compatible recipe exports.
- **Accompaniment System:** Main courses can optionally accept accompaniments (pasta, rice, fries, salad, bread, vegetables). Users create accompaniment recipes and generation pairs them with compatible main courses. **Default:** Main courses default to NOT accepting accompaniments (`accepts_accompaniment=false`); users must explicitly enable accompaniment pairing when creating/editing recipes.
- **User Preferences Integration:** Generation considers dietary restrictions, the cuisine variety weight and the enabled optional course types; groceries scale by household size. **Default cuisine variety weight:** 0.7 (where 0.0 = repeat cuisines frequently, 1.0 = maximum variety; configurable per user).
- **Recipe Rating System:** Community-driven quality feedback and reviews on shared recipes. **Quality Control:** Ratings and reviews act as quality filter with low-rated recipes buried in search results. Community self-moderation through feedback with no pre-approval required - trusting users to maintain recipe quality.
- **Recipe Favorites ("Save"):** Users can save community-shared recipes from other users. A saved recipe joins the user's candidate pool for list generation and can be added to the list by hand. Users can view and manage all saved recipes in their profile. If a recipe owner deletes their recipe, it is automatically removed from all users' saved recipes (no notifications sent to recipe creators when saved).
- **User Profile Management (Settings):** Household size, dietary restrictions, cuisine variety weight and enabled optional course types (appetizer, accompaniment, dessert, beverage, condiment).
- **Navigation:** Four items—Kitchen, Recipes, Groceries, Settings.
- **Contact Us Form:** Public form for users to submit questions, feedback, and support requests with fields for name, email, subject, and message. **Admin Notifications:** Email notifications sent to admin(s) when new contact form messages are submitted for real-time awareness and better response time.
- **Admin Panel:** Administrative interface for platform management with user management (view, edit, suspend/activate accounts, manage premium bypass flags) and contact form inbox (view submitted messages, mark as read/resolved, search/filter by date/status). **Admin Access:** Admins identified by dedicated `is_admin` flag in user profile; admin users have full access to admin panel and management features. **User Suspension Impact:** When admin suspends a user, the user cannot log in, their recipe list becomes inaccessible to them, and their shared recipes are hidden from community view. All data is preserved for potential reactivation but not visible during suspension.
- **Mobile-Responsive Design:** Touch-optimized Kitchen page and cooking mode for kitchen use, installable as a PWA with offline recipe access.

### Out of Scope for MVP

- **Machine Learning Features:** Predictive recipe recommendations, adaptive difficulty adjustment, cooking-speed-aware timers
- **Grocery Store API Integrations:** One-tap ordering through partner services, real-time inventory checking, automatic price comparison
- **Advanced Social Features:** Community contests, chef profiles, recipe collections, public/private recipe sharing settings (basic sharing and rating included in MVP)
- **Smart Kitchen Device Integration:** Connected appliances, automated inventory tracking, IoT temperature monitoring
- **Video Cooking Guidance:** Step-by-step video instructions, AR cooking assistance, live cooking sessions
- **Scheduling and Reminders:** Any calendar, date assignment, push notification or prep reminder; the "Prep ahead" rail is the only prep surface
- **Recipe Collections & Templates:** Curated themed collections, pre-built diet-specific list templates (Keto, Mediterranean, etc.)
- **Advanced Regeneration:** Partial regeneration (swap a single recipe for another suggestion), constraint relaxation suggestions when insufficient recipes

### MVP Success Criteria

Successfully demonstrate that a generated recipe list with accompaniment pairing can increase recipe variety while reducing planning complexity:

- **Recipe Variety:** Users cook at least 2x more unique recipes compared to pre-app usage
- **Planning Efficiency:** 80% reporting reduced meal planning stress and 60% time savings
- **Generation Adoption:** 70% of users generate their list at least once a month rather than only adding recipes by hand
- **Accompaniment Success:** 85% of accepting main courses successfully paired when the type is enabled
- **Preference Match:** 95% of generated recipes meet user's dietary restrictions
- **Technical Performance:** <5 seconds P95 for list generation (up to 30 main courses), <0.1% error rate
- **User Satisfaction:** >4.0/5.0 average rating for the recipe list and cooking mode
- **Freemium Conversion:** 15% of free users upgrade to the ad-free premium tier within 60 days

## Post-MVP Vision

### Phase 2 Features

- **Smarter Generation:** Additional selection factors such as recipe complexity spacing, seasonal ingredients and what the household cooked recently (MVP already includes dietary restrictions, cuisine variety and course-type preferences)
- **Community Recipe Discovery:** Enhanced social features with recipe collections, contests, and chef profiles (MVP includes basic sharing and rating)
- **Grocery Store Integration:** One-tap ordering through partner services with real-time inventory and pricing
- **Optional Reminders:** Opt-in push reminders for recipes with advance-prep notes, driven by the list rather than a calendar
- **Recipe Collections:** Curated themed collections (e.g., "Summer BBQ Favorites", "Quick Evening Dinners")
- **List Templates:** Pre-built list templates for specific diets (Keto, Mediterranean, etc.)

### Long-term Vision

Transform imkitchen into a comprehensive cooking ecosystem that connects recipe selection, grocery shopping, preparation guidance, and community learning. Users experience cooking as a creative, stress-free activity supported by intelligent automation and community inspiration.

### Expansion Opportunities

- **Global Recipe Exchange:** Cultural recipe sharing with automatic measurement and ingredient conversions
- **Smart Kitchen Integration:** Connected appliances and automated inventory tracking
- **Corporate Partnerships:** Meal kit services, grocery chains, cooking equipment manufacturers
- **Educational Content:** Cooking classes, technique videos, and skill progression tracking

## Technical Considerations

### Platform Requirements

- **Target Platforms:** Progressive Web App (PWA) for cross-platform compatibility
- **Browser/OS Support:** Modern mobile browsers (iOS Safari, Android Chrome), installable app experience
- **Performance Requirements:** <3 second load times, offline recipe access, real-time sync

### Technology Preferences

- **CLI:** Clap 4.5+ for command-line interface
- **Configuration:** config 0.15+ for application configuration management
- **Observability:** OpenTelemetry 0.31+ for metrics, logs, and distributed tracing
- **E2E Testing:** Playwright 1.56+ (TypeScript) for end-to-end testing

### Architecture Considerations

- **Vendor Lock-in:** Avoid proprietary dependencies and cloud-specific services; prioritize open standards and portable solutions
- **Design System:** Comprehensive design system with consistent components, spacing, typography, and color palette across entire application
- **User Experience:** Cohesive navigation and interaction patterns ensuring users feel immersed in a unified application experience
- **SEO Optimization:** Landing page must be SEO-optimized with proper meta tags, structured data (Schema.org), semantic HTML, fast load times, and mobile-responsive design for search engine visibility and organic user acquisition
- **Integration Requirements:** Future grocery store API compatibility
- **Security/Compliance:** OWASP security standards for all security-related implementations, JWT for cookie-based authentication tokens, user data encryption, GDPR compliance, secure payment processing ready
- **Analytics & Privacy:** Minimal anonymous analytics only - aggregate, anonymized metrics for basic usage statistics (generation counts, feature adoption rates). No individual user tracking. Privacy-first approach with less optimization data but stronger user privacy protection.
- **Recipe Import:** JSON schema validation matching HTML form structure, 10MB file size limit enforcement, batch processing for up to 20 concurrent files, malicious file detection (script injection, oversized payloads), streaming parser for large files, detailed validation error reporting per recipe, rollback capability for failed batches
- **Premium Bypass Configuration:** System configuration option to bypass premium restrictions for development, testing, demo accounts, and internal team access. Supports two implementation approaches: (1) Global bypass via application config file for entire environment (e.g., development/staging), (2) Per-user bypass flag in user profile for selective access (e.g., demo accounts, internal team members).
- **Recipe List as an Aggregate:** The recipe list is one event-sourced aggregate per user holding the ordered recipes, each recipe's cooking cursor and the checked groceries. Generation, manual add/remove, cooking progress and grocery check-off are all events on that aggregate, so the Kitchen and Groceries pages are projections of the same state.

## Constraints & Assumptions

### Constraints

- **Technical:** Must work reliably on mobile devices, offline recipe access required
- **Performance:** List generation must complete in <5 seconds (P95) for the maximum of 30 main courses
- **Data Model:** Event sourcing requires careful migration strategy for schema changes
- **Algorithm:** Must generate lists from the user's existing recipe pool, gracefully handling insufficient recipes by producing a shorter list (no minimum recipe count enforced)

### Key Assumptions

- Users will trust automated list generation if it demonstrably saves time and respects their constraints
- Accompaniment pairing will be perceived as added value, not unnecessary complexity
- Users will configure preferences (dietary restrictions) during onboarding
- Community features will drive organic growth and engagement across six course types
- An ad-supported free tier with no feature gating provides enough value to demonstrate benefit while an ad-free premium remains an attractive upgrade for engaged cooks
- Allowing unlimited regenerations removes frustration; regeneration replacing the whole list (including cooking progress) will be understood as "start fresh"
- A single list with no dates simplifies the user experience and reduces decision fatigue compared with a calendar
- Grocery partnerships will provide sustainable revenue streams
- Mobile-first approach with a Kitchen page and cooking mode is sufficient for initial market penetration
- Users will accept that a cooked recipe stays in the list as "Cooked" until they remove it or regenerate

## Risks & Open Questions

### Key Risks

- **User Adoption:** Users may resist changing established meal planning habits, especially a dated calendar vs an undated list
- **Algorithm Edge Cases:** Generation with preference-aware filtering may produce very short lists when constraints exclude most of the pool
- **Performance at Scale:** Generation must remain fast even with large recipe pools (100+ recipes)
- **Accompaniment Confusion:** Users may not understand optional accompaniment system or when/how to use it
- **Preference Configuration Friction:** Users may skip dietary restriction setup, reducing algorithm effectiveness
- **Content Quality:** Community recipe sharing across six course types may require significant moderation
- **Competition:** Existing meal planning apps may quickly copy list-based generation and accompaniment features
- **Monetization:** Ad-free premium uptake may be lower than projected
- **Regeneration Surprise:** Regeneration resets cooking progress and checked groceries; users mid-way through a list may lose work unexpectedly
- **Cooked Recipes Lingering:** Lists may fill with "Cooked" rows if users never remove or regenerate, hiding the next recipe
- **Recipe Import Security:** Malicious JSON files could contain script injections, oversized payloads, or trigger denial-of-service attacks
- **Import Schema Compatibility:** Users may have recipes in incompatible formats requiring manual conversion or extensive documentation
- **Bulk Import Performance:** Processing 20 files × 10MB could cause UI blocking or server timeouts without proper async handling
- **Saved Recipe Deletion Impact:** When recipe owner deletes a shared recipe, it's removed from all users' saved recipes. Users with small pools will get shorter generated lists, potentially reducing usefulness of the feature.

### Areas Needing Further Research

- **Competitive Analysis:** Do any existing apps offer calendar-free list generation or accompaniment pairing?
- **User Onboarding:** How to educate users on the list model, accompaniment system, and preference configuration?
- **Algorithm Edge Cases:** How to handle impossible constraint combinations (e.g., vegan + nut-free + gluten-free with limited recipes)?
- **International Expansion:** How do accompaniment categories (pasta, rice, fries) translate to other cuisines?
- **Accessibility:** How to make the Kitchen page, cooking mode timers and aisle route accessible to screen readers and keyboard navigation?
- **Content Moderation:** Policies and tools for moderating community-shared recipes across all course types
- **Performance Optimization:** Query optimization strategies for dietary tag filtering and recipe selection at scale
- **Recipe Import Standards:** Research existing recipe interchange formats (Schema.org Recipe, RecipeML, JSON-LD) to determine interoperability requirements and migration paths

## Appendices

### A. Research Summary

**Brainstorming Session (September 2025):** Comprehensive 65-minute facilitated session generated 47 distinct features and concepts. Key insights include the identification of self-limitation patterns in recipe selection and the critical importance of trust-building through demonstrated time savings.

**Architecture Design (October 2025):** Detailed technical design for the recipe list system covering generation (up to 30 main courses plus optional course types), accompaniment recipe type system, cooking mode, groceries aggregation and user preferences integration (dietary restrictions, cuisine variety, enabled course types). Includes complete domain model, algorithm design and implementation roadmap.

**Key Findings:**
- 80% of cooking planning happens on mobile devices
- Users avoid complex recipes primarily due to timing uncertainty, not skill limitations
- **Meal composition realism matters:** Users want accompaniments (rice, pasta, sides) automatically paired with main courses
- **Dates are the friction:** A calendar goes stale the first time a planned day slips; an undated list does not
- **Constraint respect is critical:** Algorithm must honor dietary restrictions for user trust
- Community features are essential for sustainable engagement and organic growth (extended to six course types)
- An ad-supported free tier with no feature gating balances trial value with a clear ad-free upgrade

### B. Stakeholder Input

Initial concept development involved extensive user journey mapping and pain point analysis. Primary stakeholder (product owner) emphasized the importance of mobile-first design and offline functionality for kitchen environments.

**Architecture Design Decisions (October 2025):**
- **One list per user, no dates:** The calendar meal plan was replaced by a single recipe list; the Kitchen page shows the next uncooked recipe instead of "today"
- **30 main courses maximum, 7 default:** Balances a useful cooking queue with grocery list size and generation time
- **Optional courses at half the main count:** Each enabled optional course type contributes up to ceil(N/2) recipes
- **Accompaniments always optional:** Respects recipe creator's intent; main courses control `accepts_accompaniment` boolean (defaults to `false`, must be explicitly enabled)
- **Custom cuisines allowed:** `Cuisine::Custom(String)` variant enables user-defined cuisine types
- **Generation replaces everything:** Regeneration goes through the generate modal and replaces the whole list, clearing checked groceries and cooking progress
- **Cuisine variety as a weight, not a taxonomy:** No cuisine tags are stored; the "cuisine variety" preference (0.1–1.0) only sizes the random pool the generator draws from, so a lower value repeats the same recipes more often
- **Advance prep recipe-defined:** Prep note stored in recipe's `advance_prep_text`, surfaced in the Kitchen "Prep ahead" rail, not a user preference
- **Cuisine variety weight default:** 0.7 (0.0=repeat frequently, 1.0=maximum variety)

### C. Design References

Design inspiration and UI/UX reference screenshots are stored in `./docs/design/`

Place screenshots of designs you like for the application look and feel in this folder.

### D. References

- [Brainstorming Session Results](./docs/brainstorming-session-results.md)
- Home Cooking Trends Report 2024
- Mobile App Usage in Kitchen Environments Study


## Next Steps

### Immediate Actions

1. **Validate Product-Market Fit**
   - Conduct user interviews with target segments (home cooking enthusiasts, busy families)
   - Test the ad-supported freemium model (ad-free conversion rate)
   - Validate accompaniment pairing and the undated list value proposition

2. **Define Technical Architecture**
   - Create detailed technical specification for the list generation algorithm
   - Design the recipe list aggregate, projections and persistence strategy
   - Plan the subscription and ad placement flow

3. **Design User Experience**
   - Wireframe the Kitchen page (hero card, list rows, "Prep ahead" rail) for desktop and mobile
   - Design cooking mode (ingredients screen, steps, timers) and the Groceries aisle route
   - Create accompaniment pairing UI patterns
   - Design preference configuration onboarding flow

4. **Build MVP**
   - Develop core list features (generation, manual add/remove, accompaniment system, user preferences)
   - Implement cooking mode with persisted per-recipe progress
   - Create recipe management across six course types
   - Build the always-current Groceries page

5. **Launch & Iterate**
   - Beta launch with early adopters
   - Monitor conversion metrics (free to premium upgrade rate)
   - Gather user feedback on the list model, cooking mode and accompaniment pairing
   - Iterate based on user behavior and feedback

### PM Handoff

This Project Brief provides the full context for imkitchen, including the recipe list model (generation, manual editing, cooking mode, groceries), the accompaniment system and user preferences integration. The detailed technical architecture is documented separately. Please start in 'PRD Generation Mode', review the brief thoroughly to work with the user to create the PRD section by section as the template indicates, asking for any necessary clarification or suggesting improvements.

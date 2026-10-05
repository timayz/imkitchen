# imkitchen Product Requirements Document (PRD)

**Author:** Jonathan
**Date:** 2025-10-31
**Project Level:** 3
**Target Scale:** Comprehensive product with freemium model, community features, and one-tap recipe list generation

---

## Goals and Background Context

### Goals

- Increase recipe variety by enabling users to cook 3x more unique recipes per month through random, preference-aware list generation
- Reduce meal planning time by 60% through one-tap generation of a complete recipe list
- Eliminate advance preparation stress by surfacing prep-ahead notes for every recipe still to cook
- Enable realistic meal composition through intelligent accompaniment pairing (85% success rate)
- Replace per-week shopping with a single, always-current groceries list derived from the recipe list
- Build sustainable community engagement through recipe sharing, rating, and discovery across six recipe types
- Achieve 15% freemium-to-premium conversion within 60 days by demonstrating value on the free tier before offering an ad-free subscription

### Background Context

Home cooks face a fundamental choice between culinary variety and planning simplicity. The complexity of coordinating recipe selection, advance preparation, and meal composition creates a self-limitation pattern where users avoid 70-80% of their saved recipes. Current meal planning apps focus on recipe storage without addressing the core problems: realistic meal composition (mains paired with appropriate sides), knowing what to cook next, and keeping groceries in sync with what was actually planned.

imkitchen solves this with a single per-user **recipe list**. Users either generate it in one tap (a random, dietary-filtered selection of main courses plus the optional courses they have enabled, with accompaniments paired to compatible mains) or fill it by hand from their own and the community's recipes. The Kitchen page always shows the recipe to cook next, guided cooking mode walks through each recipe step by step, and the Groceries page merges every ingredient in the list into one aisle-grouped checklist that updates automatically. The freemium model (full functionality with ads on the free tier, ad-free on premium) provides immediate value while creating a natural upgrade incentive.

---

## Requirements

### Functional Requirements

**Authentication & User Management**

- FR001: System shall support user registration, login, and profile management with JWT cookie-based authentication
- FR002: System shall store user preferences including dietary restrictions, cuisine variety weight (default 0.7), household size, and the set of enabled optional course types (appetizer, accompaniment, dessert, beverage, condiment)
- FR003: System shall support admin users identified by `is_admin` flag with access to admin panel

**Recipe Management**

- FR004: System shall allow users to create, edit, and delete recipes with six types: Appetizer, Main Course, Accompaniment, Dessert, Beverage, Condiment
- FR005: Recipe fields (dietary restrictions, cuisine type, advance-prep note, etc.) shall be explicitly set by users with no automatic inference
- FR006: Main courses shall default to `accepts_accompaniment=false` requiring explicit user enablement
- FR007: System shall support bulk recipe import from JSON files (max 10MB per file, 20 files per batch) via drag-and-drop
- FR008: Recipe import shall validate against HTML form schema, skip invalid recipes, and provide detailed success/failure summary
- FR009: Recipe import shall detect and block duplicate recipes (matching name or similar ingredients)
- FR010: Recipe import shall display real-time progress (imported count, failed count, remaining count)
- FR011: System shall maintain publicly accessible versioned JSON schema documentation for recipe import

**Recipe Favorites & Sharing**

- FR012: Users shall save ("favorite") community-shared recipes from other users; a saved recipe joins the user's candidate pool alongside their own recipes
- FR013: Saving and unsaving shall be available on both free and premium tiers with no count limit
- FR014: Users shall share recipes publicly with community (all six recipe types) in both free and premium tiers
- FR015: When recipe owner deletes their recipe, it shall be automatically removed from all users' saved recipes and recipe lists without notifications
- FR016: Users shall rate and review community-shared recipes with ratings acting as quality filter in search results

**Recipe List**

- FR017: Each user shall have exactly one recipe list, which may be generated, filled by hand, or both
- FR018: Generation shall be triggered from the Kitchen page through a modal asking for a meal count between 1 and 30 (default 7)
- FR019: Generation shall pick up to N main courses at random from the user's candidate pool (own recipes plus saved community recipes), filtered by dietary restrictions and weighted by the cuisine variety preference
- FR020: Each picked main course shall form a meal paired with one recipe of every optional course type enabled in preferences, drawn without reuse; an accompaniment shall only be paired with a main that accepts one; the list shall be stored in meal order (starter, main, side, dessert, drink, sauce)
- FR021: A recipe shall never appear twice in the list
- FR022: Generation shall be non-deterministic, producing a different selection each time to enable preference-based regeneration
- FR023: Generation shall replace the entire existing list, clear checked groceries, and reset cooking progress; regeneration shall require confirmation
- FR024: Algorithm shall handle insufficient recipes gracefully by producing a shorter list (no minimum count enforced)
- FR025: Users shall add any recipe (own or shared) to their list from its recipe page via an "Add to my list" button, which reads "In my list" once added
- FR026: Users shall remove any recipe from their list from the Kitchen page
- FR027: System shall complete generation of a 30-recipe list in <5 seconds (P95 performance)

**Kitchen Page**

- FR028: Home route (/) shall dynamically route authenticated users to the Kitchen page, unauthenticated users to SEO-optimized landing page
- FR029: Landing page shall include hero section, key features showcase, how-it-works (3-step), example screenshots, pricing preview, testimonials, and CTAs
- FR030: Kitchen page shall display an "Up next" header with a cooked counter (e.g. 1/11 cooked) and a Generate/Regenerate button
- FR031: Kitchen page shall display a hero card for the recipe to cook next: the first not-yet-cooked recipe, with courses ordered starter → main → side → dessert → drink → sauce, then list order; the card shall offer Start cooking, See recipe, and remove actions
- FR032: Below the hero card, Kitchen page shall list the remaining recipes as rows (uncooked first) that can be tapped to focus, removed, and that carry Cooked / Cooking badges, ending with an "Add recipes" card
- FR033: Kitchen page shall display a "Prep ahead" side rail listing uncooked recipes in the list that have an advance-prep note
- FR034: Users with no recipes shall see an "add your first recipe" onboarding screen; users with recipes but an empty list shall see a "generate your list" onboarding screen

**Cooking Mode**

- FR035: Each recipe in the list shall carry a cooking cursor (Idle → Cooking at step n → Completed) persisted per recipe
- FR036: Cooking mode (/kitchen/{recipe_id}/cook) shall open on an ingredients screen, then walk through instructions step by step with timers
- FR037: A completed recipe shall remain in the list marked "Cooked" until removed or the list is regenerated

**Groceries**

- FR038: Groceries page (/groceries) shall show the merged ingredient list of every recipe in the recipe list, scaled by household size
- FR039: Ingredients shall be grouped by aisle with an aisle "route" strip and per-aisle progress
- FR040: Ingredient check-off shall persist across visits
- FR041: Groceries shall update automatically whenever the recipe list changes, with no separate generation step; an "Edit recipes" link shall return to the Kitchen page

**Admin Panel**

- FR042: Admin panel shall provide user management (view, edit, suspend/activate accounts, manage premium bypass flags)
- FR043: Admin panel shall provide contact form inbox (view messages, mark read/resolved, search/filter)
- FR044: Suspended users shall not be able to log in; their recipe lists become inaccessible and shared recipes hidden
- FR045: All suspended user data shall be preserved for potential reactivation

**Contact & Support**

- FR046: System shall provide public contact form with fields for name, email, subject, and message
- FR047: System shall send email notifications to admin(s) when new contact form messages are submitted

**Freemium Access Control**

- FR048: System shall support premium bypass configuration via global config file (entire environment) or per-user flag (selective access)
- FR049: Free tier shall provide the full feature set (unlimited generations, saves, and list edits) with advertising displayed
- FR050: Premium tier shall remove advertising through a paid subscription

### Non-Functional Requirements

- NFR001: System shall achieve <3 second page load times on mobile devices
- NFR002: System shall complete recipe list generation in <5 seconds (P95 latency)
- NFR003: System shall maintain <0.1% error rate for recipe list generation operations
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

### Journey 1: New User Onboarding & First Recipe List

**Persona:** Sarah, busy professional with family, wants meal variety without planning stress

**Starting Point:** Discovers imkitchen through search, visits landing page

**Journey Steps:**

1. **Discovery** - Sarah lands on SEO-optimized landing page, reads value proposition about one-tap recipe lists with accompaniments
2. **Registration** - Creates account with email/password authentication
3. **Profile Setup** - Configures dietary restrictions (gluten-free), cuisine variety preference (0.7 default), household size (4), and enables appetizers and desserts as optional courses
4. **Recipe Import** - Bulk imports 25 recipes from JSON file exported from previous app (drag-and-drop interface shows real-time progress)
5. **Recipe Review** - Reviews import summary: 23 successful, 2 failed (missing required fields), duplicates blocked
6. **Community Discovery** - Browses community recipes, saves 8 additional recipes (candidate pool: 31 recipes)
7. **First Generation** - Opens the Kitchen page, which shows the "generate your list" screen; taps Generate, keeps the default of 7, and gets a list of 7 mains plus up to 4 appetizers and 4 desserts in under a second
8. **Kitchen Review** - Sees the first main course as the hero card, the rest of the list below, and notices mains paired with appropriate accompaniments (curry with rice)
9. **Groceries** - Opens the Groceries page, finds every ingredient already merged and grouped by aisle, scaled for 4 people
10. **Manual Tweak** - Removes one dessert she does not like from the Kitchen page and adds a saved community recipe from its page instead; groceries update automatically
11. **Decision Point** - Satisfied, wants an ad-free experience → upgrades to premium
12. **Premium Experience** - Same list, same features, no advertising

**Success Outcome:** Sarah has a complete recipe list with groceries ready in <10 minutes, upgraded to premium for an ad-free experience

**Pain Points Addressed:**
- Recipe import eliminated manual entry of 25 recipes
- One-tap generation produced a full list immediately
- Accompaniment pairing removed meal composition complexity
- Full free-tier functionality demonstrated value before payment

---

### Journey 2: Cooking Through the List & Regeneration

**Persona:** James, home cooking enthusiast, using imkitchen for ongoing cooking

**Starting Point:** Has a generated list of 11 recipes, 1 already cooked

**Journey Steps:**

1. **Kitchen Check** - Opens the app, header reads "Up next · 1/11 cooked", hero card shows the next uncooked recipe
2. **Prep Ahead** - Glances at the "Prep ahead" rail and sees "Marinate chicken overnight" on a recipe further down the list
3. **Start Cooking** - Taps Start cooking on the hero card, reviews the ingredients screen, then follows the steps with timers; the recipe is badged "Cooking"
4. **Completion** - Finishes the last step; the recipe is badged "Cooked", the counter reads 2/11, and the next uncooked recipe becomes the hero card
5. **Plans Change** - A family event means one planned recipe will not be cooked; James removes it from the list with the X on its row
6. **Groceries** - Before shopping, opens Groceries; the removed recipe's ingredients are gone and his earlier check-offs are preserved
7. **Grocery Shopping** - Follows the aisle route strip in store, checking off items with per-aisle progress
8. **Reordering** - Wants to cook a dessert tonight; taps its row to focus it as the hero card
9. **List Exhausted** - After cooking most of the list, taps Regenerate; the confirmation modal warns that the list, cooking progress, and checked groceries will be replaced
10. **New Rotation** - Chooses 10 recipes; sees a fresh, non-deterministic selection with different cuisine distribution and no repeated recipes

**Success Outcome:** James cooks through his list at his own pace, with groceries always matching what is left to cook

**Pain Points Addressed:**
- Always-visible "next up" recipe removed daily decision fatigue
- Prep-ahead rail surfaced advance preparation without scheduled reminders
- Single groceries list stayed in sync with list edits automatically
- Non-deterministic regeneration allowed preference-based re-planning

---

### Journey 3: Community Engagement & Recipe Contribution

**Persona:** Maria, passionate home cook, wants to share recipes and discover community favorites

**Starting Point:** Existing premium user with 45 saved community recipes

**Journey Steps:**

1. **Recipe Creation** - Creates new Thai curry recipe (Main Course type), explicitly sets dietary restrictions, cuisine type, and an advance-prep note
2. **Accompaniment Configuration** - Enables `accepts_accompaniment=true` so generation can pair it with a side
3. **Community Sharing** - Publishes recipe publicly to community, available to all users (free and premium)
4. **Recipe Discovery** - Browses community recipes filtered by "Beverage" type to round out her list
5. **Rating Engagement** - Rates recently tried community recipe 5 stars, writes review: "Perfect weeknight meal!"
6. **Save & Add** - Saves a new community recipe (joining her candidate pool) and taps "Add to my list" on its page to cook it soon
7. **Generation Impact** - Next generation draws from her saved community recipes as well as her own
8. **Recipe Popularity** - Maria's shared Thai curry gains ratings from other users, rises in community search results
9. **Recipe Deletion Scenario** - Original creator of a saved recipe deletes it → automatically removed from Maria's saved recipes and list
10. **Adaptation** - Maria notices the gap on her Kitchen page, browses community for a replacement
11. **Quality Filter** - Low-rated recipes (< 3 stars) buried in search results due to community ratings
12. **Accompaniment Sharing** - Creates and shares specialized rice pilaf recipe (Accompaniment type) for community use

**Success Outcome:** Maria contributes to community ecosystem while discovering high-quality recipes through social rating system

**Pain Points Addressed:**
- Six recipe types enabled complete meal composition sharing
- Community ratings provided quality filter without pre-approval moderation
- Automatic deletion handling prevented broken saves and lists
- "Add to my list" bridged discovery and cooking in one tap

---

## UX Design Principles

1. **Mobile-First Kitchen Optimization** - Touch-optimized interface designed for use in kitchen environments with larger tap targets, clear typography, offline recipe access, and a step-by-step cooking mode
2. **Instant Value Demonstration** - One-tap generation shows a complete list in <5 seconds to build trust through demonstrated time savings
3. **Always Know What's Next** - The Kitchen page leads with a single hero recipe so the user never has to choose from a grid
4. **Friction Reduction** - One-tap generation eliminates complex configuration; system handles accompaniments and preferences automatically, and groceries follow the list without a separate step
5. **Trust Through Transparency** - Explicit confirmation dialogs for destructive actions (regeneration), visible Cooked / Cooking state, and visible preference impacts
6. **Realistic Meal Composition** - Accompaniment pairing reflects how people actually eat (curry with rice, pasta with sauce), not isolated dishes
7. **Graceful Degradation** - Too few recipes simply yields a shorter list; empty states guide the user to add or generate rather than block
8. **Accessibility Priority** - Screen reader support, keyboard navigation, and semantic HTML for inclusive experience

---

## User Interface Design Goals

**Platform & Screens:**
- Progressive Web App (PWA) with installable experience for iOS Safari and Android Chrome
- Core screens: Landing Page, Kitchen, Cooking Mode, Recipe Management, Community Browse, Groceries, Settings, Admin Panel
- Primary navigation: Kitchen, Recipes, Groceries, Settings

**Design System:**
- Comprehensive design system with consistent components, spacing (8px grid), typography scale, and cohesive color palette
- Unified navigation patterns ensuring immersive application experience across all screens
- Reusable component library for forms, cards, modals, buttons, and data tables

**Key Interaction Patterns:**
- Generate modal with recipe count (1-30, default 7) and regeneration confirmation
- Hero card plus tappable rows on the Kitchen page; tap to focus, X to remove
- Step-by-step cooking mode with timers, entered from the hero card
- Aisle route strip and check-off on the Groceries page
- Drag-and-drop recipe import with real-time progress feedback
- Modal confirmations for destructive actions (regeneration, deletion)
- Responsive card layouts for recipe browsing with quick-action buttons ("Save", "Add to my list")

**Visual Feedback:**
- Loading states for generation operations (<5 second completion)
- Real-time progress indicators for bulk operations (import, generation)
- Cooked / Cooking badges and the cooked counter on the Kitchen page
- Per-aisle progress on the Groceries page
- Empty state illustrations with clear CTAs ("Add your first recipe", "Generate your list")
- Success/error toast notifications for user actions

**SEO & Performance:**
- Landing page optimized with meta tags, Schema.org structured data, semantic HTML
- <3 second page load times on mobile devices
- Lazy loading for recipe images
- Offline-first architecture with service worker caching

---

## Visual Design References

Each core screen maps to the requirements it demonstrates. Screens are identified by their routes in the application.

### Screen-to-Requirement Mapping

| Screen | Route | Mapped Requirements | Description |
|--------|-------|---------------------|-------------|
| **Public Pages** |
| Landing page | `/` (unauthenticated) | FR028, FR029 | SEO-optimized landing page with hero, features showcase, how-it-works, pricing preview, testimonials |
| Login | `/login` | FR001 | User authentication with JWT cookie-based login |
| Register | `/register` | FR001, FR002 | User registration with dietary restrictions, household size, cuisine variety preferences |
| Contact | `/contact` | FR046, FR047 | Public contact form with subject categories and FAQ section |
| **Kitchen & Cooking** |
| Kitchen | `/` (authenticated) | FR017, FR018, FR023, FR026, FR030-FR034 | "Up next" header with cooked counter, Generate/Regenerate, hero card, recipe rows, "Prep ahead" rail, onboarding empty states |
| Cooking mode | `/kitchen/{recipe_id}/cook` | FR035-FR037 | Ingredients screen then step-by-step instructions with timers; cursor persisted per recipe |
| **Recipe Management** |
| Recipe create / edit | `/recipes/create`, `/recipes/{id}` | FR004, FR005, FR006 | Recipe form with six types, explicit field configuration, main course accepts_accompaniment toggle, advance-prep note |
| Recipe library | `/recipes` | FR004, FR012 | Own and saved recipes with filters, all six recipe types color-coded |
| Recipe detail | `/recipes/{id}`, `/r/{slug}` | FR012, FR016, FR025 | Full recipe view with ingredients, instructions, rating summary, "Save" and "Add to my list" / "In my list" actions |
| Import | `/recipes/import` | FR007-FR011 | Bulk JSON import with drag-drop, schema documentation, real-time progress, duplicate detection |
| **Community & Groceries** |
| Community | `/cooks/{username}`, `/r/{slug}` | FR014, FR016 | Community recipe browse with filters (type/cuisine/dietary) and rating system |
| Groceries | `/groceries` | FR038-FR041 | Merged, household-scaled ingredient list grouped by aisle with route strip, per-aisle progress, persistent check-off, "Edit recipes" link |
| **Settings & Support** |
| Settings | `/settings/general`, `/settings/billing` | FR002, FR003, FR048-FR050 | Dietary restrictions, cuisine variety slider (0.7 default), household size, optional course toggles, subscription management |
| **Admin Panel** |
| Admin users | `/admin/users` | FR042, FR044, FR045 | User management with stats cards and user actions (edit, suspend, reactivate) |
| Admin contact | `/admin/contact` | FR043, FR047 | Contact inbox with message stats and quick actions (mark read, resolve) |

### Freemium Model Demonstrations

The freemium model is demonstrated at two touchpoints:

- **Advertising**: Free tier screens display ads; premium removes them (`/upgrade/modal`, `/settings/billing`)
- **Feature parity**: Generation, saves, list editing, cooking mode, and groceries are identical on both tiers

### Recipe Type Demonstrations

All six recipe types are color-coded consistently across screens:

- **Appetizer** - optional course, one per meal when enabled
- **Main Course** - the backbone of every generated list, with accompaniment pairing when `accepts_accompaniment=true`
- **Accompaniment** - added only when a picked main course accepts one
- **Dessert**, **Beverage**, **Condiment** - optional courses, one per meal when enabled

### User Flow Demonstrations

**New User Onboarding Flow:**
Landing → Register → Kitchen ("add your first recipe") → Recipes / Import → Kitchen ("generate your list") → Kitchen (list)

**Recipe Management Flow:**
Recipes → Recipe create (create) → Recipe detail (view) → Import (bulk import)

**Cooking Flow:**
Kitchen (generate or add) → Groceries (shop) → Kitchen (hero card) → Cooking mode (cook) → Kitchen (Cooked badge, next hero)

**Community Engagement Flow:**
Community (discover) → Recipe detail (rate / save / add to my list) → Kitchen (cook)

**Admin Management Flow:**
Admin users (user management) → Admin contact (support inbox)

---

## Epic List

**Epic 1: Foundation & User Management**
- Establish project infrastructure, user authentication, and profile management with dietary restrictions, household size, cuisine variety, and optional course toggles
- **Estimated Stories:** 6-8 stories
- **Key Deliverables:** Project setup, JWT authentication, user registration/login, profile CRUD, admin panel foundation

**Epic 2: Recipe Management & Import System**
- Enable users to create, manage, and bulk import recipes with six types (Appetizer, Main Course, Accompaniment, Dessert, Beverage, Condiment)
- **Estimated Stories:** 7-9 stories
- **Key Deliverables:** Recipe CRUD operations, JSON bulk import with validation, duplicate detection, real-time progress feedback, saved recipes

**Epic 3: Recipe List Engine**
- Implement one-tap recipe list generation with random selection, dietary filtering, cuisine variety weighting, optional course quotas, and accompaniment pairing, plus manual add/remove
- **Estimated Stories:** 6-8 stories
- **Key Deliverables:** Generation algorithm, candidate pool (own + saved), generate modal with count, replace-on-generate semantics, "Add to my list" / remove

**Epic 4: Kitchen, Cooking Mode & Groceries**
- Build the Kitchen page with hero card and list rows, step-by-step cooking mode with persisted progress, and the auto-updating aisle-grouped groceries list
- **Estimated Stories:** 7-9 stories
- **Key Deliverables:** Kitchen page with "Up next", cooked counter, Prep ahead rail and onboarding states; cooking cursor and cook screens; groceries merge, household scaling, aisle route, persistent check-off

**Epic 5: Community Features & Freemium Access**
- Enable recipe sharing, rating system, ad-supported free tier, premium subscription, and admin user management
- **Estimated Stories:** 6-8 stories
- **Key Deliverables:** Recipe sharing/privacy, rating and review system, advertising on free tier, subscription and upgrade flows, premium bypass configuration, admin user management

**Epic 6: Landing Page & Support**
- Implement SEO-optimized landing page and contact form with admin notifications
- **Estimated Stories:** 3-5 stories
- **Key Deliverables:** Landing page with hero/features/pricing, contact form, email notifications to admins

**Total Estimated Stories:** 35-47 stories

> **Note:** Detailed epic breakdown with full story specifications is available in [epics.md](./epics.md)

---

## Out of Scope

**Deferred to Post-MVP (Phase 2):**

- **Machine Learning Features** - Predictive recipe recommendations, adaptive difficulty adjustment, learning from cooking history
- **Grocery Store Integrations** - One-tap ordering through partner services, real-time inventory checking, automatic price comparison
- **Advanced Social Features** - Community contests, chef profiles, recipe collections, public/private sharing settings beyond basic sharing/rating
- **Smart Kitchen Devices** - Connected appliance integration, automated inventory tracking, IoT temperature monitoring
- **Video Guidance** - Step-by-step video instructions, AR cooking assistance, live cooking sessions
- **Extended Generation Factors** - Weather-based suggestions, energy level tracking, seasonal ingredient preferences
- **Recipe Collections & Templates** - Curated themed collections, pre-built diet-specific list templates (Keto, Mediterranean, etc.)
- **Advanced Regeneration** - Regenerating only part of the list, constraint relaxation suggestions when insufficient recipes
- **Payment Processing** - Credit card payment gateway integration, subscription billing, auto-renewal (premium tier access control included, payment flow deferred)

**Explicitly Out of Scope:**

- **Scheduling** - Calendar views, day slots, meal plans by week or month, scheduled reminders
- **Multi-household accounts** - Shared family accounts with multiple user access
- **Meal kit delivery service** - First-party meal kit fulfillment
- **Nutritional analysis** - Detailed macro/micronutrient tracking and goals
- **Custom course types** - User-defined course types beyond the six built-in recipe types
- **International localization** - Multi-language support, regional ingredient variations
- **Social messaging** - Direct messaging between users, community forums
- **Recipe versioning** - Historical tracking of recipe edits and changes

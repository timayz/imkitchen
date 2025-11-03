# Project Brief: imkitchen

## Executive Summary

imkitchen is an intelligent meal planning and cooking optimization platform that eliminates the mental overhead and timing complexity that prevents home cooks from exploring their full recipe repertoire. The platform automatically generates meal plans for all remaining weeks of the current month (free tier can view first week only; premium views all weeks) with intelligent accompaniment pairing, preference-aware scheduling, and comprehensive shopping lists—transforming cooking from a stressful daily decision into an effortless, enjoyable experience.

**Primary Problem:** Home cooks artificially limit their recipe choices to avoid advance preparation timing complexity and lack visibility into long-term meal planning, resulting in culinary monotony and underutilized recipe collections.

**Target Market:** Home cooking enthusiasts and busy families who want variety in their meals but struggle with planning, preparation timing, and realistic meal composition.

**Key Value Proposition:** Multi-week intelligent automation that unlocks access to complex recipes by managing timing, preparation, accompaniment pairing, and scheduling complexity automatically while respecting dietary restrictions.

## Problem Statement

### Current State and Pain Points

Home cooks face a fundamental choice between culinary variety and planning simplicity. Current meal planning approaches force users to manually coordinate:

- Recipe selection based on available time and energy across multiple weeks
- Advance preparation requirements (marination, rising, chilling)
- Ingredient freshness and shopping timing across extended planning horizons
- Realistic meal composition (main dishes paired with appropriate accompaniments)
- Equipment conflicts and kitchen workflow
- Family schedule coordination
- Dietary restrictions matching

This complexity creates a **self-limitation pattern** where users:
- Avoid recipes requiring advance preparation
- Skip recipes that need accompaniments (rice, pasta, sides) due to coordination overhead
- Plan only one week ahead, missing bulk shopping opportunities
- Ignore their actual dietary needs when selecting recipes
- Result in 70-80% of saved recipes never being cooked

Users maintain browser favorites with hundreds of recipes but repeatedly cook only 10-15 simple ones that require no accompaniments and minimal planning.

### Impact of the Problem

- **Culinary Stagnation:** Users cook 70-80% fewer recipes than they save
- **Decision Fatigue:** Daily "what's for dinner" stress compounds meal preparation burden
- **Ingredient Waste:** Poor planning leads to unused ingredients and rushed shopping
- **Family Conflict:** Last-minute meal decisions create household tension
- **Lost Opportunities:** Complex but rewarding recipes remain untried

### Why Existing Solutions Fall Short

Current meal planning apps focus on recipe storage and basic scheduling without addressing the core timing complexity and meal composition problems. They require manual coordination of:

- Preparation sequences and timing across multiple weeks
- Shopping and ingredient freshness for extended planning horizons
- Day-of-week energy and availability matching
- Accompaniment pairing (rice with curry, pasta with sauce, sides with mains)
- Dietary restrictions filtering
- Recipe rotation to prevent repetition across weeks
- Real-time disruption handling

**Critical Gap:** No existing solution generates multi-week meal plans with intelligent accompaniment pairing and preference-aware scheduling.

### Urgency and Importance

The home cooking market has expanded significantly post-2020, with families cooking 40% more meals at home. This creates both increased demand for cooking solutions and heightened frustration with planning complexity. The timing is optimal for an intelligent automation solution.

## Proposed Solution

### Core Concept and Approach

imkitchen uses multi-factor optimization to automatically generate meal plans for all remaining weeks of the current month that consider:

- **Month-Based Planning Horizon:** Generate all future weeks of current month (from next week onwards) in a single operation, automatically extending into next month if at month-end
- **Intelligent Accompaniment Pairing:** Automatically pair main courses with appropriate sides (pasta, rice, fries, salad, bread, vegetables) based on recipe compatibility
- **Preference-Aware Scheduling:** Match recipes to days based on dietary restrictions and cuisine variety preferences
- **Smart Recipe Rotation:** Ensure main courses are unique across all weeks, while allowing appetizers/desserts to repeat after full rotation
- **Personalized Shopping Lists:** Generate separate shopping lists for each week including both main recipes and accompaniments
- **Real-time Disruption Adaptation:** Regenerate individual weeks or all future weeks while preserving the current locked week

### Key Differentiators

1. **Month-Based Planning Horizon:** Automatically generates all future weeks of current month in one operation (free tier sees first week only; premium sees all weeks), providing unprecedented planning visibility with seamless month transitions
2. **Intelligent Accompaniment System:** First meal planner to automatically pair main courses with compatible sides (pasta, rice, fries, salad, bread, vegetables) based on recipe preferences
3. **Preference-Aware Algorithm:** Considers dietary restrictions and cuisine variety preferences when scheduling
4. **Smart Week Locking:** Current week locked from regeneration to preserve in-progress meals, while future weeks remain flexible
5. **Advanced Recipe Rotation:** Main courses guaranteed unique across all generated weeks; appetizers/desserts rotate intelligently to maximize variety
6. **Preparation Automation:** Detailed morning reminders with specific timing for advance preparation based on recipe requirements
7. **Community Integration:** Social recipe sharing with four recipe types (appetizer, main course, dessert, accompaniment) and rating system
8. **Shopping Intelligence:** Separate shopping lists per week with ingredient grouping, quantity optimization, and accompaniment inclusion

### Why This Solution Will Succeed

- **Eliminates Core Friction:** Directly addresses timing complexity, meal composition realism, and long-term planning visibility that existing solutions ignore
- **Trust Through Results:** Multi-week generation demonstrates immediate value—users see their entire month of meals in seconds
- **Realistic Meal Composition:** Accompaniment pairing reflects how people actually eat (curry with rice, pasta with sauce), not isolated dishes
- **Respects User Constraints:** Algorithm honors dietary restrictions—plans users can actually execute
- **Network Effects:** Community features with four recipe types create sustainable engagement and content growth
- **Clear Value Proposition:** Measurable increase in recipe variety (2x more unique recipes per month) and decreased planning stress (60% time reduction)

### High-Level Product Vision

A comprehensive cooking ecosystem that transforms meal planning from a daily stressor into an automated background process, enabling users to focus on the joy of cooking while accessing their full culinary potential.

## Target Users

### Primary User Segment: Home Cooking Enthusiasts

**Demographics:**
- Age: 28-45
- Household income: $50,000+
- Family status: Couples and families with children
- Location: Suburban and urban areas

**Current Behaviors:**
- Save 50+ recipes but cook only 10-15 regularly
- Spend 15-30 minutes weekly on meal planning
- Shop for groceries 1-2 times per week
- Avoid complex recipes due to timing uncertainty

**Specific Needs and Pain Points:**
- Want culinary variety without planning complexity
- Need reliable advance preparation reminders
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
- Limited weeknight cooking time

**Current Behaviors:**
- Heavy reliance on meal kit services or takeout
- Weekend meal preparation when possible
- Simple weeknight recipes only
- Bulk shopping trips

**Specific Needs:**
- Minimal weeknight preparation time
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
- **Engagement:** 70% weekly active user retention rate
- **Revenue:** $50,000 monthly recurring revenue by month 12
- **Growth:** 25% month-over-month user growth through first year
- **Market Position:** Recognized as leading intelligent meal planning platform

### Monetization Model

**Freemium Strategy:**
- **Free Tier:** Users generate meal plans for all future weeks of current month (from next week onwards), but can only **view/access the first generated week**. Other weeks are hidden with upgrade prompts. Unlimited regenerations allowed—each regeneration replaces all future weeks but maintains first-week-only visibility. Dashboard shows nearest day's meals only if it falls within the accessible first week. Full access to all other features including recipe management, import, rating, community sharing, and recipe favorites with a **maximum of 10 favorited recipes**.
- **Premium Tier:** Generates meal plans for all future weeks of current month with **full visibility across all generated weeks**. Unlimited regenerations. Dashboard always shows nearest day's meals from any generated week. **Unlimited recipe favorites**, priority support, and early access to new features.
- **Upgrade Incentive:** Free users experience intelligent meal planning for one week and can regenerate unlimited times to perfect it, but face constraints: (1) Cannot see beyond first week of generated meal plans, (2) Limited to 10 favorited recipes maximum. This creates natural desire to unlock full month visibility, unlimited recipe favorites, and better planning capabilities.
- **Upgrade Prompts:** Multiple touchpoints including: (1) When clicking locked weeks in calendar, (2) On dashboard if nearest day is hidden (falls outside accessible first week), (3) When viewing calendar with locked weeks visible. Persistent but not intrusive reminders throughout the experience.
- **Conversion Target:** 15% of free users upgrade to premium within 60 days

**Pricing Structure:**
- **Monthly Subscription:** $9.99/month with auto-renewal
- **Annual Subscription:** $59.94/year ($4.99/month equivalent) - **50% savings** compared to monthly
- **Payment:** Credit card auto-renewal, cancellable anytime by user through account settings
- **Subscription Management:** Users retain premium access until end of paid period, then immediately downgrade to free tier (no grace period)
- **After Expiration:** Premium users revert to free tier with first-week-only visibility. Previously generated meal plans remain accessible according to free tier rules (first week only).

### User Success Metrics

- **Recipe Variety Increase:** Users cook 3x more unique recipes per month (enabled by multi-week rotation ensuring no main course repeats)
- **Planning Time Reduction:** 60% decrease in weekly meal planning time (one-click generation of 5 weeks vs manual weekly planning)
- **Planning Horizon Extension:** Users plan average of 3.5 weeks ahead (vs 1 week with traditional apps)
- **Accompaniment Success:** 85% of main courses with accompaniment preference successfully paired with appropriate sides
- **Preference Match Rate:** 95% of generated meals meet user's dietary restrictions
- **Preparation Success:** 90% of users successfully complete advance preparation tasks
- **Stress Reduction:** 70% report decreased meal planning anxiety
- **Food Waste Reduction:** 40% decrease in unused ingredient waste (improved by multi-week shopping visibility)

### Key Performance Indicators (KPIs)

- **Daily Active Users (DAU):** Target 30% of monthly users
- **Multi-Week Adoption:** 70% of users generate 3+ weeks of meal plans (vs single week)
- **Average Weeks Generated:** 4.2 weeks per generation operation
- **Week Regeneration Rate:** <15% of weeks regenerated (indicating high initial satisfaction)
- **Recipe Completion Rate:** 85% of scheduled meals are successfully prepared
- **Accompaniment Pairing Rate:** 60% of main courses served with algorithm-selected accompaniment
- **Preference Configuration:** 80% of users configure dietary restrictions
- **Community Engagement:** 40% of users rate or review recipes monthly
- **Premium Conversion:** 15% of free users upgrade to premium within 60 days
- **Shopping List Usage:** 80% of users regularly generate shopping lists (with 2.5 weeks average per list access)
- **Recipe Import Adoption:** 45% of users import at least one recipe batch within first 30 days
- **Average Recipes Imported:** 15-25 recipes per importing user (bulk onboarding efficiency)
- **Import Success Rate:** >95% of valid JSON files successfully processed without errors
- **Algorithm Performance:** <5 seconds P95 for 5-week meal plan generation

## MVP Scope

### Core Features (Must Have)

- **Home Page Route (/):** Dynamic routing based on authentication state. If user is logged in → display Dashboard page. If user is not authenticated → display SEO-optimized landing page showcasing main features, benefits, value proposition, and call-to-action for registration/login. **Landing Page Content:** Comprehensive layout including hero section with value proposition, key features showcase (meal planning, accompaniments, shopping lists), how-it-works section (3-step process), example meal plan screenshots, pricing preview (monthly/annual tiers), testimonials/social proof (when available), and clear CTA buttons throughout.
- **Dashboard Page:** Display nearest day's recipes and preparation tasks from generated meal plan with button to generate or regenerate. **Freemium:** Shows nearest day only if it falls within free tier's accessible first week (or current week if it exists); otherwise hidden with upgrade prompt. Premium always shows nearest day from any generated week. **Empty State:** When no meal plans exist, display quick onboarding guide explaining how meal planning works.
- **Recipe Management:** Users create and manage their own recipes with optional sharing to other users (includes four recipe types: Appetizer, Main Course, Dessert, and Accompaniment). Recipe fields including dietary restrictions, cuisine type, and all other attributes must be **explicitly set by users**—no automatic detection or inference from recipe content. **Community Sharing:** All four recipe types are sharable in both free and premium tiers.
- **Recipe Import:** Bulk import recipes from JSON files (max 10MB per file, 20 files per batch) via drag-and-drop or file picker interface. Imported recipes stored as private by default in user's recipe library. Supports all four recipe types with validation against HTML form schema. Invalid recipes skipped with detailed summary report showing success/failure counts. **Progress Feedback:** Real-time progress display showing imported count, failed count, and remaining count (e.g., "Imported 45 recipes, 3 failed, 12 remaining..."). **Validation Timing:** All validation performed during processing; errors shown only in final summary report after all files are processed. **Duplicate Detection:** System detects duplicate recipes (matching name or similar ingredients) and blocks them from being imported. Duplicates shown in summary report with warning. **Validation Strictness:** Only 100% valid recipes are imported; any recipe with missing required OR optional fields is rejected entirely to ensure data quality. **Import History:** No persistent storage of import operations; summary results shown only during current import session. **JSON Schema:** Publicly accessible versioned JSON schema documentation (e.g., v1.0, v1.1) enabling third-party tools to generate compatible recipe exports.
- **Month-Based Meal Planning:** Automatically generate meal plans for all future weeks of current month (from next week onwards, extending into next month if at month-end) from user's favorite recipes. Current week is never generated but preserved if it exists from previous generation. Unlimited regenerations replace all future weeks. Accessible from home page with single "Generate" button. Algorithm gracefully handles insufficient recipes by leaving empty meal slots—no minimum recipe count enforced (e.g., 10 favorites → Week 1 full, Week 2 partial with empty slots). **Generation is non-deterministic:** Each generation produces different meal slot assignments even with same favorite recipes, enabling users to regenerate until they find an arrangement they prefer. **Empty Slot Handling:** No warnings or notifications displayed when generation results in empty slots; users discover organically through calendar view. **Regeneration Confirmation:** "Regenerate" button requires confirmation dialog to prevent accidental replacement of all future weeks (e.g., "This will replace all future weeks. Continue?"). **Month Transition Messaging:** When generation at month-end extends into next month, display explicit message about the transition (e.g., "Generating weeks for rest of March + first week of April").
- **Visual Meal Calendar:** Month-based calendar view with week navigation, displaying 3 courses per day (appetizer, main course, dessert) with preparation indicators. **Freemium:** Free tier can only view first generated week; other weeks display generic "Upgrade to unlock" placeholders with upgrade prompts. Premium tier views all generated weeks with full navigation. **Empty meal slots** display with suggestion to "Browse community recipes" with link to community recipe page.
- **Week Locking:** Current week (today falls within Monday-Sunday range) is locked and cannot be regenerated, preserving in-progress meals. Future weeks can be regenerated individually or all at once. **Current Week Indicator:** Visual badge or lock icon displayed on current week header in calendar view with tooltip explaining preservation (e.g., "Current Week - Won't be regenerated").
- **Accompaniment System:** Main courses can optionally accept accompaniments (pasta, rice, fries, salad, bread, vegetables). Users create accompaniment recipes and algorithm pairs them with compatible main courses based on preferences. **Default:** Main courses default to NOT accepting accompaniments (`accepts_accompaniment=false`); users must explicitly enable accompaniment pairing when creating/editing recipes.
- **Recipe Rotation System:** Main courses must be unique across all generated weeks until all favorites are exhausted, then slots remain empty. Appetizers and desserts can repeat after all are used once. Accompaniments can repeat freely. Empty slots appear when insufficient favorited recipes available for rotation requirements.
- **Shopping List Generation:** Separate shopping lists for each week with ingredient grouping and quantity optimization. Includes both main recipes and accompaniments. Accessible for current and all future weeks.
- **User Preferences Integration:** Algorithm considers dietary restrictions, cuisine variety preferences, and complexity spacing when generating meal plans. **Default cuisine variety weight:** 0.7 (where 0.0 = repeat cuisines frequently, 1.0 = maximum variety; configurable per user).
- **Basic Preparation Reminders:** Morning notifications for advance preparation tasks based on recipe-specific requirements. **Notification Timing:** All advance prep notifications sent at 8 AM daily, regardless of prep type (e.g., "Remember to marinate chicken for tonight's dinner").
- **Recipe Rating System:** Community-driven quality feedback and reviews on shared recipes. **Quality Control:** Ratings and reviews act as quality filter with low-rated recipes buried in search results. Community self-moderation through feedback with no pre-approval required - trusting users to maintain recipe quality.
- **Recipe Favorites:** Users can favorite both their own recipes and community-shared recipes from other users. Favorited recipes are available for meal plan generation and stored in user's recipe library. Users can view and manage all favorited recipes in their profile. **Freemium:** Free tier limited to **maximum 10 favorited recipes**; premium tier has unlimited favorites. If a recipe owner deletes their recipe, it is automatically removed from all users' favorites (no notifications sent to recipe creators when favorited). **Limit UX:** When free tier users attempt to favorite beyond 10 recipes, show upgrade modal with clear message (e.g., "You've reached your 10 favorites limit. Upgrade to Premium for unlimited favorites") - no unfavoriting option in modal, strong conversion incentive.
- **User Profile Management:** Dietary restrictions, complexity preferences, cuisine variety weight, favorites, and household size
- **Contact Us Form:** Public form for users to submit questions, feedback, and support requests with fields for name, email, subject, and message. **Admin Notifications:** Email notifications sent to admin(s) when new contact form messages are submitted for real-time awareness and better response time.
- **Admin Panel:** Administrative interface for platform management with user management (view, edit, suspend/activate accounts, manage premium bypass flags) and contact form inbox (view submitted messages, mark as read/resolved, search/filter by date/status). **Admin Access:** Admins identified by dedicated `is_admin` flag in user profile; admin users have full access to admin panel and management features. **User Suspension Impact:** When admin suspends a user, the user cannot log in, their meal plans become inaccessible to them, and their shared recipes are hidden from community view. All data is preserved for potential reactivation but not visible during suspension.
- **Mobile-Responsive Design:** Touch-optimized interface for kitchen use with week carousel navigation

### Out of Scope for MVP

- **Machine Learning Features:** ML-powered notification timing based on individual cooking speed patterns, predictive recipe recommendations, adaptive difficulty adjustment
- **Grocery Store API Integrations:** One-tap ordering through partner services, real-time inventory checking, automatic price comparison
- **Advanced Social Features:** Community contests, chef profiles, recipe collections, public/private recipe sharing settings (basic sharing and rating included in MVP)
- **Smart Kitchen Device Integration:** Connected appliances, automated inventory tracking, IoT temperature monitoring
- **Video Cooking Guidance:** Step-by-step video instructions, AR cooking assistance, live cooking sessions
- **Extended Planning Factors:** Weather-based recipe suggestions, family calendar integration, energy level tracking, seasonal ingredient preferences
- **Recipe Collections & Templates:** Curated themed collections, pre-built diet-specific meal plan templates (Keto, Mediterranean, etc.)
- **Advanced Regeneration:** Partial week regeneration (regenerate specific days within locked week), constraint relaxation suggestions when insufficient recipes

### MVP Success Criteria

Successfully demonstrate that multi-week intelligent automation with accompaniment pairing can increase recipe variety while reducing planning complexity:

- **Recipe Variety:** Users cook at least 2x more unique recipes per month compared to pre-app usage
- **Planning Efficiency:** 80% reporting reduced meal planning stress and 60% time savings
- **Month-Based Adoption:** 70% of users consistently generate meal plans monthly (month-based generation)
- **Accompaniment Success:** 85% of main courses with accompaniment preference successfully paired
- **Preference Match:** 95% of generated meals meet user's dietary restrictions
- **Technical Performance:** <5 seconds P95 for month-based meal plan generation (up to 5 weeks), <0.1% error rate
- **User Satisfaction:** >4.0/5.0 average rating for month-based meal planning feature
- **Freemium Conversion:** 15% of free users upgrade to premium within 60 days, demonstrating value of first-week access and desire for full month visibility

## Post-MVP Vision

### Phase 2 Features

- **Extended Scheduling Factors:** Additional optimization considering user energy patterns, weather forecasts, and family calendar events (MVP already includes dietary restrictions, cuisine variety, and complexity spacing)
- **Community Recipe Discovery:** Enhanced social features with recipe collections, contests, and chef profiles (MVP includes basic sharing and rating)
- **Grocery Store Integration:** One-tap ordering through partner services with real-time inventory and pricing
- **Enhanced Notifications:** ML-powered notification timing based on individual cooking speed patterns and historical prep times
- **Recipe Collections:** Curated themed collections (e.g., "Summer BBQ Favorites", "Quick Weeknight Dinners")
- **Meal Plan Templates:** Pre-built meal plan templates for specific diets (Keto, Mediterranean, etc.)

### Long-term Vision

Transform imkitchen into a comprehensive cooking ecosystem that connects meal planning, grocery shopping, preparation guidance, and community learning. Users experience cooking as a creative, stress-free activity supported by intelligent automation and community inspiration.

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
- **Integration Requirements:** Push notification APIs, future grocery store API compatibility
- **Security/Compliance:** OWASP security standards for all security-related implementations, JWT for cookie-based authentication tokens, user data encryption, GDPR compliance, secure payment processing ready
- **Analytics & Privacy:** Minimal anonymous analytics only - aggregate, anonymized metrics for basic usage statistics (generation counts, feature adoption rates). No individual user tracking. Privacy-first approach with less optimization data but stronger user privacy protection.
- **Recipe Import:** JSON schema validation matching HTML form structure, 10MB file size limit enforcement, batch processing for up to 20 concurrent files, malicious file detection (script injection, oversized payloads), streaming parser for large files, detailed validation error reporting per recipe, rollback capability for failed batches
- **Premium Bypass Configuration:** System configuration option to bypass premium restrictions for development, testing, demo accounts, and internal team access. Supports two implementation approaches: (1) Global bypass via application config file for entire environment (e.g., development/staging), (2) Per-user bypass flag in user profile for selective access (e.g., demo accounts, internal team members).
- **Recipe Snapshot/Copy for Meal Plans:** Each week's meal plan must create and store a complete snapshot/copy of all referenced recipes (main courses, accompaniments, appetizers, desserts) at generation time. This ensures meal plan integrity if original recipe owner deletes or modifies their recipe. Users can always access historical meal plans with original recipe data intact, regardless of source recipe changes.

## Constraints & Assumptions

### Constraints

- **Technical:** Must work reliably on mobile devices, offline recipe access required
- **Performance:** Multi-week generation must complete in <5 seconds for 5 weeks
- **Data Model:** Event sourcing requires careful migration strategy for schema changes
- **Algorithm:** Must generate meal plans with user's existing recipe library, gracefully handling insufficient recipes by leaving empty slots (no minimum recipe count enforced)

### Key Assumptions

- Users will trust automated month-based meal planning if it demonstrably saves time and respects their constraints
- Accompaniment pairing will be perceived as added value, not unnecessary complexity
- Users will configure preferences (dietary restrictions) during onboarding
- Community features will drive organic growth and engagement through four recipe types
- **Freemium model** (first week visibility only, unlimited regenerations) provides enough value to demonstrate benefit while creating strong upgrade incentive
- Allowing unlimited regenerations removes frustration while limited visibility (first week only) creates desire for full month visibility
- Experiencing one week of intelligent meal planning will convert 15% of free users to premium within 60 days when they want to see beyond first week for better planning
- Automatic month-based generation (no week selection required) simplifies user experience and reduces decision fatigue
- Grocery partnerships will provide sustainable revenue streams
- Mobile-first approach with week carousel navigation is sufficient for initial market penetration
- Week locking (current week) will be accepted as protecting in-progress meals, not limiting flexibility
- Month-based planning horizon provides sufficient value without overwhelming users

## Risks & Open Questions

### Key Risks

- **User Adoption:** Users may resist changing established meal planning habits, especially multi-week generation vs familiar single-week approach
- **Algorithm Complexity:** Multi-week generation with preference-aware filtering may encounter edge cases where constraints cannot be satisfied
- **Performance at Scale:** 5-week generation must remain <5 seconds even with large recipe libraries (100+ favorites)
- **Accompaniment Confusion:** Users may not understand optional accompaniment system or when/how to use it
- **Preference Configuration Friction:** Users may skip dietary restriction setup, reducing algorithm effectiveness
- **Content Quality:** Community recipe sharing (including new accompaniment type) may require significant moderation
- **Competition:** Existing meal planning apps may quickly copy month-based generation and accompaniment features
- **Monetization:** Premium feature uptake may be lower than projected
- **Freemium Visibility Frustration:** Free users may feel restricted by first-week-only visibility when they want to see full month for grocery planning or long-term visibility
- **Month-End Generation:** Users generating at month-end (e.g., last week of month) will see plans extending into next month, which may cause confusion about "current month" generation logic
- **Week Locking Frustration:** Users may want to regenerate current week and be frustrated by locking constraint
- **Recipe Import Security:** Malicious JSON files could contain script injections, oversized payloads, or trigger denial-of-service attacks
- **Import Schema Compatibility:** Users may have recipes in incompatible formats requiring manual conversion or extensive documentation
- **Bulk Import Performance:** Processing 20 files × 10MB could cause UI blocking or server timeouts without proper async handling
- **Favorited Recipe Deletion Impact:** When recipe owner deletes a shared recipe, it's removed from all users' favorites. Users with fewer favorited recipes will see more empty slots in generated meal plans, potentially reducing usefulness of the feature.

### Areas Needing Further Research

- **Competitive Analysis:** Do any existing apps offer multi-week generation or accompaniment pairing?
- **User Onboarding:** How to educate users on multi-week benefits, accompaniment system, and preference configuration?
- **Algorithm Edge Cases:** How to handle impossible constraint combinations (e.g., vegan + nut-free + gluten-free with limited recipes)?
- **International Expansion:** How do accompaniment categories (pasta, rice, fries) translate to other cuisines?
- **Accessibility:** How to make multi-week calendar view accessible to screen readers and keyboard navigation?
- **Content Moderation:** Policies and tools for moderating community-shared accompaniment recipes
- **Performance Optimization:** Query optimization strategies for dietary tag filtering and recipe selection at scale
- **Recipe Import Standards:** Research existing recipe interchange formats (Schema.org Recipe, RecipeML, JSON-LD) to determine interoperability requirements and migration paths

## Appendices

### A. Research Summary

**Brainstorming Session (September 2025):** Comprehensive 65-minute facilitated session generated 47 distinct features and concepts. Key insights include the identification of self-limitation patterns in recipe selection and the critical importance of trust-building through demonstrated time savings.

**Architecture Design (October 2025):** Detailed technical design for Enhanced Meal Planning System covering multi-week generation (up to 5 weeks), accompaniment recipe type system (7 categories), and user preferences integration (dietary restrictions, cuisine variety). Includes complete database schema, domain model, algorithm design, and 9-week implementation roadmap.

**Key Findings:**
- 80% of cooking planning happens on mobile devices
- Users avoid complex recipes primarily due to timing uncertainty, not skill limitations
- **Meal composition realism matters:** Users want accompaniments (rice, pasta, sides) automatically paired with main courses
- **Planning horizon visibility:** Month-based view reduces weekly planning friction and enables better grocery shopping
- **Constraint respect is critical:** Algorithm must honor dietary restrictions for user trust
- Community features are essential for sustainable engagement and organic growth (extended to four recipe types)
- Freemium model with first-week-only visibility (unlimited regenerations) optimally balances trial value with upgrade pressure for full month visibility and better planning

### B. Stakeholder Input

Initial concept development involved extensive user journey mapping and pain point analysis. Primary stakeholder (product owner) emphasized the importance of mobile-first design and offline functionality for kitchen environments.

**Architecture Design Decisions (October 2025):**
- **5-week maximum:** Balances long-term planning visibility with UI complexity and computation limits
- **Accompaniments always optional:** Respects recipe creator's intent; main courses control `accepts_accompaniment` boolean (defaults to `false`, must be explicitly enabled)
- **Custom cuisines allowed:** `Cuisine::Custom(String)` variant enables user-defined cuisine types
- **Regeneration confirmation required:** "Regenerate All Future Weeks" displays modal to prevent accidental bulk regeneration
- **Cuisine preferences inferred:** No explicit preference storage; algorithm infers from user's favorite recipe selection
- **Advance prep recipe-defined:** Prep timing stored in recipe's `advance_prep_text`, not user preference
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
   - Test freemium model assumptions (Monday-only visibility conversion rate)
   - Validate accompaniment pairing value proposition

2. **Define Technical Architecture**
   - Create detailed technical specification for multi-week generation algorithm
   - Design database schema and data persistence strategy
   - Plan freemium access control and upgrade flow

3. **Design User Experience**
   - Wireframe multi-week calendar navigation (desktop and mobile)
   - Design freemium placeholder UX (locked Tuesday-Sunday display)
   - Create accompaniment pairing UI patterns
   - Design preference configuration onboarding flow

4. **Build MVP**
   - Develop core meal planning features (multi-week generation, accompaniment system, user preferences)
   - Implement freemium restrictions (Monday-only visibility for free tier)
   - Create recipe management with four recipe types
   - Build shopping list generation per week

5. **Launch & Iterate**
   - Beta launch with early adopters
   - Monitor conversion metrics (free to premium upgrade rate)
   - Gather user feedback on freemium restrictions and accompaniment pairing
   - Iterate based on user behavior and feedback

### PM Handoff

This Project Brief provides the full context for imkitchen, including the enhanced meal planning features (multi-week generation, accompaniment system, user preferences integration). The detailed technical architecture is documented separately. Please start in 'PRD Generation Mode', review the brief thoroughly to work with the user to create the PRD section by section as the template indicates, asking for any necessary clarification or suggesting improvements.

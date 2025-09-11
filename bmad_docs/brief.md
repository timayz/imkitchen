# ImKitchen Product Requirements Document (PRD)

## Goals and Background Context

### Goals
- Enable home cooks to expand their culinary repertoire by 150% without increasing planning time through automated meal selection and timing intelligence
- Eliminate decision fatigue around weekly meal planning by providing one-tap "Fill My Week" automation from curated recipe collections
- Achieve 85%+ success rate for complex recipe execution through advance preparation notifications and timing coordination
- Build a community-driven recipe platform focused on practical execution guidance rather than visual presentation
- Establish market-leading retention (70%+ weekly) and conversion (15% to premium) through timing intelligence differentiation
- Support global accessibility with multi-language interface and diverse cultural recipe sharing

### Background Context
ImKitchen addresses a fundamental paradox in home cooking: despite having access to countless recipes, home cooks artificially limit themselves to 10-15 simple dishes due to timing complexity rather than skill limitations. The core insight is that users avoid complex recipes not because they lack cooking ability, but because they cannot reliably coordinate advance preparation requirements (marinades, dough rising, component cooking) with their daily schedules.

Current meal planning solutions treat this as an organizational problem, focusing on recipe collection and basic scheduling. ImKitchen recognizes it as an optimization problem requiring intelligent automation. By combining a multi-factor scheduling engine with community-driven content that prioritizes execution guidance over visual appeal, the platform transforms cooking complexity from a barrier into an accessible advantage.

### Change Log
| Date | Version | Description | Author |
|------|---------|-------------|--------|
| 2025-09-10 | v1.0 | Initial PRD creation from Project Brief | PM Agent |

## Requirements

### Functional Requirements

**FR1:** The system shall provide "Fill My Week" automated meal selection that generates weekly meal plans from the user's curated recipe collection with no-duplicate rotation until all selected recipes are used.

**FR2:** The system shall display a visual meal calendar interface showing breakfast/lunch/dinner slots with color-coded prep indicators and timing requirements.

**FR3:** The system shall send morning prep reminders with specific task lists and estimated durations for advance preparation requirements (marinades, dough prep, component cooking).

**FR4:** The system shall auto-generate shopping lists from weekly meal selections with basic ingredient grouping and quantity adjustment for family size.

**FR5:** The system shall provide a curated recipe database with 100+ professionally tested recipes including detailed timing data across common cuisines and skill levels.

**FR6:** The system shall allow users to create and input personal recipes with timing steps and share them publicly or privately.

**FR7:** The system shall implement a 5-star rating and review system focused on execution success, timing accuracy, and practical modifications.

**FR8:** The system shall support multi-language interface and recipe content for global accessibility.

**FR9:** The system shall allow easy meal rescheduling through drag-and-drop calendar interface for life disruptions.

**FR10:** The system shall enable local web browser preview of the application during development and after story completion.

### Non-Functional Requirements

**NFR1:** The system shall achieve <2 second initial load time and <500ms navigation between screens for optimal mobile experience.

**NFR2:** The system shall maintain 99.5% notification delivery reliability for timing intelligence features.

**NFR3:** The system shall support Progressive Web App (PWA) capabilities including offline functionality via service workers.

**NFR4:** The system shall maintain vendor-neutral architecture avoiding technology lock-in for platform independence.

**NFR5:** The system shall comply with GDPR requirements for EU users and implement secure API authentication with JWT tokens.

**NFR6:** The system shall support modern browsers (Chrome 90+, Safari 14+, Firefox 85+) with consistent functionality.

**NFR7:** The system shall implement mobile-first responsive design optimized for smartphone usage patterns.

**NFR8:** The system shall enable containerized deployment with Docker and Kubernetes for scalability.

## User Interface Design Goals

### Overall UX Vision
ImKitchen provides a mobile-first cooking companion that feels like having a knowledgeable friend guiding your meal planning. The interface prioritizes speed and clarity during actual cooking sessions, with large touch targets, high contrast text, and intuitive navigation that works even when hands are messy or distracted. The design balances automation (reducing decision fatigue) with user control (easy customization and rescheduling), creating a sense of intelligent assistance rather than rigid constraint.

### Key Interaction Paradigms
- **One-Tap Automation:** Primary actions like "Fill My Week" are prominent single-tap operations that provide immediate value
- **Visual Timeline Interface:** Calendar-based meal planning with drag-and-drop rescheduling and color-coded timing indicators
- **Progressive Disclosure:** Complex features like recipe creation are layered behind simple entry points to avoid overwhelming casual users
- **Context-Aware Notifications:** Timing reminders adapt to user location and calendar integration, appearing when actionable
- **Touch-First Design:** All interactions optimized for smartphone use with gesture support and minimal text input requirements

### Core Screens and Views
- **Dashboard/Home Screen:** Quick access to today's meals, upcoming prep tasks, and "Fill My Week" automation
- **Meal Calendar View:** Weekly grid showing assigned meals with prep indicators and easy rescheduling
- **Recipe Detail Pages:** Step-by-step cooking instructions with timing guidance and community reviews
- **Shopping List Interface:** Auto-generated lists with checkbox completion and quantity adjustment
- **Recipe Discovery/Browse:** Community recipe exploration with execution-focused filtering and search
- **Recipe Creation Form:** Simple input for personal recipes with timing step wizard
- **Profile/Settings:** User preferences, dietary considerations, and account management

### Accessibility: WCAG AA
Full WCAG AA compliance ensuring usability for users with visual, motor, and cognitive impairments. Critical for cooking context where users may have limited dexterity or attention.

### Branding
Clean, modern interface with warm cooking-inspired color palette (earth tones, fresh ingredient colors). Visual design emphasizes reliability and warmth over flashy aesthetics, building trust for timing intelligence features. Typography optimized for kitchen lighting conditions with high contrast and readable fonts.

### Target Device and Platforms: Web Responsive
Progressive Web App optimized for mobile-first experience with responsive scaling for tablet and desktop. Native app capabilities through PWA including offline access, push notifications, and home screen installation.

## Technical Assumptions

### Repository Structure: Monorepo
Single repository containing shared components and utilities with separate packages for web app, API services, database migrations, and shared libraries. This approach facilitates code reuse between frontend and backend while maintaining clear separation of concerns.

### Service Architecture
**Microservices within Monorepo:** Container-based microservices architecture with dedicated services for user management, recipe catalog, meal planning engine, notification system, and community features. Services communicate via RESTful APIs and GraphQL for complex queries, enabling independent scaling and development while maintaining deployment simplicity through monorepo structure.

### Testing Requirements
**Full Testing Pyramid:** Comprehensive testing strategy including unit tests for business logic, integration tests for API endpoints and database interactions, and end-to-end tests for critical user workflows. Local development environment must support test execution and provide fast feedback loops. Browser preview capability requires testing infrastructure that works across development, staging, and local environments.

### Additional Technical Assumptions and Requests

**Frontend Technology Stack:**
- Next.js/TypeScript with App Router for full-stack React framework with built-in optimization and SSR/SSG capabilities
- Progressive Web App (PWA) capabilities using Next.js PWA plugin for offline functionality and push notifications
- Tailwind CSS for mobile-responsive design system ensuring fast development and consistent styling
- Next.js internationalization (i18n) for multi-language support as specified in requirements
- Next.js development server with hot reload for browser preview functionality per user requirement

**Backend Technology Stack:**
- Node.js/TypeScript microservices for consistent language across frontend and backend
- PostgreSQL for primary data storage with ACID compliance for critical meal planning data
- Redis for caching, session management, and message queuing for background notifications
- Apache Solr for recipe search functionality with multi-language support

**Infrastructure and Deployment:**
- Next.js deployment optimized for Vercel or container-based deployment with Docker for vendor-neutral cloud portability
- Local development environment using Next.js dev server with optional Docker Compose for backend services
- CI/CD pipeline supporting Next.js build optimization, static generation, and automated testing
- Web Push API integration for reliable notification delivery without native app dependency

**Security and Compliance:**
- JWT token-based authentication with secure session management
- GDPR compliance implementation for EU users with data portability and deletion capabilities
- Content moderation tools for community-generated recipes and reviews
- Encrypted storage for sensitive user data including dietary restrictions and preferences

**Performance and Scalability:**
- Next.js automatic code splitting, image optimization, and lazy loading for optimal mobile performance (<2s initial load requirement)
- Event-driven architecture with message queues for background processing and async notifications
- Next.js PWA with IndexedDB for offline recipe storage enabling kitchen use without reliable internet
- Monitoring and logging infrastructure for 99.5% notification delivery reliability tracking

## Epic List

**Epic 1: Foundation & Core Recipe Management**
Establish project infrastructure, basic user authentication, and core recipe CRUD operations with local development environment and initial recipe database.

**Epic 2: Automated Meal Planning Engine**
Implement "Fill My Week" automation, visual meal calendar interface, and basic scheduling algorithms that form the core value proposition.

**Epic 3: Timing Intelligence & Notifications**
Build the advance preparation notification system, timing coordination features, and prep reminder functionality that differentiates ImKitchen from competitors.

**Epic 4: Community Features & Shopping Integration**
Add user-generated content capabilities, recipe rating/review system, and automated shopping list generation to complete the MVP feature set.

## Epic 1: Foundation & Core Recipe Management

**Epic Goal:** Establish a solid technical foundation with containerized development environment, user authentication, and core recipe management capabilities that provide immediate value through browseable recipe database while enabling local browser preview for development workflow.

### Story 1.1: Project Infrastructure Setup

As a developer,
I want a containerized development environment with hot reload capabilities,
so that I can develop and preview the application locally in my web browser with fast feedback loops.

#### Acceptance Criteria
1. Next.js project setup with TypeScript, Tailwind CSS, and PWA configuration
2. Next.js development server supports hot reload and serves application accessible via web browser at localhost:3000
3. Backend API setup (Next.js API routes or separate microservice) with automatic restart on code changes
4. PostgreSQL database container initializes with development schema and seed data via Docker Compose
5. Development environment starts with `npm run dev` (frontend) and `docker-compose up` (database/backend services)
6. Development environment includes debugging capabilities, TypeScript support, and clear error logging
7. Documentation provides setup instructions for cross-platform development with Next.js

### Story 1.2: User Authentication System

As a home cook,
I want to create an account and securely log in,
so that I can save my personal recipes and meal planning preferences.

#### Acceptance Criteria
1. User registration form collects email, password, and basic profile information with client-side validation
2. JWT-based authentication system provides secure login/logout functionality
3. Password requirements enforce minimum security standards with clear user feedback
4. Email verification workflow confirms account activation before full access
5. User session persists across browser sessions with secure token refresh mechanism
6. Account settings page allows password changes and profile updates
7. Authentication middleware protects API endpoints requiring user context

### Story 1.3: Recipe Database Schema and Models

As a system administrator,
I want a robust database schema for recipes with timing data,
so that the application can store and query recipe information efficiently.

#### Acceptance Criteria
1. PostgreSQL schema defines recipe table with title, description, instructions, and timing metadata
2. Ingredient table with quantities, units, and preparation notes linked to recipes
3. User table stores authentication data and profile preferences with proper indexing
4. Database migration system allows schema updates without data loss
5. Seed data script populates database with initial 20+ curated recipes for testing
6. Database indexes optimize common queries for recipe search and user data retrieval
7. Data models include validation constraints ensuring data integrity

### Story 1.4: Basic Recipe CRUD API

As a home cook,
I want to view, create, edit, and delete my personal recipes,
so that I can build my digital recipe collection.

#### Acceptance Criteria
1. REST API endpoints support full CRUD operations for recipes with proper HTTP status codes
2. Recipe creation endpoint accepts title, ingredients, instructions, and basic timing information
3. Recipe retrieval supports filtering by user ownership and basic search functionality
4. Recipe updates maintain version history and handle concurrent modification gracefully
5. Recipe deletion includes soft delete option to preserve meal planning history
6. API validation ensures required fields and data format consistency
7. Error responses provide clear messaging for client-side error handling

### Story 1.5: Recipe List and Detail Views

As a home cook,
I want to browse my recipe collection and view detailed recipe information,
so that I can access my recipes for meal planning and cooking.

#### Acceptance Criteria
1. Recipe list page displays user's recipes with thumbnail, title, and basic metadata in responsive grid
2. Recipe detail page shows complete recipe with ingredients, instructions, and timing information
3. Search functionality filters recipes by title, ingredients, or cuisine tags
4. Recipe cards include visual indicators for preparation time and difficulty level
5. Detail view optimized for mobile reading during cooking with large fonts and clear structure
6. Navigation allows easy movement between recipe list and detail views
7. Loading states and error handling provide smooth user experience

## Epic 2: Automated Meal Planning Engine

**Epic Goal:** Implement the core "Fill My Week" automation and visual meal calendar that solves the primary user pain point of decision fatigue while providing intelligent meal selection from user's curated recipe collection.

### Story 2.1: Basic Meal Calendar Interface

As a home cook,
I want to see my weekly meals in a visual calendar format,
so that I can understand my meal plan at a glance and plan my week effectively.

#### Acceptance Criteria
1. Weekly calendar view displays 7 days with breakfast/lunch/dinner slots in responsive grid layout
2. Calendar navigation allows moving between weeks with clear date indicators
3. Empty meal slots display placeholder content inviting meal assignment
4. Calendar adapts to mobile screen sizes with touch-friendly interaction areas
5. Today's date is visually highlighted with distinct styling
6. Calendar loads quickly (<500ms) and handles week transitions smoothly
7. Accessibility features support keyboard navigation and screen readers

### Story 2.2: Manual Meal Assignment

As a home cook,
I want to assign recipes to specific meal slots in my calendar,
so that I can manually plan my meals before using automation features.

#### Acceptance Criteria
1. Recipe selection modal allows browsing user's recipe collection with search functionality
2. Drag-and-drop interface enables moving recipes to calendar meal slots
3. Alternative tap-to-assign workflow works on mobile devices without drag capability
4. Assigned meals display recipe name, prep time, and visual thumbnail in calendar slot
5. Meal assignments persist in database and reload correctly on page refresh
6. Validation prevents assigning same recipe multiple times within a week
7. Clear feedback confirms successful meal assignment with visual updates

### Story 2.3: "Fill My Week" Automation Algorithm

As a busy home cook,
I want one-tap automation to fill my weekly meal plan,
so that I can eliminate decision fatigue while ensuring recipe variety.

#### Acceptance Criteria
1. "Fill My Week" button triggers algorithm that selects from user's recipe collection
2. Algorithm ensures no duplicate recipes within the same week
3. Meal selection considers basic constraints like recipe complexity distribution
4. Algorithm fills only empty meal slots, preserving manually assigned meals
5. Selection logic rotates through entire recipe collection before repeating recipes
6. Generated meal plan displays immediately in calendar with clear visual feedback
7. Algorithm performance completes selection within 2 seconds for collections up to 100 recipes

### Story 2.4: Meal Plan Editing and Rescheduling

As a home cook,
I want to easily modify my generated meal plan,
so that I can adapt to schedule changes and personal preferences.

#### Acceptance Criteria
1. Drag-and-drop rescheduling moves meals between calendar slots with visual feedback
2. Meal removal functionality clears slots and returns recipes to available pool
3. Replace meal option opens recipe selection for direct substitution
4. Undo functionality reverses recent meal plan changes
5. Bulk operations allow clearing entire days or meal types (all lunches, etc.)
6. Changes save automatically with visual confirmation of successful updates
7. Mobile-optimized editing workflow accommodates touch interactions and smaller screens

### Story 2.5: Recipe Collection Management

As a home cook,
I want to curate my recipe collection for meal planning,
so that automation only selects from recipes I actually want to cook.

#### Acceptance Criteria
1. Recipe collection page displays all user recipes with inclusion toggles for meal planning
2. Bulk selection tools enable quick inclusion/exclusion of multiple recipes
3. Collection management includes categories or tags for organizing recipes by meal type
4. Search and filter functionality helps users find specific recipes to include/exclude
5. Collection statistics show total included recipes and estimated weeks of variety
6. Import functionality allows adding recipes from public database to personal collection
7. Collection changes immediately affect "Fill My Week" algorithm behavior

## Epic 3: Timing Intelligence & Notifications

**Epic Goal:** Implement the advance preparation notification system and timing coordination features that transform complex recipe execution from a coordination challenge into a guided, reliable process.

### Story 3.1: Recipe Timing Data Model

As a recipe creator,
I want to define advance preparation steps with timing requirements,
so that users can successfully coordinate complex recipes with their daily schedules.

#### Acceptance Criteria
1. Recipe timing schema captures advance prep steps with duration estimates and deadlines
2. Timing editor interface allows recipe creators to define preparation sequences (marinate 4 hours, dough rise overnight, etc.)
3. Timing data includes step descriptions, minimum/maximum time windows, and dependency relationships
4. Recipe validation ensures timing logic is consistent and executable
5. Migration system updates existing recipes with basic timing data based on common patterns
6. Timing display shows visual timeline of preparation requirements in recipe detail view
7. API endpoints support CRUD operations for recipe timing data with proper validation

### Story 3.2: Notification Scheduling Engine

As the system,
I want to calculate optimal notification times based on meal schedules and timing requirements,
so that users receive timely preparation reminders without overwhelming them.

#### Acceptance Criteria
1. Scheduling engine processes weekly meal plans to generate notification timeline
2. Algorithm calculates backward from meal times to determine prep notification schedules
3. Notification batching combines multiple prep tasks into logical groupings (morning prep, day-before tasks)
4. Engine respects user-defined quiet hours and notification preferences
5. Scheduling handles conflicts when multiple meals have overlapping prep requirements
6. Background job processes schedule updates when meal plans change
7. Notification persistence stores scheduled reminders with retry logic for delivery failures

### Story 3.3: Web Push Notification System

As a home cook,
I want to receive advance preparation reminders on my device,
so that I can coordinate complex recipe timing without manual tracking.

#### Acceptance Criteria
1. Web Push API integration enables notification delivery to user devices
2. Notification permission request flow guides users through setup with clear value explanation
3. Push notifications include actionable content: task description, estimated duration, and completion tracking
4. Notification service worker enables reliable delivery even when browser is closed
5. Fallback mechanism uses in-app notifications when push notifications unavailable
6. User preferences allow customizing notification timing, frequency, and quiet hours
7. Notification analytics track delivery rates and user engagement to ensure system reliability

### Story 3.4: Preparation Task Management

As a home cook,
I want to track my preparation tasks and mark them complete,
so that I can stay organized and ensure nothing is missed in complex meal preparation.

#### Acceptance Criteria
1. Preparation dashboard displays upcoming tasks organized by timeframe (today, tomorrow, this week)
2. Task cards show recipe name, specific preparation step, estimated duration, and deadline
3. One-tap completion marking updates task status with timestamp
4. Visual progress indicators show preparation status for upcoming meals
5. Completed task history provides reference for timing adjustments and future planning
6. Task snoozing allows delaying reminders when life disrupts planned timing
7. Mobile-optimized interface works effectively during actual kitchen preparation

### Story 3.5: Timing Intelligence Optimization

As a user of the timing system,
I want the notification timing to improve based on my actual cooking patterns,
so that reminders become increasingly accurate and helpful over time.

#### Acceptance Criteria
1. System tracks user completion times versus estimated durations for timing adjustments
2. Learning algorithm adapts notification timing based on individual user patterns
3. Feedback mechanism allows users to report timing accuracy and suggest improvements
4. Recipe timing database updates with anonymized completion data to improve estimates
5. Personal timing preferences override global defaults for customized experience
6. Timing insights provide users with their cooking pattern analytics
7. Algorithm performance maintains real-time responsiveness while processing learning data

## Epic 4: Community Features & Shopping Integration

**Epic Goal:** Complete the MVP by enabling user-generated content with execution-focused recipe sharing, community ratings based on practical success, and automated shopping list generation that streamlines the entire cooking workflow.

### Story 4.1: Public Recipe Sharing Platform

As a cooking enthusiast,
I want to share my successful recipes with proper timing guidance,
so that other home cooks can reliably execute my dishes with the same success I've achieved.

#### Acceptance Criteria
1. Recipe publishing workflow allows users to make personal recipes publicly discoverable
2. Public recipe creation form includes timing data input with guided workflow for complex preparations
3. Recipe visibility controls allow authors to set public, private, or friends-only sharing levels
4. Public recipe discovery page displays community recipes with search and filter functionality
5. Recipe attribution clearly identifies original author with profile linking
6. Content moderation tools flag inappropriate content and maintain recipe quality standards
7. Recipe import feature allows users to add public recipes to their personal collections

### Story 4.2: Execution-Focused Rating and Review System

As a home cook,
I want to rate recipes based on execution success and timing accuracy,
so that the community can identify reliably successful recipes versus just visually appealing ones.

#### Acceptance Criteria
1. Rating system uses 5-star scale with execution-focused criteria (timing accuracy, instruction clarity, success rate)
2. Review interface prompts users to comment on timing accuracy, difficulty level, and practical modifications
3. Recipe pages display average ratings with breakdown of timing vs. taste vs. instruction quality
4. Review sorting prioritizes helpful execution feedback over generic praise
5. Verified completion badges indicate reviewers actually cooked the recipe through the platform
6. Review helpfulness voting allows community to surface most valuable practical feedback
7. Recipe success rate calculation aggregates completion data and positive reviews

### Story 4.3: Automated Shopping List Generation

As a home cook,
I want automatic shopping lists generated from my weekly meal plan,
so that I can efficiently shop for all my planned meals without manual list creation.

#### Acceptance Criteria
1. Shopping list generation analyzes weekly meal plan and aggregates all required ingredients
2. Ingredient consolidation combines quantities across recipes (3 eggs from recipe A + 2 eggs from recipe B = 5 eggs total)
3. Shopping list organization groups ingredients by store section (produce, dairy, pantry) for efficient shopping
4. List customization allows manual additions, deletions, and quantity adjustments
5. Cross-off functionality tracks shopping progress with persistent state across sessions
6. Family size scaling adjusts ingredient quantities based on household size preferences
7. Shopping list export enables sharing via text/email or printing for store use

### Story 4.4: Recipe Discovery and Recommendation

As a home cook,
I want to discover new recipes that match my preferences and success history,
so that I can expand my cooking repertoire with confidence in execution success.

#### Acceptance Criteria
1. Recipe recommendation engine suggests public recipes based on user's successful recipe patterns
2. Discovery filters include cuisine type, preparation time, difficulty level, and dietary considerations
3. "Similar recipes" feature finds alternatives to user's favorite recipes with comparable timing and complexity
4. Trending recipes highlight community favorites with high success rates and recent positive feedback
5. Seasonal recommendations promote recipes appropriate for current time of year
6. Recipe preview shows timing requirements and complexity before adding to collection
7. Recommendation accuracy improves based on user's rating and completion behavior

### Story 4.5: Community Engagement Features

As a recipe creator,
I want recognition for contributing valuable recipes and helpful feedback,
so that I'm motivated to continue sharing practical cooking knowledge with the community.

#### Acceptance Criteria
1. User profiles display contribution statistics including recipes shared, reviews written, and community helpfulness scores
2. Achievement system recognizes valuable contributors with badges for helpful reviews, successful recipes, and community engagement
3. Recipe author notifications inform creators when their recipes are cooked successfully or receive positive feedback
4. Community leaderboards highlight top contributors without creating unhealthy competition
5. Follow functionality allows users to subscribe to updates from trusted recipe creators
6. Recipe collection sharing enables users to curate and share themed recipe groups
7. Community guidelines clearly communicate execution-focused values and expected behavior

## Checklist Results Report

### Executive Summary
- **Overall PRD Completeness:** 95%
- **MVP Scope Appropriateness:** Just Right
- **Readiness for Architecture Phase:** Ready 
- **Most Critical Gaps:** Minor refinements needed in data requirements and operational specifics

### Category Analysis

| Category                         | Status  | Critical Issues |
| -------------------------------- | ------- | --------------- |
| 1. Problem Definition & Context  | PASS    | None - well grounded in Project Brief |
| 2. MVP Scope Definition          | PASS    | Excellent scope discipline with clear boundaries |
| 3. User Experience Requirements  | PASS    | Comprehensive UX vision with mobile-first focus |
| 4. Functional Requirements       | PASS    | Clear, testable requirements with FR10 addressing local preview |
| 5. Non-Functional Requirements   | PASS    | Strong performance and architecture constraints |
| 6. Epic & Story Structure        | PASS    | Excellent sequential structure with vertical slices |
| 7. Technical Guidance            | PASS    | Clear architecture direction with vendor neutrality |
| 8. Cross-Functional Requirements | PARTIAL | Data schema details could be more specific |
| 9. Clarity & Communication       | PASS    | Professional structure with clear terminology |

### Final Validation Result

✅ **READY FOR ARCHITECT** - The PRD and epics are comprehensive, properly structured, and ready for architectural design.

## Next Steps

### UX Expert Prompt
Review the ImKitchen PRD and create comprehensive UI/UX specifications focusing on mobile-first cooking companion experience. Priority areas: timing intelligence interface design, meal calendar visualization, and notification UX patterns. Ensure accessibility compliance and kitchen-context usability.

### Architect Prompt  
Design technical architecture for ImKitchen based on the PRD requirements. Focus on: Next.js/TypeScript frontend with API routes or microservices backend, notification scheduling engine, timing intelligence algorithms, and Next.js PWA implementation with offline capabilities. Address 99.5% notification reliability requirement and Next.js development environment setup.

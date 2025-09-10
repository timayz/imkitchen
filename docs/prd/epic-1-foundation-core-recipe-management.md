# Epic 1: Foundation & Core Recipe Management

Establish comprehensive project infrastructure including authentication, database design, and mobile app foundation while delivering immediate user value through personal recipe collection management and basic meal calendar functionality. This epic enables users to build their recipe library and visualize meal planning, creating the foundation for automation in Epic 2.

## Story 1.1: Project Infrastructure & Health Check

As a **developer**,  
I want **complete project setup with database, API, and mobile app foundation**,  
so that **the team can begin feature development with proper CI/CD and health monitoring**.

### Acceptance Criteria

1. Monorepo structure created with mobile app (Lynx.js), API (Go), and shared libraries
2. PostgreSQL database with basic schema and Redis caching layer configured
3. CI/CD pipeline established with automated testing and deployment to staging
4. Health check endpoints functioning and returning system status
5. Basic authentication framework integrated (OAuth2/JWT tokens)
6. Mobile app builds and displays health check page on iOS/Android

## Story 1.2: User Authentication & Profile Management

As a **home cooking enthusiast**,  
I want **secure account creation and profile management**,  
so that **I can safely store my personal recipe collection and preferences**.

### Acceptance Criteria

1. User registration with email/password and social login options (Google, Apple)
2. Secure login/logout functionality with session management
3. Profile page allowing users to update name, email, and cooking preferences
4. Password reset functionality via email verification
5. Account deletion option with data export capability
6. Cross-device authentication persistence for seamless mobile/web experience

## Story 1.3: Recipe Collection Management

As a **cooking enthusiast**,  
I want **to add, organize, and categorize my favorite recipes**,  
so that **I can build a digital collection of all my cooking preferences**.

### Acceptance Criteria

1. Manual recipe entry with title, ingredients, instructions, and prep time
2. Recipe categorization by meal type (breakfast, lunch, dinner) and complexity level
3. Recipe import from URLs with automatic ingredient extraction
4. Edit and delete recipe functionality with confirmation prompts
5. Recipe search and filtering by category, prep time, and difficulty
6. Photo attachment capability for recipe visualization

## Story 1.4: Basic Meal Calendar Display

As a **meal planner**,  
I want **a visual weekly calendar showing my planned meals**,  
so that **I can see my upcoming week at a glance and manually plan meals**.

### Acceptance Criteria

1. Weekly calendar view with breakfast, lunch, dinner slots for each day
2. Drag-and-drop recipe assignment to calendar slots from recipe collection
3. Color coding for meal complexity and advance preparation requirements
4. Navigation between weeks with clear date indicators
5. Empty state guidance encouraging users to add recipes and plan meals
6. Mobile-optimized calendar with touch-friendly interactions

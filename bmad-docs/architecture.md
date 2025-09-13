# ImKitchen Fullstack Architecture Document

## Introduction

This document outlines the complete fullstack architecture for ImKitchen, including backend systems, frontend implementation, and their integration. It serves as the single source of truth for AI-driven development, ensuring consistency across the entire technology stack.

This unified approach combines what would traditionally be separate backend and frontend architecture documents, streamlining the development process for modern fullstack applications where these concerns are increasingly intertwined.

ImKitchen is an AI-powered cooking companion that bridges the execution gap in home cooking through timing intelligence and seamless meal lifecycle management. The architecture prioritizes recipe parsing accuracy, timing coordination algorithms, offline functionality, and kitchen-optimized user interfaces.

### Starter Template or Existing Project

N/A - Greenfield project. However, we will leverage proven Rust ecosystem patterns with axum web framework, Askama templating, and PostgreSQL for rapid development while maintaining performance and type safety.

### Change Log

| Date | Version | Description | Author |
|------|---------|-------------|---------|
| 2025-09-13 | 1.0 | Initial fullstack architecture creation | Winston (Architect) |

## High Level Architecture

### Technical Summary

ImKitchen employs a monolithic Rust backend with axum 0.8+ web framework, PostgreSQL 17+ for structured data, and Redis 8.2+ for caching and session management. The frontend utilizes Askama 0.14+ templating with twinspark-js for reactive UI components, minimizing JavaScript complexity while maintaining rich interactivity. The architecture emphasizes offline-first capabilities through service workers, real-time timing coordination via WebSockets, and recipe parsing accuracy through natural language processing. The system is containerized with Docker for consistent deployment and designed to scale horizontally to support 10,000 daily active users while maintaining sub-2-second response times.

### Platform and Infrastructure Choice

**Platform:** Self-hosted with Docker containerization and cloud deployment flexibility

**Key Services:** 
- Application: Docker containers with Rust backend + Askama frontend
- Database: PostgreSQL 17+ with Redis 8.2+ caching layer  
- Storage: Local filesystem with optional cloud storage for recipe images
- Monitoring: Prometheus + Grafana for metrics, structured logging with tracing
- Load Balancing: Nginx reverse proxy with SSL termination

**Deployment Host and Regions:** Initially single-region deployment (US-East) with CDN for global asset distribution, designed for multi-region expansion

### Repository Structure

**Structure:** Monorepo with workspace-based organization supporting shared types and utilities

**Monorepo Tool:** Rust workspace with Cargo, npm for frontend asset tooling only

**Package Organization:** 
- `src/` - Unified Rust application with axum server, templates, and handlers
- `static/` - Static assets (CSS, JS, images) served by axum
- `templates/` - Askama templates embedded in the binary
- `shared/` - Shared types and utilities within the workspace
- `infrastructure/` - Docker and deployment configurations

### High Level Architecture Diagram

```mermaid
graph TD
    A[Users] --> B[CDN/Nginx]
    B --> C[ImKitchen Unified App]
    
    subgraph "Single Rust Binary"
        C --> D[Axum Web Server]
        C --> E[Askama Templates]
        C --> F[Static Asset Server]
        
        D --> G[Recipe Parser Service]
        D --> H[Meal Planner Service]  
        D --> I[Timing Engine Service]
        D --> J[Auth Service]
        D --> K[Template Handlers]
        K --> L[Fragment Handlers]
    end
    
    D --> L[PostgreSQL 17+]
    D --> M[Redis 8.2+]
    
    N[External Recipe Sites] --> G
    O[Web Push Service] --> I
    
    E --> P[Service Worker]
    P --> Q[Offline Storage]
    
    subgraph "Data Layer"  
        L
        M
        Q
    end
```

### Architectural Patterns

- **Monolithic Architecture:** Single deployable unit with clear internal service boundaries - _Rationale:_ Simplifies deployment and development for single developer while maintaining clear separation of concerns
- **Server-Side Rendering (SSR):** Askama templates rendered server-side with progressive enhancement - _Rationale:_ Optimal performance and SEO while supporting offline functionality through service workers
- **Repository Pattern:** Abstract data access with trait-based interfaces - _Rationale:_ Enables testing, database flexibility, and clean business logic separation
- **Service Layer Pattern:** Domain services for recipe parsing, meal planning, and timing calculations - _Rationale:_ Encapsulates complex business logic and enables unit testing
- **Command Query Responsibility Segregation (CQRS):** Separate read/write paths for complex operations - _Rationale:_ Optimizes performance for recipe queries vs meal plan modifications
- **Event-Driven Architecture:** Internal events for timing notifications and cooking state changes - _Rationale:_ Supports real-time features and future extensibility

## Tech Stack

### Technology Stack Table

| Category | Technology | Version | Purpose | Rationale |
|----------|------------|---------|---------|-----------|
| Frontend Language | Rust | 1.70+ | Template rendering, business logic | Type safety, performance, single language across stack |
| Frontend Framework | Askama | 0.14+ | Server-side templating | Compile-time template checking, minimal runtime overhead |
| UI Reactivity | twinspark-js | Latest | Client-side DOM reactivity | Lightweight alternative to heavy JS frameworks |
| State Management | HTML + Alpine.js | 3.x | Client-side state | Minimal JS footprint with reactive capabilities |
| Backend Language | Rust | 1.70+ | API server, business logic | Memory safety, performance, excellent ecosystem |
| Backend Framework | axum | 0.8+ | HTTP server and routing | Modern async framework with Tower ecosystem |
| API Style | REST | HTTP/1.1 | Client-server communication | Simple, well-understood, cacheable |
| Database | PostgreSQL | 17+ | Primary data storage | ACID compliance, JSON support, vector extensions |
| Cache | Redis | 8.2+ | Session storage, caching | High performance, pub/sub for real-time features |
| File Storage | Local + S3 Compatible | - | Recipe images, assets | Cost-effective with cloud migration path |
| Authentication | JWT + Sessions | - | User authentication | Stateless tokens with server-side session validation |
| Frontend Testing | wasm-pack-test | Latest | Rust WebAssembly testing | Test business logic in same language |
| Backend Testing | tokio-test | Latest | Async Rust testing | First-class async testing support |
| E2E Testing | Playwright | Latest | Browser automation | Reliable cross-browser testing |
| Build Tool | Cargo | Latest | Rust compilation | Built-in Rust toolchain |
| Bundler | Vite | 5.x | Asset bundling | Fast development builds, tree shaking |
| IaC Tool | Docker Compose | 2.x | Local development | Simple orchestration with production parity |
| CI/CD | GitHub Actions | - | Automated testing/deployment | Integrated with repository, free for open source |
| Monitoring | Prometheus | Latest | Metrics collection | Industry standard with rich ecosystem |
| Logging | tracing + OTEL | Latest | Structured logging | Rust-native observability with OpenTelemetry |
| CSS Framework | Tailwind CSS | 3.x | Utility-first styling | Rapid development, small bundle size |

## Data Models

### User

**Purpose:** Represents platform users with cooking preferences, dietary restrictions, and household configuration

**Key Attributes:**
- id: Uuid - Unique identifier
- email: String - Login credential and communication
- password_hash: String - Secure authentication storage
- dietary_preferences: Vec<DietaryRestriction> - Filtering and meal planning
- skill_level: SkillLevel - Recipe difficulty matching
- household_size: u32 - Recipe scaling and portions
- kitchen_equipment: Vec<Equipment> - Recipe feasibility checking
- created_at: DateTime - Account creation tracking
- updated_at: DateTime - Profile modification tracking

#### TypeScript Interface

```typescript
interface User {
  id: string;
  email: string;
  dietary_preferences: DietaryRestriction[];
  skill_level: 'beginner' | 'intermediate' | 'advanced';
  household_size: number;
  kitchen_equipment: Equipment[];
  created_at: string;
  updated_at: string;
}

enum DietaryRestriction {
  Vegetarian = 'vegetarian',
  Vegan = 'vegan', 
  GlutenFree = 'gluten_free',
  DairyFree = 'dairy_free',
  NutFree = 'nut_free'
}
```

#### Relationships
- One-to-many with Recipe (user's imported recipes)
- One-to-many with MealPlan (user's meal planning history)
- One-to-many with CookingSession (user's cooking activities)

### Recipe

**Purpose:** Core recipe data with ingredients, instructions, timing, and nutritional information

**Key Attributes:**
- id: Uuid - Unique identifier
- user_id: Uuid - Owner reference
- title: String - Recipe name
- description: Option<String> - Recipe summary
- ingredients: Vec<Ingredient> - Required components with quantities
- instructions: Vec<Instruction> - Step-by-step cooking guide
- prep_time: Duration - Preparation duration
- cook_time: Duration - Cooking duration
- total_time: Duration - Complete recipe duration
- servings: u32 - Default portion size
- difficulty: SkillLevel - Complexity rating
- cuisine_type: Option<String> - Culinary classification
- tags: Vec<String> - Searchable keywords
- nutritional_info: Option<Nutrition> - Health information
- source_url: Option<String> - Original recipe location
- created_at: DateTime - Import/creation time
- updated_at: DateTime - Last modification

#### TypeScript Interface

```typescript
interface Recipe {
  id: string;
  user_id: string;
  title: string;
  description?: string;
  ingredients: Ingredient[];
  instructions: Instruction[];
  prep_time: number; // minutes
  cook_time: number; // minutes  
  total_time: number; // minutes
  servings: number;
  difficulty: SkillLevel;
  cuisine_type?: string;
  tags: string[];
  nutritional_info?: Nutrition;
  source_url?: string;
  created_at: string;
  updated_at: string;
}

interface Ingredient {
  name: string;
  quantity: number;
  unit: string;
  notes?: string;
}

interface Instruction {
  step_number: number;
  description: string;
  duration?: number; // minutes
  temperature?: number; // celsius
}
```

#### Relationships
- Many-to-one with User (recipe owner)
- Many-to-many with MealPlan (planned meals)
- One-to-many with CookingSession (cooking instances)

### MealPlan

**Purpose:** Weekly meal planning with recipe assignments, shopping list generation, and schedule optimization

**Key Attributes:**
- id: Uuid - Unique identifier
- user_id: Uuid - Owner reference
- week_start: NaiveDate - Planning week beginning
- meals: HashMap<Weekday, Vec<PlannedMeal>> - Daily meal assignments
- shopping_list: Vec<ShoppingItem> - Aggregated ingredients
- generated_at: DateTime - AI generation timestamp
- confirmed_at: Option<DateTime> - User approval time
- completed_at: Option<DateTime> - Execution completion

#### TypeScript Interface

```typescript
interface MealPlan {
  id: string;
  user_id: string;
  week_start: string; // ISO date
  meals: Record<Weekday, PlannedMeal[]>;
  shopping_list: ShoppingItem[];
  generated_at: string;
  confirmed_at?: string;
  completed_at?: string;
}

interface PlannedMeal {
  meal_type: 'breakfast' | 'lunch' | 'dinner' | 'snack';
  recipe_id: string;
  scheduled_time?: string; // ISO datetime
  scaling_factor: number;
  notes?: string;
}

interface ShoppingItem {
  ingredient_name: string;
  quantity: number;
  unit: string;
  category: string; // grocery aisle
  purchased: boolean;
}
```

#### Relationships
- Many-to-one with User (meal plan owner)
- Many-to-many with Recipe (planned recipes)
- One-to-many with CookingSession (executed meals)

### CookingSession

**Purpose:** Active cooking tracking with timing coordination, progress monitoring, and feedback collection

**Key Attributes:**
- id: Uuid - Unique identifier  
- user_id: Uuid - Cook reference
- recipe_id: Uuid - Recipe being executed
- meal_plan_id: Option<Uuid> - Associated meal plan
- start_time: DateTime - Cooking commencement
- end_time: Option<DateTime> - Cooking completion
- current_step: u32 - Progress tracking
- timers: Vec<Timer> - Active cooking timers
- scaling_factor: f32 - Recipe portion adjustment
- notes: Vec<CookingNote> - User observations
- rating: Option<u8> - Recipe satisfaction (1-5)
- timing_accuracy: Option<i32> - Actual vs predicted time difference

#### TypeScript Interface

```typescript
interface CookingSession {
  id: string;
  user_id: string;
  recipe_id: string;
  meal_plan_id?: string;
  start_time: string;
  end_time?: string;
  current_step: number;
  timers: Timer[];
  scaling_factor: number;
  notes: CookingNote[];
  rating?: number; // 1-5
  timing_accuracy?: number; // minutes difference
}

interface Timer {
  id: string;
  name: string;
  duration: number; // minutes
  remaining: number; // minutes
  status: 'running' | 'paused' | 'completed';
}

interface CookingNote {
  timestamp: string;
  step_number: number;
  content: string;
  note_type: 'observation' | 'modification' | 'issue';
}
```

#### Relationships
- Many-to-one with User (cooking session owner)
- Many-to-one with Recipe (recipe being cooked)
- Many-to-one with MealPlan (associated meal plan)

## HTML Interface Specification

### Page Routes and Form Handling

ImKitchen uses server-side rendering with Askama templates. All interactions happen through standard HTML forms and twinspark-js fragment updates.

#### Authentication Forms

```html
<!-- Login Form -->
<form method="post" action="/auth/login">
  <input name="email" type="email" required>
  <input name="password" type="password" required>
  <button type="submit">Login</button>
</form>

<!-- Registration Form -->
<form method="post" action="/auth/register">
  <input name="email" type="email" required>
  <input name="password" type="password" required>
  <select name="dietary_preferences" multiple>
    <option value="vegetarian">Vegetarian</option>
    <option value="vegan">Vegan</option>
    <option value="gluten_free">Gluten Free</option>
  </select>
  <button type="submit">Register</button>
</form>
```

#### Recipe Management Forms

```html
<!-- Create Recipe Form -->
<form method="post" action="/recipes" 
      ts-req="/fragments/recipes"
      ts-trigger="submit"
      ts-target="#recipe-list">
  <input name="title" required>
  <textarea name="description"></textarea>
  <input name="prep_time" type="number" required>
  <input name="cook_time" type="number" required>
  <textarea name="ingredients" required></textarea>
  <textarea name="instructions" required></textarea>
  <button type="submit">Add Recipe</button>
</form>

<!-- Recipe Import Form -->
<form method="post" action="/recipes/import"
      ts-req="/fragments/recipe-import-status"
      ts-trigger="submit"
      ts-target="#import-status">
  <input name="url" type="url" placeholder="Recipe URL" required>
  <button type="submit">Import Recipe</button>
</form>
```

#### Cooking Session Forms

```html
<!-- Start Cooking Session -->
<form method="post" action="/cooking/sessions">
  <input name="recipe_id" type="hidden" value="{{ recipe.id }}">
  <input name="scaling_factor" type="number" step="0.1" value="1.0">
  <button type="submit">Start Cooking</button>
</form>

<!-- Add Timer -->
<form method="post" action="/cooking/sessions/{{ session.id }}/timers"
      ts-req="/fragments/timers"
      ts-trigger="submit"
      ts-target="#timer-widget">
  <input name="name" placeholder="Timer name" required>
  <input name="duration" type="number" placeholder="Minutes" required>
  <button type="submit">Add Timer</button>
</form>
```

#### Fragment Endpoints

Fragment endpoints return HTML snippets for twinspark-js to update page sections:

| Endpoint | Purpose | Returns |
|----------|---------|---------|
| `/fragments/recipes` | Recipe list updates | `<div id="recipe-list">...</div>` |
| `/fragments/recipe/{id}` | Single recipe card | `<div class="recipe-card">...</div>` |
| `/fragments/timers` | Timer widget updates | `<div id="timer-widget">...</div>` |
| `/fragments/meal-plan` | Meal plan updates | `<div id="meal-plan">...</div>` |
| `/fragments/notifications` | Notification updates | `<div id="notifications">...</div>` |

#### Response Patterns

**Successful Form Submission:**
- Full page forms: Redirect to success page (HTTP 302)
- Fragment forms: Return updated HTML fragment (HTTP 200)

**Form Validation Errors:**
- Full page forms: Re-render form page with error messages
- Fragment forms: Return form fragment with error highlights

**Authentication Required:**
- Redirect to login page (HTTP 302) with return URL parameter

## Components

### Recipe Parser Service

**Responsibility:** Extract and normalize recipe data from external URLs and user inputs with high accuracy

**Key Interfaces:**
- `parse_recipe_url(url: String) -> Result<Recipe, ParseError>`
- `extract_ingredients(text: String) -> Vec<Ingredient>`
- `parse_cooking_instructions(text: String) -> Vec<Instruction>`

**Dependencies:** HTTP client for fetching, HTML parser, NLP library for text processing

**Technology Stack:** Rust with reqwest HTTP client, scraper for HTML parsing, regex for pattern matching, potential integration with recipe microdata standards (JSON-LD)

### Meal Planning Engine

**Responsibility:** Generate optimized weekly meal plans based on user preferences, dietary constraints, and ingredient availability

**Key Interfaces:**
- `generate_meal_plan(user_id: Uuid, preferences: PlanPreferences) -> MealPlan`
- `optimize_ingredient_usage(recipes: Vec<Recipe>) -> ShoppingList`
- `calculate_nutritional_balance(meal_plan: &MealPlan) -> NutritionalSummary`

**Dependencies:** Recipe repository, user preferences, nutritional database

**Technology Stack:** Rust with constraint satisfaction algorithms, PostgreSQL for recipe queries, Redis for caching frequently accessed combinations

### Timing Coordination Engine

**Responsibility:** Calculate optimal cooking schedules and provide real-time timing guidance for single and multi-dish cooking

**Key Interfaces:**
- `calculate_cooking_schedule(recipes: Vec<Recipe>, target_time: DateTime) -> CookingSchedule`
- `start_cooking_session(recipe_id: Uuid, scaling: f32) -> CookingSession`
- `update_timer_status(session_id: Uuid, timer_id: String, status: TimerStatus) -> Result<(), TimerError>`

**Dependencies:** Recipe timing data, user feedback history, notification service

**Technology Stack:** Rust with async timers using tokio, WebSocket connections for real-time updates, Redis pub/sub for timer events

### Authentication Service

**Responsibility:** Manage user registration, login, session management, and access control

**Key Interfaces:**
- `register_user(email: String, password: String) -> Result<User, AuthError>`
- `authenticate_user(email: String, password: String) -> Result<AuthToken, AuthError>`
- `validate_token(token: String) -> Result<UserClaims, AuthError>`

**Dependencies:** User repository, password hashing, JWT library

**Technology Stack:** Rust with argon2 password hashing, jsonwebtoken for JWT handling, Redis for session storage

### Notification Service

**Responsibility:** Deliver timing alerts, cooking reminders, and system notifications across devices

**Key Interfaces:**
- `send_timer_alert(user_id: Uuid, message: String, urgency: Priority)`
- `schedule_cooking_reminder(user_id: Uuid, recipe_id: Uuid, start_time: DateTime)`
- `register_push_subscription(user_id: Uuid, subscription: PushSubscription)`

**Dependencies:** Web Push API, timing engine events, user preferences

**Technology Stack:** Rust with web-push crate for push notifications, WebSocket connections for real-time alerts

### Component Diagrams

```mermaid
graph TD
    subgraph "Unified Application Layer"
        A[axum Router]
        B[Template Handlers]
        C[API Handlers]
        D[Static File Server]
        E[Askama Templates]
        F[Authentication Middleware]
    end
    
    subgraph "Service Layer"
        G[Recipe Parser]
        H[Meal Planner] 
        I[Timing Engine]
        J[Auth Service]
        K[Notification Service]
    end
    
    subgraph "Data Layer"
        L[Recipe Repository]
        M[User Repository]
        N[Session Store]
        O[Cache Layer]
    end
    
    subgraph "Infrastructure"
        P[PostgreSQL]
        Q[Redis]
        R[Web Push API]
        S[twinspark-js Client]
        T[Service Worker]
    end
    
    A --> B
    A --> C
    A --> D
    B --> E
    B --> F
    C --> F
    
    B --> G
    B --> H
    B --> I
    C --> G
    C --> H
    C --> I
    C --> J
    
    G --> L
    H --> L
    H --> M
    I --> N
    J --> M
    K --> R
    
    L --> P
    M --> P
    N --> Q
    O --> Q
    
    E --> S
    E --> T
```

## External APIs

### Web Push API

- **Purpose:** Deliver real-time cooking notifications and timer alerts to user devices
- **Documentation:** https://developer.mozilla.org/en-US/docs/Web/API/Push_API
- **Base URL(s):** FCM: https://fcm.googleapis.com/fcm/send
- **Authentication:** Voluntary Application Server Identification (VAPID) keys
- **Rate Limits:** 1000 notifications per app per user per hour

**Key Endpoints Used:**
- `POST /fcm/send` - Send push notification to specific user subscription

**Integration Notes:** Requires service worker registration on frontend, VAPID key generation for authentication, graceful fallback to WebSocket notifications if push not available

### Recipe Website APIs

- **Purpose:** Import recipe data from popular cooking websites with structured data
- **Documentation:** Schema.org Recipe microdata specification
- **Base URL(s):** Various recipe sites (AllRecipes, Food Network, etc.)
- **Authentication:** None required for public recipes
- **Rate Limits:** Respect robots.txt and implement polite crawling delays

**Key Endpoints Used:**
- Various recipe URLs with JSON-LD or microdata markup

**Integration Notes:** Implement HTML parsing fallback for sites without structured data, respect copyright and fair use policies, cache parsed results to minimize requests

## Core Workflows

### Recipe Import and Processing Workflow

```mermaid
sequenceDiagram
    participant User
    participant Frontend
    participant API
    participant Parser
    participant Database
    participant External

    User->>Frontend: Paste recipe URL
    Frontend->>API: POST /recipes (URL import)
    API->>Parser: parse_recipe_url()
    Parser->>External: Fetch recipe page
    External-->>Parser: HTML content
    
    alt Structured data found
        Parser->>Parser: Extract JSON-LD/microdata
    else Fallback parsing
        Parser->>Parser: HTML content analysis
    end
    
    Parser-->>API: Parsed recipe data
    API->>Database: Store recipe
    Database-->>API: Recipe ID
    API-->>Frontend: Recipe created
    Frontend-->>User: Show recipe details
    
    note over Parser: 90% accuracy target
    note over API: Handle parsing failures gracefully
```

### Meal Planning Generation Workflow

```mermaid
sequenceDiagram
    participant User
    participant Frontend
    participant API
    participant Planner
    participant Database
    participant Cache

    User->>Frontend: Request meal plan
    Frontend->>API: POST /meal-plans
    API->>Database: Get user preferences
    Database-->>API: Preferences data
    
    API->>Planner: generate_meal_plan()
    Planner->>Database: Query suitable recipes
    Database-->>Planner: Recipe candidates
    
    Planner->>Planner: Apply constraints and optimization
    Planner->>Planner: Generate shopping list
    Planner-->>API: Complete meal plan
    
    API->>Database: Store meal plan
    API->>Cache: Cache plan for quick access
    API-->>Frontend: Meal plan with shopping list
    Frontend-->>User: Show plan with edit options
    
    note over Planner: Consider dietary restrictions, time constraints, ingredient optimization
```

### Cook Mode Timing Coordination Workflow

```mermaid
sequenceDiagram
    participant User
    participant Frontend
    participant API
    participant TimingEngine
    participant Notifications
    participant Cache

    User->>Frontend: Start cooking session
    Frontend->>API: POST /cooking/sessions
    API->>TimingEngine: create_cooking_session()
    TimingEngine->>TimingEngine: Calculate step timings
    TimingEngine->>Cache: Store session state
    TimingEngine-->>API: Session with schedule
    API-->>Frontend: Cooking session started
    
    loop Active cooking
        TimingEngine->>Notifications: Timer alerts
        Notifications->>Frontend: Push notification
        Frontend->>User: Timer alert display
        
        User->>Frontend: Mark step complete
        Frontend->>API: PUT /cooking/sessions/{id}/step
        API->>TimingEngine: update_progress()
        TimingEngine->>TimingEngine: Recalculate remaining times
        TimingEngine->>Cache: Update session state
    end
    
    User->>Frontend: Complete cooking
    Frontend->>API: PUT /cooking/sessions/{id}/complete
    API->>TimingEngine: finalize_session()
    TimingEngine->>Database: Store timing feedback
    
    note over TimingEngine: ±10 minute accuracy target
    note over Notifications: Real-time alerts critical for success
```

## Database Schema

```sql
-- Users table with cooking preferences
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    dietary_preferences TEXT[], -- Array of dietary restrictions
    skill_level VARCHAR(20) CHECK (skill_level IN ('beginner', 'intermediate', 'advanced')),
    household_size INTEGER DEFAULT 1,
    kitchen_equipment JSONB DEFAULT '[]', -- Array of equipment objects
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Recipes table with detailed recipe information
CREATE TABLE recipes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title VARCHAR(500) NOT NULL,
    description TEXT,
    ingredients JSONB NOT NULL, -- Array of ingredient objects
    instructions JSONB NOT NULL, -- Array of instruction objects  
    prep_time INTEGER NOT NULL, -- minutes
    cook_time INTEGER NOT NULL, -- minutes
    total_time INTEGER GENERATED ALWAYS AS (prep_time + cook_time) STORED,
    servings INTEGER DEFAULT 1,
    difficulty VARCHAR(20) CHECK (difficulty IN ('beginner', 'intermediate', 'advanced')),
    cuisine_type VARCHAR(100),
    tags TEXT[] DEFAULT '{}',
    nutritional_info JSONB,
    source_url TEXT,
    image_path VARCHAR(500),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Meal plans for weekly planning
CREATE TABLE meal_plans (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    week_start DATE NOT NULL,
    meals JSONB NOT NULL, -- Weekly meal schedule object
    shopping_list JSONB NOT NULL, -- Array of shopping items
    generated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    confirmed_at TIMESTAMP WITH TIME ZONE,
    completed_at TIMESTAMP WITH TIME ZONE,
    
    UNIQUE(user_id, week_start)
);

-- Active cooking sessions with real-time state
CREATE TABLE cooking_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    recipe_id UUID NOT NULL REFERENCES recipes(id) ON DELETE CASCADE,
    meal_plan_id UUID REFERENCES meal_plans(id) ON DELETE SET NULL,
    start_time TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    end_time TIMESTAMP WITH TIME ZONE,
    current_step INTEGER DEFAULT 0,
    timers JSONB DEFAULT '[]', -- Array of active timer objects
    scaling_factor DECIMAL(3,2) DEFAULT 1.0,
    notes JSONB DEFAULT '[]', -- Array of cooking note objects
    rating INTEGER CHECK (rating >= 1 AND rating <= 5),
    timing_accuracy INTEGER, -- Difference in minutes from predicted time
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Push notification subscriptions
CREATE TABLE push_subscriptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    endpoint TEXT NOT NULL,
    p256dh_key TEXT NOT NULL,
    auth_key TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    
    UNIQUE(user_id, endpoint)
);

-- Indexes for query optimization
CREATE INDEX idx_recipes_user_id ON recipes(user_id);
CREATE INDEX idx_recipes_difficulty ON recipes(difficulty);
CREATE INDEX idx_recipes_total_time ON recipes(total_time);
CREATE INDEX idx_recipes_cuisine ON recipes(cuisine_type);
CREATE INDEX idx_recipes_tags ON recipes USING GIN(tags);
CREATE INDEX idx_recipes_search ON recipes USING GIN(to_tsvector('english', title || ' ' || description));

CREATE INDEX idx_meal_plans_user_week ON meal_plans(user_id, week_start);
CREATE INDEX idx_cooking_sessions_user ON cooking_sessions(user_id);
CREATE INDEX idx_cooking_sessions_active ON cooking_sessions(user_id) WHERE end_time IS NULL;

-- Full-text search for recipes
ALTER TABLE recipes ADD COLUMN search_vector tsvector GENERATED ALWAYS AS (
    to_tsvector('english', title || ' ' || COALESCE(description, ''))
) STORED;
CREATE INDEX idx_recipes_fts ON recipes USING GIN(search_vector);
```

## Unified Application Architecture

### Template and Handler Organization

#### Project Structure

```text
src/
├── main.rs                   # Application entry point and server setup
├── routes/                   # Template and fragment route handlers
│   ├── mod.rs
│   ├── pages/                # Full page handlers
│   │   ├── dashboard.rs      # Dashboard page
│   │   ├── recipes.rs        # Recipe management pages
│   │   ├── planner.rs        # Meal planning interface
│   │   ├── cooking.rs        # Cook mode pages
│   │   └── auth.rs           # Authentication pages
│   └── fragments/            # HTML fragment handlers (for twinspark-js)
│       ├── recipe_card.rs    # Recipe card component updates
│       ├── timer_widget.rs   # Timer widget updates
│       ├── meal_list.rs      # Meal list updates
│       └── notifications.rs  # Notification fragments
├── templates/                # Askama templates (embedded in binary)
│   ├── layouts/              # Base layouts (main.html, auth.html)
│   ├── pages/                # Full page templates
│   │   ├── dashboard.html    # Main dashboard
│   │   ├── recipes/          # Recipe management pages
│   │   ├── planner/          # Meal planning interface
│   │   └── cook/             # Cooking mode templates
│   ├── components/           # Reusable template fragments
│   │   ├── recipe_card.html
│   │   ├── timer_widget.html
│   │   └── meal_planner_grid.html
│   └── fragments/            # Small fragments for twinspark-js updates
│       ├── recipe_item.html
│       ├── timer_display.html
│       └── notification.html
├── services/                 # Business logic services
├── repositories/             # Data access layer
├── models/                   # Domain models
├── middleware/               # HTTP middleware
└── config/                   # Configuration
static/                       # Static assets served by axum
├── css/                      # Tailwind compiled CSS
├── js/                       # twinspark-js for DOM updates
├── images/                   # UI images and icons
└── sw.js                     # Service worker
```

#### Template Handler Pattern

```rust
use askama::Template;
use axum::{response::Html, Extension, extract::Query};
use crate::{models::Recipe, auth::UserClaims, services::RecipeService};

#[derive(Template)]
#[template(path = "pages/recipes/index.html")]
struct RecipesPage {
    user: UserClaims,
    recipes: Vec<Recipe>,
    current_page: u32,
    total_pages: u32,
}

#[derive(Template)]
#[template(path = "fragments/recipe_list.html")]
struct RecipeListFragment {
    recipes: Vec<Recipe>,
}

// Full page handler
pub async fn recipes_page(
    Extension(user): Extension<UserClaims>,
    Extension(recipe_service): Extension<RecipeService>,
) -> Result<Html<String>, AppError> {
    let recipes = recipe_service.get_user_recipes(user.sub).await?;
    
    let template = RecipesPage {
        user,
        recipes,
        current_page: 1,
        total_pages: 1,
    };
    
    Ok(Html(template.render()?))
}

// Fragment handler for twinspark-js updates
pub async fn recipes_list_fragment(
    Extension(user): Extension<UserClaims>,
    Extension(recipe_service): Extension<RecipeService>,
    Query(params): Query<ListParams>,
) -> Result<Html<String>, AppError> {
    let recipes = recipe_service.get_user_recipes_filtered(user.sub, params).await?;
    
    let template = RecipeListFragment { recipes };
    
    Ok(Html(template.render()?))
}
```

### State Management Architecture

#### State Structure

```typescript
// Frontend state managed through Alpine.js stores
interface AppState {
  user: {
    id: string;
    preferences: UserPreferences;
    settings: UserSettings;
  };
  cooking: {
    activeSessions: CookingSession[];
    timers: Timer[];
    currentRecipe?: Recipe;
  };
  planner: {
    currentWeek: MealPlan;
    isGenerating: boolean;
    isDirty: boolean;
  };
  ui: {
    notifications: Notification[];
    modals: ModalState;
    connectivity: 'online' | 'offline';
  };
}
```

#### State Management Patterns

- **Server-Side Rendering:** All state managed server-side, DOM updated via HTML fragments
- **Fragment Updates:** twinspark-js requests return HTML fragments that replace DOM sections
- **Form Submissions:** Standard form posts return updated page fragments or full page redirects
- **Real-time Updates:** WebSocket events trigger fragment requests to refresh specific components

### Unified Routing Architecture

#### Route Organization

```text
Unified Routes (single axum application):
# Page routes (return full HTML pages)
/                           # Dashboard page
/auth/login                 # Login page  
/auth/register              # Registration page
/recipes                    # Recipe library page
/recipes/new                # Add/import recipe page
/recipes/{id}               # Recipe detail page
/recipes/{id}/cook          # Cook mode page
/planner                    # Meal planning page
/shopping                   # Shopping list page

# Fragment routes (return HTML fragments for twinspark-js)
/fragments/recipes          # Recipe list fragment
/fragments/recipe/{id}      # Recipe card fragment
/fragments/timers           # Timer widgets fragment
/fragments/meal-plan        # Meal plan fragment
/fragments/notifications    # Notification fragment

# Form submission routes (POST/PUT/DELETE)
/recipes                    # Create recipe (POST)
/recipes/{id}               # Update/delete recipe
/cooking/sessions           # Start cooking session
/meal-plans                 # Generate meal plan

# Static assets
/static/*                   # CSS, JS, images served by axum
```

#### Unified Route Setup Pattern

```rust
use axum::{middleware, Router, routing::{get, post}};
use tower::ServiceBuilder;
use tower_http::services::ServeDir;

pub fn create_app() -> Router {
    Router::new()
        // Public routes
        .route("/", get(pages::dashboard::dashboard_page))
        .route("/auth/login", get(pages::auth::login_page).post(pages::auth::login_submit))
        .route("/auth/register", get(pages::auth::register_page).post(pages::auth::register_submit))
        
        // Protected page routes
        .route("/recipes", get(pages::recipes::recipes_page).post(pages::recipes::create_recipe))
        .route("/recipes/new", get(pages::recipes::new_recipe_page))
        .route("/recipes/:id", get(pages::recipes::recipe_detail_page))
        .route("/planner", get(pages::planner::meal_planner_page))
        .route("/cooking/:id", get(pages::cooking::cooking_session_page))
        
        // Fragment routes for twinspark-js
        .route("/fragments/recipes", get(fragments::recipe_list::recipe_list_fragment))
        .route("/fragments/recipe/:id", get(fragments::recipe_card::recipe_card_fragment))
        .route("/fragments/timers", get(fragments::timer_widget::timers_fragment))
        .route("/fragments/meal-plan", get(fragments::meal_list::meal_plan_fragment))
        .route("/fragments/notifications", get(fragments::notifications::notifications_fragment))
        
        // Form submission routes
        .route("/cooking/sessions", post(pages::cooking::start_cooking_session))
        .route("/meal-plans", post(pages::planner::generate_meal_plan))
        
        // Apply authentication to protected routes
        .layer(ServiceBuilder::new()
            .layer(middleware::from_fn(auth::require_authentication))
            .layer(middleware::from_fn(session::load_user_context))
        )
        
        // Static file serving
        .nest_service("/static", ServeDir::new("static"))
}
```

### Client-Side Interactivity

#### twinspark-js Integration

twinspark-js handles DOM updates by making requests to fragment endpoints and updating specific page sections. No custom JavaScript API client needed.

```html
<!-- Example: Recipe card with interactive rating -->
<div class="recipe-card" 
     ts-req="/fragments/recipe/{{ recipe.id }}"
     ts-trigger="click"
     ts-target="#recipe-{{ recipe.id }}">
  <h3>{{ recipe.title }}</h3>
  <p>Prep: {{ recipe.prep_time }} min</p>
  <button class="favorite-btn">♡ Favorite</button>
</div>

<!-- Example: Timer widget that updates automatically -->
<div id="timer-widget" 
     ts-req="/fragments/timers"
     ts-trigger="every 1s"
     ts-target="#timer-widget">
  {% for timer in timers %}
    <div class="timer">{{ timer.name }}: {{ timer.remaining }}s</div>
  {% endfor %}
</div>

<!-- Example: Form submission with fragment update -->
<form ts-req="/recipes" 
      ts-trigger="submit"
      ts-target="#recipe-list">
  <input name="title" placeholder="Recipe name">
  <textarea name="instructions" placeholder="Instructions"></textarea>
  <button type="submit">Add Recipe</button>
</form>
```

#### Alpine.js for Local State

```javascript
// Simple Alpine.js store for local UI state
document.addEventListener('alpine:init', () => {
  Alpine.store('ui', {
    notifications: [],
    isOffline: false,
    activeTimers: [],
    
    addNotification(message, type = 'info') {
      this.notifications.push({ message, type, id: Date.now() });
    },
    
    removeNotification(id) {
      this.notifications = this.notifications.filter(n => n.id !== id);
    },
    
    updateConnectivity(online) {
      this.isOffline = !online;
    }
  });
});
```

#### WebSocket Integration

```javascript
// WebSocket connection for real-time updates
class TimingUpdates {
  constructor() {
    this.ws = null;
    this.connect();
  }
  
  connect() {
    if (window.location.pathname.includes('/cooking/')) {
      const sessionId = window.location.pathname.split('/').pop();
      this.ws = new WebSocket(`wss://${window.location.host}/ws/cooking/${sessionId}`);
      
      this.ws.onmessage = (event) => {
        const update = JSON.parse(event.data);
        
        // Trigger fragment refresh for timer updates
        if (update.type === 'timer_update') {
          document.querySelector('#timer-widget').dispatchEvent(
            new CustomEvent('ts:trigger', { detail: { force: true } })
          );
        }
      };
    }
  }
}
```

## Backend Architecture

### Service Architecture

#### Controller/Route Organization

```text
apps/backend/src/
├── main.rs                   # Application entry point
├── config/                   # Configuration management
│   ├── mod.rs
│   ├── database.rs
│   └── settings.rs
├── routes/                   # HTTP route handlers
│   ├── mod.rs
│   ├── auth.rs               # Authentication endpoints
│   ├── recipes.rs            # Recipe CRUD operations
│   ├── meal_plans.rs         # Meal planning endpoints
│   └── cooking.rs            # Cooking session management
├── services/                 # Business logic layer
│   ├── mod.rs
│   ├── recipe_parser.rs      # Recipe import/parsing
│   ├── meal_planner.rs       # AI meal planning
│   ├── timing_engine.rs      # Cooking coordination
│   └── notification.rs       # Push notifications
├── repositories/             # Data access layer
│   ├── mod.rs
│   ├── user_repo.rs
│   ├── recipe_repo.rs
│   └── session_repo.rs
├── models/                   # Domain models
│   ├── mod.rs
│   ├── user.rs
│   ├── recipe.rs
│   └── cooking.rs
└── middleware/               # HTTP middleware
    ├── auth.rs
    ├── cors.rs
    └── logging.rs
```

#### Controller Template

```rust
use axum::{extract::Path, response::Json, Extension};
use uuid::Uuid;
use crate::{services::CookingService, models::CookingSession, auth::UserClaims};

pub async fn start_cooking_session(
    Extension(cooking_service): Extension<CookingService>,
    Extension(user): Extension<UserClaims>,
    Json(request): Json<StartCookingRequest>,
) -> Result<Json<CookingSession>, AppError> {
    let session = cooking_service
        .start_session(user.sub, request.recipe_id, request.scaling_factor)
        .await?;
    
    Ok(Json(session))
}

pub async fn get_cooking_session(
    Extension(cooking_service): Extension<CookingService>,
    Extension(user): Extension<UserClaims>,
    Path(session_id): Path<Uuid>,
) -> Result<Json<CookingSession>, AppError> {
    let session = cooking_service
        .get_session(session_id, user.sub)
        .await?;
    
    Ok(Json(session))
}
```

### Database Architecture

#### Schema Design

The PostgreSQL schema (defined above) uses modern PostgreSQL features:
- UUID primary keys for security and distribution
- JSONB for flexible nested data (ingredients, instructions, timers)
- Generated columns for computed values (total_time)
- GIN indexes for full-text search and array operations
- Partial indexes for query optimization (active sessions only)

#### Data Access Layer

```rust
use sqlx::{PgPool, Row};
use uuid::Uuid;
use crate::models::{Recipe, User};

pub struct RecipeRepository {
    pool: PgPool,
}

impl RecipeRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_user_id(&self, user_id: Uuid) -> Result<Vec<Recipe>, sqlx::Error> {
        let recipes = sqlx::query_as!(
            Recipe,
            r#"
            SELECT id, user_id, title, description, ingredients, instructions,
                   prep_time, cook_time, total_time, servings, difficulty as "difficulty: _",
                   cuisine_type, tags, nutritional_info, source_url, image_path,
                   created_at, updated_at
            FROM recipes 
            WHERE user_id = $1 
            ORDER BY created_at DESC
            "#,
            user_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(recipes)
    }

    pub async fn search(&self, user_id: Uuid, query: &str) -> Result<Vec<Recipe>, sqlx::Error> {
        let recipes = sqlx::query_as!(
            Recipe,
            r#"
            SELECT id, user_id, title, description, ingredients, instructions,
                   prep_time, cook_time, total_time, servings, difficulty as "difficulty: _",
                   cuisine_type, tags, nutritional_info, source_url, image_path,
                   created_at, updated_at
            FROM recipes 
            WHERE user_id = $1 AND search_vector @@ plainto_tsquery('english', $2)
            ORDER BY ts_rank(search_vector, plainto_tsquery('english', $2)) DESC
            "#,
            user_id,
            query
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(recipes)
    }
}
```

### Authentication and Authorization

#### Auth Flow

```mermaid
sequenceDiagram
    participant User
    participant Frontend
    participant API
    participant AuthService
    participant Database
    participant Redis

    User->>Frontend: Login credentials
    Frontend->>API: POST /auth/login
    API->>AuthService: authenticate_user()
    AuthService->>Database: Verify credentials
    Database-->>AuthService: User record
    AuthService->>AuthService: Generate JWT + session
    AuthService->>Redis: Store session data
    AuthService-->>API: Auth tokens
    API-->>Frontend: JWT + refresh token
    Frontend->>Frontend: Store tokens securely
    Frontend-->>User: Redirect to dashboard

    note over AuthService: JWT expires in 1 hour
    note over Redis: Session expires in 30 days
```

#### Middleware/Guards

```rust
use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
    http::StatusCode,
};
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};

pub async fn require_authentication(
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = request
        .headers()
        .get("authorization")
        .and_then(|header| header.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if !auth_header.starts_with("Bearer ") {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let token = &auth_header[7..];
    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    
    let token_data = decode::<UserClaims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::new(Algorithm::HS256),
    ).map_err(|_| StatusCode::UNAUTHORIZED)?;

    request.extensions_mut().insert(token_data.claims);
    Ok(next.run(request).await)
}
```

## Unified Project Structure

```plaintext
imkitchen/
├── .github/                        # CI/CD workflows
│   └── workflows/
│       ├── ci.yml                  # Test and lint
│       └── deploy.yml              # Build and deploy
├── src/                            # Unified Rust application
│   ├── main.rs                     # Application entry point and server setup
│   ├── routes/                     # Route handlers
│   │   ├── mod.rs
│   │   ├── pages/                  # Full page handlers
│   │   │   ├── mod.rs
│   │   │   ├── dashboard.rs        # Dashboard page
│   │   │   ├── recipes.rs          # Recipe management pages
│   │   │   ├── planner.rs          # Meal planning interface
│   │   │   ├── cooking.rs          # Cook mode pages
│   │   │   └── auth.rs             # Authentication pages
│   │   └── fragments/              # HTML fragment handlers
│   │       ├── mod.rs
│   │       ├── recipe_card.rs      # Recipe card fragments
│   │       ├── timer_widget.rs     # Timer widget fragments
│   │       ├── meal_list.rs        # Meal list fragments
│   │       └── notifications.rs    # Notification fragments
│   ├── services/                   # Business logic services
│   │   ├── mod.rs
│   │   ├── recipe_parser.rs        # Recipe import/parsing
│   │   ├── meal_planner.rs         # AI meal planning
│   │   ├── timing_engine.rs        # Cooking coordination
│   │   └── notification.rs         # Push notifications
│   ├── repositories/               # Data access layer
│   │   ├── mod.rs
│   │   ├── user_repo.rs
│   │   ├── recipe_repo.rs
│   │   └── session_repo.rs
│   ├── models/                     # Domain models
│   │   ├── mod.rs
│   │   ├── user.rs
│   │   ├── recipe.rs
│   │   └── cooking.rs
│   ├── middleware/                 # HTTP middleware
│   │   ├── mod.rs
│   │   ├── auth.rs
│   │   ├── cors.rs
│   │   └── logging.rs
│   └── config/                     # Configuration management
│       ├── mod.rs
│       ├── database.rs
│       └── settings.rs
├── templates/                      # Askama templates (embedded in binary)
│   ├── layouts/                    # Base layouts
│   │   ├── main.html              # Main application layout
│   │   └── auth.html              # Authentication layout
│   ├── pages/                      # Full page templates
│   │   ├── dashboard.html         # Main dashboard
│   │   ├── recipes/               # Recipe management pages
│   │   │   ├── index.html         # Recipe library
│   │   │   ├── detail.html        # Recipe detail view
│   │   │   └── new.html           # Add/import recipe
│   │   ├── planner/               # Meal planning interface
│   │   │   └── index.html
│   │   ├── cook/                  # Cooking mode templates
│   │   │   ├── session.html       # Active cooking session
│   │   │   └── dashboard.html     # Cook mode dashboard
│   │   └── auth/                  # Authentication pages
│   │       ├── login.html
│   │       └── register.html
│   ├── components/                 # Reusable template fragments
│   │   ├── recipe_card.html
│   │   ├── timer_widget.html
│   │   ├── meal_planner_grid.html
│   │   └── navigation.html
│   └── partials/                   # Small reusable fragments
│       ├── form_input.html
│       └── notification.html
├── static/                         # Static assets served by axum
│   ├── css/                        # Compiled Tailwind CSS
│   │   └── main.css
│   ├── js/                         # Client-side JavaScript
│   │   ├── twinspark.js           # Reactivity library
│   │   ├── alpine.min.js          # State management
│   │   └── app.js                 # Application-specific JS
│   ├── images/                     # UI images and icons
│   │   ├── icons/
│   │   └── logo.svg
│   └── sw.js                       # Service worker for offline support
├── migrations/                     # Database migrations
│   ├── 001_initial_schema.sql
│   ├── 002_add_cooking_sessions.sql
│   └── 003_add_push_subscriptions.sql
├── tests/                          # All tests
│   ├── integration/                # API integration tests
│   │   ├── auth_tests.rs
│   │   ├── recipe_tests.rs
│   │   └── cooking_tests.rs
│   ├── unit/                       # Unit tests
│   │   ├── services/
│   │   ├── repositories/
│   │   └── utils/
│   └── e2e/                        # End-to-end tests
│       ├── auth_flow.spec.js
│       ├── recipe_management.spec.js
│       └── cooking_workflow.spec.js
├── infrastructure/                 # Infrastructure as Code
│   ├── docker-compose.yml         # Local development stack
│   ├── docker-compose.prod.yml    # Production configuration
│   ├── Dockerfile                 # Application container
│   ├── nginx/                     # Reverse proxy configuration
│   └── scripts/                   # Deployment and utility scripts
├── docs/                          # Documentation
│   ├── prd.md                     # Product Requirements Document
│   ├── front-end-spec.md          # UI/UX Specification
│   ├── architecture.md            # This document
│   └── api/                       # API documentation
├── .env.example                   # Environment variable template
├── Cargo.toml                     # Rust workspace configuration
├── package.json                   # Node.js for CSS/JS tooling only
├── tailwind.config.js            # Tailwind CSS configuration
└── README.md                      # Project documentation
```

## Development Workflow

### Local Development Setup

#### Prerequisites

```bash
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Node.js for frontend tooling
# Using nvm (recommended)
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
nvm install 20
nvm use 20

# Install PostgreSQL and Redis
# On macOS
brew install postgresql@17 redis

# On Ubuntu
sudo apt install postgresql-17 redis-server
```

#### Initial Setup

```bash
# Clone repository
git clone <repository-url>
cd imkitchen

# Install Rust dependencies
cargo build

# Install frontend tooling dependencies (CSS/JS processing only)
npm install

# Set up environment variables
cp .env.example .env
# Edit .env with your database credentials and settings

# Set up database
createdb imkitchen_dev
sqlx database create
sqlx migrate run

# Generate initial data (optional)
cargo run --bin seed
```

#### Development Commands

```bash
# Start all services (unified application)
docker-compose up -d postgres redis
cargo run

# Build CSS during development (in separate terminal)
npm run css:watch

# Run tests
cargo test                    # All Rust tests (unit + integration)
npm run test:e2e             # End-to-end browser tests

# Build production assets
npm run build                 # Compile CSS and optimize JS
cargo build --release        # Build optimized binary
```

### Environment Configuration

#### Required Environment Variables

```bash
# Application (.env)
DATABASE_URL=postgresql://username:password@localhost/imkitchen_dev
REDIS_URL=redis://localhost:6379
JWT_SECRET=<your-jwt-secret-key>
VAPID_PRIVATE_KEY=<your-vapid-private-key>
VAPID_PUBLIC_KEY=<your-vapid-public-key>
SERVER_HOST=0.0.0.0
SERVER_PORT=3000
RUST_LOG=info
ENVIRONMENT=development
```

## Deployment Architecture

### Deployment Strategy

**Unified Application Deployment:**
- **Platform:** Single Docker container with embedded templates and static asset serving
- **Build Command:** `npm run build && cargo build --release`
- **Binary:** Single executable with embedded Askama templates
- **Static Assets:** Served directly by axum with aggressive caching headers
- **Deployment Method:** Docker image with health checks and graceful shutdown

### CI/CD Pipeline

```yaml
name: CI/CD Pipeline

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  test:
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:17
        env:
          POSTGRES_PASSWORD: postgres
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
      redis:
        image: redis:8.2
        options: >-
          --health-cmd "redis-cli ping"
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5

    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        
      - name: Cache dependencies
        uses: actions/cache@v3
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target/
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
          
      - name: Run tests
        run: |
          cargo test --workspace
          
      - name: Run frontend tests
        run: |
          npm ci
          npm run test
          
      - name: Run E2E tests
        run: |
          npm run test:e2e

  deploy:
    needs: test
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/main'
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Build Docker image
        run: |
          docker build -t imkitchen:latest .
          
      - name: Deploy to production
        run: |
          # Deploy Docker image to production environment
          # Implementation depends on chosen hosting platform
```

### Environments

| Environment | Application URL | Purpose |
|-------------|----------------|---------|
| Development | http://localhost:3000 | Local development |
| Staging | https://staging.imkitchen.app | Pre-production testing |
| Production | https://imkitchen.app | Live environment |

## Security and Performance

### Security Requirements

**Frontend Security:**
- CSP Headers: `default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline';`
- XSS Prevention: Askama template escaping by default, input sanitization on all user content
- Secure Storage: JWT tokens in httpOnly cookies, sensitive data never in localStorage

**Backend Security:**
- Input Validation: Comprehensive validation using serde with custom validators
- Rate Limiting: 100 requests per minute per IP, 1000 per hour per authenticated user
- CORS Policy: Restrictive CORS allowing only frontend domains

**Authentication Security:**
- Token Storage: JWT in httpOnly cookies with CSRF protection
- Session Management: Redis-backed sessions with 30-day expiration, secure token refresh
- Password Policy: Minimum 8 characters, argon2id hashing with proper salt

### Performance Optimization

**Frontend Performance:**
- Bundle Size Target: <150KB initial bundle, code splitting for recipe parsing features
- Loading Strategy: Critical path CSS inlined, progressive enhancement for advanced features
- Caching Strategy: Service worker caching for recipes, aggressive browser caching for static assets

**Backend Performance:**
- Response Time Target: <200ms for API endpoints, <500ms for complex meal plan generation
- Database Optimization: Query optimization with EXPLAIN ANALYZE, connection pooling, read replicas for scaling
- Caching Strategy: Redis caching for frequently accessed recipes, user sessions, meal plan templates

## Testing Strategy

### Testing Pyramid

```text
    E2E Tests (Playwright)
    /                    \
   Integration Tests      \
  /    (Backend API)       \
Frontend Unit Tests    Backend Unit Tests
(Rust + WASM)         (Rust + tokio-test)
```

### Test Organization

#### Frontend Tests

```text
apps/frontend/tests/
├── unit/                     # Component and utility tests
│   ├── components/          # Template component tests
│   ├── services/            # Service layer tests
│   └── utils/               # Utility function tests
├── integration/             # Frontend integration tests
│   ├── api-client/         # API client tests
│   └── state-management/   # State management tests
└── e2e/                    # End-to-end browser tests
    ├── auth/               # Authentication flows
    ├── recipes/            # Recipe management flows
    ├── cooking/            # Cook mode functionality
    └── offline/            # Offline functionality tests
```

#### Backend Tests

```text
apps/backend/tests/
├── unit/                   # Individual function tests
│   ├── services/          # Service layer tests
│   ├── repositories/      # Data access tests
│   └── utils/             # Utility function tests
├── integration/           # API endpoint tests
│   ├── auth/             # Authentication endpoints
│   ├── recipes/          # Recipe CRUD operations
│   └── cooking/          # Cooking session APIs
└── load/                 # Performance and load tests
    ├── recipe-parsing/   # Parser performance tests
    └── concurrent-users/ # Multi-user load tests
```

### Test Examples

#### Frontend Component Test

```rust
use wasm_bindgen_test::*;
use web_sys::window;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_recipe_card_rendering() {
    let recipe = Recipe {
        id: "test-id".to_string(),
        title: "Test Recipe".to_string(),
        prep_time: 15,
        cook_time: 30,
        difficulty: SkillLevel::Beginner,
        ..Default::default()
    };
    
    let card = RecipeCard::new(recipe, &default_user_context());
    let rendered = card.render().unwrap();
    
    assert!(rendered.contains("Test Recipe"));
    assert!(rendered.contains("45 min")); // total time
    assert!(rendered.contains("Beginner"));
}
```

#### Backend API Test

```rust
use axum_test::TestServer;
use sqlx::PgPool;

#[tokio::test]
async fn test_create_recipe() {
    let pool = setup_test_database().await;
    let app = create_app(pool).await;
    let server = TestServer::new(app).unwrap();
    
    let recipe_request = json!({
        "title": "Test Recipe",
        "ingredients": [{"name": "flour", "quantity": 2, "unit": "cups"}],
        "instructions": [{"step_number": 1, "description": "Mix ingredients"}],
        "prep_time": 15,
        "cook_time": 30
    });
    
    let response = server
        .post("/api/v1/recipes")
        .authorization_bearer("valid-jwt-token")
        .json(&recipe_request)
        .await;
    
    assert_eq!(response.status_code(), 201);
    
    let recipe: Recipe = response.json();
    assert_eq!(recipe.title, "Test Recipe");
    assert_eq!(recipe.total_time, 45);
}
```

#### E2E Test

```typescript
import { test, expect } from '@playwright/test';

test('complete cooking workflow', async ({ page }) => {
  // Login
  await page.goto('/auth/login');
  await page.fill('#email', 'test@example.com');
  await page.fill('#password', 'password123');
  await page.click('button[type="submit"]');

  // Navigate to recipe
  await page.goto('/recipes');
  await page.click('[data-testid="recipe-card"]:first-child');
  
  // Start cooking session
  await page.click('[data-testid="start-cooking"]');
  await expect(page).toHaveURL(/\/cook\/session\//);
  
  // Verify timer functionality
  await page.click('[data-testid="start-timer"]');
  await expect(page.locator('[data-testid="timer-display"]')).toBeVisible();
  
  // Complete cooking session
  await page.click('[data-testid="complete-cooking"]');
  await page.fill('[data-testid="rating-input"]', '5');
  await page.click('[data-testid="submit-rating"]');
  
  await expect(page).toHaveURL('/dashboard');
  await expect(page.locator('[data-testid="success-message"]')).toBeVisible();
});
```

## Coding Standards

### Critical Fullstack Rules

- **Type Sharing:** Always define shared types in `shared/types` and import consistently across frontend and backend
- **Error Handling:** All API endpoints must use the unified error handling pattern with proper HTTP status codes
- **Database Migrations:** Never modify existing migrations; always create new migration files for schema changes
- **Authentication:** Never bypass authentication middleware; all protected routes must validate JWT tokens
- **Caching Strategy:** Always implement cache invalidation when modifying data that affects cached responses
- **Offline Support:** Critical user flows must degrade gracefully when offline; sync changes when connectivity restored
- **Template Security:** Never use raw HTML insertion in Askama templates; rely on built-in escaping
- **Environment Variables:** Access configuration only through the config module, never directly from `std::env`

### Naming Conventions

| Element | Frontend | Backend | Example |
|---------|----------|---------|---------|
| Templates | snake_case.html | - | `recipe_card.html` |
| Route Handlers | snake_case | snake_case | `create_recipe_handler` |
| API Endpoints | kebab-case | - | `/meal-plans/{id}/confirm` |
| Database Tables | snake_case | snake_case | `cooking_sessions` |
| Struct Names | PascalCase | PascalCase | `CookingSession` |
| Functions | snake_case | snake_case | `parse_recipe_url` |

## Error Handling Strategy

### Error Flow

```mermaid
sequenceDiagram
    participant Client
    participant API
    participant Service
    participant Database

    Client->>API: Request with invalid data
    API->>API: Validation fails
    API-->>Client: 400 Bad Request + error details
    
    Client->>API: Valid request
    API->>Service: Process request
    Service->>Database: Query data
    Database-->>Service: Database error
    Service-->>API: Service error
    API->>API: Log error + generate ID
    API-->>Client: 500 Internal Server Error + request ID
    
    note over API: Never expose internal errors to client
    note over Client: Display user-friendly error messages
```

### Error Response Format

```typescript
interface ApiError {
  error: {
    code: string;           // Machine-readable error code
    message: string;        // Human-readable error message
    details?: Record<string, any>; // Additional error context
    timestamp: string;      // ISO timestamp
    requestId: string;      // Unique request identifier for debugging
  };
}
```

### Frontend Error Handling

```typescript
class ApiErrorHandler {
  static handle(error: ApiError): void {
    const { code, message, requestId } = error.error;
    
    switch (code) {
      case 'VALIDATION_ERROR':
        NotificationService.showError('Please check your input and try again.');
        break;
      case 'RECIPE_PARSE_FAILED':
        NotificationService.showError('Unable to parse recipe. Try entering manually.');
        break;
      case 'NETWORK_ERROR':
        NotificationService.showError('Connection issue. Changes saved locally.');
        OfflineQueue.enqueue(error.originalRequest);
        break;
      default:
        NotificationService.showError(`Something went wrong. Reference ID: ${requestId}`);
        console.error('Unhandled API error:', error);
    }
  }
}
```

### Backend Error Handling

```rust
use axum::{response::Response, http::StatusCode};
use serde_json::json;
use uuid::Uuid;

#[derive(Debug)]
pub enum AppError {
    ValidationError(String),
    RecipeParseFailed(String),
    DatabaseError(sqlx::Error),
    Unauthorized,
    NotFound,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let request_id = Uuid::new_v4().to_string();
        
        let (status, code, message) = match self {
            AppError::ValidationError(msg) => {
                (StatusCode::BAD_REQUEST, "VALIDATION_ERROR", msg)
            }
            AppError::RecipeParseFailed(msg) => {
                (StatusCode::UNPROCESSABLE_ENTITY, "RECIPE_PARSE_FAILED", msg)
            }
            AppError::DatabaseError(err) => {
                tracing::error!("Database error {}: {:?}", request_id, err);
                (StatusCode::INTERNAL_SERVER_ERROR, "DATABASE_ERROR", 
                 "A database error occurred".to_string())
            }
            AppError::Unauthorized => {
                (StatusCode::UNAUTHORIZED, "UNAUTHORIZED", 
                 "Authentication required".to_string())
            }
            AppError::NotFound => {
                (StatusCode::NOT_FOUND, "NOT_FOUND", 
                 "Resource not found".to_string())
            }
        };

        let body = json!({
            "error": {
                "code": code,
                "message": message,
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "requestId": request_id
            }
        });

        (status, Json(body)).into_response()
    }
}
```

## Monitoring and Observability

### Monitoring Stack

- **Frontend Monitoring:** Real User Monitoring (RUM) with performance metrics, error tracking with Sentry
- **Backend Monitoring:** Prometheus metrics collection, Grafana dashboards for visualization  
- **Error Tracking:** Structured logging with tracing crate, error aggregation and alerting
- **Performance Monitoring:** APM with request tracing, database query performance analysis

### Key Metrics

**Frontend Metrics:**
- Core Web Vitals (LCP, FID, CLS)
- JavaScript errors and stack traces  
- API response times from client perspective
- User interaction funnel metrics (recipe import → cook → rate)

**Backend Metrics:**
- Request rate, error rate, response time (RED metrics)
- Recipe parsing success rate and accuracy
- Timing prediction accuracy vs actual cooking times
- Database query performance and slow query detection

## Checklist Results Report

*[This section will be populated when the architect-checklist is executed]*

**Architecture Status:** COMPREHENSIVE - Full-stack architecture complete with detailed specifications for all layers

**Readiness for Development:** READY - Architecture provides sufficient detail for AI agent development with clear patterns, standards, and examples

**Key Implementation Priorities:**
1. Recipe parsing service with high accuracy targets
2. Timing coordination engine for multi-dish cooking
3. Offline-first frontend with service worker implementation
4. Real-time notification system for cooking alerts

The architecture successfully addresses all PRD requirements and technical constraints while providing a clear development roadmap for the ImKitchen cooking companion platform.

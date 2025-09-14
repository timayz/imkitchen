# imkitchen Fullstack Architecture Document

## Introduction

This document outlines the complete fullstack architecture for imkitchen, including backend systems, frontend implementation, and their integration. It serves as the single source of truth for AI-driven development, ensuring consistency across the entire technology stack.

This unified approach combines what would traditionally be separate backend and frontend architecture documents, streamlining the development process for modern fullstack applications where these concerns are increasingly intertwined.

### Starter Template or Existing Project

**Project Type:** Greenfield project with Next.js 14+ full-stack foundation

**Architectural Decisions:**

- Single Next.js application combining frontend and backend
- App Router architecture for unified routing and API patterns
- TypeScript-first development with strict type safety
- Tailwind CSS for design system implementation
- Docker containerization for platform-agnostic deployment

### Change Log

| Date       | Version | Description                             | Author              |
| ---------- | ------- | --------------------------------------- | ------------------- |
| 2025-09-14 | 1.0     | Initial fullstack architecture creation | Winston (Architect) |

## High Level Architecture

### Technical Summary

imkitchen utilizes a modern Next.js 14+ fullstack architecture with App Router, combining server-side rendering, static generation, and API routes in a unified TypeScript codebase. The platform leverages PostgreSQL with Prisma ORM for type-safe database operations, implements comprehensive internationalization via next-intl, and provides progressive web app capabilities with offline cooking mode functionality. The architecture prioritizes vendor independence through Docker containerization and abstraction layers, while maintaining kitchen-optimized performance with voice interaction support and mobile-first responsive design.

### Platform and Infrastructure Choice

**Platform:** Multi-cloud compatible with Docker deployment
**Key Services:**

- **Compute:** Docker containers (supports AWS ECS, GCP Cloud Run, Azure Container Instances, DigitalOcean App Platform)
- **Database:** PostgreSQL (supports AWS RDS, GCP Cloud SQL, Azure Database, managed providers)
- **Storage:** S3-compatible object storage with abstraction layer
- **CDN:** CloudFlare or equivalent for global content delivery
- **Email:** SMTP abstraction supporting multiple providers (SendGrid, AWS SES, etc.)

**Deployment Host and Regions:** Global deployment capability with regional data compliance (US, EU, Asia-Pacific)

### Repository Structure

**Structure:** Monorepo with integrated Next.js fullstack application
**Monorepo Tool:** Native Next.js App Router with organized folder structure
**Package Organization:** Feature-based organization with shared utilities, types, and components

### High Level Architecture Diagram

```mermaid
graph TB
    subgraph "Client Layer"
        PWA[Progressive Web App]
        Mobile[Mobile Browser]
        Desktop[Desktop Browser]
    end

    subgraph "CDN/Edge"
        CF[CloudFlare CDN]
        Edge[Edge Functions]
    end

    subgraph "Next.js Application"
        SSG[Static Site Generation]
        SSR[Server-Side Rendering]
        API[API Routes]
        Middleware[Next.js Middleware]
    end

    subgraph "External Services"
        RecipeAPI[Recipe Database API]
        NutritionAPI[Nutrition API]
        VoiceAPI[Voice Recognition Service]
        EmailSVC[Email Service]
    end

    subgraph "Data Layer"
        PG[(PostgreSQL)]
        Redis[(Redis Cache)]
        S3[Object Storage]
    end

    PWA --> CF
    Mobile --> CF
    Desktop --> CF

    CF --> Edge
    Edge --> SSG
    Edge --> SSR

    SSG --> API
    SSR --> API
    API --> Middleware

    API --> RecipeAPI
    API --> NutritionAPI
    API --> VoiceAPI
    API --> EmailSVC

    API --> PG
    API --> Redis
    API --> S3
```

### Architectural Patterns

- **Jamstack with Dynamic APIs:** Static generation for public content with server-side APIs for dynamic functionality - _Rationale:_ Optimal performance for recipe discovery while supporting real-time inventory management
- **Component-Based UI with Server Components:** React Server Components with client-side hydration for interactive features - _Rationale:_ Reduced bundle size and improved performance for kitchen-optimized mobile experience
- **Repository Pattern with Prisma:** Abstract data access through service layer with type-safe ORM - _Rationale:_ Enables testing, caching strategies, and future database migration flexibility
- **API-First Design:** RESTful API routes with OpenAPI documentation and type-safe client generation - _Rationale:_ Supports future mobile app development and third-party integrations
- **Progressive Enhancement:** Core functionality works without JavaScript, enhanced with interactive features - _Rationale:_ Ensures cooking mode reliability in various network conditions
- **Multi-Tenant Architecture:** Household-based data isolation with shared recipe content - _Rationale:_ Supports family coordination while maintaining data privacy and enabling recipe community features

## Tech Stack

### Technology Stack Table

| Category             | Technology                   | Version  | Purpose                                    | Rationale                                                                                      |
| -------------------- | ---------------------------- | -------- | ------------------------------------------ | ---------------------------------------------------------------------------------------------- |
| Frontend Language    | TypeScript                   | 5.0+     | Type-safe frontend development             | Kitchen safety requires predictable interfaces; prevents runtime errors during cooking         |
| Frontend Framework   | Next.js                      | 14+      | Full-stack React framework with App Router | Unified frontend/backend development; excellent SSG/SSR for SEO; built-in internationalization |
| UI Component Library | Custom + Radix UI            | Latest   | Accessible component primitives            | Kitchen accessibility requirements; voice/keyboard navigation; high customization needs        |
| State Management     | React Context + useReducer   | Built-in | Client-side state management               | Sufficient for app complexity; reduces bundle size; integrates with Server Components          |
| Backend Language     | TypeScript                   | 5.0+     | Type-safe backend development              | Shared types between frontend/backend; prevents API contract mismatches                        |
| Backend Framework    | Next.js API Routes           | 14+      | Serverless-style API endpoints             | Unified codebase; automatic deployment optimization; edge computing support                    |
| API Style            | REST + OpenAPI               | 3.0      | RESTful APIs with documentation            | Standard, cacheable, voice-command compatible; clear contract definitions                      |
| Database             | PostgreSQL                   | 15+      | Primary relational database                | ACID compliance for inventory tracking; complex recipe relationships; international scaling    |
| Cache                | Redis                        | 7+       | Session and application caching            | Fast recipe lookup; voice command response times; shopping list synchronization                |
| File Storage         | S3-Compatible Storage        | Latest   | Recipe images and user uploads             | Global CDN distribution; vendor independence; cost optimization                                |
| Authentication       | NextAuth.js                  | 4+       | Authentication and session management      | Next.js integration; multiple providers; secure session handling                               |
| Frontend Testing     | Jest + React Testing Library | Latest   | Component and integration testing          | Kitchen workflow testing; accessibility compliance verification                                |
| Backend Testing      | Jest + Supertest             | Latest   | API endpoint and service testing           | Critical for food safety features; recipe data integrity                                       |
| E2E Testing          | Playwright                   | Latest   | End-to-end user journey testing            | Voice interaction testing; mobile cooking mode validation                                      |
| Build Tool           | Next.js Build                | Built-in | Application bundling and optimization      | Integrated toolchain; automatic optimization; edge deployment                                  |
| Bundler              | Webpack (via Next.js)        | Latest   | Module bundling and code splitting         | Automatic bundle optimization; dynamic imports for cooking mode                                |
| IaC Tool             | Docker + Docker Compose      | Latest   | Infrastructure as code                     | Platform independence; local development parity; cloud agnostic                                |
| CI/CD                | GitHub Actions               | Latest   | Continuous integration and deployment      | Free for open source; excellent Next.js integration; multi-environment support                 |
| Monitoring           | Sentry + Vercel Analytics    | Latest   | Error tracking and performance monitoring  | Real-time cooking session error detection; performance optimization                            |
| Logging              | Winston + Structured Logging | Latest   | Application logging                        | Kitchen session debugging; voice command analysis; security monitoring                         |
| CSS Framework        | Tailwind CSS                 | 3+       | Utility-first styling framework            | Kitchen-first responsive design; consistent spacing; dark mode support                         |

## Data Models

### User Model

**Purpose:** Represents individual users with their preferences, dietary restrictions, and household membership

**Key Attributes:**

- id: string (UUID) - Unique user identifier
- email: string - Primary authentication identifier
- name: string - Display name for household coordination
- dietaryPreferences: string[] - Vegetarian, vegan, keto, etc.
- allergies: string[] - Food allergies and intolerances
- householdId: string - Reference to shared household data
- language: string - Preferred interface language
- timezone: string - For meal planning and cooking schedules

#### TypeScript Interface

```typescript
interface User {
  id: string;
  email: string;
  name: string;
  dietaryPreferences: DietaryPreference[];
  allergies: string[];
  householdId: string;
  language: Language;
  timezone: string;
  createdAt: Date;
  updatedAt: Date;
}

type DietaryPreference =
  | 'vegetarian'
  | 'vegan'
  | 'gluten-free'
  | 'keto'
  | 'paleo'
  | 'dairy-free';
type Language = 'en' | 'es' | 'fr' | 'de';
```

#### Relationships

- Belongs to one Household
- Has many InventoryItems
- Has many MealPlans
- Has many RecipeRatings

### Household Model

**Purpose:** Shared kitchen space for families/roommates with coordinated meal planning and inventory

**Key Attributes:**

- id: string (UUID) - Unique household identifier
- name: string - Household display name
- members: User[] - Array of household members
- settings: HouseholdSettings - Shared preferences and configurations

#### TypeScript Interface

```typescript
interface Household {
  id: string;
  name: string;
  settings: HouseholdSettings;
  createdAt: Date;
  updatedAt: Date;
}

interface HouseholdSettings {
  defaultMeasurementUnit: 'metric' | 'imperial';
  sharedInventory: boolean;
  mealPlanningAccess: 'owner' | 'all-members';
  notificationPreferences: NotificationSettings;
}
```

#### Relationships

- Has many Users (members)
- Has one shared Inventory
- Has many shared MealPlans

### InventoryItem Model

**Purpose:** Individual ingredients and food items tracked in pantry, refrigerator, or freezer

**Key Attributes:**

- id: string (UUID) - Unique item identifier
- name: string - Ingredient name with i18n support
- quantity: number - Current quantity available
- unit: string - Measurement unit (cups, grams, pieces, etc.)
- category: InventoryCategory - Organization category
- location: StorageLocation - Where item is stored
- expirationDate: Date - When item expires
- purchaseDate: Date - When item was acquired
- estimatedCost: number - Optional cost tracking

#### TypeScript Interface

```typescript
interface InventoryItem {
  id: string;
  name: string;
  quantity: number;
  unit: MeasurementUnit;
  category: InventoryCategory;
  location: StorageLocation;
  expirationDate: Date;
  purchaseDate: Date;
  estimatedCost?: number;
  householdId: string;
  addedBy: string; // User ID
  createdAt: Date;
  updatedAt: Date;
}

type InventoryCategory =
  | 'proteins'
  | 'vegetables'
  | 'fruits'
  | 'grains'
  | 'dairy'
  | 'spices'
  | 'condiments'
  | 'beverages'
  | 'baking'
  | 'frozen';
type StorageLocation = 'pantry' | 'refrigerator' | 'freezer';
type MeasurementUnit =
  | 'grams'
  | 'kilograms'
  | 'ounces'
  | 'pounds'
  | 'cups'
  | 'tablespoons'
  | 'teaspoons'
  | 'pieces'
  | 'milliliters'
  | 'liters';
```

#### Relationships

- Belongs to one Household
- Added by one User
- Referenced in many ShoppingListItems
- Used in many Recipes (through ingredients)

### Recipe Model

**Purpose:** Cooking instructions with ingredients, steps, and metadata for meal planning

**Key Attributes:**

- id: string (UUID) - Unique recipe identifier
- title: string - Recipe name with i18n support
- description: string - Brief recipe description
- ingredients: RecipeIngredient[] - Required ingredients with quantities
- instructions: RecipeStep[] - Cooking steps in order
- cookingTime: number - Total cooking time in minutes
- difficulty: DifficultyLevel - Complexity rating
- servings: number - Number of servings recipe yields
- cuisine: string - Cuisine type for categorization
- tags: string[] - Searchable tags

#### TypeScript Interface

```typescript
interface Recipe {
  id: string;
  title: string;
  description: string;
  ingredients: RecipeIngredient[];
  instructions: RecipeStep[];
  cookingTime: number;
  prepTime: number;
  difficulty: DifficultyLevel;
  servings: number;
  cuisine: string;
  tags: string[];
  imageUrl?: string;
  nutritionInfo?: NutritionInfo;
  source: RecipeSource;
  createdAt: Date;
  updatedAt: Date;
}

interface RecipeIngredient {
  name: string;
  quantity: number;
  unit: MeasurementUnit;
  notes?: string;
  essential: boolean;
}

interface RecipeStep {
  stepNumber: number;
  instruction: string;
  duration?: number;
  temperature?: number;
  image?: string;
}

type DifficultyLevel = 'easy' | 'medium' | 'hard';
type RecipeSource = 'user-created' | 'imported' | 'api-external';
```

#### Relationships

- Has many RecipeRatings from Users
- Saved in many UserRecipeCollections
- Used in many MealPlanEntries
- Generates many ShoppingListItems

### MealPlan Model

**Purpose:** Weekly or monthly meal scheduling with family coordination

**Key Attributes:**

- id: string (UUID) - Unique meal plan identifier
- name: string - Meal plan name
- startDate: Date - Beginning of meal plan period
- endDate: Date - End of meal plan period
- entries: MealPlanEntry[] - Individual meal assignments
- householdId: string - Associated household

#### TypeScript Interface

```typescript
interface MealPlan {
  id: string;
  name: string;
  startDate: Date;
  endDate: Date;
  entries: MealPlanEntry[];
  householdId: string;
  createdBy: string; // User ID
  createdAt: Date;
  updatedAt: Date;
}

interface MealPlanEntry {
  id: string;
  date: Date;
  mealType: MealType;
  recipeId: string;
  servings: number;
  notes?: string;
  assignedCook?: string; // User ID
  status: MealStatus;
}

type MealType = 'breakfast' | 'lunch' | 'dinner' | 'snack';
type MealStatus = 'planned' | 'in-progress' | 'completed' | 'skipped';
```

#### Relationships

- Belongs to one Household
- Created by one User
- Contains many MealPlanEntries
- Generates ShoppingLists

### ShoppingList Model

**Purpose:** Automated and manual shopping lists with store organization and purchase tracking

**Key Attributes:**

- id: string (UUID) - Unique shopping list identifier
- name: string - Shopping list name
- items: ShoppingListItem[] - Items to purchase
- generatedFrom: string[] - Source meal plan IDs
- status: ShoppingListStatus - Current list status
- estimatedTotal: number - Projected cost

#### TypeScript Interface

```typescript
interface ShoppingList {
  id: string;
  name: string;
  items: ShoppingListItem[];
  generatedFrom: string[]; // MealPlan IDs
  status: ShoppingListStatus;
  estimatedTotal: number;
  householdId: string;
  createdBy: string; // User ID
  createdAt: Date;
  updatedAt: Date;
}

interface ShoppingListItem {
  id: string;
  name: string;
  quantity: number;
  unit: MeasurementUnit;
  category: StoreCategory;
  purchased: boolean;
  estimatedPrice?: number;
  actualPrice?: number;
  notes?: string;
}

type ShoppingListStatus = 'active' | 'completed' | 'archived';
type StoreCategory =
  | 'produce'
  | 'dairy'
  | 'meat'
  | 'frozen'
  | 'pantry'
  | 'bakery'
  | 'other';
```

#### Relationships

- Belongs to one Household
- Generated from MealPlans
- Created by one User
- Updates InventoryItems when marked purchased

## API Specification

### REST API Specification

```yaml
openapi: 3.0.0
info:
  title: imkitchen API
  version: 1.0.0
  description: Kitchen management platform API for inventory tracking, meal planning, and cooking guidance
servers:
  - url: https://api.imkitchen.com/v1
    description: Production API server
  - url: http://localhost:3000/api
    description: Local development server

paths:
  /auth/login:
    post:
      summary: User authentication
      requestBody:
        required: true
        content:
          application/json:
            schema:
              type: object
              properties:
                email:
                  type: string
                  format: email
                password:
                  type: string
              required: [email, password]
      responses:
        '200':
          description: Successful authentication
          content:
            application/json:
              schema:
                type: object
                properties:
                  user:
                    $ref: '#/components/schemas/User'
                  token:
                    type: string

  /inventory:
    get:
      summary: Get household inventory
      parameters:
        - name: location
          in: query
          schema:
            type: string
            enum: [pantry, refrigerator, freezer]
        - name: category
          in: query
          schema:
            type: string
      responses:
        '200':
          description: Inventory items list
          content:
            application/json:
              schema:
                type: array
                items:
                  $ref: '#/components/schemas/InventoryItem'

    post:
      summary: Add inventory item
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/InventoryItemCreate'
      responses:
        '201':
          description: Item created successfully
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/InventoryItem'

  /inventory/{itemId}:
    put:
      summary: Update inventory item
      parameters:
        - name: itemId
          in: path
          required: true
          schema:
            type: string
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/InventoryItemUpdate'
      responses:
        '200':
          description: Item updated successfully

    delete:
      summary: Remove inventory item
      parameters:
        - name: itemId
          in: path
          required: true
          schema:
            type: string
      responses:
        '204':
          description: Item deleted successfully

  /recipes:
    get:
      summary: Search recipes
      parameters:
        - name: q
          in: query
          description: Search query
          schema:
            type: string
        - name: ingredients
          in: query
          description: Available ingredients
          schema:
            type: array
            items:
              type: string
        - name: cuisine
          in: query
          schema:
            type: string
        - name: maxCookingTime
          in: query
          schema:
            type: integer
      responses:
        '200':
          description: Recipe search results
          content:
            application/json:
              schema:
                type: object
                properties:
                  recipes:
                    type: array
                    items:
                      $ref: '#/components/schemas/Recipe'
                  pagination:
                    $ref: '#/components/schemas/Pagination'

  /recipes/{recipeId}:
    get:
      summary: Get recipe details
      parameters:
        - name: recipeId
          in: path
          required: true
          schema:
            type: string
      responses:
        '200':
          description: Recipe details
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Recipe'

  /meal-plans:
    get:
      summary: Get meal plans
      parameters:
        - name: startDate
          in: query
          schema:
            type: string
            format: date
        - name: endDate
          in: query
          schema:
            type: string
            format: date
      responses:
        '200':
          description: Meal plans list
          content:
            application/json:
              schema:
                type: array
                items:
                  $ref: '#/components/schemas/MealPlan'

    post:
      summary: Create meal plan
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/MealPlanCreate'
      responses:
        '201':
          description: Meal plan created
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/MealPlan'

  /shopping-lists:
    get:
      summary: Get shopping lists
      responses:
        '200':
          description: Shopping lists
          content:
            application/json:
              schema:
                type: array
                items:
                  $ref: '#/components/schemas/ShoppingList'

    post:
      summary: Generate shopping list from meal plan
      requestBody:
        required: true
        content:
          application/json:
            schema:
              type: object
              properties:
                mealPlanId:
                  type: string
                name:
                  type: string
              required: [mealPlanId]
      responses:
        '201':
          description: Shopping list generated
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/ShoppingList'

  /voice/commands:
    post:
      summary: Process voice command
      requestBody:
        required: true
        content:
          application/json:
            schema:
              type: object
              properties:
                command:
                  type: string
                context:
                  type: object
              required: [command]
      responses:
        '200':
          description: Command processed
          content:
            application/json:
              schema:
                type: object
                properties:
                  action:
                    type: string
                  response:
                    type: string
                  data:
                    type: object

components:
  schemas:
    User:
      type: object
      properties:
        id:
          type: string
        email:
          type: string
        name:
          type: string
        dietaryPreferences:
          type: array
          items:
            type: string
        language:
          type: string

    InventoryItem:
      type: object
      properties:
        id:
          type: string
        name:
          type: string
        quantity:
          type: number
        unit:
          type: string
        category:
          type: string
        location:
          type: string
        expirationDate:
          type: string
          format: date

    Recipe:
      type: object
      properties:
        id:
          type: string
        title:
          type: string
        description:
          type: string
        cookingTime:
          type: integer
        difficulty:
          type: string
        servings:
          type: integer

  securitySchemes:
    BearerAuth:
      type: http
      scheme: bearer
      bearerFormat: JWT

security:
  - BearerAuth: []
```

## Components

### Authentication Service

**Responsibility:** User authentication, session management, and household access control

**Key Interfaces:**

- POST /api/auth/login - User login with email/password
- POST /api/auth/register - New user registration
- GET /api/auth/session - Current session validation
- POST /api/auth/logout - Session termination

**Dependencies:** NextAuth.js, PostgreSQL, Redis (session storage)

**Technology Stack:** NextAuth.js with JWT tokens, secure HTTP-only cookies, CSRF protection

### Inventory Management Service

**Responsibility:** Pantry and refrigerator tracking, expiration monitoring, quantity management

**Key Interfaces:**

- GET /api/inventory - Retrieve household inventory with filtering
- POST /api/inventory - Add new inventory items
- PUT /api/inventory/[id] - Update item quantities and expiration dates
- DELETE /api/inventory/[id] - Remove inventory items

**Dependencies:** Recipe Service (ingredient matching), Shopping List Service (replenishment)

**Technology Stack:** Prisma ORM with PostgreSQL, Redis caching for frequent lookups, image storage abstraction

### Recipe Management Service

**Responsibility:** Recipe search, storage, rating, and ingredient-based suggestions

**Key Interfaces:**

- GET /api/recipes/search - Recipe search with filters and available ingredients
- GET /api/recipes/[id] - Recipe details with instructions and nutrition
- POST /api/recipes/rate - User recipe ratings and reviews
- GET /api/recipes/suggestions - Personalized recipe recommendations

**Dependencies:** External Recipe APIs, Inventory Service (ingredient matching), User Preferences

**Technology Stack:** External API integration (Spoonacular/Edamam), PostgreSQL for user recipes, Redis for search caching

### Meal Planning Service

**Responsibility:** Weekly/monthly meal scheduling, family coordination, calendar management

**Key Interfaces:**

- GET /api/meal-plans - Retrieve meal plans for date range
- POST /api/meal-plans - Create new meal plan with recipe assignments
- PUT /api/meal-plans/[id]/entries - Update specific meal assignments
- GET /api/meal-plans/suggestions - AI-powered meal plan generation

**Dependencies:** Recipe Service (meal assignment), Inventory Service (ingredient availability), Shopping List Service (list generation)

**Technology Stack:** PostgreSQL with date indexing, real-time updates via WebSocket, Calendar integration APIs

### Shopping List Service

**Responsibility:** Automated list generation, store organization, purchase tracking

**Key Interfaces:**

- GET /api/shopping-lists - Retrieve active shopping lists
- POST /api/shopping-lists/generate - Generate list from meal plans
- PUT /api/shopping-lists/[id]/items - Mark items as purchased
- GET /api/shopping-lists/[id]/optimize - Store-optimized shopping routes

**Dependencies:** Meal Planning Service (source data), Inventory Service (current stock), Store APIs (pricing/availability)

**Technology Stack:** PostgreSQL with JSON fields for flexible item data, integration with grocery store APIs, geolocation services

### Voice Interaction Service

**Responsibility:** Voice command processing, cooking mode assistance, hands-free navigation

**Key Interfaces:**

- POST /api/voice/process - Voice command interpretation
- POST /api/voice/cooking/next - Advance cooking step via voice
- POST /api/voice/timer - Voice-activated timer management
- GET /api/voice/status - Current voice session state

**Dependencies:** All services (voice commands can access any feature), External Speech-to-Text APIs

**Technology Stack:** WebSpeech API with fallback to cloud services, real-time WebSocket connections, context-aware command processing

### Notification Service

**Responsibility:** Expiration alerts, meal reminders, cooking timers, family coordination

**Key Interfaces:**

- POST /api/notifications/send - Send immediate notifications
- GET /api/notifications/settings - User notification preferences
- POST /api/notifications/schedule - Schedule future notifications
- WebSocket /api/notifications/live - Real-time notification delivery

**Dependencies:** All services (notifications triggered by various events), Email Service (external notifications)

**Technology Stack:** WebSocket for real-time notifications, scheduled jobs with node-cron, email abstraction layer, push notification support

### Component Diagrams

```mermaid
graph TB
    subgraph "Frontend Layer"
        UI[React Components]
        State[State Management]
        Services[Service Layer]
    end

    subgraph "API Layer"
        Auth[Auth Service]
        Inventory[Inventory Service]
        Recipes[Recipe Service]
        MealPlan[Meal Planning Service]
        Shopping[Shopping List Service]
        Voice[Voice Service]
        Notifications[Notification Service]
    end

    subgraph "Data Layer"
        DB[(PostgreSQL)]
        Cache[(Redis)]
        Storage[Object Storage]
    end

    subgraph "External Services"
        RecipeAPI[Recipe APIs]
        VoiceAPI[Speech APIs]
        EmailAPI[Email Service]
        StoreAPI[Store APIs]
    end

    UI --> Services
    State --> Services
    Services --> Auth
    Services --> Inventory
    Services --> Recipes
    Services --> MealPlan
    Services --> Shopping
    Services --> Voice
    Services --> Notifications

    Auth --> DB
    Inventory --> DB
    Inventory --> Cache
    Recipes --> DB
    Recipes --> Cache
    Recipes --> RecipeAPI
    MealPlan --> DB
    Shopping --> DB
    Shopping --> StoreAPI
    Voice --> VoiceAPI
    Notifications --> EmailAPI

    Inventory --> Storage
    Recipes --> Storage
```

## External APIs

### Spoonacular Recipe API

- **Purpose:** Recipe database, nutrition information, ingredient recognition
- **Documentation:** https://spoonacular.com/food-api/docs
- **Base URL(s):** https://api.spoonacular.com/
- **Authentication:** API key in query parameter or header
- **Rate Limits:** 150 requests/day (free), 500+ (paid plans)

**Key Endpoints Used:**

- `GET /recipes/complexSearch` - Recipe search with ingredient filters
- `GET /recipes/{id}/information` - Detailed recipe information
- `GET /recipes/findByIngredients` - Recipes based on available ingredients
- `GET /food/ingredients/search` - Ingredient search and autocomplete

**Integration Notes:** Primary recipe source with fallback to user-generated content; caching essential due to rate limits; nutrition data integration for meal planning

**Error Recovery Strategy:**

- **Rate Limit Exceeded:** Switch to cached recipes and user-generated content; display rate limit notice with retry timing
- **Service Unavailable:** Fallback to OpenFoodFacts data and local recipe database; maintain core search functionality
- **API Key Invalid:** Graceful degradation to manual recipe entry with clear user messaging about reduced functionality
- **Network Timeout:** Use cached recipe data with offline indicators; retry with exponential backoff

### OpenFoodFacts API

- **Purpose:** Product information, barcode scanning, nutritional data
- **Documentation:** https://openfoodfacts.github.io/openfoodfacts-server/api/
- **Base URL(s):** https://world.openfoodfacts.org/api/v0/
- **Authentication:** None required (open data)
- **Rate Limits:** None specified (reasonable use expected)

**Key Endpoints Used:**

- `GET /product/{barcode}.json` - Product information by barcode
- `GET /cgi/search.pl` - Product search functionality
- `GET /api/v0/product/{barcode}` - Detailed product data

**Integration Notes:** Used for barcode scanning functionality in inventory management; supplement to manual ingredient entry; multilingual product data available

**Error Recovery Strategy:**

- **Service Unavailable:** Fallback to manual ingredient entry with barcode recognition disabled temporarily
- **Product Not Found:** Provide manual entry form pre-populated with barcode for future database contribution
- **Network Issues:** Cache successful barcode scans locally; sync when connectivity restored
- **Invalid Barcode:** Clear error messaging with option to manually enter product information

### Web Speech API

- **Purpose:** Voice recognition and speech synthesis for cooking mode
- **Documentation:** https://developer.mozilla.org/en-US/docs/Web/API/Web_Speech_API
- **Base URL(s):** Browser-native API (no external calls)
- **Authentication:** User permission required
- **Rate Limits:** Browser-dependent

**Key Endpoints Used:**

- `SpeechRecognition` - Voice command recognition
- `SpeechSynthesis` - Text-to-speech for cooking instructions
- `speechSynthesis.speak()` - Read cooking steps aloud

**Integration Notes:** Primary voice interface with cloud service fallback; offline capability essential for cooking mode; multiple language support required

**Error Recovery Strategy:**

- **Microphone Permission Denied:** Fallback to touch/click interface with clear instructions for enabling voice
- **Speech Recognition Failure:** Provide manual text input alternative; retry voice recognition with user feedback
- **Browser Compatibility:** Progressive enhancement with feature detection; manual controls always available
- **Noisy Environment:** Implement noise filtering; provide visual feedback for recognized commands; manual override options

## Core Workflows

### Recipe Discovery to Meal Planning Workflow

```mermaid
sequenceDiagram
    participant User
    participant UI
    participant RecipeService
    participant InventoryService
    participant MealPlanService
    participant DB

    User->>UI: Search for recipes
    UI->>RecipeService: GET /api/recipes/search?ingredients=[available]
    RecipeService->>InventoryService: Get household inventory
    InventoryService->>DB: Query inventory items
    DB-->>InventoryService: Return inventory data
    InventoryService-->>RecipeService: Available ingredients list
    RecipeService->>ExternalAPI: Search recipes with filters
    ExternalAPI-->>RecipeService: Recipe results
    RecipeService->>DB: Cache recipe data
    RecipeService-->>UI: Recipes with ingredient match %
    UI-->>User: Display recipes with availability indicators

    User->>UI: Select recipe for meal plan
    UI->>MealPlanService: POST /api/meal-plans/entries
    MealPlanService->>DB: Save meal plan entry
    MealPlanService->>InventoryService: Reserve ingredients
    InventoryService->>DB: Update inventory reservations
    MealPlanService-->>UI: Confirm meal assignment
    UI-->>User: Show updated meal plan
```

### Shopping List Generation Workflow

```mermaid
sequenceDiagram
    participant User
    participant UI
    participant ShoppingService
    participant MealPlanService
    participant InventoryService
    participant DB

    User->>UI: Generate shopping list
    UI->>ShoppingService: POST /api/shopping-lists/generate
    ShoppingService->>MealPlanService: Get week's meal plan
    MealPlanService->>DB: Query meal plan entries
    DB-->>MealPlanService: Return planned meals
    MealPlanService-->>ShoppingService: Required ingredients list

    ShoppingService->>InventoryService: Check current inventory
    InventoryService->>DB: Query available quantities
    DB-->>InventoryService: Inventory levels
    InventoryService-->>ShoppingService: Available vs needed

    ShoppingService->>DB: Create shopping list
    ShoppingService->>StoreAPI: Get pricing estimates
    StoreAPI-->>ShoppingService: Price data
    ShoppingService->>DB: Update list with estimates
    ShoppingService-->>UI: Generated shopping list
    UI-->>User: Show categorized shopping list
```

### Voice-Controlled Cooking Workflow

```mermaid
sequenceDiagram
    participant User
    participant VoiceAPI
    participant UI
    participant VoiceService
    participant RecipeService
    participant NotificationService

    User->>VoiceAPI: "Start cooking [recipe name]"
    VoiceAPI->>UI: Speech recognition result
    UI->>VoiceService: POST /api/voice/process
    VoiceService->>RecipeService: Get recipe details
    RecipeService->>DB: Query recipe steps
    DB-->>RecipeService: Recipe data
    RecipeService-->>VoiceService: Cooking instructions
    VoiceService-->>UI: Initialize cooking mode
    UI->>VoiceAPI: Speak first instruction
    VoiceAPI-->>User: Audio: "First, preheat oven to 350°F"

    User->>VoiceAPI: "Next step"
    VoiceAPI->>UI: Voice command recognized
    UI->>VoiceService: POST /api/voice/cooking/next
    VoiceService->>NotificationService: Start timer if needed
    NotificationService->>WebSocket: Real-time timer updates
    VoiceService-->>UI: Next instruction
    UI->>VoiceAPI: Speak instruction
    VoiceAPI-->>User: Audio cooking guidance

    Note over User,NotificationService: Timer completion
    NotificationService->>WebSocket: Timer alert
    WebSocket->>UI: Show/sound alert
    UI->>VoiceAPI: "Timer finished"
    VoiceAPI-->>User: Audio timer notification
```

## Database Schema

```sql
-- Users and Households
CREATE TABLE households (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    settings JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE NOT NULL,
    name VARCHAR(255) NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    dietary_preferences TEXT[],
    allergies TEXT[],
    household_id UUID REFERENCES households(id),
    language VARCHAR(5) DEFAULT 'en',
    timezone VARCHAR(50) DEFAULT 'UTC',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Inventory Management
CREATE TYPE storage_location AS ENUM ('pantry', 'refrigerator', 'freezer');
CREATE TYPE inventory_category AS ENUM ('proteins', 'vegetables', 'fruits', 'grains', 'dairy', 'spices', 'condiments', 'beverages', 'baking', 'frozen');

CREATE TABLE inventory_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    quantity DECIMAL(10,2) NOT NULL,
    unit VARCHAR(50) NOT NULL,
    category inventory_category NOT NULL,
    location storage_location NOT NULL,
    expiration_date DATE,
    purchase_date DATE DEFAULT CURRENT_DATE,
    estimated_cost DECIMAL(10,2),
    household_id UUID REFERENCES households(id),
    added_by UUID REFERENCES users(id),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Recipes
CREATE TYPE difficulty_level AS ENUM ('easy', 'medium', 'hard');
CREATE TYPE recipe_source AS ENUM ('user-created', 'imported', 'api-external');

CREATE TABLE recipes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    external_id VARCHAR(255), -- For API-sourced recipes
    title VARCHAR(500) NOT NULL,
    description TEXT,
    ingredients JSONB NOT NULL, -- Array of RecipeIngredient objects
    instructions JSONB NOT NULL, -- Array of RecipeStep objects
    cooking_time INTEGER, -- minutes
    prep_time INTEGER, -- minutes
    difficulty difficulty_level,
    servings INTEGER,
    cuisine VARCHAR(100),
    tags TEXT[],
    image_url VARCHAR(500),
    nutrition_info JSONB,
    source recipe_source DEFAULT 'user-created',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE TABLE user_recipe_ratings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id),
    recipe_id UUID REFERENCES recipes(id),
    rating INTEGER CHECK (rating >= 1 AND rating <= 5),
    review TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(user_id, recipe_id)
);

-- Meal Planning
CREATE TYPE meal_type AS ENUM ('breakfast', 'lunch', 'dinner', 'snack');
CREATE TYPE meal_status AS ENUM ('planned', 'in-progress', 'completed', 'skipped');

CREATE TABLE meal_plans (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    start_date DATE NOT NULL,
    end_date DATE NOT NULL,
    household_id UUID REFERENCES households(id),
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE TABLE meal_plan_entries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    meal_plan_id UUID REFERENCES meal_plans(id),
    date DATE NOT NULL,
    meal_type meal_type NOT NULL,
    recipe_id UUID REFERENCES recipes(id),
    servings INTEGER DEFAULT 1,
    notes TEXT,
    assigned_cook UUID REFERENCES users(id),
    status meal_status DEFAULT 'planned',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Shopping Lists
CREATE TYPE shopping_list_status AS ENUM ('active', 'completed', 'archived');
CREATE TYPE store_category AS ENUM ('produce', 'dairy', 'meat', 'frozen', 'pantry', 'bakery', 'other');

CREATE TABLE shopping_lists (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    status shopping_list_status DEFAULT 'active',
    generated_from UUID[], -- Array of meal_plan IDs
    estimated_total DECIMAL(10,2),
    household_id UUID REFERENCES households(id),
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE TABLE shopping_list_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    shopping_list_id UUID REFERENCES shopping_lists(id),
    name VARCHAR(255) NOT NULL,
    quantity DECIMAL(10,2) NOT NULL,
    unit VARCHAR(50) NOT NULL,
    category store_category DEFAULT 'other',
    purchased BOOLEAN DEFAULT FALSE,
    estimated_price DECIMAL(10,2),
    actual_price DECIMAL(10,2),
    notes TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Indexes for performance
CREATE INDEX idx_inventory_household_location ON inventory_items(household_id, location);
CREATE INDEX idx_inventory_expiration ON inventory_items(expiration_date) WHERE expiration_date IS NOT NULL;
CREATE INDEX idx_recipes_source ON recipes(source);
CREATE INDEX idx_recipes_external_id ON recipes(external_id) WHERE external_id IS NOT NULL;
CREATE INDEX idx_meal_plan_entries_date ON meal_plan_entries(date);
CREATE INDEX idx_shopping_lists_household_status ON shopping_lists(household_id, status);

-- Full-text search for recipes
CREATE INDEX idx_recipes_search ON recipes USING gin(to_tsvector('english', title || ' ' || description));
```

## Frontend Architecture

### Component Architecture

#### Component Organization

```text
src/
├── components/
│   ├── ui/                    # Base UI components (buttons, inputs, etc.)
│   ├── forms/                 # Form components with validation
│   ├── inventory/             # Inventory-specific components
│   ├── recipes/               # Recipe-related components
│   ├── meal-planning/         # Meal planning components
│   ├── shopping/              # Shopping list components
│   ├── cooking/               # Cooking mode components
│   ├── voice/                 # Voice interaction components
│   └── layout/                # Layout and navigation components
├── app/                       # Next.js 14 app directory
│   ├── (auth)/               # Auth route group
│   ├── dashboard/            # Dashboard pages
│   ├── inventory/            # Inventory pages
│   ├── recipes/              # Recipe pages
│   ├── meal-planning/        # Meal planning pages
│   ├── shopping/             # Shopping pages
│   ├── cooking/              # Cooking mode pages
│   ├── api/                  # API routes
│   ├── globals.css           # Global styles
│   └── layout.tsx            # Root layout
├── hooks/                    # Custom React hooks
├── lib/                      # Utility functions and configurations
├── stores/                   # State management
├── types/                    # TypeScript type definitions
└── services/                 # API client services
```

#### Component Template

```typescript
import { FC, ReactNode } from 'react';
import { cn } from '@/lib/utils';

interface ComponentProps {
  children?: ReactNode;
  className?: string;
  variant?: 'primary' | 'secondary' | 'danger';
  size?: 'sm' | 'md' | 'lg';
  disabled?: boolean;
  onClick?: () => void;
}

const Component: FC<ComponentProps> = ({
  children,
  className,
  variant = 'primary',
  size = 'md',
  disabled = false,
  onClick,
  ...props
}) => {
  const baseClasses = 'inline-flex items-center justify-center rounded-md font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50';

  const variantClasses = {
    primary: 'bg-orange-500 text-white hover:bg-orange-600',
    secondary: 'bg-gray-200 text-gray-900 hover:bg-gray-300',
    danger: 'bg-red-500 text-white hover:bg-red-600',
  };

  const sizeClasses = {
    sm: 'h-9 px-3 text-sm',
    md: 'h-11 px-4 text-base',
    lg: 'h-12 px-6 text-lg',
  };

  return (
    <button
      className={cn(
        baseClasses,
        variantClasses[variant],
        sizeClasses[size],
        className
      )}
      disabled={disabled}
      onClick={onClick}
      {...props}
    >
      {children}
    </button>
  );
};

export default Component;
```

### State Management Architecture

#### State Structure

```typescript
// Global App State
interface AppState {
  user: {
    currentUser: User | null;
    household: Household | null;
    preferences: UserPreferences;
    isAuthenticated: boolean;
  };

  inventory: {
    items: InventoryItem[];
    categories: InventoryCategory[];
    locations: StorageLocation[];
    filters: InventoryFilters;
    loading: boolean;
    error: string | null;
  };

  recipes: {
    searchResults: Recipe[];
    favorites: Recipe[];
    recentlyViewed: Recipe[];
    currentRecipe: Recipe | null;
    suggestions: Recipe[];
    loading: boolean;
  };

  mealPlanning: {
    currentPlan: MealPlan | null;
    weeklyPlans: MealPlan[];
    calendarView: CalendarView;
    draggedRecipe: Recipe | null;
    loading: boolean;
  };

  shopping: {
    activeLists: ShoppingList[];
    currentList: ShoppingList | null;
    categories: StoreCategory[];
    loading: boolean;
  };

  cooking: {
    activeSession: CookingSession | null;
    currentStep: number;
    timers: Timer[];
    voiceActive: boolean;
    progress: CookingProgress;
  };

  voice: {
    isListening: boolean;
    isProcessing: boolean;
    lastCommand: string;
    error: string | null;
    supported: boolean;
  };

  ui: {
    theme: 'light' | 'dark' | 'system';
    language: Language;
    notifications: Notification[];
    modals: ModalState;
    navigation: NavigationState;
  };
}
```

#### State Management Patterns

- **Context + useReducer for Global State** - Authentication, user preferences, household data
- **React Query for Server State** - API data caching, synchronization, background updates
- **useState for Local Component State** - Form inputs, UI interactions, temporary state
- **useCallback and useMemo for Performance** - Prevent unnecessary re-renders in cooking mode
- **Custom Hooks for Reusable Logic** - Voice commands, timer management, inventory operations

### Routing Architecture

#### Route Organization

```text
app/
├── (auth)/
│   ├── login/
│   └── register/
├── dashboard/
│   └── page.tsx
├── inventory/
│   ├── page.tsx
│   ├── add/
│   └── [itemId]/
├── recipes/
│   ├── page.tsx
│   ├── search/
│   ├── favorites/
│   ├── [recipeId]/
│   └── create/
├── meal-planning/
│   ├── page.tsx
│   ├── calendar/
│   └── templates/
├── shopping/
│   ├── page.tsx
│   ├── lists/
│   └── [listId]/
├── cooking/
│   ├── [recipeId]/
│   └── mode/
├── settings/
│   ├── page.tsx
│   ├── profile/
│   ├── household/
│   └── preferences/
└── api/
    ├── auth/
    ├── inventory/
    ├── recipes/
    ├── meal-plans/
    ├── shopping/
    └── voice/
```

#### Protected Route Pattern

```typescript
import { redirect } from 'next/navigation';
import { auth } from '@/lib/auth';

interface ProtectedLayoutProps {
  children: React.ReactNode;
}

export default async function ProtectedLayout({ children }: ProtectedLayoutProps) {
  const session = await auth();

  if (!session?.user) {
    redirect('/login');
  }

  return (
    <div className="min-h-screen bg-background">
      <Navigation user={session.user} />
      <main className="container mx-auto px-4 py-8">
        {children}
      </main>
    </div>
  );
}
```

### Frontend Services Layer

#### API Client Setup

```typescript
import { z } from 'zod';

const API_BASE_URL = process.env.NEXT_PUBLIC_API_URL || '/api';

class ApiClient {
  private baseURL: string;

  constructor(baseURL: string = API_BASE_URL) {
    this.baseURL = baseURL;
  }

  private async request<T>(
    endpoint: string,
    options: RequestInit = {}
  ): Promise<T> {
    const url = `${this.baseURL}${endpoint}`;

    const config: RequestInit = {
      headers: {
        'Content-Type': 'application/json',
        ...options.headers,
      },
      ...options,
    };

    const response = await fetch(url, config);

    if (!response.ok) {
      const error = await response.json();
      throw new ApiError(error.message, response.status, error);
    }

    return response.json();
  }

  get<T>(endpoint: string, params?: Record<string, string>): Promise<T> {
    const searchParams = params ? `?${new URLSearchParams(params)}` : '';
    return this.request<T>(`${endpoint}${searchParams}`);
  }

  post<T>(endpoint: string, data?: any): Promise<T> {
    return this.request<T>(endpoint, {
      method: 'POST',
      body: data ? JSON.stringify(data) : undefined,
    });
  }

  put<T>(endpoint: string, data?: any): Promise<T> {
    return this.request<T>(endpoint, {
      method: 'PUT',
      body: data ? JSON.stringify(data) : undefined,
    });
  }

  delete<T>(endpoint: string): Promise<T> {
    return this.request<T>(endpoint, { method: 'DELETE' });
  }
}

export const apiClient = new ApiClient();

class ApiError extends Error {
  constructor(
    message: string,
    public status: number,
    public details?: any
  ) {
    super(message);
    this.name = 'ApiError';
  }
}
```

#### Service Example

```typescript
import { apiClient } from '@/lib/api-client';
import {
  InventoryItem,
  InventoryItemCreate,
  InventoryItemUpdate,
} from '@/types/inventory';

export class InventoryService {
  static async getItems(filters?: {
    location?: string;
    category?: string;
    expiringSoon?: boolean;
  }): Promise<InventoryItem[]> {
    return apiClient.get<InventoryItem[]>('/inventory', filters);
  }

  static async createItem(item: InventoryItemCreate): Promise<InventoryItem> {
    return apiClient.post<InventoryItem>('/inventory', item);
  }

  static async updateItem(
    id: string,
    updates: InventoryItemUpdate
  ): Promise<InventoryItem> {
    return apiClient.put<InventoryItem>(`/inventory/${id}`, updates);
  }

  static async deleteItem(id: string): Promise<void> {
    return apiClient.delete<void>(`/inventory/${id}`);
  }

  static async getExpiringItems(days: number = 7): Promise<InventoryItem[]> {
    return apiClient.get<InventoryItem[]>('/inventory/expiring', {
      days: days.toString(),
    });
  }
}
```

## Backend Architecture

### Service Architecture

#### Controller Organization

```text
src/
├── app/
│   ├── api/
│   │   ├── auth/
│   │   │   ├── login/route.ts
│   │   │   ├── register/route.ts
│   │   │   └── session/route.ts
│   │   ├── inventory/
│   │   │   ├── route.ts
│   │   │   ├── [itemId]/route.ts
│   │   │   └── expiring/route.ts
│   │   ├── recipes/
│   │   │   ├── route.ts
│   │   │   ├── search/route.ts
│   │   │   ├── [recipeId]/route.ts
│   │   │   └── suggestions/route.ts
│   │   ├── meal-plans/
│   │   │   ├── route.ts
│   │   │   ├── [planId]/route.ts
│   │   │   └── generate/route.ts
│   │   ├── shopping/
│   │   │   ├── route.ts
│   │   │   ├── lists/route.ts
│   │   │   └── [listId]/route.ts
│   │   └── voice/
│   │       ├── process/route.ts
│   │       ├── cooking/route.ts
│   │       └── commands/route.ts
├── lib/
│   ├── services/
│   ├── repositories/
│   ├── middleware/
│   ├── utils/
│   └── validators/
└── types/
```

#### Controller Template

```typescript
import { NextRequest, NextResponse } from 'next/server';
import { z } from 'zod';
import { auth } from '@/lib/auth';
import { InventoryService } from '@/lib/services/inventory-service';
import { withErrorHandler } from '@/lib/middleware/error-handler';
import { validateRequest } from '@/lib/middleware/validation';

const CreateInventoryItemSchema = z.object({
  name: z.string().min(1).max(255),
  quantity: z.number().positive(),
  unit: z.string().min(1).max(50),
  category: z.enum([
    'proteins',
    'vegetables',
    'fruits',
    'grains',
    'dairy',
    'spices',
    'condiments',
    'beverages',
    'baking',
    'frozen',
  ]),
  location: z.enum(['pantry', 'refrigerator', 'freezer']),
  expirationDate: z
    .string()
    .optional()
    .transform(str => (str ? new Date(str) : undefined)),
  estimatedCost: z.number().optional(),
});

export async function GET(request: NextRequest) {
  return withErrorHandler(async () => {
    const session = await auth();
    if (!session?.user?.householdId) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }

    const { searchParams } = new URL(request.url);
    const location = searchParams.get('location');
    const category = searchParams.get('category');

    const items = await InventoryService.getHouseholdItems(
      session.user.householdId,
      { location, category }
    );

    return NextResponse.json(items);
  });
}

export async function POST(request: NextRequest) {
  return withErrorHandler(async () => {
    const session = await auth();
    if (!session?.user?.householdId) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }

    const body = await request.json();
    const validatedData = await validateRequest(
      CreateInventoryItemSchema,
      body
    );

    const item = await InventoryService.createItem({
      ...validatedData,
      householdId: session.user.householdId,
      addedBy: session.user.id,
    });

    return NextResponse.json(item, { status: 201 });
  });
}
```

### Database Architecture

#### Schema Design

```sql
-- Database schema defined in previous section
-- Key design principles:
-- 1. UUID primary keys for security and distribution
-- 2. JSONB for flexible nested data (ingredients, instructions)
-- 3. Proper foreign key constraints
-- 4. Enum types for controlled vocabularies
-- 5. Indexes for performance-critical queries
-- 6. Full-text search capabilities
```

#### Data Access Layer

```typescript
import { PrismaClient } from '@prisma/client';
import { z } from 'zod';

const prisma = new PrismaClient();

export class InventoryRepository {
  static async findByHousehold(
    householdId: string,
    filters?: {
      location?: string;
      category?: string;
      expiringSoon?: boolean;
    }
  ) {
    const where: any = { householdId };

    if (filters?.location) {
      where.location = filters.location;
    }

    if (filters?.category) {
      where.category = filters.category;
    }

    if (filters?.expiringSoon) {
      const futureDate = new Date();
      futureDate.setDate(futureDate.getDate() + 7);
      where.expirationDate = {
        lte: futureDate,
        gte: new Date(),
      };
    }

    return prisma.inventoryItem.findMany({
      where,
      orderBy: [{ expirationDate: 'asc' }, { name: 'asc' }],
      include: {
        addedBy: {
          select: { name: true },
        },
      },
    });
  }

  static async create(data: {
    name: string;
    quantity: number;
    unit: string;
    category: string;
    location: string;
    expirationDate?: Date;
    estimatedCost?: number;
    householdId: string;
    addedBy: string;
  }) {
    return prisma.inventoryItem.create({
      data,
      include: {
        addedBy: {
          select: { name: true },
        },
      },
    });
  }

  static async update(
    id: string,
    data: Partial<{
      quantity: number;
      expirationDate: Date;
      location: string;
      estimatedCost: number;
    }>
  ) {
    return prisma.inventoryItem.update({
      where: { id },
      data: {
        ...data,
        updatedAt: new Date(),
      },
    });
  }

  static async delete(id: string) {
    return prisma.inventoryItem.delete({
      where: { id },
    });
  }

  static async findExpiring(householdId: string, days: number = 7) {
    const futureDate = new Date();
    futureDate.setDate(futureDate.getDate() + days);

    return prisma.inventoryItem.findMany({
      where: {
        householdId,
        expirationDate: {
          lte: futureDate,
          gte: new Date(),
        },
      },
      orderBy: { expirationDate: 'asc' },
    });
  }
}
```

### Authentication and Authorization

#### Auth Flow

```mermaid
sequenceDiagram
    participant User
    participant Frontend
    participant NextAuth
    participant Database
    participant Session

    User->>Frontend: Login request
    Frontend->>NextAuth: Credentials submission
    NextAuth->>Database: Verify user credentials
    Database-->>NextAuth: User data
    NextAuth->>Session: Create session token
    Session-->>NextAuth: Session created
    NextAuth-->>Frontend: Authenticated session
    Frontend-->>User: Redirect to dashboard

    Note over User,Session: Subsequent requests
    User->>Frontend: Protected resource request
    Frontend->>NextAuth: Validate session
    NextAuth->>Session: Check token validity
    Session-->>NextAuth: Valid session
    NextAuth-->>Frontend: User context
    Frontend-->>User: Authorized response
```

#### Middleware/Guards

```typescript
import { NextRequest, NextResponse } from 'next/server';
import { getToken } from 'next-auth/jwt';

export async function middleware(request: NextRequest) {
  // Check authentication for API routes
  if (request.nextUrl.pathname.startsWith('/api/')) {
    const token = await getToken({ req: request });

    // Public API routes that don't require authentication
    const publicRoutes = ['/api/auth', '/api/health', '/api/recipes/public'];
    const isPublicRoute = publicRoutes.some(route =>
      request.nextUrl.pathname.startsWith(route)
    );

    if (!isPublicRoute && !token) {
      return NextResponse.json(
        { error: 'Authentication required' },
        { status: 401 }
      );
    }

    // Add user context to headers for API routes
    if (token) {
      const requestHeaders = new Headers(request.headers);
      requestHeaders.set('x-user-id', token.sub!);
      requestHeaders.set('x-household-id', token.householdId as string);

      return NextResponse.next({
        request: {
          headers: requestHeaders,
        },
      });
    }
  }

  // Check authentication for protected pages
  const protectedPaths = [
    '/dashboard',
    '/inventory',
    '/recipes',
    '/meal-planning',
    '/shopping',
    '/cooking',
  ];
  const isProtectedPath = protectedPaths.some(path =>
    request.nextUrl.pathname.startsWith(path)
  );

  if (isProtectedPath) {
    const token = await getToken({ req: request });

    if (!token) {
      const url = request.nextUrl.clone();
      url.pathname = '/login';
      url.searchParams.set('callbackUrl', request.nextUrl.pathname);
      return NextResponse.redirect(url);
    }
  }

  return NextResponse.next();
}

export const config = {
  matcher: [
    '/api/:path*',
    '/dashboard/:path*',
    '/inventory/:path*',
    '/recipes/:path*',
    '/meal-planning/:path*',
    '/shopping/:path*',
    '/cooking/:path*',
  ],
};
```

## Unified Project Structure

```plaintext
imkitchen/
├── .github/                    # CI/CD workflows
│   └── workflows/
│       ├── ci.yaml
│       ├── deploy-staging.yaml
│       └── deploy-production.yaml
├── .next/                      # Next.js build output (ignored)
├── public/                     # Static assets
│   ├── icons/
│   ├── images/
│   ├── locales/               # Translation files
│   │   ├── en/
│   │   ├── es/
│   │   ├── fr/
│   │   └── de/
│   ├── manifest.json          # PWA manifest
│   └── sw.js                  # Service worker
├── src/
│   ├── app/                   # Next.js 14 App Router
│   │   ├── [locale]/          # Internationalized routes
│   │   │   ├── (auth)/        # Auth route group
│   │   │   │   ├── login/
│   │   │   │   └── register/
│   │   │   ├── dashboard/
│   │   │   │   └── page.tsx
│   │   │   ├── inventory/
│   │   │   │   ├── page.tsx
│   │   │   │   ├── add/
│   │   │   │   └── [itemId]/
│   │   │   ├── recipes/
│   │   │   │   ├── page.tsx
│   │   │   │   ├── search/
│   │   │   │   ├── [recipeId]/
│   │   │   │   └── favorites/
│   │   │   ├── meal-planning/
│   │   │   │   ├── page.tsx
│   │   │   │   └── calendar/
│   │   │   ├── shopping/
│   │   │   │   ├── page.tsx
│   │   │   │   └── [listId]/
│   │   │   ├── cooking/
│   │   │   │   └── [recipeId]/
│   │   │   └── settings/
│   │   ├── api/               # API routes
│   │   │   ├── auth/
│   │   │   │   ├── [...nextauth]/
│   │   │   │   └── register/
│   │   │   ├── inventory/
│   │   │   │   ├── route.ts
│   │   │   │   ├── [itemId]/
│   │   │   │   └── expiring/
│   │   │   ├── recipes/
│   │   │   │   ├── route.ts
│   │   │   │   ├── search/
│   │   │   │   ├── [recipeId]/
│   │   │   │   └── suggestions/
│   │   │   ├── meal-plans/
│   │   │   │   ├── route.ts
│   │   │   │   └── [planId]/
│   │   │   ├── shopping/
│   │   │   │   ├── route.ts
│   │   │   │   └── lists/
│   │   │   ├── voice/
│   │   │   │   ├── process/
│   │   │   │   └── cooking/
│   │   │   └── webhooks/
│   │   ├── globals.css        # Global styles
│   │   ├── layout.tsx         # Root layout
│   │   ├── loading.tsx        # Loading UI
│   │   ├── error.tsx          # Error UI
│   │   └── not-found.tsx      # 404 page
│   ├── components/            # React components
│   │   ├── ui/                # Base UI components
│   │   │   ├── button.tsx
│   │   │   ├── input.tsx
│   │   │   ├── modal.tsx
│   │   │   └── index.ts
│   │   ├── forms/             # Form components
│   │   │   ├── inventory-form.tsx
│   │   │   ├── recipe-form.tsx
│   │   │   └── login-form.tsx
│   │   ├── inventory/         # Inventory components
│   │   │   ├── inventory-list.tsx
│   │   │   ├── inventory-item.tsx
│   │   │   └── expiration-alert.tsx
│   │   ├── recipes/           # Recipe components
│   │   │   ├── recipe-card.tsx
│   │   │   ├── recipe-detail.tsx
│   │   │   └── recipe-search.tsx
│   │   ├── meal-planning/     # Meal planning components
│   │   │   ├── calendar-view.tsx
│   │   │   ├── meal-slot.tsx
│   │   │   └── drag-drop-provider.tsx
│   │   ├── shopping/          # Shopping components
│   │   │   ├── shopping-list.tsx
│   │   │   ├── shopping-item.tsx
│   │   │   └── store-categories.tsx
│   │   ├── cooking/           # Cooking mode components
│   │   │   ├── cooking-interface.tsx
│   │   │   ├── timer-manager.tsx
│   │   │   └── voice-controls.tsx
│   │   ├── voice/             # Voice interaction components
│   │   │   ├── voice-button.tsx
│   │   │   ├── voice-status.tsx
│   │   │   └── speech-recognition.tsx
│   │   └── layout/            # Layout components
│   │       ├── navigation.tsx
│   │       ├── sidebar.tsx
│   │       ├── header.tsx
│   │       └── footer.tsx
│   ├── hooks/                 # Custom React hooks
│   │   ├── use-auth.ts
│   │   ├── use-inventory.ts
│   │   ├── use-recipes.ts
│   │   ├── use-voice.ts
│   │   ├── use-timers.ts
│   │   └── use-local-storage.ts
│   ├── lib/                   # Utility functions and configurations
│   │   ├── auth.ts            # NextAuth configuration
│   │   ├── db.ts              # Database connection
│   │   ├── redis.ts           # Redis connection
│   │   ├── storage.ts         # File storage abstraction
│   │   ├── email.ts           # Email service abstraction
│   │   ├── voice.ts           # Voice processing utilities
│   │   ├── utils.ts           # General utilities
│   │   ├── constants.ts       # Application constants
│   │   ├── validators.ts      # Zod schemas
│   │   ├── api-client.ts      # API client
│   │   └── services/          # Business logic services
│   │       ├── inventory-service.ts
│   │       ├── recipe-service.ts
│   │       ├── meal-plan-service.ts
│   │       ├── shopping-service.ts
│   │       ├── voice-service.ts
│   │       └── notification-service.ts
│   ├── stores/                # State management
│   │   ├── auth-store.ts
│   │   ├── inventory-store.ts
│   │   ├── recipe-store.ts
│   │   ├── meal-plan-store.ts
│   │   ├── shopping-store.ts
│   │   ├── cooking-store.ts
│   │   ├── voice-store.ts
│   │   └── ui-store.ts
│   ├── types/                 # TypeScript type definitions
│   │   ├── auth.ts
│   │   ├── inventory.ts
│   │   ├── recipe.ts
│   │   ├── meal-plan.ts
│   │   ├── shopping.ts
│   │   ├── voice.ts
│   │   ├── api.ts
│   │   └── index.ts
│   └── middleware.ts          # Next.js middleware
├── prisma/                    # Database schema and migrations
│   ├── schema.prisma
│   ├── migrations/
│   └── seed.ts
├── tests/                     # Test files
│   ├── __mocks__/
│   ├── components/
│   ├── pages/
│   ├── api/
│   ├── e2e/
│   └── setup.ts
├── docker/                    # Docker configuration
│   ├── Dockerfile
│   ├── docker-compose.yml
│   ├── docker-compose.prod.yml
│   └── nginx.conf
├── docs/                      # Documentation
│   ├── prd.md
│   ├── front-end-spec.md
│   ├── architecture.md
│   ├── api-docs.md
│   └── deployment.md
├── scripts/                   # Build and deployment scripts
│   ├── build.sh
│   ├── deploy.sh
│   ├── backup.sh
│   └── seed-data.ts
├── .env.example               # Environment template
├── .env.local                 # Local development environment
├── .gitignore
├── .eslintrc.json
├── .prettierrc
├── tailwind.config.js
├── next.config.js
├── tsconfig.json
├── package.json
├── pnpm-lock.yaml
├── jest.config.js
├── playwright.config.ts
└── README.md
```

## Development Workflow

### Local Development Setup

#### Prerequisites

```bash
# Install Node.js 18+ and pnpm
curl -fsSL https://fnm.vercel.app/install | bash
fnm install 18
fnm use 18
npm install -g pnpm

# Install Docker and Docker Compose
# Follow official Docker installation for your OS

# Install PostgreSQL and Redis (via Docker)
docker --version
docker-compose --version
```

#### Initial Setup

```bash
# 1. REPOSITORY & DEPENDENCIES (Epic 1, Story 1.1)
git clone <repository-url>
cd imkitchen
pnpm install

# 2. ENVIRONMENT CONFIGURATION (Epic 1, Story 1.1 - BEFORE any services)
cp .env.example .env.local
# CRITICAL: Edit .env.local with your configuration including:
# - Database connection strings
# - External API keys (Spoonacular, OpenFoodFacts)
# - Authentication secrets
# - Email service configuration

# 3. INFRASTRUCTURE SERVICES (Epic 1, Story 1.2 - BEFORE database operations)
docker-compose up -d postgres redis

# 4. DATABASE SETUP (Epic 1, Story 1.2 - AFTER services running)
pnpm db:migrate
pnpm db:generate
pnpm db:seed

# 5. DEVELOPMENT SERVER (Epic 1, Story 1.6 - FINAL step)
pnpm dev
```

**⚠️ CRITICAL TIMING:** Environment variables must be configured before starting any services. Database must be running before migrations. All setup must complete before development server start.

#### Development Commands

```bash
# Start all services
pnpm dev

# Start frontend only (assumes API running elsewhere)
pnpm dev:frontend

# Start backend only (API routes and services)
pnpm dev:backend

# Run tests
pnpm test              # Unit tests
pnpm test:integration  # Integration tests
pnpm test:e2e          # End-to-end tests
pnpm test:watch        # Watch mode

# Database operations
pnpm db:migrate        # Run migrations
pnpm db:reset          # Reset database
pnpm db:seed           # Seed with test data
pnpm db:studio         # Open Prisma Studio

# Code quality
pnpm lint              # ESLint
pnpm type-check        # TypeScript check
pnpm format            # Prettier
```

### Environment Configuration

#### Required Environment Variables

```bash
# Frontend (.env.local)
NEXT_PUBLIC_APP_URL=http://localhost:3000
NEXT_PUBLIC_API_URL=http://localhost:3000/api
NEXT_PUBLIC_VOICE_API_KEY=your_voice_api_key
NEXT_PUBLIC_SENTRY_DSN=your_sentry_dsn

# Backend (.env)
DATABASE_URL=postgresql://user:password@localhost:5432/imkitchen
REDIS_URL=redis://localhost:6379
NEXTAUTH_SECRET=your_nextauth_secret
NEXTAUTH_URL=http://localhost:3000

# External APIs
SPOONACULAR_API_KEY=your_spoonacular_key
OPENAI_API_KEY=your_openai_key_for_voice
SENDGRID_API_KEY=your_sendgrid_key

# Storage
S3_BUCKET_NAME=imkitchen-uploads
S3_ACCESS_KEY_ID=your_s3_access_key
S3_SECRET_ACCESS_KEY=your_s3_secret_key
S3_REGION=us-east-1

# Shared
NODE_ENV=development
LOG_LEVEL=debug
```

## Deployment Architecture

### Deployment Strategy

**Frontend Deployment:**

- **Platform:** Docker containers with platform-agnostic deployment
- **Build Command:** `pnpm build`
- **Output Directory:** `.next/`
- **CDN/Edge:** CloudFlare for global content delivery and edge caching

**Backend Deployment:**

- **Platform:** Same Docker container (Next.js full-stack)
- **Build Command:** `pnpm build`
- **Deployment Method:** Blue-green deployment with health checks

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
        image: postgres:15
        env:
          POSTGRES_PASSWORD: postgres
          POSTGRES_DB: imkitchen_test
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
      redis:
        image: redis:7
        options: >-
          --health-cmd "redis-cli ping"
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5

    steps:
      - uses: actions/checkout@v4
      - uses: pnpm/action-setup@v2
        with:
          version: 8
      - uses: actions/setup-node@v4
        with:
          node-version: '18'
          cache: 'pnpm'

      - name: Install dependencies
        run: pnpm install --frozen-lockfile

      - name: Type check
        run: pnpm type-check

      - name: Lint
        run: pnpm lint

      - name: Run unit tests
        run: pnpm test
        env:
          DATABASE_URL: postgresql://postgres:postgres@localhost:5432/imkitchen_test
          REDIS_URL: redis://localhost:6379

      - name: Run integration tests
        run: pnpm test:integration
        env:
          DATABASE_URL: postgresql://postgres:postgres@localhost:5432/imkitchen_test
          REDIS_URL: redis://localhost:6379

      - name: Build application
        run: pnpm build
        env:
          DATABASE_URL: postgresql://postgres:postgres@localhost:5432/imkitchen_test

  deploy-staging:
    needs: test
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/develop'

    steps:
      - uses: actions/checkout@v4

      - name: Build Docker image
        run: docker build -t imkitchen:staging .

      - name: Deploy to staging
        run: |
          # Platform-specific deployment commands
          # Could be AWS ECS, GCP Cloud Run, Azure Container Instances, etc.
          echo "Deploying to staging environment"

  deploy-production:
    needs: test
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/main'

    steps:
      - uses: actions/checkout@v4

      - name: Build Docker image
        run: docker build -t imkitchen:production .

      - name: Deploy to production
        run: |
          # Production deployment with blue-green strategy
          echo "Deploying to production environment"
```

### Environments

| Environment | Frontend URL                  | Backend URL                       | Purpose                |
| ----------- | ----------------------------- | --------------------------------- | ---------------------- |
| Development | http://localhost:3000         | http://localhost:3000/api         | Local development      |
| Staging     | https://staging.imkitchen.com | https://staging.imkitchen.com/api | Pre-production testing |
| Production  | https://app.imkitchen.com     | https://app.imkitchen.com/api     | Live environment       |

## Security and Performance

### Security Requirements

**Frontend Security:**

- CSP Headers: `default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:; connect-src 'self' https://api.spoonacular.com;`
- XSS Prevention: Content Security Policy, sanitized user inputs, secure React patterns
- Secure Storage: Sensitive data in HTTP-only cookies, non-sensitive in sessionStorage with encryption

**Backend Security:**

- Input Validation: Zod schemas for all API inputs, SQL injection prevention via Prisma ORM
- Rate Limiting: 100 requests/minute per user, 1000 requests/hour per household
- CORS Policy: Restricted to known frontend domains with credentials support

**Authentication Security:**

- Token Storage: HTTP-only cookies for auth tokens, secure flag enabled
- Session Management: NextAuth.js with database sessions, 24-hour expiry with refresh
- Password Policy: Minimum 8 characters, bcrypt hashing with 12 rounds

### Performance Optimization

**Frontend Performance:**

- Bundle Size Target: <500KB initial load, <200KB per route
- Loading Strategy: Route-based code splitting, component lazy loading, image optimization
- Caching Strategy: Service worker for recipes, localStorage for user preferences, CDN for static assets

**Backend Performance:**

- Response Time Target: <200ms for API routes, <100ms for cached responses
- Database Optimization: Proper indexing, query optimization, connection pooling with Prisma
- Caching Strategy: Redis for session data, recipe search results, and frequently accessed inventory

## Testing Strategy

### Testing Pyramid

```text
        E2E Tests (Few)
       /              \
    Integration Tests (Some)
   /                      \
Frontend Unit Tests    Backend Unit Tests
    (Many)                 (Many)
```

### Test Organization

#### Frontend Tests

```text
tests/
├── components/
│   ├── ui/
│   │   ├── button.test.tsx
│   │   └── input.test.tsx
│   ├── inventory/
│   │   ├── inventory-list.test.tsx
│   │   └── inventory-item.test.tsx
│   └── cooking/
│       ├── cooking-interface.test.tsx
│       └── timer-manager.test.tsx
├── hooks/
│   ├── use-auth.test.ts
│   ├── use-inventory.test.ts
│   └── use-voice.test.ts
├── services/
│   ├── inventory-service.test.ts
│   └── recipe-service.test.ts
└── utils/
    ├── api-client.test.ts
    └── validators.test.ts
```

#### Backend Tests

```text
tests/
├── api/
│   ├── auth/
│   │   ├── login.test.ts
│   │   └── register.test.ts
│   ├── inventory/
│   │   ├── get-items.test.ts
│   │   └── create-item.test.ts
│   └── recipes/
│       ├── search.test.ts
│       └── suggestions.test.ts
├── services/
│   ├── inventory-service.test.ts
│   ├── recipe-service.test.ts
│   └── voice-service.test.ts
├── repositories/
│   ├── inventory-repository.test.ts
│   └── recipe-repository.test.ts
└── middleware/
    ├── auth-middleware.test.ts
    └── error-handler.test.ts
```

#### E2E Tests

```text
tests/e2e/
├── auth/
│   ├── login.spec.ts
│   └── registration.spec.ts
├── inventory/
│   ├── add-items.spec.ts
│   ├── expiration-alerts.spec.ts
│   └── item-management.spec.ts
├── recipes/
│   ├── search-recipes.spec.ts
│   ├── save-favorites.spec.ts
│   └── ingredient-suggestions.spec.ts
├── meal-planning/
│   ├── create-meal-plan.spec.ts
│   ├── drag-drop-recipes.spec.ts
│   └── family-coordination.spec.ts
├── shopping/
│   ├── generate-list.spec.ts
│   ├── mark-purchased.spec.ts
│   └── store-organization.spec.ts
├── cooking/
│   ├── cooking-mode.spec.ts
│   ├── timer-management.spec.ts
│   └── voice-commands.spec.ts
└── voice/
    ├── voice-navigation.spec.ts
    └── hands-free-cooking.spec.ts
```

### Test Examples

#### Frontend Component Test

```typescript
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { InventoryList } from '@/components/inventory/inventory-list';
import { InventoryService } from '@/lib/services/inventory-service';

// Mock the service
jest.mock('@/lib/services/inventory-service');
const mockInventoryService = InventoryService as jest.Mocked<typeof InventoryService>;

const createWrapper = () => {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
    },
  });

  return ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client={queryClient}>
      {children}
    </QueryClientProvider>
  );
};

describe('InventoryList', () => {
  const mockItems = [
    {
      id: '1',
      name: 'Tomatoes',
      quantity: 3,
      unit: 'pieces',
      category: 'vegetables',
      location: 'refrigerator',
      expirationDate: new Date('2025-09-20'),
    },
    {
      id: '2',
      name: 'Milk',
      quantity: 1,
      unit: 'liters',
      category: 'dairy',
      location: 'refrigerator',
      expirationDate: new Date('2025-09-16'), // Expiring soon
    },
  ];

  beforeEach(() => {
    mockInventoryService.getItems.mockResolvedValue(mockItems);
  });

  afterEach(() => {
    jest.clearAllMocks();
  });

  it('renders inventory items correctly', async () => {
    render(<InventoryList />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByText('Tomatoes')).toBeInTheDocument();
      expect(screen.getByText('Milk')).toBeInTheDocument();
    });
  });

  it('highlights items expiring soon', async () => {
    render(<InventoryList />, { wrapper: createWrapper() });

    await waitFor(() => {
      const milkItem = screen.getByTestId('inventory-item-2');
      expect(milkItem).toHaveClass('bg-yellow-50'); // Warning background
    });
  });

  it('filters items by location', async () => {
    render(<InventoryList />, { wrapper: createWrapper() });

    const fridgeFilter = screen.getByText('Refrigerator');
    fireEvent.click(fridgeFilter);

    await waitFor(() => {
      expect(mockInventoryService.getItems).toHaveBeenCalledWith({
        location: 'refrigerator'
      });
    });
  });

  it('handles voice command for adding items', async () => {
    const mockVoiceCommand = jest.fn();
    render(<InventoryList onVoiceCommand={mockVoiceCommand} />, {
      wrapper: createWrapper()
    });

    const voiceButton = screen.getByLabelText('Voice add item');
    fireEvent.click(voiceButton);

    // Simulate voice recognition result
    fireEvent(window, new CustomEvent('voiceresult', {
      detail: { transcript: 'add 2 pounds of chicken to refrigerator' }
    }));

    await waitFor(() => {
      expect(mockVoiceCommand).toHaveBeenCalledWith(
        expect.objectContaining({
          action: 'add_item',
          item: expect.objectContaining({
            name: 'chicken',
            quantity: 2,
            unit: 'pounds',
            location: 'refrigerator'
          })
        })
      );
    });
  });
});
```

#### Backend API Test

```typescript
import { NextRequest } from 'next/server';
import { GET, POST } from '@/app/api/inventory/route';
import { InventoryService } from '@/lib/services/inventory-service';
import { auth } from '@/lib/auth';

// Mock dependencies
jest.mock('@/lib/services/inventory-service');
jest.mock('@/lib/auth');

const mockInventoryService = InventoryService as jest.Mocked<
  typeof InventoryService
>;
const mockAuth = auth as jest.MockedFunction<typeof auth>;

describe('/api/inventory', () => {
  const mockUser = {
    id: 'user-123',
    householdId: 'household-456',
    email: 'test@example.com',
  };

  beforeEach(() => {
    mockAuth.mockResolvedValue({
      user: mockUser,
      expires: new Date(Date.now() + 3600000).toISOString(),
    });
  });

  afterEach(() => {
    jest.clearAllMocks();
  });

  describe('GET /api/inventory', () => {
    it('returns inventory items for authenticated user', async () => {
      const mockItems = [
        {
          id: '1',
          name: 'Tomatoes',
          quantity: 3,
          unit: 'pieces',
          category: 'vegetables',
          location: 'refrigerator',
          householdId: 'household-456',
        },
      ];

      mockInventoryService.getHouseholdItems.mockResolvedValue(mockItems);

      const request = new NextRequest('http://localhost:3000/api/inventory');
      const response = await GET(request);
      const data = await response.json();

      expect(response.status).toBe(200);
      expect(data).toEqual(mockItems);
      expect(mockInventoryService.getHouseholdItems).toHaveBeenCalledWith(
        'household-456',
        {}
      );
    });

    it('filters items by location', async () => {
      mockInventoryService.getHouseholdItems.mockResolvedValue([]);

      const request = new NextRequest(
        'http://localhost:3000/api/inventory?location=pantry'
      );
      const response = await GET(request);

      expect(response.status).toBe(200);
      expect(mockInventoryService.getHouseholdItems).toHaveBeenCalledWith(
        'household-456',
        { location: 'pantry' }
      );
    });

    it('returns 401 for unauthenticated requests', async () => {
      mockAuth.mockResolvedValue(null);

      const request = new NextRequest('http://localhost:3000/api/inventory');
      const response = await GET(request);

      expect(response.status).toBe(401);
    });
  });

  describe('POST /api/inventory', () => {
    it('creates new inventory item', async () => {
      const newItem = {
        name: 'Chicken Breast',
        quantity: 2,
        unit: 'pounds',
        category: 'proteins',
        location: 'refrigerator',
        expirationDate: '2025-09-20',
      };

      const createdItem = {
        id: 'item-789',
        ...newItem,
        expirationDate: new Date('2025-09-20'),
        householdId: 'household-456',
        addedBy: 'user-123',
      };

      mockInventoryService.createItem.mockResolvedValue(createdItem);

      const request = new NextRequest('http://localhost:3000/api/inventory', {
        method: 'POST',
        body: JSON.stringify(newItem),
        headers: { 'Content-Type': 'application/json' },
      });

      const response = await POST(request);
      const data = await response.json();

      expect(response.status).toBe(201);
      expect(data).toEqual(createdItem);
      expect(mockInventoryService.createItem).toHaveBeenCalledWith({
        ...newItem,
        expirationDate: new Date('2025-09-20'),
        householdId: 'household-456',
        addedBy: 'user-123',
      });
    });

    it('validates request data', async () => {
      const invalidItem = {
        name: '', // Invalid: empty name
        quantity: -1, // Invalid: negative quantity
        unit: 'pounds',
        category: 'invalid-category', // Invalid: not in enum
        location: 'refrigerator',
      };

      const request = new NextRequest('http://localhost:3000/api/inventory', {
        method: 'POST',
        body: JSON.stringify(invalidItem),
        headers: { 'Content-Type': 'application/json' },
      });

      const response = await POST(request);
      const data = await response.json();

      expect(response.status).toBe(400);
      expect(data.error).toContain('validation');
    });
  });
});
```

#### E2E Test

```typescript
import { test, expect } from '@playwright/test';

test.describe('Cooking Mode', () => {
  test.beforeEach(async ({ page }) => {
    // Login user
    await page.goto('/login');
    await page.fill('[data-testid=email]', 'test@example.com');
    await page.fill('[data-testid=password]', 'password123');
    await page.click('[data-testid=login-button]');
    await expect(page).toHaveURL('/dashboard');
  });

  test('should start cooking mode and navigate through recipe steps', async ({
    page,
  }) => {
    // Navigate to recipe
    await page.goto('/recipes/classic-spaghetti-carbonara');

    // Start cooking mode
    await page.click('[data-testid=start-cooking]');
    await expect(page).toHaveURL('/cooking/classic-spaghetti-carbonara');

    // Verify cooking interface
    await expect(page.locator('[data-testid=cooking-step]')).toContainText(
      'Step 1'
    );
    await expect(page.locator('[data-testid=step-instruction]')).toContainText(
      'Bring a large pot of salted water to boil'
    );

    // Check ingredients panel
    await expect(page.locator('[data-testid=ingredients-panel]')).toBeVisible();
    await expect(page.locator('[data-testid=ingredient-item]')).toHaveCount(6);

    // Navigate to next step
    await page.click('[data-testid=next-step]');
    await expect(page.locator('[data-testid=cooking-step]')).toContainText(
      'Step 2'
    );

    // Test timer functionality
    await page.click('[data-testid=start-timer]');
    await expect(page.locator('[data-testid=active-timer]')).toBeVisible();
    await expect(page.locator('[data-testid=timer-display]')).toContainText(
      '10:00'
    );

    // Test voice commands (if supported)
    const voiceButton = page.locator('[data-testid=voice-button]');
    if (await voiceButton.isVisible()) {
      await voiceButton.click();
      await expect(page.locator('[data-testid=voice-status]')).toContainText(
        'Listening'
      );

      // Simulate voice command (this would be integration with actual voice API in real test)
      await page.evaluate(() => {
        window.dispatchEvent(
          new CustomEvent('voicecommand', {
            detail: { command: 'next step' },
          })
        );
      });

      await expect(page.locator('[data-testid=cooking-step]')).toContainText(
        'Step 3'
      );
    }

    // Complete recipe
    await page.click('[data-testid=next-step]'); // Step 3
    await page.click('[data-testid=next-step]'); // Step 4
    await page.click('[data-testid=next-step]'); // Step 5
    await page.click('[data-testid=complete-recipe]');

    // Verify completion screen
    await expect(
      page.locator('[data-testid=completion-message]')
    ).toContainText('Recipe completed!');
    await expect(page.locator('[data-testid=rating-prompt]')).toBeVisible();

    // Rate recipe
    await page.click('[data-testid=rating-star-5]');
    await page.fill(
      '[data-testid=review-text]',
      'Delicious and easy to follow!'
    );
    await page.click('[data-testid=submit-rating]');

    // Verify redirect to recipe page
    await expect(page).toHaveURL('/recipes/classic-spaghetti-carbonara');
    await expect(page.locator('[data-testid=user-rating]')).toContainText(
      '5 stars'
    );
  });

  test('should handle offline cooking mode', async ({ page, context }) => {
    // Start cooking mode while online
    await page.goto('/recipes/classic-spaghetti-carbonara');
    await page.click('[data-testid=start-cooking]');

    // Verify recipe data is cached
    await expect(page.locator('[data-testid=cooking-step]')).toContainText(
      'Step 1'
    );

    // Go offline
    await context.setOffline(true);

    // Verify cooking mode still works
    await page.click('[data-testid=next-step]');
    await expect(page.locator('[data-testid=cooking-step]')).toContainText(
      'Step 2'
    );

    // Start timer offline
    await page.click('[data-testid=start-timer]');
    await expect(page.locator('[data-testid=active-timer]')).toBeVisible();

    // Verify offline indicator
    await expect(page.locator('[data-testid=offline-indicator]')).toBeVisible();

    // Go back online
    await context.setOffline(false);

    // Verify sync happens
    await expect(
      page.locator('[data-testid=offline-indicator]')
    ).not.toBeVisible();
  });

  test('should support voice commands during cooking', async ({ page }) => {
    // Grant microphone permissions (in real test environment)
    await page.context().grantPermissions(['microphone']);

    await page.goto('/cooking/classic-spaghetti-carbonara');

    // Test voice activation
    await page.click('[data-testid=voice-button]');
    await expect(page.locator('[data-testid=voice-status]')).toContainText(
      'Listening'
    );

    // Simulate various voice commands
    const commands = [
      { command: 'next step', expectedAction: 'step advancement' },
      { command: 'previous step', expectedAction: 'step regression' },
      { command: 'set timer for 5 minutes', expectedAction: 'timer creation' },
      { command: 'pause timer', expectedAction: 'timer pause' },
      { command: 'repeat instructions', expectedAction: 'instruction repeat' },
    ];

    for (const { command, expectedAction } of commands) {
      await page.evaluate(cmd => {
        window.dispatchEvent(
          new CustomEvent('voicecommand', {
            detail: { command: cmd },
          })
        );
      }, command);

      // Verify appropriate action was taken
      // (This would be more specific based on actual implementation)
      await expect(page.locator('[data-testid=voice-feedback]')).toContainText(
        'Command processed'
      );
    }
  });
});
```

## Coding Standards

### Critical Fullstack Rules

- **Type Sharing:** Always define shared types in `src/types/` and import from there - prevents frontend/backend interface mismatches
- **API Client Usage:** Never make direct fetch calls - use the centralized API client service for consistent error handling and authentication
- **Environment Variables:** Access only through config objects in `src/lib/config.ts`, never process.env directly - enables proper validation and fallbacks
- **Error Handling:** All API routes must use the standard error handler middleware for consistent error responses and logging
- **Database Access:** Only access database through service layer, never direct Prisma calls in API routes - enables caching and business logic separation
- **Authentication Checks:** Always verify authentication in API routes using middleware, never skip auth validation
- **Voice Command Structure:** Voice commands must follow the predefined schema in `src/types/voice.ts` for consistency
- **Cache Invalidation:** Update relevant cache keys when data changes, use the cache service for coordinated invalidation
- **File Uploads:** Use the storage abstraction layer, never direct cloud provider SDKs - maintains vendor independence
- **State Updates:** Use proper state management patterns, never directly mutate state objects

### Naming Conventions

| Element               | Frontend             | Backend              | Example                   |
| --------------------- | -------------------- | -------------------- | ------------------------- |
| Components            | PascalCase           | -                    | `InventoryList.tsx`       |
| Hooks                 | camelCase with 'use' | -                    | `useInventoryItems.ts`    |
| API Routes            | -                    | kebab-case           | `/api/meal-plans`         |
| Database Tables       | -                    | snake_case           | `inventory_items`         |
| Services              | PascalCase + Service | PascalCase + Service | `InventoryService`        |
| Types/Interfaces      | PascalCase           | PascalCase           | `InventoryItem`           |
| Constants             | SCREAMING_SNAKE_CASE | SCREAMING_SNAKE_CASE | `MAX_RECIPE_TITLE_LENGTH` |
| Environment Variables | SCREAMING_SNAKE_CASE | SCREAMING_SNAKE_CASE | `DATABASE_URL`            |

## Error Handling Strategy

### Error Flow

```mermaid
sequenceDiagram
    participant Frontend
    participant APIRoute
    participant Service
    participant Database
    participant ErrorHandler
    participant Logger
    participant User

    Frontend->>APIRoute: Request with invalid data
    APIRoute->>Service: Process request
    Service->>Database: Database operation
    Database-->>Service: Database error
    Service-->>ErrorHandler: Throw ServiceError
    ErrorHandler->>Logger: Log error details
    ErrorHandler->>APIRoute: Formatted error response
    APIRoute-->>Frontend: HTTP error response
    Frontend->>User: User-friendly error message
```

### Error Response Format

```typescript
interface ApiError {
  error: {
    code: string;
    message: string;
    details?: Record<string, any>;
    timestamp: string;
    requestId: string;
  };
}
```

### Frontend Error Handling

```typescript
import { toast } from '@/components/ui/toast';
import { ApiError } from '@/types/api';

export class ErrorHandler {
  static handle(error: unknown, context?: string) {
    console.error(`Error in ${context}:`, error);

    if (error instanceof ApiError) {
      return this.handleApiError(error);
    }

    if (error instanceof ValidationError) {
      return this.handleValidationError(error);
    }

    if (error instanceof NetworkError) {
      return this.handleNetworkError(error);
    }

    // Generic error handling
    toast.error('An unexpected error occurred. Please try again.');
    return {
      title: 'Error',
      message: 'An unexpected error occurred',
      type: 'error' as const,
    };
  }

  private static handleApiError(error: ApiError) {
    const userMessage = this.getUserFriendlyMessage(error.code);
    toast.error(userMessage);

    return {
      title: 'Error',
      message: userMessage,
      type: 'error' as const,
      code: error.code,
    };
  }

  private static getUserFriendlyMessage(errorCode: string): string {
    const messages: Record<string, string> = {
      INVENTORY_ITEM_NOT_FOUND:
        "The inventory item you're looking for doesn't exist.",
      RECIPE_NOT_FOUND: 'This recipe is no longer available.',
      MEAL_PLAN_CONFLICT:
        "There's a conflict with your meal plan. Please check your scheduled meals.",
      VOICE_COMMAND_NOT_RECOGNIZED:
        "I didn't understand that command. Please try again.",
      INSUFFICIENT_INGREDIENTS:
        "You don't have enough ingredients for this recipe.",
      HOUSEHOLD_ACCESS_DENIED:
        "You don't have permission to access this household's data.",
      EXPIRED_SESSION: 'Your session has expired. Please log in again.',
      RATE_LIMIT_EXCEEDED:
        'Too many requests. Please wait a moment and try again.',
    };

    return messages[errorCode] || 'Something went wrong. Please try again.';
  }
}

// Usage in components
export const useErrorHandler = () => {
  return (error: unknown, context?: string) => {
    return ErrorHandler.handle(error, context);
  };
};
```

### Backend Error Handling

```typescript
import { NextRequest, NextResponse } from 'next/server';
import { v4 as uuidv4 } from 'uuid';
import { logger } from '@/lib/logger';

export class ServiceError extends Error {
  constructor(
    message: string,
    public code: string,
    public statusCode: number = 500,
    public details?: Record<string, any>
  ) {
    super(message);
    this.name = 'ServiceError';
  }
}

export class ValidationError extends Error {
  constructor(
    message: string,
    public field: string,
    public value?: any
  ) {
    super(message);
    this.name = 'ValidationError';
  }
}

export function withErrorHandler<T>(
  handler: () => Promise<T>
): Promise<T | NextResponse> {
  return handler().catch((error: unknown) => {
    const requestId = uuidv4();
    const timestamp = new Date().toISOString();

    logger.error('API Error', {
      requestId,
      timestamp,
      error: error instanceof Error ? error.message : 'Unknown error',
      stack: error instanceof Error ? error.stack : undefined,
    });

    if (error instanceof ServiceError) {
      return NextResponse.json(
        {
          error: {
            code: error.code,
            message: error.message,
            details: error.details,
            timestamp,
            requestId,
          },
        },
        { status: error.statusCode }
      );
    }

    if (error instanceof ValidationError) {
      return NextResponse.json(
        {
          error: {
            code: 'VALIDATION_ERROR',
            message: error.message,
            details: { field: error.field, value: error.value },
            timestamp,
            requestId,
          },
        },
        { status: 400 }
      );
    }

    // Generic error response
    return NextResponse.json(
      {
        error: {
          code: 'INTERNAL_SERVER_ERROR',
          message: 'An internal server error occurred',
          timestamp,
          requestId,
        },
      },
      { status: 500 }
    );
  });
}

// Usage in API routes
export async function POST(request: NextRequest) {
  return withErrorHandler(async () => {
    // Your API logic here
    const data = await request.json();

    if (!data.name) {
      throw new ValidationError('Name is required', 'name', data.name);
    }

    const result = await SomeService.create(data);
    return NextResponse.json(result);
  });
}
```

## Monitoring and Observability

### Monitoring Stack

- **Frontend Monitoring:** Sentry for error tracking, Vercel Analytics for performance metrics, LogRocket for session replay
- **Backend Monitoring:** Sentry for error tracking, custom metrics via Winston logging, database query monitoring via Prisma
- **Error Tracking:** Centralized error tracking with Sentry, real-time alerts for critical errors
- **Performance Monitoring:** Core Web Vitals tracking, API response time monitoring, cooking mode performance metrics

### Key Metrics

**Frontend Metrics:**

- Core Web Vitals (LCP, FID, CLS)
- JavaScript errors and crash rates
- API response times from user perspective
- Voice command success rates
- Cooking mode session completion rates

**Backend Metrics:**

- Request rate and error rate per endpoint
- Database query performance and slow query detection
- External API integration response times and failure rates
- Voice processing accuracy and response times
- User engagement metrics (recipes cooked, meal plans created)

**Kitchen-Specific Performance Metrics:**

- **Cooking Mode Performance:**
  - Step navigation response time (<100ms target)
  - Timer accuracy and synchronization across devices
  - Voice command recognition latency in kitchen environments
  - Offline mode transition success rates
- **Inventory Management Efficiency:**
  - Barcode scanning success rates and speed
  - Inventory update propagation time across household members
  - Expiration alert delivery accuracy and timing
- **Meal Planning Workflow:**
  - Recipe suggestion generation time based on inventory
  - Drag-and-drop interface responsiveness on mobile devices
  - Shopping list generation speed for complex meal plans
- **Real-Time Synchronization:**
  - WebSocket connection stability during cooking sessions
  - Family coordination update latency
  - Conflict resolution success rates for shared meal plans

**Performance Alerting Thresholds:**

- Voice command response time >500ms (kitchen usability impact)
- Recipe search results >2 seconds (user experience degradation)
- Shopping list generation >5 seconds (workflow interruption)
- Database queries >1 second (performance bottleneck)
- External API failures >5% (service reliability concern)

## Checklist Results Report

_Architecture checklist validation will be performed to ensure comprehensive coverage of all technical requirements and best practices before finalization._

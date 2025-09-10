# imkitchen Fullstack Architecture Document

## Introduction

This document outlines the complete fullstack architecture for **imkitchen**, including backend systems, frontend implementation, and their integration. It serves as the single source of truth for AI-driven development, ensuring consistency across the entire technology stack.

This unified approach combines what would traditionally be separate backend and frontend architecture documents, streamlining the development process for modern fullstack applications where these concerns are increasingly intertwined.

### Starter Template or Existing Project

Based on the PRD technical assumptions, **imkitchen** is a greenfield project with specific technology preferences:

- **Frontend Framework:** Lynx.js for cross-platform mobile development (as specified in PRD)
- **Backend Technology:** Go-based API services for performance-critical scheduling algorithms
- **Database:** PostgreSQL + Redis for relational data and caching
- **Mobile-First Approach:** Cross-platform targeting iOS/Android with responsive web interface

**No existing starter template** is prescribed, making this a custom architecture design. However, I recommend evaluating:

1. **Lynx.js Project Templates** - If available, for rapid cross-platform mobile setup
2. **Go Web API Starters** - Popular frameworks like Gin, Echo, or Fiber with PostgreSQL integration
3. **Monorepo Templates** - Nx or Turborepo configurations for managing mobile + API + shared packages

**Decision:** Custom architecture without relying on specific starters, allowing optimal technology integration for the unique "Fill My Week" automation requirements.

### Change Log
| Date | Version | Description | Author |
|------|---------|-------------|---------|
| 2025-09-06 | 1.0 | Initial fullstack architecture based on PRD and frontend specifications | Winston (Architect) |

## High Level Architecture

### Technical Summary

**imkitchen** implements a modern hybrid architecture combining mobile-first cross-platform development with high-performance backend services. The Lynx.js frontend delivers native mobile experiences on iOS/Android plus responsive web, while Go-based microservices handle performance-critical meal planning algorithms and recipe management. PostgreSQL provides robust relational data storage with Redis caching to achieve the 2-second "Fill My Week" generation requirement. The system deploys as a cloud-native architecture supporting horizontal scaling for community features, with clear API boundaries enabling independent frontend/backend development and deployment.

### Platform and Infrastructure Choice

Based on the PRD's mobile-first requirements, performance targets, and scalability needs, I present three viable platform options:

**Option 1: AWS Full Stack** (Recommended)
- **Pros:** Mature ecosystem, excellent Go Lambda support, PostgreSQL RDS, Redis ElastiCache, comprehensive mobile app deployment
- **Cons:** Higher complexity, potential vendor lock-in, learning curve for team
- **Key Services:** ECS/Lambda, RDS PostgreSQL, ElastiCache Redis, API Gateway, CloudFront CDN

**Option 2: Vercel + Supabase**
- **Pros:** Rapid development, excellent DX, built-in auth, real-time features
- **Cons:** Limited Go support, newer platform, potential scaling limitations
- **Key Services:** Vercel hosting, Supabase PostgreSQL, built-in auth/storage

**Option 3: Google Cloud Platform**
- **Pros:** Strong mobile analytics, excellent Go support, competitive pricing
- **Cons:** Smaller ecosystem than AWS, less Lynx.js community examples
- **Key Services:** Cloud Run, Cloud SQL, Memorystore Redis, Cloud CDN

**Recommendation:** AWS Full Stack for production-grade requirements, mature Go ecosystem, and comprehensive mobile app deployment support.

**Platform:** AWS  
**Key Services:** ECS (Go services), RDS PostgreSQL, ElastiCache Redis, API Gateway, CloudFront, S3 (recipe images)  
**Deployment Host and Regions:** us-east-1 (primary), us-west-2 (failover), eu-west-1 (international expansion)

### Repository Structure

**Structure:** Monorepo with clear package boundaries for mobile, API, and shared components  
**Monorepo Tool:** Nx for comprehensive build orchestration, dependency management, and cross-platform code sharing  
**Package Organization:** Apps (mobile, web, api), packages (shared-types, ui-components, recipe-utils), tools (build-scripts, deployment-configs)

### High Level Architecture Diagram

```mermaid
graph TD
    A[Mobile Apps - Lynx.js] --> B[CloudFront CDN]
    C[Web App - Lynx.js] --> B
    B --> D[API Gateway]
    D --> E[Go Microservices - ECS]
    E --> F[PostgreSQL - RDS]
    E --> G[Redis Cache - ElastiCache]
    E --> H[S3 Recipe Images]
    E --> I[External Recipe APIs]
    J[Community Features] --> E
    K[Push Notifications] --> A
    L[Analytics] --> M[CloudWatch]
    E --> L
```

### Architectural Patterns

- **Microservices Architecture:** Separate services for meal planning, recipe management, user management, and community features - _Rationale:_ Enables independent scaling of performance-critical meal planning algorithms while maintaining development velocity

- **API Gateway Pattern:** Single entry point with authentication, rate limiting, and routing to appropriate microservices - _Rationale:_ Centralizes cross-cutting concerns and provides clean abstraction for mobile clients

- **Repository Pattern:** Abstract data access layer with interface-driven design - _Rationale:_ Enables comprehensive testing and future database optimization without business logic changes

- **CQRS (Command Query Responsibility Segregation):** Separate read/write models for recipe data and meal planning - _Rationale:_ Optimizes read performance for recipe browsing while maintaining write consistency for meal plan generation

- **Event-Driven Architecture:** Asynchronous processing for meal plan generation and community notifications - _Rationale:_ Supports sub-2-second response times by offloading heavy computations

- **Component-Based Frontend:** Lynx.js components with shared design system across mobile and web - _Rationale:_ Maximizes code reuse while maintaining platform-specific optimizations

## Tech Stack

### Technology Stack Table

| Category | Technology | Version | Purpose | Rationale |
|----------|------------|---------|---------|-----------|
| Frontend Language | TypeScript | 5.x | Type-safe mobile/web development | Enables shared types across mobile/web, reduces runtime errors in automation workflows |
| Frontend Framework | Lynx.js | Latest | Cross-platform mobile + web | PRD requirement, single codebase for iOS/Android/Web with native performance |
| UI Component Library | Custom Design System | - | imkitchen-specific components | Supports unique "Fill My Week" automation UI and warm kitchen aesthetic |
| State Management | Zustand | 4.x | Lightweight React state | Simple, performant state management for mobile contexts, supports automation workflows |
| Backend Language | Go | 1.21+ | High-performance API services | Optimal for 2-second meal plan generation requirement and rotation algorithms |
| Backend Framework | Gin | 1.9+ | Fast HTTP web framework | Proven performance, excellent middleware ecosystem, cloud-agnostic |
| API Style | REST | - | HTTP-based APIs | Simple, cacheable, excellent mobile network handling, broad tooling support |
| Database | PostgreSQL | 15+ | Primary relational database | ACID compliance for recipe/user data, excellent Go integration, multi-cloud support |
| Cache | Redis | 7.x | High-performance caching | Essential for 2-second generation target, session storage, rate limiting |
| File Storage | MinIO (S3-compatible) | Latest | Recipe images and assets | Self-hosted S3-compatible storage, multi-cloud deployment, cost control |
| Authentication | Supabase Auth | Latest | User auth and management | Open-source, self-hostable, social login support, excellent mobile SDKs |
| Frontend Testing | Vitest + Testing Library | Latest | Component and unit testing | Fast, modern testing for Lynx.js components and automation logic |
| Backend Testing | Go Testing + Testify | Latest | API and service testing | Built-in Go testing with assertions, perfect for algorithm validation |
| E2E Testing | Playwright | 1.x | Cross-platform automation | Mobile and web testing, critical for "Fill My Week" workflow validation |
| Build Tool | Nx | 17+ | Monorepo orchestration | Shared code between mobile/API, efficient builds, TypeScript integration |
| Bundler | Vite | 5.x | Fast frontend builds | Modern bundling for Lynx.js, excellent development experience |
| IaC Tool | Terraform | 1.6+ | Infrastructure as code | Cloud-agnostic, mature ecosystem, supports multiple providers |
| CI/CD | GitHub Actions | - | Automated testing and deployment | Free tier, multi-cloud deployment support, excellent community actions |
| Monitoring | Grafana + Prometheus | Latest | Application monitoring | Open-source, cloud-agnostic, excellent Go metrics, customizable dashboards |
| Logging | Loki + Promtail | Latest | Centralized logging | Cloud-agnostic log aggregation, integrates with Grafana, cost-effective |
| CSS Framework | Tailwind CSS | 3.x | Utility-first styling | Rapid UI development, excellent mobile responsiveness, design system support |

### Platform Strategy

**Platform:** Multi-Cloud Strategy (Cloud-Agnostic)
- **Primary:** Digital Ocean or Hetzner Cloud for cost-effective, developer-friendly hosting
- **Containerization:** Docker + Kubernetes for maximum portability
- **CDN:** Cloudflare for provider-agnostic performance and security
- **Database:** Managed PostgreSQL (provider-agnostic) with automated backups

**Key Benefits:** No vendor lock-in, cost control, deployment flexibility, open-source ecosystem

## Data Models

### User

**Purpose:** Central user entity supporting authentication, preferences, and personalized meal planning automation

**Key Attributes:**
- id: UUID - Primary identifier for cross-service references
- email: string - Authentication and communication
- displayName: string - User interface personalization  
- dietaryRestrictions: string[] - Array of dietary constraints for meal planning
- cookingSkillLevel: enum - Beginner, Intermediate, Advanced for complexity filtering
- weeklyAvailability: JSON - Time availability patterns for intelligent scheduling
- preferredMealComplexity: enum - Simple, Moderate, Complex for automation preferences
- rotationResetCount: integer - Tracking recipe rotation cycles

#### TypeScript Interface

```typescript
interface User {
  id: string;
  email: string;
  displayName: string;
  dietaryRestrictions: string[];
  cookingSkillLevel: 'beginner' | 'intermediate' | 'advanced';
  weeklyAvailability: {
    [day: string]: {
      morningEnergy: 'low' | 'medium' | 'high';
      availableTime: number; // minutes
    };
  };
  preferredMealComplexity: 'simple' | 'moderate' | 'complex';
  rotationResetCount: number;
  createdAt: Date;
  updatedAt: Date;
}
```

#### Relationships
- User → many Recipes (recipe collection)
- User → many MealPlans (generated meal plans)
- User → many RecipeRatings (community participation)

### Recipe

**Purpose:** Core recipe entity with metadata essential for intelligent meal planning automation and community features

**Key Attributes:**
- id: UUID - Primary identifier
- title: string - Recipe display name
- description: text - Recipe overview for community features
- ingredients: Ingredient[] - Structured ingredient list for shopping automation
- instructions: Step[] - Cooking steps with time estimates
- prepTime: integer - Total preparation time in minutes for scheduling
- cookTime: integer - Active cooking time for availability matching  
- complexity: enum - Simple, Moderate, Complex for automation filtering
- cuisineType: string - Cultural categorization for variety optimization
- mealType: enum - Breakfast, Lunch, Dinner, Snack for calendar assignment
- servings: integer - Portion size for ingredient scaling
- imageUrl: string - Recipe photo URL for visual appeal

#### TypeScript Interface

```typescript
interface Recipe {
  id: string;
  userId: string;
  title: string;
  description: string;
  ingredients: {
    name: string;
    amount: number;
    unit: string;
    category: 'produce' | 'dairy' | 'pantry' | 'protein' | 'other';
  }[];
  instructions: {
    stepNumber: number;
    instruction: string;
    estimatedMinutes?: number;
  }[];
  prepTime: number;
  cookTime: number;
  complexity: 'simple' | 'moderate' | 'complex';
  cuisineType: string;
  mealType: 'breakfast' | 'lunch' | 'dinner' | 'snack';
  servings: number;
  imageUrl?: string;
  isPublic: boolean;
  createdAt: Date;
  updatedAt: Date;
}
```

#### Relationships
- Recipe → one User (owner)
- Recipe → many MealPlanEntries (usage in meal plans)
- Recipe → many RecipeRatings (community feedback)

### MealPlan

**Purpose:** Weekly meal planning container with automation metadata for user meal organization and rotation tracking

**Key Attributes:**
- id: UUID - Primary identifier
- userId: UUID - Plan owner reference
- weekStartDate: Date - Monday of the target week for calendar alignment
- generationType: enum - Automated, Manual, Mixed for generation tracking
- generatedAt: DateTime - Automation timestamp for performance monitoring
- totalEstimatedTime: integer - Weekly cooking time for user planning
- isActive: boolean - Current active plan for user interface

#### TypeScript Interface

```typescript
interface MealPlan {
  id: string;
  userId: string;
  weekStartDate: Date;
  generationType: 'automated' | 'manual' | 'mixed';
  generatedAt: Date;
  totalEstimatedTime: number; // minutes
  isActive: boolean;
  entries: MealPlanEntry[];
  createdAt: Date;
  updatedAt: Date;
}
```

#### Relationships
- MealPlan → one User (owner)
- MealPlan → many MealPlanEntries (daily meal assignments)

### MealPlanEntry

**Purpose:** Individual meal assignment within weekly plan supporting manual overrides and scheduling optimization

**Key Attributes:**
- id: UUID - Primary identifier
- mealPlanId: UUID - Parent plan reference
- recipeId: UUID - Assigned recipe reference  
- date: Date - Specific day assignment
- mealType: enum - Breakfast, Lunch, Dinner for calendar positioning
- isManualOverride: boolean - Track user modifications vs automation
- scheduledPrepTime: DateTime - Optimal prep timing based on recipe requirements
- isCompleted: boolean - User cooking completion tracking

#### TypeScript Interface

```typescript
interface MealPlanEntry {
  id: string;
  mealPlanId: string;
  recipeId: string;
  date: Date;
  mealType: 'breakfast' | 'lunch' | 'dinner';
  isManualOverride: boolean;
  scheduledPrepTime?: Date;
  isCompleted: boolean;
  notes?: string;
  createdAt: Date;
  updatedAt: Date;
}
```

#### Relationships
- MealPlanEntry → one MealPlan (parent plan)
- MealPlanEntry → one Recipe (assigned recipe)

### RecipeRating

**Purpose:** Community feedback system enabling recipe quality validation and social engagement features

**Key Attributes:**
- id: UUID - Primary identifier
- recipeId: UUID - Rated recipe reference
- userId: UUID - Rating contributor
- rating: integer - 1-5 star rating scale
- review: text - Optional written feedback
- difficulty: enum - Community difficulty assessment vs original recipe rating
- wouldCookAgain: boolean - Recipe recommendation indicator

#### TypeScript Interface

```typescript
interface RecipeRating {
  id: string;
  recipeId: string;
  userId: string;
  rating: number; // 1-5
  review?: string;
  difficulty: 'easier' | 'as_expected' | 'harder';
  wouldCookAgain: boolean;
  createdAt: Date;
  updatedAt: Date;
}
```

#### Relationships
- RecipeRating → one Recipe (rated recipe)  
- RecipeRating → one User (rating author)

## API Specification

### REST API Specification

```yaml
openapi: 3.0.0
info:
  title: imkitchen API
  version: 1.0.0
  description: Intelligent meal planning automation API supporting cross-platform mobile and web applications
servers:
  - url: https://api.imkitchen.app/v1
    description: Production API
  - url: https://staging-api.imkitchen.app/v1
    description: Staging API
  - url: http://localhost:8080/v1
    description: Local development

security:
  - BearerAuth: []

paths:
  # Authentication Endpoints
  /auth/login:
    post:
      tags: [Authentication]
      summary: User login
      requestBody:
        required: true
        content:
          application/json:
            schema:
              type: object
              required: [email, password]
              properties:
                email:
                  type: string
                  format: email
                password:
                  type: string
                  minLength: 8
      responses:
        '200':
          description: Login successful
          content:
            application/json:
              schema:
                type: object
                properties:
                  accessToken:
                    type: string
                  refreshToken:
                    type: string
                  user:
                    $ref: '#/components/schemas/User'

  # Core Meal Planning - "Fill My Week" Automation
  /meal-plans/generate:
    post:
      tags: [Meal Planning]
      summary: Generate automated weekly meal plan
      description: Core "Fill My Week" automation endpoint - must complete in <2 seconds
      requestBody:
        required: false
        content:
          application/json:
            schema:
              type: object
              properties:
                weekStartDate:
                  type: string
                  format: date
                  description: Monday of target week, defaults to current week
                preferences:
                  type: object
                  properties:
                    maxPrepTimePerMeal:
                      type: integer
                      description: Maximum prep time in minutes
                    preferredComplexity:
                      type: array
                      items:
                        type: string
                        enum: [simple, moderate, complex]
      responses:
        '200':
          description: Meal plan generated successfully
          content:
            application/json:
              schema:
                type: object
                properties:
                  mealPlan:
                    $ref: '#/components/schemas/MealPlan'
                  shoppingList:
                    $ref: '#/components/schemas/ShoppingList'
                  generationTime:
                    type: number
                    description: Generation time in milliseconds

  # Recipe Management
  /recipes:
    get:
      tags: [Recipes]
      summary: Get user's recipe collection
      parameters:
        - name: search
          in: query
          schema:
            type: string
        - name: mealType
          in: query
          schema:
            type: string
            enum: [breakfast, lunch, dinner, snack]
        - name: complexity
          in: query
          schema:
            type: string
            enum: [simple, moderate, complex]
      responses:
        '200':
          description: Recipe collection
          content:
            application/json:
              schema:
                type: object
                properties:
                  recipes:
                    type: array
                    items:
                      $ref: '#/components/schemas/Recipe'
                  total:
                    type: integer

    post:
      tags: [Recipes]
      summary: Create new recipe
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/RecipeCreate'
      responses:
        '201':
          description: Recipe created
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Recipe'

  # Community Features
  /community/recipes:
    get:
      tags: [Community]
      summary: Browse public community recipes
      parameters:
        - name: sortBy
          in: query
          schema:
            type: string
            enum: [rating, recent, popular]
            default: rating
        - name: minRating
          in: query
          schema:
            type: number
            minimum: 1
            maximum: 5
      responses:
        '200':
          description: Community recipes
          content:
            application/json:
              schema:
                type: array
                items:
                  allOf:
                    - $ref: '#/components/schemas/Recipe'
                    - type: object
                      properties:
                        averageRating:
                          type: number
                        ratingCount:
                          type: integer

components:
  securitySchemes:
    BearerAuth:
      type: http
      scheme: bearer
      bearerFormat: JWT

  schemas:
    User:
      type: object
      properties:
        id:
          type: string
          format: uuid
        email:
          type: string
          format: email
        displayName:
          type: string
        dietaryRestrictions:
          type: array
          items:
            type: string
        cookingSkillLevel:
          type: string
          enum: [beginner, intermediate, advanced]
        preferredMealComplexity:
          type: string
          enum: [simple, moderate, complex]

    Recipe:
      type: object
      properties:
        id:
          type: string
          format: uuid
        title:
          type: string
        ingredients:
          type: array
          items:
            type: object
            properties:
              name:
                type: string
              amount:
                type: number
              unit:
                type: string
              category:
                type: string
                enum: [produce, dairy, pantry, protein, other]
        prepTime:
          type: integer
        cookTime:
          type: integer
        complexity:
          type: string
          enum: [simple, moderate, complex]
        mealType:
          type: string
          enum: [breakfast, lunch, dinner, snack]

    MealPlan:
      type: object
      properties:
        id:
          type: string
          format: uuid
        weekStartDate:
          type: string
          format: date
        generationType:
          type: string
          enum: [automated, manual, mixed]
        entries:
          type: array
          items:
            $ref: '#/components/schemas/MealPlanEntry'

    MealPlanEntry:
      type: object
      properties:
        id:
          type: string
          format: uuid
        recipeId:
          type: string
          format: uuid
        recipe:
          $ref: '#/components/schemas/Recipe'
        date:
          type: string
          format: date
        mealType:
          type: string
          enum: [breakfast, lunch, dinner]
        isManualOverride:
          type: boolean

    ShoppingList:
      type: object
      properties:
        categories:
          type: object
          properties:
            produce:
              type: array
              items:
                type: object
                properties:
                  name:
                    type: string
                  totalAmount:
                    type: number
                  unit:
                    type: string
            dairy:
              type: array
            pantry:
              type: array
            protein:
              type: array

    RecipeCreate:
      type: object
      required: [title, ingredients, prepTime, cookTime, complexity, mealType]
      properties:
        title:
          type: string
        ingredients:
          type: array
        prepTime:
          type: integer
        cookTime:
          type: integer
        complexity:
          type: string
          enum: [simple, moderate, complex]
        mealType:
          type: string
          enum: [breakfast, lunch, dinner, snack]
```

## 6. Components

### Frontend Components (Lynx.js)

#### Core UI Components
- **FillMyWeekButton**: Primary CTA component with loading states and progress indicators
- **MealPlanGrid**: Weekly calendar view with drag-drop meal reordering
- **RecipeCard**: Standardized recipe display with ratings, prep time, and dietary tags
- **DietaryRestrictionPicker**: Multi-select component for user preferences
- **IngredientList**: Interactive shopping list with check-off functionality
- **SkillLevelSelector**: User proficiency input with progressive disclosure

#### Service Integration Components
- **AuthProvider**: Supabase Auth integration with session management
- **DataProvider**: API client wrapper with caching and offline support
- **NotificationProvider**: Push notification handling and in-app alerts
- **ErrorBoundary**: Global error handling with user-friendly fallbacks

#### Navigation Components
- **TabNavigator**: Bottom tab navigation (Meal Plan, Recipes, Profile, Shopping)
- **StackNavigator**: Screen-to-screen navigation with transition animations
- **DrawerMenu**: Settings and secondary navigation

### Backend Services (Go)

#### Core Services
- **AuthService**: User authentication, JWT management, session validation
- **MealPlanService**: Core business logic for meal plan generation and optimization
- **RecipeService**: Recipe CRUD operations, search, and recommendation logic
- **UserService**: User profile management and preference handling
- **NotificationService**: Push notification scheduling and delivery

#### Data Access Layer
- **RecipeRepository**: PostgreSQL operations for recipe data
- **UserRepository**: User data persistence and retrieval
- **MealPlanRepository**: Meal plan storage and history tracking
- **CacheService**: Redis integration for performance optimization

#### External Integration Services
- **RecipeAPIClient**: Third-party recipe API integration (Spoonacular, Edamam)
- **NutritionService**: Nutritional analysis and dietary restriction validation
- **ImageService**: Recipe image processing and optimization (MinIO integration)

### Shared Components

#### Common Utilities
- **ValidationService**: Input validation rules shared between frontend and backend
- **DateTimeUtils**: Meal planning date calculations and timezone handling
- **NutritionCalculator**: Shared nutritional analysis logic
- **ErrorCodes**: Standardized error codes and messages

#### API Layer
- **APIClient**: Type-safe HTTP client with automatic retries and caching
- **WebSocketClient**: Real-time updates for collaborative meal planning
- **OfflineQueue**: Request queuing for offline-first functionality

### Component Boundaries and Interfaces

#### Frontend → Backend Communication
```typescript
// API Client Interface
interface APIClient {
  auth: AuthAPI;
  mealPlans: MealPlanAPI;
  recipes: RecipeAPI;
  users: UserAPI;
}

// Service Layer Interface
interface MealPlanAPI {
  generate(preferences: MealPlanPreferences): Promise<MealPlan>;
  save(mealPlan: MealPlan): Promise<void>;
  getHistory(userId: string, limit: number): Promise<MealPlan[]>;
}
```

#### Service Dependencies
```go
// Backend Service Dependencies
type MealPlanService struct {
    recipeRepo     RecipeRepository
    userRepo       UserRepository
    mealPlanRepo   MealPlanRepository
    cacheService   CacheService
    recipeAPI      RecipeAPIClient
    nutritionSvc   NutritionService
}

// Repository Interfaces
type RecipeRepository interface {
    FindByDietaryRestrictions(restrictions []string) ([]Recipe, error)
    FindByComplexity(level string) ([]Recipe, error)
    FindByMealType(mealType string) ([]Recipe, error)
    CreateBatch(recipes []Recipe) error
}
```

### Component Scalability Design

#### Horizontal Scaling Strategy
- **Stateless Services**: All backend services designed without server-side state
- **Database Connection Pooling**: Efficient PostgreSQL connection management
- **Cache Distribution**: Redis cluster support for distributed caching
- **Load Balancing**: Service mesh ready with health check endpoints

#### Component Isolation
- **Service Boundaries**: Clear separation of concerns between services
- **Database Per Service**: Each major service owns its data domain
- **API Versioning**: Backward-compatible API evolution strategy
- **Circuit Breakers**: Fault isolation between external API dependencies

## 7. External APIs

### Recipe Data Sources

#### Primary: Spoonacular API
```go
// Spoonacular API Client Configuration
type SpoonacularClient struct {
    APIKey     string
    BaseURL    string
    RateLimit  int // requests per minute
    Timeout    time.Duration
}

// Core Recipe Search Interface
type RecipeSearchParams struct {
    Query              string   `json:"query"`
    Diet               []string `json:"diet"`              // vegetarian, vegan, gluten-free
    Intolerances       []string `json:"intolerances"`      // dairy, nuts, shellfish
    IncludeIngredients []string `json:"includeIngredients"`
    ExcludeIngredients []string `json:"excludeIngredients"`
    MaxReadyTime       int      `json:"maxReadyTime"`      // minutes
    MaxServings        int      `json:"maxServings"`
    MinServings        int      `json:"minServings"`
    Sort               string   `json:"sort"`              // popularity, healthiness, time
    Number             int      `json:"number"`            // max 100 per request
}
```

#### Secondary: Edamam Recipe API
- **Purpose**: Backup recipe source and nutritional analysis
- **Rate Limits**: 10,000 calls/month (Developer plan)
- **Key Features**: Extensive dietary filters, nutrition facts, recipe analysis
- **Failover Strategy**: Automatic switch when Spoonacular unavailable

### Nutritional Analysis

#### Edamam Nutrition API
```typescript
// Nutrition Analysis Request
interface NutritionAnalysisRequest {
  title: string;
  ingr: string[];  // ingredients list
  url?: string;    // optional recipe URL
}

// Nutrition Response Structure
interface NutritionResponse {
  uri: string;
  calories: number;
  totalWeight: number;
  dietLabels: string[];      // LOW_CARB, LOW_FAT, etc.
  healthLabels: string[];    // VEGAN, VEGETARIAN, GLUTEN_FREE
  cautions: string[];        // allergen warnings
  totalNutrients: {
    [key: string]: {
      label: string;
      quantity: number;
      unit: string;
    }
  };
}
```

### Authentication Service

#### Supabase Auth Integration
```typescript
// Supabase Configuration
interface SupabaseConfig {
  url: string;
  anonKey: string;
  jwtSecret: string;
  serviceRoleKey: string;
}

// Authentication Methods
interface AuthMethods {
  signUp(email: string, password: string): Promise<AuthResponse>;
  signIn(email: string, password: string): Promise<AuthResponse>;
  signOut(): Promise<void>;
  resetPassword(email: string): Promise<void>;
  updateProfile(updates: UserProfileUpdate): Promise<User>;
}
```

### Image Processing

#### MinIO Integration
```go
// Image Upload Configuration
type ImageUploadConfig struct {
    Endpoint        string
    AccessKey       string
    SecretKey       string
    BucketName      string
    MaxFileSize     int64  // bytes
    AllowedTypes    []string // ["image/jpeg", "image/png", "image/webp"]
    CompressionQuality int  // 1-100
}

// Image Processing Pipeline
type ImageProcessor struct {
    Resize     []ImageSize `json:"resize"`
    Compress   bool        `json:"compress"`
    Format     string      `json:"format"`     // webp, jpeg, png
    Watermark  bool        `json:"watermark"`
}
```

### Push Notifications

#### Firebase Cloud Messaging (FCM)
```typescript
// FCM Message Structure
interface PushNotification {
  token: string;              // device FCM token
  notification: {
    title: string;
    body: string;
    image?: string;
  };
  data?: {
    mealPlanId?: string;
    recipeId?: string;
    action: 'meal_plan_ready' | 'recipe_suggestion' | 'shopping_reminder';
  };
  android?: {
    priority: 'high' | 'normal';
    ttl: string;              // time to live
  };
  apns?: {
    payload: {
      aps: {
        badge: number;
        sound: string;
      }
    }
  };
}
```

### External API Error Handling

#### Circuit Breaker Pattern
```go
// Circuit Breaker Configuration
type CircuitBreakerConfig struct {
    MaxFailures     int           `json:"maxFailures"`     // 5
    ResetTimeout    time.Duration `json:"resetTimeout"`    // 60 seconds
    RetryTimeout    time.Duration `json:"retryTimeout"`    // 30 seconds
    HealthCheck     func() error  `json:"-"`
}

// API Client with Circuit Breaker
type ExternalAPIClient struct {
    httpClient      *http.Client
    circuitBreaker  *CircuitBreaker
    rateLimiter     *RateLimiter
    retryPolicy     RetryPolicy
}
```

#### Rate Limiting Strategy
```go
// Rate Limiter per API Provider
type RateLimiter struct {
    RequestsPerMinute int
    BurstSize         int
    TokenBucket       *TokenBucket
}

// API Request Queue
type APIRequestQueue struct {
    Priority    int                    `json:"priority"`    // 1-5, 1 = highest
    Request     ExternalAPIRequest     `json:"request"`
    Callback    func(response, error)  `json:"-"`
    Retry       RetryConfig           `json:"retry"`
    CreatedAt   time.Time             `json:"createdAt"`
}
```

### API Integration Security

#### API Key Management
- **Encryption**: All API keys encrypted at rest using AES-256
- **Rotation**: Automatic key rotation every 90 days
- **Environment Separation**: Different keys for development, staging, production
- **Access Logging**: All external API calls logged with request ID correlation

#### Request Authentication
```go
// API Request Signing
func SignRequest(request *http.Request, secret string) {
    timestamp := strconv.FormatInt(time.Now().Unix(), 10)
    message := request.Method + request.URL.Path + timestamp
    signature := generateHMAC(message, secret)
    
    request.Header.Set("X-Timestamp", timestamp)
    request.Header.Set("X-Signature", signature)
    request.Header.Set("User-Agent", "ImKitchen/1.0")
}
```

### Monitoring and Observability

#### External API Metrics
- **Response Times**: P50, P95, P99 latencies per API endpoint
- **Error Rates**: HTTP 4xx, 5xx errors tracked separately
- **Rate Limit Usage**: Current usage vs. quota for each provider
- **Circuit Breaker Status**: Open/closed state per service
- **Queue Depth**: Pending external API requests

#### Alerting Thresholds
- **High Error Rate**: >5% error rate over 5-minute window
- **Slow Response**: P95 latency >3 seconds for recipe APIs
- **Rate Limit Approaching**: >80% of quota used
- **Circuit Breaker Open**: Any external service circuit breaker opens

## 8. Core Workflows

### Primary Workflow: "Fill My Week" Meal Plan Generation

#### Workflow Overview
```mermaid
sequenceDiagram
    participant U as User
    participant F as Frontend
    participant A as Auth Service
    participant M as MealPlan Service
    participant R as Recipe Service
    participant C as Cache
    participant E as External APIs

    U->>F: Tap "Fill My Week"
    F->>A: Validate JWT Token
    A->>F: User Authenticated
    F->>M: POST /meal-plans/generate
    M->>C: Check cached user preferences
    C->>M: User preferences + history
    M->>R: Get available recipes
    R->>E: Search external APIs (if needed)
    E->>R: Recipe data
    R->>M: Filtered recipes
    M->>M: Run meal planning algorithm
    M->>C: Cache generated meal plan
    M->>F: Return meal plan
    F->>U: Display 7-day meal plan (<2s)
```

#### Detailed Workflow Steps

**Step 1: User Authentication & Preferences Retrieval**
```typescript
async function initiateWeeklyMealPlan(userId: string): Promise<MealPlanWorkflow> {
  // 1. Validate user session
  const user = await authService.validateUser(userId);
  
  // 2. Get user preferences (cached)
  const preferences = await userService.getPreferences(userId);
  
  // 3. Get meal plan history for variety
  const history = await mealPlanService.getRecentHistory(userId, 4); // 4 weeks
  
  return {
    user,
    preferences,
    recentMeals: history.flatMap(plan => plan.meals),
    workflowId: generateWorkflowId()
  };
}
```

**Step 2: Recipe Pool Generation**
```go
// Recipe selection algorithm
func (s *MealPlanService) buildRecipePool(ctx context.Context, req MealPlanRequest) (*RecipePool, error) {
    pool := &RecipePool{
        Breakfast: make([]Recipe, 0),
        Lunch:     make([]Recipe, 0),
        Dinner:    make([]Recipe, 0),
        Snacks:    make([]Recipe, 0),
    }
    
    // Get recipes from cache first
    cachedRecipes, err := s.cacheService.GetRecipesByPreferences(req.UserPreferences)
    if err == nil && len(cachedRecipes) > 100 {
        return s.filterAndCategorizeRecipes(cachedRecipes, req), nil
    }
    
    // Fallback to external APIs
    searchParams := RecipeSearchParams{
        Diet:               req.UserPreferences.DietaryRestrictions,
        Intolerances:       req.UserPreferences.Allergies,
        MaxReadyTime:       req.UserPreferences.MaxCookTime,
        ExcludeIngredients: req.RecentIngredients, // avoid repetition
        Number:             150, // request extra for filtering
    }
    
    recipes, err := s.recipeAPI.SearchRecipes(ctx, searchParams)
    if err != nil {
        return nil, fmt.Errorf("failed to fetch recipes: %w", err)
    }
    
    return s.filterAndCategorizeRecipes(recipes, req), nil
}
```

**Step 3: Intelligent Meal Plan Algorithm**
```go
func (s *MealPlanService) generateOptimalMealPlan(pool *RecipePool, constraints MealPlanConstraints) (*MealPlan, error) {
    mealPlan := &MealPlan{
        ID:        uuid.New().String(),
        UserID:    constraints.UserID,
        WeekStart: constraints.WeekStart,
        Meals:     make(map[string][]MealEntry),
    }
    
    // Algorithm priorities:
    // 1. Dietary restrictions compliance (hard constraint)
    // 2. Variety across week (avoid same meal type repetition)
    // 3. Cooking skill level appropriateness
    // 4. Prep time distribution (balance quick vs. complex meals)
    // 5. Ingredient overlap optimization (reduce shopping list)
    
    days := []string{"monday", "tuesday", "wednesday", "thursday", "friday", "saturday", "sunday"}
    
    for _, day := range days {
        dayMeals, err := s.selectDayMeals(pool, constraints, mealPlan, day)
        if err != nil {
            return nil, fmt.Errorf("failed to generate meals for %s: %w", day, err)
        }
        mealPlan.Meals[day] = dayMeals
    }
    
    // Final optimization pass
    mealPlan = s.optimizeIngredientOverlap(mealPlan)
    mealPlan = s.validateNutritionalBalance(mealPlan)
    
    return mealPlan, nil
}
```

### Secondary Workflow: Recipe Discovery and Curation

#### User-Driven Recipe Search
```typescript
// Advanced recipe search with filters
interface RecipeSearchWorkflow {
  searchQuery: string;
  filters: {
    mealType: MealType[];
    dietaryRestrictions: string[];
    cookingTime: TimeRange;
    difficulty: SkillLevel;
    cuisineType: string[];
    rating: number; // minimum rating
  };
  pagination: {
    page: number;
    limit: number; // max 50
  };
}

async function searchRecipes(workflow: RecipeSearchWorkflow): Promise<RecipeSearchResult> {
  // 1. Build search query with filters
  const searchParams = buildSearchParameters(workflow);
  
  // 2. Check cache for similar searches
  const cacheKey = generateSearchCacheKey(searchParams);
  const cachedResults = await cacheService.get(cacheKey);
  
  if (cachedResults && !isStale(cachedResults)) {
    return cachedResults;
  }
  
  // 3. Execute parallel searches across APIs
  const [spoonacularResults, edamamResults] = await Promise.allSettled([
    spoonacularAPI.search(searchParams),
    edamamAPI.search(searchParams)
  ]);
  
  // 4. Merge and rank results
  const mergedResults = mergeAndDeduplicateResults(spoonacularResults, edamamResults);
  const rankedResults = applyPersonalizedRanking(mergedResults, workflow.userId);
  
  // 5. Cache results
  await cacheService.set(cacheKey, rankedResults, RECIPE_CACHE_TTL);
  
  return rankedResults;
}
```

### Workflow: Shopping List Generation

```go
// Automated shopping list creation from meal plan
func (s *ShoppingListService) generateFromMealPlan(mealPlanID string, userID string) (*ShoppingList, error) {
    mealPlan, err := s.mealPlanRepo.FindByID(mealPlanID)
    if err != nil {
        return nil, err
    }
    
    // 1. Extract all ingredients from meal plan
    allIngredients := s.extractIngredients(mealPlan)
    
    // 2. Consolidate similar ingredients (e.g., "1 cup flour" + "2 tbsp flour")
    consolidatedIngredients := s.consolidateIngredients(allIngredients)
    
    // 3. Check user's pantry (if available)
    pantryItems, _ := s.userRepo.GetPantryItems(userID)
    neededIngredients := s.filterOutPantryItems(consolidatedIngredients, pantryItems)
    
    // 4. Categorize by store section
    categorizedList := s.categorizeIngredients(neededIngredients)
    
    // 5. Apply user's store preferences (aisle ordering)
    storePrefs, _ := s.userRepo.GetStorePreferences(userID)
    optimizedList := s.optimizeForStore(categorizedList, storePrefs)
    
    shoppingList := &ShoppingList{
        ID:           uuid.New().String(),
        UserID:       userID,
        MealPlanID:   mealPlanID,
        Items:        optimizedList,
        CreatedAt:    time.Now(),
        EstimatedCost: s.calculateEstimatedCost(optimizedList),
    }
    
    return s.shoppingListRepo.Create(shoppingList)
}
```

### Workflow: User Onboarding and Preference Learning

```typescript
// Progressive user preference learning
interface OnboardingWorkflow {
  step1_BasicInfo: {
    dietaryRestrictions: string[];
    allergies: string[];
    cookingSkillLevel: SkillLevel;
  };
  
  step2_PreferenceQuiz: {
    favoriteCuisines: string[];
    dislikedIngredients: string[];
    cookingTimePreference: 'quick' | 'moderate' | 'elaborate';
    mealComplexityPreference: 'simple' | 'varied' | 'adventurous';
  };
  
  step3_TrialMealPlan: {
    generateSamplePlan: boolean;
    collectFeedback: boolean;
  };
}

async function executeOnboardingWorkflow(userId: string): Promise<UserProfile> {
  const workflow = new OnboardingWorkflow();
  
  // Step 1: Collect basic dietary requirements
  const basicPrefs = await collectBasicPreferences(userId);
  workflow.step1_BasicInfo = basicPrefs;
  
  // Step 2: Preference discovery through interactive quiz
  const detailedPrefs = await runPreferenceQuiz(userId, basicPrefs);
  workflow.step2_PreferenceQuiz = detailedPrefs;
  
  // Step 3: Generate trial meal plan for immediate feedback
  const trialMealPlan = await generateTrialMealPlan(userId, {
    ...basicPrefs,
    ...detailedPrefs
  });
  
  // Step 4: Collect initial feedback
  const feedback = await collectTrialFeedback(userId, trialMealPlan.id);
  
  // Step 5: Create refined user profile
  const userProfile = await createRefinedUserProfile(userId, workflow, feedback);
  
  return userProfile;
}
```

### Workflow Error Handling and Recovery

```go
// Workflow-level error handling
type WorkflowError struct {
    WorkflowID   string    `json:"workflowId"`
    StepID       string    `json:"stepId"`
    ErrorType    string    `json:"errorType"`
    ErrorMessage string    `json:"errorMessage"`
    RetryCount   int       `json:"retryCount"`
    Timestamp    time.Time `json:"timestamp"`
    UserID       string    `json:"userId"`
}

// Automatic retry with exponential backoff
func (s *WorkflowService) executeWithRetry(ctx context.Context, step WorkflowStep) error {
    maxRetries := 3
    baseDelay := time.Second
    
    for attempt := 0; attempt <= maxRetries; attempt++ {
        err := step.Execute(ctx)
        if err == nil {
            return nil // Success
        }
        
        if !isRetryableError(err) {
            return fmt.Errorf("non-retryable error in step %s: %w", step.ID, err)
        }
        
        if attempt == maxRetries {
            return fmt.Errorf("step %s failed after %d attempts: %w", step.ID, maxRetries, err)
        }
        
        // Exponential backoff
        delay := baseDelay * time.Duration(1<<attempt)
        time.Sleep(delay)
        
        s.logRetryAttempt(step.WorkflowID, step.ID, attempt+1, err)
    }
    
    return nil
}
```

### Workflow Performance Monitoring

```go
// Workflow step timing and performance tracking
func (s *WorkflowService) trackStepPerformance(workflowID, stepID string) func() {
    startTime := time.Now()
    
    return func() {
        duration := time.Since(startTime)
        
        // Record metrics
        s.metricsService.RecordStepDuration(workflowID, stepID, duration)
        
        // Alert if step is taking too long
        if stepID == "meal_plan_generation" && duration > 2*time.Second {
            s.alertService.SendAlert(AlertCritical, fmt.Sprintf(
                "Meal plan generation exceeded 2s threshold: %v", duration))
        }
        
        s.logger.Info("workflow step completed",
            "workflowId", workflowID,
            "stepId", stepID,
            "duration", duration.String(),
        )
    }
}
```

## 9. Database Schema

### PostgreSQL Schema Design

#### Core Tables

**Users Table**
```sql
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE NOT NULL,
    email_verified BOOLEAN DEFAULT false,
    encrypted_password VARCHAR(255),
    
    -- Profile Information
    first_name VARCHAR(100),
    last_name VARCHAR(100),
    avatar_url TEXT,
    
    -- Preferences
    dietary_restrictions TEXT[] DEFAULT '{}',
    allergies TEXT[] DEFAULT '{}',
    cooking_skill_level VARCHAR(20) CHECK (cooking_skill_level IN ('beginner', 'intermediate', 'advanced')),
    preferred_meal_complexity VARCHAR(20) CHECK (preferred_meal_complexity IN ('simple', 'moderate', 'complex')),
    max_cook_time INTEGER DEFAULT 60, -- minutes
    
    -- Learning Algorithm Data
    rotation_reset_count INTEGER DEFAULT 0,
    preference_learning_data JSONB DEFAULT '{}',
    
    -- Metadata
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    last_active_at TIMESTAMP WITH TIME ZONE,
    deleted_at TIMESTAMP WITH TIME ZONE,
    
    -- Indexes
    CONSTRAINT email_format CHECK (email ~* '^[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}$')
);

-- Indexes for users table
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_created_at ON users(created_at);
CREATE INDEX idx_users_dietary_restrictions ON users USING GIN(dietary_restrictions);
CREATE INDEX idx_users_deleted_at ON users(deleted_at) WHERE deleted_at IS NULL;
```

**Recipes Table**
```sql
CREATE TABLE recipes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    external_id VARCHAR(255), -- from Spoonacular/Edamam
    external_source VARCHAR(50), -- 'spoonacular', 'edamam', 'user_generated'
    
    -- Basic Recipe Info
    title VARCHAR(255) NOT NULL,
    description TEXT,
    image_url TEXT,
    source_url TEXT,
    
    -- Timing
    prep_time INTEGER NOT NULL, -- minutes
    cook_time INTEGER NOT NULL, -- minutes
    total_time INTEGER GENERATED ALWAYS AS (prep_time + cook_time) STORED,
    
    -- Classification
    meal_type VARCHAR(20)[] CHECK (meal_type <@ ARRAY['breakfast', 'lunch', 'dinner', 'snack']),
    complexity VARCHAR(20) CHECK (complexity IN ('simple', 'moderate', 'complex')),
    cuisine_type VARCHAR(50),
    
    -- Recipe Data
    servings INTEGER DEFAULT 4,
    ingredients JSONB NOT NULL, -- [{"name": "flour", "amount": 2, "unit": "cups"}]
    instructions JSONB NOT NULL, -- [{"step": 1, "instruction": "Mix flour..."}]
    
    -- Nutritional Information
    nutrition JSONB, -- calories, protein, carbs, fat, fiber, etc.
    dietary_labels TEXT[] DEFAULT '{}', -- vegetarian, vegan, gluten-free, etc.
    
    -- Quality Metrics
    average_rating DECIMAL(3,2) DEFAULT 0.0,
    total_ratings INTEGER DEFAULT 0,
    difficulty_score INTEGER CHECK (difficulty_score BETWEEN 1 AND 10),
    
    -- Metadata
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    deleted_at TIMESTAMP WITH TIME ZONE
);

-- Indexes for recipes table
CREATE INDEX idx_recipes_meal_type ON recipes USING GIN(meal_type);
CREATE INDEX idx_recipes_complexity ON recipes(complexity);
CREATE INDEX idx_recipes_total_time ON recipes(total_time);
CREATE INDEX idx_recipes_dietary_labels ON recipes USING GIN(dietary_labels);
CREATE INDEX idx_recipes_average_rating ON recipes(average_rating DESC);
CREATE INDEX idx_recipes_external_source_id ON recipes(external_source, external_id);
CREATE INDEX idx_recipes_nutrition ON recipes USING GIN(nutrition);
CREATE INDEX idx_recipes_fulltext ON recipes USING GIN(to_tsvector('english', title || ' ' || COALESCE(description, '')));
```

**Meal Plans Table**
```sql
CREATE TABLE meal_plans (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- Planning Period
    week_start DATE NOT NULL,
    week_end DATE GENERATED ALWAYS AS (week_start + INTERVAL '6 days') STORED,
    
    -- Meal Plan Data
    meals JSONB NOT NULL, -- {"monday": [{"mealType": "breakfast", "recipeId": "...", "servings": 2}]}
    
    -- Generation Metadata
    generation_algorithm_version VARCHAR(20) DEFAULT 'v1.0',
    generation_parameters JSONB, -- user preferences at generation time
    generation_duration_ms INTEGER,
    
    -- Status
    status VARCHAR(20) DEFAULT 'active' CHECK (status IN ('draft', 'active', 'archived', 'deleted')),
    
    -- User Interaction
    user_feedback JSONB DEFAULT '{}', -- ratings, modifications, notes
    completion_percentage DECIMAL(5,2) DEFAULT 0.0, -- % of meals actually prepared
    
    -- Metadata
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    archived_at TIMESTAMP WITH TIME ZONE
);

-- Indexes for meal_plans table
CREATE INDEX idx_meal_plans_user_id ON meal_plans(user_id);
CREATE INDEX idx_meal_plans_week_start ON meal_plans(week_start DESC);
CREATE INDEX idx_meal_plans_status ON meal_plans(status);
CREATE INDEX idx_meal_plans_user_week ON meal_plans(user_id, week_start DESC);
CREATE INDEX idx_meal_plans_meals ON meal_plans USING GIN(meals);
```

**Recipe Ratings Table**
```sql
CREATE TABLE recipe_ratings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    recipe_id UUID NOT NULL REFERENCES recipes(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- Rating Data
    overall_rating INTEGER CHECK (overall_rating BETWEEN 1 AND 5),
    difficulty_rating INTEGER CHECK (difficulty_rating BETWEEN 1 AND 5),
    taste_rating INTEGER CHECK (taste_rating BETWEEN 1 AND 5),
    
    -- Feedback
    review_text TEXT,
    would_make_again BOOLEAN,
    actual_prep_time INTEGER, -- minutes (vs. estimated)
    actual_cook_time INTEGER, -- minutes (vs. estimated)
    
    -- Context
    meal_plan_id UUID REFERENCES meal_plans(id) ON DELETE SET NULL,
    cooking_context VARCHAR(50), -- 'weeknight', 'weekend', 'special_occasion'
    
    -- Metadata
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    
    -- Constraints
    UNIQUE(recipe_id, user_id)
);

-- Indexes for recipe_ratings table
CREATE INDEX idx_recipe_ratings_recipe_id ON recipe_ratings(recipe_id);
CREATE INDEX idx_recipe_ratings_user_id ON recipe_ratings(user_id);
CREATE INDEX idx_recipe_ratings_overall_rating ON recipe_ratings(overall_rating);
CREATE INDEX idx_recipe_ratings_created_at ON recipe_ratings(created_at DESC);
```

#### Supporting Tables

**Shopping Lists Table**
```sql
CREATE TABLE shopping_lists (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    meal_plan_id UUID REFERENCES meal_plans(id) ON DELETE SET NULL,
    
    -- List Data
    name VARCHAR(255) NOT NULL DEFAULT 'Weekly Shopping List',
    items JSONB NOT NULL, -- [{"name": "flour", "quantity": 2, "unit": "lbs", "category": "baking", "checked": false}]
    
    -- Organization
    store_preferences JSONB, -- aisle ordering, preferred brands
    estimated_total_cost DECIMAL(10,2),
    
    -- Status
    status VARCHAR(20) DEFAULT 'active' CHECK (status IN ('active', 'completed', 'archived')),
    completed_at TIMESTAMP WITH TIME ZONE,
    
    -- Metadata
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Indexes for shopping_lists table
CREATE INDEX idx_shopping_lists_user_id ON shopping_lists(user_id);
CREATE INDEX idx_shopping_lists_meal_plan_id ON shopping_lists(meal_plan_id);
CREATE INDEX idx_shopping_lists_status ON shopping_lists(status);
CREATE INDEX idx_shopping_lists_created_at ON shopping_lists(created_at DESC);
```

**User Preferences History Table**
```sql
CREATE TABLE user_preference_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- Preference Tracking
    preference_type VARCHAR(50) NOT NULL, -- 'dietary_restriction_added', 'skill_level_updated'
    old_value JSONB,
    new_value JSONB,
    change_reason VARCHAR(255), -- 'onboarding', 'user_update', 'algorithm_learning'
    
    -- Context
    triggered_by VARCHAR(50), -- 'user_action', 'feedback_analysis', 'preference_quiz'
    confidence_score DECIMAL(5,4), -- algorithm confidence in learned preferences
    
    -- Metadata
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Indexes for user_preference_history table
CREATE INDEX idx_user_pref_history_user_id ON user_preference_history(user_id);
CREATE INDEX idx_user_pref_history_type ON user_preference_history(preference_type);
CREATE INDEX idx_user_pref_history_created_at ON user_preference_history(created_at DESC);
```

#### Performance and Analytics Tables

**Recipe Performance Metrics**
```sql
CREATE TABLE recipe_performance_metrics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    recipe_id UUID NOT NULL REFERENCES recipes(id) ON DELETE CASCADE,
    
    -- Usage Statistics (daily aggregation)
    date DATE NOT NULL,
    times_included_in_meal_plans INTEGER DEFAULT 0,
    times_rated INTEGER DEFAULT 0,
    average_rating_that_day DECIMAL(3,2),
    
    -- Performance Metrics
    successful_generations INTEGER DEFAULT 0, -- times algorithm successfully used this recipe
    user_modifications INTEGER DEFAULT 0, -- times users changed/replaced this recipe
    completion_rate DECIMAL(5,2) DEFAULT 0.0, -- % of times users actually cooked this
    
    -- Algorithm Learning
    algorithm_score DECIMAL(8,6) DEFAULT 0.0, -- internal scoring for meal plan generation
    
    -- Metadata
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    
    -- Constraints
    UNIQUE(recipe_id, date)
);

-- Indexes for recipe_performance_metrics table
CREATE INDEX idx_recipe_perf_recipe_date ON recipe_performance_metrics(recipe_id, date DESC);
CREATE INDEX idx_recipe_perf_date ON recipe_performance_metrics(date DESC);
CREATE INDEX idx_recipe_perf_algorithm_score ON recipe_performance_metrics(algorithm_score DESC);
```

#### Views for Common Queries

**Active User Meal Plans View**
```sql
CREATE VIEW active_user_meal_plans AS
SELECT 
    mp.*,
    u.email,
    u.dietary_restrictions,
    u.cooking_skill_level,
    EXTRACT(EPOCH FROM (NOW() - mp.created_at))/3600 AS hours_since_creation,
    jsonb_array_length(mp.meals->'monday') + 
    jsonb_array_length(mp.meals->'tuesday') + 
    jsonb_array_length(mp.meals->'wednesday') + 
    jsonb_array_length(mp.meals->'thursday') + 
    jsonb_array_length(mp.meals->'friday') + 
    jsonb_array_length(mp.meals->'saturday') + 
    jsonb_array_length(mp.meals->'sunday') AS total_planned_meals
FROM meal_plans mp
JOIN users u ON mp.user_id = u.id
WHERE mp.status = 'active'
    AND u.deleted_at IS NULL
    AND mp.week_start <= CURRENT_DATE
    AND mp.week_end >= CURRENT_DATE;
```

**Recipe Recommendations View**
```sql
CREATE VIEW recipe_recommendations AS
SELECT 
    r.*,
    rpm.algorithm_score,
    rpm.completion_rate,
    CASE 
        WHEN r.total_ratings > 50 THEN r.average_rating
        WHEN r.total_ratings > 10 THEN (r.average_rating * r.total_ratings + 3.5 * 10) / (r.total_ratings + 10)
        ELSE 3.5 -- default rating for new recipes
    END AS weighted_rating
FROM recipes r
LEFT JOIN (
    SELECT 
        recipe_id,
        AVG(algorithm_score) as algorithm_score,
        AVG(completion_rate) as completion_rate
    FROM recipe_performance_metrics 
    WHERE date >= CURRENT_DATE - INTERVAL '30 days'
    GROUP BY recipe_id
) rpm ON r.id = rpm.recipe_id
WHERE r.deleted_at IS NULL;
```

#### Database Functions

**Update Recipe Rating Aggregates Function**
```sql
CREATE OR REPLACE FUNCTION update_recipe_rating_aggregates()
RETURNS TRIGGER AS $$
BEGIN
    -- Update recipe aggregates when rating is inserted/updated
    UPDATE recipes 
    SET 
        average_rating = (
            SELECT ROUND(AVG(overall_rating)::numeric, 2)
            FROM recipe_ratings 
            WHERE recipe_id = COALESCE(NEW.recipe_id, OLD.recipe_id)
        ),
        total_ratings = (
            SELECT COUNT(*)
            FROM recipe_ratings 
            WHERE recipe_id = COALESCE(NEW.recipe_id, OLD.recipe_id)
        ),
        updated_at = NOW()
    WHERE id = COALESCE(NEW.recipe_id, OLD.recipe_id);
    
    RETURN COALESCE(NEW, OLD);
END;
$$ LANGUAGE plpgsql;

-- Trigger for automatic rating aggregation
CREATE TRIGGER trigger_update_recipe_ratings
    AFTER INSERT OR UPDATE OR DELETE ON recipe_ratings
    FOR EACH ROW
    EXECUTE FUNCTION update_recipe_rating_aggregates();
```

**Search Recipes Function**
```sql
CREATE OR REPLACE FUNCTION search_recipes(
    search_query TEXT DEFAULT NULL,
    dietary_restrictions TEXT[] DEFAULT '{}',
    max_cook_time INTEGER DEFAULT NULL,
    meal_types TEXT[] DEFAULT '{}',
    complexity_levels TEXT[] DEFAULT '{}',
    min_rating DECIMAL DEFAULT 0.0,
    limit_count INTEGER DEFAULT 50,
    offset_count INTEGER DEFAULT 0
)
RETURNS TABLE(
    recipe_id UUID,
    title VARCHAR,
    image_url TEXT,
    total_time INTEGER,
    complexity VARCHAR,
    average_rating DECIMAL,
    relevance_score REAL
) AS $$
BEGIN
    RETURN QUERY
    SELECT 
        r.id,
        r.title,
        r.image_url,
        r.total_time,
        r.complexity,
        r.average_rating,
        CASE 
            WHEN search_query IS NOT NULL THEN 
                ts_rank(to_tsvector('english', r.title || ' ' || COALESCE(r.description, '')), 
                        plainto_tsquery('english', search_query))
            ELSE 1.0
        END::REAL as relevance_score
    FROM recipes r
    WHERE r.deleted_at IS NULL
        AND (search_query IS NULL OR 
             to_tsvector('english', r.title || ' ' || COALESCE(r.description, '')) @@ 
             plainto_tsquery('english', search_query))
        AND (array_length(dietary_restrictions, 1) IS NULL OR 
             r.dietary_labels && dietary_restrictions)
        AND (max_cook_time IS NULL OR r.total_time <= max_cook_time)
        AND (array_length(meal_types, 1) IS NULL OR r.meal_type && meal_types)
        AND (array_length(complexity_levels, 1) IS NULL OR r.complexity = ANY(complexity_levels))
        AND r.average_rating >= min_rating
    ORDER BY relevance_score DESC, r.average_rating DESC
    LIMIT limit_count
    OFFSET offset_count;
END;
$$ LANGUAGE plpgsql;
```

## 10. Frontend Architecture

### Lynx.js Mobile Application Structure

#### Application Architecture Pattern: MVVM + State Management

```typescript
// App-level architecture
interface AppArchitecture {
  presentation: {
    views: ReactNativeComponents;
    viewModels: StateManagementLayer;
    navigation: NavigationSystem;
  };
  domain: {
    services: BusinessLogicServices;
    repositories: DataAccessLayer;
    models: DomainModels;
  };
  infrastructure: {
    api: HTTPClients;
    storage: LocalStorageServices;
    notifications: PushNotificationServices;
  };
}
```

### State Management Architecture

#### Global State Structure (Redux Toolkit)
```typescript
// Root state interface
interface RootState {
  auth: AuthState;
  user: UserState;
  mealPlans: MealPlanState;
  recipes: RecipeState;
  shoppingLists: ShoppingListState;
  ui: UIState;
  cache: CacheState;
}

// Authentication state
interface AuthState {
  user: User | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  token: string | null;
  refreshToken: string | null;
  sessionExpiry: Date | null;
}

// Meal plan state with optimistic updates
interface MealPlanState {
  currentWeekPlan: MealPlan | null;
  upcomingWeekPlans: MealPlan[];
  planHistory: MealPlan[];
  isGenerating: boolean;
  generationProgress: number; // 0-100
  lastGenerated: Date | null;
  optimisticUpdates: OptimisticUpdate[];
}

// Recipe state with search and caching
interface RecipeState {
  searchResults: Recipe[];
  favoriteRecipes: Recipe[];
  recentlyViewed: Recipe[];
  searchQuery: string;
  searchFilters: RecipeFilters;
  isSearching: boolean;
  cache: {
    [searchKey: string]: {
      results: Recipe[];
      timestamp: Date;
      ttl: number;
    }
  };
}
```

#### State Management with Redux Toolkit
```typescript
// Async thunks for complex operations
export const generateMealPlan = createAsyncThunk(
  'mealPlans/generate',
  async (preferences: MealPlanPreferences, { getState, rejectWithValue }) => {
    try {
      const state = getState() as RootState;
      const userId = state.auth.user?.id;
      
      if (!userId) {
        return rejectWithValue('User not authenticated');
      }

      // Optimistic update
      const optimisticPlan: MealPlan = {
        id: 'optimistic-' + Date.now(),
        userId,
        weekStart: preferences.weekStart,
        meals: {},
        status: 'generating',
        createdAt: new Date(),
      };

      // Start generation with progress tracking
      const response = await api.post('/meal-plans/generate', {
        preferences,
        userId,
      });

      return response.data;
    } catch (error) {
      return rejectWithValue(error.response?.data || 'Generation failed');
    }
  }
);

// Meal plan slice with optimistic updates
const mealPlanSlice = createSlice({
  name: 'mealPlans',
  initialState,
  reducers: {
    updateMealOptimistic: (state, action: PayloadAction<MealUpdate>) => {
      // Immediately update UI while API call is in flight
      const { planId, dayOfWeek, mealIndex, newRecipe } = action.payload;
      const plan = state.currentWeekPlan;
      if (plan && plan.id === planId) {
        plan.meals[dayOfWeek][mealIndex] = {
          ...plan.meals[dayOfWeek][mealIndex],
          recipeId: newRecipe.id,
          recipe: newRecipe,
        };
      }
      
      // Track optimistic update for potential rollback
      state.optimisticUpdates.push({
        id: generateUpdateId(),
        type: 'meal_update',
        payload: action.payload,
        timestamp: Date.now(),
      });
    },
    
    revertOptimisticUpdate: (state, action: PayloadAction<string>) => {
      // Rollback failed optimistic update
      const updateId = action.payload;
      const updateIndex = state.optimisticUpdates.findIndex(u => u.id === updateId);
      
      if (updateIndex !== -1) {
        const update = state.optimisticUpdates[updateIndex];
        // Implement rollback logic based on update type
        state.optimisticUpdates.splice(updateIndex, 1);
      }
    },
  },
  
  extraReducers: (builder) => {
    builder
      .addCase(generateMealPlan.pending, (state) => {
        state.isGenerating = true;
        state.generationProgress = 0;
      })
      .addCase(generateMealPlan.fulfilled, (state, action) => {
        state.currentWeekPlan = action.payload;
        state.isGenerating = false;
        state.generationProgress = 100;
        state.lastGenerated = new Date();
        // Clear optimistic updates on success
        state.optimisticUpdates = [];
      })
      .addCase(generateMealPlan.rejected, (state, action) => {
        state.isGenerating = false;
        state.generationProgress = 0;
        // Revert optimistic updates on failure
        state.optimisticUpdates = [];
      });
  },
});
```

### Component Architecture

#### Atomic Design System Implementation
```typescript
// Atoms: Basic building blocks
interface AtomComponents {
  Button: ButtonComponent;
  Text: TextComponent;
  Input: InputComponent;
  Image: ImageComponent;
  Icon: IconComponent;
  Badge: BadgeComponent;
}

// Molecules: Simple component combinations
interface MoleculeComponents {
  SearchBar: SearchBarComponent;
  RecipeCard: RecipeCardComponent;
  MealSlot: MealSlotComponent;
  RatingStars: RatingStarsComponent;
  IngredientItem: IngredientItemComponent;
  FilterChip: FilterChipComponent;
}

// Organisms: Complex component sections
interface OrganismComponents {
  Header: HeaderComponent;
  MealPlanGrid: MealPlanGridComponent;
  RecipeList: RecipeListComponent;
  ShoppingList: ShoppingListComponent;
  UserProfile: UserProfileComponent;
  NavigationBar: NavigationBarComponent;
}

// Templates: Page layouts
interface TemplateComponents {
  MainLayout: MainLayoutComponent;
  AuthLayout: AuthLayoutComponent;
  OnboardingLayout: OnboardingLayoutComponent;
}
```

#### Core UI Components

**FillMyWeekButton Component**
```typescript
interface FillMyWeekButtonProps {
  onPress: () => void;
  isGenerating: boolean;
  progress?: number; // 0-100
  lastGenerated?: Date;
  disabled?: boolean;
}

const FillMyWeekButton: React.FC<FillMyWeekButtonProps> = ({
  onPress,
  isGenerating,
  progress = 0,
  lastGenerated,
  disabled = false,
}) => {
  const animatedValue = useRef(new Animated.Value(0)).current;
  
  useEffect(() => {
    if (isGenerating) {
      // Pulse animation during generation
      Animated.loop(
        Animated.sequence([
          Animated.timing(animatedValue, {
            toValue: 1,
            duration: 1000,
            useNativeDriver: true,
          }),
          Animated.timing(animatedValue, {
            toValue: 0,
            duration: 1000,
            useNativeDriver: true,
          }),
        ])
      ).start();
    } else {
      animatedValue.setValue(0);
    }
  }, [isGenerating]);

  const pulseStyle = {
    opacity: animatedValue.interpolate({
      inputRange: [0, 1],
      outputRange: [1, 0.7],
    }),
  };

  return (
    <TouchableOpacity
      onPress={onPress}
      disabled={disabled || isGenerating}
      style={[styles.button, disabled && styles.buttonDisabled]}
      accessibilityLabel="Generate weekly meal plan"
      accessibilityHint="Creates a personalized 7-day meal plan"
    >
      <Animated.View style={[styles.buttonContent, isGenerating && pulseStyle]}>
        {isGenerating ? (
          <View style={styles.generatingContent}>
            <ActivityIndicator size="small" color="#FFFFFF" />
            <Text style={styles.buttonText}>
              Generating... {Math.round(progress)}%
            </Text>
            <ProgressBar progress={progress / 100} color="#FFFFFF" />
          </View>
        ) : (
          <View style={styles.defaultContent}>
            <Icon name="magic-wand" size={24} color="#FFFFFF" />
            <Text style={styles.buttonText}>Fill My Week</Text>
            {lastGenerated && (
              <Text style={styles.lastGeneratedText}>
                Last: {formatRelativeTime(lastGenerated)}
              </Text>
            )}
          </View>
        )}
      </Animated.View>
    </TouchableOpacity>
  );
};
```

**MealPlanGrid Component**
```typescript
interface MealPlanGridProps {
  mealPlan: MealPlan | null;
  onMealPress: (day: string, mealIndex: number, meal: MealEntry) => void;
  onMealReplace: (day: string, mealIndex: number) => void;
  isEditable: boolean;
}

const MealPlanGrid: React.FC<MealPlanGridProps> = ({
  mealPlan,
  onMealPress,
  onMealReplace,
  isEditable = true,
}) => {
  const [draggedMeal, setDraggedMeal] = useState<MealEntry | null>(null);
  
  const daysOfWeek = ['monday', 'tuesday', 'wednesday', 'thursday', 'friday', 'saturday', 'sunday'];
  const mealTypes = ['breakfast', 'lunch', 'dinner'];

  const handleMealDrop = (targetDay: string, targetMealType: string, sourceMeal: MealEntry) => {
    // Implement drag-and-drop meal reordering
    onMealReplace(targetDay, getMealIndex(targetMealType));
  };

  if (!mealPlan) {
    return (
      <View style={styles.emptyState}>
        <Icon name="calendar-outline" size={64} color="#CCCCCC" />
        <Text style={styles.emptyStateText}>
          No meal plan generated yet
        </Text>
        <Text style={styles.emptyStateSubtext}>
          Tap "Fill My Week" to create your personalized meal plan
        </Text>
      </View>
    );
  }

  return (
    <ScrollView
      horizontal
      showsHorizontalScrollIndicator={false}
      contentContainerStyle={styles.gridContainer}
    >
      {daysOfWeek.map((day) => (
        <View key={day} style={styles.dayColumn}>
          <Text style={styles.dayHeader}>
            {formatDayHeader(day, mealPlan.weekStart)}
          </Text>
          
          {mealTypes.map((mealType, mealIndex) => {
            const meal = mealPlan.meals[day]?.[mealIndex];
            
            return (
              <MealSlot
                key={`${day}-${mealType}`}
                meal={meal}
                mealType={mealType}
                day={day}
                mealIndex={mealIndex}
                onPress={() => meal && onMealPress(day, mealIndex, meal)}
                onReplace={() => onMealReplace(day, mealIndex)}
                onDragStart={(meal) => setDraggedMeal(meal)}
                onDrop={(droppedMeal) => handleMealDrop(day, mealType, droppedMeal)}
                isDragging={draggedMeal?.id === meal?.id}
                isDropTarget={draggedMeal !== null && draggedMeal.id !== meal?.id}
                isEditable={isEditable}
              />
            );
          })}
        </View>
      ))}
    </ScrollView>
  );
};
```

### Navigation Architecture

#### React Navigation Setup
```typescript
// Navigation stack configuration
type RootStackParamList = {
  AuthFlow: undefined;
  MainFlow: undefined;
  OnboardingFlow: undefined;
  MealPlanDetails: { mealPlanId: string };
  RecipeDetails: { recipeId: string };
  RecipeSearch: { initialQuery?: string };
  ShoppingList: { mealPlanId?: string };
  UserProfile: undefined;
  Settings: undefined;
};

type MainTabParamList = {
  MealPlan: undefined;
  Recipes: undefined;
  Shopping: undefined;
  Profile: undefined;
};

// Navigation configuration with authentication flow
const RootNavigator: React.FC = () => {
  const { isAuthenticated, isLoading } = useSelector((state: RootState) => state.auth);
  const { hasCompletedOnboarding } = useSelector((state: RootState) => state.user);

  if (isLoading) {
    return <SplashScreen />;
  }

  return (
    <NavigationContainer
      linking={linkingConfiguration}
      theme={navigationTheme}
      ref={navigationRef}
    >
      <Stack.Navigator screenOptions={{ headerShown: false }}>
        {!isAuthenticated ? (
          <Stack.Screen name="AuthFlow" component={AuthFlowNavigator} />
        ) : !hasCompletedOnboarding ? (
          <Stack.Screen name="OnboardingFlow" component={OnboardingFlowNavigator} />
        ) : (
          <>
            <Stack.Screen name="MainFlow" component={MainTabNavigator} />
            <Stack.Screen 
              name="MealPlanDetails" 
              component={MealPlanDetailsScreen}
              options={{ presentation: 'modal' }}
            />
            <Stack.Screen 
              name="RecipeDetails" 
              component={RecipeDetailsScreen}
              options={{ presentation: 'modal' }}
            />
          </>
        )}
      </Stack.Navigator>
    </NavigationContainer>
  );
};
```

### Data Layer Architecture

#### API Client with Caching
```typescript
// API client with automatic caching and retry logic
class APIClient {
  private httpClient: AxiosInstance;
  private cache: CacheService;
  private authService: AuthService;
  
  constructor() {
    this.httpClient = axios.create({
      baseURL: Config.API_BASE_URL,
      timeout: 10000,
    });
    
    this.setupInterceptors();
    this.setupRetryLogic();
  }

  private setupInterceptors() {
    // Request interceptor for authentication
    this.httpClient.interceptors.request.use(
      async (config) => {
        const token = await this.authService.getValidToken();
        if (token) {
          config.headers.Authorization = `Bearer ${token}`;
        }
        
        // Add request ID for tracking
        config.headers['X-Request-ID'] = generateRequestId();
        
        return config;
      },
      (error) => Promise.reject(error)
    );

    // Response interceptor for error handling and caching
    this.httpClient.interceptors.response.use(
      (response) => {
        // Cache successful responses
        this.cacheResponse(response);
        return response;
      },
      async (error) => {
        if (error.response?.status === 401) {
          // Handle authentication errors
          await this.authService.handleAuthError();
        }
        
        // Log error for monitoring
        this.logError(error);
        
        return Promise.reject(error);
      }
    );
  }

  // Cached API methods
  async getMealPlan(userId: string, weekStart: Date): Promise<MealPlan> {
    const cacheKey = `meal_plan_${userId}_${weekStart.toISOString().split('T')[0]}`;
    
    // Check cache first
    const cached = await this.cache.get(cacheKey);
    if (cached && !this.cache.isExpired(cached)) {
      return cached.data;
    }

    // Fetch from API
    const response = await this.httpClient.get(`/meal-plans`, {
      params: { userId, weekStart: weekStart.toISOString() },
    });

    // Cache the response
    await this.cache.set(cacheKey, response.data, { ttl: 3600 }); // 1 hour

    return response.data;
  }

  async generateMealPlan(preferences: MealPlanPreferences): Promise<MealPlan> {
    // No caching for generation requests
    const response = await this.httpClient.post('/meal-plans/generate', preferences, {
      timeout: 30000, // Extended timeout for generation
    });

    // Invalidate related cache entries
    await this.cache.invalidatePattern(`meal_plan_${preferences.userId}_*`);

    return response.data;
  }
}
```

### Offline-First Architecture

#### Offline Data Management
```typescript
// Offline-first data synchronization
class OfflineDataManager {
  private storage: AsyncStorageService;
  private syncQueue: SyncQueue;
  private networkState: NetworkStateService;

  async syncWhenOnline(): Promise<void> {
    if (!this.networkState.isConnected) {
      return;
    }

    const pendingActions = await this.syncQueue.getPendingActions();
    
    for (const action of pendingActions) {
      try {
        await this.executeAction(action);
        await this.syncQueue.markCompleted(action.id);
      } catch (error) {
        if (this.isRetryableError(error)) {
          await this.syncQueue.scheduleRetry(action.id);
        } else {
          await this.syncQueue.markFailed(action.id, error);
        }
      }
    }
  }

  async saveMealPlanOffline(mealPlan: MealPlan): Promise<void> {
    // Save to local storage
    await this.storage.setItem(`meal_plan_${mealPlan.id}`, mealPlan);
    
    // Queue for sync when online
    await this.syncQueue.addAction({
      type: 'SAVE_MEAL_PLAN',
      payload: mealPlan,
      timestamp: Date.now(),
      retryCount: 0,
    });
  }

  async getMealPlanOffline(mealPlanId: string): Promise<MealPlan | null> {
    return await this.storage.getItem(`meal_plan_${mealPlanId}`);
  }
}
```

### Performance Optimization

#### Lazy Loading and Code Splitting
```typescript
// Lazy-loaded screens for better initial load time
const MealPlanScreen = lazy(() => import('../screens/MealPlanScreen'));
const RecipeSearchScreen = lazy(() => import('../screens/RecipeSearchScreen'));
const ShoppingListScreen = lazy(() => import('../screens/ShoppingListScreen'));
const ProfileScreen = lazy(() => import('../screens/ProfileScreen'));

// Image lazy loading with placeholders
const LazyImage: React.FC<{ source: string; placeholder?: string }> = ({ 
  source, 
  placeholder = 'default-recipe-image' 
}) => {
  const [isLoaded, setIsLoaded] = useState(false);
  const [hasError, setHasError] = useState(false);

  return (
    <View style={styles.imageContainer}>
      {!isLoaded && !hasError && (
        <Image source={{ uri: placeholder }} style={styles.placeholderImage} />
      )}
      
      <Image
        source={{ uri: source }}
        style={[styles.image, !isLoaded && styles.hiddenImage]}
        onLoad={() => setIsLoaded(true)}
        onError={() => setHasError(true)}
        resizeMode="cover"
      />
      
      {hasError && (
        <View style={styles.errorState}>
          <Icon name="image-off" size={24} color="#CCCCCC" />
        </View>
      )}
    </View>
  );
};
```

#### Memory Management and Performance
```typescript
// Performance monitoring and optimization
class PerformanceMonitor {
  private static instance: PerformanceMonitor;
  private performanceObserver: PerformanceObserver;

  static getInstance(): PerformanceMonitor {
    if (!PerformanceMonitor.instance) {
      PerformanceMonitor.instance = new PerformanceMonitor();
    }
    return PerformanceMonitor.instance;
  }

  trackScreenTransition(screenName: string, startTime: number): void {
    const endTime = performance.now();
    const duration = endTime - startTime;
    
    // Log slow screen transitions
    if (duration > 500) { // > 500ms
      console.warn(`Slow screen transition to ${screenName}: ${duration}ms`);
      
      // Report to analytics
      Analytics.track('slow_screen_transition', {
        screen: screenName,
        duration,
        timestamp: Date.now(),
      });
    }
  }

  trackMealPlanGeneration(startTime: number, endTime: number): void {
    const duration = endTime - startTime;
    
    Analytics.track('meal_plan_generation_time', {
      duration,
      timestamp: Date.now(),
    });
    
    // Alert if generation exceeds 2-second target
    if (duration > 2000) {
      console.warn(`Meal plan generation exceeded target: ${duration}ms`);
    }
  }
}
```

## 11. Backend Architecture

### Go Service Architecture

#### Hexagonal Architecture Implementation

```go
// Domain-driven design with hexagonal architecture
package main

// Domain Layer - Core business logic
type Domain struct {
    entities    map[string]interface{}
    valueObjects map[string]interface{}
    services    map[string]interface{}
}

// Application Layer - Use cases and orchestration
type Application struct {
    useCases    map[string]UseCase
    services    map[string]ApplicationService
    validators  map[string]Validator
}

// Infrastructure Layer - External concerns
type Infrastructure struct {
    repositories map[string]Repository
    clients     map[string]ExternalClient
    cache       CacheService
    messaging   MessageBroker
}
```

### Service Architecture with Gin Framework

#### Main Server Setup
```go
// main.go - Application bootstrap
package main

import (
    "context"
    "fmt"
    "log"
    "net/http"
    "os"
    "os/signal"
    "syscall"
    "time"

    "github.com/gin-gonic/gin"
    "github.com/imkitchen/internal/config"
    "github.com/imkitchen/internal/handlers"
    "github.com/imkitchen/internal/middleware"
    "github.com/imkitchen/internal/services"
    "github.com/imkitchen/internal/repositories"
)

func main() {
    // Load configuration
    cfg, err := config.Load()
    if err != nil {
        log.Fatalf("Failed to load config: %v", err)
    }

    // Initialize dependencies
    deps := initializeDependencies(cfg)
    
    // Setup Gin router
    router := setupRouter(deps, cfg)
    
    // Create HTTP server
    server := &http.Server{
        Addr:         fmt.Sprintf(":%d", cfg.Server.Port),
        Handler:      router,
        ReadTimeout:  cfg.Server.ReadTimeout,
        WriteTimeout: cfg.Server.WriteTimeout,
        IdleTimeout:  cfg.Server.IdleTimeout,
    }

    // Start server in goroutine
    go func() {
        log.Printf("Starting server on port %d", cfg.Server.Port)
        if err := server.ListenAndServe(); err != nil && err != http.ErrServerClosed {
            log.Fatalf("Server failed to start: %v", err)
        }
    }()

    // Graceful shutdown
    waitForShutdown(server, cfg.Server.GracefulTimeout)
}

func setupRouter(deps *Dependencies, cfg *config.Config) *gin.Engine {
    if cfg.Environment == "production" {
        gin.SetMode(gin.ReleaseMode)
    }

    router := gin.New()

    // Global middleware
    router.Use(
        middleware.Logger(),
        middleware.Recovery(),
        middleware.CORS(cfg.CORS),
        middleware.RequestID(),
        middleware.Security(cfg.Security),
    )

    // Health check endpoint
    router.GET("/health", handlers.HealthCheck(deps))
    router.GET("/readiness", handlers.ReadinessCheck(deps))

    // API version 1
    v1 := router.Group("/api/v1")
    {
        // Authentication endpoints
        auth := v1.Group("/auth")
        {
            auth.POST("/register", deps.AuthHandler.Register)
            auth.POST("/login", deps.AuthHandler.Login)
            auth.POST("/refresh", deps.AuthHandler.RefreshToken)
            auth.POST("/logout", middleware.RequireAuth(deps.AuthService), deps.AuthHandler.Logout)
            auth.POST("/forgot-password", deps.AuthHandler.ForgotPassword)
            auth.POST("/reset-password", deps.AuthHandler.ResetPassword)
        }

        // Protected routes
        protected := v1.Group("/")
        protected.Use(middleware.RequireAuth(deps.AuthService))
        {
            // User management
            users := protected.Group("/users")
            {
                users.GET("/me", deps.UserHandler.GetCurrentUser)
                users.PUT("/me", deps.UserHandler.UpdateCurrentUser)
                users.DELETE("/me", deps.UserHandler.DeleteCurrentUser)
                users.POST("/preferences", deps.UserHandler.UpdatePreferences)
                users.GET("/preferences/history", deps.UserHandler.GetPreferenceHistory)
            }

            // Meal plan management
            mealPlans := protected.Group("/meal-plans")
            {
                mealPlans.POST("/generate", middleware.RateLimit("meal-plan-generation", 5, time.Minute), deps.MealPlanHandler.Generate)
                mealPlans.GET("/", deps.MealPlanHandler.GetUserMealPlans)
                mealPlans.GET("/:id", deps.MealPlanHandler.GetMealPlan)
                mealPlans.PUT("/:id", deps.MealPlanHandler.UpdateMealPlan)
                mealPlans.DELETE("/:id", deps.MealPlanHandler.DeleteMealPlan)
                mealPlans.POST("/:id/feedback", deps.MealPlanHandler.SubmitFeedback)
            }

            // Recipe management
            recipes := protected.Group("/recipes")
            {
                recipes.GET("/", deps.RecipeHandler.SearchRecipes)
                recipes.GET("/:id", deps.RecipeHandler.GetRecipe)
                recipes.POST("/:id/rating", deps.RecipeHandler.RateRecipe)
                recipes.GET("/:id/nutrition", deps.RecipeHandler.GetNutritionInfo)
                recipes.POST("/favorites/:id", deps.RecipeHandler.AddToFavorites)
                recipes.DELETE("/favorites/:id", deps.RecipeHandler.RemoveFromFavorites)
            }

            // Shopping list management
            shopping := protected.Group("/shopping-lists")
            {
                shopping.GET("/", deps.ShoppingHandler.GetUserShoppingLists)
                shopping.POST("/", deps.ShoppingHandler.CreateShoppingList)
                shopping.GET("/:id", deps.ShoppingHandler.GetShoppingList)
                shopping.PUT("/:id", deps.ShoppingHandler.UpdateShoppingList)
                shopping.DELETE("/:id", deps.ShoppingHandler.DeleteShoppingList)
                shopping.PUT("/:id/items/:itemId", deps.ShoppingHandler.UpdateShoppingItem)
            }
        }
    }

    return router
}
```

#### High-Performance Caching Layer

```go
// services/cache_service.go
package services

import (
    "context"
    "encoding/json"
    "fmt"
    "time"

    "github.com/go-redis/redis/v8"
    "github.com/imkitchen/pkg/logger"
)

type RedisCacheService struct {
    client *redis.Client
    logger logger.Logger
    defaultTTL time.Duration
}

func NewRedisCacheService(client *redis.Client, logger logger.Logger) *RedisCacheService {
    return &RedisCacheService{
        client:     client,
        logger:     logger,
        defaultTTL: time.Hour,
    }
}

// Intelligent caching for meal plan generation
func (c *RedisCacheService) GetOrSetMealPlan(ctx context.Context, key string, generator func() (*domain.MealPlan, error)) (*domain.MealPlan, error) {
    // Try to get from cache first
    cached, err := c.client.Get(ctx, key).Result()
    if err == nil {
        var mealPlan domain.MealPlan
        if err := json.Unmarshal([]byte(cached), &mealPlan); err == nil {
            c.logger.Debug("Cache hit for meal plan", "key", key)
            return &mealPlan, nil
        }
    }

    // Cache miss - generate new meal plan
    c.logger.Debug("Cache miss for meal plan", "key", key)
    mealPlan, err := generator()
    if err != nil {
        return nil, err
    }

    // Store in cache asynchronously to not block response
    go func() {
        if data, err := json.Marshal(mealPlan); err == nil {
            c.client.SetEX(context.Background(), key, data, c.defaultTTL)
        }
    }()

    return mealPlan, nil
}

// Multi-level caching strategy for recipe search
func (c *RedisCacheService) CacheRecipeSearch(ctx context.Context, searchKey string, recipes []*domain.Recipe, ttl time.Duration) {
    data, err := json.Marshal(recipes)
    if err != nil {
        c.logger.Error("Failed to marshal recipes for caching", "error", err)
        return
    }

    // Use pipeline for better performance
    pipe := c.client.Pipeline()
    
    // Cache full results
    pipe.SetEX(ctx, searchKey, data, ttl)
    
    // Cache individual recipes for faster lookups
    for _, recipe := range recipes {
        recipeKey := fmt.Sprintf("recipe:%s", recipe.ID)
        recipeData, _ := json.Marshal(recipe)
        pipe.SetEX(ctx, recipeKey, recipeData, ttl*2) // Longer TTL for individual recipes
    }
    
    _, err = pipe.Exec(ctx)
    if err != nil {
        c.logger.Error("Failed to execute cache pipeline", "error", err)
    }
}
```

## 12. Unified Project Structure

### Monorepo Organization (Nx Workspace)

```
imkitchen/
├── README.md
├── package.json                    # Root package.json with workspace config
├── nx.json                         # Nx workspace configuration  
├── tsconfig.base.json             # Base TypeScript config
├── .gitignore
├── .env.example
├── docker-compose.yml             # Local development environment
├── Dockerfile.frontend            # Frontend production build
├── Dockerfile.backend             # Backend production build
├── kubernetes/                    # K8s deployment manifests
│   ├── namespace.yaml
│   ├── configmap.yaml
│   ├── secrets.yaml
│   ├── backend-deployment.yaml
│   ├── frontend-deployment.yaml
│   ├── postgres-deployment.yaml
│   ├── redis-deployment.yaml
│   └── ingress.yaml
├── apps/
│   ├── mobile/                    # Lynx.js Mobile App
│   │   ├── src/
│   │   │   ├── screens/          # Screen components
│   │   │   │   ├── auth/         # Authentication screens
│   │   │   │   ├── meal-plans/   # Meal planning screens
│   │   │   │   ├── recipes/      # Recipe browsing screens
│   │   │   │   ├── shopping/     # Shopping list screens
│   │   │   │   └── profile/      # User profile screens
│   │   │   ├── components/       # Reusable UI components
│   │   │   │   ├── atoms/        # Basic building blocks
│   │   │   │   ├── molecules/    # Simple combinations
│   │   │   │   ├── organisms/    # Complex sections
│   │   │   │   └── templates/    # Page layouts
│   │   │   ├── services/         # API clients and services
│   │   │   ├── store/            # Redux store configuration
│   │   │   └── types/            # TypeScript type definitions
│   │   └── __tests__/            # Mobile app tests
│   └── backend/                   # Go Backend Service
│       ├── internal/
│       │   ├── handlers/         # HTTP request handlers
│       │   ├── services/         # Application services
│       │   ├── repositories/     # Data access layer
│       │   ├── middleware/       # HTTP middleware
│       │   └── clients/          # External API clients
│       ├── migrations/           # Database migrations
│       └── tests/                # Backend tests
├── libs/                         # Shared libraries
│   ├── shared-types/            # Shared TypeScript types
│   ├── ui-components/           # Shared UI components
│   ├── api-client/              # Shared API client
│   └── validation/              # Shared validation rules
└── tools/                       # Development tools
    ├── database/
    ├── scripts/
    └── docker/
```

## 13. Development Workflow

### Git Workflow Strategy

#### Branch Strategy (GitFlow)
- **main**: Production-ready code
- **develop**: Integration branch for features
- **feature/\***: Individual feature development
- **release/\***: Release preparation
- **hotfix/\***: Critical production fixes

#### Commit Convention
```
type(scope): description

feat(mobile): add meal plan generation progress indicator
fix(backend): resolve race condition in recipe caching
docs(architecture): update deployment section
test(mobile): add unit tests for MealPlanGrid component
refactor(backend): optimize database query performance
```

### CI/CD Pipeline

#### GitHub Actions Workflow
```yaml
# .github/workflows/ci.yml
name: Continuous Integration

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main, develop]

jobs:
  test:
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:15
        env:
          POSTGRES_PASSWORD: postgres
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5

    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: '18'
          cache: 'npm'
      
      - name: Install dependencies
        run: npm ci
      
      - name: Run linting
        run: npm run lint
        
      - name: Run type checking  
        run: npm run type-check
        
      - name: Run unit tests
        run: npm run test:unit
        
      - name: Run integration tests
        run: npm run test:integration
        env:
          DATABASE_URL: postgresql://postgres:postgres@localhost:5432/test

  build:
    needs: test
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/main'
    
    steps:
      - uses: actions/checkout@v4
      - name: Build Docker images
        run: |
          docker build -t imkitchen-backend -f Dockerfile.backend .
          docker build -t imkitchen-mobile -f Dockerfile.frontend .
      
      - name: Push to registry
        run: |
          echo "${{ secrets.DOCKER_PASSWORD }}" | docker login -u "${{ secrets.DOCKER_USERNAME }}" --password-stdin
          docker push imkitchen-backend:latest
          docker push imkitchen-mobile:latest
```

## 14. Deployment Architecture

### Cloud-Agnostic Kubernetes Deployment

#### Production Architecture
- **Load Balancer**: NGINX Ingress Controller
- **Container Orchestration**: Kubernetes 1.28+
- **Database**: Managed PostgreSQL (Cloud SQL/RDS equivalent)
- **Cache**: Redis Cluster
- **Object Storage**: MinIO or S3-compatible service
- **Monitoring**: Prometheus + Grafana
- **Logging**: ELK Stack (Elasticsearch, Logstash, Kibana)

#### Kubernetes Manifests
```yaml
# kubernetes/backend-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: imkitchen-backend
  namespace: imkitchen
spec:
  replicas: 3
  selector:
    matchLabels:
      app: imkitchen-backend
  template:
    metadata:
      labels:
        app: imkitchen-backend
    spec:
      containers:
      - name: backend
        image: imkitchen-backend:latest
        ports:
        - containerPort: 8080
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: imkitchen-secrets
              key: database-url
        - name: REDIS_URL
          valueFrom:
            secretKeyRef:
              name: imkitchen-secrets  
              key: redis-url
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "500m"
        readinessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 10
          periodSeconds: 5
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 30
```

## 15. Security and Performance

### Security Implementation
- **Authentication**: JWT with Supabase Auth
- **Authorization**: Role-based access control (RBAC)
- **Input Validation**: Schema validation on all endpoints  
- **SQL Injection Prevention**: Parameterized queries only
- **XSS Prevention**: Content Security Policy headers
- **HTTPS**: TLS 1.3 encryption in transit
- **Secrets Management**: Kubernetes secrets + external secret store
- **Rate Limiting**: Per-user and per-endpoint limits
- **Audit Logging**: All user actions logged with correlation IDs

### Performance Optimization
- **Backend**: Connection pooling, query optimization, Redis caching
- **Frontend**: Code splitting, image lazy loading, state management optimization
- **Database**: Proper indexing, query analysis, read replicas
- **CDN**: Static asset distribution via CDN
- **Monitoring**: Real-time performance metrics and alerting

## 16. Testing Strategy

### Testing Pyramid
- **Unit Tests**: 80% coverage target
- **Integration Tests**: API endpoint and database testing
- **End-to-End Tests**: Critical user journey automation
- **Performance Tests**: Load testing for meal plan generation (<2s)
- **Security Tests**: Vulnerability scanning and penetration testing

### Test Technologies
- **Frontend**: Jest, React Native Testing Library, Detox (E2E)
- **Backend**: Go testing package, testify, dockertest
- **Database**: Test containers for isolation
- **API**: Postman/Newman for API testing
- **Performance**: K6 for load testing

## 17. Monitoring and Observability

### Metrics Collection
- **Application Metrics**: Custom business metrics (meal plans generated, user engagement)
- **System Metrics**: CPU, memory, disk, network usage
- **Database Metrics**: Query performance, connection pool status
- **External API Metrics**: Response times, error rates, quota usage

### Alerting Strategy
- **Critical**: System down, database unreachable, high error rate (>5%)
- **Warning**: High response times (>2s for meal generation), approaching rate limits
- **Info**: Deployment events, configuration changes

### Logging Standards
- **Structured logging**: JSON format with correlation IDs
- **Log levels**: ERROR, WARN, INFO, DEBUG
- **Security logging**: Authentication events, authorization failures
- **Business logging**: User actions, meal plan generations, recipe searches

This comprehensive fullstack architecture provides a complete technical blueprint for implementing the imkitchen meal planning application with cloud-agnostic, scalable, and maintainable design patterns.

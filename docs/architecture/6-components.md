# 6. Components

## Frontend Components (Lynx.js)

### Core UI Components
- **FillMyWeekButton**: Primary CTA component with loading states and progress indicators
- **MealPlanGrid**: Weekly calendar view with drag-drop meal reordering
- **RecipeCard**: Standardized recipe display with ratings, prep time, and dietary tags
- **DietaryRestrictionPicker**: Multi-select component for user preferences
- **IngredientList**: Interactive shopping list with check-off functionality
- **SkillLevelSelector**: User proficiency input with progressive disclosure

### Service Integration Components
- **AuthProvider**: Supabase Auth integration with session management
- **DataProvider**: API client wrapper with caching and offline support
- **NotificationProvider**: Push notification handling and in-app alerts
- **ErrorBoundary**: Global error handling with user-friendly fallbacks

### Navigation Components
- **TabNavigator**: Bottom tab navigation (Meal Plan, Recipes, Profile, Shopping)
- **StackNavigator**: Screen-to-screen navigation with transition animations
- **DrawerMenu**: Settings and secondary navigation

## Backend Services (Go)

### Core Services
- **AuthService**: User authentication, JWT management, session validation
- **MealPlanService**: Core business logic for meal plan generation and optimization
- **RecipeService**: Recipe CRUD operations, search, and recommendation logic
- **UserService**: User profile management and preference handling
- **NotificationService**: Push notification scheduling and delivery

### Data Access Layer
- **RecipeRepository**: PostgreSQL operations for recipe data
- **UserRepository**: User data persistence and retrieval
- **MealPlanRepository**: Meal plan storage and history tracking
- **CacheService**: Redis integration for performance optimization

### External Integration Services
- **RecipeAPIClient**: Third-party recipe API integration (Spoonacular, Edamam)
- **NutritionService**: Nutritional analysis and dietary restriction validation
- **ImageService**: Recipe image processing and optimization (MinIO integration)

## Shared Components

### Common Utilities
- **ValidationService**: Input validation rules shared between frontend and backend
- **DateTimeUtils**: Meal planning date calculations and timezone handling
- **NutritionCalculator**: Shared nutritional analysis logic
- **ErrorCodes**: Standardized error codes and messages

### API Layer
- **APIClient**: Type-safe HTTP client with automatic retries and caching
- **WebSocketClient**: Real-time updates for collaborative meal planning
- **OfflineQueue**: Request queuing for offline-first functionality

## Component Boundaries and Interfaces

### Frontend → Backend Communication
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

### Service Dependencies
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

## Component Scalability Design

### Horizontal Scaling Strategy
- **Stateless Services**: All backend services designed without server-side state
- **Database Connection Pooling**: Efficient PostgreSQL connection management
- **Cache Distribution**: Redis cluster support for distributed caching
- **Load Balancing**: Service mesh ready with health check endpoints

### Component Isolation
- **Service Boundaries**: Clear separation of concerns between services
- **Database Per Service**: Each major service owns its data domain
- **API Versioning**: Backward-compatible API evolution strategy
- **Circuit Breakers**: Fault isolation between external API dependencies

# 11. Backend Architecture

## Go Service Architecture

### Hexagonal Architecture Implementation

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

## Service Architecture with Gin Framework

### Main Server Setup
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

### High-Performance Caching Layer

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

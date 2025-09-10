# 7. External APIs

## Recipe Data Sources

### Primary: Spoonacular API
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

### Secondary: Edamam Recipe API
- **Purpose**: Backup recipe source and nutritional analysis
- **Rate Limits**: 10,000 calls/month (Developer plan)
- **Key Features**: Extensive dietary filters, nutrition facts, recipe analysis
- **Failover Strategy**: Automatic switch when Spoonacular unavailable

## Nutritional Analysis

### Edamam Nutrition API
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

## Authentication Service

### Supabase Auth Integration
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

## Image Processing

### MinIO Integration
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

## Push Notifications

### Firebase Cloud Messaging (FCM)
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

## External API Error Handling

### Circuit Breaker Pattern
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

### Rate Limiting Strategy
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

## API Integration Security

### API Key Management
- **Encryption**: All API keys encrypted at rest using AES-256
- **Rotation**: Automatic key rotation every 90 days
- **Environment Separation**: Different keys for development, staging, production
- **Access Logging**: All external API calls logged with request ID correlation

### Request Authentication
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

## Monitoring and Observability

### External API Metrics
- **Response Times**: P50, P95, P99 latencies per API endpoint
- **Error Rates**: HTTP 4xx, 5xx errors tracked separately
- **Rate Limit Usage**: Current usage vs. quota for each provider
- **Circuit Breaker Status**: Open/closed state per service
- **Queue Depth**: Pending external API requests

### Alerting Thresholds
- **High Error Rate**: >5% error rate over 5-minute window
- **Slow Response**: P95 latency >3 seconds for recipe APIs
- **Rate Limit Approaching**: >80% of quota used
- **Circuit Breaker Open**: Any external service circuit breaker opens

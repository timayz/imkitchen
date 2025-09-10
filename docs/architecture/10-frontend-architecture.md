# 10. Frontend Architecture

## Lynx.js Mobile Application Structure

### Application Architecture Pattern: MVVM + State Management

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

## State Management Architecture

### Global State Structure (Redux Toolkit)
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

### State Management with Redux Toolkit
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

## Component Architecture

### Atomic Design System Implementation
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

### Core UI Components

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

## Navigation Architecture

### React Navigation Setup
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

## Data Layer Architecture

### API Client with Caching
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

## Offline-First Architecture

### Offline Data Management
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

## Performance Optimization

### Lazy Loading and Code Splitting
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

### Memory Management and Performance
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

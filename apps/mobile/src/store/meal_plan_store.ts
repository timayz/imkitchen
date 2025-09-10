import { create } from 'zustand';
import { devtools, persist } from 'zustand/middleware';
import type {
  MealPlanResponse,
  CreateMealPlanInput,
  UpdateMealPlanInput,
  UpdateMealSlotInput,
  MealPlanFilters,
  DayOfWeek,
  MealType,
} from '@imkitchen/shared-types';
import { mealPlanService } from '../services/meal_plan_service';
import { shoppingIntegrationService } from '../services/shopping_integration_service';

interface OptimisticUpdate {
  id: string;
  type: 'create' | 'update' | 'delete' | 'move';
  timestamp: number;
  originalData?: any;
  newData?: any;
}

interface MealPlanState {
  // Current state
  currentMealPlan: MealPlanResponse | null;
  mealPlans: Record<string, MealPlanResponse>; // Cached meal plans by week key
  currentWeek: Date;
  
  // Loading states
  loading: boolean;
  loadingWeek: string | null;
  refreshing: boolean;
  generating: boolean;
  generationProgress: number;
  
  // Error handling
  error: string | null;
  
  // Generation metadata
  lastGenerationTime: number | null;
  lastVarietyScore: number | null;
  rotationCycle: number | null;
  
  // Shopping list integration
  autoGenerateShoppingLists: boolean;
  shoppingListNotifications: Array<{
    message: string;
    type: 'success' | 'info' | 'warning';
    timestamp: number;
  }>;
  
  // Optimistic updates
  optimisticUpdates: OptimisticUpdate[];
  
  // Actions
  setCurrentWeek: (week: Date) => void;
  loadMealPlan: (weekStart: Date, forceRefresh?: boolean) => Promise<void>;
  createMealPlan: (input: CreateMealPlanInput) => Promise<void>;
  updateMealPlan: (id: string, input: UpdateMealPlanInput) => Promise<void>;
  updateMealSlot: (
    mealPlanId: string,
    day: DayOfWeek,
    mealType: MealType,
    input: UpdateMealSlotInput
  ) => Promise<void>;
  deleteMealSlot: (mealPlanId: string, day: DayOfWeek, mealType: MealType) => Promise<void>;
  moveMeal: (
    mealPlanId: string,
    fromDay: DayOfWeek,
    fromMealType: MealType,
    toDay: DayOfWeek,
    toMealType: MealType
  ) => Promise<void>;
  refreshCurrentMealPlan: () => Promise<void>;
  
  // Fill My Week generation
  generateWeeklyMealPlan: (options?: {
    weekStartDate?: Date;
    maxPrepTimePerMeal?: number;
    preferredComplexityLevel?: 'simple' | 'moderate' | 'complex';
    cuisinePreferences?: string[];
    avoidRecipeIDs?: string[];
  }) => Promise<void>;
  
  clearError: () => void;
  reset: () => void;
  
  // Shopping list integration actions
  setAutoGenerateShoppingLists: (enabled: boolean) => void;
  addShoppingListNotification: (message: string, type: 'success' | 'info' | 'warning') => void;
  clearShoppingListNotifications: () => void;
  
  // Optimistic updates
  addOptimisticUpdate: (update: OptimisticUpdate) => void;
  removeOptimisticUpdate: (id: string) => void;
  applyOptimisticUpdates: (mealPlan: MealPlanResponse) => MealPlanResponse;
}

const getWeekKey = (date: Date): string => {
  const weekStart = mealPlanService.getWeekStart(date);
  return weekStart.toISOString().split('T')[0];
};

const initialState = {
  currentMealPlan: null,
  mealPlans: {},
  currentWeek: mealPlanService.getWeekStart(new Date()),
  loading: false,
  loadingWeek: null,
  refreshing: false,
  generating: false,
  generationProgress: 0,
  error: null,
  lastGenerationTime: null,
  lastVarietyScore: null,
  rotationCycle: null,
  autoGenerateShoppingLists: true,
  shoppingListNotifications: [],
  optimisticUpdates: [],
};

export const useMealPlanStore = create<MealPlanState>()(
  devtools(
    persist(
      (set, get) => ({
        ...initialState,

        setCurrentWeek: (week: Date) => {
          const weekStart = mealPlanService.getWeekStart(week);
          set((state) => ({
            currentWeek: weekStart,
            currentMealPlan: state.mealPlans[getWeekKey(weekStart)] || null,
          }));
        },

        loadMealPlan: async (weekStart: Date, forceRefresh = false) => {
          const weekKey = getWeekKey(weekStart);
          const { mealPlans, loadingWeek } = get();

          // Prevent duplicate requests for the same week
          if (loadingWeek === weekKey) return;

          // Use cached data if available and not forcing refresh
          if (!forceRefresh && mealPlans[weekKey]) {
            set({ currentMealPlan: mealPlans[weekKey], error: null });
            return;
          }

          set({ loading: true, loadingWeek: weekKey, error: null });

          try {
            const mealPlan = await mealPlanService.getMealPlanByWeek(weekStart);
            const processedMealPlan = get().applyOptimisticUpdates(mealPlan);

            set((state) => ({
              currentMealPlan: processedMealPlan,
              mealPlans: {
                ...state.mealPlans,
                [weekKey]: processedMealPlan,
              },
              loading: false,
              loadingWeek: null,
              error: null,
            }));
          } catch (error) {
            const errorMessage = error instanceof Error ? error.message : 'Failed to load meal plan';
            
            // If the error is "not found", create an empty meal plan structure
            if (errorMessage.includes('not found') || errorMessage.includes('404')) {
              set((state) => ({
                currentMealPlan: null,
                loading: false,
                loadingWeek: null,
                error: null,
              }));
            } else {
              set({
                loading: false,
                loadingWeek: null,
                error: errorMessage,
              });
            }
          }
        },

        createMealPlan: async (input: CreateMealPlanInput) => {
          const optimisticId = `optimistic-${Date.now()}`;
          
          // Add optimistic update
          get().addOptimisticUpdate({
            id: optimisticId,
            type: 'create',
            timestamp: Date.now(),
            newData: input,
          });

          try {
            const mealPlan = await mealPlanService.createMealPlan(input);
            const weekKey = getWeekKey(input.weekStartDate);

            set((state) => ({
              currentMealPlan: mealPlan,
              mealPlans: {
                ...state.mealPlans,
                [weekKey]: mealPlan,
              },
              error: null,
            }));

            get().removeOptimisticUpdate(optimisticId);
            
            // Trigger shopping list generation if enabled
            if (get().autoGenerateShoppingLists) {
              shoppingIntegrationService.onMealPlanGenerated(mealPlan).catch(error => {
                console.error('Shopping list generation failed:', error);
                get().addShoppingListNotification(
                  'Failed to generate shopping list automatically',
                  'warning'
                );
              });
            }
          } catch (error) {
            get().removeOptimisticUpdate(optimisticId);
            const errorMessage = error instanceof Error ? error.message : 'Failed to create meal plan';
            set({ error: errorMessage });
            throw error;
          }
        },

        updateMealPlan: async (id: string, input: UpdateMealPlanInput) => {
          const optimisticId = `optimistic-${Date.now()}`;
          const originalData = get().currentMealPlan;

          // Add optimistic update
          get().addOptimisticUpdate({
            id: optimisticId,
            type: 'update',
            timestamp: Date.now(),
            originalData,
            newData: input,
          });

          try {
            const mealPlan = await mealPlanService.updateMealPlan(id, input);
            const weekKey = getWeekKey(new Date(mealPlan.weekStartDate));

            set((state) => ({
              currentMealPlan: mealPlan,
              mealPlans: {
                ...state.mealPlans,
                [weekKey]: mealPlan,
              },
              error: null,
            }));

            get().removeOptimisticUpdate(optimisticId);
            
            // Trigger shopping list update if enabled and original data exists
            if (get().autoGenerateShoppingLists && originalData) {
              shoppingIntegrationService.onMealPlanUpdated(mealPlan, originalData).catch(error => {
                console.error('Shopping list update failed:', error);
                get().addShoppingListNotification(
                  'Failed to update shopping list automatically',
                  'warning'
                );
              });
            }
          } catch (error) {
            get().removeOptimisticUpdate(optimisticId);
            const errorMessage = error instanceof Error ? error.message : 'Failed to update meal plan';
            set({ error: errorMessage });
            throw error;
          }
        },

        updateMealSlot: async (
          mealPlanId: string,
          day: DayOfWeek,
          mealType: MealType,
          input: UpdateMealSlotInput
        ) => {
          const optimisticId = `optimistic-${Date.now()}`;
          const originalData = get().currentMealPlan;

          // Add optimistic update
          get().addOptimisticUpdate({
            id: optimisticId,
            type: 'update',
            timestamp: Date.now(),
            originalData,
            newData: { day, mealType, ...input },
          });

          try {
            const mealPlan = await mealPlanService.updateMealSlot(mealPlanId, day, mealType, input);
            const weekKey = getWeekKey(new Date(mealPlan.weekStartDate));

            set((state) => ({
              currentMealPlan: mealPlan,
              mealPlans: {
                ...state.mealPlans,
                [weekKey]: mealPlan,
              },
              error: null,
            }));

            get().removeOptimisticUpdate(optimisticId);
          } catch (error) {
            get().removeOptimisticUpdate(optimisticId);
            const errorMessage = error instanceof Error ? error.message : 'Failed to update meal slot';
            set({ error: errorMessage });
            throw error;
          }
        },

        deleteMealSlot: async (mealPlanId: string, day: DayOfWeek, mealType: MealType) => {
          const optimisticId = `optimistic-${Date.now()}`;
          const originalData = get().currentMealPlan;

          // Add optimistic update
          get().addOptimisticUpdate({
            id: optimisticId,
            type: 'delete',
            timestamp: Date.now(),
            originalData,
            newData: { day, mealType },
          });

          try {
            const mealPlan = await mealPlanService.deleteMealSlot(mealPlanId, day, mealType);
            const weekKey = getWeekKey(new Date(mealPlan.weekStartDate));

            set((state) => ({
              currentMealPlan: mealPlan,
              mealPlans: {
                ...state.mealPlans,
                [weekKey]: mealPlan,
              },
              error: null,
            }));

            get().removeOptimisticUpdate(optimisticId);
          } catch (error) {
            get().removeOptimisticUpdate(optimisticId);
            const errorMessage = error instanceof Error ? error.message : 'Failed to delete meal slot';
            set({ error: errorMessage });
            throw error;
          }
        },

        moveMeal: async (
          mealPlanId: string,
          fromDay: DayOfWeek,
          fromMealType: MealType,
          toDay: DayOfWeek,
          toMealType: MealType
        ) => {
          const optimisticId = `optimistic-${Date.now()}`;
          const originalData = get().currentMealPlan;

          // Add optimistic update
          get().addOptimisticUpdate({
            id: optimisticId,
            type: 'move',
            timestamp: Date.now(),
            originalData,
            newData: { fromDay, fromMealType, toDay, toMealType },
          });

          try {
            const mealPlan = await mealPlanService.moveMeal(
              mealPlanId,
              fromDay,
              fromMealType,
              toDay,
              toMealType
            );
            const weekKey = getWeekKey(new Date(mealPlan.weekStartDate));

            set((state) => ({
              currentMealPlan: mealPlan,
              mealPlans: {
                ...state.mealPlans,
                [weekKey]: mealPlan,
              },
              error: null,
            }));

            get().removeOptimisticUpdate(optimisticId);
          } catch (error) {
            get().removeOptimisticUpdate(optimisticId);
            const errorMessage = error instanceof Error ? error.message : 'Failed to move meal';
            set({ error: errorMessage });
            throw error;
          }
        },

        refreshCurrentMealPlan: async () => {
          const { currentWeek } = get();
          set({ refreshing: true });
          
          try {
            await get().loadMealPlan(currentWeek, true);
          } finally {
            set({ refreshing: false });
          }
        },

        generateWeeklyMealPlan: async (options) => {
          const weekStart = options?.weekStartDate || get().currentWeek;
          const weekKey = getWeekKey(weekStart);

          set({ 
            generating: true, 
            generationProgress: 0, 
            error: null 
          });

          try {
            const response = await mealPlanService.generateWeeklyMealPlan(options);
            
            // Update store with generated meal plan
            set((state) => ({
              currentMealPlan: response.mealPlan,
              mealPlans: {
                ...state.mealPlans,
                [weekKey]: response.mealPlan,
              },
              generating: false,
              generationProgress: 100,
              lastGenerationTime: response.generationTimeMs,
              lastVarietyScore: response.varietyScore,
              rotationCycle: response.rotationCycle,
              error: null,
            }));

            // Reset progress after a short delay
            setTimeout(() => {
              set({ generationProgress: 0 });
            }, 1000);

            // Trigger shopping list generation if enabled
            if (get().autoGenerateShoppingLists) {
              shoppingIntegrationService.onMealPlanGenerated(response.mealPlan).catch(error => {
                console.error('Shopping list generation failed:', error);
                get().addShoppingListNotification(
                  'Generated meal plan successfully, but failed to create shopping list',
                  'warning'
                );
              });
            }

          } catch (error) {
            const errorMessage = error instanceof Error ? error.message : 'Failed to generate meal plan';
            set({
              generating: false,
              generationProgress: 0,
              error: errorMessage,
            });
            throw error;
          }
        },

        clearError: () => set({ error: null }),

        reset: () => set(initialState),

        addOptimisticUpdate: (update: OptimisticUpdate) => {
          set((state) => ({
            optimisticUpdates: [...state.optimisticUpdates, update],
          }));
        },

        removeOptimisticUpdate: (id: string) => {
          set((state) => ({
            optimisticUpdates: state.optimisticUpdates.filter((update) => update.id !== id),
          }));
        },

        applyOptimisticUpdates: (mealPlan: MealPlanResponse): MealPlanResponse => {
          const { optimisticUpdates } = get();
          
          if (optimisticUpdates.length === 0) {
            return mealPlan;
          }

          let updatedMealPlan = { ...mealPlan };
          
          // Apply optimistic updates in chronological order
          optimisticUpdates
            .sort((a, b) => a.timestamp - b.timestamp)
            .forEach((update) => {
              // Apply optimistic update logic here
              // This would modify the meal plan based on the update type
              // For simplicity, we're returning the original meal plan
              // In a real implementation, you'd apply the specific changes
            });

          return updatedMealPlan;
        },

        // Shopping list integration actions
        setAutoGenerateShoppingLists: (enabled: boolean) => {
          set({ autoGenerateShoppingLists: enabled });
        },

        addShoppingListNotification: (message: string, type: 'success' | 'info' | 'warning') => {
          set((state) => ({
            shoppingListNotifications: [
              ...state.shoppingListNotifications,
              {
                message,
                type,
                timestamp: Date.now(),
              },
            ],
          }));

          // Auto-clear notification after 5 seconds
          setTimeout(() => {
            set((state) => ({
              shoppingListNotifications: state.shoppingListNotifications.filter(
                (notification) => Date.now() - notification.timestamp < 5000
              ),
            }));
          }, 5000);
        },

        clearShoppingListNotifications: () => {
          set({ shoppingListNotifications: [] });
        },
      }),
      {
        name: 'meal-plan-store',
        // Only persist essential data, not loading states or errors
        partialize: (state) => ({
          mealPlans: state.mealPlans,
          currentWeek: state.currentWeek,
          lastGenerationTime: state.lastGenerationTime,
          lastVarietyScore: state.lastVarietyScore,
          rotationCycle: state.rotationCycle,
          autoGenerateShoppingLists: state.autoGenerateShoppingLists,
        }),
      }
    ),
    { name: 'MealPlanStore' }
  )
);
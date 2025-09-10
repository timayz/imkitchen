import React, { useState, useRef, useEffect } from 'react';
import {
  View,
  Text,
  ScrollView,
  StyleSheet,
  ActivityIndicator,
  Dimensions,
  TouchableOpacity,
} from 'react-native';
import type {
  MealPlanGridProps,
  DayOfWeek,
  MealType,
  MealSlotWithRecipe,
} from '@imkitchen/shared-types';
import { MealSlot } from '../molecules/MealSlot';
import { EmptyMealPlanState } from '../atoms/EmptyMealPlanState';
import { QuickSwapModal } from '../meal-plans/QuickSwapModal';
import { ChangeHistoryPanel } from '../meal-plans/ChangeHistoryPanel';
import { UndoRedoControls } from '../atoms/UndoRedoControls';
import { mealPlanService } from '../../services/meal_plan_service';
import { ChangeHistoryTracker, type ChangeRecord } from '../../services/ChangeHistoryTracker';
import { mealPlanStateService } from '../../services/MealPlanStateService';
import type { ColorTokens } from '../../theme/tokens';

const { width: screenWidth } = Dimensions.get('window');

const DAYS_OF_WEEK: { key: DayOfWeek; label: string }[] = [
  { key: 'monday', label: 'Mon' },
  { key: 'tuesday', label: 'Tue' },
  { key: 'wednesday', label: 'Wed' },
  { key: 'thursday', label: 'Thu' },
  { key: 'friday', label: 'Fri' },
  { key: 'saturday', label: 'Sat' },
  { key: 'sunday', label: 'Sun' },
];

const MEAL_TYPES: { key: MealType; label: string; icon: string }[] = [
  { key: 'breakfast', label: 'Breakfast', icon: '🌅' },
  { key: 'lunch', label: 'Lunch', icon: '☀️' },
  { key: 'dinner', label: 'Dinner', icon: '🌙' },
];

export const MealPlanGrid: React.FC<MealPlanGridProps> = ({
  mealPlan,
  onMealPress,
  onMealLongPress,
  onEmptySlotPress,
  onMealPlanUpdate, // New prop for updating meal plan state
  isEditable = false,
  loading = false,
  error = null,
}) => {
  const [draggedMeal, setDraggedMeal] = React.useState<{
    day: DayOfWeek;
    mealType: MealType;
  } | null>(null);

  const [swapModalVisible, setSwapModalVisible] = useState(false);
  const [swapModalData, setSwapModalData] = useState<{
    day: DayOfWeek;
    mealType: MealType;
    meal: MealSlotWithRecipe;
  } | null>(null);

  const [historyPanelVisible, setHistoryPanelVisible] = useState(false);
  const [historyRefreshing, setHistoryRefreshing] = useState(false);
  
  // Change history tracker
  const changeTracker = useRef<ChangeHistoryTracker | null>(null);
  const [historyState, setHistoryState] = useState({
    canUndo: false,
    canRedo: false,
    currentIndex: -1,
    totalChanges: 0,
  });
  const [changes, setChanges] = useState<ChangeRecord[]>([]);

  // Initialize change tracker when meal plan changes
  useEffect(() => {
    if (mealPlan?.id) {
      changeTracker.current = new ChangeHistoryTracker(mealPlan.id, {
        maxHistoryLength: 20,
        enableBatching: true,
        batchTimeout: 2000,
      });
      updateHistoryState();
    }
  }, [mealPlan?.id]);

  const updateHistoryState = () => {
    if (changeTracker.current) {
      const state = changeTracker.current.getState();
      setHistoryState(state);
      setChanges(changeTracker.current.getHistory());
    }
  };

  const handleDragStart = (day: DayOfWeek, mealType: MealType) => {
    setDraggedMeal({ day, mealType });
    
    // Start batch operation for drag sequence
    if (changeTracker.current) {
      const sourceMeal = getMealForSlot(day, mealType);
      changeTracker.current.startBatch(
        `Drag ${sourceMeal?.recipe?.title || 'meal'} from ${day} ${mealType}`
      );
    }
  };

  const handleDragEnd = async (
    sourceDay: DayOfWeek,
    sourceMealType: MealType,
    targetDay?: DayOfWeek,
    targetMealType?: MealType
  ) => {
    setDraggedMeal(null);
    
    if (!mealPlan || !targetDay || !targetMealType || !changeTracker.current) {
      // End batch if no valid target
      changeTracker.current?.endBatch();
      return;
    }

    // Skip if dropping on same slot
    if (sourceDay === targetDay && sourceMealType === targetMealType) {
      changeTracker.current.endBatch();
      return;
    }

    try {
      // Create before state snapshot
      const beforeState = changeTracker.current.createSnapshot(
        mealPlan.populatedMeals,
        {
          totalEstimatedTime: mealPlan.totalEstimatedTime,
          completionPercentage: mealPlan.completionPercentage,
        }
      );

      // Perform the meal reorder operation
      await mealPlanService.reorderMeal(
        mealPlan.id,
        sourceDay,
        sourceMealType,
        targetDay,
        targetMealType
      );

      // Create after state snapshot (would need updated meal plan data)
      // For now, we'll use the same data - in a real app this would be updated
      const afterState = changeTracker.current.createSnapshot(
        mealPlan.populatedMeals,
        {
          totalEstimatedTime: mealPlan.totalEstimatedTime,
          completionPercentage: mealPlan.completionPercentage,
        }
      );

      const sourceMeal = getMealForSlot(sourceDay, sourceMealType);
      const targetMeal = getMealForSlot(targetDay, targetMealType);

      // Record the change in the batch
      changeTracker.current.recordChange({
        type: 'reorder',
        description: changeTracker.current.generateDescription(
          'reorder',
          sourceDay,
          sourceMealType,
          sourceMeal?.recipe?.title,
          targetDay,
          targetMealType
        ),
        beforeState,
        afterState,
        metadata: {
          reason: 'User drag and drop',
          userInitiated: true,
          affectedSlots: [
            { day: sourceDay, mealType: sourceMealType },
            { day: targetDay, mealType: targetMealType },
          ],
        },
      });

      // End the batch operation
      changeTracker.current.endBatch();
      updateHistoryState();

      console.log('Meal reordered successfully');
    } catch (error) {
      console.error('Failed to reorder meal:', error);
      // End batch on error
      changeTracker.current.endBatch();
    }
  };

  const handleLockToggle = async (day: DayOfWeek, mealType: MealType, isLocked: boolean) => {
    if (!mealPlan || !changeTracker.current) return;

    try {
      // Create before state snapshot
      const beforeState = changeTracker.current.createSnapshot(
        mealPlan.populatedMeals,
        {
          totalEstimatedTime: mealPlan.totalEstimatedTime,
          completionPercentage: mealPlan.completionPercentage,
        }
      );

      // Perform the lock toggle operation
      await mealPlanService.toggleMealLock(
        mealPlan.id,
        day,
        mealType,
        isLocked
      );

      // Create after state snapshot
      const afterState = changeTracker.current.createSnapshot(
        mealPlan.populatedMeals,
        {
          totalEstimatedTime: mealPlan.totalEstimatedTime,
          completionPercentage: mealPlan.completionPercentage,
        }
      );

      const meal = getMealForSlot(day, mealType);

      // Record the change
      changeTracker.current.recordChange({
        type: isLocked ? 'lock' : 'unlock',
        description: changeTracker.current.generateDescription(
          isLocked ? 'lock' : 'unlock',
          day,
          mealType,
          meal?.recipe?.title
        ),
        beforeState,
        afterState,
        metadata: {
          reason: 'User lock toggle',
          userInitiated: true,
          affectedSlots: [{ day, mealType }],
        },
      });

      updateHistoryState();
      console.log('Meal lock toggled successfully');
    } catch (error) {
      console.error('Failed to toggle meal lock:', error);
    }
  };

  const handleSwapPress = (day: DayOfWeek, mealType: MealType, meal: MealSlotWithRecipe) => {
    setSwapModalData({ day, mealType, meal });
    setSwapModalVisible(true);
  };

  const handleSwapModalClose = () => {
    setSwapModalVisible(false);
    setSwapModalData(null);
  };

  const handleSwapConfirmed = async (newRecipeId: string) => {
    if (!swapModalData || !mealPlan || !changeTracker.current) return;

    try {
      // Create before state snapshot
      const beforeState = changeTracker.current.createSnapshot(
        mealPlan.populatedMeals,
        {
          totalEstimatedTime: mealPlan.totalEstimatedTime,
          completionPercentage: mealPlan.completionPercentage,
        }
      );

      // Call the meal plan service to perform the swap
      await mealPlanService.swapRecipe(
        mealPlan.id,
        swapModalData.day,
        swapModalData.mealType,
        newRecipeId,
        'Quick swap from meal plan grid'
      );

      // Create after state snapshot
      const afterState = changeTracker.current.createSnapshot(
        mealPlan.populatedMeals,
        {
          totalEstimatedTime: mealPlan.totalEstimatedTime,
          completionPercentage: mealPlan.completionPercentage,
        }
      );

      // Record the change
      changeTracker.current.recordChange({
        type: 'swap',
        description: changeTracker.current.generateDescription(
          'swap',
          swapModalData.day,
          swapModalData.mealType,
          swapModalData.meal.recipe?.title
        ),
        beforeState,
        afterState,
        metadata: {
          reason: 'Quick swap from meal plan grid',
          userInitiated: true,
          affectedSlots: [{ day: swapModalData.day, mealType: swapModalData.mealType }],
        },
      });

      updateHistoryState();
      console.log('Recipe swapped successfully');
    } catch (error) {
      console.error('Failed to swap recipe:', error);
      throw error; // Re-throw to let the modal handle the error
    }
  };

  const handleGetSwapSuggestions = async (filters: Record<string, any>) => {
    if (!swapModalData || !mealPlan) return [];

    try {
      return await mealPlanService.getSwapSuggestions(
        mealPlan.id,
        swapModalData.day,
        swapModalData.mealType,
        filters
      );
    } catch (error) {
      console.error('Failed to get swap suggestions:', error);
      return [];
    }
  };

  const handlePreviewShoppingListChanges = async (recipeId: string) => {
    if (!swapModalData || !mealPlan) {
      return { itemsAdded: 0, itemsRemoved: 0, estimatedCostChange: 0 };
    }

    try {
      return await mealPlanService.previewShoppingListChanges(
        mealPlan.id,
        swapModalData.day,
        swapModalData.mealType,
        recipeId
      );
    } catch (error) {
      console.error('Failed to preview shopping list changes:', error);
      return { itemsAdded: 0, itemsRemoved: 0, estimatedCostChange: 0 };
    }
  };

  // Undo/Redo handlers
  const handleUndo = async () => {
    if (!changeTracker.current || !mealPlan) return;
    
    const result = changeTracker.current.undo();
    if (result) {
      try {
        // Apply the before state to restore the meal plan
        const restorationResult = await mealPlanStateService.applySnapshot(
          mealPlan,
          result.newState
        );
        
        if (restorationResult.success) {
          console.log('Undoing change:', result.change.description);
          console.log('Changes applied:', restorationResult.changesApplied);
          
          // TODO: Update the meal plan state in parent component
          // This would typically trigger a refresh or state update
        } else {
          console.error('Failed to undo change:', restorationResult.error);
          // Revert the undo in the change tracker
          changeTracker.current.redo();
        }
      } catch (error) {
        console.error('Error during undo operation:', error);
        // Revert the undo in the change tracker
        changeTracker.current.redo();
      }
      
      updateHistoryState();
    }
  };

  const handleRedo = async () => {
    if (!changeTracker.current || !mealPlan) return;
    
    const result = changeTracker.current.redo();
    if (result) {
      try {
        // Apply the after state to restore the meal plan
        const restorationResult = await mealPlanStateService.applySnapshot(
          mealPlan,
          result.newState
        );
        
        if (restorationResult.success) {
          console.log('Redoing change:', result.change.description);
          console.log('Changes applied:', restorationResult.changesApplied);
          
          // TODO: Update the meal plan state in parent component
          // This would typically trigger a refresh or state update
        } else {
          console.error('Failed to redo change:', restorationResult.error);
          // Revert the redo in the change tracker
          changeTracker.current.undo();
        }
      } catch (error) {
        console.error('Error during redo operation:', error);
        // Revert the redo in the change tracker
        changeTracker.current.undo();
      }
      
      updateHistoryState();
    }
  };

  const handleJumpToChange = async (index: number) => {
    if (!changeTracker.current || !mealPlan) return;
    
    try {
      // Get the target change's after state
      const changes = changeTracker.current.getHistory();
      const targetChange = changes[index];
      
      if (!targetChange) {
        console.error('Invalid change index:', index);
        return;
      }
      
      // Apply the target state
      const restorationResult = await mealPlanStateService.applySnapshot(
        mealPlan,
        targetChange.afterState
      );
      
      if (restorationResult.success) {
        // Update the change tracker's current index
        // This is a simplified approach - in a full implementation,
        // we'd need a method to set the current index directly
        console.log('Jumped to change:', targetChange.description);
        console.log('Changes applied:', restorationResult.changesApplied);
        
        // TODO: Update the meal plan state in parent component
        // TODO: Update change tracker's internal state to reflect the jump
      } else {
        console.error('Failed to jump to change:', restorationResult.error);
      }
    } catch (error) {
      console.error('Error during jump to change:', error);
    }
  };

  const handleClearHistory = () => {
    if (!changeTracker.current) return;
    
    changeTracker.current.clear();
    updateHistoryState();
  };

  const handleHistoryRefresh = async () => {
    setHistoryRefreshing(true);
    // In a real app, this might reload the meal plan data
    setTimeout(() => {
      updateHistoryState();
      setHistoryRefreshing(false);
    }, 1000);
  };
  if (loading) {
    return (
      <View style={styles.centerContainer}>
        <ActivityIndicator size="large" color="#007AFF" />
        <Text style={styles.loadingText}>Loading meal plan...</Text>
      </View>
    );
  }

  if (error) {
    return (
      <View style={styles.centerContainer}>
        <Text style={styles.errorIcon}>⚠️</Text>
        <Text style={styles.errorText}>Failed to load meal plan</Text>
        <Text style={styles.errorDetails}>{error}</Text>
      </View>
    );
  }

  if (!mealPlan) {
    return (
      <EmptyMealPlanState
        title="No Meal Plan Yet"
        message="Create your first meal plan to see your weekly schedule"
        actionText="Create Meal Plan"
        onActionPress={() => console.log('Create meal plan')}
      />
    );
  }

  const getMealForSlot = (day: DayOfWeek, mealType: MealType): MealSlotWithRecipe | undefined => {
    const dayMeals = mealPlan.populatedMeals[day];
    return dayMeals?.find(meal => meal.mealType === mealType);
  };

  const handleMealPress = (day: DayOfWeek, mealType: MealType) => {
    const meal = getMealForSlot(day, mealType);
    if (meal && meal.recipeId) {
      onMealPress?.(day, mealType, meal);
    } else if (onEmptySlotPress) {
      onEmptySlotPress(day, mealType);
    }
  };

  const handleMealLongPress = (day: DayOfWeek, mealType: MealType) => {
    const meal = getMealForSlot(day, mealType);
    onMealLongPress?.(day, mealType, meal);
  };

  const formatWeekDate = (date: Date): string => {
    return date.toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
    });
  };

  const getWeekDates = (weekStart: Date) => {
    const dates = [];
    const start = new Date(weekStart);
    
    for (let i = 0; i < 7; i++) {
      const date = new Date(start);
      date.setDate(start.getDate() + i);
      dates.push(date);
    }
    
    return dates;
  };

  const weekDates = getWeekDates(new Date(mealPlan.weekStartDate));
  const isToday = (date: Date) => {
    const today = new Date();
    return date.toDateString() === today.toDateString();
  };

  return (
    <View style={styles.container}>
      {/* Undo/Redo Controls */}
      {isEditable && (historyState.canUndo || historyState.canRedo) && (
        <View style={styles.undoRedoContainer}>
          <UndoRedoControls
            canUndo={historyState.canUndo}
            canRedo={historyState.canRedo}
            onUndo={handleUndo}
            onRedo={handleRedo}
            undoDescription={
              historyState.canUndo && changes[historyState.currentIndex]
                ? changes[historyState.currentIndex].description
                : undefined
            }
            redoDescription={
              historyState.canRedo && changes[historyState.currentIndex + 1]
                ? changes[historyState.currentIndex + 1].description
                : undefined
            }
            size="small"
            style="outlined"
          />
          
          <TouchableOpacity
            style={styles.historyButton}
            onPress={() => setHistoryPanelVisible(true)}
            accessibilityRole="button"
            accessibilityLabel="View change history"
          >
            <Text style={styles.historyButtonText}>📋</Text>
          </TouchableOpacity>
        </View>
      )}
      
      <ScrollView 
        horizontal 
        showsHorizontalScrollIndicator={false}
        contentContainerStyle={styles.scrollContainer}
      >
        {DAYS_OF_WEEK.map((day, dayIndex) => {
          const dayDate = weekDates[dayIndex];
          const todayStyle = isToday(dayDate) ? styles.todayColumn : {};
          
          return (
            <View key={day.key} style={[styles.dayColumn, todayStyle]}>
              {/* Day Header */}
              <View style={[styles.dayHeader, isToday(dayDate) && styles.todayHeader]}>
                <Text style={[styles.dayLabel, isToday(dayDate) && styles.todayDayLabel]}>
                  {day.label}
                </Text>
                <Text style={[styles.dayDate, isToday(dayDate) && styles.todayDayDate]}>
                  {formatWeekDate(dayDate)}
                </Text>
              </View>

              {/* Meal Slots */}
              {MEAL_TYPES.map(mealType => {
                const meal = getMealForSlot(day.key, mealType.key);
                const isEmpty = !meal || !meal.recipeId;

                const isDragging = draggedMeal?.day === day.key && draggedMeal?.mealType === mealType.key;
                const isDragTarget = draggedMeal !== null && !isDragging;

                return (
                  <MealSlot
                    key={`${day.key}-${mealType.key}`}
                    day={day.key}
                    mealType={mealType.key}
                    meal={meal}
                    mealTypeLabel={mealType.label}
                    mealTypeIcon={mealType.icon}
                    isEmpty={isEmpty}
                    isEditable={isEditable}
                    dragEnabled={isEditable && !isEmpty}
                    dropEnabled={isEditable}
                    isDragTarget={isDragTarget}
                    isDragging={isDragging}
                    showLockToggle={isEditable}
                    showSwapButton={isEditable}
                    onPress={() => handleMealPress(day.key, mealType.key)}
                    onLongPress={() => handleMealLongPress(day.key, mealType.key)}
                    onDragStart={handleDragStart}
                    onDragEnd={handleDragEnd}
                    onLockToggle={handleLockToggle}
                    onSwapPress={handleSwapPress}
                  />
                );
              })}
            </View>
          );
        })}
      </ScrollView>

      {/* Week Summary */}
      {mealPlan.totalEstimatedTime > 0 && (
        <View style={styles.summary}>
          <Text style={styles.summaryText}>
            Total cooking time: {Math.floor(mealPlan.totalEstimatedTime / 60)}h {mealPlan.totalEstimatedTime % 60}m
          </Text>
          {mealPlan.completionPercentage !== undefined && (
            <Text style={styles.completionText}>
              Progress: {Math.round(mealPlan.completionPercentage)}%
            </Text>
          )}
        </View>
      )}

      {/* Quick Swap Modal */}
      {swapModalVisible && swapModalData && (
        <QuickSwapModal
          visible={swapModalVisible}
          currentMeal={swapModalData.meal}
          day={swapModalData.day}
          mealType={swapModalData.mealType}
          onClose={handleSwapModalClose}
          onSwapConfirmed={handleSwapConfirmed}
          onGetSuggestions={handleGetSwapSuggestions}
          onPreviewShoppingListChanges={handlePreviewShoppingListChanges}
        />
      )}
      
      {/* Change History Panel */}
      <ChangeHistoryPanel
        visible={historyPanelVisible}
        onClose={() => setHistoryPanelVisible(false)}
        changes={changes}
        currentIndex={historyState.currentIndex}
        canUndo={historyState.canUndo}
        canRedo={historyState.canRedo}
        onUndo={handleUndo}
        onRedo={handleRedo}
        onJumpToChange={handleJumpToChange}
        onClearHistory={handleClearHistory}
        refreshing={historyRefreshing}
        onRefresh={handleHistoryRefresh}
      />
    </View>
  );
};

// Themed style functions
const getDayColumnStyles = (colors: ColorTokens, isToday: boolean) => ({
  width: (screenWidth - 32) / 7,
  marginRight: 1,
  backgroundColor: isToday ? colors.primaryLight + '20' : colors.surface,
  borderRadius: 8,
  overflow: 'hidden' as const,
  marginBottom: 8,
  borderWidth: isToday ? 2 : 0,
  borderColor: isToday ? colors.primary : 'transparent',
});

const getDayHeaderStyles = (colors: ColorTokens, isToday: boolean) => ({
  padding: 12,
  backgroundColor: isToday ? colors.primary : colors.backgroundSecondary,
  alignItems: 'center' as const,
  borderBottomWidth: 1,
  borderBottomColor: colors.border,
});

const getDayLabelStyles = (colors: ColorTokens, isToday: boolean) => ({
  fontSize: 12,
  fontWeight: '600' as const,
  color: isToday ? colors.textInverse : colors.text,
});

const getDayDateStyles = (colors: ColorTokens, isToday: boolean) => ({
  fontSize: 10,
  color: isToday ? colors.textInverse : colors.textSecondary,
  marginTop: 2,
});

const getSummaryStyles = (colors: ColorTokens) => ({
  backgroundColor: colors.surface,
  padding: 16,
  margin: 16,
  borderRadius: 12,
  shadowColor: colors.text,
  shadowOffset: { width: 0, height: 2 },
  shadowOpacity: 0.1,
  shadowRadius: 4,
  elevation: 3,
});

const getSummaryTextStyles = (colors: ColorTokens) => ({
  fontSize: 14,
  color: colors.text,
  marginBottom: 4,
});

const getCompletionTextStyles = (colors: ColorTokens) => ({
  fontSize: 14,
  color: colors.success,
  fontWeight: '600' as const,
});

const styles = StyleSheet.create({
  centerContainer: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    padding: 20,
  },
  loadingText: {
    marginTop: 10,
    fontSize: 16,
  },
  errorIcon: {
    fontSize: 48,
    marginBottom: 16,
  },
  errorText: {
    fontSize: 18,
    fontWeight: '600',
    marginBottom: 8,
  },
  errorDetails: {
    fontSize: 14,
    textAlign: 'center',
  },
  scrollContainer: {
    paddingHorizontal: 16,
    paddingBottom: 16,
  },
  undoRedoContainer: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    paddingHorizontal: 16,
    paddingVertical: 8,
    borderBottomWidth: 1,
  },
  historyButton: {
    padding: 8,
    borderRadius: 20,
    marginLeft: 12,
  },
  historyButtonText: {
    fontSize: 16,
  },
});
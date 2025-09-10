import React, { useState, useEffect, useCallback } from 'react';
import {
  View,
  StyleSheet,
  SafeAreaView,
  Alert,
  RefreshControl,
  ScrollView,
} from 'react-native';
import type {
  MealPlanResponse,
  DayOfWeek,
  MealType,
  MealSlotWithRecipe,
} from '@imkitchen/shared-types';
import { MealPlanGrid } from '../../components/organisms/MealPlanGrid';
import { WeekNavigator } from '../../components/atoms/WeekNavigator';
import { ErrorBoundary } from '../../components/atoms/ErrorBoundary';
import { useTheme } from '../../theme/ThemeProvider';
// import { useMealPlanStore } from '../../store/meal_plan_store'; // Will implement later

interface MealPlanScreenProps {
  navigation: any; // In a real app, this would be properly typed
}

export const MealPlanScreen: React.FC<MealPlanScreenProps> = ({ navigation }) => {
  const { colors } = useTheme();
  const [currentWeek, setCurrentWeek] = useState<Date>(() => {
    // Start with current Monday
    const today = new Date();
    const monday = new Date(today);
    monday.setDate(today.getDate() - today.getDay() + 1);
    monday.setHours(0, 0, 0, 0);
    return monday;
  });
  
  const [mealPlan, setMealPlan] = useState<MealPlanResponse | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [refreshing, setRefreshing] = useState(false);

  // Mock data for now - will be replaced with actual API calls
  const loadMealPlan = useCallback(async (weekStart: Date) => {
    setLoading(true);
    setError(null);
    
    try {
      // Simulate API call
      await new Promise(resolve => setTimeout(resolve, 1000));
      
      // Mock meal plan data
      const mockMealPlan: MealPlanResponse = {
        id: 'mock-meal-plan-1',
        userId: 'user-1',
        weekStartDate: weekStart,
        generationType: 'manual',
        generatedAt: new Date(),
        totalEstimatedTime: 420, // 7 hours total
        isActive: true,
        status: 'active',
        completionPercentage: 30,
        populatedMeals: {
          monday: [
            {
              day: 'monday',
              mealType: 'breakfast',
              servings: 2,
              isCompleted: true,
              recipe: {
                id: 'recipe-1',
                title: 'Avocado Toast',
                prepTime: 10,
                cookTime: 5,
                totalTime: 15,
                complexity: 'simple',
                mealType: ['breakfast'],
                servings: 2,
                ingredients: [],
                instructions: [],
                dietaryLabels: ['vegetarian'],
                averageRating: 4.5,
                totalRatings: 12,
                createdAt: new Date(),
                updatedAt: new Date(),
                imageUrl: 'https://example.com/avocado-toast.jpg',
              },
            },
            {
              day: 'monday',
              mealType: 'lunch',
              servings: 2,
              isCompleted: false,
              recipe: {
                id: 'recipe-2',
                title: 'Caesar Salad',
                prepTime: 15,
                cookTime: 0,
                totalTime: 15,
                complexity: 'simple',
                mealType: ['lunch'],
                servings: 2,
                ingredients: [],
                instructions: [],
                dietaryLabels: ['vegetarian'],
                averageRating: 4.2,
                totalRatings: 8,
                createdAt: new Date(),
                updatedAt: new Date(),
              },
            },
          ],
          tuesday: [],
          wednesday: [],
          thursday: [],
          friday: [],
          saturday: [],
          sunday: [],
        },
        createdAt: new Date(),
        updatedAt: new Date(),
      };
      
      setMealPlan(mockMealPlan);
    } catch (err) {
      setError('Failed to load meal plan');
      console.error('Error loading meal plan:', err);
    } finally {
      setLoading(false);
      setRefreshing(false);
    }
  }, []);

  useEffect(() => {
    loadMealPlan(currentWeek);
  }, [currentWeek, loadMealPlan]);

  const handlePreviousWeek = () => {
    const prevWeek = new Date(currentWeek);
    prevWeek.setDate(prevWeek.getDate() - 7);
    setCurrentWeek(prevWeek);
  };

  const handleNextWeek = () => {
    const nextWeek = new Date(currentWeek);
    nextWeek.setDate(nextWeek.getDate() + 7);
    setCurrentWeek(nextWeek);
  };

  const handleMealPress = (day: DayOfWeek, mealType: MealType, meal?: MealSlotWithRecipe) => {
    if (meal?.recipe) {
      // Navigate to recipe detail
      navigation.navigate('RecipeDetail', { recipeId: meal.recipe.id });
    }
  };

  const handleMealLongPress = (day: DayOfWeek, mealType: MealType, meal?: MealSlotWithRecipe) => {
    const options = ['Edit Meal', 'Remove Meal', 'Mark as Completed', 'Cancel'];
    
    Alert.alert(
      'Meal Options',
      `${day.charAt(0).toUpperCase() + day.slice(1)} ${mealType}`,
      options.map((option, index) => ({
        text: option,
        style: option === 'Cancel' ? 'cancel' : 'default',
        onPress: () => {
          switch (index) {
            case 0: // Edit
              console.log('Edit meal', day, mealType);
              break;
            case 1: // Remove
              console.log('Remove meal', day, mealType);
              break;
            case 2: // Mark completed
              console.log('Mark completed', day, mealType);
              break;
          }
        },
      })),
      { cancelable: true }
    );
  };

  const handleEmptySlotPress = (day: DayOfWeek, mealType: MealType) => {
    // Navigate to recipe selection or meal planning screen
    navigation.navigate('RecipeList', {
      returnRoute: 'MealPlan',
      assignToSlot: { day, mealType, weekStart: currentWeek },
    });
  };

  const handleRefresh = useCallback(() => {
    setRefreshing(true);
    loadMealPlan(currentWeek);
  }, [currentWeek, loadMealPlan]);

  return (
    <ErrorBoundary>
      <SafeAreaView style={[styles.container, { backgroundColor: colors.background }]}>
        <WeekNavigator
          currentWeek={currentWeek}
          onPreviousWeek={handlePreviousWeek}
          onNextWeek={handleNextWeek}
          showWeekSelector={true}
        />
        
        <ScrollView
          style={styles.content}
          refreshControl={
            <RefreshControl
              refreshing={refreshing}
              onRefresh={handleRefresh}
              colors={[colors.primary]}
              tintColor={colors.primary}
            />
          }
        >
          <MealPlanGrid
            mealPlan={mealPlan}
            onMealPress={handleMealPress}
            onMealLongPress={handleMealLongPress}
            onEmptySlotPress={handleEmptySlotPress}
            isEditable={true}
            loading={loading}
            error={error}
          />
        </ScrollView>
      </SafeAreaView>
    </ErrorBoundary>
  );
};

const styles = StyleSheet.create({
  container: {
    flex: 1,
  },
  content: {
    flex: 1,
  },
});

export default MealPlanScreen;
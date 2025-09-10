import React, { useState, useEffect } from 'react';
import {
  View,
  Text,
  ScrollView,
  TouchableOpacity,
  StyleSheet,
  ActivityIndicator,
  Dimensions,
} from 'react-native';
import { RecipeCard } from '../molecules/RecipeCard';
import type { CommunityRecipe } from '@imkitchen/shared-types';

interface TrendingRecipesCarouselProps {
  recipes: CommunityRecipe[];
  onRecipePress: (recipe: CommunityRecipe) => void;
  onImportRecipe: (recipe: CommunityRecipe) => void;
  isLoading?: boolean;
  error?: string | null;
  timeFilter?: 'day' | 'week' | 'month';
  onTimeFilterChange?: (filter: 'day' | 'week' | 'month') => void;
}

const { width: screenWidth } = Dimensions.get('window');
const RECIPE_CARD_WIDTH = screenWidth * 0.75;

export const TrendingRecipesCarousel: React.FC<TrendingRecipesCarouselProps> = ({
  recipes,
  onRecipePress,
  onImportRecipe,
  isLoading = false,
  error = null,
  timeFilter = 'week',
  onTimeFilterChange,
}) => {
  const [selectedTimeFilter, setSelectedTimeFilter] = useState(timeFilter);

  useEffect(() => {
    setSelectedTimeFilter(timeFilter);
  }, [timeFilter]);

  const handleTimeFilterChange = (filter: 'day' | 'week' | 'month') => {
    setSelectedTimeFilter(filter);
    onTimeFilterChange?.(filter);
  };

  const renderTimeFilterButton = (filter: 'day' | 'week' | 'month', label: string) => (
    <TouchableOpacity
      key={filter}
      style={[
        styles.filterButton,
        selectedTimeFilter === filter && styles.activeFilterButton,
      ]}
      onPress={() => handleTimeFilterChange(filter)}
      accessibilityRole="button"
      accessibilityLabel={`Filter trending recipes by ${label.toLowerCase()}`}
      accessibilityState={{ selected: selectedTimeFilter === filter }}
    >
      <Text
        style={[
          styles.filterButtonText,
          selectedTimeFilter === filter && styles.activeFilterButtonText,
        ]}
      >
        {label}
      </Text>
    </TouchableOpacity>
  );

  const renderRecipeCard = (recipe: CommunityRecipe, index: number) => (
    <View key={recipe.id} style={styles.recipeCardContainer}>
      <RecipeCard
        recipe={{
          id: recipe.id,
          title: recipe.title,
          description: recipe.description || '',
          imageUrl: recipe.imageURL,
          prepTime: recipe.prepTime,
          cookTime: recipe.cookTime,
          totalTime: recipe.totalTime,
          complexity: recipe.complexity,
          cuisineType: recipe.cuisineType,
          mealType: recipe.mealType,
          servings: recipe.servings,
          averageRating: recipe.averageRating,
          totalRatings: recipe.totalRatings,
          ingredients: [], // Would be loaded separately
          instructions: [], // Would be loaded separately
          dietaryLabels: [],
          isPublic: true,
          userID: '',
          createdAt: recipe.createdAt,
          updatedAt: recipe.updatedAt,
        }}
        onPress={() => onRecipePress(recipe)}
        showRatingInteraction={false}
        style={styles.recipeCard}
      />
      
      {/* Trending Badge */}
      <View style={styles.trendingBadge}>
        <Text style={styles.trendingBadgeText}>🔥 #{index + 1}</Text>
      </View>
      
      {/* Quick Import Button */}
      <TouchableOpacity
        style={styles.quickImportButton}
        onPress={() => onImportRecipe(recipe)}
        accessibilityRole="button"
        accessibilityLabel={`Import ${recipe.title}`}
      >
        <Text style={styles.quickImportText}>Import</Text>
      </TouchableOpacity>
      
      {/* Recipe Stats */}
      <View style={styles.recipeStats}>
        <Text style={styles.statText}>
          📥 {recipe.importCount} imports
        </Text>
        <Text style={styles.statText}>
          📈 Trending score: {recipe.trendingScore.toFixed(1)}
        </Text>
      </View>
    </View>
  );

  const renderEmptyState = () => (
    <View style={styles.emptyState}>
      <Text style={styles.emptyStateTitle}>No Trending Recipes</Text>
      <Text style={styles.emptyStateSubtitle}>
        Check back later for the latest trending recipes!
      </Text>
    </View>
  );

  const renderErrorState = () => (
    <View style={styles.errorState}>
      <Text style={styles.errorTitle}>Failed to Load Trending Recipes</Text>
      <Text style={styles.errorMessage}>{error}</Text>
    </View>
  );

  const renderLoadingState = () => (
    <View style={styles.loadingState}>
      <ActivityIndicator size="large" color="#007AFF" />
      <Text style={styles.loadingText}>Loading trending recipes...</Text>
    </View>
  );

  return (
    <View style={styles.container}>
      {/* Header */}
      <View style={styles.header}>
        <Text style={styles.title}>Trending Recipes</Text>
        {onTimeFilterChange && (
          <View style={styles.timeFilters}>
            {renderTimeFilterButton('day', 'Today')}
            {renderTimeFilterButton('week', 'Week')}
            {renderTimeFilterButton('month', 'Month')}
          </View>
        )}
      </View>

      {/* Content */}
      {isLoading && renderLoadingState()}
      {error && renderErrorState()}
      {!isLoading && !error && recipes.length === 0 && renderEmptyState()}
      {!isLoading && !error && recipes.length > 0 && (
        <ScrollView
          horizontal
          showsHorizontalScrollIndicator={false}
          contentContainerStyle={styles.scrollContent}
          snapToInterval={RECIPE_CARD_WIDTH + 16}
          decelerationRate="fast"
          snapToAlignment="start"
        >
          {recipes.map((recipe, index) => renderRecipeCard(recipe, index))}
        </ScrollView>
      )}
    </View>
  );
};

const styles = StyleSheet.create({
  container: {
    backgroundColor: '#fff',
  },
  header: {
    paddingHorizontal: 20,
    paddingVertical: 16,
    borderBottomWidth: 1,
    borderBottomColor: '#f0f0f0',
  },
  title: {
    fontSize: 20,
    fontWeight: '700',
    color: '#333',
    marginBottom: 12,
  },
  timeFilters: {
    flexDirection: 'row',
    gap: 8,
  },
  filterButton: {
    paddingHorizontal: 16,
    paddingVertical: 8,
    borderRadius: 20,
    borderWidth: 1,
    borderColor: '#e0e0e0',
    backgroundColor: '#f5f5f5',
  },
  activeFilterButton: {
    backgroundColor: '#007AFF',
    borderColor: '#007AFF',
  },
  filterButtonText: {
    fontSize: 14,
    fontWeight: '600',
    color: '#666',
  },
  activeFilterButtonText: {
    color: '#fff',
  },
  scrollContent: {
    paddingHorizontal: 20,
    paddingVertical: 16,
  },
  recipeCardContainer: {
    width: RECIPE_CARD_WIDTH,
    marginRight: 16,
    position: 'relative',
  },
  recipeCard: {
    width: '100%',
    marginBottom: 0,
  },
  trendingBadge: {
    position: 'absolute',
    top: 12,
    left: 12,
    backgroundColor: '#FF6B35',
    paddingHorizontal: 8,
    paddingVertical: 4,
    borderRadius: 12,
    zIndex: 2,
  },
  trendingBadgeText: {
    fontSize: 12,
    fontWeight: '700',
    color: '#fff',
  },
  quickImportButton: {
    position: 'absolute',
    top: 12,
    right: 12,
    backgroundColor: '#007AFF',
    paddingHorizontal: 12,
    paddingVertical: 6,
    borderRadius: 16,
    zIndex: 2,
  },
  quickImportText: {
    fontSize: 12,
    fontWeight: '600',
    color: '#fff',
  },
  recipeStats: {
    padding: 12,
    backgroundColor: '#f9f9f9',
    marginTop: 8,
    borderRadius: 8,
  },
  statText: {
    fontSize: 12,
    color: '#666',
    marginBottom: 2,
  },
  loadingState: {
    alignItems: 'center',
    justifyContent: 'center',
    paddingVertical: 60,
  },
  loadingText: {
    fontSize: 14,
    color: '#666',
    marginTop: 12,
  },
  emptyState: {
    alignItems: 'center',
    justifyContent: 'center',
    paddingVertical: 60,
    paddingHorizontal: 20,
  },
  emptyStateTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#333',
    marginBottom: 8,
    textAlign: 'center',
  },
  emptyStateSubtitle: {
    fontSize: 14,
    color: '#666',
    textAlign: 'center',
  },
  errorState: {
    alignItems: 'center',
    justifyContent: 'center',
    paddingVertical: 60,
    paddingHorizontal: 20,
  },
  errorTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#333',
    marginBottom: 8,
    textAlign: 'center',
  },
  errorMessage: {
    fontSize: 14,
    color: '#666',
    textAlign: 'center',
  },
});
import React, { useState, useEffect, useCallback } from 'react';
import {
  View,
  Text,
  Modal,
  TouchableOpacity,
  ScrollView,
  TextInput,
  ActivityIndicator,
  StyleSheet,
  SafeAreaView,
} from 'react-native';
import type {
  Recipe,
  DayOfWeek,
  MealType,
  MealSlotWithRecipe,
} from '@imkitchen/shared-types';

export interface SwapSuggestion {
  recipe: Recipe;
  compatibilityScore: number;
  reasons: string[];
  timeDifference: number; // minutes difference from original
  complexityMatch: boolean;
  cuisineMatch: boolean;
  shoppingListImpact: {
    itemsAdded: number;
    itemsRemoved: number;
    estimatedCostChange: number;
  };
}

export interface SwapFilters {
  maxPrepTime?: number;
  complexity?: 'simple' | 'moderate' | 'complex';
  cuisine?: string;
  maxTimeDifference?: number;
  excludeRecipeIds?: string[];
}

interface QuickSwapModalProps {
  visible: boolean;
  currentMeal: MealSlotWithRecipe;
  day: DayOfWeek;
  mealType: MealType;
  onClose: () => void;
  onSwapConfirmed: (newRecipeId: string) => Promise<void>;
  onGetSuggestions: (filters: SwapFilters) => Promise<SwapSuggestion[]>;
  onPreviewShoppingListChanges?: (recipeId: string) => Promise<{
    itemsAdded: number;
    itemsRemoved: number;
    estimatedCostChange: number;
  }>;
}

export const QuickSwapModal: React.FC<QuickSwapModalProps> = ({
  visible,
  currentMeal,
  day,
  mealType,
  onClose,
  onSwapConfirmed,
  onGetSuggestions,
  onPreviewShoppingListChanges,
}) => {
  const [suggestions, setSuggestions] = useState<SwapSuggestion[]>([]);
  const [loading, setLoading] = useState(false);
  const [swapping, setSwapping] = useState(false);
  const [searchQuery, setSearchQuery] = useState('');
  const [filters, setFilters] = useState<SwapFilters>({
    maxTimeDifference: 30, // 30 minutes
    excludeRecipeIds: [currentMeal.recipe?.id],
  });
  const [selectedSuggestion, setSelectedSuggestion] = useState<SwapSuggestion | null>(null);
  const [showFilters, setShowFilters] = useState(false);

  const loadSuggestions = useCallback(async () => {
    if (!currentMeal.recipe) return;

    setLoading(true);
    try {
      const searchFilters = {
        ...filters,
        excludeRecipeIds: [currentMeal.recipe.id, ...(filters.excludeRecipeIds || [])],
      };
      
      const newSuggestions = await onGetSuggestions(searchFilters);
      setSuggestions(newSuggestions);
    } catch (error) {
      console.error('Failed to load swap suggestions:', error);
      setSuggestions([]);
    } finally {
      setLoading(false);
    }
  }, [currentMeal.recipe, filters, onGetSuggestions]);

  useEffect(() => {
    if (visible && currentMeal.recipe) {
      loadSuggestions();
    }
  }, [visible, loadSuggestions]);

  const handleSwapConfirm = async () => {
    if (!selectedSuggestion) return;

    setSwapping(true);
    try {
      await onSwapConfirmed(selectedSuggestion.recipe.id);
      onClose();
    } catch (error) {
      console.error('Failed to swap meal:', error);
    } finally {
      setSwapping(false);
    }
  };

  const handleSuggestionSelect = async (suggestion: SwapSuggestion) => {
    setSelectedSuggestion(suggestion);
    
    // Update shopping list preview if handler is provided
    if (onPreviewShoppingListChanges) {
      try {
        const impact = await onPreviewShoppingListChanges(suggestion.recipe.id);
        // Update suggestion with real shopping list data
        const updatedSuggestion = {
          ...suggestion,
          shoppingListImpact: impact,
        };
        setSelectedSuggestion(updatedSuggestion);
      } catch (error) {
        console.error('Failed to preview shopping list changes:', error);
      }
    }
  };

  const filteredSuggestions = suggestions.filter(suggestion => {
    if (!searchQuery) return true;
    return suggestion.recipe.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
           suggestion.recipe.ingredients.some(ing => 
             ing.name.toLowerCase().includes(searchQuery.toLowerCase())
           );
  });

  const formatTime = (minutes: number) => {
    if (minutes < 60) return `${minutes}m`;
    const hours = Math.floor(minutes / 60);
    const remainingMinutes = minutes % 60;
    return remainingMinutes > 0 ? `${hours}h ${remainingMinutes}m` : `${hours}h`;
  };

  const getCompatibilityColor = (score: number) => {
    if (score >= 0.8) return '#4CAF50'; // Excellent match
    if (score >= 0.6) return '#FF9800'; // Good match
    return '#F44336'; // Fair match
  };

  const renderFilters = () => (
    <View style={styles.filtersContainer}>
      <View style={styles.filterRow}>
        <Text style={styles.filterLabel}>Max prep time:</Text>
        <View style={styles.timeFilterButtons}>
          {[15, 30, 60, 120].map(time => (
            <TouchableOpacity
              key={time}
              style={[
                styles.timeButton,
                filters.maxPrepTime === time && styles.activeTimeButton,
              ]}
              onPress={() => setFilters(prev => ({ ...prev, maxPrepTime: time }))}
            >
              <Text style={[
                styles.timeButtonText,
                filters.maxPrepTime === time && styles.activeTimeButtonText,
              ]}>
                {formatTime(time)}
              </Text>
            </TouchableOpacity>
          ))}
        </View>
      </View>

      <View style={styles.filterRow}>
        <Text style={styles.filterLabel}>Complexity:</Text>
        <View style={styles.complexityButtons}>
          {['simple', 'moderate', 'complex'].map(complexity => (
            <TouchableOpacity
              key={complexity}
              style={[
                styles.complexityButton,
                filters.complexity === complexity && styles.activeComplexityButton,
              ]}
              onPress={() => setFilters(prev => ({ 
                ...prev, 
                complexity: prev.complexity === complexity ? undefined : complexity as any
              }))}
            >
              <Text style={[
                styles.complexityButtonText,
                filters.complexity === complexity && styles.activeComplexityButtonText,
              ]}>
                {complexity}
              </Text>
            </TouchableOpacity>
          ))}
        </View>
      </View>

      <TouchableOpacity
        style={styles.applyFiltersButton}
        onPress={() => {
          setShowFilters(false);
          loadSuggestions();
        }}
      >
        <Text style={styles.applyFiltersButtonText}>Apply Filters</Text>
      </TouchableOpacity>
    </View>
  );

  const renderSuggestionItem = (suggestion: SwapSuggestion, index: number) => (
    <TouchableOpacity
      key={suggestion.recipe.id}
      style={[
        styles.suggestionCard,
        selectedSuggestion?.recipe.id === suggestion.recipe.id && styles.selectedSuggestionCard,
      ]}
      onPress={() => handleSuggestionSelect(suggestion)}
      accessibilityRole="button"
      accessibilityLabel={`Replace with ${suggestion.recipe.title}`}
      accessibilityHint={`${Math.round(suggestion.compatibilityScore * 100)}% match, ${formatTime(suggestion.recipe.totalTime || 0)} cooking time`}
    >
      <View style={styles.suggestionHeader}>
        <View style={styles.suggestionTitleContainer}>
          <Text style={styles.suggestionTitle} numberOfLines={2}>
            {suggestion.recipe.title}
          </Text>
          <View style={[
            styles.compatibilityScore,
            { backgroundColor: getCompatibilityColor(suggestion.compatibilityScore) }
          ]}>
            <Text style={styles.compatibilityScoreText}>
              {Math.round(suggestion.compatibilityScore * 100)}%
            </Text>
          </View>
        </View>
        
        <View style={styles.suggestionMeta}>
          <Text style={styles.suggestionTime}>
            {formatTime(suggestion.recipe.totalTime || 0)}
          </Text>
          <Text style={styles.suggestionComplexity}>
            {suggestion.recipe.complexity}
          </Text>
        </View>
      </View>

      <View style={styles.reasonsContainer}>
        <Text style={styles.reasonsTitle}>Why this match:</Text>
        {suggestion.reasons.slice(0, 2).map((reason, idx) => (
          <Text key={idx} style={styles.reasonText}>
            • {reason}
          </Text>
        ))}
      </View>

      <View style={styles.impactContainer}>
        <Text style={styles.impactTitle}>Shopping list impact:</Text>
        <View style={styles.impactRow}>
          <Text style={styles.impactText}>
            +{suggestion.shoppingListImpact.itemsAdded} items
          </Text>
          <Text style={styles.impactText}>
            -{suggestion.shoppingListImpact.itemsRemoved} items
          </Text>
          <Text style={[
            styles.impactText,
            suggestion.shoppingListImpact.estimatedCostChange > 0 ? styles.costIncrease : styles.costDecrease
          ]}>
            {suggestion.shoppingListImpact.estimatedCostChange > 0 ? '+' : ''}
            ${suggestion.shoppingListImpact.estimatedCostChange.toFixed(2)}
          </Text>
        </View>
      </View>

      {selectedSuggestion?.recipe.id === suggestion.recipe.id && (
        <View style={styles.selectedIndicator}>
          <Text style={styles.selectedIndicatorText}>✓ Selected</Text>
        </View>
      )}
    </TouchableOpacity>
  );

  if (!currentMeal.recipe) return null;

  return (
    <Modal
      visible={visible}
      animationType="slide"
      presentationStyle="pageSheet"
      onRequestClose={onClose}
    >
      <SafeAreaView style={styles.container}>
        <View style={styles.header}>
          <View style={styles.headerLeft}>
            <TouchableOpacity onPress={onClose} style={styles.closeButton}>
              <Text style={styles.closeButtonText}>✕</Text>
            </TouchableOpacity>
          </View>

          <View style={styles.headerCenter}>
            <Text style={styles.modalTitle}>Quick Swap</Text>
            <Text style={styles.modalSubtitle}>
              Replace {currentMeal.recipe.title} • {day} {mealType}
            </Text>
          </View>

          <View style={styles.headerRight}>
            <TouchableOpacity
              onPress={() => setShowFilters(!showFilters)}
              style={styles.filtersButton}
            >
              <Text style={styles.filtersButtonText}>Filters</Text>
            </TouchableOpacity>
          </View>
        </View>

        {showFilters && renderFilters()}

        <View style={styles.searchContainer}>
          <TextInput
            style={styles.searchInput}
            placeholder="Search recipes..."
            value={searchQuery}
            onChangeText={setSearchQuery}
            accessibilityLabel="Search recipes"
            accessibilityHint="Search through suggested recipe replacements"
          />
        </View>

        {loading ? (
          <View style={styles.loadingContainer}>
            <ActivityIndicator size="large" color="#007AFF" />
            <Text style={styles.loadingText}>Finding similar recipes...</Text>
          </View>
        ) : (
          <ScrollView style={styles.suggestionsContainer} showsVerticalScrollIndicator={false}>
            {filteredSuggestions.length > 0 ? (
              filteredSuggestions.map(renderSuggestionItem)
            ) : (
              <View style={styles.emptyState}>
                <Text style={styles.emptyStateText}>
                  No matching recipes found
                </Text>
                <Text style={styles.emptyStateSubtext}>
                  Try adjusting your filters or search terms
                </Text>
              </View>
            )}
          </ScrollView>
        )}

        {selectedSuggestion && (
          <View style={styles.footer}>
            <TouchableOpacity
              style={[styles.swapButton, swapping && styles.swapButtonDisabled]}
              onPress={handleSwapConfirm}
              disabled={swapping}
              accessibilityRole="button"
              accessibilityLabel={`Confirm swap to ${selectedSuggestion.recipe.title}`}
            >
              {swapping ? (
                <ActivityIndicator size="small" color="#FFF" />
              ) : (
                <Text style={styles.swapButtonText}>
                  Swap to {selectedSuggestion.recipe.title}
                </Text>
              )}
            </TouchableOpacity>
          </View>
        )}
      </SafeAreaView>
    </Modal>
  );
};

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#F8F9FA',
  },
  header: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    padding: 16,
    backgroundColor: '#FFF',
    borderBottomWidth: 1,
    borderBottomColor: '#E9ECEF',
  },
  headerLeft: {
    flex: 1,
    alignItems: 'flex-start',
  },
  headerCenter: {
    flex: 2,
    alignItems: 'center',
  },
  headerRight: {
    flex: 1,
    alignItems: 'flex-end',
  },
  closeButton: {
    padding: 8,
  },
  closeButtonText: {
    fontSize: 18,
    color: '#6C757D',
  },
  modalTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#343A40',
  },
  modalSubtitle: {
    fontSize: 12,
    color: '#6C757D',
    marginTop: 2,
  },
  filtersButton: {
    paddingVertical: 6,
    paddingHorizontal: 12,
    backgroundColor: '#E9ECEF',
    borderRadius: 16,
  },
  filtersButtonText: {
    fontSize: 14,
    color: '#495057',
    fontWeight: '500',
  },
  filtersContainer: {
    backgroundColor: '#FFF',
    padding: 16,
    borderBottomWidth: 1,
    borderBottomColor: '#E9ECEF',
  },
  filterRow: {
    marginBottom: 12,
  },
  filterLabel: {
    fontSize: 14,
    fontWeight: '600',
    color: '#495057',
    marginBottom: 8,
  },
  timeFilterButtons: {
    flexDirection: 'row',
    gap: 8,
  },
  timeButton: {
    paddingVertical: 6,
    paddingHorizontal: 12,
    backgroundColor: '#E9ECEF',
    borderRadius: 16,
  },
  activeTimeButton: {
    backgroundColor: '#007AFF',
  },
  timeButtonText: {
    fontSize: 12,
    color: '#495057',
    fontWeight: '500',
  },
  activeTimeButtonText: {
    color: '#FFF',
  },
  complexityButtons: {
    flexDirection: 'row',
    gap: 8,
  },
  complexityButton: {
    paddingVertical: 6,
    paddingHorizontal: 12,
    backgroundColor: '#E9ECEF',
    borderRadius: 16,
  },
  activeComplexityButton: {
    backgroundColor: '#28A745',
  },
  complexityButtonText: {
    fontSize: 12,
    color: '#495057',
    fontWeight: '500',
    textTransform: 'capitalize',
  },
  activeComplexityButtonText: {
    color: '#FFF',
  },
  applyFiltersButton: {
    backgroundColor: '#007AFF',
    paddingVertical: 10,
    paddingHorizontal: 20,
    borderRadius: 8,
    alignSelf: 'flex-start',
    marginTop: 8,
  },
  applyFiltersButtonText: {
    color: '#FFF',
    fontSize: 14,
    fontWeight: '600',
  },
  searchContainer: {
    padding: 16,
    backgroundColor: '#FFF',
  },
  searchInput: {
    borderWidth: 1,
    borderColor: '#DEE2E6',
    borderRadius: 8,
    paddingVertical: 10,
    paddingHorizontal: 12,
    fontSize: 16,
    backgroundColor: '#FFF',
  },
  loadingContainer: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    padding: 40,
  },
  loadingText: {
    marginTop: 16,
    fontSize: 16,
    color: '#6C757D',
  },
  suggestionsContainer: {
    flex: 1,
    padding: 16,
  },
  suggestionCard: {
    backgroundColor: '#FFF',
    borderRadius: 12,
    padding: 16,
    marginBottom: 12,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.1,
    shadowRadius: 4,
    elevation: 2,
  },
  selectedSuggestionCard: {
    borderWidth: 2,
    borderColor: '#007AFF',
    backgroundColor: '#F0F9FF',
  },
  suggestionHeader: {
    marginBottom: 12,
  },
  suggestionTitleContainer: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'flex-start',
    marginBottom: 8,
  },
  suggestionTitle: {
    flex: 1,
    fontSize: 16,
    fontWeight: '600',
    color: '#343A40',
    lineHeight: 20,
  },
  compatibilityScore: {
    paddingVertical: 4,
    paddingHorizontal: 8,
    borderRadius: 12,
    marginLeft: 8,
  },
  compatibilityScoreText: {
    fontSize: 12,
    fontWeight: '700',
    color: '#FFF',
  },
  suggestionMeta: {
    flexDirection: 'row',
    gap: 16,
  },
  suggestionTime: {
    fontSize: 14,
    color: '#6C757D',
  },
  suggestionComplexity: {
    fontSize: 14,
    color: '#6C757D',
    textTransform: 'capitalize',
  },
  reasonsContainer: {
    marginBottom: 12,
  },
  reasonsTitle: {
    fontSize: 12,
    fontWeight: '600',
    color: '#495057',
    marginBottom: 4,
  },
  reasonText: {
    fontSize: 12,
    color: '#6C757D',
    lineHeight: 16,
  },
  impactContainer: {
    backgroundColor: '#F8F9FA',
    padding: 8,
    borderRadius: 6,
    marginBottom: 8,
  },
  impactTitle: {
    fontSize: 11,
    fontWeight: '600',
    color: '#495057',
    marginBottom: 4,
  },
  impactRow: {
    flexDirection: 'row',
    gap: 12,
  },
  impactText: {
    fontSize: 11,
    color: '#6C757D',
  },
  costIncrease: {
    color: '#DC3545',
  },
  costDecrease: {
    color: '#28A745',
  },
  selectedIndicator: {
    alignSelf: 'flex-end',
  },
  selectedIndicatorText: {
    fontSize: 12,
    color: '#007AFF',
    fontWeight: '600',
  },
  emptyState: {
    alignItems: 'center',
    padding: 40,
  },
  emptyStateText: {
    fontSize: 16,
    fontWeight: '600',
    color: '#6C757D',
    marginBottom: 8,
  },
  emptyStateSubtext: {
    fontSize: 14,
    color: '#ADB5BD',
    textAlign: 'center',
  },
  footer: {
    padding: 16,
    backgroundColor: '#FFF',
    borderTopWidth: 1,
    borderTopColor: '#E9ECEF',
  },
  swapButton: {
    backgroundColor: '#007AFF',
    paddingVertical: 14,
    borderRadius: 8,
    alignItems: 'center',
  },
  swapButtonDisabled: {
    backgroundColor: '#6C757D',
  },
  swapButtonText: {
    color: '#FFF',
    fontSize: 16,
    fontWeight: '600',
  },
});
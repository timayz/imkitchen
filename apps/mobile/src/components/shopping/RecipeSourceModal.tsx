import React from 'react';
import {
  View,
  Text,
  TouchableOpacity,
  StyleSheet,
  Modal,
  ScrollView,
  Platform,
} from 'react-native';
import type { RecipeSourceModalProps } from '../../types/shopping';

export const RecipeSourceModal: React.FC<RecipeSourceModalProps> = ({
  isVisible,
  onClose,
  ingredientName,
  recipeSources,
}) => {
  // Mock recipe data - in a real app, this would come from a recipe service/store
  const getRecipeDetails = (recipeId: string) => {
    // This would typically fetch from a recipe store/service
    const mockRecipes: { [key: string]: { name: string; mealType: string; difficulty: string } } = {
      '1': { name: 'Grilled Chicken Salad', mealType: 'Lunch', difficulty: 'Easy' },
      '2': { name: 'Pasta Primavera', mealType: 'Dinner', difficulty: 'Medium' },
      '3': { name: 'Berry Smoothie Bowl', mealType: 'Breakfast', difficulty: 'Easy' },
      '4': { name: 'Beef Stir-fry', mealType: 'Dinner', difficulty: 'Medium' },
      '5': { name: 'Avocado Toast', mealType: 'Breakfast', difficulty: 'Easy' },
    };

    return mockRecipes[recipeId] || { 
      name: `Recipe ${recipeId}`, 
      mealType: 'Unknown', 
      difficulty: 'Unknown' 
    };
  };

  const getMealTypeIcon = (mealType: string) => {
    switch (mealType.toLowerCase()) {
      case 'breakfast':
        return '🌅';
      case 'lunch':
        return '☀️';
      case 'dinner':
        return '🌙';
      case 'snack':
        return '🍎';
      default:
        return '🍽️';
    }
  };

  const getDifficultyColor = (difficulty: string) => {
    switch (difficulty.toLowerCase()) {
      case 'easy':
        return '#28a745';
      case 'medium':
        return '#ffc107';
      case 'hard':
        return '#dc3545';
      default:
        return '#6c757d';
    }
  };

  if (!recipeSources || recipeSources.length === 0) {
    return null;
  }

  return (
    <Modal
      visible={isVisible}
      transparent
      animationType="slide"
      onRequestClose={onClose}
    >
      <View style={styles.modalOverlay}>
        <View style={styles.modalContent}>
          {/* Header */}
          <View style={styles.header}>
            <View style={styles.headerContent}>
              <Text style={styles.modalTitle}>Used in Recipes</Text>
              <Text style={styles.ingredientName}>{ingredientName}</Text>
            </View>
            <TouchableOpacity
              style={styles.closeButton}
              onPress={onClose}
              activeOpacity={0.7}
            >
              <Text style={styles.closeButtonText}>✕</Text>
            </TouchableOpacity>
          </View>

          {/* Recipe list */}
          <ScrollView 
            style={styles.recipeList}
            showsVerticalScrollIndicator={false}
          >
            {recipeSources.map((recipeId, index) => {
              const recipe = getRecipeDetails(recipeId);
              
              return (
                <View key={recipeId} style={styles.recipeCard}>
                  <View style={styles.recipeHeader}>
                    <View style={styles.recipeInfo}>
                      <Text style={styles.recipeIcon}>
                        {getMealTypeIcon(recipe.mealType)}
                      </Text>
                      <View style={styles.recipeDetails}>
                        <Text style={styles.recipeName}>{recipe.name}</Text>
                        <View style={styles.recipeMeta}>
                          <Text style={styles.mealType}>{recipe.mealType}</Text>
                          <View style={styles.separator} />
                          <Text style={[
                            styles.difficulty,
                            { color: getDifficultyColor(recipe.difficulty) }
                          ]}>
                            {recipe.difficulty}
                          </Text>
                        </View>
                      </View>
                    </View>
                    
                    {/* Recipe number badge */}
                    <View style={styles.recipeBadge}>
                      <Text style={styles.recipeBadgeText}>
                        #{index + 1}
                      </Text>
                    </View>
                  </View>

                  {/* Usage context (could be expanded with more details) */}
                  <View style={styles.usageContext}>
                    <Text style={styles.usageText}>
                      🥄 This ingredient is needed for this recipe
                    </Text>
                  </View>
                </View>
              );
            })}

            {/* Summary */}
            <View style={styles.summary}>
              <Text style={styles.summaryText}>
                📊 {ingredientName} is used in {recipeSources.length} {recipeSources.length === 1 ? 'recipe' : 'recipes'}
              </Text>
              <Text style={styles.summarySubtext}>
                Make sure to get enough for all your planned meals!
              </Text>
            </View>
          </ScrollView>

          {/* Footer actions */}
          <View style={styles.footer}>
            <TouchableOpacity
              style={styles.footerButton}
              onPress={onClose}
            >
              <Text style={styles.footerButtonText}>Got it!</Text>
            </TouchableOpacity>
          </View>
        </View>
      </View>
    </Modal>
  );
};

const styles = StyleSheet.create({
  modalOverlay: {
    flex: 1,
    backgroundColor: 'rgba(0, 0, 0, 0.5)',
    justifyContent: 'center',
    alignItems: 'center',
    padding: 20,
  },
  modalContent: {
    backgroundColor: '#ffffff',
    borderRadius: 16,
    maxHeight: '80%',
    width: '100%',
    maxWidth: 400,
    overflow: 'hidden',
  },
  header: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    paddingHorizontal: 20,
    paddingVertical: 16,
    backgroundColor: '#f8f9fa',
    borderBottomWidth: 1,
    borderBottomColor: '#e9ecef',
  },
  headerContent: {
    flex: 1,
  },
  modalTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#2d3436',
    marginBottom: 2,
  },
  ingredientName: {
    fontSize: 14,
    color: '#636e72',
    fontWeight: '500',
  },
  closeButton: {
    width: 32,
    height: 32,
    borderRadius: 16,
    backgroundColor: '#e9ecef',
    alignItems: 'center',
    justifyContent: 'center',
    marginLeft: 12,
  },
  closeButtonText: {
    fontSize: 16,
    color: '#6c757d',
    fontWeight: '600',
  },
  recipeList: {
    flex: 1,
    paddingHorizontal: 20,
    paddingVertical: 16,
  },
  recipeCard: {
    backgroundColor: '#ffffff',
    borderRadius: 12,
    borderWidth: 1,
    borderColor: '#e9ecef',
    padding: 16,
    marginBottom: 12,
    shadowColor: '#000',
    shadowOffset: {
      width: 0,
      height: 1,
    },
    shadowOpacity: 0.1,
    shadowRadius: 2,
    elevation: 2,
  },
  recipeHeader: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    marginBottom: 8,
  },
  recipeInfo: {
    flexDirection: 'row',
    alignItems: 'center',
    flex: 1,
  },
  recipeIcon: {
    fontSize: 24,
    marginRight: 12,
  },
  recipeDetails: {
    flex: 1,
  },
  recipeName: {
    fontSize: 16,
    fontWeight: '600',
    color: '#2d3436',
    marginBottom: 4,
  },
  recipeMeta: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  mealType: {
    fontSize: 12,
    color: '#636e72',
    textTransform: 'uppercase',
    fontWeight: '500',
  },
  separator: {
    width: 4,
    height: 4,
    borderRadius: 2,
    backgroundColor: '#dee2e6',
    marginHorizontal: 8,
  },
  difficulty: {
    fontSize: 12,
    fontWeight: '600',
    textTransform: 'uppercase',
  },
  recipeBadge: {
    backgroundColor: '#007bff',
    borderRadius: 12,
    paddingHorizontal: 8,
    paddingVertical: 4,
    marginLeft: 8,
  },
  recipeBadgeText: {
    color: '#ffffff',
    fontSize: 12,
    fontWeight: '600',
  },
  usageContext: {
    backgroundColor: '#f8f9fa',
    borderRadius: 8,
    padding: 8,
  },
  usageText: {
    fontSize: 12,
    color: '#495057',
    fontStyle: 'italic',
  },
  summary: {
    backgroundColor: '#e3f2fd',
    borderRadius: 12,
    padding: 16,
    marginTop: 8,
    alignItems: 'center',
  },
  summaryText: {
    fontSize: 14,
    fontWeight: '600',
    color: '#1976d2',
    textAlign: 'center',
    marginBottom: 4,
  },
  summarySubtext: {
    fontSize: 12,
    color: '#455a64',
    textAlign: 'center',
  },
  footer: {
    paddingHorizontal: 20,
    paddingVertical: 16,
    borderTopWidth: 1,
    borderTopColor: '#e9ecef',
  },
  footerButton: {
    backgroundColor: '#007bff',
    borderRadius: 8,
    paddingVertical: 12,
    alignItems: 'center',
  },
  footerButtonText: {
    color: '#ffffff',
    fontSize: 16,
    fontWeight: '600',
  },
});
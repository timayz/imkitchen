import React, { useEffect, useState } from 'react';
import {
  View,
  Text,
  ScrollView,
  TouchableOpacity,
  StyleSheet,
  Alert,
  Share,
  Dimensions,
} from 'react-native';
import { SafeAreaView } from 'react-native-safe-area-context';
import { useNavigation, useRoute, RouteProp } from '@react-navigation/native';
import type { NativeStackNavigationProp } from '@react-navigation/native-stack';
import type { Recipe, RecipeIngredient, RecipeInstruction } from '@imkitchen/shared-types';
import { useRecipeStore } from '../../store/recipe_store';
import OptimizedRecipeImage from '../../components/atoms/OptimizedRecipeImage';

type RecipeStackParamList = {
  RecipeList: undefined;
  RecipeDetail: { recipeId: string };
  EditRecipe: { recipeId: string };
};

type NavigationProp = NativeStackNavigationProp<RecipeStackParamList>;
type RouteProp = RouteProp<RecipeStackParamList, 'RecipeDetail'>;

const { width: screenWidth } = Dimensions.get('window');

export const RecipeDetailScreen: React.FC = () => {
  const navigation = useNavigation<NavigationProp>();
  const route = useRoute<RouteProp>();
  const { recipeId } = route.params;
  
  const {
    currentRecipe,
    loading,
    error,
    getRecipe,
    deleteRecipe,
    setCurrentRecipe,
    clearError,
  } = useRecipeStore();

  const [servingMultiplier, setServingMultiplier] = useState(1);

  useEffect(() => {
    getRecipe(recipeId);
    
    return () => {
      setCurrentRecipe(null);
    };
  }, [recipeId]);

  useEffect(() => {
    if (currentRecipe) {
      setServingMultiplier(1);
    }
  }, [currentRecipe]);

  const formatTime = (minutes: number): string => {
    if (minutes < 60) {
      return `${minutes} min`;
    }
    const hours = Math.floor(minutes / 60);
    const remainingMinutes = minutes % 60;
    return remainingMinutes > 0 ? `${hours}h ${remainingMinutes}m` : `${hours}h`;
  };

  const getComplexityColor = (complexity: string): string => {
    switch (complexity) {
      case 'simple':
        return '#4CAF50';
      case 'moderate':
        return '#FF9800';
      case 'complex':
        return '#F44336';
      default:
        return '#666';
    }
  };

  const formatMealTypes = (mealTypes: string[]): string => {
    return mealTypes
      .map(type => type.charAt(0).toUpperCase() + type.slice(1))
      .join(', ');
  };

  const adjustIngredientAmount = (amount: number): string => {
    const adjustedAmount = amount * servingMultiplier;
    
    // Handle fractions
    if (adjustedAmount < 1 && adjustedAmount > 0) {
      const fraction = adjustedAmount;
      if (fraction === 0.25) return '1/4';
      if (fraction === 0.33) return '1/3';
      if (fraction === 0.5) return '1/2';
      if (fraction === 0.66) return '2/3';
      if (fraction === 0.75) return '3/4';
      return adjustedAmount.toFixed(2);
    }
    
    // Handle mixed numbers
    if (adjustedAmount >= 1) {
      const wholePart = Math.floor(adjustedAmount);
      const fractionalPart = adjustedAmount - wholePart;
      
      if (fractionalPart === 0) return wholePart.toString();
      
      let fractionString = '';
      if (fractionalPart === 0.25) fractionString = '1/4';
      else if (fractionalPart === 0.33) fractionString = '1/3';
      else if (fractionalPart === 0.5) fractionString = '1/2';
      else if (fractionalPart === 0.66) fractionString = '2/3';
      else if (fractionalPart === 0.75) fractionString = '3/4';
      else fractionString = fractionalPart.toFixed(2);
      
      return fractionString ? `${wholePart} ${fractionString}` : wholePart.toString();
    }
    
    return adjustedAmount.toString();
  };

  const handleEdit = () => {
    navigation.navigate('EditRecipe', { recipeId });
  };

  const handleDelete = () => {
    Alert.alert(
      'Delete Recipe',
      'Are you sure you want to delete this recipe? This action cannot be undone.',
      [
        { text: 'Cancel', style: 'cancel' },
        {
          text: 'Delete',
          style: 'destructive',
          onPress: async () => {
            const success = await deleteRecipe(recipeId);
            if (success) {
              navigation.goBack();
            }
          },
        },
      ]
    );
  };

  const handleShare = async () => {
    if (!currentRecipe) return;

    const shareContent = `${currentRecipe.title}\n\n${
      currentRecipe.description ? `${currentRecipe.description}\n\n` : ''
    }Prep: ${formatTime(currentRecipe.prepTime)} | Cook: ${formatTime(
      currentRecipe.cookTime
    )} | Servings: ${currentRecipe.servings}\n\nShared from imkitchen`;

    try {
      await Share.share({
        message: shareContent,
        title: currentRecipe.title,
      });
    } catch (error) {
      console.error('Error sharing recipe:', error);
    }
  };

  const adjustServings = (multiplier: number) => {
    setServingMultiplier(multiplier);
  };

  const renderRating = (rating: number, totalRatings: number) => {
    if (totalRatings === 0) return null;

    const stars = Math.round(rating);
    const starArray = Array.from({ length: 5 }, (_, index) => (
      <Text
        key={index}
        style={[
          styles.star,
          { color: index < stars ? '#FFD700' : '#DDD' }
        ]}
      >
        ★
      </Text>
    ));

    return (
      <View style={styles.ratingContainer}>
        <View style={styles.stars}>{starArray}</View>
        <Text style={styles.ratingText}>({totalRatings} review{totalRatings !== 1 ? 's' : ''})</Text>
      </View>
    );
  };

  const renderIngredient = (ingredient: RecipeIngredient, index: number) => (
    <View key={index} style={styles.ingredientItem}>
      <View style={styles.ingredientAmount}>
        <Text style={styles.ingredientAmountText}>
          {adjustIngredientAmount(ingredient.amount)} {ingredient.unit}
        </Text>
      </View>
      <Text style={styles.ingredientName}>{ingredient.name}</Text>
    </View>
  );

  const renderInstruction = (instruction: RecipeInstruction, index: number) => (
    <View key={index} style={styles.instructionItem}>
      <View style={styles.stepNumber}>
        <Text style={styles.stepNumberText}>{instruction.stepNumber}</Text>
      </View>
      <View style={styles.instructionContent}>
        <Text style={styles.instructionText}>{instruction.instruction}</Text>
        {instruction.estimatedMinutes && (
          <Text style={styles.instructionTime}>
            ~{instruction.estimatedMinutes} min
          </Text>
        )}
      </View>
    </View>
  );

  if (loading && !currentRecipe) {
    return (
      <SafeAreaView style={styles.container}>
        <View style={styles.loadingContainer}>
          <Text style={styles.loadingText}>Loading recipe...</Text>
        </View>
      </SafeAreaView>
    );
  }

  if (error && !currentRecipe) {
    return (
      <SafeAreaView style={styles.container}>
        <View style={styles.errorContainer}>
          <Text style={styles.errorText}>{error}</Text>
          <TouchableOpacity onPress={() => getRecipe(recipeId)} style={styles.retryButton}>
            <Text style={styles.retryButtonText}>Retry</Text>
          </TouchableOpacity>
        </View>
      </SafeAreaView>
    );
  }

  if (!currentRecipe) {
    return (
      <SafeAreaView style={styles.container}>
        <View style={styles.errorContainer}>
          <Text style={styles.errorText}>Recipe not found</Text>
          <TouchableOpacity onPress={() => navigation.goBack()} style={styles.retryButton}>
            <Text style={styles.retryButtonText}>Go Back</Text>
          </TouchableOpacity>
        </View>
      </SafeAreaView>
    );
  }

  const adjustedServings = Math.round(currentRecipe.servings * servingMultiplier);

  return (
    <SafeAreaView style={styles.container}>
      {/* Header */}
      <View style={styles.header}>
        <TouchableOpacity onPress={() => navigation.goBack()}>
          <Text style={styles.backButton}>← Back</Text>
        </TouchableOpacity>
        <View style={styles.headerActions}>
          <TouchableOpacity onPress={handleShare} style={styles.actionButton}>
            <Text style={styles.actionButtonText}>Share</Text>
          </TouchableOpacity>
          <TouchableOpacity onPress={handleEdit} style={styles.actionButton}>
            <Text style={styles.actionButtonText}>Edit</Text>
          </TouchableOpacity>
          <TouchableOpacity onPress={handleDelete} style={styles.deleteButton}>
            <Text style={styles.deleteButtonText}>Delete</Text>
          </TouchableOpacity>
        </View>
      </View>

      <ScrollView style={styles.content} showsVerticalScrollIndicator={false}>
        {/* Hero Image */}
        <View style={styles.imageContainer}>
          <OptimizedRecipeImage
            imageUrl={currentRecipe.imageUrl}
            title={currentRecipe.title}
            width={screenWidth}
            height={250}
            priority="high"
            resizeMode="cover"
          />
          <View style={styles.imageOverlay}>
            <View
              style={[
                styles.complexityBadge,
                { backgroundColor: getComplexityColor(currentRecipe.complexity) }
              ]}
            >
              <Text style={styles.complexityText}>
                {currentRecipe.complexity.toUpperCase()}
              </Text>
            </View>
          </View>
        </View>

        {/* Recipe Info */}
        <View style={styles.infoSection}>
          <Text style={styles.title}>{currentRecipe.title}</Text>
          
          {currentRecipe.description && (
            <Text style={styles.description}>{currentRecipe.description}</Text>
          )}

          {/* Meal Types */}
          {currentRecipe.mealType.length > 0 && (
            <Text style={styles.mealTypes}>
              {formatMealTypes(currentRecipe.mealType)}
            </Text>
          )}

          {/* Rating */}
          {renderRating(currentRecipe.averageRating, currentRecipe.totalRatings)}

          {/* Recipe Stats */}
          <View style={styles.statsContainer}>
            <View style={styles.statItem}>
              <Text style={styles.statValue}>{formatTime(currentRecipe.prepTime)}</Text>
              <Text style={styles.statLabel}>Prep</Text>
            </View>
            <View style={styles.statItem}>
              <Text style={styles.statValue}>{formatTime(currentRecipe.cookTime)}</Text>
              <Text style={styles.statLabel}>Cook</Text>
            </View>
            <View style={styles.statItem}>
              <Text style={[styles.statValue, styles.totalTime]}>
                {formatTime(currentRecipe.totalTime)}
              </Text>
              <Text style={styles.statLabel}>Total</Text>
            </View>
          </View>

          {/* Dietary Labels */}
          {currentRecipe.dietaryLabels.length > 0 && (
            <View style={styles.labelsContainer}>
              {currentRecipe.dietaryLabels.map((label, index) => (
                <View key={index} style={styles.dietaryLabel}>
                  <Text style={styles.dietaryLabelText}>{label}</Text>
                </View>
              ))}
            </View>
          )}

          {/* Cuisine Type */}
          {currentRecipe.cuisineType && (
            <Text style={styles.cuisineType}>{currentRecipe.cuisineType} Cuisine</Text>
          )}
        </View>

        {/* Servings Adjuster */}
        <View style={styles.servingsSection}>
          <Text style={styles.sectionTitle}>Servings</Text>
          <View style={styles.servingsAdjuster}>
            <TouchableOpacity
              onPress={() => adjustServings(0.5)}
              style={[styles.servingButton, servingMultiplier === 0.5 && styles.servingButtonActive]}
            >
              <Text style={[styles.servingButtonText, servingMultiplier === 0.5 && styles.servingButtonTextActive]}>
                {Math.round(currentRecipe.servings * 0.5)}
              </Text>
            </TouchableOpacity>
            <TouchableOpacity
              onPress={() => adjustServings(1)}
              style={[styles.servingButton, servingMultiplier === 1 && styles.servingButtonActive]}
            >
              <Text style={[styles.servingButtonText, servingMultiplier === 1 && styles.servingButtonTextActive]}>
                {currentRecipe.servings}
              </Text>
            </TouchableOpacity>
            <TouchableOpacity
              onPress={() => adjustServings(2)}
              style={[styles.servingButton, servingMultiplier === 2 && styles.servingButtonActive]}
            >
              <Text style={[styles.servingButtonText, servingMultiplier === 2 && styles.servingButtonTextActive]}>
                {currentRecipe.servings * 2}
              </Text>
            </TouchableOpacity>
          </View>
          <Text style={styles.servingsNote}>Ingredients automatically adjusted</Text>
        </View>

        {/* Ingredients */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>
            Ingredients ({adjustedServings} serving{adjustedServings !== 1 ? 's' : ''})
          </Text>
          <View style={styles.ingredientsList}>
            {JSON.parse(currentRecipe.ingredients as any).map(renderIngredient)}
          </View>
        </View>

        {/* Instructions */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>Instructions</Text>
          <View style={styles.instructionsList}>
            {JSON.parse(currentRecipe.instructions as any).map(renderInstruction)}
          </View>
        </View>
      </ScrollView>
    </SafeAreaView>
  );
};

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#fff',
  },
  header: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    paddingHorizontal: 16,
    paddingVertical: 12,
    borderBottomWidth: 1,
    borderBottomColor: '#e0e0e0',
  },
  backButton: {
    fontSize: 16,
    color: '#007AFF',
  },
  headerActions: {
    flexDirection: 'row',
    gap: 8,
  },
  actionButton: {
    paddingHorizontal: 12,
    paddingVertical: 6,
  },
  actionButtonText: {
    color: '#007AFF',
    fontSize: 16,
  },
  deleteButton: {
    paddingHorizontal: 12,
    paddingVertical: 6,
  },
  deleteButtonText: {
    color: '#FF3B30',
    fontSize: 16,
  },
  content: {
    flex: 1,
  },
  imageContainer: {
    position: 'relative',
    height: 250,
  },
  imageOverlay: {
    position: 'absolute',
    top: 16,
    right: 16,
  },
  complexityBadge: {
    paddingHorizontal: 12,
    paddingVertical: 6,
    borderRadius: 16,
  },
  complexityText: {
    color: '#fff',
    fontSize: 12,
    fontWeight: '700',
  },
  infoSection: {
    paddingHorizontal: 16,
    paddingVertical: 20,
  },
  title: {
    fontSize: 28,
    fontWeight: '700',
    color: '#333',
    marginBottom: 8,
    lineHeight: 34,
  },
  description: {
    fontSize: 16,
    color: '#666',
    marginBottom: 16,
    lineHeight: 22,
  },
  mealTypes: {
    fontSize: 14,
    color: '#007AFF',
    fontWeight: '600',
    marginBottom: 12,
  },
  ratingContainer: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 16,
  },
  stars: {
    flexDirection: 'row',
    marginRight: 8,
  },
  star: {
    fontSize: 18,
  },
  ratingText: {
    fontSize: 14,
    color: '#666',
  },
  statsContainer: {
    flexDirection: 'row',
    justifyContent: 'space-around',
    paddingVertical: 16,
    backgroundColor: '#f8f8f8',
    borderRadius: 12,
    marginBottom: 16,
  },
  statItem: {
    alignItems: 'center',
  },
  statValue: {
    fontSize: 18,
    fontWeight: '700',
    color: '#333',
  },
  totalTime: {
    color: '#007AFF',
  },
  statLabel: {
    fontSize: 14,
    color: '#666',
    marginTop: 4,
  },
  labelsContainer: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: 8,
    marginBottom: 12,
  },
  dietaryLabel: {
    backgroundColor: '#e8f5e8',
    paddingHorizontal: 12,
    paddingVertical: 6,
    borderRadius: 16,
  },
  dietaryLabelText: {
    fontSize: 12,
    color: '#4CAF50',
    fontWeight: '600',
  },
  cuisineType: {
    fontSize: 14,
    color: '#999',
    fontStyle: 'italic',
  },
  servingsSection: {
    paddingHorizontal: 16,
    paddingVertical: 20,
    backgroundColor: '#f8f8f8',
  },
  servingsAdjuster: {
    flexDirection: 'row',
    justifyContent: 'center',
    gap: 12,
    marginBottom: 8,
  },
  servingButton: {
    paddingHorizontal: 20,
    paddingVertical: 10,
    borderRadius: 20,
    backgroundColor: '#fff',
    borderWidth: 2,
    borderColor: '#e0e0e0',
  },
  servingButtonActive: {
    backgroundColor: '#007AFF',
    borderColor: '#007AFF',
  },
  servingButtonText: {
    fontSize: 16,
    fontWeight: '600',
    color: '#333',
  },
  servingButtonTextActive: {
    color: '#fff',
  },
  servingsNote: {
    fontSize: 12,
    color: '#666',
    textAlign: 'center',
  },
  section: {
    paddingHorizontal: 16,
    paddingVertical: 20,
  },
  sectionTitle: {
    fontSize: 20,
    fontWeight: '700',
    color: '#333',
    marginBottom: 16,
  },
  ingredientsList: {
    gap: 12,
  },
  ingredientItem: {
    flexDirection: 'row',
    alignItems: 'center',
    paddingVertical: 8,
  },
  ingredientAmount: {
    width: 80,
    marginRight: 16,
  },
  ingredientAmountText: {
    fontSize: 14,
    fontWeight: '600',
    color: '#007AFF',
    textAlign: 'right',
  },
  ingredientName: {
    flex: 1,
    fontSize: 16,
    color: '#333',
    lineHeight: 20,
  },
  instructionsList: {
    gap: 16,
  },
  instructionItem: {
    flexDirection: 'row',
    alignItems: 'flex-start',
  },
  stepNumber: {
    width: 32,
    height: 32,
    borderRadius: 16,
    backgroundColor: '#007AFF',
    justifyContent: 'center',
    alignItems: 'center',
    marginRight: 16,
    marginTop: 2,
  },
  stepNumberText: {
    color: '#fff',
    fontSize: 14,
    fontWeight: '700',
  },
  instructionContent: {
    flex: 1,
  },
  instructionText: {
    fontSize: 16,
    color: '#333',
    lineHeight: 22,
  },
  instructionTime: {
    fontSize: 12,
    color: '#666',
    marginTop: 4,
    fontStyle: 'italic',
  },
  loadingContainer: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
  },
  loadingText: {
    fontSize: 16,
    color: '#666',
  },
  errorContainer: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    paddingHorizontal: 32,
  },
  errorText: {
    fontSize: 16,
    color: '#FF3B30',
    textAlign: 'center',
    marginBottom: 16,
  },
  retryButton: {
    paddingHorizontal: 20,
    paddingVertical: 10,
    backgroundColor: '#007AFF',
    borderRadius: 8,
  },
  retryButtonText: {
    color: '#fff',
    fontSize: 16,
    fontWeight: '600',
  },
});
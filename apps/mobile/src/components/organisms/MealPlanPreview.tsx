import React from 'react';
import {
  View,
  Text,
  StyleSheet,
  ScrollView,
  TouchableOpacity,
  Image,
} from 'react-native';
import type { MealPlanGenerationResponse, MealSlotWithRecipe, DayOfWeek, MealType } from '@imkitchen/shared-types';

interface MealPlanPreviewProps {
  generationResponse: MealPlanGenerationResponse;
  onAccept: () => void;
  onRegenerate: () => void;
  onClose: () => void;
  loading?: boolean;
}

const MealPlanPreview: React.FC<MealPlanPreviewProps> = ({
  generationResponse,
  onAccept,
  onRegenerate,
  onClose,
  loading = false,
}) => {
  const { mealPlan, generationTimeMs, varietyScore, recipesUsed, rotationCycle, warnings } = generationResponse;

  const days: Array<{ key: DayOfWeek; label: string }> = [
    { key: 'monday', label: 'Mon' },
    { key: 'tuesday', label: 'Tue' },
    { key: 'wednesday', label: 'Wed' },
    { key: 'thursday', label: 'Thu' },
    { key: 'friday', label: 'Fri' },
    { key: 'saturday', label: 'Sat' },
    { key: 'sunday', label: 'Sun' },
  ];

  const mealTypes: Array<{ key: MealType; label: string; emoji: string }> = [
    { key: 'breakfast', label: 'Breakfast', emoji: '🌅' },
    { key: 'lunch', label: 'Lunch', emoji: '☀️' },
    { key: 'dinner', label: 'Dinner', emoji: '🌙' },
  ];

  const getVarietyScoreColor = (score: number): string => {
    if (score >= 0.8) return '#4CAF50'; // Excellent - Green
    if (score >= 0.6) return '#FF9800'; // Good - Orange  
    return '#F44336'; // Needs improvement - Red
  };

  const getVarietyScoreLabel = (score: number): string => {
    if (score >= 0.8) return 'Excellent Variety';
    if (score >= 0.6) return 'Good Variety';
    return 'Limited Variety';
  };

  const getComplexityColor = (complexity?: string): string => {
    switch (complexity) {
      case 'simple': return '#4CAF50';
      case 'moderate': return '#FF9800';
      case 'complex': return '#F44336';
      default: return '#999999';
    }
  };

  const renderMealCard = (meal: MealSlotWithRecipe, dayKey: DayOfWeek, mealTypeKey: MealType) => {
    const recipe = meal.recipe;
    if (!recipe) {
      return (
        <View key={`${dayKey}-${mealTypeKey}`} style={styles.emptyMealCard}>
          <Text style={styles.emptyMealText}>No recipe</Text>
        </View>
      );
    }

    return (
      <View key={`${dayKey}-${mealTypeKey}`} style={styles.mealCard}>
        {recipe.imageUrl && (
          <Image
            source={{ uri: recipe.imageUrl }}
            style={styles.recipeImage}
            resizeMode="cover"
          />
        )}
        <View style={styles.mealInfo}>
          <Text style={styles.recipeTitle} numberOfLines={2}>
            {recipe.title}
          </Text>
          <View style={styles.recipeMetadata}>
            <View style={styles.metadataItem}>
              <Text style={styles.metadataLabel}>⏱️</Text>
              <Text style={styles.metadataValue}>
                {recipe.prepTime + recipe.cookTime}m
              </Text>
            </View>
            <View style={styles.metadataItem}>
              <View style={[
                styles.complexityDot,
                { backgroundColor: getComplexityColor(recipe.complexity) }
              ]} />
              <Text style={styles.metadataValue}>
                {recipe.complexity}
              </Text>
            </View>
            {recipe.cuisineType && (
              <View style={styles.metadataItem}>
                <Text style={styles.metadataValue}>
                  {recipe.cuisineType}
                </Text>
              </View>
            )}
          </View>
        </View>
      </View>
    );
  };

  return (
    <View style={styles.container}>
      {/* Header */}
      <View style={styles.header}>
        <TouchableOpacity onPress={onClose} style={styles.closeButton}>
          <Text style={styles.closeButtonText}>✕</Text>
        </TouchableOpacity>
        <Text style={styles.headerTitle}>Meal Plan Preview</Text>
        <View style={styles.headerSpacer} />
      </View>

      {/* Generation Statistics */}
      <View style={styles.statsContainer}>
        <View style={styles.statItem}>
          <Text style={styles.statValue}>{generationTimeMs}ms</Text>
          <Text style={styles.statLabel}>Generation Time</Text>
        </View>
        <View style={styles.statItem}>
          <Text style={[styles.statValue, { color: getVarietyScoreColor(varietyScore) }]}>
            {Math.round(varietyScore * 100)}%
          </Text>
          <Text style={styles.statLabel}>{getVarietyScoreLabel(varietyScore)}</Text>
        </View>
        <View style={styles.statItem}>
          <Text style={styles.statValue}>{recipesUsed}</Text>
          <Text style={styles.statLabel}>Recipes Used</Text>
        </View>
        <View style={styles.statItem}>
          <Text style={styles.statValue}>#{rotationCycle}</Text>
          <Text style={styles.statLabel}>Rotation Cycle</Text>
        </View>
      </View>

      {/* Warnings */}
      {warnings && warnings.length > 0 && (
        <View style={styles.warningsContainer}>
          <Text style={styles.warningsTitle}>⚠️ Considerations:</Text>
          {warnings.map((warning, index) => (
            <Text key={index} style={styles.warningText}>
              • {warning}
            </Text>
          ))}
        </View>
      )}

      {/* Meal Plan Grid */}
      <ScrollView style={styles.gridContainer} showsVerticalScrollIndicator={false}>
        <View style={styles.gridHeader}>
          <View style={styles.dayHeaderSpacer} />
          {mealTypes.map(({ key, emoji }) => (
            <View key={key} style={styles.mealTypeHeader}>
              <Text style={styles.mealTypeEmoji}>{emoji}</Text>
            </View>
          ))}
        </View>

        {days.map(({ key: dayKey, label: dayLabel }) => (
          <View key={dayKey} style={styles.dayRow}>
            <View style={styles.dayHeader}>
              <Text style={styles.dayLabel}>{dayLabel}</Text>
            </View>
            {mealTypes.map(({ key: mealTypeKey }) => {
              const dayMeals = mealPlan.populatedMeals[dayKey];
              const meal = dayMeals.find(m => m.mealType === mealTypeKey);
              
              return (
                <View key={mealTypeKey} style={styles.mealCell}>
                  {meal && renderMealCard(meal, dayKey, mealTypeKey)}
                </View>
              );
            })}
          </View>
        ))}
      </ScrollView>

      {/* Action Buttons */}
      <View style={styles.buttonContainer}>
        <TouchableOpacity
          style={[styles.button, styles.regenerateButton]}
          onPress={onRegenerate}
          disabled={loading}
        >
          <Text style={[styles.buttonText, styles.regenerateButtonText]}>
            🔄 Regenerate
          </Text>
        </TouchableOpacity>
        <TouchableOpacity
          style={[styles.button, styles.acceptButton]}
          onPress={onAccept}
          disabled={loading}
        >
          <Text style={[styles.buttonText, styles.acceptButtonText]}>
            ✓ Accept Plan
          </Text>
        </TouchableOpacity>
      </View>
    </View>
  );
};

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#FFFFFF',
  },
  header: {
    flexDirection: 'row',
    alignItems: 'center',
    paddingHorizontal: 16,
    paddingVertical: 12,
    borderBottomWidth: 1,
    borderBottomColor: '#E0E0E0',
    backgroundColor: '#FAFAFA',
  },
  closeButton: {
    width: 32,
    height: 32,
    borderRadius: 16,
    backgroundColor: '#F0F0F0',
    justifyContent: 'center',
    alignItems: 'center',
  },
  closeButtonText: {
    fontSize: 16,
    fontWeight: '600',
    color: '#666666',
  },
  headerTitle: {
    flex: 1,
    fontSize: 18,
    fontWeight: '700',
    textAlign: 'center',
    color: '#333333',
  },
  headerSpacer: {
    width: 32,
  },
  statsContainer: {
    flexDirection: 'row',
    paddingHorizontal: 16,
    paddingVertical: 12,
    backgroundColor: '#F8F9FA',
    borderBottomWidth: 1,
    borderBottomColor: '#E0E0E0',
  },
  statItem: {
    flex: 1,
    alignItems: 'center',
  },
  statValue: {
    fontSize: 18,
    fontWeight: '700',
    color: '#333333',
  },
  statLabel: {
    fontSize: 10,
    color: '#666666',
    textAlign: 'center',
    marginTop: 2,
  },
  warningsContainer: {
    backgroundColor: '#FFF3CD',
    paddingHorizontal: 16,
    paddingVertical: 12,
    borderBottomWidth: 1,
    borderBottomColor: '#E0E0E0',
  },
  warningsTitle: {
    fontSize: 14,
    fontWeight: '600',
    color: '#856404',
    marginBottom: 4,
  },
  warningText: {
    fontSize: 12,
    color: '#856404',
    marginBottom: 2,
  },
  gridContainer: {
    flex: 1,
  },
  gridHeader: {
    flexDirection: 'row',
    paddingHorizontal: 16,
    paddingVertical: 8,
    backgroundColor: '#F8F9FA',
    borderBottomWidth: 1,
    borderBottomColor: '#E0E0E0',
  },
  dayHeaderSpacer: {
    width: 60,
  },
  mealTypeHeader: {
    flex: 1,
    alignItems: 'center',
    marginHorizontal: 2,
  },
  mealTypeEmoji: {
    fontSize: 16,
  },
  dayRow: {
    flexDirection: 'row',
    paddingHorizontal: 16,
    paddingVertical: 8,
    borderBottomWidth: 1,
    borderBottomColor: '#F0F0F0',
  },
  dayHeader: {
    width: 60,
    justifyContent: 'center',
    alignItems: 'center',
  },
  dayLabel: {
    fontSize: 12,
    fontWeight: '600',
    color: '#666666',
  },
  mealCell: {
    flex: 1,
    marginHorizontal: 2,
    minHeight: 80,
  },
  mealCard: {
    backgroundColor: '#FFFFFF',
    borderRadius: 8,
    borderWidth: 1,
    borderColor: '#E0E0E0',
    overflow: 'hidden',
    flex: 1,
  },
  emptyMealCard: {
    backgroundColor: '#F8F9FA',
    borderRadius: 8,
    borderWidth: 1,
    borderColor: '#E0E0E0',
    borderStyle: 'dashed',
    justifyContent: 'center',
    alignItems: 'center',
    flex: 1,
    minHeight: 80,
  },
  emptyMealText: {
    fontSize: 10,
    color: '#999999',
  },
  recipeImage: {
    width: '100%',
    height: 40,
    backgroundColor: '#F0F0F0',
  },
  mealInfo: {
    padding: 6,
    flex: 1,
  },
  recipeTitle: {
    fontSize: 10,
    fontWeight: '600',
    color: '#333333',
    marginBottom: 4,
    lineHeight: 12,
  },
  recipeMetadata: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    alignItems: 'center',
  },
  metadataItem: {
    flexDirection: 'row',
    alignItems: 'center',
    marginRight: 6,
    marginBottom: 2,
  },
  metadataLabel: {
    fontSize: 8,
    marginRight: 2,
  },
  metadataValue: {
    fontSize: 8,
    color: '#666666',
  },
  complexityDot: {
    width: 6,
    height: 6,
    borderRadius: 3,
    marginRight: 2,
  },
  buttonContainer: {
    flexDirection: 'row',
    paddingHorizontal: 16,
    paddingVertical: 12,
    gap: 12,
    borderTopWidth: 1,
    borderTopColor: '#E0E0E0',
    backgroundColor: '#FFFFFF',
  },
  button: {
    flex: 1,
    paddingVertical: 12,
    borderRadius: 8,
    justifyContent: 'center',
    alignItems: 'center',
  },
  regenerateButton: {
    backgroundColor: '#F0F0F0',
    borderWidth: 1,
    borderColor: '#CCCCCC',
  },
  acceptButton: {
    backgroundColor: '#4CAF50',
  },
  buttonText: {
    fontSize: 16,
    fontWeight: '600',
  },
  regenerateButtonText: {
    color: '#666666',
  },
  acceptButtonText: {
    color: '#FFFFFF',
  },
});

export default MealPlanPreview;
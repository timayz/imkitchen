import React from 'react';
import {
  View,
  Text,
  TouchableOpacity,
  Image,
  StyleSheet,
} from 'react-native';
import type { Recipe } from '@imkitchen/shared-types';

export interface RecipeSimilarity {
  recipe: Recipe;
  compatibilityScore: number;
  reasons: string[];
  timeDifference: number; // minutes difference from original
  complexityMatch: boolean;
  cuisineMatch: boolean;
  nutritionSimilarity?: number; // 0-1 score
  ingredientOverlap?: number; // percentage of shared ingredients
  preparationSimilarity?: number; // similar cooking methods
  shoppingListImpact: {
    itemsAdded: number;
    itemsRemoved: number;
    estimatedCostChange: number;
  };
}

interface RecipeSimilarityCardProps {
  similarity: RecipeSimilarity;
  isSelected: boolean;
  onPress: () => void;
  showDetailedReasons?: boolean;
  compactMode?: boolean;
}

export const RecipeSimilarityCard: React.FC<RecipeSimilarityCardProps> = ({
  similarity,
  isSelected,
  onPress,
  showDetailedReasons = true,
  compactMode = false,
}) => {
  const { recipe, compatibilityScore, reasons, timeDifference, shoppingListImpact } = similarity;

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

  const getCompatibilityLabel = (score: number) => {
    if (score >= 0.8) return 'Excellent Match';
    if (score >= 0.6) return 'Good Match';
    return 'Fair Match';
  };

  const getTimeDifferenceText = () => {
    if (timeDifference === 0) return 'Same cooking time';
    const sign = timeDifference > 0 ? '+' : '';
    return `${sign}${timeDifference}min`;
  };

  const getTimeDifferenceColor = () => {
    if (Math.abs(timeDifference) <= 5) return '#28A745'; // Green for similar times
    if (Math.abs(timeDifference) <= 15) return '#FFC107'; // Yellow for moderate difference
    return '#DC3545'; // Red for significant difference
  };

  const renderCompactCard = () => (
    <TouchableOpacity
      style={[styles.compactCard, isSelected && styles.selectedCard]}
      onPress={onPress}
      accessibilityRole="button"
      accessibilityLabel={`Replace with ${recipe.title}`}
      accessibilityHint={`${Math.round(compatibilityScore * 100)}% match, ${formatTime(recipe.totalTime || 0)} cooking time`}
    >
      <View style={styles.compactHeader}>
        <View style={styles.compactImageContainer}>
          {recipe.imageUrl ? (
            <Image
              source={{ uri: recipe.imageUrl }}
              style={styles.compactImage}
              resizeMode="cover"
            />
          ) : (
            <View style={styles.compactPlaceholder}>
              <Text style={styles.placeholderEmoji}>🍽️</Text>
            </View>
          )}
          <View style={[
            styles.compactScoreBadge,
            { backgroundColor: getCompatibilityColor(compatibilityScore) }
          ]}>
            <Text style={styles.compactScoreText}>
              {Math.round(compatibilityScore * 100)}%
            </Text>
          </View>
        </View>

        <View style={styles.compactContent}>
          <Text style={styles.compactTitle} numberOfLines={2}>
            {recipe.title}
          </Text>
          <View style={styles.compactMeta}>
            <Text style={styles.compactTime}>
              {formatTime(recipe.totalTime || 0)}
            </Text>
            <Text style={[styles.compactTimeDiff, { color: getTimeDifferenceColor() }]}>
              ({getTimeDifferenceText()})
            </Text>
          </View>
          <Text style={styles.compactReason} numberOfLines={1}>
            {reasons[0]}
          </Text>
        </View>
      </View>

      {isSelected && (
        <View style={styles.selectedIndicator}>
          <Text style={styles.selectedText}>✓ Selected</Text>
        </View>
      )}
    </TouchableOpacity>
  );

  const renderFullCard = () => (
    <TouchableOpacity
      style={[styles.card, isSelected && styles.selectedCard]}
      onPress={onPress}
      accessibilityRole="button"
      accessibilityLabel={`Replace with ${recipe.title}`}
      accessibilityHint={`${getCompatibilityLabel(compatibilityScore)}, ${formatTime(recipe.totalTime || 0)} cooking time`}
    >
      {/* Recipe Header */}
      <View style={styles.cardHeader}>
        <View style={styles.imageContainer}>
          {recipe.imageUrl ? (
            <Image
              source={{ uri: recipe.imageUrl }}
              style={styles.recipeImage}
              resizeMode="cover"
            />
          ) : (
            <View style={styles.imagePlaceholder}>
              <Text style={styles.placeholderEmoji}>🍽️</Text>
            </View>
          )}
          
          {/* Compatibility Score Badge */}
          <View style={[
            styles.scoreBadge,
            { backgroundColor: getCompatibilityColor(compatibilityScore) }
          ]}>
            <Text style={styles.scoreText}>
              {Math.round(compatibilityScore * 100)}%
            </Text>
            <Text style={styles.scoreLabel}>match</Text>
          </View>
        </View>

        <View style={styles.recipeInfo}>
          <Text style={styles.recipeTitle} numberOfLines={2}>
            {recipe.title}
          </Text>
          
          <View style={styles.recipeMeta}>
            <View style={styles.metaItem}>
              <Text style={styles.metaLabel}>Time:</Text>
              <Text style={styles.metaValue}>
                {formatTime(recipe.totalTime || 0)}
              </Text>
              <Text style={[styles.timeDifference, { color: getTimeDifferenceColor() }]}>
                ({getTimeDifferenceText()})
              </Text>
            </View>
            
            <View style={styles.metaItem}>
              <Text style={styles.metaLabel}>Complexity:</Text>
              <Text style={styles.metaValue}>
                {recipe.complexity}
              </Text>
              {similarity.complexityMatch && (
                <Text style={styles.matchIndicator}>✓</Text>
              )}
            </View>

            {recipe.cuisineType && (
              <View style={styles.metaItem}>
                <Text style={styles.metaLabel}>Cuisine:</Text>
                <Text style={styles.metaValue}>
                  {recipe.cuisineType}
                </Text>
                {similarity.cuisineMatch && (
                  <Text style={styles.matchIndicator}>✓</Text>
                )}
              </View>
            )}
          </View>
        </View>
      </View>

      {/* Similarity Reasons */}
      {showDetailedReasons && reasons.length > 0 && (
        <View style={styles.reasonsSection}>
          <Text style={styles.reasonsTitle}>Why this is a good match:</Text>
          <View style={styles.reasonsList}>
            {reasons.slice(0, 3).map((reason, index) => (
              <View key={index} style={styles.reasonItem}>
                <Text style={styles.reasonBullet}>•</Text>
                <Text style={styles.reasonText}>{reason}</Text>
              </View>
            ))}
          </View>
        </View>
      )}

      {/* Advanced Similarity Metrics */}
      {(similarity.ingredientOverlap || similarity.nutritionSimilarity || similarity.preparationSimilarity) && (
        <View style={styles.metricsSection}>
          <Text style={styles.metricsTitle}>Similarity breakdown:</Text>
          <View style={styles.metricsGrid}>
            {similarity.ingredientOverlap && (
              <View style={styles.metric}>
                <Text style={styles.metricLabel}>Ingredients</Text>
                <Text style={styles.metricValue}>
                  {Math.round(similarity.ingredientOverlap)}% shared
                </Text>
              </View>
            )}
            {similarity.nutritionSimilarity && (
              <View style={styles.metric}>
                <Text style={styles.metricLabel}>Nutrition</Text>
                <Text style={styles.metricValue}>
                  {Math.round(similarity.nutritionSimilarity * 100)}% similar
                </Text>
              </View>
            )}
            {similarity.preparationSimilarity && (
              <View style={styles.metric}>
                <Text style={styles.metricLabel}>Preparation</Text>
                <Text style={styles.metricValue}>
                  {Math.round(similarity.preparationSimilarity * 100)}% similar
                </Text>
              </View>
            )}
          </View>
        </View>
      )}

      {/* Shopping List Impact */}
      <View style={styles.impactSection}>
        <Text style={styles.impactTitle}>Shopping list changes:</Text>
        <View style={styles.impactGrid}>
          <View style={styles.impactItem}>
            <Text style={styles.impactValue}>+{shoppingListImpact.itemsAdded}</Text>
            <Text style={styles.impactLabel}>items added</Text>
          </View>
          <View style={styles.impactItem}>
            <Text style={styles.impactValue}>-{shoppingListImpact.itemsRemoved}</Text>
            <Text style={styles.impactLabel}>items removed</Text>
          </View>
          <View style={styles.impactItem}>
            <Text style={[
              styles.impactValue,
              shoppingListImpact.estimatedCostChange > 0 ? styles.costIncrease : styles.costDecrease
            ]}>
              {shoppingListImpact.estimatedCostChange > 0 ? '+' : ''}
              ${Math.abs(shoppingListImpact.estimatedCostChange).toFixed(2)}
            </Text>
            <Text style={styles.impactLabel}>cost change</Text>
          </View>
        </View>
      </View>

      {isSelected && (
        <View style={styles.selectedBanner}>
          <Text style={styles.selectedBannerText}>✓ Selected for replacement</Text>
        </View>
      )}
    </TouchableOpacity>
  );

  return compactMode ? renderCompactCard() : renderFullCard();
};

const styles = StyleSheet.create({
  // Full card styles
  card: {
    backgroundColor: '#FFF',
    borderRadius: 12,
    padding: 16,
    marginBottom: 16,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.1,
    shadowRadius: 4,
    elevation: 2,
    borderWidth: 1,
    borderColor: 'transparent',
  },
  selectedCard: {
    borderColor: '#007AFF',
    backgroundColor: '#F0F9FF',
    shadowOpacity: 0.15,
  },
  cardHeader: {
    flexDirection: 'row',
    marginBottom: 16,
  },
  imageContainer: {
    position: 'relative',
    marginRight: 12,
  },
  recipeImage: {
    width: 80,
    height: 80,
    borderRadius: 8,
  },
  imagePlaceholder: {
    width: 80,
    height: 80,
    backgroundColor: '#F8F9FA',
    borderRadius: 8,
    justifyContent: 'center',
    alignItems: 'center',
  },
  placeholderEmoji: {
    fontSize: 32,
  },
  scoreBadge: {
    position: 'absolute',
    top: -6,
    right: -6,
    paddingVertical: 4,
    paddingHorizontal: 6,
    borderRadius: 12,
    minWidth: 40,
    alignItems: 'center',
  },
  scoreText: {
    fontSize: 11,
    fontWeight: '700',
    color: '#FFF',
  },
  scoreLabel: {
    fontSize: 8,
    color: '#FFF',
    opacity: 0.9,
  },
  recipeInfo: {
    flex: 1,
  },
  recipeTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#343A40',
    lineHeight: 20,
    marginBottom: 8,
  },
  recipeMeta: {
    gap: 6,
  },
  metaItem: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 4,
  },
  metaLabel: {
    fontSize: 12,
    color: '#6C757D',
    fontWeight: '500',
  },
  metaValue: {
    fontSize: 12,
    color: '#495057',
    textTransform: 'capitalize',
  },
  timeDifference: {
    fontSize: 11,
    fontWeight: '500',
  },
  matchIndicator: {
    fontSize: 10,
    color: '#28A745',
  },
  reasonsSection: {
    marginBottom: 16,
  },
  reasonsTitle: {
    fontSize: 12,
    fontWeight: '600',
    color: '#495057',
    marginBottom: 8,
  },
  reasonsList: {
    gap: 4,
  },
  reasonItem: {
    flexDirection: 'row',
    alignItems: 'flex-start',
    gap: 6,
  },
  reasonBullet: {
    fontSize: 12,
    color: '#007AFF',
    marginTop: 1,
  },
  reasonText: {
    flex: 1,
    fontSize: 12,
    color: '#6C757D',
    lineHeight: 16,
  },
  metricsSection: {
    marginBottom: 16,
  },
  metricsTitle: {
    fontSize: 11,
    fontWeight: '600',
    color: '#495057',
    marginBottom: 8,
  },
  metricsGrid: {
    flexDirection: 'row',
    gap: 16,
  },
  metric: {
    flex: 1,
  },
  metricLabel: {
    fontSize: 10,
    color: '#6C757D',
    marginBottom: 2,
  },
  metricValue: {
    fontSize: 11,
    color: '#495057',
    fontWeight: '500',
  },
  impactSection: {
    backgroundColor: '#F8F9FA',
    padding: 12,
    borderRadius: 8,
    marginBottom: 8,
  },
  impactTitle: {
    fontSize: 11,
    fontWeight: '600',
    color: '#495057',
    marginBottom: 8,
  },
  impactGrid: {
    flexDirection: 'row',
    justifyContent: 'space-around',
  },
  impactItem: {
    alignItems: 'center',
  },
  impactValue: {
    fontSize: 14,
    fontWeight: '600',
    color: '#495057',
  },
  impactLabel: {
    fontSize: 9,
    color: '#6C757D',
    textAlign: 'center',
    marginTop: 2,
  },
  costIncrease: {
    color: '#DC3545',
  },
  costDecrease: {
    color: '#28A745',
  },
  selectedBanner: {
    backgroundColor: '#007AFF',
    margin: -16,
    marginTop: 12,
    padding: 8,
    borderBottomLeftRadius: 12,
    borderBottomRightRadius: 12,
    alignItems: 'center',
  },
  selectedBannerText: {
    color: '#FFF',
    fontSize: 12,
    fontWeight: '600',
  },

  // Compact card styles
  compactCard: {
    backgroundColor: '#FFF',
    borderRadius: 8,
    padding: 12,
    marginBottom: 8,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 1 },
    shadowOpacity: 0.05,
    shadowRadius: 2,
    elevation: 1,
    borderWidth: 1,
    borderColor: 'transparent',
  },
  compactHeader: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  compactImageContainer: {
    position: 'relative',
    marginRight: 12,
  },
  compactImage: {
    width: 50,
    height: 50,
    borderRadius: 6,
  },
  compactPlaceholder: {
    width: 50,
    height: 50,
    backgroundColor: '#F8F9FA',
    borderRadius: 6,
    justifyContent: 'center',
    alignItems: 'center',
  },
  compactScoreBadge: {
    position: 'absolute',
    top: -4,
    right: -4,
    paddingVertical: 2,
    paddingHorizontal: 4,
    borderRadius: 8,
  },
  compactScoreText: {
    fontSize: 9,
    fontWeight: '700',
    color: '#FFF',
  },
  compactContent: {
    flex: 1,
  },
  compactTitle: {
    fontSize: 14,
    fontWeight: '600',
    color: '#343A40',
    marginBottom: 4,
  },
  compactMeta: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 4,
    marginBottom: 4,
  },
  compactTime: {
    fontSize: 11,
    color: '#6C757D',
  },
  compactTimeDiff: {
    fontSize: 10,
    fontWeight: '500',
  },
  compactReason: {
    fontSize: 11,
    color: '#6C757D',
    fontStyle: 'italic',
  },
  selectedIndicator: {
    alignSelf: 'flex-end',
    marginTop: 8,
  },
  selectedText: {
    fontSize: 11,
    color: '#007AFF',
    fontWeight: '600',
  },
});
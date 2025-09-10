import React, { useState } from 'react';
import {
  View,
  Text,
  Image,
  TouchableOpacity,
  StyleSheet,
  ViewStyle,
} from 'react-native';
import type { Recipe } from '@imkitchen/shared-types';
import { RatingStars } from '../atoms/RatingStars';
import { ReviewModal } from './ReviewModal';
import { useCommunityStore } from '../../store/community_store';

interface RecipeCardProps {
  recipe: Recipe;
  onPress: () => void;
  style?: ViewStyle;
  showRatingInteraction?: boolean; // Enable rating interactions for community recipes
}

export const RecipeCard: React.FC<RecipeCardProps> = ({
  recipe,
  onPress,
  style,
  showRatingInteraction = false,
}) => {
  const [showReviewModal, setShowReviewModal] = useState(false);
  const { submitRating, updateRating, userRatings, loading } = useCommunityStore();
  
  const userRating = userRatings[recipe.id];
  const formatTime = (minutes: number): string => {
    if (minutes < 60) {
      return `${minutes}m`;
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

  const handleRatingSubmit = async (ratingSubmission: any) => {
    try {
      if (userRating) {
        await updateRating(recipe.id, ratingSubmission);
      } else {
        await submitRating(recipe.id, ratingSubmission);
      }
    } catch (error) {
      // Error handling is done in the store
      throw error;
    }
  };

  const handleRatingPress = () => {
    if (showRatingInteraction) {
      setShowReviewModal(true);
    }
  };

  const renderRating = (rating: number, totalRatings: number) => {
    if (totalRatings === 0 && !showRatingInteraction) return null;

    return (
      <View style={styles.ratingContainer}>
        <TouchableOpacity
          onPress={handleRatingPress}
          disabled={!showRatingInteraction}
          activeOpacity={showRatingInteraction ? 0.7 : 1}
        >
          <RatingStars
            rating={rating}
            size="small"
            interactive={false} // We handle interaction at the container level
          />
        </TouchableOpacity>
        
        <View style={styles.ratingInfo}>
          {totalRatings > 0 ? (
            <Text style={styles.ratingText}>({totalRatings})</Text>
          ) : showRatingInteraction ? (
            <Text style={styles.ratingText}>Tap to rate</Text>
          ) : null}
          
          {showRatingInteraction && userRating && (
            <Text style={styles.userRatingIndicator}>★ Your rating</Text>
          )}
        </View>
      </View>
    );
  };

  return (
    <TouchableOpacity
      style={[styles.card, style]}
      onPress={onPress}
      activeOpacity={0.7}
    >
      {/* Recipe Image */}
      <View style={styles.imageContainer}>
        {recipe.imageUrl ? (
          <Image
            source={{ uri: recipe.imageUrl }}
            style={styles.image}
            resizeMode="cover"
          />
        ) : (
          <View style={styles.placeholderImage}>
            <Text style={styles.placeholderText}>🍽️</Text>
          </View>
        )}
        
        {/* Complexity Badge */}
        <View style={styles.badgeContainer}>
          <View
            style={[
              styles.complexityBadge,
              { backgroundColor: getComplexityColor(recipe.complexity) }
            ]}
          >
            <Text style={styles.complexityText}>
              {recipe.complexity.toUpperCase()}
            </Text>
          </View>
        </View>
      </View>

      {/* Recipe Content */}
      <View style={styles.content}>
        {/* Title */}
        <Text style={styles.title} numberOfLines={2}>
          {recipe.title}
        </Text>

        {/* Description */}
        {recipe.description && (
          <Text style={styles.description} numberOfLines={2}>
            {recipe.description}
          </Text>
        )}

        {/* Meal Types */}
        {recipe.mealType.length > 0 && (
          <Text style={styles.mealTypes}>
            {formatMealTypes(recipe.mealType)}
          </Text>
        )}

        {/* Rating */}
        {renderRating(recipe.averageRating, recipe.totalRatings)}

        {/* Recipe Info Row */}
        <View style={styles.infoRow}>
          <View style={styles.timeInfo}>
            <Text style={styles.timeLabel}>Prep:</Text>
            <Text style={styles.timeValue}>{formatTime(recipe.prepTime)}</Text>
          </View>
          
          <View style={styles.timeInfo}>
            <Text style={styles.timeLabel}>Cook:</Text>
            <Text style={styles.timeValue}>{formatTime(recipe.cookTime)}</Text>
          </View>
          
          <View style={styles.timeInfo}>
            <Text style={styles.timeLabel}>Total:</Text>
            <Text style={[styles.timeValue, styles.totalTime]}>
              {formatTime(recipe.totalTime)}
            </Text>
          </View>
          
          <View style={styles.servingsInfo}>
            <Text style={styles.servingsValue}>{recipe.servings}</Text>
            <Text style={styles.servingsLabel}>servings</Text>
          </View>
        </View>

        {/* Dietary Labels */}
        {recipe.dietaryLabels.length > 0 && (
          <View style={styles.labelsContainer}>
            {recipe.dietaryLabels.slice(0, 3).map((label, index) => (
              <View key={index} style={styles.dietaryLabel}>
                <Text style={styles.dietaryLabelText}>{label}</Text>
              </View>
            ))}
            {recipe.dietaryLabels.length > 3 && (
              <Text style={styles.moreLabelText}>
                +{recipe.dietaryLabels.length - 3}
              </Text>
            )}
          </View>
        )}

        {/* Cuisine Type */}
        {recipe.cuisineType && (
          <Text style={styles.cuisineType}>{recipe.cuisineType} Cuisine</Text>
        )}
      </View>

      {/* Review Modal */}
      {showRatingInteraction && (
        <ReviewModal
          visible={showReviewModal}
          onClose={() => setShowReviewModal(false)}
          onSubmit={handleRatingSubmit}
          recipeTitle={recipe.title}
          existingRating={userRating ? {
            rating: userRating.rating,
            review: userRating.review,
            difficulty: userRating.difficulty,
            wouldCookAgain: userRating.wouldCookAgain,
          } : undefined}
          loading={loading.submit}
        />
      )}
    </TouchableOpacity>
  );
};

const styles = StyleSheet.create({
  card: {
    backgroundColor: '#fff',
    borderRadius: 12,
    shadowColor: '#000',
    shadowOffset: {
      width: 0,
      height: 2,
    },
    shadowOpacity: 0.1,
    shadowRadius: 4,
    elevation: 3,
    marginBottom: 16,
  },
  imageContainer: {
    position: 'relative',
    height: 180,
    borderTopLeftRadius: 12,
    borderTopRightRadius: 12,
    overflow: 'hidden',
  },
  image: {
    width: '100%',
    height: '100%',
  },
  placeholderImage: {
    width: '100%',
    height: '100%',
    backgroundColor: '#f5f5f5',
    justifyContent: 'center',
    alignItems: 'center',
  },
  placeholderText: {
    fontSize: 48,
  },
  badgeContainer: {
    position: 'absolute',
    top: 12,
    right: 12,
  },
  complexityBadge: {
    paddingHorizontal: 8,
    paddingVertical: 4,
    borderRadius: 12,
  },
  complexityText: {
    color: '#fff',
    fontSize: 10,
    fontWeight: '700',
  },
  content: {
    padding: 16,
  },
  title: {
    fontSize: 18,
    fontWeight: '700',
    color: '#333',
    marginBottom: 6,
    lineHeight: 22,
  },
  description: {
    fontSize: 14,
    color: '#666',
    marginBottom: 8,
    lineHeight: 18,
  },
  mealTypes: {
    fontSize: 12,
    color: '#007AFF',
    fontWeight: '600',
    marginBottom: 8,
  },
  ratingContainer: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 12,
  },
  ratingInfo: {
    flexDirection: 'column',
    marginLeft: 8,
  },
  ratingText: {
    fontSize: 12,
    color: '#666',
  },
  userRatingIndicator: {
    fontSize: 10,
    color: '#007AFF',
    fontWeight: '600',
    marginTop: 2,
  },
  infoRow: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 12,
  },
  timeInfo: {
    flexDirection: 'row',
    alignItems: 'center',
    marginRight: 16,
  },
  timeLabel: {
    fontSize: 12,
    color: '#666',
    marginRight: 4,
  },
  timeValue: {
    fontSize: 12,
    fontWeight: '600',
    color: '#333',
  },
  totalTime: {
    color: '#007AFF',
  },
  servingsInfo: {
    flexDirection: 'row',
    alignItems: 'center',
    marginLeft: 'auto',
  },
  servingsValue: {
    fontSize: 16,
    fontWeight: '700',
    color: '#333',
    marginRight: 4,
  },
  servingsLabel: {
    fontSize: 12,
    color: '#666',
  },
  labelsContainer: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 8,
    flexWrap: 'wrap',
  },
  dietaryLabel: {
    backgroundColor: '#e8f5e8',
    paddingHorizontal: 8,
    paddingVertical: 4,
    borderRadius: 12,
    marginRight: 6,
    marginBottom: 4,
  },
  dietaryLabelText: {
    fontSize: 10,
    color: '#4CAF50',
    fontWeight: '600',
  },
  moreLabelText: {
    fontSize: 10,
    color: '#666',
    fontStyle: 'italic',
  },
  cuisineType: {
    fontSize: 12,
    color: '#999',
    fontStyle: 'italic',
  },
});
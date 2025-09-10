import React from 'react';
import {
  View,
  Text,
  TouchableOpacity,
  StyleSheet,
  ViewStyle,
} from 'react-native';

interface RatingStarsProps {
  rating: number;
  maxRating?: number;
  size?: 'small' | 'medium' | 'large';
  interactive?: boolean;
  onRatingChange?: (rating: number) => void;
  style?: ViewStyle;
}

export const RatingStars: React.FC<RatingStarsProps> = ({
  rating,
  maxRating = 5,
  size = 'medium',
  interactive = false,
  onRatingChange,
  style,
}) => {
  const getSizeStyles = () => {
    switch (size) {
      case 'small':
        return {
          fontSize: 14,
          marginHorizontal: 1,
        };
      case 'large':
        return {
          fontSize: 24,
          marginHorizontal: 2,
        };
      default:
        return {
          fontSize: 18,
          marginHorizontal: 1.5,
        };
    }
  };

  const sizeStyles = getSizeStyles();

  const handleStarPress = (starRating: number) => {
    if (interactive && onRatingChange) {
      onRatingChange(starRating);
    }
  };

  const renderStar = (index: number) => {
    const starRating = index + 1;
    const isActive = starRating <= Math.round(rating);
    const isPartiallyActive = rating > index && rating < starRating;
    
    const StarComponent = interactive ? TouchableOpacity : View;
    
    return (
      <StarComponent
        key={index}
        onPress={interactive ? () => handleStarPress(starRating) : undefined}
        activeOpacity={interactive ? 0.7 : 1}
        accessibilityRole={interactive ? "button" : "text"}
        accessibilityLabel={
          interactive 
            ? `Rate ${starRating} star${starRating > 1 ? 's' : ''}`
            : `${starRating} star${starRating > 1 ? 's' : ''}`
        }
        accessibilityState={interactive ? { selected: isActive } : undefined}
      >
        <Text
          style={[
            styles.star,
            {
              fontSize: sizeStyles.fontSize,
              marginHorizontal: sizeStyles.marginHorizontal,
              color: isActive || isPartiallyActive ? '#FFD700' : '#DDD',
            },
            interactive && styles.interactiveStar,
          ]}
        >
          ★
        </Text>
      </StarComponent>
    );
  };

  return (
    <View 
      style={[styles.container, style]}
      accessible={!interactive}
      accessibilityLabel={!interactive ? `Rating: ${rating} out of ${maxRating} stars` : undefined}
      accessibilityRole={!interactive ? "text" : undefined}
    >
      {Array.from({ length: maxRating }, (_, index) => renderStar(index))}
    </View>
  );
};

const styles = StyleSheet.create({
  container: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  star: {
    fontWeight: '400',
  },
  interactiveStar: {
    shadowColor: '#000',
    shadowOffset: {
      width: 0,
      height: 1,
    },
    shadowOpacity: 0.1,
    shadowRadius: 1,
    elevation: 1,
  },
});
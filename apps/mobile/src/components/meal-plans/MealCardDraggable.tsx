import React, { useRef } from 'react';
import {
  View,
  Text,
  Image,
  StyleSheet,
  PanResponder,
  Animated,
  Dimensions,
  TouchableOpacity,
} from 'react-native';
import type {
  DayOfWeek,
  MealType,
  MealSlotWithRecipe,
} from '@imkitchen/shared-types';

const { width: screenWidth } = Dimensions.get('window');

interface MealCardDraggableProps {
  day: DayOfWeek;
  mealType: MealType;
  meal: MealSlotWithRecipe;
  mealTypeLabel: string;
  mealTypeIcon: string;
  isLocked?: boolean;
  isDragging?: boolean;
  onDragStart?: (day: DayOfWeek, mealType: MealType) => void;
  onDragEnd?: (day: DayOfWeek, mealType: MealType, targetDay?: DayOfWeek, targetMealType?: MealType) => void;
  onPress?: () => void;
  onLongPress?: () => void;
}

export const MealCardDraggable: React.FC<MealCardDraggableProps> = ({
  day,
  mealType,
  meal,
  mealTypeLabel,
  mealTypeIcon,
  isLocked = false,
  isDragging = false,
  onDragStart,
  onDragEnd,
  onPress,
  onLongPress,
}) => {
  const pan = useRef(new Animated.ValueXY()).current;
  const scale = useRef(new Animated.Value(1)).current;
  const opacity = useRef(new Animated.Value(1)).current;

  const panResponder = useRef(
    PanResponder.create({
      onMoveShouldSetPanResponder: (evt, gestureState) => {
        // Only allow drag if not locked and has sufficient movement
        return !isLocked && (Math.abs(gestureState.dx) > 5 || Math.abs(gestureState.dy) > 5);
      },
      onPanResponderGrant: (evt, gestureState) => {
        // Start drag
        onDragStart?.(day, mealType);
        
        // Animate to drag state
        Animated.parallel([
          Animated.spring(scale, {
            toValue: 1.1,
            useNativeDriver: false,
          }),
          Animated.spring(opacity, {
            toValue: 0.8,
            useNativeDriver: false,
          }),
        ]).start();
      },
      onPanResponderMove: Animated.event(
        [null, { dx: pan.x, dy: pan.y }],
        { useNativeDriver: false }
      ),
      onPanResponderRelease: (evt, gestureState) => {
        // Determine drop target based on final position
        const dropTarget = getDropTarget(gestureState.moveX, gestureState.moveY);
        
        // Animate back to original position
        Animated.parallel([
          Animated.spring(pan, {
            toValue: { x: 0, y: 0 },
            useNativeDriver: false,
          }),
          Animated.spring(scale, {
            toValue: 1,
            useNativeDriver: false,
          }),
          Animated.spring(opacity, {
            toValue: 1,
            useNativeDriver: false,
          }),
        ]).start();

        // Call drop handler
        onDragEnd?.(day, mealType, dropTarget?.day, dropTarget?.mealType);
      },
    })
  ).current;

  const getDropTarget = (x: number, y: number): { day: DayOfWeek; mealType: MealType } | null => {
    // Calculate which day and meal type based on screen position
    // This is a simplified calculation - in a real app you'd need more precise positioning
    const dayWidth = screenWidth / 7;
    const dayIndex = Math.floor(x / dayWidth);
    
    const days: DayOfWeek[] = ['monday', 'tuesday', 'wednesday', 'thursday', 'friday', 'saturday', 'sunday'];
    const mealTypes: MealType[] = ['breakfast', 'lunch', 'dinner'];
    
    // Simple meal type detection based on Y position
    const mealTypeIndex = Math.floor((y - 100) / 120); // Approximate meal slot height
    
    if (dayIndex >= 0 && dayIndex < 7 && mealTypeIndex >= 0 && mealTypeIndex < 3) {
      return {
        day: days[dayIndex],
        mealType: mealTypes[mealTypeIndex],
      };
    }
    
    return null;
  };

  const getComplexityColor = (complexity?: string): string => {
    switch (complexity) {
      case 'simple':
        return '#4CAF50';
      case 'moderate':
        return '#FF9800';
      case 'complex':
        return '#F44336';
      default:
        return '#e0e0e0';
    }
  };

  const formatTime = (minutes?: number): string => {
    if (!minutes) return '';
    if (minutes < 60) {
      return `${minutes}m`;
    }
    const hours = Math.floor(minutes / 60);
    const remainingMinutes = minutes % 60;
    return remainingMinutes > 0 ? `${hours}h ${remainingMinutes}m` : `${hours}h`;
  };

  if (!meal.recipe) {
    return null;
  }

  const { recipe } = meal;

  return (
    <Animated.View
      style={[
        styles.container,
        {
          transform: [
            { translateX: pan.x },
            { translateY: pan.y },
            { scale: scale },
          ],
          opacity: opacity,
        },
        isLocked && styles.lockedContainer,
        isDragging && styles.draggingContainer,
      ]}
      testID="meal-card-container"
      {...(isLocked ? {} : panResponder.panHandlers)}
    >
      <TouchableOpacity
        style={styles.content}
        testID="meal-card-touchable"
        onPress={onPress}
        onLongPress={onLongPress}
        activeOpacity={0.7}
        disabled={isDragging}
      >
        {/* Drag Handle */}
        {!isLocked && (
          <View style={styles.dragHandle} testID="drag-handle">
            <View style={styles.dragIndicator} />
            <View style={styles.dragIndicator} />
            <View style={styles.dragIndicator} />
          </View>
        )}

        {/* Lock Icon */}
        {isLocked && (
          <View style={styles.lockIcon}>
            <Text style={styles.lockText}>🔒</Text>
          </View>
        )}

        {/* Recipe Image */}
        <View style={styles.imageContainer}>
          {recipe.imageUrl ? (
            <Image
              source={{ uri: recipe.imageUrl }}
              style={styles.recipeImage}
              resizeMode="cover"
            />
          ) : (
            <View style={styles.placeholderImage}>
              <Text style={styles.placeholderEmoji}>🍽️</Text>
            </View>
          )}
          
          {/* Complexity Badge */}
          <View style={styles.badgeContainer}>
            <View
              style={[
                styles.complexityBadge,
                { backgroundColor: getComplexityColor(recipe.complexity) }
              ]}
            />
          </View>
        </View>

        {/* Recipe Info */}
        <View style={styles.recipeInfo}>
          <Text style={styles.mealTypeLabel}>
            {mealTypeIcon} {mealTypeLabel}
          </Text>
          
          <Text style={styles.recipeTitle} numberOfLines={2}>
            {recipe.title}
          </Text>
          
          <View style={styles.recipeDetails}>
            <Text style={styles.timeText}>
              {formatTime(recipe.totalTime)}
            </Text>
            {meal.servings > 0 && (
              <Text style={styles.servingsText}>
                {meal.servings} servings
              </Text>
            )}
          </View>

          {meal.isCompleted && (
            <View style={styles.completedBadge}>
              <Text style={styles.completedText}>✓ Done</Text>
            </View>
          )}

          {meal.notes && (
            <Text style={styles.notesText} numberOfLines={1}>
              📝 {meal.notes}
            </Text>
          )}
        </View>
      </TouchableOpacity>
    </Animated.View>
  );
};

const styles = StyleSheet.create({
  container: {
    marginVertical: 4,
    borderRadius: 12,
    backgroundColor: '#fff',
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.1,
    shadowRadius: 4,
    elevation: 3,
    overflow: 'hidden',
  },
  lockedContainer: {
    borderWidth: 2,
    borderColor: '#FFC107',
    backgroundColor: '#FFFBF0',
  },
  draggingContainer: {
    shadowOpacity: 0.3,
    shadowRadius: 8,
    elevation: 8,
    borderWidth: 2,
    borderColor: '#2196F3',
  },
  content: {
    flex: 1,
  },
  dragHandle: {
    position: 'absolute',
    top: 8,
    left: 8,
    zIndex: 1,
    flexDirection: 'row',
    alignItems: 'center',
  },
  dragIndicator: {
    width: 3,
    height: 12,
    backgroundColor: '#666',
    marginRight: 1,
    borderRadius: 1,
  },
  lockIcon: {
    position: 'absolute',
    top: 8,
    left: 8,
    zIndex: 1,
    backgroundColor: 'rgba(255, 193, 7, 0.9)',
    borderRadius: 12,
    width: 24,
    height: 24,
    justifyContent: 'center',
    alignItems: 'center',
  },
  lockText: {
    fontSize: 12,
  },
  imageContainer: {
    position: 'relative',
    height: 80,
    backgroundColor: '#f5f5f5',
  },
  recipeImage: {
    width: '100%',
    height: '100%',
  },
  placeholderImage: {
    width: '100%',
    height: '100%',
    justifyContent: 'center',
    alignItems: 'center',
    backgroundColor: '#f5f5f5',
  },
  placeholderEmoji: {
    fontSize: 32,
  },
  badgeContainer: {
    position: 'absolute',
    top: 8,
    right: 8,
  },
  complexityBadge: {
    width: 12,
    height: 12,
    borderRadius: 6,
    borderWidth: 2,
    borderColor: '#fff',
  },
  recipeInfo: {
    padding: 12,
    flex: 1,
  },
  mealTypeLabel: {
    fontSize: 10,
    color: '#666',
    marginBottom: 4,
    fontWeight: '500',
  },
  recipeTitle: {
    fontSize: 14,
    fontWeight: '600',
    color: '#333',
    marginBottom: 6,
    lineHeight: 16,
  },
  recipeDetails: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 6,
  },
  timeText: {
    fontSize: 11,
    color: '#666',
    marginRight: 8,
  },
  servingsText: {
    fontSize: 11,
    color: '#666',
  },
  completedBadge: {
    alignSelf: 'flex-start',
    backgroundColor: '#e8f5e8',
    paddingHorizontal: 8,
    paddingVertical: 3,
    borderRadius: 10,
    marginBottom: 4,
  },
  completedText: {
    fontSize: 10,
    color: '#4CAF50',
    fontWeight: '600',
  },
  notesText: {
    fontSize: 10,
    color: '#666',
    fontStyle: 'italic',
  },
});
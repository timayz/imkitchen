import React, { useRef, useState } from 'react';
import {
  View,
  Text,
  TouchableOpacity,
  Image,
  StyleSheet,
  Animated,
  PanResponder,
} from 'react-native';
import type {
  MealSlotProps,
  DayOfWeek,
  MealType,
  MealSlotWithRecipe,
} from '@imkitchen/shared-types';
import { MealCardDraggable } from '../meal-plans/MealCardDraggable';
import { MealCalendarDropZone } from '../meal-plans/MealCalendarDropZone';
import { MealLockToggle } from '../atoms/MealLockToggle';
import { useTheme } from '../../theme/ThemeProvider';
import type { ColorTokens } from '../../theme/tokens';
import {
  createScaleAnimation,
  createFadeAnimation,
  ANIMATION_DURATION,
  withPerformanceMonitoring
} from '../../theme/animations';

interface ExtendedMealSlotProps extends Omit<MealSlotProps, 'onPress' | 'onLongPress'> {
  mealTypeLabel: string;
  mealTypeIcon: string;
  onPress?: () => void;
  onLongPress?: () => void;
  onDragStart?: (day: DayOfWeek, mealType: MealType) => void;
  onDragEnd?: (day: DayOfWeek, mealType: MealType, targetDay?: DayOfWeek, targetMealType?: MealType) => void;
  onLockToggle?: (day: DayOfWeek, mealType: MealType, isLocked: boolean) => void;
  onSwapPress?: (day: DayOfWeek, mealType: MealType, meal: MealSlotWithRecipe) => void;
  isDragTarget?: boolean;
  isDragging?: boolean;
  showLockToggle?: boolean;
  showSwapButton?: boolean;
}

export const MealSlot: React.FC<ExtendedMealSlotProps> = ({
  day,
  mealType,
  meal,
  mealTypeLabel,
  mealTypeIcon,
  onPress,
  onLongPress,
  onDragStart,
  onDragEnd,
  onLockToggle,
  onSwapPress,
  isEditable = false,
  isEmpty = true,
  dragEnabled = false,
  dropEnabled = false,
  isDragTarget = false,
  isDragging = false,
  showLockToggle = false,
  showSwapButton = false,
}) => {
  const { colors } = useTheme();
  const [isPressed, setIsPressed] = useState(false);
  const [isHovered, setIsHovered] = useState(false);
  
  // Animation values
  const scaleAnim = useRef(new Animated.Value(1)).current;
  const fadeAnim = useRef(new Animated.Value(1)).current;
  const slideAnim = useRef(new Animated.Value(0)).current;
  
  const isLocked = meal?.isLocked ?? false;
  const canDrag = dragEnabled && !isEmpty && !isLocked && isEditable;
  const canShowLockToggle = showLockToggle && !isEmpty && isEditable;
  const canShowSwapButton = showSwapButton && !isEmpty && isEditable && meal;

  const handleLockToggle = () => {
    if (onLockToggle) {
      onLockToggle(day, mealType, !isLocked);
    }
  };

  const handleSwapPress = () => {
    if (onSwapPress && meal) {
      onSwapPress(day, mealType, meal);
    }
  };

  const getComplexityColor = (complexity?: string): string => {
    switch (complexity) {
      case 'simple':
        return colors.success;
      case 'moderate':
        return colors.warning;
      case 'complex':
        return colors.error;
      default:
        return colors.border;
    }
  };
  
  // Micro-interaction handlers
  const handlePressIn = () => {
    setIsPressed(true);
    const animation = withPerformanceMonitoring(
      createScaleAnimation(scaleAnim, 0.95, ANIMATION_DURATION.FAST)
    );
    animation.start();
  };
  
  const handlePressOut = () => {
    setIsPressed(false);
    const animation = withPerformanceMonitoring(
      createScaleAnimation(scaleAnim, 1, ANIMATION_DURATION.FAST)
    );
    animation.start();
  };
  
  const handleLongPressIn = () => {
    const pulseAnimation = withPerformanceMonitoring(
      Animated.sequence([
        createScaleAnimation(scaleAnim, 0.9, ANIMATION_DURATION.FAST),
        createScaleAnimation(scaleAnim, 1.05, ANIMATION_DURATION.FAST),
        createScaleAnimation(scaleAnim, 1, ANIMATION_DURATION.FAST),
      ])
    );
    pulseAnimation.start();
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

  const renderRecipeContent = () => {
    if (!meal?.recipe) return null;

    const { recipe } = meal;

    return (
      <>
        {/* Recipe Image or Placeholder */}
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
          <Text style={[styles.recipeTitle, { color: colors.text }]} numberOfLines={2}>
            {recipe.title}
          </Text>
          
          <View style={styles.recipeDetails}>
            <Text style={[styles.timeText, { color: colors.textSecondary }]}>
              {formatTime(recipe.totalTime)}
            </Text>
            {meal.servings > 0 && (
              <Text style={[styles.servingsText, { color: colors.textSecondary }]}>
                {meal.servings} servings
              </Text>
            )}
          </View>

          {meal.isCompleted && (
            <View style={[styles.completedBadge, { backgroundColor: colors.success + '20' }]}>
              <Text style={[styles.completedText, { color: colors.success }]}>✓ Done</Text>
            </View>
          )}

          {meal.notes && (
            <Text style={[styles.notesText, { color: colors.textSecondary }]} numberOfLines={1}>
              📝 {meal.notes}
            </Text>
          )}
        </View>
      </>
    );
  };

  const renderEmptyContent = () => {
    return (
      <View style={styles.emptyContent}>
        <Text style={styles.emptyIcon}>{mealTypeIcon}</Text>
        <Text style={[styles.emptyLabel, { color: colors.textTertiary }]}>{mealTypeLabel}</Text>
        {isEditable && (
          <Text style={[styles.emptyHint, { color: colors.textTertiary }]}>Tap to add</Text>
        )}
      </View>
    );
  };

  // If meal exists and can be dragged, use MealCardDraggable
  if (!isEmpty && meal && canDrag) {
    return (
      <View style={styles.draggableContainer}>
        <MealCardDraggable
          day={day}
          mealType={mealType}
          meal={meal}
          mealTypeLabel={mealTypeLabel}
          mealTypeIcon={mealTypeIcon}
          isLocked={isLocked}
          isDragging={isDragging}
          onDragStart={onDragStart}
          onDragEnd={onDragEnd}
          onPress={onPress}
          onLongPress={onLongPress}
        />
        {/* Lock Toggle */}
        {canShowLockToggle && (
          <View style={styles.lockToggleContainer}>
            <MealLockToggle
              isLocked={isLocked}
              onToggle={handleLockToggle}
              size="small"
            />
          </View>
        )}

        {/* Swap Button */}
        {canShowSwapButton && !isLocked && (
          <View style={styles.swapButtonContainer}>
            <TouchableOpacity
              style={[styles.swapButton, { backgroundColor: colors.info }]}
              onPress={handleSwapPress}
              accessibilityRole="button"
              accessibilityLabel="Quick swap recipe"
              accessibilityHint="Find similar recipes to replace this meal"
            >
              <Text style={[styles.swapButtonText, { color: colors.textInverse }]}>⇄</Text>
            </TouchableOpacity>
          </View>
        )}
      </View>
    );
  }

  // For empty slots or non-draggable meals, use drop zone or regular slot
  const content = isEmpty ? renderEmptyContent() : renderRecipeContent();

  return (
    <MealCalendarDropZone
      day={day}
      mealType={mealType}
      mealTypeLabel={mealTypeLabel}
      mealTypeIcon={mealTypeIcon}
      isActive={isDragTarget}
      isValidTarget={dropEnabled && !isLocked}
    >
      <Animated.View
        style={[
          { transform: [{ scale: scaleAnim }, { translateX: slideAnim }] },
          { opacity: fadeAnim },
        ]}
      >
        <TouchableOpacity
          style={[
            getContainerStyles(colors, isEmpty, meal?.isCompleted, isLocked, isDragTarget && dropEnabled),
          ]}
          onPress={onPress}
          onLongPress={onLongPress}
          onPressIn={handlePressIn}
          onPressOut={handlePressOut}
          onLongPress={() => {
            handleLongPressIn();
            onLongPress?.();
          }}
          activeOpacity={1}
          disabled={!isEditable && isEmpty}
          accessibilityRole="button"
          accessibilityLabel={isEmpty ? `Add ${mealTypeLabel.toLowerCase()}` : `${mealTypeLabel}: ${meal?.recipe?.title}`}
          accessibilityState={{ 
            disabled: !isEditable && isEmpty,
            selected: isPressed
          }}
        >
          {content}
          
          {/* Lock Toggle for non-draggable meals */}
          {canShowLockToggle && !canDrag && (
            <View style={styles.lockToggleOverlay}>
              <MealLockToggle
                isLocked={isLocked}
                onToggle={handleLockToggle}
                size="small"
              />
            </View>
          )}
        </TouchableOpacity>
      </Animated.View>
    </MealCalendarDropZone>
  );
};

// Themed style functions
const getContainerStyles = (colors: ColorTokens, isEmpty: boolean, isCompleted?: boolean, isLocked?: boolean, isDragTarget?: boolean) => ({
  marginVertical: 4,
  borderRadius: 8,
  overflow: 'hidden' as const,
  minHeight: 80,
  backgroundColor: isEmpty 
    ? colors.backgroundSecondary
    : colors.surface,
  borderWidth: isEmpty ? 1 : 0,
  borderColor: isEmpty ? colors.border : 'transparent',
  borderStyle: isEmpty ? 'dashed' as const : 'solid' as const,
  shadowColor: isEmpty ? 'transparent' : colors.text,
  shadowOffset: isEmpty ? { width: 0, height: 0 } : { width: 0, height: 1 },
  shadowOpacity: isEmpty ? 0 : 0.1,
  shadowRadius: isEmpty ? 0 : 2,
  elevation: isEmpty ? 0 : 2,
  opacity: isCompleted ? 0.7 : 1,
  borderLeftWidth: isCompleted ? 4 : isEmpty ? 1 : 0,
  borderLeftColor: isCompleted ? colors.success : isEmpty ? colors.border : 'transparent',
  ...(isDragTarget && {
    borderColor: colors.info,
    borderWidth: 2,
    backgroundColor: colors.info + '10',
  }),
  ...(isLocked && {
    borderWidth: 2,
    borderColor: colors.warning,
    backgroundColor: colors.warning + '10',
    opacity: 0.9,
  }),
});

const styles = StyleSheet.create({
  draggableContainer: {
    position: 'relative',
  },
  lockToggleContainer: {
    position: 'absolute',
    top: 8,
    right: 8,
    zIndex: 2,
  },
  lockToggleOverlay: {
    position: 'absolute',
    top: 8,
    right: 8,
    zIndex: 1,
  },
  swapButtonContainer: {
    position: 'absolute',
    bottom: 8,
    left: 8,
    zIndex: 2,
  },
  swapButton: {
    borderRadius: 14,
    width: 28,
    height: 28,
    justifyContent: 'center',
    alignItems: 'center',
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.2,
    shadowRadius: 4,
    elevation: 3,
  },
  swapButtonText: {
    fontSize: 14,
    fontWeight: '600',
  },
  imageContainer: {
    position: 'relative',
    height: 60,
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
    fontSize: 24,
  },
  badgeContainer: {
    position: 'absolute',
    top: 4,
    right: 4,
  },
  complexityBadge: {
    width: 8,
    height: 8,
    borderRadius: 4,
  },
  recipeInfo: {
    padding: 8,
    flex: 1,
  },
  recipeTitle: {
    fontSize: 12,
    fontWeight: '600',
    marginBottom: 4,
    lineHeight: 14,
  },
  recipeDetails: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 4,
  },
  timeText: {
    fontSize: 10,
    marginRight: 8,
  },
  servingsText: {
    fontSize: 10,
  },
  completedBadge: {
    alignSelf: 'flex-start',
    paddingHorizontal: 6,
    paddingVertical: 2,
    borderRadius: 8,
    marginBottom: 4,
  },
  completedText: {
    fontSize: 9,
    fontWeight: '600',
  },
  notesText: {
    fontSize: 9,
    fontStyle: 'italic',
  },
  emptyContent: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    padding: 16,
  },
  emptyIcon: {
    fontSize: 20,
    marginBottom: 4,
  },
  emptyLabel: {
    fontSize: 10,
    fontWeight: '600',
    marginBottom: 2,
  },
  emptyHint: {
    fontSize: 8,
    fontStyle: 'italic',
  },
});
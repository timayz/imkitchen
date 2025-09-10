import React, { useRef } from 'react';
import {
  View,
  Text,
  PanResponder,
  Animated,
  StyleSheet,
  Dimensions,
} from 'react-native';
import type {
  MealSlotProps,
  DayOfWeek,
  MealType,
  MealSlotWithRecipe,
  DragDropMealData,
} from '@imkitchen/shared-types';
import { MealSlot } from './MealSlot';
import { useDragDrop } from '../atoms/DragDropProvider';

const { width: screenWidth, height: screenHeight } = Dimensions.get('window');

interface DraggableMealSlotProps extends Omit<MealSlotProps, 'onPress' | 'onLongPress'> {
  mealTypeLabel: string;
  mealTypeIcon: string;
  onPress?: () => void;
  onLongPress?: () => void;
  onDrop?: (dragData: DragDropMealData, dropDay: DayOfWeek, dropMealType: MealType) => void;
}

export const DraggableMealSlot: React.FC<DraggableMealSlotProps> = ({
  day,
  mealType,
  meal,
  mealTypeLabel,
  mealTypeIcon,
  onPress,
  onLongPress,
  onDrop,
  isEditable = false,
  isEmpty = true,
  dragEnabled = false,
  dropEnabled = false,
}) => {
  const { startDrag, endDrag, setDropTarget, isDragging, isValidDropTarget, dragData } = useDragDrop();
  
  const pan = useRef(new Animated.ValueXY()).current;
  const scale = useRef(new Animated.Value(1)).current;
  const opacity = useRef(new Animated.Value(1)).current;
  
  const panResponder = useRef(
    PanResponder.create({
      onMoveShouldSetPanResponder: (_, gestureState) => {
        // Only enable drag if not empty, drag is enabled, and editable
        return dragEnabled && !isEmpty && isEditable && 
               (Math.abs(gestureState.dx) > 10 || Math.abs(gestureState.dy) > 10);
      },
      
      onPanResponderGrant: () => {
        if (!meal?.recipe) return;
        
        // Start drag operation
        const dragData: DragDropMealData = {
          recipeId: meal.recipe.id,
          recipe: meal.recipe,
          sourceMealPlanId: 'current', // This would come from props in real implementation
          sourceDay: day,
          sourceMealType: mealType,
        };
        
        startDrag(dragData);
        
        // Animate drag start
        Animated.parallel([
          Animated.spring(scale, {
            toValue: 1.1,
            useNativeDriver: true,
          }),
          Animated.spring(opacity, {
            toValue: 0.8,
            useNativeDriver: true,
          }),
        ]).start();
      },
      
      onPanResponderMove: (_, gestureState) => {
        if (!isDragging) return;
        
        // Update pan position
        pan.setValue({ x: gestureState.dx, y: gestureState.dy });
        
        // Calculate drop target based on current position
        const dropTarget = calculateDropTarget(gestureState.moveX, gestureState.moveY);
        if (dropTarget && isValidDropTarget(dropTarget)) {
          setDropTarget(dropTarget);
        } else {
          setDropTarget(null);
        }
      },
      
      onPanResponderRelease: (_, gestureState) => {
        if (!isDragging || !dragData) return;
        
        // Calculate final drop target
        const dropTarget = calculateDropTarget(gestureState.moveX, gestureState.moveY);
        
        if (dropTarget && isValidDropTarget(dropTarget) && onDrop) {
          // Perform drop operation
          onDrop(dragData, dropTarget.day, dropTarget.mealType);
        }
        
        // Reset animations
        Animated.parallel([
          Animated.spring(pan, {
            toValue: { x: 0, y: 0 },
            useNativeDriver: true,
          }),
          Animated.spring(scale, {
            toValue: 1,
            useNativeDriver: true,
          }),
          Animated.spring(opacity, {
            toValue: 1,
            useNativeDriver: true,
          }),
        ]).start(() => {
          endDrag();
        });
      },
    })
  ).current;

  const calculateDropTarget = (x: number, y: number) => {
    // This is a simplified calculation
    // In a real implementation, you would need to map screen coordinates
    // to meal slot positions based on the actual layout
    
    // For now, return null as this would require more complex layout calculations
    return null;
  };

  const handlePress = () => {
    if (!isDragging) {
      onPress?.();
    }
  };

  const handleLongPress = () => {
    if (!isDragging) {
      onLongPress?.();
    }
  };

  const isDropTarget = dropEnabled && isDragging && dragData && 
    dragData.sourceDay !== day && dragData.sourceMealType !== mealType;

  return (
    <Animated.View
      {...(dragEnabled && !isEmpty ? panResponder.panHandlers : {})}
      style={[
        styles.container,
        {
          transform: [
            { translateX: pan.x },
            { translateY: pan.y },
            { scale },
          ],
          opacity,
        },
        isDropTarget && styles.dropTargetContainer,
      ]}
    >
      <MealSlot
        day={day}
        mealType={mealType}
        meal={meal}
        mealTypeLabel={mealTypeLabel}
        mealTypeIcon={mealTypeIcon}
        onPress={handlePress}
        onLongPress={handleLongPress}
        isEditable={isEditable}
        isEmpty={isEmpty}
        dragEnabled={dragEnabled}
        dropEnabled={dropEnabled}
      />
      
      {dragEnabled && !isEmpty && (
        <View style={styles.dragIndicator}>
          <Text style={styles.dragIndicatorText}>⋮⋮</Text>
        </View>
      )}
      
      {isDropTarget && (
        <View style={styles.dropOverlay}>
          <Text style={styles.dropOverlayText}>Drop here</Text>
        </View>
      )}
    </Animated.View>
  );
};

const styles = StyleSheet.create({
  container: {
    position: 'relative',
  },
  dropTargetContainer: {
    borderColor: '#2196f3',
    borderWidth: 2,
    borderStyle: 'dashed',
    borderRadius: 8,
    backgroundColor: 'rgba(33, 150, 243, 0.1)',
  },
  dragIndicator: {
    position: 'absolute',
    top: 4,
    left: 4,
    backgroundColor: 'rgba(0, 0, 0, 0.3)',
    paddingHorizontal: 4,
    paddingVertical: 2,
    borderRadius: 4,
  },
  dragIndicatorText: {
    fontSize: 8,
    color: '#fff',
    fontWeight: 'bold',
  },
  dropOverlay: {
    position: 'absolute',
    top: 0,
    left: 0,
    right: 0,
    bottom: 0,
    backgroundColor: 'rgba(33, 150, 243, 0.2)',
    borderRadius: 8,
    justifyContent: 'center',
    alignItems: 'center',
    borderWidth: 2,
    borderColor: '#2196f3',
    borderStyle: 'dashed',
  },
  dropOverlayText: {
    fontSize: 12,
    color: '#2196f3',
    fontWeight: 'bold',
  },
});
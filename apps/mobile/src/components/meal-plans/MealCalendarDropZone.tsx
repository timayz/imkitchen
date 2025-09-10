import React from 'react';
import {
  View,
  Text,
  StyleSheet,
  Animated,
} from 'react-native';
import type {
  DayOfWeek,
  MealType,
} from '@imkitchen/shared-types';

interface MealCalendarDropZoneProps {
  day: DayOfWeek;
  mealType: MealType;
  mealTypeLabel: string;
  mealTypeIcon: string;
  isActive?: boolean;
  isValidTarget?: boolean;
  onDrop?: (day: DayOfWeek, mealType: MealType) => void;
  children?: React.ReactNode;
}

export const MealCalendarDropZone: React.FC<MealCalendarDropZoneProps> = ({
  day,
  mealType,
  mealTypeLabel,
  mealTypeIcon,
  isActive = false,
  isValidTarget = true,
  onDrop,
  children,
}) => {
  const borderOpacity = React.useRef(new Animated.Value(0)).current;
  const backgroundOpacity = React.useRef(new Animated.Value(0)).current;

  React.useEffect(() => {
    if (isActive && isValidTarget) {
      // Animate to active drop state
      Animated.parallel([
        Animated.timing(borderOpacity, {
          toValue: 1,
          duration: 200,
          useNativeDriver: false,
        }),
        Animated.timing(backgroundOpacity, {
          toValue: 0.1,
          duration: 200,
          useNativeDriver: false,
        }),
      ]).start();
    } else {
      // Animate back to normal state
      Animated.parallel([
        Animated.timing(borderOpacity, {
          toValue: 0,
          duration: 200,
          useNativeDriver: false,
        }),
        Animated.timing(backgroundOpacity, {
          toValue: 0,
          duration: 200,
          useNativeDriver: false,
        }),
      ]).start();
    }
  }, [isActive, isValidTarget, borderOpacity, backgroundOpacity]);

  const renderEmptyState = () => (
    <View style={styles.emptyContent}>
      <Text style={styles.emptyIcon}>{mealTypeIcon}</Text>
      <Text style={styles.emptyLabel}>{mealTypeLabel}</Text>
      {isActive && isValidTarget && (
        <Text style={styles.dropHint}>Drop here</Text>
      )}
      {isActive && !isValidTarget && (
        <Text style={styles.invalidDropHint}>Cannot drop here</Text>
      )}
    </View>
  );

  const animatedBorderColor = borderOpacity.interpolate({
    inputRange: [0, 1],
    outputRange: ['transparent', isValidTarget ? '#2196F3' : '#F44336'],
  });

  const animatedBackgroundColor = backgroundOpacity.interpolate({
    inputRange: [0, 1],
    outputRange: ['transparent', isValidTarget ? '#E3F2FD' : '#FFEBEE'],
  });

  return (
    <Animated.View
      style={[
        styles.container,
        {
          borderColor: animatedBorderColor,
          backgroundColor: animatedBackgroundColor,
        },
        isActive && styles.activeContainer,
        isActive && !isValidTarget && styles.invalidContainer,
      ]}
      testID="drop-zone-container"
      isActive={isActive}
      isValidTarget={isValidTarget}
    >
      {children || renderEmptyState()}
      
      {/* Drop indicator overlay */}
      {isActive && (
        <Animated.View
          style={[
            styles.dropIndicator,
            {
              opacity: borderOpacity,
              borderColor: isValidTarget ? '#2196F3' : '#F44336',
            },
          ]}
        >
          <View style={styles.dropIndicatorContent}>
            <Text style={[
              styles.dropIndicatorText,
              { color: isValidTarget ? '#2196F3' : '#F44336' }
            ]}>
              {isValidTarget ? '↓ Drop Here' : '✕ Invalid'}
            </Text>
          </View>
        </Animated.View>
      )}
    </Animated.View>
  );
};

const styles = StyleSheet.create({
  container: {
    minHeight: 100,
    margin: 2,
    borderRadius: 8,
    borderWidth: 2,
    borderColor: 'transparent',
    backgroundColor: 'transparent',
    position: 'relative',
    overflow: 'hidden',
  },
  activeContainer: {
    borderWidth: 2,
  },
  invalidContainer: {
    borderStyle: 'dashed',
  },
  emptyContent: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    padding: 16,
    backgroundColor: '#f8f9fa',
    borderRadius: 6,
    borderWidth: 1,
    borderColor: '#e9ecef',
    borderStyle: 'dashed',
  },
  emptyIcon: {
    fontSize: 24,
    marginBottom: 4,
    opacity: 0.6,
  },
  emptyLabel: {
    fontSize: 12,
    fontWeight: '600',
    color: '#999',
    marginBottom: 2,
  },
  dropHint: {
    fontSize: 10,
    color: '#2196F3',
    fontWeight: '600',
    marginTop: 4,
  },
  invalidDropHint: {
    fontSize: 10,
    color: '#F44336',
    fontWeight: '600',
    marginTop: 4,
  },
  dropIndicator: {
    position: 'absolute',
    top: 0,
    left: 0,
    right: 0,
    bottom: 0,
    borderWidth: 2,
    borderRadius: 6,
    borderStyle: 'dashed',
    justifyContent: 'center',
    alignItems: 'center',
    backgroundColor: 'rgba(255, 255, 255, 0.9)',
  },
  dropIndicatorContent: {
    padding: 8,
    borderRadius: 4,
    backgroundColor: 'rgba(255, 255, 255, 0.95)',
  },
  dropIndicatorText: {
    fontSize: 12,
    fontWeight: '700',
    textAlign: 'center',
  },
});
import React from 'react';
import {
  TouchableOpacity,
  Text,
  StyleSheet,
  Animated,
} from 'react-native';

interface MealLockToggleProps {
  isLocked: boolean;
  onToggle: () => void;
  size?: 'small' | 'medium' | 'large';
  disabled?: boolean;
}

export const MealLockToggle: React.FC<MealLockToggleProps> = ({
  isLocked,
  onToggle,
  size = 'medium',
  disabled = false,
}) => {
  const scaleValue = React.useRef(new Animated.Value(1)).current;

  const handlePress = () => {
    if (disabled) return;

    // Animate button press
    Animated.sequence([
      Animated.timing(scaleValue, {
        toValue: 0.9,
        duration: 100,
        useNativeDriver: true,
      }),
      Animated.timing(scaleValue, {
        toValue: 1,
        duration: 100,
        useNativeDriver: true,
      }),
    ]).start();

    onToggle();
  };

  const getSizeStyles = () => {
    switch (size) {
      case 'small':
        return {
          width: 20,
          height: 20,
          borderRadius: 10,
          fontSize: 10,
        };
      case 'large':
        return {
          width: 36,
          height: 36,
          borderRadius: 18,
          fontSize: 14,
        };
      default: // medium
        return {
          width: 28,
          height: 28,
          borderRadius: 14,
          fontSize: 12,
        };
    }
  };

  const sizeStyles = getSizeStyles();

  return (
    <Animated.View style={[{ transform: [{ scale: scaleValue }] }]}>
      <TouchableOpacity
        style={[
          styles.container,
          {
            width: sizeStyles.width,
            height: sizeStyles.height,
            borderRadius: sizeStyles.borderRadius,
          },
          isLocked ? styles.lockedContainer : styles.unlockedContainer,
          disabled && styles.disabledContainer,
        ]}
        onPress={handlePress}
        disabled={disabled}
        activeOpacity={0.7}
        accessibilityRole="button"
        accessibilityLabel={isLocked ? "Unlock meal" : "Lock meal"}
        accessibilityHint={
          isLocked 
            ? "Unlocks this meal so it can be moved or changed during regeneration"
            : "Locks this meal to prevent changes during regeneration"
        }
      >
        <Text 
          style={[
            styles.icon,
            { fontSize: sizeStyles.fontSize },
            isLocked ? styles.lockedIcon : styles.unlockedIcon,
            disabled && styles.disabledIcon,
          ]}
        >
          {isLocked ? '🔒' : '🔓'}
        </Text>
      </TouchableOpacity>
    </Animated.View>
  );
};

const styles = StyleSheet.create({
  container: {
    justifyContent: 'center',
    alignItems: 'center',
    borderWidth: 1,
    shadowOffset: { width: 0, height: 1 },
    shadowOpacity: 0.2,
    shadowRadius: 2,
    elevation: 2,
  },
  lockedContainer: {
    backgroundColor: '#FFF3CD',
    borderColor: '#FFC107',
  },
  unlockedContainer: {
    backgroundColor: '#F8F9FA',
    borderColor: '#6C757D',
  },
  disabledContainer: {
    backgroundColor: '#E9ECEF',
    borderColor: '#ADB5BD',
    shadowOpacity: 0.1,
    elevation: 1,
  },
  icon: {
    textAlign: 'center',
  },
  lockedIcon: {
    // No additional styling needed
  },
  unlockedIcon: {
    opacity: 0.7,
  },
  disabledIcon: {
    opacity: 0.4,
  },
});
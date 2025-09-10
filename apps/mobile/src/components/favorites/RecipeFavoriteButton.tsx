import React, { useState } from 'react';
import { TouchableOpacity, Text, StyleSheet, Animated, View } from 'react-native';

interface RecipeFavoriteButtonProps {
  recipeId: string;
  isFavorite: boolean;
  onToggle: (recipeId: string) => Promise<void> | void;
  disabled?: boolean;
  size?: 'small' | 'medium' | 'large';
  showLabel?: boolean;
  style?: any;
}

export const RecipeFavoriteButton: React.FC<RecipeFavoriteButtonProps> = ({
  recipeId,
  isFavorite,
  onToggle,
  disabled = false,
  size = 'medium',
  showLabel = false,
  style
}) => {
  const [isLoading, setIsLoading] = useState(false);
  const [scaleAnim] = useState(new Animated.Value(1));

  const handlePress = async () => {
    if (disabled || isLoading) return;

    setIsLoading(true);

    // Animate button press
    Animated.sequence([
      Animated.timing(scaleAnim, {
        toValue: 0.85,
        duration: 100,
        useNativeDriver: true,
      }),
      Animated.timing(scaleAnim, {
        toValue: 1.1,
        duration: 150,
        useNativeDriver: true,
      }),
      Animated.timing(scaleAnim, {
        toValue: 1,
        duration: 100,
        useNativeDriver: true,
      }),
    ]).start();

    try {
      await onToggle(recipeId);
    } catch (error) {
      console.error('Failed to toggle favorite:', error);
    } finally {
      setIsLoading(false);
    }
  };

  const getSizeStyles = () => {
    switch (size) {
      case 'small':
        return {
          width: 28,
          height: 28,
          borderRadius: 14,
          iconSize: 14,
          labelSize: 12,
        };
      case 'large':
        return {
          width: 48,
          height: 48,
          borderRadius: 24,
          iconSize: 24,
          labelSize: 14,
        };
      default: // medium
        return {
          width: 36,
          height: 36,
          borderRadius: 18,
          iconSize: 18,
          labelSize: 13,
        };
    }
  };

  const sizeStyles = getSizeStyles();

  const buttonStyle = [
    styles.button,
    {
      width: sizeStyles.width,
      height: sizeStyles.height,
      borderRadius: sizeStyles.borderRadius,
    },
    isFavorite ? styles.favoriteButton : styles.unfavoriteButton,
    disabled && styles.disabledButton,
    style,
  ];

  const icon = isFavorite ? '❤️' : '🤍';
  const label = isFavorite ? 'Favorited' : 'Add to Favorites';

  return (
    <View style={styles.container}>
      <Animated.View style={{ transform: [{ scale: scaleAnim }] }}>
        <TouchableOpacity
          style={buttonStyle}
          onPress={handlePress}
          disabled={disabled || isLoading}
          activeOpacity={0.7}
        >
          <Text style={[styles.icon, { fontSize: sizeStyles.iconSize }]}>
            {isLoading ? '⏳' : icon}
          </Text>
        </TouchableOpacity>
      </Animated.View>

      {showLabel && (
        <Text style={[styles.label, { fontSize: sizeStyles.labelSize }]}>
          {label}
        </Text>
      )}
    </View>
  );
};

const styles = StyleSheet.create({
  container: {
    alignItems: 'center',
  },
  button: {
    justifyContent: 'center',
    alignItems: 'center',
    borderWidth: 2,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 1 },
    shadowOpacity: 0.1,
    shadowRadius: 2,
    elevation: 2,
  },
  favoriteButton: {
    backgroundColor: '#FEF2F2',
    borderColor: '#EF4444',
  },
  unfavoriteButton: {
    backgroundColor: '#FFFFFF',
    borderColor: '#E5E7EB',
  },
  disabledButton: {
    opacity: 0.6,
  },
  icon: {
    textAlign: 'center',
  },
  label: {
    marginTop: 4,
    color: '#6B7280',
    textAlign: 'center',
    fontWeight: '500',
  },
});
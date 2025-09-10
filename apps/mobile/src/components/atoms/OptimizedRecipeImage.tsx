import React, { useCallback, useMemo } from 'react';
import { View, Text, StyleSheet, ViewStyle, ImageStyle } from 'react-native';
import LazyImage from './LazyImage';
import { useDeviceCapabilities } from '../../hooks/useDeviceCapabilities';

interface OptimizedRecipeImageProps {
  imageUrl?: string;
  title: string;
  width?: number;
  height?: number;
  style?: ViewStyle | ImageStyle;
  resizeMode?: 'cover' | 'contain' | 'stretch' | 'repeat' | 'center';
  priority?: 'high' | 'normal' | 'low';
  showTitle?: boolean;
  fallbackEmoji?: string;
  onLoad?: () => void;
  onError?: (error: any) => void;
}

export const OptimizedRecipeImage: React.FC<OptimizedRecipeImageProps> = ({
  imageUrl,
  title,
  width = 300,
  height = 200,
  style,
  resizeMode = 'cover',
  priority = 'normal',
  showTitle = false,
  fallbackEmoji = '🍽️',
  onLoad,
  onError,
}) => {
  const { isLowEndDevice, hasSlowConnection } = useDeviceCapabilities();

  // Create intelligent placeholder based on recipe content
  const renderIntelligentPlaceholder = useCallback(() => {
    // Simple emoji selection based on title keywords
    const getRecipeEmoji = (recipeTitle: string): string => {
      const titleLower = recipeTitle.toLowerCase();
      
      if (titleLower.includes('pasta') || titleLower.includes('spaghetti') || titleLower.includes('linguine')) return '🍝';
      if (titleLower.includes('pizza')) return '🍕';
      if (titleLower.includes('burger') || titleLower.includes('sandwich')) return '🍔';
      if (titleLower.includes('salad')) return '🥗';
      if (titleLower.includes('soup') || titleLower.includes('broth')) return '🍲';
      if (titleLower.includes('cake') || titleLower.includes('dessert') || titleLower.includes('sweet')) return '🎂';
      if (titleLower.includes('bread') || titleLower.includes('toast')) return '🍞';
      if (titleLower.includes('chicken') || titleLower.includes('poultry')) return '🍗';
      if (titleLower.includes('fish') || titleLower.includes('salmon') || titleLower.includes('tuna')) return '🐟';
      if (titleLower.includes('beef') || titleLower.includes('steak')) return '🥩';
      if (titleLower.includes('rice') || titleLower.includes('risotto')) return '🍚';
      if (titleLower.includes('curry') || titleLower.includes('spicy')) return '🍛';
      if (titleLower.includes('breakfast') || titleLower.includes('pancake') || titleLower.includes('waffle')) return '🥞';
      if (titleLower.includes('coffee') || titleLower.includes('latte')) return '☕';
      if (titleLower.includes('smoothie') || titleLower.includes('juice')) return '🥤';
      if (titleLower.includes('ice cream') || titleLower.includes('frozen')) return '🍨';
      if (titleLower.includes('cookie') || titleLower.includes('biscuit')) return '🍪';
      
      return fallbackEmoji;
    };

    const emoji = getRecipeEmoji(title);

    return (
      <View style={[styles.placeholder, { width, height }]}>
        <View style={styles.placeholderContent}>
          <Text style={styles.placeholderEmoji}>{emoji}</Text>
          {showTitle && (
            <Text style={styles.placeholderTitle} numberOfLines={2}>
              {title}
            </Text>
          )}
          <Text style={styles.placeholderSubtext}>Recipe Image</Text>
        </View>
      </View>
    );
  }, [title, width, height, showTitle, fallbackEmoji]);

  // Create enhanced fallback for when image fails to load
  const renderEnhancedFallback = useCallback(() => {
    return (
      <View style={[styles.fallback, { width, height }]}>
        <View style={styles.fallbackContent}>
          <View style={styles.fallbackIconContainer}>
            <Text style={styles.fallbackEmoji}>{fallbackEmoji}</Text>
          </View>
          <Text style={styles.fallbackTitle} numberOfLines={2}>
            {title}
          </Text>
          <Text style={styles.fallbackSubtext}>Image not available</Text>
        </View>
      </View>
    );
  }, [title, width, height, fallbackEmoji]);

  // Optimize settings based on device capabilities
  const imageSettings = useMemo(() => {
    let progressive = true;
    let highPriority = priority === 'high';

    // Reduce progressive loading for low-end devices
    if (isLowEndDevice() || hasSlowConnection()) {
      progressive = false;
      // Only high priority images get immediate loading on slow devices
      if (priority !== 'high') {
        highPriority = false;
      }
    }

    return {
      progressive,
      highPriority,
    };
  }, [isLowEndDevice, hasSlowConnection, priority]);

  // Don't render image component if no URL provided
  if (!imageUrl) {
    return renderIntelligentPlaceholder();
  }

  return (
    <View style={[styles.container, { width, height }, style]}>
      <LazyImage
        uri={imageUrl}
        width={width}
        height={height}
        resizeMode={resizeMode}
        placeholder={renderIntelligentPlaceholder()}
        fallbackComponent={renderEnhancedFallback()}
        progressive={imageSettings.progressive}
        highPriority={imageSettings.highPriority}
        onLoad={onLoad}
        onError={onError}
      />
      
      {/* Optional title overlay */}
      {showTitle && imageUrl && (
        <View style={styles.titleOverlay}>
          <Text style={styles.titleText} numberOfLines={2}>
            {title}
          </Text>
        </View>
      )}
    </View>
  );
};

const styles = StyleSheet.create({
  container: {
    position: 'relative',
    backgroundColor: '#f5f5f5',
    borderRadius: 8,
    overflow: 'hidden',
  },
  placeholder: {
    backgroundColor: '#f8f8f8',
    justifyContent: 'center',
    alignItems: 'center',
    padding: 16,
  },
  placeholderContent: {
    alignItems: 'center',
  },
  placeholderEmoji: {
    fontSize: 48,
    marginBottom: 8,
  },
  placeholderTitle: {
    fontSize: 14,
    fontWeight: '600',
    color: '#333',
    textAlign: 'center',
    marginBottom: 4,
    lineHeight: 18,
  },
  placeholderSubtext: {
    fontSize: 12,
    color: '#888',
    textAlign: 'center',
  },
  fallback: {
    backgroundColor: '#f0f0f0',
    justifyContent: 'center',
    alignItems: 'center',
    padding: 16,
    borderWidth: 1,
    borderColor: '#e0e0e0',
    borderStyle: 'dashed',
  },
  fallbackContent: {
    alignItems: 'center',
  },
  fallbackIconContainer: {
    width: 60,
    height: 60,
    borderRadius: 30,
    backgroundColor: '#e8e8e8',
    justifyContent: 'center',
    alignItems: 'center',
    marginBottom: 12,
  },
  fallbackEmoji: {
    fontSize: 32,
  },
  fallbackTitle: {
    fontSize: 14,
    fontWeight: '600',
    color: '#555',
    textAlign: 'center',
    marginBottom: 4,
    lineHeight: 18,
  },
  fallbackSubtext: {
    fontSize: 11,
    color: '#999',
    textAlign: 'center',
  },
  titleOverlay: {
    position: 'absolute',
    bottom: 0,
    left: 0,
    right: 0,
    backgroundColor: 'rgba(0, 0, 0, 0.7)',
    padding: 12,
  },
  titleText: {
    color: '#fff',
    fontSize: 14,
    fontWeight: '600',
    lineHeight: 18,
    textAlign: 'center',
  },
});

export default OptimizedRecipeImage;
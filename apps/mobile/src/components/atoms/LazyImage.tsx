import React, { useState, useEffect, useRef } from 'react';
import {
  View,
  Image,
  Animated,
  StyleSheet,
  ViewStyle,
  ImageStyle,
  Dimensions,
  ActivityIndicator,
} from 'react-native';
import { useImageCache } from '../../hooks/useImageCache';
import { useDeviceCapabilities } from '../../hooks/useDeviceCapabilities';

const { width: screenWidth } = Dimensions.get('window');

interface LazyImageProps {
  uri: string;
  width?: number;
  height?: number;
  style?: ImageStyle | ViewStyle;
  resizeMode?: 'cover' | 'contain' | 'stretch' | 'repeat' | 'center';
  placeholder?: React.ReactNode;
  fallbackComponent?: React.ReactNode;
  progressive?: boolean;
  highPriority?: boolean;
  onLoad?: () => void;
  onError?: (error: any) => void;
}

interface ImageSizes {
  thumbnail: string;
  small: string;
  medium: string;
  large: string;
  original: string;
}

export const LazyImage: React.FC<LazyImageProps> = ({
  uri,
  width = screenWidth,
  height = 200,
  style,
  resizeMode = 'cover',
  placeholder,
  fallbackComponent,
  progressive = true,
  highPriority = false,
  onLoad,
  onError,
}) => {
  const [isLoading, setIsLoading] = useState(true);
  const [hasError, setHasError] = useState(false);
  const [currentImageUri, setCurrentImageUri] = useState<string | null>(null);
  const [showHighRes, setShowHighRes] = useState(false);
  
  const fadeAnim = useRef(new Animated.Value(0)).current;
  const { getCachedImageUri, cacheImage } = useImageCache();
  const { getOptimalImageSize, isLowEndDevice, hasSlowConnection } = useDeviceCapabilities();

  // Generate responsive image URLs based on device capabilities
  const getResponsiveImageSizes = (originalUri: string): ImageSizes => {
    const baseUrl = originalUri.split('?')[0];
    const params = new URLSearchParams(originalUri.split('?')[1] || '');
    
    return {
      thumbnail: `${baseUrl}?${params.toString()}&w=150&h=150&q=60&f=webp`,
      small: `${baseUrl}?${params.toString()}&w=300&h=300&q=70&f=webp`,
      medium: `${baseUrl}?${params.toString()}&w=600&h=600&q=75&f=webp`,
      large: `${baseUrl}?${params.toString()}&w=1200&h=1200&q=80&f=webp`,
      original: originalUri,
    };
  };

  // Determine optimal image size based on device and connection
  const getOptimalImageUri = (): string => {
    const sizes = getResponsiveImageSizes(uri);
    const optimalSize = getOptimalImageSize(width, height);
    
    if (isLowEndDevice() || hasSlowConnection()) {
      return sizes.small;
    }
    
    switch (optimalSize) {
      case 'thumbnail':
        return sizes.thumbnail;
      case 'small':
        return sizes.small;
      case 'medium':
        return sizes.medium;
      case 'large':
        return sizes.large;
      default:
        return sizes.medium; // Safe default
    }
  };

  // Progressive loading effect
  useEffect(() => {
    if (!progressive) return;

    const loadProgressively = async () => {
      try {
        setIsLoading(true);
        setHasError(false);

        // First, try to load from cache
        const cachedUri = await getCachedImageUri(uri);
        if (cachedUri) {
          setCurrentImageUri(cachedUri);
          setIsLoading(false);
          fadeIn();
          return;
        }

        // Load thumbnail first for immediate feedback
        const sizes = getResponsiveImageSizes(uri);
        setCurrentImageUri(sizes.thumbnail);
        fadeIn();

        // Pre-cache the optimal size in background
        const optimalUri = getOptimalImageUri();
        await cacheImage(optimalUri, highPriority ? 'high' : 'normal');

        // Switch to high-res after a short delay
        setTimeout(() => {
          setCurrentImageUri(optimalUri);
          setShowHighRes(true);
        }, 300);

        setIsLoading(false);
      } catch (error) {
        console.warn('Progressive image loading failed:', error);
        handleImageError(error);
      }
    };

    loadProgressively();
  }, [uri, progressive, highPriority]);

  // Non-progressive loading
  useEffect(() => {
    if (progressive) return;

    const loadImage = async () => {
      try {
        setIsLoading(true);
        setHasError(false);

        // Check cache first
        const cachedUri = await getCachedImageUri(uri);
        if (cachedUri) {
          setCurrentImageUri(cachedUri);
          setIsLoading(false);
          fadeIn();
          return;
        }

        // Load optimal image directly
        const optimalUri = getOptimalImageUri();
        await cacheImage(optimalUri, highPriority ? 'high' : 'normal');
        setCurrentImageUri(optimalUri);
        setIsLoading(false);
        fadeIn();
      } catch (error) {
        handleImageError(error);
      }
    };

    loadImage();
  }, [uri, progressive]);

  const fadeIn = () => {
    Animated.timing(fadeAnim, {
      toValue: 1,
      duration: 300,
      useNativeDriver: true,
    }).start();
  };

  const handleImageLoad = () => {
    setIsLoading(false);
    onLoad?.();
  };

  const handleImageError = (error: any) => {
    console.error('Image loading error:', error);
    setHasError(true);
    setIsLoading(false);
    onError?.(error);
  };

  const renderPlaceholder = () => {
    if (placeholder) {
      return placeholder;
    }

    return (
      <View style={[styles.placeholder, { width, height }]}>
        <View style={styles.placeholderContent}>
          {isLoading ? (
            <ActivityIndicator size="small" color="#007AFF" />
          ) : (
            <View style={styles.placeholderIcon}>
              <View style={styles.placeholderIconInner}>
                🖼️
              </View>
            </View>
          )}
        </View>
      </View>
    );
  };

  const renderFallback = () => {
    if (fallbackComponent) {
      return fallbackComponent;
    }

    return (
      <View style={[styles.fallback, { width, height }]}>
        <View style={styles.fallbackContent}>
          <View style={styles.fallbackIcon}>📷</View>
          <View style={styles.fallbackText}>Image unavailable</View>
        </View>
      </View>
    );
  };

  const imageStyle = [
    {
      width,
      height,
      opacity: showHighRes ? 1 : 0.8, // Slightly transparent during low-res phase
    },
    style,
  ];

  if (hasError) {
    return renderFallback();
  }

  return (
    <View style={[styles.container, { width, height }]}>
      {/* Placeholder shown while loading */}
      {(isLoading || !currentImageUri) && renderPlaceholder()}
      
      {/* Main image with fade-in animation */}
      {currentImageUri && (
        <Animated.View
          style={[
            styles.imageContainer,
            {
              opacity: fadeAnim,
              width,
              height,
            },
          ]}
        >
          <Image
            source={{ uri: currentImageUri }}
            style={imageStyle}
            resizeMode={resizeMode}
            onLoad={handleImageLoad}
            onError={handleImageError}
            fadeDuration={0} // Disable default fade to use our custom animation
          />
        </Animated.View>
      )}
      
      {/* Loading indicator overlay for progressive loading */}
      {progressive && isLoading && currentImageUri && (
        <View style={styles.loadingOverlay}>
          <ActivityIndicator size="small" color="#007AFF" />
        </View>
      )}
    </View>
  );
};

const styles = StyleSheet.create({
  container: {
    position: 'relative',
    backgroundColor: '#f5f5f5',
  },
  imageContainer: {
    position: 'absolute',
    top: 0,
    left: 0,
  },
  placeholder: {
    backgroundColor: '#f5f5f5',
    justifyContent: 'center',
    alignItems: 'center',
  },
  placeholderContent: {
    alignItems: 'center',
    justifyContent: 'center',
  },
  placeholderIcon: {
    width: 40,
    height: 40,
    borderRadius: 20,
    backgroundColor: '#e0e0e0',
    justifyContent: 'center',
    alignItems: 'center',
  },
  placeholderIconInner: {
    fontSize: 20,
  },
  fallback: {
    backgroundColor: '#f0f0f0',
    justifyContent: 'center',
    alignItems: 'center',
    borderWidth: 1,
    borderColor: '#e0e0e0',
    borderStyle: 'dashed',
  },
  fallbackContent: {
    alignItems: 'center',
  },
  fallbackIcon: {
    fontSize: 32,
    marginBottom: 8,
  },
  fallbackText: {
    fontSize: 12,
    color: '#666',
    textAlign: 'center',
  },
  loadingOverlay: {
    position: 'absolute',
    top: 8,
    right: 8,
    backgroundColor: 'rgba(255, 255, 255, 0.8)',
    borderRadius: 12,
    padding: 4,
  },
});

export default LazyImage;
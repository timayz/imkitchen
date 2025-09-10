import React, { useState, useEffect, useCallback, useMemo } from 'react';
import {
  View,
  FlatList,
  Dimensions,
  StyleSheet,
  TouchableOpacity,
  Text,
  ViewStyle,
} from 'react-native';
import LazyImage from '../atoms/LazyImage';
import { useImageBatchCache } from '../../hooks/useImageCache';
import { useDeviceCapabilities } from '../../hooks/useDeviceCapabilities';

const { width: screenWidth } = Dimensions.get('window');

export interface ImageItem {
  id: string;
  uri: string;
  title?: string;
  aspectRatio?: number;
}

interface ProgressiveImageGalleryProps {
  images: ImageItem[];
  numColumns?: number;
  spacing?: number;
  onImagePress?: (image: ImageItem, index: number) => void;
  preloadCount?: number;
  style?: ViewStyle;
  priority?: 'high' | 'normal' | 'low';
  enableProgressiveLoading?: boolean;
  showLoadingProgress?: boolean;
}

export const ProgressiveImageGallery: React.FC<ProgressiveImageGalleryProps> = ({
  images,
  numColumns = 2,
  spacing = 8,
  onImagePress,
  preloadCount = 6,
  style,
  priority = 'normal',
  enableProgressiveLoading = true,
  showLoadingProgress = false,
}) => {
  const [visibleRange, setVisibleRange] = useState({ start: 0, end: preloadCount });
  const [loadingProgress, setLoadingProgress] = useState(0);
  
  const { batchState, preloadImageBatch } = useImageBatchCache();
  const { shouldPreloadImages, getConcurrentImageLoads } = useDeviceCapabilities();

  // Calculate image dimensions based on screen size and columns
  const imageWidth = useMemo(() => {
    const totalSpacing = spacing * (numColumns + 1);
    return (screenWidth - totalSpacing) / numColumns;
  }, [numColumns, spacing]);

  // Preload visible images and next batch
  useEffect(() => {
    if (!enableProgressiveLoading || !shouldPreloadImages()) return;

    const preloadImages = async () => {
      const imagesToPreload = images
        .slice(0, Math.min(visibleRange.end + preloadCount, images.length))
        .map(img => img.uri);

      if (imagesToPreload.length > 0) {
        await preloadImageBatch(
          imagesToPreload, 
          priority,
          showLoadingProgress ? setLoadingProgress : undefined
        );
      }
    };

    preloadImages();
  }, [visibleRange, images, enableProgressiveLoading, shouldPreloadImages, preloadCount, priority]);

  // Update visible range based on scroll position
  const handleViewableItemsChanged = useCallback(({ viewableItems }: any) => {
    if (viewableItems.length === 0) return;

    const firstVisible = viewableItems[0].index || 0;
    const lastVisible = viewableItems[viewableItems.length - 1].index || 0;
    
    // Expand range to include buffer for smooth scrolling
    const bufferSize = getConcurrentImageLoads();
    const start = Math.max(0, firstVisible - bufferSize);
    const end = Math.min(images.length - 1, lastVisible + bufferSize);

    setVisibleRange({ start, end });
  }, [images.length, getConcurrentImageLoads]);

  const handleImagePress = useCallback((image: ImageItem, index: number) => {
    onImagePress?.(image, index);
  }, [onImagePress]);

  const renderImageItem = useCallback(({ item, index }: { item: ImageItem; index: number }) => {
    const isInVisibleRange = index >= visibleRange.start && index <= visibleRange.end;
    const imageHeight = imageWidth / (item.aspectRatio || 1);

    return (
      <TouchableOpacity
        style={[
          styles.imageContainer,
          {
            width: imageWidth,
            height: imageHeight,
            marginBottom: spacing,
          }
        ]}
        onPress={() => handleImagePress(item, index)}
        activeOpacity={0.8}
      >
        <LazyImage
          uri={item.uri}
          width={imageWidth}
          height={imageHeight}
          style={styles.image}
          resizeMode="cover"
          progressive={enableProgressiveLoading}
          highPriority={index < preloadCount}
          placeholder={
            <View style={[styles.placeholder, { width: imageWidth, height: imageHeight }]}>
              <View style={styles.placeholderContent}>
                <Text style={styles.placeholderIcon}>🖼️</Text>
                {item.title && (
                  <Text style={styles.placeholderTitle} numberOfLines={2}>
                    {item.title}
                  </Text>
                )}
              </View>
            </View>
          }
        />
        
        {/* Image title overlay */}
        {item.title && (
          <View style={styles.titleOverlay}>
            <Text style={styles.titleText} numberOfLines={2}>
              {item.title}
            </Text>
          </View>
        )}

        {/* Loading indicator for progressive loading */}
        {enableProgressiveLoading && isInVisibleRange && batchState.isProcessing && (
          <View style={styles.loadingIndicator}>
            <View style={styles.loadingDot} />
          </View>
        )}
      </TouchableOpacity>
    );
  }, [
    visibleRange, 
    imageWidth, 
    spacing, 
    handleImagePress, 
    enableProgressiveLoading, 
    preloadCount,
    batchState.isProcessing
  ]);

  const keyExtractor = useCallback((item: ImageItem) => item.id, []);

  const getItemLayout = useCallback((data: any, index: number) => {
    const item = images[index];
    const height = imageWidth / (item?.aspectRatio || 1) + spacing;
    return {
      length: height,
      offset: height * index,
      index,
    };
  }, [images, imageWidth, spacing]);

  const renderLoadingProgress = () => {
    if (!showLoadingProgress || !batchState.isProcessing) return null;

    return (
      <View style={styles.progressContainer}>
        <View style={styles.progressBar}>
          <View 
            style={[
              styles.progressFill, 
              { width: `${loadingProgress}%` }
            ]} 
          />
        </View>
        <Text style={styles.progressText}>
          Loading images... {Math.round(loadingProgress)}%
        </Text>
      </View>
    );
  };

  return (
    <View style={[styles.container, style]}>
      {renderLoadingProgress()}
      
      <FlatList
        data={images}
        renderItem={renderImageItem}
        keyExtractor={keyExtractor}
        numColumns={numColumns}
        columnWrapperStyle={numColumns > 1 ? styles.row : undefined}
        contentContainerStyle={[
          styles.listContent,
          { paddingHorizontal: spacing }
        ]}
        onViewableItemsChanged={handleViewableItemsChanged}
        viewabilityConfig={{
          viewAreaCoveragePercentThreshold: 50,
        }}
        getItemLayout={getItemLayout}
        removeClippedSubviews={true}
        maxToRenderPerBatch={getConcurrentImageLoads()}
        updateCellsBatchingPeriod={100}
        windowSize={10}
        initialNumToRender={preloadCount}
        showsVerticalScrollIndicator={false}
      />
    </View>
  );
};

const styles = StyleSheet.create({
  container: {
    flex: 1,
  },
  listContent: {
    paddingTop: 8,
    paddingBottom: 16,
  },
  row: {
    justifyContent: 'space-between',
  },
  imageContainer: {
    position: 'relative',
    backgroundColor: '#f5f5f5',
    borderRadius: 8,
    overflow: 'hidden',
    elevation: 2,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 1 },
    shadowOpacity: 0.1,
    shadowRadius: 2,
  },
  image: {
    width: '100%',
    height: '100%',
  },
  placeholder: {
    backgroundColor: '#f5f5f5',
    justifyContent: 'center',
    alignItems: 'center',
    padding: 16,
  },
  placeholderContent: {
    alignItems: 'center',
  },
  placeholderIcon: {
    fontSize: 32,
    marginBottom: 8,
  },
  placeholderTitle: {
    fontSize: 12,
    color: '#666',
    textAlign: 'center',
    lineHeight: 16,
  },
  titleOverlay: {
    position: 'absolute',
    bottom: 0,
    left: 0,
    right: 0,
    backgroundColor: 'rgba(0, 0, 0, 0.6)',
    padding: 8,
  },
  titleText: {
    color: '#fff',
    fontSize: 12,
    fontWeight: '600',
    lineHeight: 16,
  },
  loadingIndicator: {
    position: 'absolute',
    top: 8,
    right: 8,
  },
  loadingDot: {
    width: 8,
    height: 8,
    borderRadius: 4,
    backgroundColor: '#007AFF',
    opacity: 0.8,
  },
  progressContainer: {
    padding: 16,
    backgroundColor: '#f8f8f8',
    borderBottomWidth: 1,
    borderBottomColor: '#e0e0e0',
  },
  progressBar: {
    height: 4,
    backgroundColor: '#e0e0e0',
    borderRadius: 2,
    overflow: 'hidden',
    marginBottom: 8,
  },
  progressFill: {
    height: '100%',
    backgroundColor: '#007AFF',
    borderRadius: 2,
  },
  progressText: {
    fontSize: 12,
    color: '#666',
    textAlign: 'center',
  },
});

export default ProgressiveImageGallery;
import React, { useState } from 'react';
import {
  View,
  Text,
  Modal,
  Image,
  ScrollView,
  TouchableOpacity,
  StyleSheet,
  Dimensions,
  ActivityIndicator,
} from 'react-native';
import { RatingDistribution } from './RatingDistribution';
import type { CommunityRecipe, RecipeNutrition } from '@imkitchen/shared-types';

interface RecipePreviewModalProps {
  recipe: CommunityRecipe | null;
  visible: boolean;
  onClose: () => void;
  onImport: (recipe: CommunityRecipe) => void;
  onNavigateToDetail: (recipeId: string) => void;
  nutrition?: RecipeNutrition;
  isImporting?: boolean;
}

const { width: screenWidth, height: screenHeight } = Dimensions.get('window');

export const RecipePreviewModal: React.FC<RecipePreviewModalProps> = ({
  recipe,
  visible,
  onClose,
  onImport,
  onNavigateToDetail,
  nutrition,
  isImporting = false,
}) => {
  const [imageLoading, setImageLoading] = useState(true);

  if (!recipe) return null;

  const handleImport = () => {
    onImport(recipe);
  };

  const handleViewDetail = () => {
    onNavigateToDetail(recipe.id);
    onClose();
  };

  const formatTime = (minutes: number) => {
    if (minutes < 60) return `${minutes}m`;
    const hours = Math.floor(minutes / 60);
    const mins = minutes % 60;
    return mins > 0 ? `${hours}h ${mins}m` : `${hours}h`;
  };

  const renderNutritionInfo = () => {
    if (!nutrition) return null;

    return (
      <View style={styles.nutritionSection}>
        <Text style={styles.sectionTitle}>Nutrition (per serving)</Text>
        <View style={styles.nutritionGrid}>
          {nutrition.calories && (
            <View style={styles.nutritionItem}>
              <Text style={styles.nutritionValue}>{nutrition.calories}</Text>
              <Text style={styles.nutritionLabel}>Calories</Text>
            </View>
          )}
          {nutrition.protein && (
            <View style={styles.nutritionItem}>
              <Text style={styles.nutritionValue}>{nutrition.protein}g</Text>
              <Text style={styles.nutritionLabel}>Protein</Text>
            </View>
          )}
          {nutrition.carbs && (
            <View style={styles.nutritionItem}>
              <Text style={styles.nutritionValue}>{nutrition.carbs}g</Text>
              <Text style={styles.nutritionLabel}>Carbs</Text>
            </View>
          )}
          {nutrition.fat && (
            <View style={styles.nutritionItem}>
              <Text style={styles.nutritionValue}>{nutrition.fat}g</Text>
              <Text style={styles.nutritionLabel}>Fat</Text>
            </View>
          )}
        </View>
      </View>
    );
  };

  return (
    <Modal
      visible={visible}
      animationType="slide"
      presentationStyle="pageSheet"
      onRequestClose={onClose}
    >
      <View style={styles.container}>
        <View style={styles.header}>
          <TouchableOpacity
            style={styles.closeButton}
            onPress={onClose}
            accessibilityRole="button"
            accessibilityLabel="Close recipe preview"
          >
            <Text style={styles.closeButtonText}>✕</Text>
          </TouchableOpacity>
        </View>

        <ScrollView style={styles.content} showsVerticalScrollIndicator={false}>
          {/* Recipe Image */}
          <View style={styles.imageContainer}>
            {recipe.imageURL ? (
              <>
                {imageLoading && (
                  <View style={styles.imageLoadingOverlay}>
                    <ActivityIndicator size="large" color="#007AFF" />
                  </View>
                )}
                <Image
                  source={{ uri: recipe.imageURL }}
                  style={styles.recipeImage}
                  onLoadStart={() => setImageLoading(true)}
                  onLoadEnd={() => setImageLoading(false)}
                  resizeMode="cover"
                />
              </>
            ) : (
              <View style={styles.placeholderImage}>
                <Text style={styles.placeholderText}>🍳</Text>
              </View>
            )}
          </View>

          {/* Recipe Info */}
          <View style={styles.infoSection}>
            <Text style={styles.title}>{recipe.title}</Text>
            {recipe.description && (
              <Text style={styles.description}>{recipe.description}</Text>
            )}

            {/* Timing and Complexity */}
            <View style={styles.metaRow}>
              <View style={styles.metaItem}>
                <Text style={styles.metaLabel}>Prep</Text>
                <Text style={styles.metaValue}>{formatTime(recipe.prepTime)}</Text>
              </View>
              <View style={styles.metaItem}>
                <Text style={styles.metaLabel}>Cook</Text>
                <Text style={styles.metaValue}>{formatTime(recipe.cookTime)}</Text>
              </View>
              <View style={styles.metaItem}>
                <Text style={styles.metaLabel}>Total</Text>
                <Text style={styles.metaValue}>{formatTime(recipe.totalTime)}</Text>
              </View>
              <View style={styles.metaItem}>
                <Text style={styles.metaLabel}>Serves</Text>
                <Text style={styles.metaValue}>{recipe.servings}</Text>
              </View>
            </View>

            <View style={styles.complexityRow}>
              <Text style={styles.complexityLabel}>Complexity: </Text>
              <Text style={[
                styles.complexityValue,
                recipe.complexity === 'simple' && styles.complexitySimple,
                recipe.complexity === 'moderate' && styles.complexityModerate,
                recipe.complexity === 'complex' && styles.complexityComplex,
              ]}>
                {recipe.complexity.charAt(0).toUpperCase() + recipe.complexity.slice(1)}
              </Text>
            </View>

            {/* Rating and Community Stats */}
            <View style={styles.ratingSection}>
              <Text style={styles.sectionTitle}>Community Rating</Text>
              <View style={styles.ratingRow}>
                <Text style={styles.averageRating}>
                  {recipe.averageRating.toFixed(1)} ⭐
                </Text>
                <Text style={styles.ratingCount}>
                  ({recipe.totalRatings} rating{recipe.totalRatings !== 1 ? 's' : ''})
                </Text>
              </View>
              
              <RatingDistribution distribution={recipe.ratingDistribution} />

              <View style={styles.communityStats}>
                <Text style={styles.statText}>
                  Imported {recipe.importCount} times
                </Text>
                {recipe.contributorName && (
                  <Text style={styles.statText}>
                    Originally by {recipe.contributorName}
                  </Text>
                )}
              </View>
            </View>

            {/* Tags */}
            {recipe.userTags.length > 0 && (
              <View style={styles.tagsSection}>
                <Text style={styles.sectionTitle}>Tags</Text>
                <View style={styles.tagsContainer}>
                  {recipe.userTags.slice(0, 6).map((tag, index) => (
                    <View key={index} style={styles.tag}>
                      <Text style={styles.tagText}>#{tag}</Text>
                    </View>
                  ))}
                  {recipe.userTags.length > 6 && (
                    <Text style={styles.moreTagsText}>
                      +{recipe.userTags.length - 6} more
                    </Text>
                  )}
                </View>
              </View>
            )}

            {/* Nutrition */}
            {renderNutritionInfo()}
          </View>
        </ScrollView>

        {/* Action Buttons */}
        <View style={styles.actionButtons}>
          <TouchableOpacity
            style={styles.secondaryButton}
            onPress={handleViewDetail}
            accessibilityRole="button"
            accessibilityLabel="View full recipe details"
          >
            <Text style={styles.secondaryButtonText}>View Full Recipe</Text>
          </TouchableOpacity>
          
          <TouchableOpacity
            style={[styles.primaryButton, isImporting && styles.disabledButton]}
            onPress={handleImport}
            disabled={isImporting}
            accessibilityRole="button"
            accessibilityLabel={isImporting ? 'Importing recipe' : 'Import recipe to my collection'}
          >
            {isImporting ? (
              <>
                <ActivityIndicator size="small" color="#fff" style={styles.buttonLoader} />
                <Text style={styles.primaryButtonText}>Importing...</Text>
              </>
            ) : (
              <Text style={styles.primaryButtonText}>Import Recipe</Text>
            )}
          </TouchableOpacity>
        </View>
      </View>
    </Modal>
  );
};

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#fff',
  },
  header: {
    flexDirection: 'row',
    justifyContent: 'flex-end',
    paddingHorizontal: 20,
    paddingTop: 16,
    paddingBottom: 8,
    borderBottomWidth: 1,
    borderBottomColor: '#f0f0f0',
  },
  closeButton: {
    width: 32,
    height: 32,
    borderRadius: 16,
    backgroundColor: '#f0f0f0',
    justifyContent: 'center',
    alignItems: 'center',
  },
  closeButtonText: {
    fontSize: 18,
    color: '#666',
    fontWeight: '600',
  },
  content: {
    flex: 1,
  },
  imageContainer: {
    position: 'relative',
  },
  recipeImage: {
    width: screenWidth,
    height: screenWidth * 0.6,
  },
  placeholderImage: {
    width: screenWidth,
    height: screenWidth * 0.6,
    backgroundColor: '#f5f5f5',
    justifyContent: 'center',
    alignItems: 'center',
  },
  placeholderText: {
    fontSize: 48,
  },
  imageLoadingOverlay: {
    position: 'absolute',
    top: 0,
    left: 0,
    right: 0,
    bottom: 0,
    backgroundColor: '#f5f5f5',
    justifyContent: 'center',
    alignItems: 'center',
    zIndex: 1,
  },
  infoSection: {
    padding: 20,
  },
  title: {
    fontSize: 24,
    fontWeight: '700',
    color: '#333',
    marginBottom: 8,
  },
  description: {
    fontSize: 16,
    color: '#666',
    lineHeight: 22,
    marginBottom: 16,
  },
  metaRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    marginBottom: 12,
  },
  metaItem: {
    alignItems: 'center',
  },
  metaLabel: {
    fontSize: 12,
    color: '#999',
    fontWeight: '600',
    textTransform: 'uppercase',
  },
  metaValue: {
    fontSize: 16,
    color: '#333',
    fontWeight: '600',
    marginTop: 2,
  },
  complexityRow: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 20,
  },
  complexityLabel: {
    fontSize: 14,
    color: '#666',
  },
  complexityValue: {
    fontSize: 14,
    fontWeight: '600',
  },
  complexitySimple: {
    color: '#4CAF50',
  },
  complexityModerate: {
    color: '#FF9800',
  },
  complexityComplex: {
    color: '#F44336',
  },
  ratingSection: {
    marginBottom: 20,
  },
  sectionTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#333',
    marginBottom: 12,
  },
  ratingRow: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 12,
  },
  averageRating: {
    fontSize: 20,
    fontWeight: '700',
    color: '#333',
    marginRight: 8,
  },
  ratingCount: {
    fontSize: 14,
    color: '#666',
  },
  communityStats: {
    marginTop: 12,
  },
  statText: {
    fontSize: 14,
    color: '#666',
    marginBottom: 4,
  },
  tagsSection: {
    marginBottom: 20,
  },
  tagsContainer: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    alignItems: 'center',
  },
  tag: {
    backgroundColor: '#f0f0f0',
    borderRadius: 12,
    paddingHorizontal: 10,
    paddingVertical: 4,
    marginRight: 8,
    marginBottom: 8,
  },
  tagText: {
    fontSize: 12,
    color: '#666',
    fontWeight: '600',
  },
  moreTagsText: {
    fontSize: 12,
    color: '#999',
    fontStyle: 'italic',
    marginLeft: 4,
  },
  nutritionSection: {
    marginBottom: 20,
  },
  nutritionGrid: {
    flexDirection: 'row',
    justifyContent: 'space-between',
  },
  nutritionItem: {
    alignItems: 'center',
    flex: 1,
  },
  nutritionValue: {
    fontSize: 16,
    fontWeight: '700',
    color: '#333',
  },
  nutritionLabel: {
    fontSize: 12,
    color: '#666',
    marginTop: 2,
  },
  actionButtons: {
    flexDirection: 'row',
    gap: 12,
    paddingHorizontal: 20,
    paddingVertical: 16,
    borderTopWidth: 1,
    borderTopColor: '#f0f0f0',
    backgroundColor: '#fff',
  },
  primaryButton: {
    flex: 1,
    backgroundColor: '#007AFF',
    paddingVertical: 14,
    borderRadius: 12,
    alignItems: 'center',
    flexDirection: 'row',
    justifyContent: 'center',
  },
  secondaryButton: {
    flex: 1,
    backgroundColor: '#f0f0f0',
    paddingVertical: 14,
    borderRadius: 12,
    alignItems: 'center',
  },
  primaryButtonText: {
    fontSize: 16,
    fontWeight: '600',
    color: '#fff',
  },
  secondaryButtonText: {
    fontSize: 16,
    fontWeight: '600',
    color: '#333',
  },
  disabledButton: {
    backgroundColor: '#ccc',
  },
  buttonLoader: {
    marginRight: 8,
  },
});
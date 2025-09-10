import React, { useEffect, useState, useCallback } from 'react';
import {
  View,
  Text,
  FlatList,
  StyleSheet,
  SafeAreaView,
  RefreshControl,
  TouchableOpacity,
  ActivityIndicator,
  Alert,
} from 'react-native';
import { useCommunityStore } from '../../store/community_store';
import { RatingStars } from '../../components/atoms/RatingStars';
import { ReviewModal } from '../../components/molecules/ReviewModal';
import type { RecipeRatingWithRecipeInfo } from '@imkitchen/shared-types';

interface PersonalRatingsScreenProps {
  onNavigateToRecipe: (recipeId: string) => void;
}

export const PersonalRatingsScreen: React.FC<PersonalRatingsScreenProps> = ({
  onNavigateToRecipe,
}) => {
  const {
    userRatingHistory,
    loading,
    error,
    fetchUserRatingHistory,
    updateRating,
    deleteRating,
    clearError,
  } = useCommunityStore();

  const [selectedRating, setSelectedRating] = useState<RecipeRatingWithRecipeInfo | null>(null);
  const [showEditModal, setShowEditModal] = useState(false);
  const [refreshing, setRefreshing] = useState(false);

  useEffect(() => {
    if (!userRatingHistory) {
      fetchUserRatingHistory(1);
    }
  }, [userRatingHistory, fetchUserRatingHistory]);

  const handleRefresh = useCallback(async () => {
    setRefreshing(true);
    clearError();
    try {
      await fetchUserRatingHistory(1);
    } finally {
      setRefreshing(false);
    }
  }, [fetchUserRatingHistory, clearError]);

  const handleLoadMore = useCallback(() => {
    if (
      userRatingHistory &&
      userRatingHistory.page < userRatingHistory.totalPages &&
      !loading.history
    ) {
      fetchUserRatingHistory(userRatingHistory.page + 1);
    }
  }, [userRatingHistory, loading.history, fetchUserRatingHistory]);

  const handleEditRating = (rating: RecipeRatingWithRecipeInfo) => {
    setSelectedRating(rating);
    setShowEditModal(true);
  };

  const handleUpdateRating = async (ratingSubmission: any) => {
    if (!selectedRating) return;

    try {
      await updateRating(selectedRating.recipeId, ratingSubmission);
      setShowEditModal(false);
      setSelectedRating(null);
      // Refresh the list to show updated rating
      await fetchUserRatingHistory(1);
    } catch (error) {
      // Error handling is done in the store
      throw error;
    }
  };

  const handleDeleteRating = (rating: RecipeRatingWithRecipeInfo) => {
    Alert.alert(
      'Delete Rating',
      'Are you sure you want to delete this rating? This action cannot be undone.',
      [
        {
          text: 'Cancel',
          style: 'cancel',
        },
        {
          text: 'Delete',
          style: 'destructive',
          onPress: async () => {
            try {
              await deleteRating(rating.recipeId);
              // Refresh the list to remove deleted rating
              await fetchUserRatingHistory(1);
            } catch (error) {
              const message = error instanceof Error ? error.message : 'Failed to delete rating';
              Alert.alert('Delete Failed', message);
            }
          },
        },
      ]
    );
  };

  const renderRatingItem = ({ item }: { item: RecipeRatingWithRecipeInfo }) => (
    <View style={styles.ratingItem}>
      {/* Recipe Info */}
      <TouchableOpacity
        style={styles.recipeHeader}
        onPress={() => onNavigateToRecipe(item.recipeId)}
      >
        <Text style={styles.recipeTitle} numberOfLines={2}>
          {item.recipeTitle}
        </Text>
        <Text style={styles.recipeInfo}>
          {item.recipeCuisineType && `${item.recipeCuisineType} • `}
          {item.recipeComplexity}
        </Text>
      </TouchableOpacity>

      {/* Rating Info */}
      <View style={styles.ratingContent}>
        <View style={styles.ratingHeader}>
          <RatingStars rating={item.rating} size="medium" />
          <Text style={styles.ratingDate}>
            {new Date(item.createdAt).toLocaleDateString()}
          </Text>
        </View>

        {/* Review Text */}
        {item.review && (
          <Text style={styles.reviewText} numberOfLines={3}>
            {item.review}
          </Text>
        )}

        {/* Additional Details */}
        <View style={styles.detailsRow}>
          {item.difficulty && (
            <View style={styles.detailItem}>
              <Text style={styles.detailLabel}>Difficulty:</Text>
              <Text style={[
                styles.detailValue,
                { color: getDifficultyColor(item.difficulty) }
              ]}>
                {getDifficultyLabel(item.difficulty)}
              </Text>
            </View>
          )}

          <View style={styles.detailItem}>
            <Text style={styles.detailLabel}>Cook again:</Text>
            <Text style={[
              styles.detailValue,
              { color: item.wouldCookAgain ? '#4CAF50' : '#F44336' }
            ]}>
              {item.wouldCookAgain ? 'Yes' : 'No'}
            </Text>
          </View>
        </View>

        {/* Action Buttons */}
        <View style={styles.actionButtons}>
          <TouchableOpacity
            style={[styles.actionButton, styles.editButton]}
            onPress={() => handleEditRating(item)}
          >
            <Text style={styles.editButtonText}>Edit</Text>
          </TouchableOpacity>

          <TouchableOpacity
            style={[styles.actionButton, styles.deleteButton]}
            onPress={() => handleDeleteRating(item)}
          >
            <Text style={styles.deleteButtonText}>Delete</Text>
          </TouchableOpacity>
        </View>
      </View>
    </View>
  );

  const renderFooter = () => {
    if (!loading.history) return null;

    return (
      <View style={styles.loadingFooter}>
        <ActivityIndicator size="small" color="#007AFF" />
        <Text style={styles.loadingText}>Loading more ratings...</Text>
      </View>
    );
  };

  const renderEmpty = () => (
    <View style={styles.emptyContainer}>
      <Text style={styles.emptyTitle}>No Ratings Yet</Text>
      <Text style={styles.emptySubtitle}>
        Start rating recipes to see your history here!
      </Text>
    </View>
  );

  if (loading.history && !userRatingHistory) {
    return (
      <SafeAreaView style={styles.container}>
        <View style={styles.loadingContainer}>
          <ActivityIndicator size="large" color="#007AFF" />
          <Text style={styles.loadingText}>Loading your ratings...</Text>
        </View>
      </SafeAreaView>
    );
  }

  if (error) {
    return (
      <SafeAreaView style={styles.container}>
        <View style={styles.errorContainer}>
          <Text style={styles.errorTitle}>Failed to Load Ratings</Text>
          <Text style={styles.errorMessage}>{error}</Text>
          <TouchableOpacity style={styles.retryButton} onPress={handleRefresh}>
            <Text style={styles.retryButtonText}>Try Again</Text>
          </TouchableOpacity>
        </View>
      </SafeAreaView>
    );
  }

  return (
    <SafeAreaView style={styles.container}>
      <View style={styles.header}>
        <Text style={styles.title}>My Ratings</Text>
        {userRatingHistory && (
          <Text style={styles.subtitle}>
            {userRatingHistory.total} rating{userRatingHistory.total !== 1 ? 's' : ''}
          </Text>
        )}
      </View>

      <FlatList
        data={userRatingHistory?.ratings || []}
        renderItem={renderRatingItem}
        keyExtractor={(item) => item.id}
        contentContainerStyle={styles.listContainer}
        refreshControl={
          <RefreshControl refreshing={refreshing} onRefresh={handleRefresh} />
        }
        onEndReached={handleLoadMore}
        onEndReachedThreshold={0.1}
        ListFooterComponent={renderFooter}
        ListEmptyComponent={renderEmpty}
        showsVerticalScrollIndicator={false}
      />

      {/* Edit Modal */}
      <ReviewModal
        visible={showEditModal}
        onClose={() => {
          setShowEditModal(false);
          setSelectedRating(null);
        }}
        onSubmit={handleUpdateRating}
        recipeTitle={selectedRating?.recipeTitle || ''}
        existingRating={selectedRating ? {
          rating: selectedRating.rating,
          review: selectedRating.review,
          difficulty: selectedRating.difficulty,
          wouldCookAgain: selectedRating.wouldCookAgain,
        } : undefined}
        loading={loading.submit}
      />
    </SafeAreaView>
  );
};

const getDifficultyLabel = (difficulty: string): string => {
  switch (difficulty) {
    case 'easier':
      return 'Easier';
    case 'harder':
      return 'Harder';
    default:
      return 'As expected';
  }
};

const getDifficultyColor = (difficulty: string): string => {
  switch (difficulty) {
    case 'easier':
      return '#4CAF50';
    case 'harder':
      return '#F44336';
    default:
      return '#FF9800';
  }
};

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#f5f5f5',
  },
  header: {
    paddingHorizontal: 20,
    paddingVertical: 16,
    backgroundColor: '#fff',
    borderBottomWidth: 1,
    borderBottomColor: '#eee',
  },
  title: {
    fontSize: 24,
    fontWeight: '700',
    color: '#333',
    marginBottom: 4,
  },
  subtitle: {
    fontSize: 14,
    color: '#666',
  },
  listContainer: {
    paddingHorizontal: 20,
    paddingVertical: 16,
  },
  ratingItem: {
    backgroundColor: '#fff',
    borderRadius: 12,
    marginBottom: 16,
    shadowColor: '#000',
    shadowOffset: {
      width: 0,
      height: 2,
    },
    shadowOpacity: 0.1,
    shadowRadius: 4,
    elevation: 3,
  },
  recipeHeader: {
    padding: 16,
    borderBottomWidth: 1,
    borderBottomColor: '#f0f0f0',
  },
  recipeTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#333',
    marginBottom: 4,
  },
  recipeInfo: {
    fontSize: 12,
    color: '#666',
    textTransform: 'capitalize',
  },
  ratingContent: {
    padding: 16,
  },
  ratingHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 12,
  },
  ratingDate: {
    fontSize: 12,
    color: '#666',
  },
  reviewText: {
    fontSize: 14,
    color: '#333',
    lineHeight: 20,
    marginBottom: 12,
  },
  detailsRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    marginBottom: 16,
  },
  detailItem: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  detailLabel: {
    fontSize: 12,
    color: '#666',
    marginRight: 4,
  },
  detailValue: {
    fontSize: 12,
    fontWeight: '600',
  },
  actionButtons: {
    flexDirection: 'row',
    justifyContent: 'flex-end',
    gap: 8,
  },
  actionButton: {
    paddingHorizontal: 16,
    paddingVertical: 8,
    borderRadius: 6,
    borderWidth: 1,
  },
  editButton: {
    borderColor: '#007AFF',
    backgroundColor: 'rgba(0, 122, 255, 0.1)',
  },
  editButtonText: {
    fontSize: 12,
    fontWeight: '600',
    color: '#007AFF',
  },
  deleteButton: {
    borderColor: '#F44336',
    backgroundColor: 'rgba(244, 67, 54, 0.1)',
  },
  deleteButtonText: {
    fontSize: 12,
    fontWeight: '600',
    color: '#F44336',
  },
  loadingContainer: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
  },
  loadingFooter: {
    flexDirection: 'row',
    justifyContent: 'center',
    alignItems: 'center',
    paddingVertical: 20,
    gap: 8,
  },
  loadingText: {
    fontSize: 14,
    color: '#666',
  },
  errorContainer: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    paddingHorizontal: 20,
  },
  errorTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#333',
    marginBottom: 8,
    textAlign: 'center',
  },
  errorMessage: {
    fontSize: 14,
    color: '#666',
    textAlign: 'center',
    marginBottom: 20,
  },
  retryButton: {
    backgroundColor: '#007AFF',
    paddingHorizontal: 24,
    paddingVertical: 12,
    borderRadius: 8,
  },
  retryButtonText: {
    fontSize: 16,
    fontWeight: '600',
    color: '#fff',
  },
  emptyContainer: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    paddingHorizontal: 20,
    paddingTop: 100,
  },
  emptyTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#333',
    marginBottom: 8,
    textAlign: 'center',
  },
  emptySubtitle: {
    fontSize: 14,
    color: '#666',
    textAlign: 'center',
  },
});
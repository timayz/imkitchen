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
  TextInput,
} from 'react-native';
import { useCommunityStore } from '../../store/community_store';
import { RecipeCard } from '../../components/molecules/RecipeCard';
import { RatingDistribution } from '../../components/molecules/RatingDistribution';
import type { CommunityRecipe, CommunityRecipeFilters } from '@imkitchen/shared-types';

interface CommunityRecipesScreenProps {
  onNavigateToRecipe: (recipeId: string) => void;
}

export const CommunityRecipesScreen: React.FC<CommunityRecipesScreenProps> = ({
  onNavigateToRecipe,
}) => {
  const {
    communityRecipes,
    loading,
    error,
    filters,
    currentPage,
    hasMoreRecipes,
    fetchCommunityRecipes,
    searchCommunityRecipes,
    getTrendingRecipes,
    getHighlyRatedRecipes,
    setFilters,
    clearFilters,
    clearError,
  } = useCommunityStore();

  const [searchQuery, setSearchQuery] = useState('');
  const [activeTab, setActiveTab] = useState<'all' | 'trending' | 'highly-rated'>('all');
  const [showFilters, setShowFilters] = useState(false);
  const [refreshing, setRefreshing] = useState(false);

  useEffect(() => {
    if (communityRecipes.length === 0) {
      handleTabChange('all');
    }
  }, []);

  const handleTabChange = useCallback(async (tab: 'all' | 'trending' | 'highly-rated') => {
    setActiveTab(tab);
    clearError();
    
    try {
      switch (tab) {
        case 'trending':
          await getTrendingRecipes(20);
          break;
        case 'highly-rated':
          await getHighlyRatedRecipes(3, 20);
          break;
        default:
          await fetchCommunityRecipes(undefined, 1);
          break;
      }
    } catch (error) {
      // Error handling is done in the store
    }
  }, [fetchCommunityRecipes, getTrendingRecipes, getHighlyRatedRecipes, clearError]);

  const handleSearch = useCallback(async () => {
    if (searchQuery.trim()) {
      try {
        await searchCommunityRecipes(searchQuery.trim());
        setActiveTab('all');
      } catch (error) {
        // Error handling is done in the store
      }
    } else {
      await fetchCommunityRecipes(undefined, 1);
    }
  }, [searchQuery, searchCommunityRecipes, fetchCommunityRecipes]);

  const handleRefresh = useCallback(async () => {
    setRefreshing(true);
    clearError();
    try {
      await handleTabChange(activeTab);
    } finally {
      setRefreshing(false);
    }
  }, [activeTab, handleTabChange, clearError]);

  const handleLoadMore = useCallback(() => {
    if (hasMoreRecipes && !loading.recipes && activeTab === 'all') {
      fetchCommunityRecipes(filters, currentPage + 1);
    }
  }, [hasMoreRecipes, loading.recipes, activeTab, filters, currentPage, fetchCommunityRecipes]);

  const handleSortChange = useCallback((sortBy: string) => {
    const newFilters: Partial<CommunityRecipeFilters> = { sortBy };
    setFilters(newFilters);
    if (activeTab === 'all') {
      fetchCommunityRecipes({ ...filters, ...newFilters }, 1);
    }
  }, [filters, setFilters, fetchCommunityRecipes, activeTab]);

  const renderRecipeItem = ({ item }: { item: CommunityRecipe }) => (
    <RecipeCard
      recipe={{
        id: item.id,
        title: item.title,
        description: item.description || '',
        imageUrl: item.imageURL,
        prepTime: item.prepTime,
        cookTime: item.cookTime,
        totalTime: item.totalTime,
        complexity: item.complexity,
        cuisineType: item.cuisineType,
        mealType: item.mealType,
        servings: item.servings,
        averageRating: item.averageRating,
        totalRatings: item.totalRatings,
        ingredients: [], // Would be loaded separately
        instructions: [], // Would be loaded separately
        dietaryLabels: [],
        isPublic: true,
        userID: '', // Community recipes don't have single user
        createdAt: item.createdAt,
        updatedAt: item.updatedAt,
      }}
      onPress={() => onNavigateToRecipe(item.id)}
      showRatingInteraction={true}
      style={styles.recipeCard}
    />
  );

  const renderHeader = () => (
    <View style={styles.header}>
      {/* Search Bar */}
      <View style={styles.searchContainer}>
        <TextInput
          style={styles.searchInput}
          placeholder="Search community recipes..."
          value={searchQuery}
          onChangeText={setSearchQuery}
          onSubmitEditing={handleSearch}
          returnKeyType="search"
        />
        <TouchableOpacity
          style={styles.searchButton}
          onPress={handleSearch}
        >
          <Text style={styles.searchButtonText}>🔍</Text>
        </TouchableOpacity>
      </View>

      {/* Tab Navigation */}
      <View style={styles.tabContainer}>
        <TouchableOpacity
          style={[styles.tab, activeTab === 'all' && styles.activeTab]}
          onPress={() => handleTabChange('all')}
        >
          <Text style={[styles.tabText, activeTab === 'all' && styles.activeTabText]}>
            All Recipes
          </Text>
        </TouchableOpacity>

        <TouchableOpacity
          style={[styles.tab, activeTab === 'trending' && styles.activeTab]}
          onPress={() => handleTabChange('trending')}
        >
          <Text style={[styles.tabText, activeTab === 'trending' && styles.activeTabText]}>
            Trending
          </Text>
        </TouchableOpacity>

        <TouchableOpacity
          style={[styles.tab, activeTab === 'highly-rated' && styles.activeTab]}
          onPress={() => handleTabChange('highly-rated')}
        >
          <Text style={[styles.tabText, activeTab === 'highly-rated' && styles.activeTabText]}>
            Top Rated
          </Text>
        </TouchableOpacity>
      </View>

      {/* Sort Options (only for All Recipes) */}
      {activeTab === 'all' && (
        <View style={styles.sortContainer}>
          <Text style={styles.sortLabel}>Sort by:</Text>
          <TouchableOpacity
            style={[styles.sortButton, filters.sortBy === 'rating' && styles.activeSortButton]}
            onPress={() => handleSortChange('rating')}
          >
            <Text style={[styles.sortButtonText, filters.sortBy === 'rating' && styles.activeSortButtonText]}>
              Rating
            </Text>
          </TouchableOpacity>

          <TouchableOpacity
            style={[styles.sortButton, filters.sortBy === 'recent' && styles.activeSortButton]}
            onPress={() => handleSortChange('recent')}
          >
            <Text style={[styles.sortButtonText, filters.sortBy === 'recent' && styles.activeSortButtonText]}>
              Recent
            </Text>
          </TouchableOpacity>

          <TouchableOpacity
            style={[styles.sortButton, filters.sortBy === 'popular' && styles.activeSortButton]}
            onPress={() => handleSortChange('popular')}
          >
            <Text style={[styles.sortButtonText, filters.sortBy === 'popular' && styles.activeSortButtonText]}>
              Popular
            </Text>
          </TouchableOpacity>
        </View>
      )}
    </View>
  );

  const renderFooter = () => {
    if (!loading.recipes) return null;

    return (
      <View style={styles.loadingFooter}>
        <ActivityIndicator size="small" color="#007AFF" />
        <Text style={styles.loadingText}>Loading more recipes...</Text>
      </View>
    );
  };

  const renderEmpty = () => (
    <View style={styles.emptyContainer}>
      <Text style={styles.emptyTitle}>No Recipes Found</Text>
      <Text style={styles.emptySubtitle}>
        {searchQuery 
          ? 'Try a different search term or browse all recipes.'
          : 'Be the first to share a recipe with the community!'}
      </Text>
      {searchQuery && (
        <TouchableOpacity
          style={styles.clearSearchButton}
          onPress={() => {
            setSearchQuery('');
            handleTabChange('all');
          }}
        >
          <Text style={styles.clearSearchButtonText}>Clear Search</Text>
        </TouchableOpacity>
      )}
    </View>
  );

  if (loading.recipes && communityRecipes.length === 0) {
    return (
      <SafeAreaView style={styles.container}>
        <View style={styles.loadingContainer}>
          <ActivityIndicator size="large" color="#007AFF" />
          <Text style={styles.loadingText}>Loading community recipes...</Text>
        </View>
      </SafeAreaView>
    );
  }

  if (error) {
    return (
      <SafeAreaView style={styles.container}>
        <View style={styles.errorContainer}>
          <Text style={styles.errorTitle}>Failed to Load Recipes</Text>
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
      <FlatList
        data={communityRecipes}
        renderItem={renderRecipeItem}
        keyExtractor={(item) => item.id}
        ListHeaderComponent={renderHeader}
        ListFooterComponent={renderFooter}
        ListEmptyComponent={renderEmpty}
        contentContainerStyle={styles.listContainer}
        refreshControl={
          <RefreshControl refreshing={refreshing} onRefresh={handleRefresh} />
        }
        onEndReached={handleLoadMore}
        onEndReachedThreshold={0.1}
        showsVerticalScrollIndicator={false}
      />
    </SafeAreaView>
  );
};

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#f5f5f5',
  },
  header: {
    backgroundColor: '#fff',
    paddingHorizontal: 20,
    paddingTop: 16,
    paddingBottom: 12,
    borderBottomWidth: 1,
    borderBottomColor: '#eee',
    marginBottom: 16,
  },
  searchContainer: {
    flexDirection: 'row',
    marginBottom: 16,
    gap: 8,
  },
  searchInput: {
    flex: 1,
    height: 40,
    borderWidth: 1,
    borderColor: '#ddd',
    borderRadius: 8,
    paddingHorizontal: 12,
    fontSize: 16,
    backgroundColor: '#f9f9f9',
  },
  searchButton: {
    width: 40,
    height: 40,
    borderRadius: 8,
    backgroundColor: '#007AFF',
    justifyContent: 'center',
    alignItems: 'center',
  },
  searchButtonText: {
    fontSize: 16,
  },
  tabContainer: {
    flexDirection: 'row',
    marginBottom: 16,
  },
  tab: {
    flex: 1,
    paddingVertical: 12,
    paddingHorizontal: 8,
    borderRadius: 8,
    marginHorizontal: 2,
    backgroundColor: '#f0f0f0',
    alignItems: 'center',
  },
  activeTab: {
    backgroundColor: '#007AFF',
  },
  tabText: {
    fontSize: 14,
    fontWeight: '600',
    color: '#666',
  },
  activeTabText: {
    color: '#fff',
  },
  sortContainer: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 8,
  },
  sortLabel: {
    fontSize: 14,
    color: '#666',
    fontWeight: '600',
  },
  sortButton: {
    paddingHorizontal: 12,
    paddingVertical: 6,
    borderRadius: 6,
    borderWidth: 1,
    borderColor: '#ddd',
    backgroundColor: '#f9f9f9',
  },
  activeSortButton: {
    borderColor: '#007AFF',
    backgroundColor: 'rgba(0, 122, 255, 0.1)',
  },
  sortButtonText: {
    fontSize: 12,
    color: '#666',
    fontWeight: '600',
  },
  activeSortButtonText: {
    color: '#007AFF',
  },
  listContainer: {
    paddingHorizontal: 20,
  },
  recipeCard: {
    marginBottom: 0,
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
    marginBottom: 20,
  },
  clearSearchButton: {
    backgroundColor: '#007AFF',
    paddingHorizontal: 20,
    paddingVertical: 10,
    borderRadius: 8,
  },
  clearSearchButtonText: {
    fontSize: 14,
    fontWeight: '600',
    color: '#fff',
  },
  recipeCardContainer: {
    position: 'relative',
    marginBottom: 16,
  },
  quickImportButton: {
    position: 'absolute',
    top: 12,
    right: 12,
    backgroundColor: '#007AFF',
    paddingHorizontal: 12,
    paddingVertical: 6,
    borderRadius: 16,
    zIndex: 1,
  },
  quickImportText: {
    fontSize: 12,
    fontWeight: '600',
    color: '#fff',
  },
  filtersButton: {
    paddingHorizontal: 12,
    paddingVertical: 6,
    borderRadius: 6,
    borderWidth: 1,
    borderColor: '#007AFF',
    backgroundColor: 'rgba(0, 122, 255, 0.1)',
    marginLeft: 8,
  },
  filtersButtonText: {
    fontSize: 12,
    color: '#007AFF',
    fontWeight: '600',
  },
  advancedFilters: {
    backgroundColor: '#f9f9f9',
    padding: 16,
    marginHorizontal: 20,
    borderRadius: 8,
    marginBottom: 16,
  },
  filterSectionTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#333',
    marginBottom: 12,
  },
  filterGroup: {
    marginBottom: 16,
  },
  filterLabel: {
    fontSize: 14,
    fontWeight: '600',
    color: '#666',
    marginBottom: 8,
  },
  filterChips: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: 8,
  },
  filterChip: {
    paddingHorizontal: 12,
    paddingVertical: 6,
    borderRadius: 16,
    borderWidth: 1,
    borderColor: '#ddd',
    backgroundColor: '#fff',
  },
  activeFilterChip: {
    borderColor: '#007AFF',
    backgroundColor: '#007AFF',
  },
  filterChipText: {
    fontSize: 12,
    color: '#666',
    fontWeight: '600',
  },
  activeFilterChipText: {
    color: '#fff',
  },
});
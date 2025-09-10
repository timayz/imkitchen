import React, { useEffect, useState, useCallback } from 'react';
import {
  View,
  Text,
  FlatList,
  TextInput,
  TouchableOpacity,
  StyleSheet,
  ActivityIndicator,
  RefreshControl,
  Alert,
} from 'react-native';
import { SafeAreaView } from 'react-native-safe-area-context';
import { useNavigation, useFocusEffect } from '@react-navigation/native';
import type { NativeStackNavigationProp } from '@react-navigation/native-stack';
import type { Recipe } from '@imkitchen/shared-types';
import { useRecipeStore } from '../../store/recipe_store';
import { RecipeCard } from '../../components/molecules/RecipeCard';
import { OfflineStatusIndicator } from '../../components/atoms/OfflineStatusIndicator';
import { SyncStatusBanner } from '../../components/atoms/SyncStatusBanner';
import { useCacheWarming } from '../../hooks/useCacheWarming';

type RecipeStackParamList = {
  RecipeList: undefined;
  RecipeDetail: { recipeId: string };
  AddRecipe: undefined;
  ImportRecipe: undefined;
};

type NavigationProp = NativeStackNavigationProp<RecipeStackParamList>;

export const RecipeListScreen: React.FC = () => {
  const navigation = useNavigation<NavigationProp>();
  const {
    recipes,
    searchResults,
    loading,
    error,
    filters,
    searchRecipes,
    setFilters,
    clearFilters,
    clearError,
    networkStatus,
    syncInProgress,
    syncErrors,
    lastSyncAttempt,
    syncPendingChanges,
    isOffline,
  } = useRecipeStore();

  const [searchQuery, setSearchQuery] = useState(filters.search || '');
  const [isRefreshing, setIsRefreshing] = useState(false);
  
  // Initialize cache warming
  const { warmSpecificRecipes } = useCacheWarming({
    enabled: true,
    onAppForeground: true,
    intervalMinutes: 30,
    maxRecipesToWarm: 15,
  });

  // Load initial recipes when screen is focused
  useFocusEffect(
    useCallback(() => {
      if (recipes.length === 0) {
        searchRecipes();
      }
    }, [])
  );

  // Warm cache for currently displayed recipes when they change
  useEffect(() => {
    if (recipes.length > 0 && !isOffline()) {
      const recipeIds = recipes.slice(0, 10).map(recipe => recipe.id); // Warm first 10 recipes
      warmSpecificRecipes(recipeIds);
    }
  }, [recipes, isOffline, warmSpecificRecipes]);

  // Handle search query changes
  const handleSearch = useCallback(
    (query: string) => {
      setSearchQuery(query);
      setFilters({ search: query || undefined });
      searchRecipes({ ...filters, search: query || undefined });
    },
    [filters, setFilters, searchRecipes]
  );

  // Handle refresh
  const handleRefresh = useCallback(async () => {
    setIsRefreshing(true);
    await searchRecipes({ ...filters, page: 1 });
    setIsRefreshing(false);
  }, [filters, searchRecipes]);

  // Handle load more
  const handleLoadMore = useCallback(() => {
    if (
      searchResults &&
      searchResults.page < searchResults.totalPages &&
      !loading
    ) {
      const nextPage = searchResults.page + 1;
      setFilters({ page: nextPage });
      searchRecipes({ ...filters, page: nextPage });
    }
  }, [searchResults, loading, filters, setFilters, searchRecipes]);

  // Handle recipe selection
  const handleRecipePress = useCallback(
    (recipe: Recipe) => {
      navigation.navigate('RecipeDetail', { recipeId: recipe.id });
    },
    [navigation]
  );

  // Handle add recipe
  const handleAddRecipe = useCallback(() => {
    navigation.navigate('AddRecipe');
  }, [navigation]);

  // Handle import recipe
  const handleImportRecipe = useCallback(() => {
    navigation.navigate('ImportRecipe');
  }, [navigation]);

  // Handle filter clear
  const handleClearFilters = useCallback(() => {
    setSearchQuery('');
    clearFilters();
    searchRecipes();
  }, [clearFilters, searchRecipes]);

  // Show error alert
  useEffect(() => {
    if (error) {
      Alert.alert('Error', error, [
        { text: 'OK', onPress: clearError },
      ]);
    }
  }, [error, clearError]);

  const renderRecipeItem = ({ item }: { item: Recipe }) => (
    <RecipeCard
      recipe={item}
      onPress={() => handleRecipePress(item)}
      style={styles.recipeCard}
    />
  );

  const renderEmpty = () => (
    <View style={styles.emptyContainer}>
      <Text style={styles.emptyText}>
        {searchQuery
          ? `No recipes found for "${searchQuery}"`
          : 'No recipes yet. Add your first recipe!'}
      </Text>
      <TouchableOpacity
        style={styles.addButton}
        onPress={handleAddRecipe}
      >
        <Text style={styles.addButtonText}>Add Recipe</Text>
      </TouchableOpacity>
    </View>
  );

  const renderFooter = () => {
    if (!loading) return null;
    
    return (
      <View style={styles.loadingFooter}>
        <ActivityIndicator size="small" color="#007AFF" />
      </View>
    );
  };

  const hasActiveFilters = searchQuery || 
    filters.mealType?.length || 
    filters.complexity?.length || 
    filters.maxTotalTime ||
    filters.cuisineType ||
    filters.dietaryLabels?.length;

  return (
    <SafeAreaView style={styles.container}>
      {/* Header */}
      <View style={styles.header}>
        <View style={styles.titleContainer}>
          <Text style={styles.title}>My Recipes</Text>
          <OfflineStatusIndicator
            networkStatus={networkStatus}
            syncInProgress={syncInProgress}
            syncErrors={syncErrors}
            lastSyncAttempt={lastSyncAttempt}
            size="small"
            showLabel={false}
          />
        </View>
        <View style={styles.headerButtons}>
          <TouchableOpacity
            style={styles.importButton}
            onPress={handleImportRecipe}
          >
            <Text style={styles.importButtonText}>Import</Text>
          </TouchableOpacity>
          <TouchableOpacity
            style={styles.addButton}
            onPress={handleAddRecipe}
          >
            <Text style={styles.addButtonText}>+</Text>
          </TouchableOpacity>
        </View>
      </View>

      {/* Sync Status Banner */}
      <SyncStatusBanner
        networkStatus={networkStatus}
        syncInProgress={syncInProgress}
        syncErrors={syncErrors}
        lastSyncAttempt={lastSyncAttempt}
        onRetrySync={syncPendingChanges}
        onDismissErrors={() => {
          // Clear sync errors by triggering a new sync attempt when online
          if (!isOffline()) {
            syncPendingChanges();
          }
        }}
      />

      {/* Search Bar */}
      <View style={styles.searchContainer}>
        <TextInput
          style={styles.searchInput}
          placeholder="Search recipes..."
          value={searchQuery}
          onChangeText={handleSearch}
          returnKeyType="search"
          onSubmitEditing={() => handleSearch(searchQuery)}
        />
        {hasActiveFilters && (
          <TouchableOpacity
            style={styles.clearFiltersButton}
            onPress={handleClearFilters}
          >
            <Text style={styles.clearFiltersText}>Clear</Text>
          </TouchableOpacity>
        )}
      </View>

      {/* Recipe List */}
      <FlatList
        data={recipes}
        renderItem={renderRecipeItem}
        keyExtractor={(item) => item.id}
        onEndReached={handleLoadMore}
        onEndReachedThreshold={0.1}
        refreshControl={
          <RefreshControl
            refreshing={isRefreshing}
            onRefresh={handleRefresh}
            tintColor="#007AFF"
          />
        }
        ListEmptyComponent={renderEmpty}
        ListFooterComponent={renderFooter}
        contentContainerStyle={recipes.length === 0 ? styles.emptyList : undefined}
        showsVerticalScrollIndicator={false}
      />

      {/* Search Results Info */}
      {searchResults && (
        <View style={styles.resultsInfo}>
          <Text style={styles.resultsText}>
            {searchResults.total} recipe{searchResults.total !== 1 ? 's' : ''} found
          </Text>
        </View>
      )}
    </SafeAreaView>
  );
};

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#fff',
  },
  header: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    paddingHorizontal: 16,
    paddingVertical: 12,
    borderBottomWidth: 1,
    borderBottomColor: '#e0e0e0',
  },
  titleContainer: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 8,
  },
  title: {
    fontSize: 24,
    fontWeight: 'bold',
    color: '#333',
  },
  headerButtons: {
    flexDirection: 'row',
    gap: 8,
  },
  importButton: {
    paddingHorizontal: 12,
    paddingVertical: 8,
    backgroundColor: '#f0f0f0',
    borderRadius: 8,
  },
  importButtonText: {
    color: '#007AFF',
    fontWeight: '600',
  },
  addButton: {
    paddingHorizontal: 12,
    paddingVertical: 8,
    backgroundColor: '#007AFF',
    borderRadius: 8,
    minWidth: 40,
    alignItems: 'center',
  },
  addButtonText: {
    color: '#fff',
    fontWeight: '600',
    fontSize: 16,
  },
  searchContainer: {
    flexDirection: 'row',
    alignItems: 'center',
    paddingHorizontal: 16,
    paddingVertical: 12,
    gap: 8,
  },
  searchInput: {
    flex: 1,
    height: 40,
    paddingHorizontal: 12,
    backgroundColor: '#f8f8f8',
    borderRadius: 8,
    fontSize: 16,
  },
  clearFiltersButton: {
    paddingHorizontal: 12,
    paddingVertical: 8,
  },
  clearFiltersText: {
    color: '#007AFF',
    fontSize: 14,
  },
  recipeCard: {
    marginHorizontal: 16,
    marginVertical: 8,
  },
  emptyContainer: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    paddingHorizontal: 32,
  },
  emptyList: {
    flexGrow: 1,
  },
  emptyText: {
    fontSize: 18,
    color: '#666',
    textAlign: 'center',
    marginBottom: 24,
    lineHeight: 24,
  },
  loadingFooter: {
    paddingVertical: 16,
    alignItems: 'center',
  },
  resultsInfo: {
    paddingHorizontal: 16,
    paddingVertical: 8,
    backgroundColor: '#f8f8f8',
    borderTopWidth: 1,
    borderTopColor: '#e0e0e0',
  },
  resultsText: {
    fontSize: 14,
    color: '#666',
    textAlign: 'center',
  },
});
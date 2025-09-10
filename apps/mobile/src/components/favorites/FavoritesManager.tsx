import React, { useEffect, useState } from 'react';
import { View, Text, FlatList, StyleSheet, ActivityIndicator, Alert, TextInput } from 'react-native';
import { RecipeFavorite, useFavoritesStore } from '../../store/favorites_store';
import { RecipeFavoriteButton } from './RecipeFavoriteButton';
import { FavoritesImportExport } from './FavoritesImportExport';

interface FavoritesManagerProps {
  onRecipeSelect?: (recipeId: string) => void;
  showImportExport?: boolean;
}

export const FavoritesManager: React.FC<FavoritesManagerProps> = ({
  onRecipeSelect,
  showImportExport = true
}) => {
  const {
    favorites,
    isLoading,
    error,
    loadFavorites,
    toggleFavorite,
    clearError
  } = useFavoritesStore();

  const [searchQuery, setSearchQuery] = useState('');
  const [filteredFavorites, setFilteredFavorites] = useState<RecipeFavorite[]>([]);

  useEffect(() => {
    loadFavorites();
  }, [loadFavorites]);

  useEffect(() => {
    if (error) {
      Alert.alert(
        'Error',
        error,
        [{ text: 'OK', onPress: clearError }]
      );
    }
  }, [error, clearError]);

  useEffect(() => {
    if (searchQuery.trim()) {
      const filtered = favorites.filter(favorite =>
        favorite.recipe.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
        favorite.recipe.description?.toLowerCase().includes(searchQuery.toLowerCase()) ||
        favorite.recipe.tags?.some(tag => tag.toLowerCase().includes(searchQuery.toLowerCase()))
      );
      setFilteredFavorites(filtered);
    } else {
      setFilteredFavorites(favorites);
    }
  }, [favorites, searchQuery]);

  const handleToggleFavorite = async (recipeId: string) => {
    try {
      await toggleFavorite(recipeId);
    } catch (err) {
      console.error('Failed to toggle favorite:', err);
      Alert.alert(
        'Error',
        'Failed to update favorite status. Please try again.',
        [{ text: 'OK' }]
      );
    }
  };

  const renderFavoriteItem = ({ item }: { item: RecipeFavorite }) => (
    <View style={styles.favoriteItem}>
      <View style={styles.favoriteContent}>
        <View style={styles.recipeInfo}>
          <Text style={styles.recipeName} numberOfLines={2}>
            {item.recipe.name}
          </Text>
          <Text style={styles.recipeDescription} numberOfLines={3}>
            {item.recipe.description || 'No description available'}
          </Text>
          <View style={styles.recipeStats}>
            <Text style={styles.statText}>
              ⏱️ {item.recipe.prepTime}min
            </Text>
            <Text style={styles.statText}>
              🍳 {item.recipe.complexity}
            </Text>
            <Text style={styles.statText}>
              ⭐ Added {new Date(item.favoriteAt).toLocaleDateString()}
            </Text>
          </View>
          {item.recipe.tags && item.recipe.tags.length > 0 && (
            <View style={styles.tagsContainer}>
              {item.recipe.tags.slice(0, 3).map((tag, index) => (
                <Text key={index} style={styles.tag}>
                  {tag}
                </Text>
              ))}
              {item.recipe.tags.length > 3 && (
                <Text style={styles.moreTagsText}>
                  +{item.recipe.tags.length - 3} more
                </Text>
              )}
            </View>
          )}
        </View>

        <View style={styles.favoriteActions}>
          <RecipeFavoriteButton
            recipeId={item.recipeId}
            isFavorite={true}
            onToggle={() => handleToggleFavorite(item.recipeId)}
            size="large"
          />
        </View>
      </View>
    </View>
  );

  if (isLoading && favorites.length === 0) {
    return (
      <View style={styles.loadingContainer}>
        <ActivityIndicator size="large" color="#EF4444" />
        <Text style={styles.loadingText}>Loading your favorites...</Text>
      </View>
    );
  }

  return (
    <View style={styles.container}>
      <View style={styles.header}>
        <Text style={styles.title}>Your Favorite Recipes</Text>
        <Text style={styles.subtitle}>
          {favorites.length} recipe{favorites.length !== 1 ? 's' : ''} marked as favorite
        </Text>
      </View>

      {favorites.length > 0 && (
        <View style={styles.searchContainer}>
          <TextInput
            style={styles.searchInput}
            placeholder="Search your favorites..."
            value={searchQuery}
            onChangeText={setSearchQuery}
            clearButtonMode="while-editing"
          />
        </View>
      )}

      {filteredFavorites.length === 0 && !isLoading ? (
        <View style={styles.emptyContainer}>
          <Text style={styles.emptyIcon}>❤️</Text>
          <Text style={styles.emptyTitle}>
            {searchQuery ? 'No matching favorites' : 'No favorites yet'}
          </Text>
          <Text style={styles.emptyDescription}>
            {searchQuery 
              ? 'Try adjusting your search terms'
              : 'Start exploring recipes and tap the heart icon to add them here'
            }
          </Text>
        </View>
      ) : (
        <FlatList
          data={filteredFavorites}
          renderItem={renderFavoriteItem}
          keyExtractor={item => item.recipeId}
          showsVerticalScrollIndicator={false}
          contentContainerStyle={styles.listContent}
          refreshing={isLoading}
          onRefresh={loadFavorites}
        />
      )}

      {showImportExport && (
        <FavoritesImportExport />
      )}
    </View>
  );
};

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#F9FAFB',
  },
  loadingContainer: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    backgroundColor: '#F9FAFB',
  },
  loadingText: {
    marginTop: 16,
    fontSize: 16,
    color: '#6B7280',
    textAlign: 'center',
  },
  header: {
    padding: 20,
    backgroundColor: '#FFFFFF',
    borderBottomWidth: 1,
    borderBottomColor: '#E5E7EB',
  },
  title: {
    fontSize: 24,
    fontWeight: '700',
    color: '#1F2937',
    marginBottom: 4,
  },
  subtitle: {
    fontSize: 14,
    color: '#6B7280',
  },
  searchContainer: {
    padding: 16,
    backgroundColor: '#FFFFFF',
  },
  searchInput: {
    backgroundColor: '#F3F4F6',
    borderRadius: 12,
    padding: 12,
    fontSize: 16,
    borderWidth: 1,
    borderColor: '#E5E7EB',
  },
  listContent: {
    padding: 16,
  },
  favoriteItem: {
    backgroundColor: '#FFFFFF',
    borderRadius: 12,
    marginBottom: 12,
    borderWidth: 1,
    borderColor: '#E5E7EB',
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 1 },
    shadowOpacity: 0.05,
    shadowRadius: 2,
    elevation: 2,
  },
  favoriteContent: {
    flexDirection: 'row',
    padding: 16,
  },
  recipeInfo: {
    flex: 1,
    marginRight: 12,
  },
  recipeName: {
    fontSize: 18,
    fontWeight: '600',
    color: '#1F2937',
    marginBottom: 4,
  },
  recipeDescription: {
    fontSize: 14,
    color: '#6B7280',
    marginBottom: 8,
    lineHeight: 20,
  },
  recipeStats: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    marginBottom: 8,
    gap: 12,
  },
  statText: {
    fontSize: 12,
    color: '#6B7280',
  },
  tagsContainer: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: 6,
  },
  tag: {
    backgroundColor: '#F3F4F6',
    borderRadius: 12,
    paddingHorizontal: 8,
    paddingVertical: 4,
    fontSize: 12,
    color: '#6B7280',
  },
  moreTagsText: {
    fontSize: 12,
    color: '#9CA3AF',
    fontStyle: 'italic',
  },
  favoriteActions: {
    justifyContent: 'center',
    alignItems: 'center',
  },
  emptyContainer: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    padding: 32,
  },
  emptyIcon: {
    fontSize: 48,
    marginBottom: 16,
  },
  emptyTitle: {
    fontSize: 20,
    fontWeight: '600',
    color: '#1F2937',
    marginBottom: 8,
    textAlign: 'center',
  },
  emptyDescription: {
    fontSize: 14,
    color: '#6B7280',
    textAlign: 'center',
    lineHeight: 20,
  },
});
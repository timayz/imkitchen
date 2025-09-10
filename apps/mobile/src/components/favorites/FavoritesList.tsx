import React, { useEffect, useState } from 'react';
import { View, Text, FlatList, StyleSheet, TouchableOpacity, ActivityIndicator } from 'react-native';
import { RecipeFavorite, useFavoritesStore } from '../../store/favorites_store';
import { RecipeFavoriteButton } from './RecipeFavoriteButton';

interface FavoritesListProps {
  onRecipePress?: (recipeId: string) => void;
  limit?: number;
  horizontal?: boolean;
  showHeader?: boolean;
}

export const FavoritesList: React.FC<FavoritesListProps> = ({
  onRecipePress,
  limit,
  horizontal = false,
  showHeader = true
}) => {
  const {
    favorites,
    isLoading,
    error,
    loadFavorites,
    toggleFavorite
  } = useFavoritesStore();

  const [displayedFavorites, setDisplayedFavorites] = useState<RecipeFavorite[]>([]);

  useEffect(() => {
    loadFavorites();
  }, [loadFavorites]);

  useEffect(() => {
    const sortedFavorites = [...favorites].sort((a, b) => 
      new Date(b.favoriteAt).getTime() - new Date(a.favoriteAt).getTime()
    );
    
    setDisplayedFavorites(limit ? sortedFavorites.slice(0, limit) : sortedFavorites);
  }, [favorites, limit]);

  const handleToggleFavorite = async (recipeId: string) => {
    try {
      await toggleFavorite(recipeId);
    } catch (err) {
      console.error('Failed to toggle favorite:', err);
    }
  };

  const handleRecipePress = (recipeId: string) => {
    onRecipePress?.(recipeId);
  };

  const renderFavoriteItem = ({ item }: { item: RecipeFavorite }) => (
    <TouchableOpacity
      style={[
        styles.favoriteCard,
        horizontal && styles.horizontalCard
      ]}
      onPress={() => handleRecipePress(item.recipeId)}
      activeOpacity={0.7}
    >
      <View style={styles.cardContent}>
        <View style={styles.recipeInfo}>
          <Text 
            style={styles.recipeName} 
            numberOfLines={horizontal ? 2 : 1}
          >
            {item.recipe.name}
          </Text>
          
          {!horizontal && (
            <Text style={styles.recipeDescription} numberOfLines={2}>
              {item.recipe.description || 'No description'}
            </Text>
          )}
          
          <View style={styles.recipeStats}>
            <Text style={styles.statText}>
              ⏱️ {item.recipe.prepTime}min
            </Text>
            <Text style={styles.statText}>
              🍳 {item.recipe.complexity}
            </Text>
          </View>

          {item.recipe.tags && item.recipe.tags.length > 0 && (
            <View style={styles.tagsContainer}>
              {item.recipe.tags.slice(0, horizontal ? 1 : 2).map((tag, index) => (
                <Text key={index} style={styles.tag}>
                  {tag}
                </Text>
              ))}
            </View>
          )}
        </View>

        <View style={styles.favoriteAction}>
          <RecipeFavoriteButton
            recipeId={item.recipeId}
            isFavorite={true}
            onToggle={handleToggleFavorite}
            size={horizontal ? 'small' : 'medium'}
          />
        </View>
      </View>

      <View style={styles.favoriteMetadata}>
        <Text style={styles.favoriteDate}>
          Added {new Date(item.favoriteAt).toLocaleDateString()}
        </Text>
      </View>
    </TouchableOpacity>
  );

  const renderEmptyState = () => (
    <View style={styles.emptyContainer}>
      <Text style={styles.emptyIcon}>❤️</Text>
      <Text style={styles.emptyTitle}>No favorites yet</Text>
      <Text style={styles.emptyDescription}>
        Tap the heart icon on recipes to add them here
      </Text>
    </View>
  );

  const renderHeader = () => {
    if (!showHeader) return null;

    return (
      <View style={styles.header}>
        <Text style={styles.headerTitle}>
          Favorite Recipes
        </Text>
        <Text style={styles.headerSubtitle}>
          {favorites.length} recipe{favorites.length !== 1 ? 's' : ''}
        </Text>
      </View>
    );
  };

  if (isLoading && favorites.length === 0) {
    return (
      <View style={styles.loadingContainer}>
        <ActivityIndicator size="large" color="#EF4444" />
        <Text style={styles.loadingText}>Loading favorites...</Text>
      </View>
    );
  }

  if (displayedFavorites.length === 0) {
    return (
      <View style={styles.container}>
        {renderHeader()}
        {renderEmptyState()}
      </View>
    );
  }

  return (
    <View style={styles.container}>
      {renderHeader()}
      
      <FlatList
        data={displayedFavorites}
        renderItem={renderFavoriteItem}
        keyExtractor={item => item.recipeId}
        horizontal={horizontal}
        showsHorizontalScrollIndicator={false}
        showsVerticalScrollIndicator={false}
        contentContainerStyle={[
          styles.listContent,
          horizontal && styles.horizontalListContent
        ]}
        ItemSeparatorComponent={() => (
          <View style={horizontal ? styles.horizontalSeparator : styles.verticalSeparator} />
        )}
      />
    </View>
  );
};

const styles = StyleSheet.create({
  container: {
    flex: 1,
  },
  loadingContainer: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    padding: 32,
  },
  loadingText: {
    marginTop: 16,
    fontSize: 14,
    color: '#6B7280',
    textAlign: 'center',
  },
  header: {
    padding: 16,
    backgroundColor: '#FFFFFF',
    borderBottomWidth: 1,
    borderBottomColor: '#E5E7EB',
  },
  headerTitle: {
    fontSize: 20,
    fontWeight: '600',
    color: '#1F2937',
    marginBottom: 4,
  },
  headerSubtitle: {
    fontSize: 14,
    color: '#6B7280',
  },
  listContent: {
    padding: 16,
  },
  horizontalListContent: {
    paddingVertical: 16,
    paddingHorizontal: 16,
  },
  favoriteCard: {
    backgroundColor: '#FFFFFF',
    borderRadius: 12,
    borderWidth: 1,
    borderColor: '#E5E7EB',
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 1 },
    shadowOpacity: 0.05,
    shadowRadius: 2,
    elevation: 2,
  },
  horizontalCard: {
    width: 200,
  },
  cardContent: {
    padding: 16,
    flexDirection: 'row',
  },
  recipeInfo: {
    flex: 1,
    marginRight: 12,
  },
  recipeName: {
    fontSize: 16,
    fontWeight: '600',
    color: '#1F2937',
    marginBottom: 4,
  },
  recipeDescription: {
    fontSize: 14,
    color: '#6B7280',
    marginBottom: 8,
    lineHeight: 18,
  },
  recipeStats: {
    flexDirection: 'row',
    gap: 12,
    marginBottom: 8,
  },
  statText: {
    fontSize: 12,
    color: '#6B7280',
  },
  tagsContainer: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: 4,
  },
  tag: {
    backgroundColor: '#F3F4F6',
    borderRadius: 8,
    paddingHorizontal: 6,
    paddingVertical: 2,
    fontSize: 11,
    color: '#6B7280',
  },
  favoriteAction: {
    justifyContent: 'flex-start',
    alignItems: 'center',
  },
  favoriteMetadata: {
    paddingHorizontal: 16,
    paddingBottom: 12,
    borderTopWidth: 1,
    borderTopColor: '#F3F4F6',
  },
  favoriteDate: {
    fontSize: 11,
    color: '#9CA3AF',
    textAlign: 'right',
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
    fontSize: 18,
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
  horizontalSeparator: {
    width: 12,
  },
  verticalSeparator: {
    height: 12,
  },
});
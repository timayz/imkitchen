import React, { useState, useEffect, useCallback } from 'react';
import {
  View,
  StyleSheet,
  SafeAreaView,
  Alert,
  RefreshControl,
  ScrollView,
  Text,
  TouchableOpacity,
} from 'react-native';
import type { ShoppingListScreenProps } from '../../types/shopping';
import { useShoppingStore } from '../../store/shopping_store';
import { ShoppingProgressBar } from '../../components/shopping/ShoppingProgressBar';
import { GrocerySectionHeader } from '../../components/shopping/GrocerySectionHeader';
import { ShoppingItemCard } from '../../components/shopping/ShoppingItemCard';
import { ShoppingListExportButton } from '../../components/shopping/ShoppingListExportButton';
import { RecipeSourceModal } from '../../components/shopping/RecipeSourceModal';

interface ExpandedSections {
  [category: string]: boolean;
}

interface RecipeSourceModalState {
  isVisible: boolean;
  ingredientName: string;
  recipeSources: string[];
}

export const ShoppingListScreen: React.FC<ShoppingListScreenProps> = ({ 
  listId,
}) => {
  const {
    currentList,
    isLoading,
    error,
    loadShoppingList,
    toggleItemCompleted,
    clearError,
  } = useShoppingStore();

  const [refreshing, setRefreshing] = useState(false);
  const [expandedSections, setExpandedSections] = useState<ExpandedSections>({
    produce: true,
    dairy: true,
    pantry: true,
    protein: true,
    other: true,
  });
  const [recipeSourceModal, setRecipeSourceModal] = useState<RecipeSourceModalState>({
    isVisible: false,
    ingredientName: '',
    recipeSources: [],
  });

  // Load shopping list on mount and when listId changes
  useEffect(() => {
    if (listId) {
      loadShoppingList(listId).catch(err => {
        Alert.alert(
          'Error',
          'Failed to load shopping list. Please try again.',
          [{ text: 'OK' }]
        );
      });
    }
  }, [listId, loadShoppingList]);

  const handleRefresh = useCallback(async () => {
    if (!listId) return;
    
    setRefreshing(true);
    try {
      await loadShoppingList(listId);
    } catch (error) {
      Alert.alert(
        'Error',
        'Failed to refresh shopping list. Please try again.',
        [{ text: 'OK' }]
      );
    } finally {
      setRefreshing(false);
    }
  }, [listId, loadShoppingList]);

  const handleToggleSection = useCallback((category: string) => {
    setExpandedSections(prev => ({
      ...prev,
      [category]: !prev[category],
    }));
  }, []);

  const handleToggleItemCompleted = useCallback(async (
    itemId: string,
    isCompleted: boolean,
    notes?: string
  ) => {
    if (!listId) return;

    try {
      await toggleItemCompleted(listId, itemId, isCompleted, notes);
    } catch (error) {
      Alert.alert(
        'Error',
        'Failed to update item. Please try again.',
        [{ text: 'OK' }]
      );
    }
  }, [listId, toggleItemCompleted]);

  const handleShowRecipeSources = useCallback((
    ingredientName: string,
    recipeSources: string[] = []
  ) => {
    setRecipeSourceModal({
      isVisible: true,
      ingredientName,
      recipeSources,
    });
  }, []);

  const handleCloseRecipeModal = useCallback(() => {
    setRecipeSourceModal({
      isVisible: false,
      ingredientName: '',
      recipeSources: [],
    });
  }, []);

  // Clear any errors when component unmounts
  useEffect(() => {
    return () => {
      if (error) {
        clearError();
      }
    };
  }, [error, clearError]);

  if (!listId) {
    return (
      <SafeAreaView style={styles.container}>
        <View style={styles.emptyState}>
          <Text style={styles.emptyIcon}>🛒</Text>
          <Text style={styles.emptyTitle}>No Shopping List Selected</Text>
          <Text style={styles.emptyMessage}>
            Select a shopping list to view your items.
          </Text>
        </View>
      </SafeAreaView>
    );
  }

  if (isLoading && !currentList) {
    return (
      <SafeAreaView style={styles.container}>
        <View style={styles.loadingState}>
          <Text style={styles.loadingText}>Loading shopping list...</Text>
        </View>
      </SafeAreaView>
    );
  }

  if (!currentList) {
    return (
      <SafeAreaView style={styles.container}>
        <View style={styles.emptyState}>
          <Text style={styles.emptyIcon}>📋</Text>
          <Text style={styles.emptyTitle}>Shopping List Not Found</Text>
          <Text style={styles.emptyMessage}>
            The requested shopping list could not be found.
          </Text>
        </View>
      </SafeAreaView>
    );
  }

  const categoryOrder = ['produce', 'dairy', 'protein', 'pantry', 'other'];
  const categoriesWithItems = categoryOrder.filter(
    category => currentList.categories[category]?.length > 0
  );

  return (
    <SafeAreaView style={styles.container}>
      {/* Header with progress and export */}
      <View style={styles.header}>
        <View style={styles.headerTop}>
          <Text style={styles.listName}>{currentList.name}</Text>
          <ShoppingListExportButton 
            listId={currentList.id}
            disabled={isLoading}
          />
        </View>
        
        <ShoppingProgressBar
          totalItems={currentList.totalItems}
          completedItems={currentList.completedItems}
          showPercentage={true}
        />
      </View>

      {/* Shopping list content */}
      <ScrollView
        style={styles.content}
        refreshControl={
          <RefreshControl refreshing={refreshing} onRefresh={handleRefresh} />
        }
        showsVerticalScrollIndicator={false}
      >
        {categoriesWithItems.length === 0 ? (
          <View style={styles.emptyState}>
            <Text style={styles.emptyIcon}>✅</Text>
            <Text style={styles.emptyTitle}>All Done!</Text>
            <Text style={styles.emptyMessage}>
              You've completed all items in this shopping list.
            </Text>
          </View>
        ) : (
          categoriesWithItems.map(category => {
            const items = currentList.categories[category] || [];
            const completedItems = items.filter(item => item.isCompleted).length;
            const isExpanded = expandedSections[category];

            return (
              <View key={category} style={styles.categorySection}>
                <GrocerySectionHeader
                  category={category}
                  itemCount={items.length}
                  completedCount={completedItems}
                  isExpanded={isExpanded}
                  onToggleExpanded={() => handleToggleSection(category)}
                />
                
                {isExpanded && (
                  <View style={styles.itemsContainer}>
                    {items.map(item => (
                      <ShoppingItemCard
                        key={item.id}
                        item={item}
                        onToggleCompleted={(isCompleted, notes) =>
                          handleToggleItemCompleted(item.id, isCompleted, notes)
                        }
                        onShowRecipeSources={() =>
                          handleShowRecipeSources(
                            item.ingredientName,
                            item.recipeSources
                          )
                        }
                        disabled={isLoading}
                      />
                    ))}
                  </View>
                )}
              </View>
            );
          })
        )}
      </ScrollView>

      {/* Recipe sources modal */}
      <RecipeSourceModal
        isVisible={recipeSourceModal.isVisible}
        onClose={handleCloseRecipeModal}
        ingredientName={recipeSourceModal.ingredientName}
        recipeSources={recipeSourceModal.recipeSources}
      />
    </SafeAreaView>
  );
};

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#f8f9fa',
  },
  header: {
    backgroundColor: '#ffffff',
    paddingHorizontal: 16,
    paddingVertical: 12,
    borderBottomWidth: 1,
    borderBottomColor: '#e9ecef',
    shadowColor: '#000',
    shadowOffset: {
      width: 0,
      height: 2,
    },
    shadowOpacity: 0.1,
    shadowRadius: 3.84,
    elevation: 5,
  },
  headerTop: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 12,
  },
  listName: {
    fontSize: 20,
    fontWeight: '600',
    color: '#2d3436',
    flex: 1,
    marginRight: 12,
  },
  content: {
    flex: 1,
  },
  categorySection: {
    backgroundColor: '#ffffff',
    marginBottom: 8,
  },
  itemsContainer: {
    paddingBottom: 8,
  },
  emptyState: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    padding: 32,
  },
  emptyIcon: {
    fontSize: 64,
    marginBottom: 16,
  },
  emptyTitle: {
    fontSize: 24,
    fontWeight: '600',
    color: '#2d3436',
    textAlign: 'center',
    marginBottom: 8,
  },
  emptyMessage: {
    fontSize: 16,
    color: '#636e72',
    textAlign: 'center',
    lineHeight: 24,
  },
  loadingState: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    padding: 32,
  },
  loadingText: {
    fontSize: 16,
    color: '#636e72',
    textAlign: 'center',
  },
});
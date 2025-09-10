import { create } from 'zustand';
import { devtools, persist } from 'zustand/middleware';
import type {
  ShoppingList,
  ShoppingListFilters,
  ShoppingListExportOptions,
  ShoppingState,
} from '../types/shopping';
import { shoppingService } from '../services/shopping_service';

interface OptimisticUpdate {
  id: string;
  type: 'toggle' | 'update' | 'delete';
  timestamp: number;
  itemId?: string;
  originalData?: any;
  newData?: any;
}

interface ExtendedShoppingState extends ShoppingState {
  // Additional state management
  optimisticUpdates: OptimisticUpdate[];
  
  // Additional actions
  addOptimisticUpdate: (update: OptimisticUpdate) => void;
  removeOptimisticUpdate: (id: string) => void;
  applyOptimisticUpdates: (shoppingList: ShoppingList) => ShoppingList;
}

const initialState = {
  shoppingLists: [],
  currentList: null,
  isGenerating: false,
  isLoading: false,
  error: null,
  optimisticUpdates: [],
};

export const useShoppingStore = create<ExtendedShoppingState>()(
  devtools(
    persist(
      (set, get) => ({
        ...initialState,

        generateShoppingList: async (mealPlanId: string, mergeExisting = false) => {
          set({ isGenerating: true, error: null });

          try {
            const shoppingList = await shoppingService.generateShoppingList({
              mealPlanId,
              mergeExisting,
            });

            const processedList = get().applyOptimisticUpdates(shoppingList);

            set((state) => ({
              shoppingLists: [processedList, ...state.shoppingLists.filter(list => list.id !== shoppingList.id)],
              currentList: processedList,
              isGenerating: false,
              error: null,
            }));
          } catch (error) {
            const errorMessage = error instanceof Error ? error.message : 'Failed to generate shopping list';
            set({
              isGenerating: false,
              error: errorMessage,
            });
            throw error;
          }
        },

        loadShoppingLists: async (filters?: ShoppingListFilters) => {
          set({ isLoading: true, error: null });

          try {
            const shoppingLists = await shoppingService.getShoppingLists(filters);
            
            set({
              shoppingLists,
              isLoading: false,
              error: null,
            });
          } catch (error) {
            const errorMessage = error instanceof Error ? error.message : 'Failed to load shopping lists';
            set({
              isLoading: false,
              error: errorMessage,
            });
            throw error;
          }
        },

        loadShoppingList: async (listId: string) => {
          set({ isLoading: true, error: null });

          try {
            const shoppingList = await shoppingService.getShoppingList(listId);
            const processedList = get().applyOptimisticUpdates(shoppingList);

            set((state) => ({
              currentList: processedList,
              shoppingLists: state.shoppingLists.map(list => 
                list.id === listId ? processedList : list
              ),
              isLoading: false,
              error: null,
            }));
          } catch (error) {
            const errorMessage = error instanceof Error ? error.message : 'Failed to load shopping list';
            set({
              isLoading: false,
              error: errorMessage,
            });
            throw error;
          }
        },

        toggleItemCompleted: async (listId: string, itemId: string, isCompleted: boolean, notes?: string) => {
          const optimisticId = `optimistic-${Date.now()}`;
          
          // Add optimistic update for immediate UI feedback
          get().addOptimisticUpdate({
            id: optimisticId,
            type: 'toggle',
            timestamp: Date.now(),
            itemId,
            originalData: { isCompleted: !isCompleted, notes },
            newData: { isCompleted, notes },
          });

          // Update UI optimistically
          set((state) => {
            const updatedLists = state.shoppingLists.map(list => {
              if (list.id === listId) {
                const updatedCategories = { ...list.categories };
                
                // Find and update the item across all categories
                Object.keys(updatedCategories).forEach(category => {
                  updatedCategories[category] = updatedCategories[category].map(item => {
                    if (item.id === itemId) {
                      return {
                        ...item,
                        isCompleted,
                        notes,
                        completedAt: isCompleted ? new Date() : undefined,
                      };
                    }
                    return item;
                  });
                });

                // Recalculate completed items count
                const allItems = Object.values(updatedCategories).flat();
                const completedItems = allItems.filter(item => item.isCompleted).length;

                const updatedList = {
                  ...list,
                  categories: updatedCategories,
                  completedItems,
                };

                return updatedList;
              }
              return list;
            });

            return {
              shoppingLists: updatedLists,
              currentList: state.currentList?.id === listId 
                ? updatedLists.find(list => list.id === listId) || state.currentList
                : state.currentList,
            };
          });

          try {
            await shoppingService.updateShoppingItem(listId, itemId, { isCompleted, notes });
            get().removeOptimisticUpdate(optimisticId);
          } catch (error) {
            // Revert optimistic update on error
            get().removeOptimisticUpdate(optimisticId);
            
            // Revert UI changes
            set((state) => {
              const updatedLists = state.shoppingLists.map(list => {
                if (list.id === listId) {
                  const updatedCategories = { ...list.categories };
                  
                  Object.keys(updatedCategories).forEach(category => {
                    updatedCategories[category] = updatedCategories[category].map(item => {
                      if (item.id === itemId) {
                        return {
                          ...item,
                          isCompleted: !isCompleted,
                          notes: undefined,
                          completedAt: undefined,
                        };
                      }
                      return item;
                    });
                  });

                  const allItems = Object.values(updatedCategories).flat();
                  const completedItems = allItems.filter(item => item.isCompleted).length;

                  return {
                    ...list,
                    categories: updatedCategories,
                    completedItems,
                  };
                }
                return list;
              });

              return {
                shoppingLists: updatedLists,
                currentList: state.currentList?.id === listId 
                  ? updatedLists.find(list => list.id === listId) || state.currentList
                  : state.currentList,
              };
            });

            const errorMessage = error instanceof Error ? error.message : 'Failed to update shopping item';
            set({ error: errorMessage });
            throw error;
          }
        },

        exportShoppingList: async (listId: string, options: ShoppingListExportOptions): Promise<Blob> => {
          try {
            const blob = await shoppingService.exportShoppingList(listId, options);
            return blob;
          } catch (error) {
            const errorMessage = error instanceof Error ? error.message : 'Failed to export shopping list';
            set({ error: errorMessage });
            throw error;
          }
        },

        deleteShoppingList: async (listId: string) => {
          const optimisticId = `optimistic-${Date.now()}`;
          const originalList = get().shoppingLists.find(list => list.id === listId);

          // Add optimistic update
          get().addOptimisticUpdate({
            id: optimisticId,
            type: 'delete',
            timestamp: Date.now(),
            originalData: originalList,
          });

          // Update UI optimistically
          set((state) => ({
            shoppingLists: state.shoppingLists.filter(list => list.id !== listId),
            currentList: state.currentList?.id === listId ? null : state.currentList,
          }));

          try {
            await shoppingService.deleteShoppingList(listId);
            get().removeOptimisticUpdate(optimisticId);
          } catch (error) {
            // Revert optimistic update on error
            get().removeOptimisticUpdate(optimisticId);
            
            if (originalList) {
              set((state) => ({
                shoppingLists: [...state.shoppingLists, originalList],
                currentList: state.currentList || originalList,
              }));
            }

            const errorMessage = error instanceof Error ? error.message : 'Failed to delete shopping list';
            set({ error: errorMessage });
            throw error;
          }
        },

        clearError: () => set({ error: null }),

        addOptimisticUpdate: (update: OptimisticUpdate) => {
          set((state) => ({
            optimisticUpdates: [...state.optimisticUpdates, update],
          }));
        },

        removeOptimisticUpdate: (id: string) => {
          set((state) => ({
            optimisticUpdates: state.optimisticUpdates.filter((update) => update.id !== id),
          }));
        },

        applyOptimisticUpdates: (shoppingList: ShoppingList): ShoppingList => {
          const { optimisticUpdates } = get();
          
          if (optimisticUpdates.length === 0) {
            return shoppingList;
          }

          let updatedList = { ...shoppingList };
          
          // Apply optimistic updates in chronological order
          optimisticUpdates
            .sort((a, b) => a.timestamp - b.timestamp)
            .forEach((update) => {
              if (update.type === 'toggle' && update.itemId) {
                // Apply item toggle updates
                const updatedCategories = { ...updatedList.categories };
                
                Object.keys(updatedCategories).forEach(category => {
                  updatedCategories[category] = updatedCategories[category].map(item => {
                    if (item.id === update.itemId) {
                      return {
                        ...item,
                        ...update.newData,
                      };
                    }
                    return item;
                  });
                });

                updatedList = {
                  ...updatedList,
                  categories: updatedCategories,
                };
              }
            });

          return updatedList;
        },
      }),
      {
        name: 'shopping-store',
        // Only persist essential data, not loading states or errors
        partialize: (state) => ({
          shoppingLists: state.shoppingLists,
          currentList: state.currentList,
        }),
      }
    ),
    { name: 'ShoppingStore' }
  )
);
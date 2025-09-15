'use client';

import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useDrag, useDrop } from 'react-dnd/dist/hooks';
import {
  CategoryType,
  InventoryItem,
  DragItem,
  DropResult,
} from '@/types/inventory';
import { apiClient } from '@/lib/api-client';

// API function for updating item category
async function updateItemCategory(
  itemId: string,
  category: CategoryType
): Promise<InventoryItem> {
  return apiClient.put<InventoryItem>(`/inventory/${itemId}`, { category });
}

// Hook for drag functionality
export function useDragItem(item: InventoryItem) {
  const [{ isDragging }, drag] = useDrag(() => ({
    type: 'inventory-item',
    item: {
      id: item.id,
      type: 'inventory-item',
      item,
    } as DragItem,
    collect: (monitor: {
      isDragging?: () => boolean;
      isOver?: () => boolean;
      canDrop?: () => boolean;
    }) => ({
      isDragging: monitor.isDragging?.() ?? false,
    }),
  }));

  return { isDragging, drag };
}

// Hook for drop functionality
export function useDropCategory(
  category: CategoryType,
  onDrop?: (item: InventoryItem) => void
) {
  const queryClient = useQueryClient();

  const updateCategoryMutation = useMutation({
    mutationFn: ({
      itemId,
      category,
    }: {
      itemId: string;
      category: CategoryType;
    }) => updateItemCategory(itemId, category),
    onMutate: async ({ itemId, category }) => {
      // Cancel outgoing refetches
      await queryClient.cancelQueries({ queryKey: ['inventory'] });

      // Snapshot previous value
      const previousData = queryClient.getQueryData(['inventory']);

      // Optimistically update
      queryClient.setQueryData(
        ['inventory'],
        (old: InventoryItem[] | undefined) => {
          if (!old) return old;
          return old.map(item =>
            item.id === itemId ? { ...item, category } : item
          );
        }
      );

      return { previousData };
    },
    onError: (_err, _variables, context) => {
      // Rollback on error
      if (context?.previousData) {
        queryClient.setQueryData(['inventory'], context.previousData);
      }
    },
    onSettled: () => {
      queryClient.invalidateQueries({ queryKey: ['inventory'] });
      queryClient.invalidateQueries({ queryKey: ['categoryStats'] });
    },
  });

  const [{ isOver, canDrop }, drop] = useDrop(() => ({
    accept: 'inventory-item',
    drop: (dragItem: DragItem): DropResult => {
      if (dragItem.item.category !== category) {
        updateCategoryMutation.mutate({
          itemId: dragItem.item.id,
          category,
        });

        if (onDrop) {
          onDrop(dragItem.item);
        }
      }

      return { category };
    },
    collect: (monitor: {
      isDragging?: () => boolean;
      isOver?: () => boolean;
      canDrop?: () => boolean;
    }) => ({
      isOver: monitor.isOver?.() ?? false,
      canDrop: monitor.canDrop?.() ?? false,
    }),
  }));

  return {
    isOver,
    canDrop,
    drop,
    isUpdating: updateCategoryMutation.isPending,
  };
}

// Hook for handling keyboard accessibility
export function useKeyboardDragDrop(
  item: InventoryItem,
  availableCategories: CategoryType[],
  onCategoryChange: (category: CategoryType) => void
) {
  const handleKeyDown = (event: React.KeyboardEvent) => {
    if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault();
      // Could open a category selection modal or dropdown
      // For now, cycle through categories
      const currentIndex = availableCategories.indexOf(item.category);
      const nextIndex = (currentIndex + 1) % availableCategories.length;
      onCategoryChange(availableCategories[nextIndex]);
    }
  };

  return { handleKeyDown };
}

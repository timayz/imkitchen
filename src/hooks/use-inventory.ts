import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { InventoryService } from '@/lib/services/inventory-service';
import type {
  InventoryItem,
  InventoryItemCreate,
  InventoryItemUpdate,
  InventoryFilters,
} from '@/types/inventory';

const INVENTORY_QUERY_KEY = 'inventory';

export function useInventoryItems(filters?: InventoryFilters) {
  return useQuery({
    queryKey: [INVENTORY_QUERY_KEY, filters],
    queryFn: () => InventoryService.getItems(filters),
    staleTime: 5 * 60 * 1000, // 5 minutes
  });
}

export function useCreateInventoryItem() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (item: InventoryItemCreate) =>
      InventoryService.createItem(item),
    onSuccess: () => {
      // Invalidate all inventory queries
      queryClient.invalidateQueries({ queryKey: [INVENTORY_QUERY_KEY] });
    },
  });
}

export function useUpdateInventoryItem() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({
      id,
      updates,
    }: {
      id: string;
      updates: InventoryItemUpdate;
    }) => InventoryService.updateItem(id, updates),
    onMutate: async ({ id, updates }) => {
      // Cancel any outgoing refetches
      await queryClient.cancelQueries({ queryKey: [INVENTORY_QUERY_KEY] });

      // Snapshot the previous value
      const previousItems = queryClient.getQueriesData({
        queryKey: [INVENTORY_QUERY_KEY],
      });

      // Optimistically update
      queryClient.setQueriesData(
        { queryKey: [INVENTORY_QUERY_KEY] },
        (old: InventoryItem[] | undefined) => {
          if (!old) return old;
          return old.map(item =>
            item.id === id ? { ...item, ...updates } : item
          );
        }
      );

      return { previousItems };
    },
    onError: (_err, _variables, context) => {
      // Revert optimistic updates
      if (context?.previousItems) {
        context.previousItems.forEach(([queryKey, data]) => {
          queryClient.setQueryData(queryKey, data);
        });
      }
    },
    onSettled: () => {
      queryClient.invalidateQueries({ queryKey: [INVENTORY_QUERY_KEY] });
    },
  });
}

export function useDeleteInventoryItem() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (id: string) => InventoryService.deleteItem(id),
    onMutate: async id => {
      // Cancel any outgoing refetches
      await queryClient.cancelQueries({ queryKey: [INVENTORY_QUERY_KEY] });

      // Snapshot the previous value
      const previousItems = queryClient.getQueriesData({
        queryKey: [INVENTORY_QUERY_KEY],
      });

      // Optimistically update
      queryClient.setQueriesData(
        { queryKey: [INVENTORY_QUERY_KEY] },
        (old: InventoryItem[] | undefined) => {
          if (!old) return old;
          return old.filter(item => item.id !== id);
        }
      );

      return { previousItems };
    },
    onError: (_err, _variables, context) => {
      // Revert optimistic updates
      if (context?.previousItems) {
        context.previousItems.forEach(([queryKey, data]) => {
          queryClient.setQueryData(queryKey, data);
        });
      }
    },
    onSettled: () => {
      queryClient.invalidateQueries({ queryKey: [INVENTORY_QUERY_KEY] });
    },
  });
}

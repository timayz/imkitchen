import { useState } from 'react';
import { ExpirationAlert } from './expiration-alert';
import { Button } from '@/components/ui/button';
import { ConfirmModal } from '@/components/ui/modal';
import {
  useUpdateInventoryItem,
  useDeleteInventoryItem,
} from '@/hooks/use-inventory';
import { useSwipe } from '@/hooks/use-swipe';
import type { InventoryItem } from '@/types/inventory';
import { cn } from '@/lib/utils';

interface InventoryItemProps {
  item: InventoryItem;
  onEdit?: (item: InventoryItem) => void;
  className?: string;
}

export function InventoryItemComponent({
  item,
  onEdit,
  className,
}: InventoryItemProps) {
  const [showDeleteModal, setShowDeleteModal] = useState(false);
  const [showSwipeActions, setShowSwipeActions] = useState(false);
  const updateMutation = useUpdateInventoryItem();
  const deleteMutation = useDeleteInventoryItem();

  const swipeRef = useSwipe(
    {
      onSwipeLeft: () => setShowSwipeActions(true),
      onSwipeRight: () => setShowSwipeActions(false),
    },
    {
      threshold: 50,
    }
  );

  const handleDelete = async () => {
    try {
      await deleteMutation.mutateAsync(item.id);
      setShowDeleteModal(false);
    } catch (error) {
      console.error('Failed to delete item:', error);
    }
  };

  const handleQuantityChange = async (newQuantity: number) => {
    if (newQuantity < 0) return;

    try {
      await updateMutation.mutateAsync({
        id: item.id,
        updates: { quantity: newQuantity },
      });
    } catch (error) {
      console.error('Failed to update quantity:', error);
    }
  };

  return (
    <>
      <div
        ref={swipeRef as React.RefObject<HTMLDivElement>}
        className={cn(
          'bg-white rounded-lg border border-gray-200 p-4 hover:shadow-md transition-all duration-200 relative overflow-hidden',
          showSwipeActions && 'transform -translate-x-20',
          className
        )}
      >
        <div className="flex items-start justify-between">
          <div className="flex-1 min-w-0">
            <h3 className="text-lg font-medium text-gray-900 truncate">
              {item.name}
            </h3>

            <div className="mt-1 flex items-center space-x-4 text-sm text-gray-500">
              <span className="capitalize">{item.category}</span>
              <span className="capitalize">{item.location}</span>
              {item.addedByUser && (
                <span>Added by {item.addedByUser.name || 'Unknown'}</span>
              )}
            </div>

            <div className="mt-2 flex items-center space-x-4">
              <div className="flex items-center space-x-2">
                <span className="text-sm text-gray-700">Quantity:</span>
                <div className="flex items-center space-x-1">
                  <Button
                    variant="ghost"
                    size="sm"
                    className="h-6 w-6 p-0"
                    onClick={() => handleQuantityChange(item.quantity - 1)}
                    disabled={item.quantity <= 1 || updateMutation.isPending}
                  >
                    -
                  </Button>
                  <span className="mx-2 text-sm font-medium">
                    {item.quantity} {item.unit}
                  </span>
                  <Button
                    variant="ghost"
                    size="sm"
                    className="h-6 w-6 p-0"
                    onClick={() => handleQuantityChange(item.quantity + 1)}
                    disabled={updateMutation.isPending}
                  >
                    +
                  </Button>
                </div>
              </div>

              {item.estimatedCost && (
                <span className="text-sm text-gray-700">
                  ${item.estimatedCost.toFixed(2)}
                </span>
              )}
            </div>

            {item.expirationDate && (
              <div className="mt-3">
                <ExpirationAlert
                  expirationDate={new Date(item.expirationDate)}
                />
              </div>
            )}
          </div>

          {/* Desktop Actions */}
          <div className="ml-4 hidden sm:flex flex-col space-y-2">
            <Button
              variant="secondary"
              size="sm"
              onClick={() => onEdit?.(item)}
              disabled={updateMutation.isPending}
            >
              Edit
            </Button>

            <Button
              variant="danger"
              size="sm"
              onClick={() => setShowDeleteModal(true)}
              disabled={deleteMutation.isPending}
            >
              Delete
            </Button>
          </div>

          {/* Mobile: Tap to toggle actions */}
          <div className="ml-4 sm:hidden">
            <Button
              variant="ghost"
              size="sm"
              onClick={() => setShowSwipeActions(!showSwipeActions)}
              className="h-8 w-8 p-0"
            >
              ⋯
            </Button>
          </div>
        </div>

        {/* Mobile Swipe Actions */}
        {showSwipeActions && (
          <div className="absolute right-0 top-0 h-full flex items-center bg-gray-100 px-2 sm:hidden">
            <div className="flex space-x-2">
              <Button
                variant="secondary"
                size="sm"
                onClick={() => {
                  onEdit?.(item);
                  setShowSwipeActions(false);
                }}
                disabled={updateMutation.isPending}
              >
                Edit
              </Button>

              <Button
                variant="danger"
                size="sm"
                onClick={() => {
                  setShowDeleteModal(true);
                  setShowSwipeActions(false);
                }}
                disabled={deleteMutation.isPending}
              >
                Delete
              </Button>
            </div>
          </div>
        )}
      </div>

      <ConfirmModal
        isOpen={showDeleteModal}
        onClose={() => setShowDeleteModal(false)}
        onConfirm={handleDelete}
        title="Delete Item"
        message={`Are you sure you want to delete "${item.name}"? This action cannot be undone.`}
        confirmLabel="Delete"
        cancelLabel="Cancel"
        variant="danger"
      />
    </>
  );
}

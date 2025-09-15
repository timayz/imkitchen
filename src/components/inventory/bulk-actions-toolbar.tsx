'use client';

import { useState } from 'react';
import { Check, X, ChevronDown, Package, MapPin } from 'lucide-react';
import { clsx } from 'clsx';
import {
  InventoryCategory,
  CategoryType,
  StorageLocation,
  CustomCategory,
} from '@/types/inventory';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { InventoryService } from '@/lib/services/inventory-service';

interface BulkActionsToolbarProps {
  selectedItemIds: string[];
  onClearSelection: () => void;
  customCategories?: CustomCategory[];
  className?: string;
}

const predefinedCategories: InventoryCategory[] = [
  'proteins',
  'vegetables',
  'fruits',
  'grains',
  'dairy',
  'spices',
  'condiments',
  'beverages',
  'baking',
  'frozen',
];

const storageLocations: StorageLocation[] = [
  'pantry',
  'refrigerator',
  'freezer',
];

export function BulkActionsToolbar({
  selectedItemIds,
  onClearSelection,
  customCategories = [],
  className = '',
}: BulkActionsToolbarProps) {
  const [showCategoryDropdown, setShowCategoryDropdown] = useState(false);
  const [showLocationDropdown, setShowLocationDropdown] = useState(false);
  const queryClient = useQueryClient();

  const bulkUpdateMutation = useMutation({
    mutationFn: InventoryService.bulkUpdateItems,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['inventory'] });
      queryClient.invalidateQueries({ queryKey: ['categoryStats'] });
      onClearSelection();
    },
  });

  const handleCategoryChange = async (category: CategoryType) => {
    try {
      await bulkUpdateMutation.mutateAsync({
        itemIds: selectedItemIds,
        updates: { category },
      });
      setShowCategoryDropdown(false);
    } catch (error) {
      console.error('Failed to update categories:', error);
    }
  };

  const handleLocationChange = async (location: StorageLocation) => {
    try {
      await bulkUpdateMutation.mutateAsync({
        itemIds: selectedItemIds,
        updates: { location },
      });
      setShowLocationDropdown(false);
    } catch (error) {
      console.error('Failed to update locations:', error);
    }
  };

  const formatCategoryName = (category: string) => {
    if (predefinedCategories.includes(category as InventoryCategory)) {
      return category.charAt(0).toUpperCase() + category.slice(1);
    }
    const customCategory = customCategories.find(c => c.id === category);
    return customCategory?.name || category;
  };

  if (selectedItemIds.length === 0) {
    return null;
  }

  return (
    <div
      className={clsx(
        'fixed bottom-4 left-1/2 transform -translate-x-1/2 z-50',
        'bg-white border border-gray-200 rounded-lg shadow-lg',
        'flex items-center gap-4 px-6 py-4',
        'animate-in slide-in-from-bottom-2 duration-200',
        className
      )}
    >
      {/* Selection Info */}
      <div className="flex items-center gap-2">
        <div className="w-5 h-5 bg-orange-500 rounded-full flex items-center justify-center">
          <Check className="w-3 h-3 text-white" />
        </div>
        <span className="font-medium text-gray-900">
          {selectedItemIds.length} item{selectedItemIds.length !== 1 ? 's' : ''}{' '}
          selected
        </span>
      </div>

      {/* Actions */}
      <div className="flex items-center gap-2">
        {/* Change Category */}
        <div className="relative">
          <button
            onClick={() => setShowCategoryDropdown(!showCategoryDropdown)}
            disabled={bulkUpdateMutation.isPending}
            className="flex items-center gap-2 px-3 py-2 text-sm border border-gray-300 rounded-lg hover:bg-gray-50 transition-colors disabled:opacity-50"
          >
            <Package className="w-4 h-4" />
            Change Category
            <ChevronDown className="w-3 h-3" />
          </button>

          {showCategoryDropdown && (
            <div className="absolute bottom-full mb-2 left-0 bg-white border border-gray-200 rounded-lg shadow-lg min-w-48 max-h-64 overflow-y-auto">
              {/* Predefined Categories */}
              <div className="p-2">
                <div className="text-xs font-medium text-gray-500 px-2 py-1">
                  Predefined
                </div>
                {predefinedCategories.map(category => (
                  <button
                    key={category}
                    onClick={() => handleCategoryChange(category)}
                    className="w-full text-left px-2 py-2 text-sm hover:bg-gray-50 rounded transition-colors"
                  >
                    {formatCategoryName(category)}
                  </button>
                ))}
              </div>

              {/* Custom Categories */}
              {customCategories.length > 0 && (
                <div className="border-t border-gray-200 p-2">
                  <div className="text-xs font-medium text-gray-500 px-2 py-1">
                    Custom
                  </div>
                  {customCategories.map(category => (
                    <button
                      key={category.id}
                      onClick={() => handleCategoryChange(category.id)}
                      className="w-full text-left px-2 py-2 text-sm hover:bg-gray-50 rounded transition-colors flex items-center gap-2"
                    >
                      <div
                        className="w-3 h-3 rounded-full"
                        style={{ backgroundColor: category.color }}
                      />
                      {category.name}
                    </button>
                  ))}
                </div>
              )}
            </div>
          )}
        </div>

        {/* Change Location */}
        <div className="relative">
          <button
            onClick={() => setShowLocationDropdown(!showLocationDropdown)}
            disabled={bulkUpdateMutation.isPending}
            className="flex items-center gap-2 px-3 py-2 text-sm border border-gray-300 rounded-lg hover:bg-gray-50 transition-colors disabled:opacity-50"
          >
            <MapPin className="w-4 h-4" />
            Change Location
            <ChevronDown className="w-3 h-3" />
          </button>

          {showLocationDropdown && (
            <div className="absolute bottom-full mb-2 left-0 bg-white border border-gray-200 rounded-lg shadow-lg min-w-40">
              <div className="p-2">
                {storageLocations.map(location => (
                  <button
                    key={location}
                    onClick={() => handleLocationChange(location)}
                    className="w-full text-left px-2 py-2 text-sm hover:bg-gray-50 rounded transition-colors"
                  >
                    {location.charAt(0).toUpperCase() + location.slice(1)}
                  </button>
                ))}
              </div>
            </div>
          )}
        </div>

        {/* Clear Selection */}
        <button
          onClick={onClearSelection}
          className="p-2 text-gray-600 hover:bg-gray-100 rounded-lg transition-colors"
          title="Clear selection"
        >
          <X className="w-4 h-4" />
        </button>
      </div>

      {/* Loading State */}
      {bulkUpdateMutation.isPending && (
        <div className="flex items-center gap-2 text-sm text-gray-600">
          <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-orange-500"></div>
          Updating...
        </div>
      )}
    </div>
  );
}

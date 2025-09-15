import { useState } from 'react';
import { InventoryItemComponent } from './inventory-item';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Select } from '@/components/ui/select';
import { LoadingSpinner } from '@/components/ui/loading-spinner';
import { useInventoryItems } from '@/hooks/use-inventory';
import type {
  InventoryItem,
  InventoryFilters,
  StorageLocation,
  InventoryCategory,
} from '@/types/inventory';
import { cn } from '@/lib/utils';

interface InventoryListProps {
  onAddItem?: () => void;
  onEditItem?: (item: InventoryItem) => void;
  className?: string;
}

export function InventoryList({
  onAddItem,
  onEditItem,
  className,
}: InventoryListProps) {
  const [selectedLocation, setSelectedLocation] = useState<
    StorageLocation | 'all'
  >('all');
  const [selectedCategory, setSelectedCategory] = useState<
    InventoryCategory | 'all'
  >('all');
  const [searchQuery, setSearchQuery] = useState('');

  const filters: InventoryFilters = {
    ...(selectedLocation !== 'all' && {
      location: selectedLocation as StorageLocation,
    }),
    ...(selectedCategory !== 'all' && {
      category: selectedCategory as InventoryCategory,
    }),
    ...(searchQuery && { search: searchQuery }),
  };

  const { data: items = [], isLoading, error } = useInventoryItems(filters);

  // Group items by location
  const groupedItems = items.reduce(
    (acc: Record<StorageLocation, InventoryItem[]>, item) => {
      if (!acc[item.location]) {
        acc[item.location] = [];
      }
      acc[item.location].push(item);
      return acc;
    },
    {} as Record<StorageLocation, InventoryItem[]>
  );

  const locations: StorageLocation[] = ['pantry', 'refrigerator', 'freezer'];
  const categories: InventoryCategory[] = [
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

  const getLocationDisplayName = (location: StorageLocation) => {
    return location.charAt(0).toUpperCase() + location.slice(1);
  };

  const getCategoryDisplayName = (category: InventoryCategory) => {
    return category.charAt(0).toUpperCase() + category.slice(1);
  };

  if (error) {
    return (
      <div className="text-center py-8">
        <p className="text-red-600">Failed to load inventory items</p>
        <Button
          variant="secondary"
          className="mt-4"
          onClick={() => window.location.reload()}
        >
          Retry
        </Button>
      </div>
    );
  }

  return (
    <div className={cn('space-y-6', className)}>
      {/* Header and Controls */}
      <div className="flex flex-col sm:flex-row justify-between items-start sm:items-center space-y-4 sm:space-y-0">
        <h1 className="text-2xl font-bold text-gray-900">Kitchen Inventory</h1>
        <Button onClick={onAddItem}>Add Item</Button>
      </div>

      {/* Filters */}
      <div className="grid grid-cols-1 sm:grid-cols-3 gap-4">
        <div>
          <label
            htmlFor="search"
            className="block text-sm font-medium text-gray-700 mb-1"
          >
            Search
          </label>
          <Input
            id="search"
            type="text"
            placeholder="Search items..."
            value={searchQuery}
            onChange={e => setSearchQuery(e.target.value)}
          />
        </div>

        <div>
          <label
            htmlFor="location"
            className="block text-sm font-medium text-gray-700 mb-1"
          >
            Location
          </label>
          <Select
            id="location"
            value={selectedLocation}
            onChange={e =>
              setSelectedLocation(e.target.value as StorageLocation | 'all')
            }
          >
            <option value="all">All Locations</option>
            {locations.map(location => (
              <option key={location} value={location}>
                {getLocationDisplayName(location)}
              </option>
            ))}
          </Select>
        </div>

        <div>
          <label
            htmlFor="category"
            className="block text-sm font-medium text-gray-700 mb-1"
          >
            Category
          </label>
          <Select
            id="category"
            value={selectedCategory}
            onChange={e =>
              setSelectedCategory(e.target.value as InventoryCategory | 'all')
            }
          >
            <option value="all">All Categories</option>
            {categories.map(category => (
              <option key={category} value={category}>
                {getCategoryDisplayName(category)}
              </option>
            ))}
          </Select>
        </div>
      </div>

      {/* Loading State */}
      {isLoading && (
        <div className="text-center py-8">
          <LoadingSpinner />
          <p className="mt-2 text-gray-600">Loading inventory...</p>
        </div>
      )}

      {/* Inventory Sections */}
      {!isLoading && (
        <div className="space-y-8">
          {selectedLocation === 'all' ? (
            // Show all locations as separate sections
            locations.map(location => {
              const locationItems = groupedItems[location] || [];

              return (
                <div key={location} className="space-y-4">
                  <div className="flex items-center justify-between">
                    <h2 className="text-xl font-semibold text-gray-900">
                      {getLocationDisplayName(location)}
                    </h2>
                    <span className="text-sm text-gray-500">
                      {locationItems.length} items
                    </span>
                  </div>

                  {locationItems.length === 0 ? (
                    <div className="text-center py-8 bg-gray-50 rounded-lg">
                      <p className="text-gray-500">No items in {location}</p>
                      <Button
                        variant="ghost"
                        className="mt-2"
                        onClick={onAddItem}
                      >
                        Add your first item
                      </Button>
                    </div>
                  ) : (
                    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                      {locationItems.map((item: InventoryItem) => (
                        <InventoryItemComponent
                          key={item.id}
                          item={item}
                          {...(onEditItem && { onEdit: onEditItem })}
                        />
                      ))}
                    </div>
                  )}
                </div>
              );
            })
          ) : (
            // Show single location
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <h2 className="text-xl font-semibold text-gray-900">
                  {getLocationDisplayName(selectedLocation as StorageLocation)}
                </h2>
                <span className="text-sm text-gray-500">
                  {items.length} items
                </span>
              </div>

              {items.length === 0 ? (
                <div className="text-center py-8 bg-gray-50 rounded-lg">
                  <p className="text-gray-500">
                    No items found {searchQuery && `for "${searchQuery}"`}
                  </p>
                  <Button variant="ghost" className="mt-2" onClick={onAddItem}>
                    Add an item
                  </Button>
                </div>
              ) : (
                <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                  {items.map((item: InventoryItem) => (
                    <InventoryItemComponent
                      key={item.id}
                      item={item}
                      {...(onEditItem && { onEdit: onEditItem })}
                    />
                  ))}
                </div>
              )}
            </div>
          )}
        </div>
      )}
    </div>
  );
}

'use client';

import { useState } from 'react';
import { Plus, Filter, SortAsc, SortDesc } from 'lucide-react';
import { InventoryItemComponent } from './inventory-item';
import { CategoryTabs } from './category-tabs';
import { EmptyCategoryState } from './empty-category-state';
import { BulkActionsToolbar } from './bulk-actions-toolbar';
import { CategoryStatsWidget } from './category-stats-widget';
import { CategoryManager } from './category-manager';
import { DragDropProvider } from './drag-drop-provider';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { LoadingSpinner } from '@/components/ui/loading-spinner';
import { useInventoryItems } from '@/hooks/use-inventory';
import { useCategories } from '@/hooks/use-categories';
import { useDropCategory } from '@/hooks/use-drag-drop';
import type {
  InventoryItem,
  CategoryFilters,
  CategoryType,
  SortOption,
  SortDirection,
} from '@/types/inventory';
import { cn } from '@/lib/utils';

interface EnhancedInventoryListProps {
  onAddItem?: () => void;
  onEditItem?: (item: InventoryItem) => void;
  className?: string;
}

export function EnhancedInventoryList({
  onAddItem,
  onEditItem,
  className,
}: EnhancedInventoryListProps) {
  const [selectedCategory, setSelectedCategory] = useState<
    CategoryType | 'all'
  >('all');
  const [viewMode, setViewMode] = useState<'grid' | 'list'>('grid');
  const [searchQuery, setSearchQuery] = useState('');
  const [sortBy, setSortBy] = useState<SortOption>('recently_added');
  const [sortDirection, setSortDirection] = useState<SortDirection>('desc');
  const [selectedItemIds, setSelectedItemIds] = useState<string[]>([]);
  const [showCategoryManager, setShowCategoryManager] = useState(false);
  const [showAdvancedFilters, setShowAdvancedFilters] = useState(false);

  const { data: customCategories = [] } = useCategories();

  const filters: CategoryFilters = {
    ...(selectedCategory !== 'all' && { category: selectedCategory }),
    ...(searchQuery && { search: searchQuery }),
    sortBy,
    sortDirection,
  };

  const { data: items = [], isLoading, error } = useInventoryItems(filters);

  // Drag and drop for category
  const { isOver, canDrop, drop } = useDropCategory(
    selectedCategory === 'all' ? 'proteins' : selectedCategory,
    item => {
      console.log(`Item ${item.name} dropped into category`);
    }
  );

  const handleCategoryChange = (category: CategoryType | 'all') => {
    setSelectedCategory(category);
    setSelectedItemIds([]); // Clear selection when changing categories
  };

  const handleSelectionChange = (itemId: string, selected: boolean) => {
    setSelectedItemIds(prev =>
      selected ? [...prev, itemId] : prev.filter(id => id !== itemId)
    );
  };

  const handleClearSelection = () => {
    setSelectedItemIds([]);
  };

  const handleSelectAll = () => {
    if (selectedItemIds.length === items.length) {
      setSelectedItemIds([]);
    } else {
      setSelectedItemIds(items.map(item => item.id));
    }
  };

  const handleSortChange = (newSortBy: SortOption) => {
    if (newSortBy === sortBy) {
      setSortDirection(prev => (prev === 'asc' ? 'desc' : 'asc'));
    } else {
      setSortBy(newSortBy);
      setSortDirection('asc');
    }
  };

  const getSortIcon = (option: SortOption) => {
    if (sortBy !== option) return null;
    return sortDirection === 'asc' ? (
      <SortAsc className="w-4 h-4" />
    ) : (
      <SortDesc className="w-4 h-4" />
    );
  };

  const allCategories = [
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
    ...customCategories.map(c => c.id),
  ];

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
    <DragDropProvider>
      <div className={cn('space-y-6', className)}>
        {/* Header */}
        <div className="flex flex-col lg:flex-row justify-between items-start lg:items-center space-y-4 lg:space-y-0">
          <h1 className="text-2xl font-bold text-gray-900">
            Kitchen Inventory
          </h1>
          <div className="flex items-center gap-3">
            <Button
              variant="secondary"
              onClick={() => setShowCategoryManager(true)}
            >
              Manage Categories
            </Button>
            <Button onClick={onAddItem}>
              <Plus className="w-4 h-4 mr-2" />
              Add Item
            </Button>
          </div>
        </div>

        {/* Category Navigation */}
        <CategoryTabs
          selectedCategory={selectedCategory}
          onCategoryChange={handleCategoryChange}
          customCategories={customCategories}
          onManageCategories={() => setShowCategoryManager(true)}
          viewMode={viewMode}
          onViewModeChange={setViewMode}
        />

        {/* Search and Filters */}
        <div className="flex flex-col sm:flex-row gap-4">
          <div className="flex-1">
            <Input
              type="text"
              placeholder="Search items..."
              value={searchQuery}
              onChange={e => setSearchQuery(e.target.value)}
              className="w-full"
            />
          </div>

          <div className="flex items-center gap-2">
            <Button
              variant="secondary"
              onClick={() => setShowAdvancedFilters(!showAdvancedFilters)}
              className="flex items-center gap-2"
            >
              <Filter className="w-4 h-4" />
              Filters
            </Button>

            {items.length > 0 && (
              <Button
                variant="secondary"
                onClick={handleSelectAll}
                className="whitespace-nowrap"
              >
                {selectedItemIds.length === items.length
                  ? 'Deselect All'
                  : 'Select All'}
              </Button>
            )}
          </div>
        </div>

        {/* Advanced Filters */}
        {showAdvancedFilters && (
          <div className="bg-gray-50 rounded-lg p-4">
            <h3 className="font-medium text-gray-900 mb-3">Sort Options</h3>
            <div className="flex flex-wrap gap-2">
              {(
                [
                  'alphabetical',
                  'expiration',
                  'quantity',
                  'recently_added',
                ] as SortOption[]
              ).map(option => (
                <button
                  key={option}
                  onClick={() => handleSortChange(option)}
                  className={cn(
                    'flex items-center gap-2 px-3 py-2 rounded-lg text-sm font-medium transition-colors',
                    sortBy === option
                      ? 'bg-orange-500 text-white'
                      : 'bg-white border border-gray-300 text-gray-700 hover:bg-gray-50'
                  )}
                >
                  {option.charAt(0).toUpperCase() +
                    option.slice(1).replace('_', ' ')}
                  {getSortIcon(option)}
                </button>
              ))}
            </div>
          </div>
        )}

        {/* Stats Widget */}
        <CategoryStatsWidget
          selectedCategory={selectedCategory}
          customCategories={customCategories}
        />

        {/* Loading State */}
        {isLoading && (
          <div className="text-center py-8">
            <LoadingSpinner />
            <p className="mt-2 text-gray-600">Loading inventory...</p>
          </div>
        )}

        {/* Inventory Items */}
        {!isLoading && (
          <div
            ref={ref => {
              drop(ref);
            }}
            className={cn(
              'min-h-96 transition-colors',
              isOver &&
                canDrop &&
                'bg-orange-50 border-2 border-dashed border-orange-300 rounded-lg'
            )}
          >
            {items.length === 0 ? (
              <EmptyCategoryState
                category={selectedCategory}
                onAddItem={onAddItem || (() => {})}
                onQuickAdd={category => {
                  // Handle quick add for specific category
                  console.log('Quick add for category:', category);
                  onAddItem?.();
                }}
              />
            ) : (
              <div
                className={cn(
                  viewMode === 'grid'
                    ? 'grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4'
                    : 'space-y-3'
                )}
              >
                {items.map(item => (
                  <InventoryItemComponent
                    key={item.id}
                    item={item}
                    onEdit={onEditItem || (() => {})}
                    isSelected={selectedItemIds.includes(item.id)}
                    onSelectionChange={handleSelectionChange}
                    enableDrag={true}
                    availableCategories={allCategories}
                  />
                ))}
              </div>
            )}
          </div>
        )}

        {/* Bulk Actions Toolbar */}
        <BulkActionsToolbar
          selectedItemIds={selectedItemIds}
          onClearSelection={handleClearSelection}
          customCategories={customCategories}
        />

        {/* Category Manager Modal */}
        {showCategoryManager && (
          <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-50">
            <div className="bg-white rounded-lg max-w-2xl w-full max-h-96 overflow-hidden">
              <CategoryManager onClose={() => setShowCategoryManager(false)} />
            </div>
          </div>
        )}
      </div>
    </DragDropProvider>
  );
}

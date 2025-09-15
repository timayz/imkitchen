'use client';

import { useState } from 'react';
import { clsx } from 'clsx';
import {
  Beef,
  Carrot,
  Apple,
  Wheat,
  Milk,
  Coffee,
  Wine,
  Cookie,
  Snowflake,
  Plus,
  Grid3X3,
  List,
  Settings,
  Zap, // Using Zap instead of Pepper which doesn't exist
} from 'lucide-react';
import {
  InventoryCategory,
  CategoryType,
  CustomCategory,
} from '@/types/inventory';

// Category icons mapping
const categoryIcons: Record<
  InventoryCategory,
  React.ComponentType<{ className?: string }>
> = {
  proteins: Beef,
  vegetables: Carrot,
  fruits: Apple,
  grains: Wheat,
  dairy: Milk,
  spices: Zap,
  condiments: Coffee,
  beverages: Wine,
  baking: Cookie,
  frozen: Snowflake,
};

// Category colors
const categoryColors: Record<InventoryCategory, string> = {
  proteins: 'text-red-600 bg-red-50 border-red-200',
  vegetables: 'text-green-600 bg-green-50 border-green-200',
  fruits: 'text-orange-600 bg-orange-50 border-orange-200',
  grains: 'text-yellow-600 bg-yellow-50 border-yellow-200',
  dairy: 'text-blue-600 bg-blue-50 border-blue-200',
  spices: 'text-purple-600 bg-purple-50 border-purple-200',
  condiments: 'text-brown-600 bg-amber-50 border-amber-200',
  beverages: 'text-indigo-600 bg-indigo-50 border-indigo-200',
  baking: 'text-pink-600 bg-pink-50 border-pink-200',
  frozen: 'text-cyan-600 bg-cyan-50 border-cyan-200',
};

interface CategoryTabsProps {
  selectedCategory: CategoryType | 'all';
  onCategoryChange: (category: CategoryType | 'all') => void;
  customCategories?: CustomCategory[];
  onManageCategories?: () => void;
  viewMode: 'grid' | 'list';
  onViewModeChange: (mode: 'grid' | 'list') => void;
  className?: string;
}

export function CategoryTabs({
  selectedCategory,
  onCategoryChange,
  customCategories = [],
  onManageCategories,
  viewMode,
  onViewModeChange,
  className,
}: CategoryTabsProps) {
  const [showMobile, setShowMobile] = useState(false);

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

  const renderCategoryTab = (
    category: CategoryType | 'all',
    label: string,
    icon?: React.ComponentType<{ className?: string }>,
    color?: string
  ) => {
    const Icon = icon;
    const isSelected = selectedCategory === category;

    return (
      <button
        key={category}
        onClick={() => onCategoryChange(category)}
        className={clsx(
          'flex items-center gap-2 px-4 py-2 rounded-lg border text-sm font-medium transition-all',
          'hover:shadow-sm focus:outline-none focus:ring-2 focus:ring-orange-500 focus:ring-offset-1',
          isSelected
            ? color ||
                'text-orange-600 bg-orange-50 border-orange-200 shadow-sm'
            : 'text-gray-600 bg-white border-gray-200 hover:bg-gray-50',
          'whitespace-nowrap'
        )}
      >
        {Icon && <Icon className="w-4 h-4" />}
        <span>{label}</span>
      </button>
    );
  };

  return (
    <div className={clsx('space-y-4', className)}>
      {/* Desktop Categories */}
      <div className="hidden md:block">
        <div className="flex items-center justify-between mb-3">
          <h3 className="text-lg font-semibold text-gray-900">Categories</h3>
          <div className="flex items-center gap-2">
            {/* View Mode Toggle */}
            <div className="flex items-center border rounded-lg">
              <button
                onClick={() => onViewModeChange('grid')}
                className={clsx(
                  'p-2 rounded-l-lg transition-colors',
                  viewMode === 'grid'
                    ? 'bg-orange-500 text-white'
                    : 'text-gray-600 hover:bg-gray-50'
                )}
              >
                <Grid3X3 className="w-4 h-4" />
              </button>
              <button
                onClick={() => onViewModeChange('list')}
                className={clsx(
                  'p-2 rounded-r-lg transition-colors',
                  viewMode === 'list'
                    ? 'bg-orange-500 text-white'
                    : 'text-gray-600 hover:bg-gray-50'
                )}
              >
                <List className="w-4 h-4" />
              </button>
            </div>

            {onManageCategories && (
              <button
                onClick={onManageCategories}
                className="p-2 text-gray-600 hover:bg-gray-50 rounded-lg transition-colors"
                title="Manage Categories"
              >
                <Settings className="w-4 h-4" />
              </button>
            )}
          </div>
        </div>

        <div className="flex flex-wrap gap-2">
          {/* All Items Tab */}
          {renderCategoryTab('all', 'All Items')}

          {/* Predefined Categories */}
          {predefinedCategories.map(category => {
            const Icon = categoryIcons[category];
            const color = categoryColors[category];
            return renderCategoryTab(
              category,
              category.charAt(0).toUpperCase() + category.slice(1),
              Icon,
              color
            );
          })}

          {/* Custom Categories */}
          {customCategories.map(category =>
            renderCategoryTab(
              category.id,
              category.name,
              undefined,
              `text-gray-600 bg-gray-50 border-gray-200`
            )
          )}

          {/* Add Custom Category */}
          {onManageCategories && (
            <button
              onClick={onManageCategories}
              className="flex items-center gap-2 px-4 py-2 rounded-lg border border-dashed border-gray-300 text-gray-600 hover:border-gray-400 hover:bg-gray-50 transition-colors"
            >
              <Plus className="w-4 h-4" />
              <span>Add Category</span>
            </button>
          )}
        </div>
      </div>

      {/* Mobile Categories */}
      <div className="md:hidden">
        <div className="flex items-center justify-between mb-3">
          <button
            onClick={() => setShowMobile(!showMobile)}
            className="flex items-center gap-2 px-4 py-2 bg-white border border-gray-200 rounded-lg"
          >
            <span className="font-medium">
              {selectedCategory === 'all'
                ? 'All Items'
                : selectedCategory === 'proteins' ||
                    selectedCategory === 'vegetables' ||
                    selectedCategory === 'fruits' ||
                    selectedCategory === 'grains' ||
                    selectedCategory === 'dairy' ||
                    selectedCategory === 'spices' ||
                    selectedCategory === 'condiments' ||
                    selectedCategory === 'beverages' ||
                    selectedCategory === 'baking' ||
                    selectedCategory === 'frozen'
                  ? selectedCategory.charAt(0).toUpperCase() +
                    selectedCategory.slice(1)
                  : customCategories.find(c => c.id === selectedCategory)
                      ?.name || 'Category'}
            </span>
            <div
              className={clsx(
                'transition-transform',
                showMobile && 'rotate-180'
              )}
            >
              ↓
            </div>
          </button>

          <div className="flex items-center gap-2">
            <div className="flex items-center border rounded-lg">
              <button
                onClick={() => onViewModeChange('grid')}
                className={clsx(
                  'p-2 rounded-l-lg transition-colors',
                  viewMode === 'grid'
                    ? 'bg-orange-500 text-white'
                    : 'text-gray-600 hover:bg-gray-50'
                )}
              >
                <Grid3X3 className="w-4 h-4" />
              </button>
              <button
                onClick={() => onViewModeChange('list')}
                className={clsx(
                  'p-2 rounded-r-lg transition-colors',
                  viewMode === 'list'
                    ? 'bg-orange-500 text-white'
                    : 'text-gray-600 hover:bg-gray-50'
                )}
              >
                <List className="w-4 h-4" />
              </button>
            </div>
          </div>
        </div>

        {/* Collapsible Category List */}
        {showMobile && (
          <div className="bg-white border border-gray-200 rounded-lg p-4 space-y-2">
            {renderCategoryTab('all', 'All Items')}

            {predefinedCategories.map(category => {
              const Icon = categoryIcons[category];
              const color = categoryColors[category];
              return renderCategoryTab(
                category,
                category.charAt(0).toUpperCase() + category.slice(1),
                Icon,
                color
              );
            })}

            {customCategories.map(category =>
              renderCategoryTab(
                category.id,
                category.name,
                undefined,
                `text-gray-600 bg-gray-50 border-gray-200`
              )
            )}

            {onManageCategories && (
              <button
                onClick={onManageCategories}
                className="w-full flex items-center gap-2 px-4 py-2 rounded-lg border border-dashed border-gray-300 text-gray-600 hover:border-gray-400 hover:bg-gray-50 transition-colors"
              >
                <Plus className="w-4 h-4" />
                <span>Add Category</span>
              </button>
            )}
          </div>
        )}
      </div>
    </div>
  );
}

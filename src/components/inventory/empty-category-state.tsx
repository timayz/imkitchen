'use client';

import { Plus, Package, Info } from 'lucide-react';
import { InventoryCategory, CategoryType } from '@/types/inventory';

interface EmptyCategoryStateProps {
  category: CategoryType | 'all';
  onAddItem?: () => void;
  onQuickAdd?: (category: InventoryCategory) => void;
  isLoading?: boolean;
  className?: string;
}

const categoryTips: Record<InventoryCategory, string[]> = {
  proteins: [
    'Add chicken, fish, or beans for your main dishes',
    'Track expiration dates for meat and seafood',
    'Consider freezing portions for longer storage',
  ],
  vegetables: [
    'Fresh vegetables should be stored in the refrigerator',
    'Check expiration dates regularly to reduce waste',
    'Root vegetables can often be stored in the pantry',
  ],
  fruits: [
    'Some fruits ripen better at room temperature',
    'Citrus fruits last longer in the refrigerator',
    'Frozen fruits are great for smoothies',
  ],
  grains: [
    'Store grains in airtight containers',
    'Whole grains have shorter shelf life than refined',
    'Rice and pasta are pantry staples',
  ],
  dairy: [
    'Check expiration dates frequently',
    'Store in the coldest part of the refrigerator',
    'Hard cheeses last longer than soft ones',
  ],
  spices: [
    'Store in cool, dark places',
    'Ground spices lose potency after 2-3 years',
    'Whole spices last longer than ground',
  ],
  condiments: [
    'Most condiments can be stored at room temperature',
    'Refrigerate after opening for best quality',
    'Check dates on opened bottles regularly',
  ],
  beverages: [
    'Store cold beverages in the refrigerator',
    'Hot beverages can be stored in the pantry',
    'Track expiration dates on perishable drinks',
  ],
  baking: [
    'Store dry goods in airtight containers',
    'Baking soda and powder lose effectiveness over time',
    'Flour should be stored in cool, dry places',
  ],
  frozen: [
    'Label items with dates when freezing',
    'Use frozen items within recommended timeframes',
    'First in, first out rotation helps prevent waste',
  ],
};

export function EmptyCategoryState({
  category,
  onAddItem,
  onQuickAdd,
  isLoading = false,
  className = '',
}: EmptyCategoryStateProps) {
  if (isLoading) {
    return (
      <div
        className={`flex flex-col items-center justify-center py-12 text-gray-500 ${className}`}
      >
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-orange-500 mb-4"></div>
        <p>Loading items...</p>
      </div>
    );
  }

  const getCategoryInfo = () => {
    if (category === 'all') {
      return {
        title: 'No items in your inventory',
        description:
          'Start building your kitchen inventory by adding your first item.',
        tips: [
          'Add items as you shop or unpack groceries',
          'Include expiration dates to track freshness',
          'Organize by categories for easy finding',
        ],
      };
    }

    const categoryName =
      typeof category === 'string' && category in categoryTips
        ? (category as InventoryCategory)
        : null;

    if (categoryName) {
      return {
        title: `No ${categoryName} items`,
        description: `Add ${categoryName} items to start tracking your kitchen inventory.`,
        tips: categoryTips[categoryName],
      };
    }

    return {
      title: 'No items in this category',
      description: 'Add items to start organizing your kitchen inventory.',
      tips: [
        'Add items by clicking the button below',
        'Items can be moved between categories later',
      ],
    };
  };

  const { title, description, tips } = getCategoryInfo();

  return (
    <div
      className={`flex flex-col items-center justify-center py-16 px-6 text-center ${className}`}
    >
      {/* Icon */}
      <div className="mb-6">
        <div className="w-16 h-16 bg-gray-100 rounded-full flex items-center justify-center">
          <Package className="w-8 h-8 text-gray-400" />
        </div>
      </div>

      {/* Title and Description */}
      <h3 className="text-xl font-semibold text-gray-900 mb-2">{title}</h3>
      <p className="text-gray-600 mb-8 max-w-md">{description}</p>

      {/* Action Buttons */}
      <div className="flex flex-col sm:flex-row gap-3 mb-8">
        {onAddItem && (
          <button
            onClick={onAddItem}
            className="flex items-center gap-2 px-6 py-3 bg-orange-500 text-white rounded-lg hover:bg-orange-600 transition-colors font-medium"
          >
            <Plus className="w-5 h-5" />
            Add Item
          </button>
        )}

        {onQuickAdd &&
          category !== 'all' &&
          typeof category === 'string' &&
          category in categoryTips && (
            <button
              onClick={() => onQuickAdd(category as InventoryCategory)}
              className="flex items-center gap-2 px-6 py-3 bg-white border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50 transition-colors font-medium"
            >
              <Plus className="w-5 h-5" />
              Quick Add {category.charAt(0).toUpperCase() + category.slice(1)}
            </button>
          )}
      </div>

      {/* Tips */}
      {tips.length > 0 && (
        <div className="max-w-lg">
          <div className="flex items-center gap-2 text-sm font-medium text-gray-700 mb-3">
            <Info className="w-4 h-4" />
            <span>Helpful Tips</span>
          </div>
          <ul className="text-sm text-gray-600 space-y-2 text-left">
            {tips.map((tip, index) => (
              <li key={index} className="flex items-start gap-2">
                <span className="text-orange-500 mt-1">•</span>
                <span>{tip}</span>
              </li>
            ))}
          </ul>
        </div>
      )}
    </div>
  );
}

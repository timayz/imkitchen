'use client';

import { BarChart3, TrendingUp, AlertTriangle, Package } from 'lucide-react';
import { useCategoryStats } from '@/hooks/use-categories';
import {
  CategoryType,
  InventoryCategory,
  CustomCategory,
} from '@/types/inventory';

interface CategoryStatsWidgetProps {
  selectedCategory?: CategoryType | 'all';
  customCategories?: CustomCategory[];
  className?: string;
}

export function CategoryStatsWidget({
  selectedCategory = 'all',
  customCategories = [],
  className = '',
}: CategoryStatsWidgetProps) {
  const { data: stats = [], isLoading, error } = useCategoryStats();

  const formatCategoryName = (category: CategoryType) => {
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

    if (predefinedCategories.includes(category as InventoryCategory)) {
      return category.charAt(0).toUpperCase() + category.slice(1);
    }

    const customCategory = customCategories.find(c => c.id === category);
    return customCategory?.name || category;
  };

  const getFilteredStats = () => {
    if (selectedCategory === 'all') {
      return stats;
    }
    return stats.filter(stat => stat.category === selectedCategory);
  };

  const getTotalStats = () => {
    const filteredStats = getFilteredStats();
    return filteredStats.reduce(
      (totals, stat) => ({
        totalItems: totals.totalItems + stat.itemCount,
        expiringThisWeek: totals.expiringThisWeek + stat.expiringThisWeek,
        expiringToday: totals.expiringToday + stat.expiringToday,
        totalValue: totals.totalValue + (stat.totalValue || 0),
      }),
      { totalItems: 0, expiringThisWeek: 0, expiringToday: 0, totalValue: 0 }
    );
  };

  if (isLoading) {
    return (
      <div
        className={`bg-white rounded-lg border border-gray-200 p-6 ${className}`}
      >
        <div className="flex items-center justify-center py-4">
          <div className="animate-spin rounded-full h-6 w-6 border-b-2 border-orange-500"></div>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div
        className={`bg-white rounded-lg border border-red-200 p-6 ${className}`}
      >
        <div className="text-red-600 text-sm text-center">
          Failed to load statistics
        </div>
      </div>
    );
  }

  const filteredStats = getFilteredStats();
  const totals = getTotalStats();

  return (
    <div
      className={`bg-white rounded-lg border border-gray-200 p-6 ${className}`}
    >
      {/* Header */}
      <div className="flex items-center gap-2 mb-4">
        <BarChart3 className="w-5 h-5 text-orange-500" />
        <h3 className="font-semibold text-gray-900">
          {selectedCategory === 'all'
            ? 'Inventory Overview'
            : `${formatCategoryName(selectedCategory)} Stats`}
        </h3>
      </div>

      {/* Summary Cards */}
      <div className="grid grid-cols-2 lg:grid-cols-4 gap-4 mb-6">
        <div className="bg-blue-50 rounded-lg p-4">
          <div className="flex items-center gap-2 mb-1">
            <Package className="w-4 h-4 text-blue-600" />
            <span className="text-sm font-medium text-blue-900">
              Total Items
            </span>
          </div>
          <div className="text-2xl font-bold text-blue-900">
            {totals.totalItems}
          </div>
        </div>

        <div className="bg-yellow-50 rounded-lg p-4">
          <div className="flex items-center gap-2 mb-1">
            <TrendingUp className="w-4 h-4 text-yellow-600" />
            <span className="text-sm font-medium text-yellow-900">
              This Week
            </span>
          </div>
          <div className="text-2xl font-bold text-yellow-900">
            {totals.expiringThisWeek}
          </div>
          <div className="text-xs text-yellow-700">expiring</div>
        </div>

        <div className="bg-red-50 rounded-lg p-4">
          <div className="flex items-center gap-2 mb-1">
            <AlertTriangle className="w-4 h-4 text-red-600" />
            <span className="text-sm font-medium text-red-900">Today</span>
          </div>
          <div className="text-2xl font-bold text-red-900">
            {totals.expiringToday}
          </div>
          <div className="text-xs text-red-700">expiring</div>
        </div>

        <div className="bg-green-50 rounded-lg p-4">
          <div className="flex items-center gap-2 mb-1">
            <span className="text-sm font-medium text-green-900">
              Est. Value
            </span>
          </div>
          <div className="text-2xl font-bold text-green-900">
            ${totals.totalValue.toFixed(0)}
          </div>
        </div>
      </div>

      {/* Category Breakdown */}
      {selectedCategory === 'all' && filteredStats.length > 0 && (
        <div>
          <h4 className="font-medium text-gray-900 mb-3">Category Breakdown</h4>
          <div className="space-y-2 max-h-48 overflow-y-auto">
            {filteredStats
              .sort((a, b) => b.itemCount - a.itemCount)
              .map(stat => (
                <div
                  key={stat.category}
                  className="flex items-center justify-between py-2 px-3 bg-gray-50 rounded-lg"
                >
                  <div className="flex items-center gap-3">
                    <span className="font-medium text-gray-900">
                      {formatCategoryName(stat.category)}
                    </span>
                    <span className="text-sm text-gray-600">
                      {stat.itemCount} item{stat.itemCount !== 1 ? 's' : ''}
                    </span>
                  </div>
                  <div className="flex items-center gap-4 text-sm">
                    {stat.expiringThisWeek > 0 && (
                      <span className="text-yellow-600">
                        {stat.expiringThisWeek} expiring
                      </span>
                    )}
                    {stat.expiringToday > 0 && (
                      <span className="text-red-600 font-medium">
                        {stat.expiringToday} urgent
                      </span>
                    )}
                    {stat.totalValue && stat.totalValue > 0 && (
                      <span className="text-green-600">
                        ${stat.totalValue.toFixed(0)}
                      </span>
                    )}
                  </div>
                </div>
              ))}
          </div>
        </div>
      )}

      {/* Empty State */}
      {filteredStats.length === 0 && (
        <div className="text-center py-8 text-gray-500">
          <Package className="w-12 h-12 mx-auto mb-3 text-gray-300" />
          <p>No data available</p>
          <p className="text-sm">Add items to see statistics</p>
        </div>
      )}
    </div>
  );
}

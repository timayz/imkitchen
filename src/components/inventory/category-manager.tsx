'use client';

import { useState } from 'react';
import { Plus, Edit, Trash2, X, Save, Palette } from 'lucide-react';
import {
  useCategories,
  useCreateCategory,
  useUpdateCategory,
  useDeleteCategory,
} from '@/hooks/use-categories';
import {
  CustomCategory,
  CustomCategoryCreate,
  CustomCategoryUpdate,
} from '@/types/inventory';

interface CategoryManagerProps {
  onClose?: () => void;
  className?: string;
}

const DEFAULT_COLORS = [
  '#ef4444',
  '#f97316',
  '#f59e0b',
  '#eab308',
  '#84cc16',
  '#22c55e',
  '#10b981',
  '#14b8a6',
  '#06b6d4',
  '#0ea5e9',
  '#3b82f6',
  '#6366f1',
  '#8b5cf6',
  '#a855f7',
  '#d946ef',
  '#ec4899',
  '#f43f5e',
  '#64748b',
  '#374151',
  '#1f2937',
];

const DEFAULT_ICONS = [
  'utensils',
  'leaf',
  'star',
  'heart',
  'home',
  'box',
  'package',
  'gift',
  'tag',
  'bookmark',
  'flag',
  'diamond',
  'circle',
  'square',
];

export function CategoryManager({
  onClose,
  className = '',
}: CategoryManagerProps) {
  const [isCreating, setIsCreating] = useState(false);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [newCategory, setNewCategory] = useState<CustomCategoryCreate>({
    name: '',
    color: DEFAULT_COLORS[0],
    icon: DEFAULT_ICONS[0],
  });
  const [editingCategory, setEditingCategory] = useState<CustomCategoryUpdate>(
    {}
  );

  const { data: categories = [], isLoading, error } = useCategories();
  const createMutation = useCreateCategory();
  const updateMutation = useUpdateCategory();
  const deleteMutation = useDeleteCategory();

  const handleCreate = async () => {
    if (!newCategory.name.trim()) return;

    try {
      await createMutation.mutateAsync(newCategory);
      setNewCategory({
        name: '',
        color: DEFAULT_COLORS[0],
        icon: DEFAULT_ICONS[0],
      });
      setIsCreating(false);
    } catch (error) {
      console.error('Failed to create category:', error);
    }
  };

  const handleUpdate = async (id: string) => {
    if (!editingCategory.name?.trim() && editingCategory.name !== undefined)
      return;

    try {
      await updateMutation.mutateAsync({ id, data: editingCategory });
      setEditingId(null);
      setEditingCategory({});
    } catch (error) {
      console.error('Failed to update category:', error);
    }
  };

  const handleDelete = async (id: string, name: string) => {
    const confirmed = confirm(
      `Are you sure you want to delete the "${name}" category? This action cannot be undone.`
    );
    if (!confirmed) return;

    try {
      await deleteMutation.mutateAsync(id);
    } catch (error) {
      console.error('Failed to delete category:', error);
    }
  };

  const startEditing = (category: CustomCategory) => {
    setEditingId(category.id);
    setEditingCategory({
      name: category.name,
      color: category.color,
      icon: category.icon,
    });
  };

  const cancelEditing = () => {
    setEditingId(null);
    setEditingCategory({});
  };

  if (isLoading) {
    return (
      <div
        className={`bg-white rounded-lg border border-gray-200 p-6 ${className}`}
      >
        <div className="flex items-center justify-center py-8">
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
        <div className="text-red-600 text-center">
          Failed to load categories. Please try again.
        </div>
      </div>
    );
  }

  return (
    <div className={`bg-white rounded-lg border border-gray-200 ${className}`}>
      {/* Header */}
      <div className="flex items-center justify-between p-6 border-b border-gray-200">
        <h2 className="text-xl font-semibold text-gray-900">
          Manage Categories
        </h2>
        {onClose && (
          <button
            onClick={onClose}
            className="p-2 text-gray-400 hover:text-gray-600 hover:bg-gray-100 rounded-lg transition-colors"
          >
            <X className="w-5 h-5" />
          </button>
        )}
      </div>

      <div className="p-6">
        {/* Add New Category */}
        <div className="mb-6">
          {!isCreating ? (
            <button
              onClick={() => setIsCreating(true)}
              disabled={createMutation.isPending}
              className="flex items-center gap-2 px-4 py-2 bg-orange-500 text-white rounded-lg hover:bg-orange-600 transition-colors disabled:opacity-50"
            >
              <Plus className="w-4 h-4" />
              Add Custom Category
            </button>
          ) : (
            <div className="bg-gray-50 rounded-lg p-4 space-y-4">
              <h3 className="font-medium text-gray-900">Create New Category</h3>

              <div>
                <label
                  htmlFor="category-name"
                  className="block text-sm font-medium text-gray-700 mb-1"
                >
                  Category Name
                </label>
                <input
                  id="category-name"
                  type="text"
                  value={newCategory.name}
                  onChange={e =>
                    setNewCategory({ ...newCategory, name: e.target.value })
                  }
                  placeholder="Enter category name"
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-orange-500 focus:border-transparent"
                  maxLength={50}
                />
              </div>

              <div>
                <span className="block text-sm font-medium text-gray-700 mb-2">
                  Color
                </span>
                <div
                  className="flex flex-wrap gap-2"
                  role="group"
                  aria-label="Select category color"
                >
                  {DEFAULT_COLORS.map(color => (
                    <button
                      key={color}
                      onClick={() => setNewCategory({ ...newCategory, color })}
                      aria-label={`Select color ${color}`}
                      className={`w-8 h-8 rounded-full border-2 ${
                        newCategory.color === color
                          ? 'border-gray-900'
                          : 'border-gray-300'
                      }`}
                      style={{ backgroundColor: color }}
                    />
                  ))}
                </div>
              </div>

              <div>
                <span className="block text-sm font-medium text-gray-700 mb-2">
                  Icon
                </span>
                <div
                  className="flex flex-wrap gap-2"
                  role="group"
                  aria-label="Select category icon"
                >
                  {DEFAULT_ICONS.map(icon => (
                    <button
                      key={icon}
                      onClick={() => setNewCategory({ ...newCategory, icon })}
                      aria-label={`Select icon ${icon}`}
                      className={`px-3 py-2 text-sm border rounded-lg ${
                        newCategory.icon === icon
                          ? 'border-orange-500 bg-orange-50 text-orange-700'
                          : 'border-gray-300 hover:bg-gray-50'
                      }`}
                    >
                      {icon}
                    </button>
                  ))}
                </div>
              </div>

              <div className="flex gap-2">
                <button
                  onClick={handleCreate}
                  disabled={
                    !newCategory.name.trim() || createMutation.isPending
                  }
                  className="flex items-center gap-2 px-4 py-2 bg-orange-500 text-white rounded-lg hover:bg-orange-600 transition-colors disabled:opacity-50"
                >
                  <Save className="w-4 h-4" />
                  {createMutation.isPending ? 'Creating...' : 'Create'}
                </button>
                <button
                  onClick={() => {
                    setIsCreating(false);
                    setNewCategory({
                      name: '',
                      color: DEFAULT_COLORS[0],
                      icon: DEFAULT_ICONS[0],
                    });
                  }}
                  className="px-4 py-2 text-gray-600 border border-gray-300 rounded-lg hover:bg-gray-50 transition-colors"
                >
                  Cancel
                </button>
              </div>
            </div>
          )}
        </div>

        {/* Existing Categories */}
        <div className="space-y-3">
          <h3 className="font-medium text-gray-900">
            Custom Categories ({categories.length})
          </h3>

          {categories.length === 0 ? (
            <div className="text-center py-8 text-gray-500">
              <Palette className="w-12 h-12 mx-auto mb-3 text-gray-300" />
              <p>No custom categories yet.</p>
              <p className="text-sm">
                Create your first custom category to get started.
              </p>
            </div>
          ) : (
            <div className="space-y-2">
              {categories.map(category => (
                <div
                  key={category.id}
                  className="flex items-center justify-between p-3 border border-gray-200 rounded-lg hover:bg-gray-50"
                >
                  {editingId === category.id ? (
                    <div className="flex-1 flex items-center gap-3">
                      <div
                        className="w-6 h-6 rounded-full"
                        style={{
                          backgroundColor:
                            editingCategory.color || category.color,
                        }}
                      />
                      <input
                        type="text"
                        value={editingCategory.name || ''}
                        onChange={e =>
                          setEditingCategory({
                            ...editingCategory,
                            name: e.target.value,
                          })
                        }
                        className="flex-1 px-2 py-1 border border-gray-300 rounded text-sm focus:ring-2 focus:ring-orange-500"
                        maxLength={50}
                      />
                      <div className="flex gap-1">
                        <button
                          onClick={() => handleUpdate(category.id)}
                          disabled={updateMutation.isPending}
                          className="p-1 text-green-600 hover:bg-green-50 rounded transition-colors"
                        >
                          <Save className="w-4 h-4" />
                        </button>
                        <button
                          onClick={cancelEditing}
                          className="p-1 text-gray-600 hover:bg-gray-50 rounded transition-colors"
                        >
                          <X className="w-4 h-4" />
                        </button>
                      </div>
                    </div>
                  ) : (
                    <>
                      <div className="flex items-center gap-3">
                        <div
                          className="w-6 h-6 rounded-full"
                          style={{ backgroundColor: category.color }}
                        />
                        <span className="font-medium text-gray-900">
                          {category.name}
                        </span>
                        <span className="text-xs text-gray-500">
                          ({category.icon})
                        </span>
                      </div>
                      <div className="flex gap-1">
                        <button
                          onClick={() => startEditing(category)}
                          className="p-2 text-gray-600 hover:bg-gray-100 rounded transition-colors"
                        >
                          <Edit className="w-4 h-4" />
                        </button>
                        <button
                          onClick={() =>
                            handleDelete(category.id, category.name)
                          }
                          disabled={deleteMutation.isPending}
                          className="p-2 text-red-600 hover:bg-red-50 rounded transition-colors disabled:opacity-50"
                        >
                          <Trash2 className="w-4 h-4" />
                        </button>
                      </div>
                    </>
                  )}
                </div>
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

import { useState, useEffect } from 'react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Select } from '@/components/ui/select';
import {
  useCreateInventoryItem,
  useUpdateInventoryItem,
} from '@/hooks/use-inventory';
import { useAutoSave } from '@/hooks/use-local-storage';
import type {
  InventoryItem,
  InventoryItemCreate,
  InventoryItemUpdate,
  InventoryCategory,
  StorageLocation,
  MeasurementUnit,
} from '@/types/inventory';
import { cn } from '@/lib/utils';

interface InventoryFormProps {
  item?: InventoryItem;
  onSuccess?: (item: InventoryItem) => void;
  onCancel?: () => void;
  className?: string;
}

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

const locations: StorageLocation[] = ['pantry', 'refrigerator', 'freezer'];

const units: MeasurementUnit[] = [
  'pieces',
  'grams',
  'kilograms',
  'ounces',
  'pounds',
  'cups',
  'tablespoons',
  'teaspoons',
  'milliliters',
  'liters',
];

export function InventoryForm({
  item,
  onSuccess,
  onCancel,
  className,
}: InventoryFormProps) {
  const [formData, setFormData] = useState<InventoryItemCreate>({
    name: '',
    quantity: 1,
    unit: 'pieces',
    category: 'vegetables',
    location: 'pantry',
    expirationDate: null,
    purchaseDate: new Date(),
    estimatedCost: null,
  });

  const [errors, setErrors] = useState<Record<string, string>>({});
  const [saveStatus, setSaveStatus] = useState<'saving' | 'saved' | 'idle'>(
    'idle'
  );

  const createMutation = useCreateInventoryItem();
  const updateMutation = useUpdateInventoryItem();

  const isEditing = !!item;
  const isLoading = createMutation.isPending || updateMutation.isPending;

  // Auto-save draft for new items
  const storageKey = `inventory-form-draft-${item?.id || 'new'}`;

  // Auto-save form data
  useAutoSave(storageKey, formData, 1000);

  // Show save status
  useEffect(() => {
    setSaveStatus('saving');
    const timer = setTimeout(() => setSaveStatus('saved'), 1000);
    return () => clearTimeout(timer);
  }, [formData]);

  // Initialize form data when editing
  useEffect(() => {
    if (item) {
      setFormData({
        name: item.name,
        quantity: item.quantity,
        unit: item.unit,
        category: item.category,
        location: item.location,
        expirationDate: item.expirationDate
          ? new Date(item.expirationDate)
          : null,
        purchaseDate: item.purchaseDate
          ? new Date(item.purchaseDate)
          : new Date(),
        estimatedCost: item.estimatedCost ?? null,
      });
    }
  }, [item]);

  const validateForm = () => {
    const newErrors: Record<string, string> = {};

    if (!formData.name.trim()) {
      newErrors.name = 'Name is required';
    }

    if (formData.quantity <= 0) {
      newErrors.quantity = 'Quantity must be greater than 0';
    }

    if (
      formData.estimatedCost !== undefined &&
      formData.estimatedCost !== null &&
      formData.estimatedCost < 0
    ) {
      newErrors.estimatedCost = 'Cost cannot be negative';
    }

    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    if (!validateForm()) return;

    try {
      if (isEditing && item) {
        const updates: InventoryItemUpdate = {};

        // Always include required fields if they have values
        if (formData.name) updates.name = formData.name;
        if (formData.quantity) updates.quantity = formData.quantity;
        if (formData.unit) updates.unit = formData.unit;
        if (formData.category) updates.category = formData.category;
        if (formData.location) updates.location = formData.location;

        // For optional fields when editing, include them even if they're null/undefined
        // This allows clearing fields that previously had values
        updates.expirationDate = formData.expirationDate || null;
        updates.estimatedCost = formData.estimatedCost || null;

        const updatedItem = await updateMutation.mutateAsync({
          id: item.id,
          updates,
        });
        onSuccess?.(updatedItem);
      } else {
        const newItem = await createMutation.mutateAsync(formData);
        onSuccess?.(newItem);
        // Clear saved draft and reset form for creating another item
        localStorage.removeItem(storageKey);
        setFormData({
          name: '',
          quantity: 1,
          unit: 'pieces',
          category: 'vegetables',
          location: 'pantry',
          expirationDate: null,
          purchaseDate: new Date(),
          estimatedCost: null,
        });
      }
    } catch (error) {
      console.error('Form submission error:', error);
    }
  };

  const handleChange = (field: keyof InventoryItemCreate, value: unknown) => {
    setFormData(prev => ({ ...prev, [field]: value }));

    // Clear error when user starts typing
    if (errors[field]) {
      setErrors(prev => {
        const newErrors = { ...prev };
        delete newErrors[field];
        return newErrors;
      });
    }
  };

  const formatDateForInput = (date?: Date | null) => {
    if (!date) return '';
    return date.toISOString().split('T')[0];
  };

  return (
    <form onSubmit={handleSubmit} className={cn('space-y-4', className)}>
      {/* Auto-save status indicator */}
      {!isEditing && saveStatus !== 'idle' && (
        <div className="flex items-center space-x-2 text-xs text-gray-500 bg-gray-50 px-3 py-2 rounded">
          {saveStatus === 'saving' && (
            <>
              <div className="animate-spin rounded-full h-3 w-3 border-b-2 border-gray-400"></div>
              <span>Saving draft...</span>
            </>
          )}
          {saveStatus === 'saved' && (
            <>
              <div className="h-3 w-3 rounded-full bg-green-400"></div>
              <span>Draft saved</span>
            </>
          )}
        </div>
      )}
      <div>
        <label
          htmlFor="name"
          className="block text-sm font-medium text-gray-700 mb-1"
        >
          Name *
        </label>
        <Input
          id="name"
          type="text"
          value={formData.name}
          onChange={e => handleChange('name', e.target.value)}
          placeholder="e.g., Organic Tomatoes"
          className={errors.name ? 'border-red-300' : ''}
        />
        {errors.name && (
          <p className="mt-1 text-sm text-red-600">{errors.name}</p>
        )}
      </div>

      <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
        <div>
          <label
            htmlFor="quantity"
            className="block text-sm font-medium text-gray-700 mb-1"
          >
            Quantity *
          </label>
          <Input
            id="quantity"
            type="number"
            step="0.1"
            value={formData.quantity}
            onChange={e =>
              handleChange('quantity', parseFloat(e.target.value) || 0)
            }
            className={errors.quantity ? 'border-red-300' : ''}
          />
          {errors.quantity && (
            <p className="mt-1 text-sm text-red-600">{errors.quantity}</p>
          )}
        </div>

        <div>
          <label
            htmlFor="unit"
            className="block text-sm font-medium text-gray-700 mb-1"
          >
            Unit *
          </label>
          <Select
            id="unit"
            value={formData.unit}
            onChange={e => handleChange('unit', e.target.value)}
          >
            {units.map(unit => (
              <option key={unit} value={unit}>
                {unit.charAt(0).toUpperCase() + unit.slice(1)}
              </option>
            ))}
          </Select>
        </div>
      </div>

      <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
        <div>
          <label
            htmlFor="category"
            className="block text-sm font-medium text-gray-700 mb-1"
          >
            Category *
          </label>
          <Select
            id="category"
            value={formData.category}
            onChange={e => handleChange('category', e.target.value)}
          >
            {categories.map(category => (
              <option key={category} value={category}>
                {category.charAt(0).toUpperCase() + category.slice(1)}
              </option>
            ))}
          </Select>
        </div>

        <div>
          <label
            htmlFor="location"
            className="block text-sm font-medium text-gray-700 mb-1"
          >
            Location *
          </label>
          <Select
            id="location"
            value={formData.location}
            onChange={e => handleChange('location', e.target.value)}
          >
            {locations.map(location => (
              <option key={location} value={location}>
                {location.charAt(0).toUpperCase() + location.slice(1)}
              </option>
            ))}
          </Select>
        </div>
      </div>

      <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
        <div>
          <label
            htmlFor="expirationDate"
            className="block text-sm font-medium text-gray-700 mb-1"
          >
            Expiration Date
          </label>
          <Input
            id="expirationDate"
            type="date"
            value={formatDateForInput(formData.expirationDate)}
            onChange={e =>
              handleChange(
                'expirationDate',
                e.target.value ? new Date(e.target.value) : null
              )
            }
          />
        </div>

        <div>
          <label
            htmlFor="estimatedCost"
            className="block text-sm font-medium text-gray-700 mb-1"
          >
            Cost (optional)
          </label>
          <Input
            id="estimatedCost"
            type="number"
            step="0.01"
            placeholder="0.00"
            value={formData.estimatedCost || ''}
            onChange={e =>
              handleChange(
                'estimatedCost',
                e.target.value ? parseFloat(e.target.value) : null
              )
            }
            className={errors.estimatedCost ? 'border-red-300' : ''}
          />
          {errors.estimatedCost && (
            <p className="mt-1 text-sm text-red-600">{errors.estimatedCost}</p>
          )}
        </div>
      </div>

      <div className="flex space-x-3 pt-4">
        {onCancel && (
          <Button
            type="button"
            variant="secondary"
            onClick={onCancel}
            disabled={isLoading}
          >
            Cancel
          </Button>
        )}
        <Button
          type="submit"
          disabled={isLoading}
          className="flex-1 sm:flex-none"
        >
          {isLoading
            ? isEditing
              ? 'Updating...'
              : 'Adding...'
            : isEditing
              ? 'Update Item'
              : 'Add Item'}
        </Button>
      </div>
    </form>
  );
}

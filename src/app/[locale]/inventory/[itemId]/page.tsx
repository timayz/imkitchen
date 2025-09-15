'use client';

import { useParams, useRouter } from 'next/navigation';
import { useInventoryItems } from '@/hooks/use-inventory';
import { InventoryForm } from '@/components/forms/inventory-form';
import { LoadingSpinner } from '@/components/ui/loading-spinner';
import { Button } from '@/components/ui/button';
import type { InventoryItem } from '@/types/inventory';

export default function InventoryItemDetailPage() {
  const params = useParams();
  const router = useRouter();
  const itemId = params.itemId as string;

  const { data: items = [], isLoading, error } = useInventoryItems();
  const item = items.find((item: InventoryItem) => item.id === itemId);

  const handleSuccess = () => {
    router.push('/inventory');
  };

  const handleCancel = () => {
    router.back();
  };

  if (isLoading) {
    return (
      <div className="container mx-auto px-4 py-8 text-center">
        <LoadingSpinner />
        <p className="mt-2 text-gray-600">Loading item details...</p>
      </div>
    );
  }

  if (error || !item) {
    return (
      <div className="container mx-auto px-4 py-8 text-center">
        <p className="text-red-600 mb-4">
          {error ? 'Failed to load item details' : 'Item not found'}
        </p>
        <Button variant="secondary" onClick={() => router.push('/inventory')}>
          Back to Inventory
        </Button>
      </div>
    );
  }

  return (
    <div className="container mx-auto px-4 py-8 max-w-2xl">
      <div className="mb-6">
        <h1 className="text-2xl font-bold text-gray-900 mb-2">
          Edit {item.name}
        </h1>
        <p className="text-gray-600">
          Update the details of this inventory item.
        </p>
      </div>

      <div className="bg-white rounded-lg border border-gray-200 p-6">
        <InventoryForm
          item={item}
          onSuccess={handleSuccess}
          onCancel={handleCancel}
        />
      </div>
    </div>
  );
}

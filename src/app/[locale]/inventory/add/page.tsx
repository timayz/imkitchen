'use client';

import { useRouter } from 'next/navigation';
import { InventoryForm } from '@/components/forms/inventory-form';

export default function AddInventoryItemPage() {
  const router = useRouter();

  const handleSuccess = () => {
    router.push('/inventory');
  };

  const handleCancel = () => {
    router.back();
  };

  return (
    <div className="container mx-auto px-4 py-8 max-w-2xl">
      <div className="mb-6">
        <h1 className="text-2xl font-bold text-gray-900 mb-2">Add New Item</h1>
        <p className="text-gray-600">
          Add a new item to your kitchen inventory.
        </p>
      </div>

      <div className="bg-white rounded-lg border border-gray-200 p-6">
        <InventoryForm onSuccess={handleSuccess} onCancel={handleCancel} />
      </div>
    </div>
  );
}

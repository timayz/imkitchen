'use client';

import { useState, useEffect } from 'react';
import { useSession } from 'next-auth/react';
import { useRouter, useParams } from 'next/navigation';
import { InventoryList } from '@/components/inventory/inventory-list';
import { InventoryForm } from '@/components/forms/inventory-form';
import { Modal } from '@/components/ui/modal';
import type { InventoryItem } from '@/types/inventory';

export default function InventoryPage() {
  const { data: session, status } = useSession();
  const router = useRouter();
  const params = useParams();
  const [showAddModal, setShowAddModal] = useState(false);
  const [editingItem, setEditingItem] = useState<InventoryItem | null>(null);

  useEffect(() => {
    if (status === 'unauthenticated') {
      router.push(`/${params.locale}/login`);
    }
  }, [status, router, params.locale]);

  if (status === 'loading') {
    return (
      <div className="container mx-auto px-4 py-8">
        <div className="flex justify-center items-center min-h-[400px]">
          <div className="text-lg">Loading...</div>
        </div>
      </div>
    );
  }

  if (!session?.user) {
    return null;
  }

  const handleAddItem = () => {
    setShowAddModal(true);
  };

  const handleEditItem = (item: InventoryItem) => {
    setEditingItem(item);
  };

  const handleFormSuccess = () => {
    setShowAddModal(false);
    setEditingItem(null);
  };

  const handleCloseModals = () => {
    setShowAddModal(false);
    setEditingItem(null);
  };

  return (
    <div className="container mx-auto px-4 py-8">
      <InventoryList onAddItem={handleAddItem} onEditItem={handleEditItem} />

      {/* Add Item Modal */}
      <Modal
        isOpen={showAddModal}
        onClose={handleCloseModals}
        title="Add New Item"
        className="sm:max-w-md"
      >
        <InventoryForm
          onSuccess={handleFormSuccess}
          onCancel={handleCloseModals}
        />
      </Modal>

      {/* Edit Item Modal */}
      <Modal
        isOpen={!!editingItem}
        onClose={handleCloseModals}
        title="Edit Item"
        className="sm:max-w-md"
      >
        {editingItem && (
          <InventoryForm
            item={editingItem}
            onSuccess={handleFormSuccess}
            onCancel={handleCloseModals}
          />
        )}
      </Modal>
    </div>
  );
}

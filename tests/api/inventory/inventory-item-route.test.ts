/* eslint-disable @typescript-eslint/no-explicit-any */
import { NextRequest } from 'next/server';
import { PUT, DELETE } from '@/app/api/inventory/[itemId]/route';
import { prisma } from '@/lib/db';
import { auth } from '@/lib/auth';

const mockAuth = auth as jest.MockedFunction<typeof auth>;
const mockPrisma = prisma as any;

describe('/api/inventory/[itemId]', () => {
  const mockSession = {
    user: {
      id: 'user-1',
      householdId: 'household-1',
      email: 'test@example.com',
      name: 'Test User',
    },
  };

  const mockInventoryItem = {
    id: 'item-1',
    name: 'Test Item',
    quantity: 5,
    unit: 'pieces',
    category: 'VEGETABLES',
    location: 'PANTRY',
    expirationDate: new Date('2025-12-31'),
    purchaseDate: new Date('2025-01-01'),
    estimatedCost: 10.5,
    householdId: 'household-1',
    addedBy: 'user-1',
    createdAt: new Date(),
    updatedAt: new Date(),
    addedByUser: { name: 'Test User' },
  };

  beforeEach(() => {
    jest.clearAllMocks();
    // Suppress console.error for expected test failures
    jest.spyOn(console, 'error').mockImplementation(() => {});
  });

  afterEach(() => {
    jest.restoreAllMocks();
  });

  describe('PUT /api/inventory/[itemId]', () => {
    const validUpdatePayload = {
      name: 'Updated Item',
      quantity: 8,
      category: 'fruits',
      location: 'refrigerator',
    };

    it('should update inventory item for authenticated user', async () => {
      mockAuth.mockResolvedValue(mockSession as any);
      // First call for ownership check returns the item
      // Second call for duplicate check returns null (no duplicate found)
      mockPrisma.inventoryItem.findFirst
        .mockResolvedValueOnce(mockInventoryItem as any) // Ownership check
        .mockResolvedValueOnce(null); // Duplicate check - no duplicate found

      mockPrisma.inventoryItem.update.mockResolvedValue({
        ...mockInventoryItem,
        ...validUpdatePayload,
        category: 'fruits',
        location: 'refrigerator',
      } as any);

      const request = new NextRequest(
        'http://localhost:3000/api/inventory/item-1',
        {
          method: 'PUT',
          body: JSON.stringify(validUpdatePayload),
        }
      );

      const response = await PUT(request, {
        params: Promise.resolve({ itemId: 'item-1' }),
      });
      const data = await response.json();

      expect(response.status).toBe(200);
      expect(data.name).toBe('Updated Item');
      expect(data.category).toBe('fruits');
      expect(data.location).toBe('refrigerator');

      expect(mockPrisma.inventoryItem.update).toHaveBeenCalledWith({
        where: { id: 'item-1' },
        data: expect.objectContaining({
          name: 'Updated Item',
          quantity: 8,
          category: 'fruits',
          location: 'refrigerator',
          updatedAt: expect.any(Date),
        }),
        include: { addedByUser: { select: { name: true } } },
      });
    });

    it('should return 401 for unauthenticated user', async () => {
      mockAuth.mockResolvedValue(null);

      const request = new NextRequest(
        'http://localhost:3000/api/inventory/item-1',
        {
          method: 'PUT',
          body: JSON.stringify(validUpdatePayload),
        }
      );

      const response = await PUT(request, {
        params: Promise.resolve({ itemId: 'item-1' }),
      });

      expect(response.status).toBe(401);
      expect(await response.json()).toMatchObject({ error: 'Unauthorized' });
    });

    it('should return 404 for item not found or access denied', async () => {
      mockAuth.mockResolvedValue(mockSession as any);
      mockPrisma.inventoryItem.findFirst.mockResolvedValue(null);

      const request = new NextRequest(
        'http://localhost:3000/api/inventory/item-1',
        {
          method: 'PUT',
          body: JSON.stringify(validUpdatePayload),
        }
      );

      const response = await PUT(request, {
        params: Promise.resolve({ itemId: 'item-1' }),
      });

      expect(response.status).toBe(404);
      expect(await response.json()).toMatchObject({
        error: 'Item not found or access denied',
      });
    });

    it('should return 400 for duplicate item name', async () => {
      const duplicateItem = {
        ...mockInventoryItem,
        id: 'item-2',
        name: 'Updated Item',
      };

      mockAuth.mockResolvedValue(mockSession as any);
      mockPrisma.inventoryItem.findFirst
        .mockResolvedValueOnce(mockInventoryItem as any) // First call for ownership check
        .mockResolvedValueOnce(duplicateItem as any); // Second call for duplicate check

      const request = new NextRequest(
        'http://localhost:3000/api/inventory/item-1',
        {
          method: 'PUT',
          body: JSON.stringify(validUpdatePayload),
        }
      );

      const response = await PUT(request, {
        params: Promise.resolve({ itemId: 'item-1' }),
      });

      expect(response.status).toBe(400);
      expect(await response.json()).toMatchObject({
        error: 'Item with this name already exists',
      });
    });

    it('should allow updating same item with same name', async () => {
      mockAuth.mockResolvedValue(mockSession as any);
      mockPrisma.inventoryItem.findFirst
        .mockResolvedValueOnce(mockInventoryItem as any) // Ownership check
        .mockResolvedValueOnce(null); // Duplicate check (no other item with same name)

      mockPrisma.inventoryItem.update.mockResolvedValue({
        ...mockInventoryItem,
        name: 'Test Item', // Same name
        quantity: 8,
      } as any);

      const request = new NextRequest(
        'http://localhost:3000/api/inventory/item-1',
        {
          method: 'PUT',
          body: JSON.stringify({
            name: 'Test Item', // Same as current name
            quantity: 8,
          }),
        }
      );

      const response = await PUT(request, {
        params: Promise.resolve({ itemId: 'item-1' }),
      });

      expect(response.status).toBe(200);
      // Should not perform duplicate check when name hasn't changed
      expect(mockPrisma.inventoryItem.findFirst).toHaveBeenCalledTimes(1);
    });

    it('should return 400 for invalid payload', async () => {
      mockAuth.mockResolvedValue(mockSession as any);
      mockPrisma.inventoryItem.findFirst.mockResolvedValue(
        mockInventoryItem as any
      );

      const invalidPayload = {
        quantity: -1, // Invalid: negative quantity
        category: 'invalid-category', // Invalid enum value
      };

      const request = new NextRequest(
        'http://localhost:3000/api/inventory/item-1',
        {
          method: 'PUT',
          body: JSON.stringify(invalidPayload),
        }
      );

      const response = await PUT(request, {
        params: Promise.resolve({ itemId: 'item-1' }),
      });

      expect(response.status).toBe(400);
      expect(await response.json()).toMatchObject({
        error: 'Validation failed',
        details: expect.any(Array),
      });
    });

    it('should verify household ownership before update', async () => {
      mockAuth.mockResolvedValue(mockSession as any);

      const request = new NextRequest(
        'http://localhost:3000/api/inventory/item-1',
        {
          method: 'PUT',
          body: JSON.stringify(validUpdatePayload),
        }
      );

      await PUT(request, { params: Promise.resolve({ itemId: 'item-1' }) });

      expect(mockPrisma.inventoryItem.findFirst).toHaveBeenCalledWith({
        where: {
          id: 'item-1',
          householdId: 'household-1',
        },
      });
    });

    it('should handle partial updates correctly', async () => {
      mockAuth.mockResolvedValue(mockSession as any);
      // Only need ownership check for quantity-only update (no name change)
      mockPrisma.inventoryItem.findFirst.mockResolvedValue(
        mockInventoryItem as any
      );
      mockPrisma.inventoryItem.update.mockResolvedValue({
        ...mockInventoryItem,
        quantity: 10,
      } as any);

      const partialUpdate = { quantity: 10 };

      const request = new NextRequest(
        'http://localhost:3000/api/inventory/item-1',
        {
          method: 'PUT',
          body: JSON.stringify(partialUpdate),
        }
      );

      const response = await PUT(request, {
        params: Promise.resolve({ itemId: 'item-1' }),
      });

      expect(response.status).toBe(200);
      expect(mockPrisma.inventoryItem.update).toHaveBeenCalledWith({
        where: { id: 'item-1' },
        data: expect.objectContaining({
          quantity: 10,
          updatedAt: expect.any(Date),
        }),
        include: { addedByUser: { select: { name: true } } },
      });
    });
  });

  describe('DELETE /api/inventory/[itemId]', () => {
    it('should delete inventory item for authenticated user', async () => {
      mockAuth.mockResolvedValue(mockSession as any);
      mockPrisma.inventoryItem.findFirst.mockResolvedValue(
        mockInventoryItem as any
      );
      mockPrisma.inventoryItem.delete.mockResolvedValue(
        mockInventoryItem as any
      );

      const request = new NextRequest(
        'http://localhost:3000/api/inventory/item-1',
        {
          method: 'DELETE',
        }
      );

      const response = await DELETE(request, {
        params: Promise.resolve({ itemId: 'item-1' }),
      });

      expect(response.status).toBe(204);
      expect(await response.json()).toMatchObject({
        message: 'Item deleted successfully',
      });

      expect(mockPrisma.inventoryItem.delete).toHaveBeenCalledWith({
        where: { id: 'item-1' },
      });
    });

    it('should return 401 for unauthenticated user', async () => {
      mockAuth.mockResolvedValue(null);

      const request = new NextRequest(
        'http://localhost:3000/api/inventory/item-1',
        {
          method: 'DELETE',
        }
      );

      const response = await DELETE(request, {
        params: Promise.resolve({ itemId: 'item-1' }),
      });

      expect(response.status).toBe(401);
      expect(await response.json()).toMatchObject({ error: 'Unauthorized' });
    });

    it('should return 404 for item not found or access denied', async () => {
      mockAuth.mockResolvedValue(mockSession as any);
      mockPrisma.inventoryItem.findFirst.mockResolvedValue(null);

      const request = new NextRequest(
        'http://localhost:3000/api/inventory/item-1',
        {
          method: 'DELETE',
        }
      );

      const response = await DELETE(request, {
        params: Promise.resolve({ itemId: 'item-1' }),
      });

      expect(response.status).toBe(404);
      expect(await response.json()).toMatchObject({
        error: 'Item not found or access denied',
      });
    });

    it('should verify household ownership before deletion', async () => {
      mockAuth.mockResolvedValue(mockSession as any);
      mockPrisma.inventoryItem.findFirst.mockResolvedValue(
        mockInventoryItem as any
      );
      mockPrisma.inventoryItem.delete.mockResolvedValue(
        mockInventoryItem as any
      );

      const request = new NextRequest(
        'http://localhost:3000/api/inventory/item-1',
        {
          method: 'DELETE',
        }
      );

      await DELETE(request, { params: Promise.resolve({ itemId: 'item-1' }) });

      expect(mockPrisma.inventoryItem.findFirst).toHaveBeenCalledWith({
        where: {
          id: 'item-1',
          householdId: 'household-1',
        },
      });
    });

    it('should prevent deletion of items from different households', async () => {
      const otherHouseholdSession = {
        ...mockSession,
        user: { ...mockSession.user, householdId: 'household-2' },
      };

      mockAuth.mockResolvedValue(otherHouseholdSession as any);
      mockPrisma.inventoryItem.findFirst.mockResolvedValue(null); // Not found in user's household

      const request = new NextRequest(
        'http://localhost:3000/api/inventory/item-1',
        {
          method: 'DELETE',
        }
      );

      const response = await DELETE(request, {
        params: Promise.resolve({ itemId: 'item-1' }),
      });

      expect(response.status).toBe(404);
      expect(mockPrisma.inventoryItem.delete).not.toHaveBeenCalled();
    });
  });
});

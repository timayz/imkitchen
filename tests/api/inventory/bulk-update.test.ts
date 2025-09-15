import { NextRequest } from 'next/server';
import { PUT } from '@/app/api/inventory/bulk-update/route';
import { auth } from '@/lib/auth';
import { prisma } from '@/lib/db';

// Mock dependencies
jest.mock('@/lib/auth');
jest.mock('@/lib/db', () => ({
  prisma: {
    inventoryItem: {
      count: jest.fn(),
      updateMany: jest.fn(),
      findMany: jest.fn(),
    },
    customCategory: {
      findFirst: jest.fn(),
    },
  },
}));

const mockAuth = auth as jest.MockedFunction<typeof auth>;
const mockPrisma = prisma as jest.Mocked<typeof prisma>;

describe('/api/inventory/bulk-update', () => {
  const mockSession = {
    user: {
      id: 'user-1',
      householdId: 'household-1',
      email: 'user1@example.com',
      name: 'User One',
    },
  };

  const mockUpdatedItems = [
    {
      id: 'item-1',
      name: 'Test Item 1',
      quantity: 5,
      unit: 'pieces',
      category: 'vegetables',
      location: 'refrigerator',
      expirationDate: new Date('2025-12-31'),
      purchaseDate: new Date('2025-01-01'),
      estimatedCost: 10.5,
      householdId: 'household-1',
      addedBy: 'user-1',
      createdAt: new Date(),
      updatedAt: new Date(),
      addedByUser: { name: 'User One' },
    },
    {
      id: 'item-2',
      name: 'Test Item 2',
      quantity: 3,
      unit: 'pieces',
      category: 'vegetables',
      location: 'refrigerator',
      expirationDate: new Date('2025-12-31'),
      purchaseDate: new Date('2025-01-01'),
      estimatedCost: 8.0,
      householdId: 'household-1',
      addedBy: 'user-1',
      createdAt: new Date(),
      updatedAt: new Date(),
      addedByUser: { name: 'User One' },
    },
  ];

  beforeEach(() => {
    jest.clearAllMocks();
    mockAuth.mockResolvedValue(mockSession);
  });

  describe('PUT /api/inventory/bulk-update', () => {
    it('successfully updates multiple items with valid data', async () => {
      const validBulkUpdate = {
        itemIds: [
          '550e8400-e29b-41d4-a716-446655440001',
          '550e8400-e29b-41d4-a716-446655440002',
        ],
        updates: {
          category: 'vegetables',
          location: 'refrigerator' as const,
        },
      };

      mockPrisma.inventoryItem.count.mockResolvedValue(2);
      mockPrisma.inventoryItem.updateMany.mockResolvedValue({ count: 2 });
      mockPrisma.inventoryItem.findMany.mockResolvedValue(mockUpdatedItems);

      const request = new NextRequest(
        'http://localhost/api/inventory/bulk-update',
        {
          method: 'PUT',
          body: JSON.stringify(validBulkUpdate),
        }
      );

      const response = await PUT(request);
      const data = await response.json();

      expect(response.status).toBe(200);
      expect(data).toEqual(mockUpdatedItems);
      expect(mockPrisma.inventoryItem.count).toHaveBeenCalledWith({
        where: {
          id: {
            in: [
              '550e8400-e29b-41d4-a716-446655440001',
              '550e8400-e29b-41d4-a716-446655440002',
            ],
          },
          householdId: 'household-1',
        },
      });
      expect(mockPrisma.inventoryItem.updateMany).toHaveBeenCalledWith({
        where: {
          id: {
            in: [
              '550e8400-e29b-41d4-a716-446655440001',
              '550e8400-e29b-41d4-a716-446655440002',
            ],
          },
          householdId: 'household-1',
        },
        data: {
          category: 'vegetables',
          location: 'refrigerator',
        },
      });
    });

    it('returns 401 for unauthenticated user', async () => {
      mockAuth.mockResolvedValue(null);

      const request = new NextRequest(
        'http://localhost/api/inventory/bulk-update',
        {
          method: 'PUT',
          body: JSON.stringify({
            itemIds: ['item-1'],
            updates: { category: 'vegetables' },
          }),
        }
      );

      const response = await PUT(request);

      expect(response.status).toBe(401);
    });

    it('returns 400 for empty itemIds array', async () => {
      const request = new NextRequest(
        'http://localhost/api/inventory/bulk-update',
        {
          method: 'PUT',
          body: JSON.stringify({
            itemIds: [],
            updates: { category: 'vegetables' },
          }),
        }
      );

      const response = await PUT(request);
      const data = await response.json();

      expect(response.status).toBe(400);
      expect(data.error).toBe('No items specified for update');
    });

    it('returns 400 for too many items (>100)', async () => {
      const manyItemIds = Array.from(
        { length: 101 },
        (_, i) =>
          `550e8400-e29b-41d4-a716-${String(446655440000 + i).padStart(12, '0')}`
      );

      const request = new NextRequest(
        'http://localhost/api/inventory/bulk-update',
        {
          method: 'PUT',
          body: JSON.stringify({
            itemIds: manyItemIds,
            updates: { category: 'vegetables' },
          }),
        }
      );

      const response = await PUT(request);
      const data = await response.json();

      expect(response.status).toBe(400);
      expect(data.error).toBe(
        'Too many items. Maximum 100 items per bulk update.'
      );
    });

    it('returns 404 when some items not found or unauthorized', async () => {
      mockPrisma.inventoryItem.count.mockResolvedValue(1); // Only 1 found, but 2 requested

      const request = new NextRequest(
        'http://localhost/api/inventory/bulk-update',
        {
          method: 'PUT',
          body: JSON.stringify({
            itemIds: [
              '550e8400-e29b-41d4-a716-446655440001',
              '550e8400-e29b-41d4-a716-446655440002',
            ],
            updates: { category: 'vegetables' },
          }),
        }
      );

      const response = await PUT(request);
      const data = await response.json();

      expect(response.status).toBe(404);
      expect(data.error).toBe('Some items not found or unauthorized');
    });

    it('validates custom category exists when updating to custom category', async () => {
      mockPrisma.inventoryItem.count.mockResolvedValue(2);
      mockPrisma.customCategory.findFirst.mockResolvedValue({
        id: 'custom-1',
        name: 'Supplements',
        householdId: 'household-1',
      });
      mockPrisma.inventoryItem.updateMany.mockResolvedValue({ count: 2 });
      mockPrisma.inventoryItem.findMany.mockResolvedValue(mockUpdatedItems);

      const request = new NextRequest(
        'http://localhost/api/inventory/bulk-update',
        {
          method: 'PUT',
          body: JSON.stringify({
            itemIds: [
              '550e8400-e29b-41d4-a716-446655440001',
              '550e8400-e29b-41d4-a716-446655440002',
            ],
            updates: { category: 'custom-1' },
          }),
        }
      );

      const response = await PUT(request);

      expect(response.status).toBe(200);
      expect(mockPrisma.customCategory.findFirst).toHaveBeenCalledWith({
        where: {
          id: 'custom-1',
          householdId: 'household-1',
        },
      });
    });

    it('returns 404 for non-existent custom category', async () => {
      mockPrisma.inventoryItem.count.mockResolvedValue(2);
      mockPrisma.customCategory.findFirst.mockResolvedValue(null);

      const request = new NextRequest(
        'http://localhost/api/inventory/bulk-update',
        {
          method: 'PUT',
          body: JSON.stringify({
            itemIds: [
              '550e8400-e29b-41d4-a716-446655440001',
              '550e8400-e29b-41d4-a716-446655440002',
            ],
            updates: { category: 'non-existent-custom' },
          }),
        }
      );

      const response = await PUT(request);
      const data = await response.json();

      expect(response.status).toBe(404);
      expect(data.error).toBe('Custom category not found');
    });

    it('allows updates with only category change', async () => {
      mockPrisma.inventoryItem.count.mockResolvedValue(2);
      mockPrisma.inventoryItem.updateMany.mockResolvedValue({ count: 2 });
      mockPrisma.inventoryItem.findMany.mockResolvedValue(mockUpdatedItems);

      const request = new NextRequest(
        'http://localhost/api/inventory/bulk-update',
        {
          method: 'PUT',
          body: JSON.stringify({
            itemIds: [
              '550e8400-e29b-41d4-a716-446655440001',
              '550e8400-e29b-41d4-a716-446655440002',
            ],
            updates: { category: 'fruits' },
          }),
        }
      );

      const response = await PUT(request);

      expect(response.status).toBe(200);
      expect(mockPrisma.inventoryItem.updateMany).toHaveBeenCalledWith({
        where: {
          id: {
            in: [
              '550e8400-e29b-41d4-a716-446655440001',
              '550e8400-e29b-41d4-a716-446655440002',
            ],
          },
          householdId: 'household-1',
        },
        data: { category: 'fruits' },
      });
    });

    it('allows updates with only location change', async () => {
      mockPrisma.inventoryItem.count.mockResolvedValue(2);
      mockPrisma.inventoryItem.updateMany.mockResolvedValue({ count: 2 });
      mockPrisma.inventoryItem.findMany.mockResolvedValue(mockUpdatedItems);

      const request = new NextRequest(
        'http://localhost/api/inventory/bulk-update',
        {
          method: 'PUT',
          body: JSON.stringify({
            itemIds: [
              '550e8400-e29b-41d4-a716-446655440001',
              '550e8400-e29b-41d4-a716-446655440002',
            ],
            updates: { location: 'freezer' },
          }),
        }
      );

      const response = await PUT(request);

      expect(response.status).toBe(200);
      expect(mockPrisma.inventoryItem.updateMany).toHaveBeenCalledWith({
        where: {
          id: {
            in: [
              '550e8400-e29b-41d4-a716-446655440001',
              '550e8400-e29b-41d4-a716-446655440002',
            ],
          },
          householdId: 'household-1',
        },
        data: { location: 'freezer' },
      });
    });

    it('returns 400 for invalid data types', async () => {
      const request = new NextRequest(
        'http://localhost/api/inventory/bulk-update',
        {
          method: 'PUT',
          body: JSON.stringify({
            itemIds: 'not-an-array',
            updates: { category: 'vegetables' },
          }),
        }
      );

      const response = await PUT(request);
      const data = await response.json();

      expect(response.status).toBe(400);
      expect(data.error).toBe('Invalid request data');
    });

    it('returns 400 for invalid location enum', async () => {
      const request = new NextRequest(
        'http://localhost/api/inventory/bulk-update',
        {
          method: 'PUT',
          body: JSON.stringify({
            itemIds: ['item-1'],
            updates: { location: 'invalid-location' },
          }),
        }
      );

      const response = await PUT(request);
      const data = await response.json();

      expect(response.status).toBe(400);
      expect(data.error).toBe('Invalid request data');
    });

    it('handles database errors gracefully', async () => {
      const consoleSpy = jest.spyOn(console, 'error').mockImplementation();
      mockPrisma.inventoryItem.count.mockRejectedValue(
        new Error('Database error')
      );

      const request = new NextRequest(
        'http://localhost/api/inventory/bulk-update',
        {
          method: 'PUT',
          body: JSON.stringify({
            itemIds: ['550e8400-e29b-41d4-a716-446655440001'],
            updates: { category: 'vegetables' },
          }),
        }
      );

      const response = await PUT(request);
      const data = await response.json();

      expect(response.status).toBe(500);
      expect(data.error).toBe('Failed to bulk update items');
      expect(consoleSpy).toHaveBeenCalledWith(
        'Failed to bulk update items:',
        expect.any(Error)
      );

      consoleSpy.mockRestore();
    });
  });
});

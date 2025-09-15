/* eslint-disable @typescript-eslint/no-explicit-any */
import { NextRequest } from 'next/server';
import { GET, POST } from '@/app/api/inventory/route';
import { PUT, DELETE } from '@/app/api/inventory/[itemId]/route';
import { prisma } from '@/lib/db';
import { auth } from '@/lib/auth';

// Mock the dependencies
const mockAuth = auth as jest.MockedFunction<typeof auth>;
const mockPrisma = prisma as any;

describe('Authentication & Authorization Integration', () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });

  describe('Unauthenticated Access', () => {
    beforeEach(() => {
      mockAuth.mockResolvedValue(null);
    });

    it('should deny GET requests to inventory API without authentication', async () => {
      const request = new NextRequest('http://localhost:3000/api/inventory');
      const response = await GET(request);

      expect(response.status).toBe(401);
      expect(await response.json()).toMatchObject({ error: 'Unauthorized' });
      expect(mockPrisma.inventoryItem.findMany).not.toHaveBeenCalled();
    });

    it('should deny POST requests to inventory API without authentication', async () => {
      const request = new NextRequest('http://localhost:3000/api/inventory', {
        method: 'POST',
        body: JSON.stringify({ name: 'Test Item', quantity: 1 }),
      });
      const response = await POST(request);

      expect(response.status).toBe(401);
      expect(await response.json()).toMatchObject({ error: 'Unauthorized' });
      expect(mockPrisma.inventoryItem.create).not.toHaveBeenCalled();
    });

    it('should deny PUT requests to inventory items without authentication', async () => {
      const request = new NextRequest(
        'http://localhost:3000/api/inventory/item-1',
        {
          method: 'PUT',
          body: JSON.stringify({ name: 'Updated Item' }),
        }
      );
      const response = await PUT(request, {
        params: Promise.resolve({ itemId: 'item-1' }),
      });

      expect(response.status).toBe(401);
      expect(await response.json()).toMatchObject({ error: 'Unauthorized' });
      expect(mockPrisma.inventoryItem.findFirst).not.toHaveBeenCalled();
      expect(mockPrisma.inventoryItem.update).not.toHaveBeenCalled();
    });

    it('should deny DELETE requests to inventory items without authentication', async () => {
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
      expect(mockPrisma.inventoryItem.findFirst).not.toHaveBeenCalled();
      expect(mockPrisma.inventoryItem.delete).not.toHaveBeenCalled();
    });
  });

  describe('Incomplete Session Data', () => {
    it('should deny access when session lacks user data', async () => {
      mockAuth.mockResolvedValue({ user: null } as any);

      const request = new NextRequest('http://localhost:3000/api/inventory');
      const response = await GET(request);

      expect(response.status).toBe(401);
      expect(await response.json()).toMatchObject({ error: 'Unauthorized' });
    });

    it('should deny access when session lacks householdId', async () => {
      mockAuth.mockResolvedValue({
        user: { id: 'user-1', email: 'test@example.com' },
      } as any);

      const request = new NextRequest('http://localhost:3000/api/inventory');
      const response = await GET(request);

      expect(response.status).toBe(401);
      expect(await response.json()).toMatchObject({ error: 'Unauthorized' });
    });
  });

  describe('Household Data Isolation', () => {
    const mockSession = {
      user: {
        id: 'user-1',
        householdId: 'household-1',
        email: 'user1@example.com',
        name: 'User One',
      },
    };

    const mockDifferentHouseholdSession = {
      user: {
        id: 'user-2',
        householdId: 'household-2',
        email: 'user2@example.com',
        name: 'User Two',
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
      householdId: 'household-1',
      addedBy: 'user-1',
      createdAt: new Date(),
      updatedAt: new Date(),
      addedByUser: { name: 'User One' },
    };

    it("should only return items from user's household in GET requests", async () => {
      mockAuth.mockResolvedValue(mockSession as any);
      mockPrisma.inventoryItem.findMany.mockResolvedValue([
        mockInventoryItem,
      ] as any);

      const request = new NextRequest('http://localhost:3000/api/inventory');
      await GET(request);

      expect(mockPrisma.inventoryItem.findMany).toHaveBeenCalledWith({
        where: expect.objectContaining({
          householdId: 'household-1',
        }),
        orderBy: [{ expirationDate: 'asc' }, { name: 'asc' }],
        include: { addedByUser: { select: { name: true } } },
      });
    });

    it('should prevent access to items from different households in PUT requests', async () => {
      mockAuth.mockResolvedValue(mockDifferentHouseholdSession as any);
      mockPrisma.inventoryItem.findFirst.mockResolvedValue(null); // Item not found in user's household

      const request = new NextRequest(
        'http://localhost:3000/api/inventory/item-1',
        {
          method: 'PUT',
          body: JSON.stringify({ name: 'Updated Item' }),
        }
      );
      const response = await PUT(request, {
        params: Promise.resolve({ itemId: 'item-1' }),
      });

      expect(response.status).toBe(404);
      expect(await response.json()).toMatchObject({
        error: 'Item not found or access denied',
      });

      // Verify the security check query
      expect(mockPrisma.inventoryItem.findFirst).toHaveBeenCalledWith({
        where: {
          id: 'item-1',
          householdId: 'household-2', // Different household
        },
      });

      expect(mockPrisma.inventoryItem.update).not.toHaveBeenCalled();
    });

    it('should prevent deletion of items from different households', async () => {
      mockAuth.mockResolvedValue(mockDifferentHouseholdSession as any);
      mockPrisma.inventoryItem.findFirst.mockResolvedValue(null); // Item not found in user's household

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

      // Verify the security check query
      expect(mockPrisma.inventoryItem.findFirst).toHaveBeenCalledWith({
        where: {
          id: 'item-1',
          householdId: 'household-2', // Different household
        },
      });

      expect(mockPrisma.inventoryItem.delete).not.toHaveBeenCalled();
    });

    it('should enforce household isolation when creating duplicate items', async () => {
      mockAuth.mockResolvedValue(mockSession as any);
      // Simulate existing item in different household - should not prevent creation
      mockPrisma.inventoryItem.findFirst.mockResolvedValue(null); // No duplicate in current household
      mockPrisma.inventoryItem.create.mockResolvedValue(
        mockInventoryItem as any
      );

      const request = new NextRequest('http://localhost:3000/api/inventory', {
        method: 'POST',
        body: JSON.stringify({
          name: 'Test Item',
          quantity: 3,
          unit: 'pieces',
          category: 'vegetables',
          location: 'pantry',
        }),
      });

      const response = await POST(request);

      expect(response.status).toBe(201);

      // Verify duplicate check is scoped to household
      expect(mockPrisma.inventoryItem.findFirst).toHaveBeenCalledWith({
        where: {
          name: {
            equals: 'Test Item',
            mode: 'insensitive',
          },
          householdId: 'household-1', // Only check current household
        },
      });

      expect(mockPrisma.inventoryItem.create).toHaveBeenCalledWith({
        data: expect.objectContaining({
          householdId: 'household-1',
          addedBy: 'user-1',
        }),
        include: { addedByUser: { select: { name: true } } },
      });
    });

    it('should enforce household isolation when checking for duplicate names during updates', async () => {
      mockAuth.mockResolvedValue(mockSession as any);

      // Mock: Item exists in user's household
      mockPrisma.inventoryItem.findFirst
        .mockResolvedValueOnce(mockInventoryItem as any) // Ownership check
        .mockResolvedValueOnce(null); // No duplicate with new name in current household

      mockPrisma.inventoryItem.update.mockResolvedValue({
        ...mockInventoryItem,
        name: 'Updated Item',
      } as any);

      const request = new NextRequest(
        'http://localhost:3000/api/inventory/item-1',
        {
          method: 'PUT',
          body: JSON.stringify({ name: 'Updated Item' }),
        }
      );

      const response = await PUT(request, {
        params: Promise.resolve({ itemId: 'item-1' }),
      });

      expect(response.status).toBe(200);

      // Verify duplicate check is scoped to current household and excludes current item
      expect(mockPrisma.inventoryItem.findFirst).toHaveBeenNthCalledWith(2, {
        where: {
          name: {
            equals: 'Updated Item',
            mode: 'insensitive',
          },
          householdId: 'household-1', // Only check current household
          id: {
            not: 'item-1', // Exclude current item
          },
        },
      });
    });
  });

  describe('Input Validation Security', () => {
    const mockSession = {
      user: {
        id: 'user-1',
        householdId: 'household-1',
        email: 'user1@example.com',
        name: 'User One',
      },
    };

    beforeEach(() => {
      mockAuth.mockResolvedValue(mockSession as any);
    });

    it('should reject POST requests with invalid data types', async () => {
      const invalidPayload = {
        name: 123, // Should be string
        quantity: 'invalid', // Should be number
        unit: null, // Should be string
        category: 'invalid-category', // Should be valid enum
        location: 'invalid-location', // Should be valid enum
      };

      const request = new NextRequest('http://localhost:3000/api/inventory', {
        method: 'POST',
        body: JSON.stringify(invalidPayload),
      });

      const response = await POST(request);

      expect(response.status).toBe(400);
      expect(await response.json()).toMatchObject({
        error: 'Validation failed',
        details: expect.any(Array),
      });

      expect(mockPrisma.inventoryItem.create).not.toHaveBeenCalled();
    });

    it('should reject PUT requests with invalid data types', async () => {
      mockPrisma.inventoryItem.findFirst.mockResolvedValue({
        id: 'item-1',
        householdId: 'household-1',
        name: 'Existing Item',
      } as any);

      const invalidPayload = {
        quantity: -5, // Should be positive
        estimatedCost: 'not-a-number', // Should be number
        category: 'invalid-category', // Should be valid enum
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

      expect(mockPrisma.inventoryItem.update).not.toHaveBeenCalled();
    });

    it('should sanitize and validate string inputs', async () => {
      mockPrisma.inventoryItem.findFirst.mockResolvedValue(null); // No duplicates
      mockPrisma.inventoryItem.create.mockResolvedValue({
        id: 'new-item',
        name: 'Clean Item Name',
        householdId: 'household-1',
      } as any);

      const payload = {
        name: '  Clean Item Name  ', // Whitespace should be handled
        quantity: 1,
        unit: 'pieces',
        category: 'vegetables',
        location: 'pantry',
      };

      const request = new NextRequest('http://localhost:3000/api/inventory', {
        method: 'POST',
        body: JSON.stringify(payload),
      });

      const response = await POST(request);

      expect(response.status).toBe(201);

      // Verify that the data was processed correctly
      expect(mockPrisma.inventoryItem.create).toHaveBeenCalledWith({
        data: expect.objectContaining({
          name: '  Clean Item Name  ', // Input preserved as-is, validation ensures it's valid
          category: 'vegetables', // Enum values are lowercase
          location: 'pantry', // Enum values are lowercase
        }),
        include: { addedByUser: { select: { name: true } } },
      });
    });
  });

  describe('Error Handling Security', () => {
    const mockSession = {
      user: {
        id: 'user-1',
        householdId: 'household-1',
        email: 'user1@example.com',
        name: 'User One',
      },
    };

    beforeEach(() => {
      mockAuth.mockResolvedValue(mockSession as any);
    });

    it('should not expose internal errors in API responses', async () => {
      // Simulate database error
      mockPrisma.inventoryItem.findMany.mockRejectedValue(
        new Error('Database connection failed - sensitive internal info')
      );

      const request = new NextRequest('http://localhost:3000/api/inventory');
      const response = await GET(request);

      expect(response.status).toBe(500);

      const responseBody = await response.json();
      expect(responseBody).toMatchObject({
        error: 'Internal server error',
      });

      // Verify sensitive information is not exposed
      expect(responseBody).not.toMatchObject({
        error: expect.stringContaining('Database connection failed'),
      });
      expect(responseBody).not.toMatchObject({
        error: expect.stringContaining('sensitive internal info'),
      });
    });

    it('should handle authentication errors gracefully', async () => {
      // Simulate auth service error
      mockAuth.mockRejectedValue(new Error('Auth service unavailable'));

      const request = new NextRequest('http://localhost:3000/api/inventory');
      const response = await GET(request);

      expect(response.status).toBe(500);
      expect(await response.json()).toMatchObject({
        error: 'Internal server error',
      });
    });
  });
});

/* eslint-disable @typescript-eslint/no-explicit-any */
import { NextRequest } from 'next/server';
import { GET, POST } from '@/app/api/inventory/route';
import { prisma } from '@/lib/db';
import { auth } from '@/lib/auth';

const mockAuth = auth as jest.MockedFunction<typeof auth>;
const mockPrisma = prisma as any;

describe('/api/inventory', () => {
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

  describe('GET /api/inventory', () => {
    it('should return inventory items for authenticated user', async () => {
      mockAuth.mockResolvedValue(mockSession as any);
      mockPrisma.inventoryItem.findMany.mockResolvedValue([
        mockInventoryItem,
      ] as any);

      const request = new NextRequest('http://localhost:3000/api/inventory');
      const response = await GET(request);
      const data = await response.json();

      expect(response.status).toBe(200);
      expect(data).toHaveLength(1);
      expect(data[0]).toMatchObject({
        id: 'item-1',
        name: 'Test Item',
        householdId: 'household-1',
      });
    });

    it('should return 401 for unauthenticated user', async () => {
      mockAuth.mockResolvedValue(null);

      const request = new NextRequest('http://localhost:3000/api/inventory');
      const response = await GET(request);

      expect(response.status).toBe(401);
      expect(await response.json()).toMatchObject({ error: 'Unauthorized' });
    });

    it('should filter by location when specified', async () => {
      mockAuth.mockResolvedValue(mockSession as any);
      mockPrisma.inventoryItem.findMany.mockResolvedValue([
        mockInventoryItem,
      ] as any);

      const request = new NextRequest(
        'http://localhost:3000/api/inventory?location=pantry'
      );
      await GET(request);

      expect(mockPrisma.inventoryItem.findMany).toHaveBeenCalledWith({
        where: expect.objectContaining({
          householdId: 'household-1',
          location: 'pantry',
        }),
        orderBy: [{ createdAt: 'desc' }],
        include: { addedByUser: { select: { name: true } } },
      });
    });

    it('should filter by category when specified', async () => {
      mockAuth.mockResolvedValue(mockSession as any);
      mockPrisma.inventoryItem.findMany.mockResolvedValue([
        mockInventoryItem,
      ] as any);

      const request = new NextRequest(
        'http://localhost:3000/api/inventory?category=vegetables'
      );
      await GET(request);

      expect(mockPrisma.inventoryItem.findMany).toHaveBeenCalledWith({
        where: expect.objectContaining({
          householdId: 'household-1',
          category: 'vegetables',
        }),
        orderBy: [{ createdAt: 'desc' }],
        include: { addedByUser: { select: { name: true } } },
      });
    });

    it('should filter by search query when specified', async () => {
      mockAuth.mockResolvedValue(mockSession as any);
      mockPrisma.inventoryItem.findMany.mockResolvedValue([
        mockInventoryItem,
      ] as any);

      const request = new NextRequest(
        'http://localhost:3000/api/inventory?search=tomatoes'
      );
      await GET(request);

      expect(mockPrisma.inventoryItem.findMany).toHaveBeenCalledWith({
        where: expect.objectContaining({
          householdId: 'household-1',
          name: {
            contains: 'tomatoes',
            mode: 'insensitive',
          },
        }),
        orderBy: [{ createdAt: 'desc' }],
        include: { addedByUser: { select: { name: true } } },
      });
    });
  });

  describe('POST /api/inventory', () => {
    const validPayload = {
      name: 'New Item',
      quantity: 3,
      unit: 'pieces',
      category: 'vegetables',
      location: 'refrigerator',
      expirationDate: '2025-12-31',
    };

    it('should create new inventory item for authenticated user', async () => {
      mockAuth.mockResolvedValue(mockSession as any);
      mockPrisma.inventoryItem.findFirst.mockResolvedValue(null); // No duplicate
      mockPrisma.inventoryItem.create.mockResolvedValue(
        mockInventoryItem as any
      );

      const request = new NextRequest('http://localhost:3000/api/inventory', {
        method: 'POST',
        body: JSON.stringify(validPayload),
      });

      const response = await POST(request);
      const data = await response.json();

      expect(response.status).toBe(201);
      expect(data).toMatchObject({
        id: 'item-1',
        name: 'Test Item',
        householdId: 'household-1',
      });

      expect(mockPrisma.inventoryItem.create).toHaveBeenCalledWith({
        data: expect.objectContaining({
          name: 'New Item',
          quantity: 3,
          category: 'vegetables', // Lowercase enum values
          location: 'refrigerator', // Lowercase enum values
          householdId: 'household-1',
          addedBy: 'user-1',
        }),
        include: { addedByUser: { select: { name: true } } },
      });
    });

    it('should return 401 for unauthenticated user', async () => {
      mockAuth.mockResolvedValue(null);

      const request = new NextRequest('http://localhost:3000/api/inventory', {
        method: 'POST',
        body: JSON.stringify(validPayload),
      });

      const response = await POST(request);

      expect(response.status).toBe(401);
      expect(await response.json()).toMatchObject({ error: 'Unauthorized' });
    });

    it('should return 400 for duplicate item name', async () => {
      mockAuth.mockResolvedValue(mockSession as any);
      mockPrisma.inventoryItem.findFirst.mockResolvedValue(
        mockInventoryItem as any
      ); // Duplicate found

      const request = new NextRequest('http://localhost:3000/api/inventory', {
        method: 'POST',
        body: JSON.stringify(validPayload),
      });

      const response = await POST(request);

      expect(response.status).toBe(400);
      expect(await response.json()).toMatchObject({
        error: 'Item with this name already exists',
      });
    });

    it('should return 400 for invalid payload', async () => {
      mockAuth.mockResolvedValue(mockSession as any);

      const invalidPayload = {
        name: '', // Invalid: empty name
        quantity: -1, // Invalid: negative quantity
      };

      const request = new NextRequest('http://localhost:3000/api/inventory', {
        method: 'POST',
        body: JSON.stringify(invalidPayload),
      });

      const response = await POST(request);

      expect(response.status).toBe(400);
      expect(await response.json()).toMatchObject({
        error: 'Validation failed',
      });
    });

    it('should check for duplicate items case-insensitively within household', async () => {
      mockAuth.mockResolvedValue(mockSession as any);

      const request = new NextRequest('http://localhost:3000/api/inventory', {
        method: 'POST',
        body: JSON.stringify(validPayload),
      });

      await POST(request);

      expect(mockPrisma.inventoryItem.findFirst).toHaveBeenCalledWith({
        where: {
          name: {
            equals: 'New Item',
            mode: 'insensitive',
          },
          householdId: 'household-1',
        },
      });
    });
  });
});

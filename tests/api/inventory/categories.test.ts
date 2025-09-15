import { NextRequest } from 'next/server';
import { GET, POST } from '@/app/api/inventory/categories/route';
import { auth } from '@/lib/auth';
import { prisma } from '@/lib/db';

// Mock dependencies
jest.mock('@/lib/auth');
jest.mock('@/lib/db', () => ({
  prisma: {
    customCategory: {
      findMany: jest.fn(),
      findFirst: jest.fn(),
      create: jest.fn(),
    },
  },
}));

const mockAuth = auth as jest.MockedFunction<typeof auth>;
const mockPrisma = prisma.customCategory as jest.Mocked<
  typeof prisma.customCategory
>;

describe('/api/inventory/categories', () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });

  describe('GET', () => {
    it('returns categories for authenticated user', async () => {
      mockAuth.mockResolvedValue({
        user: { householdId: 'household-1' },
      });

      const mockCategories = [
        {
          id: 'cat-1',
          name: 'Supplements',
          color: '#8b5cf6',
          icon: 'pill',
          householdId: 'household-1',
          createdBy: 'user-1',
          createdAt: new Date(),
          updatedAt: new Date(),
        },
      ];

      mockPrisma.findMany.mockResolvedValue(mockCategories);

      const request = new NextRequest(
        'http://localhost/api/inventory/categories'
      );
      const response = await GET(request);
      const data = await response.json();

      expect(response.status).toBe(200);
      expect(data).toEqual(mockCategories);
      expect(mockPrisma.findMany).toHaveBeenCalledWith({
        where: { householdId: 'household-1' },
        orderBy: { name: 'asc' },
      });
    });

    it('returns 401 for unauthenticated user', async () => {
      mockAuth.mockResolvedValue(null);

      const request = new NextRequest(
        'http://localhost/api/inventory/categories'
      );
      const response = await GET(request);

      expect(response.status).toBe(401);
    });
  });

  describe('POST', () => {
    const validCategoryData = {
      name: 'Supplements',
      color: '#8b5cf6',
      icon: 'pill',
    };

    it('creates new category for authenticated user', async () => {
      mockAuth.mockResolvedValue({
        user: { householdId: 'household-1', id: 'user-1' },
      });

      mockPrisma.findFirst.mockResolvedValue(null); // No existing category
      mockPrisma.create.mockResolvedValue({
        id: 'cat-1',
        ...validCategoryData,
        householdId: 'household-1',
        createdBy: 'user-1',
        createdAt: new Date(),
        updatedAt: new Date(),
      });

      const request = new NextRequest(
        'http://localhost/api/inventory/categories',
        {
          method: 'POST',
          body: JSON.stringify(validCategoryData),
        }
      );

      const response = await POST(request);
      const data = await response.json();

      expect(response.status).toBe(201);
      expect(data.name).toBe(validCategoryData.name);
      expect(mockPrisma.create).toHaveBeenCalledWith({
        data: {
          ...validCategoryData,
          householdId: 'household-1',
          createdBy: 'user-1',
        },
      });
    });

    it('returns 400 for duplicate category name', async () => {
      mockAuth.mockResolvedValue({
        user: { householdId: 'household-1', id: 'user-1' },
      });

      mockPrisma.findFirst.mockResolvedValue({ id: 'existing-cat' }); // Existing category

      const request = new NextRequest(
        'http://localhost/api/inventory/categories',
        {
          method: 'POST',
          body: JSON.stringify(validCategoryData),
        }
      );

      const response = await POST(request);
      const data = await response.json();

      expect(response.status).toBe(400);
      expect(data.error).toBe('Category name already exists');
    });

    it('returns 400 for invalid data', async () => {
      mockAuth.mockResolvedValue({
        user: { householdId: 'household-1', id: 'user-1' },
      });

      const invalidData = {
        name: '', // Invalid: empty name
        color: 'invalid-color', // Invalid: not hex color
        icon: 'pill',
      };

      const request = new NextRequest(
        'http://localhost/api/inventory/categories',
        {
          method: 'POST',
          body: JSON.stringify(invalidData),
        }
      );

      const response = await POST(request);

      expect(response.status).toBe(400);
    });
  });
});

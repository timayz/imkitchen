import { AuthService } from '@/lib/services/auth-service';
import { AuthError } from '@/types/auth';

// Mock Prisma
jest.mock('@/lib/db', () => ({
  prisma: {
    user: {
      findUnique: jest.fn(),
      create: jest.fn(),
      update: jest.fn(),
    },
    household: {
      create: jest.fn(),
    },
    $transaction: jest.fn(),
  },
}));

// Mock bcrypt
jest.mock('bcrypt', () => ({
  hash: jest.fn(),
  compare: jest.fn(),
}));

import { prisma } from '@/lib/db';
import bcrypt from 'bcrypt';

// eslint-disable-next-line @typescript-eslint/no-explicit-any
const mockPrisma = prisma as any;
const mockBcrypt = bcrypt as jest.Mocked<typeof bcrypt>;

describe('AuthService', () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });

  describe('hashPassword', () => {
    it('should hash password with correct salt rounds', async () => {
      const password = 'testPassword123';
      const hashedPassword = 'hashedPassword';

      mockBcrypt.hash.mockResolvedValue(hashedPassword as never);

      const result = await AuthService.hashPassword(password);

      expect(bcrypt.hash).toHaveBeenCalledWith(password, 12);
      expect(result).toBe(hashedPassword);
    });
  });

  describe('comparePassword', () => {
    it('should compare password correctly', async () => {
      const password = 'testPassword123';
      const hash = 'hashedPassword';

      mockBcrypt.compare.mockResolvedValue(true as never);

      const result = await AuthService.comparePassword(password, hash);

      expect(bcrypt.compare).toHaveBeenCalledWith(password, hash);
      expect(result).toBe(true);
    });
  });

  describe('registerUser', () => {
    it('should register a new user successfully', async () => {
      const registerData = {
        email: 'test@example.com',
        name: 'Test User',
        password: 'Password123',
        householdName: 'Test Household',
        dietaryPreferences: [],
        allergies: [],
        language: 'EN' as const,
        timezone: 'UTC',
      };

      const mockHousehold = {
        id: 'household-id',
        name: 'Test Household',
      };

      const mockUser = {
        id: 'user-id',
        email: 'test@example.com',
        name: 'Test User',
        householdId: 'household-id',
        language: 'EN',
        timezone: 'UTC',
      };

      mockPrisma.user.findUnique.mockResolvedValue(null);
      mockBcrypt.hash.mockResolvedValue('hashedPassword' as never);
      mockPrisma.$transaction.mockResolvedValue({
        user: mockUser,
        household: mockHousehold,
      });

      const result = await AuthService.registerUser(registerData);

      expect(mockPrisma.user.findUnique).toHaveBeenCalledWith({
        where: { email: registerData.email },
      });
      expect(result).toEqual({
        id: 'user-id',
        email: 'test@example.com',
        name: 'Test User',
        householdId: 'household-id',
        language: 'EN',
        timezone: 'UTC',
      });
    });

    it('should throw error if user already exists', async () => {
      const registerData = {
        email: 'existing@example.com',
        name: 'Test User',
        password: 'Password123',
        householdName: 'Test Household',
        dietaryPreferences: [],
        allergies: [],
        language: 'EN' as const,
        timezone: 'UTC',
      };

      const existingUser = {
        id: 'existing-user-id',
        email: 'existing@example.com',
      };

      mockPrisma.user.findUnique.mockResolvedValue(
        existingUser as typeof existingUser & { passwordHash?: string }
      );

      await expect(AuthService.registerUser(registerData)).rejects.toThrow(
        AuthError.EMAIL_ALREADY_EXISTS
      );
    });
  });

  describe('verifyCredentials', () => {
    it('should verify valid credentials', async () => {
      const email = 'test@example.com';
      const password = 'Password123';

      const mockUser = {
        id: 'user-id',
        email: 'test@example.com',
        name: 'Test User',
        passwordHash: 'hashedPassword',
        householdId: 'household-id',
        language: 'EN',
        timezone: 'UTC',
      };

      mockPrisma.user.findUnique.mockResolvedValue(
        mockUser as typeof mockUser & { passwordHash: string }
      );
      mockBcrypt.compare.mockResolvedValue(true as never);

      const result = await AuthService.verifyCredentials(email, password);

      expect(mockPrisma.user.findUnique).toHaveBeenCalledWith({
        where: { email },
      });
      expect(bcrypt.compare).toHaveBeenCalledWith(password, 'hashedPassword');
      expect(result).toEqual({
        id: 'user-id',
        email: 'test@example.com',
        name: 'Test User',
        householdId: 'household-id',
        language: 'EN',
        timezone: 'UTC',
      });
    });

    it('should return null for invalid credentials', async () => {
      const email = 'test@example.com';
      const password = 'wrongPassword';

      const mockUser = {
        id: 'user-id',
        email: 'test@example.com',
        passwordHash: 'hashedPassword',
      };

      mockPrisma.user.findUnique.mockResolvedValue(
        mockUser as typeof mockUser & { passwordHash: string }
      );
      mockBcrypt.compare.mockResolvedValue(false as never);

      const result = await AuthService.verifyCredentials(email, password);

      expect(result).toBeNull();
    });

    it('should return null for non-existent user', async () => {
      const email = 'nonexistent@example.com';
      const password = 'Password123';

      mockPrisma.user.findUnique.mockResolvedValue(null);

      const result = await AuthService.verifyCredentials(email, password);

      expect(result).toBeNull();
    });
  });
});

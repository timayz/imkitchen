import { UserRepository } from '../../src/lib/repositories/user-repository';
import { db } from '../../src/lib/db';
import { DietaryPreference, Language } from '@prisma/client';

// Mock Prisma
jest.mock('../../src/lib/db', () => ({
  db: {
    user: {
      findUnique: jest.fn(),
      findMany: jest.fn(),
      create: jest.fn(),
      update: jest.fn(),
      delete: jest.fn(),
      count: jest.fn(),
    },
    household: {
      findUnique: jest.fn(),
    },
    session: {
      findMany: jest.fn(),
    },
    $transaction: jest.fn(),
  },
}));

describe('UserRepository', () => {
  let userRepository: UserRepository;
  const mockDb = db as jest.Mocked<typeof db>;

  beforeEach(() => {
    userRepository = new UserRepository();
    jest.clearAllMocks();
  });

  describe('findByEmail', () => {
    it('should find user by email', async () => {
      const mockUser = {
        id: 'user-1',
        email: 'test@example.com',
        name: 'Test User',
        passwordHash: 'hash',
        dietaryPreferences: [],
        allergies: [],
        householdId: 'household-1',
        language: Language.EN,
        timezone: 'UTC',
        createdAt: new Date(),
        updatedAt: new Date(),
      };

      mockDb.user.findUnique.mockResolvedValue(mockUser);

      const result = await userRepository.findByEmail('test@example.com');

      expect(result).toEqual(mockUser);
      expect(mockDb.user.findUnique).toHaveBeenCalledWith({
        where: { email: 'test@example.com' },
      });
    });

    it('should return null when user not found', async () => {
      mockDb.user.findUnique.mockResolvedValue(null);

      const result = await userRepository.findByEmail('nonexistent@example.com');

      expect(result).toBeNull();
    });
  });

  describe('create', () => {
    it('should create a new user', async () => {
      const createData = {
        email: 'new@example.com',
        name: 'New User',
        passwordHash: 'hash',
        householdId: 'household-1',
      };

      const mockHousehold = { id: 'household-1', name: 'Test Household' };
      const mockUser = { ...createData, id: 'user-1', createdAt: new Date(), updatedAt: new Date() };

      mockDb.household.findUnique.mockResolvedValue(mockHousehold);
      mockDb.user.findUnique.mockResolvedValue(null); // No existing user
      mockDb.user.create.mockResolvedValue(mockUser);

      const result = await userRepository.create(createData);

      expect(result).toEqual(mockUser);
    });

    it('should throw error if household not found', async () => {
      const createData = {
        email: 'new@example.com',
        name: 'New User',
        passwordHash: 'hash',
        householdId: 'invalid-household',
      };

      mockDb.household.findUnique.mockResolvedValue(null);

      await expect(userRepository.create(createData)).rejects.toThrow('Household not found');
    });

    it('should throw error if email already exists', async () => {
      const createData = {
        email: 'existing@example.com',
        name: 'New User',
        passwordHash: 'hash',
        householdId: 'household-1',
      };

      const mockHousehold = { id: 'household-1', name: 'Test Household' };
      const existingUser = { id: 'user-1', email: 'existing@example.com' };

      mockDb.household.findUnique.mockResolvedValue(mockHousehold);
      mockDb.user.findUnique.mockResolvedValue(existingUser);

      await expect(userRepository.create(createData)).rejects.toThrow('Email already registered');
    });
  });

  describe('findByHousehold', () => {
    it('should find all users in a household', async () => {
      const mockUsers = [
        { id: 'user-1', name: 'User 1', householdId: 'household-1' },
        { id: 'user-2', name: 'User 2', householdId: 'household-1' },
      ];

      mockDb.user.findMany.mockResolvedValue(mockUsers);

      const result = await userRepository.findByHousehold('household-1');

      expect(result).toEqual(mockUsers);
      expect(mockDb.user.findMany).toHaveBeenCalledWith({
        where: { householdId: 'household-1' },
        orderBy: { createdAt: 'asc' },
      });
    });
  });

  describe('updatePassword', () => {
    it('should update user password', async () => {
      const mockUser = {
        id: 'user-1',
        passwordHash: 'new-hash',
      };

      mockDb.user.update.mockResolvedValue(mockUser);

      const result = await userRepository.updatePassword('user-1', 'new-hash');

      expect(result).toEqual(mockUser);
      expect(mockDb.user.update).toHaveBeenCalledWith({
        where: { id: 'user-1' },
        data: { passwordHash: 'new-hash' },
      });
    });
  });
});
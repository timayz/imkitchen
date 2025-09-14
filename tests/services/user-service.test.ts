import { UserService } from '../../src/lib/services/user-service';
import { userRepository } from '../../src/lib/repositories/user-repository';
import { householdRepository } from '../../src/lib/repositories/household-repository';
import { db } from '../../src/lib/db';
import bcrypt from 'bcryptjs';
import { DietaryPreference, Language } from '@prisma/client';

// Mock dependencies
jest.mock('../../src/lib/repositories/user-repository');
jest.mock('../../src/lib/repositories/household-repository');
jest.mock('../../src/lib/db');
jest.mock('bcryptjs');

describe('UserService', () => {
  let userService: UserService;
  const mockUserRepository = userRepository as jest.Mocked<typeof userRepository>;
  const mockHouseholdRepository = householdRepository as jest.Mocked<typeof householdRepository>;
  const mockDb = db as jest.Mocked<typeof db>;
  const mockBcrypt = bcrypt as jest.Mocked<typeof bcrypt>;

  beforeEach(() => {
    userService = new UserService();
    jest.clearAllMocks();
  });

  describe('createUserWithHousehold', () => {
    it('should create user with new household', async () => {
      const userData = {
        email: 'test@example.com',
        name: 'Test User',
        password: 'password123',
        householdName: 'Test Household',
      };

      const mockHashedPassword = 'hashed-password';
      const mockHousehold = { id: 'household-1', name: 'Test Household' };
      const mockUser = { id: 'user-1', email: 'test@example.com', householdId: 'household-1' };

      mockBcrypt.hash.mockResolvedValue(mockHashedPassword);
      mockDb.$transaction.mockResolvedValue({ user: mockUser, household: mockHousehold });

      const result = await userService.createUserWithHousehold(userData);

      expect(result).toEqual({ user: mockUser, household: mockHousehold });
      expect(mockBcrypt.hash).toHaveBeenCalledWith('password123', 12);
    });
  });

  describe('authenticateUser', () => {
    it('should authenticate user with valid credentials', async () => {
      const email = 'test@example.com';
      const password = 'password123';
      const mockUser = {
        id: 'user-1',
        email,
        passwordHash: 'hashed-password',
        name: 'Test User',
        householdId: 'household-1',
      };

      mockUserRepository.findByEmail.mockResolvedValue(mockUser);
      mockBcrypt.compare.mockResolvedValue(true);

      const result = await userService.authenticateUser(email, password);

      expect(result).toEqual(mockUser);
      expect(mockUserRepository.findByEmail).toHaveBeenCalledWith(email);
      expect(mockBcrypt.compare).toHaveBeenCalledWith(password, 'hashed-password');
    });

    it('should return null for invalid email', async () => {
      mockUserRepository.findByEmail.mockResolvedValue(null);

      const result = await userService.authenticateUser('invalid@example.com', 'password');

      expect(result).toBeNull();
    });

    it('should return null for invalid password', async () => {
      const mockUser = {
        id: 'user-1',
        email: 'test@example.com',
        passwordHash: 'hashed-password',
      };

      mockUserRepository.findByEmail.mockResolvedValue(mockUser);
      mockBcrypt.compare.mockResolvedValue(false);

      const result = await userService.authenticateUser('test@example.com', 'wrong-password');

      expect(result).toBeNull();
    });
  });

  describe('updatePassword', () => {
    it('should update password with valid current password', async () => {
      const userId = 'user-1';
      const currentPassword = 'old-password';
      const newPassword = 'new-password';
      const mockUser = {
        id: userId,
        passwordHash: 'old-hashed-password',
      };

      mockUserRepository.findById.mockResolvedValue(mockUser);
      mockBcrypt.compare.mockResolvedValue(true);
      mockBcrypt.hash.mockResolvedValue('new-hashed-password');
      mockUserRepository.updatePassword.mockResolvedValue(mockUser);

      await userService.updatePassword(userId, currentPassword, newPassword);

      expect(mockUserRepository.updatePassword).toHaveBeenCalledWith(userId, 'new-hashed-password');
    });

    it('should throw error for invalid current password', async () => {
      const userId = 'user-1';
      const mockUser = {
        id: userId,
        passwordHash: 'hashed-password',
      };

      mockUserRepository.findById.mockResolvedValue(mockUser);
      mockBcrypt.compare.mockResolvedValue(false);

      await expect(
        userService.updatePassword(userId, 'wrong-password', 'new-password')
      ).rejects.toThrow('Current password is incorrect');
    });

    it('should throw error if user not found', async () => {
      mockUserRepository.findById.mockResolvedValue(null);

      await expect(
        userService.updatePassword('invalid-user', 'old-password', 'new-password')
      ).rejects.toThrow('User not found');
    });
  });

  describe('getUserProfile', () => {
    it('should return complete user profile', async () => {
      const userId = 'user-1';
      const mockUserWithHousehold = {
        id: userId,
        email: 'test@example.com',
        name: 'Test User',
        dietaryPreferences: [DietaryPreference.VEGETARIAN],
        allergies: ['nuts'],
        language: Language.EN,
        timezone: 'UTC',
        household: {
          id: 'household-1',
          name: 'Test Household',
          settings: {},
        },
      };

      const mockStats = {
        joinedAt: new Date(),
        householdMemberCount: 2,
        activeSessions: 1,
      };

      const mockActivity = {
        recentSessions: 5,
        lastLoginAt: new Date(),
      };

      mockUserRepository.findWithHousehold.mockResolvedValue(mockUserWithHousehold);
      mockUserRepository.getUserStats.mockResolvedValue(mockStats);
      mockUserRepository.getUserActivity.mockResolvedValue(mockActivity);

      const result = await userService.getUserProfile(userId);

      expect(result).toMatchObject({
        id: userId,
        email: 'test@example.com',
        name: 'Test User',
        household: {
          id: 'household-1',
          name: 'Test Household',
          memberCount: 2,
        },
      });
    });

    it('should throw error if user not found', async () => {
      mockUserRepository.findWithHousehold.mockResolvedValue(null);

      await expect(userService.getUserProfile('invalid-user')).rejects.toThrow('User not found');
    });
  });
});
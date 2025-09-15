import { User, DietaryPreference, Language } from '@prisma/client';
import { db } from '../db';
import { BaseRepository } from './base-repository';
import { logDatabaseOperation } from '../logger';

// User repository types
export type UserWithHousehold = User & {
  household: {
    id: string;
    name: string;
    settings: Record<string, unknown>;
  };
};

export type CreateUserData = {
  email: string;
  name: string;
  passwordHash: string;
  dietaryPreferences?: DietaryPreference[];
  allergies?: string[];
  householdId: string;
  language?: Language;
  timezone?: string;
};

export type UpdateUserData = Partial<
  Omit<CreateUserData, 'email' | 'householdId'>
>;

export interface IUserRepository {
  findByEmail(email: string): Promise<User | null>;
  findByHousehold(householdId: string): Promise<User[]>;
  findWithHousehold(id: string): Promise<UserWithHousehold | null>;
  updatePassword(id: string, passwordHash: string): Promise<User>;
  updatePreferences(
    id: string,
    preferences: {
      dietaryPreferences?: DietaryPreference[];
      allergies?: string[];
      language?: Language;
      timezone?: string;
    }
  ): Promise<User>;
  searchUsers(query: string, householdId?: string): Promise<User[]>;
  getUserStats(id: string): Promise<{
    joinedAt: Date;
    householdMemberCount: number;
    activeSessions: number;
  }>;
}

// User repository implementation
export class UserRepository
  extends BaseRepository<User, CreateUserData, UpdateUserData>
  implements IUserRepository
{
  constructor() {
    super(db, 'User');
  }

  protected getModel() {
    return this.db.user;
  }

  // Find user by email
  async findByEmail(email: string): Promise<User | null> {
    return logDatabaseOperation(
      'findByEmail',
      this.modelName,
      async () => {
        return this.getModel().findUnique({
          where: { email },
        });
      },
      { email }
    );
  }

  // Find all users in a household
  async findByHousehold(householdId: string): Promise<User[]> {
    return logDatabaseOperation(
      'findByHousehold',
      this.modelName,
      async () => {
        return this.getModel().findMany({
          where: { householdId },
          orderBy: { createdAt: 'asc' },
        });
      },
      { householdId }
    );
  }

  // Find user with household information
  async findWithHousehold(id: string): Promise<UserWithHousehold | null> {
    return logDatabaseOperation(
      'findWithHousehold',
      this.modelName,
      async () => {
        return this.getModel().findUnique({
          where: { id },
          include: {
            household: {
              select: {
                id: true,
                name: true,
                settings: true,
              },
            },
          },
        });
      },
      { id }
    );
  }

  // Update user password
  async updatePassword(id: string, passwordHash: string): Promise<User> {
    return logDatabaseOperation(
      'updatePassword',
      this.modelName,
      async () => {
        return this.getModel().update({
          where: { id },
          data: { passwordHash },
        });
      },
      { id }
    );
  }

  // Update user preferences
  async updatePreferences(
    id: string,
    preferences: {
      dietaryPreferences?: DietaryPreference[];
      allergies?: string[];
      language?: Language;
      timezone?: string;
    }
  ): Promise<User> {
    return logDatabaseOperation(
      'updatePreferences',
      this.modelName,
      async () => {
        return this.getModel().update({
          where: { id },
          data: preferences,
        });
      },
      { id, preferences }
    );
  }

  // Search users by name or email
  async searchUsers(query: string, householdId?: string): Promise<User[]> {
    return logDatabaseOperation(
      'searchUsers',
      this.modelName,
      async () => {
        const whereClause = {
          OR: [
            { name: { contains: query, mode: 'insensitive' } },
            { email: { contains: query, mode: 'insensitive' } },
          ],
        };

        if (householdId) {
          whereClause.householdId = householdId;
        }

        return this.getModel().findMany({
          where: whereClause,
          select: {
            id: true,
            email: true,
            name: true,
            language: true,
            createdAt: true,
            household: {
              select: {
                id: true,
                name: true,
              },
            },
          },
          orderBy: { name: 'asc' },
          take: 20, // Limit search results
        });
      },
      { query, householdId }
    );
  }

  // Get user statistics
  async getUserStats(id: string): Promise<{
    joinedAt: Date;
    householdMemberCount: number;
    activeSessions: number;
  }> {
    return logDatabaseOperation(
      'getUserStats',
      this.modelName,
      async () => {
        const user = await this.getModel().findUnique({
          where: { id },
          include: {
            household: {
              include: {
                _count: {
                  select: { users: true },
                },
              },
            },
            sessions: {
              where: {
                expiresAt: { gt: new Date() },
              },
            },
          },
        });

        if (!user) {
          throw new Error('User not found');
        }

        return {
          joinedAt: user.createdAt,
          householdMemberCount: user.household._count.users,
          activeSessions: user.sessions.length,
        };
      },
      { id }
    );
  }

  // Create user with validation
  async create(data: CreateUserData): Promise<User> {
    return logDatabaseOperation('create', this.modelName, async () => {
      // Validate household exists
      const household = await this.db.household.findUnique({
        where: { id: data.householdId },
      });

      if (!household) {
        throw new Error('Household not found');
      }

      // Check if email is already taken
      const existingUser = await this.findByEmail(data.email);
      if (existingUser) {
        throw new Error('Email already registered');
      }

      return this.getModel().create({
        data: {
          ...data,
          dietaryPreferences: data.dietaryPreferences || [],
          allergies: data.allergies || [],
          language: data.language || Language.EN,
          timezone: data.timezone || 'UTC',
        },
      });
    });
  }

  // Update user with validation
  async update(id: string, data: UpdateUserData): Promise<User> {
    return logDatabaseOperation(
      'update',
      this.modelName,
      async () => {
        // Ensure user exists
        const user = await this.findById(id);
        if (!user) {
          throw new Error('User not found');
        }

        return this.getModel().update({
          where: { id },
          data,
        });
      },
      { id }
    );
  }

  // Find users with dietary restrictions
  async findUsersWithDietaryRestrictions(
    householdId: string,
    restrictions: DietaryPreference[]
  ): Promise<Partial<User>[]> {
    return logDatabaseOperation(
      'findUsersWithDietaryRestrictions',
      this.modelName,
      async () => {
        return this.getModel().findMany({
          where: {
            householdId,
            dietaryPreferences: {
              hasSome: restrictions,
            },
          },
          select: {
            id: true,
            name: true,
            email: true,
            dietaryPreferences: true,
            allergies: true,
          },
        }) as Promise<Partial<User>[]>;
      },
      { householdId, restrictions }
    );
  }

  // Find users with allergies
  async findUsersWithAllergies(
    householdId: string,
    allergens: string[]
  ): Promise<Partial<User>[]> {
    return logDatabaseOperation(
      'findUsersWithAllergies',
      this.modelName,
      async () => {
        return this.getModel().findMany({
          where: {
            householdId,
            allergies: {
              hasSome: allergens,
            },
          },
          select: {
            id: true,
            name: true,
            email: true,
            allergies: true,
          },
        }) as Promise<Partial<User>[]>;
      },
      { householdId, allergens }
    );
  }

  // Bulk update user languages
  async updateLanguageForHousehold(
    householdId: string,
    language: Language
  ): Promise<{ count: number }> {
    return logDatabaseOperation(
      'updateLanguageForHousehold',
      this.modelName,
      async () => {
        return this.getModel().updateMany({
          where: { householdId },
          data: { language },
        });
      },
      { householdId, language }
    );
  }

  // Get user activity summary
  async getUserActivity(
    id: string,
    days: number = 30
  ): Promise<{
    recentSessions: number;
    lastLoginAt: Date | null;
  }> {
    return logDatabaseOperation(
      'getUserActivity',
      this.modelName,
      async () => {
        const cutoffDate = new Date();
        cutoffDate.setDate(cutoffDate.getDate() - days);

        const sessions = await this.db.session.findMany({
          where: {
            userId: id,
            createdAt: { gte: cutoffDate },
          },
          orderBy: { createdAt: 'desc' },
        });

        return {
          recentSessions: sessions.length,
          lastLoginAt: sessions.length > 0 ? sessions[0].createdAt : null,
        };
      },
      { id, days }
    );
  }
}

// Export singleton instance
export const userRepository = new UserRepository();

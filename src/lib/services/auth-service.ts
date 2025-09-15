import bcrypt from 'bcrypt';
import { prisma } from '@/lib/db';
import {
  RegisterData,
  ChangePasswordData,
} from '@/lib/validators/auth-schemas';
import { AuthUser, UserProfile } from '@/types/auth';
import { AuthError } from '@/types/auth';

export class AuthService {
  private static readonly SALT_ROUNDS = 12;

  /**
   * Hash a password using bcrypt
   */
  static async hashPassword(password: string): Promise<string> {
    return bcrypt.hash(password, this.SALT_ROUNDS);
  }

  /**
   * Compare a password with its hash
   */
  static async comparePassword(
    password: string,
    hash: string
  ): Promise<boolean> {
    return bcrypt.compare(password, hash);
  }

  /**
   * Register a new user and create their household
   */
  static async registerUser(data: RegisterData): Promise<AuthUser> {
    try {
      // Check if user already exists
      const existingUser = await prisma.user.findUnique({
        where: { email: data.email },
      });

      if (existingUser) {
        throw new Error(AuthError.EMAIL_ALREADY_EXISTS);
      }

      // Hash password
      const passwordHash = await this.hashPassword(data.password);

      // Create user and household in a transaction
      const result = await prisma.$transaction(async tx => {
        // Create household first
        const household = await tx.household.create({
          data: {
            name: data.householdName,
            settings: {
              defaultMeasurementUnit: 'metric',
              sharedInventory: true,
              mealPlanningAccess: 'all-members',
              notificationPreferences: {
                expirationAlerts: true,
                mealPlanReminders: true,
                shoppingListUpdates: true,
              },
            },
          },
        });

        // Create user
        const user = await tx.user.create({
          data: {
            email: data.email,
            name: data.name,
            passwordHash,
            householdId: household.id,
            dietaryPreferences: data.dietaryPreferences || [],
            allergies: data.allergies || [],
            language: data.language || 'en',
            timezone: data.timezone || 'UTC',
          },
        });

        return { user, household };
      });

      return {
        id: result.user.id,
        email: result.user.email,
        name: result.user.name,
        householdId: result.user.householdId,
        language: result.user.language,
        timezone: result.user.timezone,
      };
    } catch (error) {
      if (error instanceof Error) {
        throw error;
      }
      throw new Error('Registration failed');
    }
  }

  /**
   * Get user profile with household information
   */
  static async getUserProfile(userId: string): Promise<UserProfile | null> {
    try {
      const user = await prisma.user.findUnique({
        where: { id: userId },
        include: {
          household: {
            include: {
              _count: {
                select: { users: true },
              },
            },
          },
        },
      });

      if (!user) {
        return null;
      }

      return {
        id: user.id,
        email: user.email,
        name: user.name,
        dietaryPreferences: user.dietaryPreferences,
        allergies: user.allergies,
        language: user.language,
        timezone: user.timezone,
        householdId: user.householdId,
        household: {
          id: user.household.id,
          name: user.household.name,
          memberCount: user.household._count.users,
          settings: user.household.settings as Record<string, unknown>,
        },
        createdAt: user.createdAt,
        updatedAt: user.updatedAt,
      };
    } catch (error) {
      console.error('Error fetching user profile:', error);
      return null;
    }
  }

  /**
   * Update user profile
   */
  static async updateUserProfile(
    userId: string,
    data: Partial<
      Pick<
        RegisterData,
        'name' | 'dietaryPreferences' | 'allergies' | 'language' | 'timezone'
      >
    >
  ): Promise<AuthUser | null> {
    try {
      const user = await prisma.user.update({
        where: { id: userId },
        data: {
          ...(data.name && { name: data.name }),
          ...(data.dietaryPreferences !== undefined && {
            dietaryPreferences: data.dietaryPreferences,
          }),
          ...(data.allergies !== undefined && { allergies: data.allergies }),
          ...(data.language && { language: data.language }),
          ...(data.timezone && { timezone: data.timezone }),
        },
      });

      return {
        id: user.id,
        email: user.email,
        name: user.name,
        householdId: user.householdId,
        language: user.language,
        timezone: user.timezone,
      };
    } catch (error) {
      console.error('Error updating user profile:', error);
      return null;
    }
  }

  /**
   * Change user password
   */
  static async changePassword(
    userId: string,
    data: ChangePasswordData
  ): Promise<boolean> {
    try {
      // Get current user
      const user = await prisma.user.findUnique({
        where: { id: userId },
      });

      if (!user || !user.passwordHash) {
        throw new Error(AuthError.USER_NOT_FOUND);
      }

      // Verify current password
      const isCurrentPasswordValid = await this.comparePassword(
        data.currentPassword,
        user.passwordHash
      );

      if (!isCurrentPasswordValid) {
        throw new Error(AuthError.INVALID_CREDENTIALS);
      }

      // Hash new password
      const newPasswordHash = await this.hashPassword(data.newPassword);

      // Update password
      await prisma.user.update({
        where: { id: userId },
        data: { passwordHash: newPasswordHash },
      });

      return true;
    } catch (error) {
      console.error('Error changing password:', error);
      return false;
    }
  }

  /**
   * Verify user credentials
   */
  static async verifyCredentials(
    email: string,
    password: string
  ): Promise<AuthUser | null> {
    try {
      const user = await prisma.user.findUnique({
        where: { email },
      });

      if (!user || !user.passwordHash) {
        return null;
      }

      const isValid = await this.comparePassword(password, user.passwordHash);

      if (!isValid) {
        return null;
      }

      return {
        id: user.id,
        email: user.email,
        name: user.name,
        householdId: user.householdId,
        language: user.language,
        timezone: user.timezone,
      };
    } catch (error) {
      console.error('Error verifying credentials:', error);
      return null;
    }
  }

  /**
   * Get user by ID
   */
  static async getUserById(userId: string): Promise<AuthUser | null> {
    try {
      const user = await prisma.user.findUnique({
        where: { id: userId },
      });

      if (!user) {
        return null;
      }

      return {
        id: user.id,
        email: user.email,
        name: user.name,
        householdId: user.householdId,
        language: user.language,
        timezone: user.timezone,
      };
    } catch (error) {
      console.error('Error fetching user by ID:', error);
      return null;
    }
  }

  /**
   * Get user by email
   */
  static async getUserByEmail(email: string): Promise<AuthUser | null> {
    try {
      const user = await prisma.user.findUnique({
        where: { email },
      });

      if (!user) {
        return null;
      }

      return {
        id: user.id,
        email: user.email,
        name: user.name,
        householdId: user.householdId,
        language: user.language,
        timezone: user.timezone,
      };
    } catch (error) {
      console.error('Error fetching user by email:', error);
      return null;
    }
  }
}

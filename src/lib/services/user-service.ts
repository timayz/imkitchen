import { User, DietaryPreference, Language } from '@prisma/client';
import bcrypt from 'bcryptjs';
import { userRepository, CreateUserData, UpdateUserData } from '../repositories/user-repository';
import { householdRepository } from '../repositories/household-repository';
import { logger, withLogging } from '../logger';
import { db } from '../db';

// Service layer types
export interface CreateUserWithHouseholdData {
  email: string;
  name: string;
  password: string;
  householdName: string;
  dietaryPreferences?: DietaryPreference[];
  allergies?: string[];
  language?: Language;
  timezone?: string;
  householdSettings?: Record<string, unknown>;
}

export interface UserProfileData {
  id: string;
  email: string;
  name: string;
  dietaryPreferences: DietaryPreference[];
  allergies: string[];
  language: Language;
  timezone: string;
  household: {
    id: string;
    name: string;
    memberCount: number;
  };
  stats: {
    joinedAt: Date;
    lastActivityAt: Date | null;
    activeSessions: number;
  };
}

// User service class
export class UserService {
  
  // Create user with new household (registration flow)
  async createUserWithHousehold(data: CreateUserWithHouseholdData): Promise<{
    user: User;
    household: { id: string; name: string; settings: Record<string, unknown>; createdAt: Date; updatedAt: Date };
  }> {
    return withLogging(
      'createUserWithHousehold',
      async () => {
        // Hash password
        const passwordHash = await bcrypt.hash(data.password, 12);

        // Use transaction to ensure atomicity
        return db.$transaction(async (tx) => {
          // Create household first
          const household = await tx.household.create({
            data: {
              name: data.householdName,
              settings: data.householdSettings || {
                defaultMeasurementUnit: 'metric',
                sharedInventory: true,
                mealPlanningAccess: 'all-members',
                notificationPreferences: {
                  expirationAlerts: true,
                  mealReminders: true,
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
              dietaryPreferences: data.dietaryPreferences || [],
              allergies: data.allergies || [],
              householdId: household.id,
              language: data.language || Language.EN,
              timezone: data.timezone || 'UTC',
            },
          });

          logger.info('User and household created successfully', {
            userId: user.id,
            householdId: household.id,
            email: data.email,
          });

          return { user, household };
        });
      },
      { email: data.email, householdName: data.householdName }
    );
  }

  // Add user to existing household
  async addUserToHousehold(userData: CreateUserData): Promise<User> {
    return withLogging(
      'addUserToHousehold',
      async () => {
        // Verify household exists and get current member count
        const household = await householdRepository.findWithMembers(userData.householdId);
        if (!household) {
          throw new Error('Household not found');
        }

        // Check household member limit (configurable)
        const maxMembers = (household.settings as any)?.maxMembers || 10;
        if (household._count.users >= maxMembers) {
          throw new Error(`Household has reached maximum member limit of ${maxMembers}`);
        }

        const user = await userRepository.create(userData);

        logger.info('User added to household', {
          userId: user.id,
          householdId: userData.householdId,
          email: userData.email,
        });

        return user;
      },
      { email: userData.email, householdId: userData.householdId }
    );
  }

  // Authenticate user
  async authenticateUser(email: string, password: string): Promise<User | null> {
    return withLogging(
      'authenticateUser',
      async () => {
        const user = await userRepository.findByEmail(email);
        if (!user) {
          logger.warn('Authentication failed: user not found', { email });
          return null;
        }

        const isValidPassword = await bcrypt.compare(password, user.passwordHash);
        if (!isValidPassword) {
          logger.warn('Authentication failed: invalid password', { 
            userId: user.id, 
            email 
          });
          return null;
        }

        logger.info('User authenticated successfully', {
          userId: user.id,
          email,
        });

        return user;
      },
      { email }
    );
  }

  // Update user password
  async updatePassword(userId: string, currentPassword: string, newPassword: string): Promise<void> {
    return withLogging(
      'updatePassword',
      async () => {
        const user = await userRepository.findById(userId);
        if (!user) {
          throw new Error('User not found');
        }

        // Verify current password
        const isValidCurrentPassword = await bcrypt.compare(currentPassword, user.passwordHash);
        if (!isValidCurrentPassword) {
          throw new Error('Current password is incorrect');
        }

        // Hash new password
        const newPasswordHash = await bcrypt.hash(newPassword, 12);

        await userRepository.updatePassword(userId, newPasswordHash);

        logger.info('User password updated', { userId });
      },
      { userId }
    );
  }

  // Get user profile with household info
  async getUserProfile(userId: string): Promise<UserProfileData> {
    return withLogging(
      'getUserProfile',
      async () => {
        const [userWithHousehold, userStats, userActivity] = await Promise.all([
          userRepository.findWithHousehold(userId),
          userRepository.getUserStats(userId),
          userRepository.getUserActivity(userId),
        ]);

        if (!userWithHousehold) {
          throw new Error('User not found');
        }

        return {
          id: userWithHousehold.id,
          email: userWithHousehold.email,
          name: userWithHousehold.name,
          dietaryPreferences: userWithHousehold.dietaryPreferences,
          allergies: userWithHousehold.allergies,
          language: userWithHousehold.language,
          timezone: userWithHousehold.timezone,
          household: {
            id: userWithHousehold.household.id,
            name: userWithHousehold.household.name,
            memberCount: userStats.householdMemberCount,
          },
          stats: {
            joinedAt: userStats.joinedAt,
            lastActivityAt: userActivity.lastLoginAt,
            activeSessions: userStats.activeSessions,
          },
        };
      },
      { userId }
    );
  }

  // Update user preferences
  async updateUserPreferences(
    userId: string,
    preferences: {
      dietaryPreferences?: DietaryPreference[];
      allergies?: string[];
      language?: Language;
      timezone?: string;
      name?: string;
    }
  ): Promise<User> {
    return withLogging(
      'updateUserPreferences',
      async () => {
        // Separate user table updates from preferences
        const userUpdates: UpdateUserData = {};
        const preferenceUpdates: Partial<{ dietaryPreferences: DietaryPreference[]; allergies: string[]; language: Language; timezone: string }> = {};

        if (preferences.name !== undefined) {
          userUpdates.name = preferences.name;
        }

        if (preferences.dietaryPreferences !== undefined) {
          preferenceUpdates.dietaryPreferences = preferences.dietaryPreferences;
        }

        if (preferences.allergies !== undefined) {
          preferenceUpdates.allergies = preferences.allergies;
        }

        if (preferences.language !== undefined) {
          preferenceUpdates.language = preferences.language;
        }

        if (preferences.timezone !== undefined) {
          preferenceUpdates.timezone = preferences.timezone;
        }

        // Update in transaction if both types of updates needed
        if (Object.keys(userUpdates).length > 0 && Object.keys(preferenceUpdates).length > 0) {
          return db.$transaction(async (tx) => {
            await tx.user.update({
              where: { id: userId },
              data: userUpdates,
            });

            return tx.user.update({
              where: { id: userId },
              data: preferenceUpdates,
            });
          });
        }

        // Otherwise use the appropriate single update
        if (Object.keys(preferenceUpdates).length > 0) {
          return userRepository.updatePreferences(userId, preferenceUpdates);
        } else {
          return userRepository.update(userId, userUpdates);
        }
      },
      { userId, preferences }
    );
  }

  // Remove user from household
  async removeUserFromHousehold(userId: string, householdId: string): Promise<void> {
    return withLogging(
      'removeUserFromHousehold',
      async () => {
        // Verify user is in the household
        const user = await userRepository.findById(userId);
        if (!user || user.householdId !== householdId) {
          throw new Error('User is not a member of this household');
        }

        await householdRepository.removeMember(householdId, userId);

        logger.info('User removed from household', {
          userId,
          householdId,
          email: user.email,
        });
      },
      { userId, householdId }
    );
  }

  // Search users with dietary compatibility
  async findCompatibleUsers(
    householdId: string,
    dietaryRestrictions: DietaryPreference[],
    allergens: string[]
  ): Promise<User[]> {
    return withLogging(
      'findCompatibleUsers',
      async () => {
        const householdUsers = await userRepository.findByHousehold(householdId);

        // Filter users who don't conflict with the given restrictions
        return householdUsers.filter(user => {
          // Check if user has any conflicting dietary preferences
          const hasConflictingPreferences = user.dietaryPreferences.some(pref => {
            // Define conflicts (this is simplified - you might want more complex logic)
            const conflicts: Record<DietaryPreference, DietaryPreference[]> = {
              [DietaryPreference.VEGETARIAN]: [],
              [DietaryPreference.VEGAN]: [DietaryPreference.VEGETARIAN],
              [DietaryPreference.KETO]: [],
              [DietaryPreference.PALEO]: [],
              [DietaryPreference.GLUTEN_FREE]: [],
              [DietaryPreference.DAIRY_FREE]: [],
            };

            return conflicts[pref]?.some(conflict => dietaryRestrictions.includes(conflict));
          });

          // Check if user has any of the specified allergens
          const hasConflictingAllergies = user.allergies.some(allergy => 
            allergens.includes(allergy)
          );

          return !hasConflictingPreferences && !hasConflictingAllergies;
        });
      },
      { householdId, dietaryRestrictions, allergens }
    );
  }

  // Get household dietary summary
  async getHouseholdDietarySummary(householdId: string): Promise<{
    allDietaryPreferences: DietaryPreference[];
    allAllergies: string[];
    commonRestrictions: DietaryPreference[];
    conflictingPreferences: DietaryPreference[];
  }> {
    return withLogging(
      'getHouseholdDietarySummary',
      async () => {
        const users = await userRepository.findByHousehold(householdId);

        const allDietaryPreferences = Array.from(
          new Set(users.flatMap(user => user.dietaryPreferences))
        );

        const allAllergies = Array.from(
          new Set(users.flatMap(user => user.allergies))
        );

        // Find common restrictions (present in all users)
        const commonRestrictions = allDietaryPreferences.filter(pref =>
          users.every(user => user.dietaryPreferences.includes(pref))
        );

        // Simplified conflict detection
        const conflictingPreferences = allDietaryPreferences.filter(pref => {
          if (pref === DietaryPreference.VEGAN) {
            return users.some(user => 
              !user.dietaryPreferences.includes(DietaryPreference.VEGAN) &&
              !user.dietaryPreferences.includes(DietaryPreference.VEGETARIAN)
            );
          }
          return false;
        });

        return {
          allDietaryPreferences,
          allAllergies,
          commonRestrictions,
          conflictingPreferences,
        };
      },
      { householdId }
    );
  }
}

// Export singleton instance
export const userService = new UserService();
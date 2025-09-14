import { Household, User } from '@prisma/client';
import { db } from '../db';
import { BaseRepository } from './base-repository';
import { logDatabaseOperation } from '../logger';

// Household repository types
export type HouseholdWithMembers = Household & {
  users: User[];
  _count: {
    users: number;
  };
};

export type CreateHouseholdData = {
  name: string;
  settings?: Record<string, unknown>;
};

export type UpdateHouseholdData = Partial<CreateHouseholdData>;

export interface IHouseholdRepository {
  findWithMembers(id: string): Promise<HouseholdWithMembers | null>;
  addMember(householdId: string, userId: string): Promise<void>;
  removeMember(householdId: string, userId: string): Promise<void>;
  updateSettings(id: string, settings: Record<string, unknown>): Promise<Household>;
  getHouseholdStats(id: string): Promise<{
    memberCount: number;
    createdAt: Date;
    activeMembersCount: number;
    totalSessions: number;
  }>;
  findByMemberId(userId: string): Promise<HouseholdWithMembers | null>;
  searchHouseholds(query: string): Promise<Household[]>;
}

// Household repository implementation
export class HouseholdRepository 
  extends BaseRepository<Household, CreateHouseholdData, UpdateHouseholdData> 
  implements IHouseholdRepository {

  constructor() {
    super(db, 'Household');
  }

  protected getModel() {
    return this.db.household;
  }

  // Find household with all members
  async findWithMembers(id: string): Promise<HouseholdWithMembers | null> {
    return logDatabaseOperation(
      'findWithMembers',
      this.modelName,
      async () => {
        return this.getModel().findUnique({
          where: { id },
          include: {
            users: {
              select: {
                id: true,
                email: true,
                name: true,
                language: true,
                timezone: true,
                dietaryPreferences: true,
                allergies: true,
                createdAt: true,
              },
              orderBy: { createdAt: 'asc' },
            },
            _count: {
              select: { users: true },
            },
          },
        });
      },
      { id }
    );
  }

  // Add member to household
  async addMember(householdId: string, userId: string): Promise<void> {
    return logDatabaseOperation(
      'addMember',
      this.modelName,
      async () => {
        // Verify household exists
        const household = await this.findById(householdId);
        if (!household) {
          throw new Error('Household not found');
        }

        // Verify user exists and is not already in another household
        const user = await this.db.user.findUnique({
          where: { id: userId },
        });

        if (!user) {
          throw new Error('User not found');
        }

        if (user.householdId === householdId) {
          throw new Error('User is already a member of this household');
        }

        if (user.householdId) {
          throw new Error('User is already a member of another household');
        }

        // Update user's household
        await this.db.user.update({
          where: { id: userId },
          data: { householdId },
        });
      },
      { householdId, userId }
    );
  }

  // Remove member from household
  async removeMember(householdId: string, userId: string): Promise<void> {
    return logDatabaseOperation(
      'removeMember',
      this.modelName,
      async () => {
        // Verify user is in the household
        const user = await this.db.user.findFirst({
          where: {
            id: userId,
            householdId,
          },
        });

        if (!user) {
          throw new Error('User is not a member of this household');
        }

        // Check if this is the last member
        const memberCount = await this.db.user.count({
          where: { householdId },
        });

        if (memberCount === 1) {
          // If removing the last member, delete the household
          await this.executeInTransaction(async (tx) => {
            // Delete user sessions first
            await tx.session.deleteMany({
              where: { userId },
            });

            // Delete the user
            await tx.user.delete({
              where: { id: userId },
            });

            // Delete the household
            await tx.household.delete({
              where: { id: householdId },
            });
          });
        } else {
          // Just remove the user from the household
          await this.db.user.delete({
            where: { id: userId },
          });
        }
      },
      { householdId, userId }
    );
  }

  // Update household settings
  async updateSettings(id: string, settings: Record<string, unknown>): Promise<Household> {
    return logDatabaseOperation(
      'updateSettings',
      this.modelName,
      async () => {
        // Merge with existing settings
        const household = await this.findById(id);
        if (!household) {
          throw new Error('Household not found');
        }

        const mergedSettings = {
          ...(household.settings as object || {}),
          ...settings,
        };

        return this.getModel().update({
          where: { id },
          data: { settings: mergedSettings },
        });
      },
      { id, settings }
    );
  }

  // Get household statistics
  async getHouseholdStats(id: string): Promise<{
    memberCount: number;
    createdAt: Date;
    activeMembersCount: number;
    totalSessions: number;
  }> {
    return logDatabaseOperation(
      'getHouseholdStats',
      this.modelName,
      async () => {
        const household = await this.getModel().findUnique({
          where: { id },
          include: {
            users: {
              include: {
                sessions: {
                  where: {
                    expiresAt: { gt: new Date() },
                  },
                },
                _count: {
                  select: { sessions: true },
                },
              },
            },
            _count: {
              select: { users: true },
            },
          },
        });

        if (!household) {
          throw new Error('Household not found');
        }

        const activeMembersCount = household.users.filter(
          user => user.sessions.length > 0
        ).length;

        const totalSessions = household.users.reduce(
          (sum, user) => sum + user._count.sessions,
          0
        );

        return {
          memberCount: household._count.users,
          createdAt: household.createdAt,
          activeMembersCount,
          totalSessions,
        };
      },
      { id }
    );
  }

  // Find household by member ID
  async findByMemberId(userId: string): Promise<HouseholdWithMembers | null> {
    return logDatabaseOperation(
      'findByMemberId',
      this.modelName,
      async () => {
        const user = await this.db.user.findUnique({
          where: { id: userId },
          select: { householdId: true },
        });

        if (!user || !user.householdId) {
          return null;
        }

        return this.findWithMembers(user.householdId);
      },
      { userId }
    );
  }

  // Search households by name
  async searchHouseholds(query: string): Promise<Household[]> {
    return logDatabaseOperation(
      'searchHouseholds',
      this.modelName,
      async () => {
        return this.getModel().findMany({
          where: {
            name: {
              contains: query,
              mode: 'insensitive',
            },
          },
          include: {
            _count: {
              select: { users: true },
            },
          },
          orderBy: { name: 'asc' },
          take: 20, // Limit search results
        });
      },
      { query }
    );
  }

  // Create household with validation
  async create(data: CreateHouseholdData): Promise<Household> {
    return logDatabaseOperation(
      'create',
      this.modelName,
      async () => {
        // Validate household name
        if (!data.name || data.name.trim().length === 0) {
          throw new Error('Household name is required');
        }

        // Check for duplicate names (optional - you may want to allow duplicates)
        const existingHousehold = await this.getModel().findFirst({
          where: {
            name: {
              equals: data.name.trim(),
              mode: 'insensitive',
            },
          },
        });

        if (existingHousehold) {
          throw new Error('A household with this name already exists');
        }

        const defaultSettings = {
          defaultMeasurementUnit: 'metric',
          sharedInventory: true,
          mealPlanningAccess: 'all-members',
          notificationPreferences: {
            expirationAlerts: true,
            mealReminders: true,
            shoppingListUpdates: true,
          },
          ...data.settings,
        };

        return this.getModel().create({
          data: {
            name: data.name.trim(),
            settings: defaultSettings,
          },
        });
      }
    );
  }

  // Transfer household ownership
  async transferOwnership(
    householdId: string,
    currentOwnerId: string,
    newOwnerId: string
  ): Promise<void> {
    return logDatabaseOperation(
      'transferOwnership',
      this.modelName,
      async () => {
        // Verify both users are in the household
        const [currentOwner, newOwner] = await Promise.all([
          this.db.user.findFirst({
            where: { id: currentOwnerId, householdId },
          }),
          this.db.user.findFirst({
            where: { id: newOwnerId, householdId },
          }),
        ]);

        if (!currentOwner || !newOwner) {
          throw new Error('One or both users are not members of this household');
        }

        // Update household settings to reflect new ownership
        await this.updateSettings(householdId, {
          ownerId: newOwnerId,
          ownershipTransferredAt: new Date(),
          previousOwnerId: currentOwnerId,
        });
      },
      { householdId, currentOwnerId, newOwnerId }
    );
  }

  // Get household member activity
  async getMemberActivity(
    householdId: string,
    days: number = 30
  ): Promise<{
    memberId: string;
    memberName: string;
    sessionCount: number;
    lastActivityAt: Date | null;
  }[]> {
    return logDatabaseOperation(
      'getMemberActivity',
      this.modelName,
      async () => {
        const cutoffDate = new Date();
        cutoffDate.setDate(cutoffDate.getDate() - days);

        const household = await this.getModel().findUnique({
          where: { id: householdId },
          include: {
            users: {
              include: {
                sessions: {
                  where: {
                    createdAt: { gte: cutoffDate },
                  },
                  orderBy: { createdAt: 'desc' },
                },
              },
            },
          },
        });

        if (!household) {
          throw new Error('Household not found');
        }

        return household.users.map(user => ({
          memberId: user.id,
          memberName: user.name,
          sessionCount: user.sessions.length,
          lastActivityAt: user.sessions.length > 0 ? user.sessions[0].createdAt : null,
        }));
      },
      { householdId, days }
    );
  }

  // Cleanup inactive households
  async cleanupInactiveHouseholds(daysInactive: number = 90): Promise<number> {
    return logDatabaseOperation(
      'cleanupInactiveHouseholds',
      this.modelName,
      async () => {
        const cutoffDate = new Date();
        cutoffDate.setDate(cutoffDate.getDate() - daysInactive);

        // Find households with no recent activity
        const inactiveHouseholds = await this.getModel().findMany({
          where: {
            users: {
              every: {
                sessions: {
                  every: {
                    createdAt: { lt: cutoffDate },
                  },
                },
              },
            },
          },
          include: {
            users: {
              include: {
                sessions: true,
              },
            },
          },
        });

        let deletedCount = 0;

        for (const household of inactiveHouseholds) {
          await this.executeInTransaction(async (tx) => {
            // Delete all sessions for household members
            await tx.session.deleteMany({
              where: {
                userId: {
                  in: household.users.map(user => user.id),
                },
              },
            });

            // Delete all users in the household
            await tx.user.deleteMany({
              where: { householdId: household.id },
            });

            // Delete the household
            await tx.household.delete({
              where: { id: household.id },
            });

            deletedCount++;
          });
        }

        return deletedCount;
      },
      { daysInactive }
    );
  }
}

// Export singleton instance
export const householdRepository = new HouseholdRepository();
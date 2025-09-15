import { Household } from '@prisma/client';
import {
  householdRepository,
  CreateHouseholdData,
} from '../repositories/household-repository';
import { userRepository } from '../repositories/user-repository';
import { logger, withLogging } from '../logger';
import { db } from '../db';

// Household settings interface
interface HouseholdSettings {
  ownerId?: string;
  maxMembers?: number;
  mealPlanningAccess?: 'owner-only' | 'all-members';
  [key: string]: unknown;
}

// Service layer types
export interface HouseholdSummary {
  id: string;
  name: string;
  memberCount: number;
  createdAt: Date;
  settings: Record<string, unknown>;
  members: {
    id: string;
    name: string;
    email: string;
    role: 'owner' | 'member';
    joinedAt: Date;
    lastActivity: Date | null;
  }[];
  stats: {
    activeMembersCount: number;
    totalSessions: number;
  };
}

export interface HouseholdInviteData {
  householdId: string;
  invitedBy: string;
  inviteeEmail: string;
  role: 'member';
  expiresAt?: Date;
}

// Household service class
export class HouseholdService {
  // Get complete household information
  async getHouseholdSummary(householdId: string): Promise<HouseholdSummary> {
    return withLogging(
      'getHouseholdSummary',
      async () => {
        const [household, stats, memberActivity] = await Promise.all([
          householdRepository.findWithMembers(householdId),
          householdRepository.getHouseholdStats(householdId),
          householdRepository.getMemberActivity(householdId),
        ]);

        if (!household) {
          throw new Error('Household not found');
        }

        // Determine member roles (simplified - you might want more complex logic)
        const ownerId =
          (household.settings as HouseholdSettings)?.ownerId ||
          household.users[0]?.id;

        const members = household.users.map(user => {
          const activity = memberActivity.find(
            activity => activity.memberId === user.id
          );

          return {
            id: user.id,
            name: user.name,
            email: user.email,
            role:
              user.id === ownerId ? ('owner' as const) : ('member' as const),
            joinedAt: user.createdAt,
            lastActivity: activity?.lastActivityAt || null,
          };
        });

        return {
          id: household.id,
          name: household.name,
          memberCount: household._count.users,
          createdAt: household.createdAt,
          settings: household.settings,
          members,
          stats: {
            activeMembersCount: stats.activeMembersCount,
            totalSessions: stats.totalSessions,
          },
        };
      },
      { householdId }
    );
  }

  // Update household settings
  async updateHouseholdSettings(
    householdId: string,
    userId: string,
    settings: Record<string, unknown>
  ): Promise<Household> {
    return withLogging(
      'updateHouseholdSettings',
      async () => {
        // Verify user is a member of the household
        const user = await userRepository.findById(userId);
        if (!user || user.householdId !== householdId) {
          throw new Error('User is not a member of this household');
        }

        // Check if user has permission to update settings (simplified)
        const household = await householdRepository.findById(householdId);
        if (!household) {
          throw new Error('Household not found');
        }

        const ownerId =
          (household.settings as HouseholdSettings)?.ownerId ||
          household.users?.[0]?.id;
        const mealPlanningAccess =
          (household.settings as HouseholdSettings)?.mealPlanningAccess ||
          'all-members';

        if (ownerId !== userId && mealPlanningAccess === 'owner') {
          throw new Error('Only the household owner can update these settings');
        }

        const updatedHousehold = await householdRepository.updateSettings(
          householdId,
          settings
        );

        logger.info('Household settings updated', {
          householdId,
          userId,
          settings: Object.keys(settings),
        });

        return updatedHousehold;
      },
      { householdId, userId }
    );
  }

  // Invite user to household (simplified - would need proper invite system)
  async createHouseholdInvite(inviteData: HouseholdInviteData): Promise<{
    inviteId: string;
    expiresAt: Date;
  }> {
    return withLogging(
      'createHouseholdInvite',
      async () => {
        // Verify inviter is a member of the household
        const inviter = await userRepository.findById(inviteData.invitedBy);
        if (!inviter || inviter.householdId !== inviteData.householdId) {
          throw new Error('Inviter is not a member of this household');
        }

        // Check if invitee already exists
        const existingUser = await userRepository.findByEmail(
          inviteData.inviteeEmail
        );
        if (existingUser) {
          throw new Error('User with this email already exists');
        }

        // Get household to check member limits
        const household = await householdRepository.findWithMembers(
          inviteData.householdId
        );
        if (!household) {
          throw new Error('Household not found');
        }

        const maxMembers =
          (household.settings as HouseholdSettings)?.maxMembers || 10;
        if (household._count.users >= maxMembers) {
          throw new Error(
            `Household has reached maximum member limit of ${maxMembers}`
          );
        }

        // In a real implementation, you'd create an invite record
        // For now, we'll just return a mock invite
        const inviteId = `invite_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
        const expiresAt =
          inviteData.expiresAt ||
          new Date(Date.now() + 7 * 24 * 60 * 60 * 1000); // 7 days

        logger.info('Household invite created', {
          inviteId,
          householdId: inviteData.householdId,
          invitedBy: inviteData.invitedBy,
          inviteeEmail: inviteData.inviteeEmail,
        });

        return { inviteId, expiresAt };
      },
      {
        householdId: inviteData.householdId,
        inviteeEmail: inviteData.inviteeEmail,
      }
    );
  }

  // Remove member from household
  async removeMemberFromHousehold(
    householdId: string,
    memberIdToRemove: string,
    removedBy: string
  ): Promise<void> {
    return withLogging(
      'removeMemberFromHousehold',
      async () => {
        // Verify the user performing the action is a member
        const remover = await userRepository.findById(removedBy);
        if (!remover || remover.householdId !== householdId) {
          throw new Error('You are not a member of this household');
        }

        // Verify the user to be removed is a member
        const memberToRemove = await userRepository.findById(memberIdToRemove);
        if (!memberToRemove || memberToRemove.householdId !== householdId) {
          throw new Error('User is not a member of this household');
        }

        // Get household to check permissions
        const household = await householdRepository.findById(householdId);
        if (!household) {
          throw new Error('Household not found');
        }

        const ownerId = (household.settings as HouseholdSettings)?.ownerId;

        // Permission checks
        if (memberIdToRemove === removedBy) {
          // User is removing themselves - always allowed
        } else if (ownerId === removedBy) {
          // Owner can remove anyone
        } else {
          throw new Error('You do not have permission to remove this member');
        }

        // Don't allow removing the owner unless they're removing themselves
        if (ownerId === memberIdToRemove && removedBy !== memberIdToRemove) {
          throw new Error('Cannot remove the household owner');
        }

        await householdRepository.removeMember(householdId, memberIdToRemove);

        logger.info('Member removed from household', {
          householdId,
          memberIdToRemove,
          removedBy,
          email: memberToRemove.email,
        });
      },
      { householdId, memberIdToRemove, removedBy }
    );
  }

  // Transfer household ownership
  async transferOwnership(
    householdId: string,
    currentOwnerId: string,
    newOwnerId: string
  ): Promise<void> {
    return withLogging(
      'transferOwnership',
      async () => {
        // Verify current owner
        const currentOwner = await userRepository.findById(currentOwnerId);
        if (!currentOwner || currentOwner.householdId !== householdId) {
          throw new Error('Current owner is not a member of this household');
        }

        // Verify new owner
        const newOwner = await userRepository.findById(newOwnerId);
        if (!newOwner || newOwner.householdId !== householdId) {
          throw new Error('New owner is not a member of this household');
        }

        // Verify current user is actually the owner
        const household = await householdRepository.findById(householdId);
        if (!household) {
          throw new Error('Household not found');
        }

        const ownerId =
          (household.settings as HouseholdSettings)?.ownerId ||
          household.users?.[0]?.id;
        if (ownerId !== currentOwnerId) {
          throw new Error('You are not the current owner of this household');
        }

        await householdRepository.transferOwnership(
          householdId,
          currentOwnerId,
          newOwnerId
        );

        logger.info('Household ownership transferred', {
          householdId,
          fromOwnerId: currentOwnerId,
          toOwnerId: newOwnerId,
        });
      },
      { householdId, currentOwnerId, newOwnerId }
    );
  }

  // Delete household (owner only)
  async deleteHousehold(householdId: string, userId: string): Promise<void> {
    return withLogging(
      'deleteHousehold',
      async () => {
        // Verify user is the owner
        const user = await userRepository.findById(userId);
        if (!user || user.householdId !== householdId) {
          throw new Error('User is not a member of this household');
        }

        const household =
          await householdRepository.findWithMembers(householdId);
        if (!household) {
          throw new Error('Household not found');
        }

        const ownerId =
          (household.settings as HouseholdSettings)?.ownerId ||
          household.users[0]?.id;
        if (ownerId !== userId) {
          throw new Error('Only the household owner can delete the household');
        }

        // Delete in transaction
        await db.$transaction(async tx => {
          // Delete all sessions for household members
          const userIds = household.users.map(user => user.id);
          await tx.session.deleteMany({
            where: { userId: { in: userIds } },
          });

          // Delete all users in the household
          await tx.user.deleteMany({
            where: { householdId },
          });

          // Delete the household
          await tx.household.delete({
            where: { id: householdId },
          });
        });

        logger.info('Household deleted', {
          householdId,
          deletedBy: userId,
          memberCount: household._count.users,
        });
      },
      { householdId, userId }
    );
  }

  // Get household analytics
  async getHouseholdAnalytics(
    householdId: string,
    days: number = 30
  ): Promise<{
    memberActivity: {
      memberId: string;
      memberName: string;
      sessionCount: number;
      lastActivityAt: Date | null;
    }[];
    totalSessions: number;
    avgSessionsPerMember: number;
    mostActiveDay: string | null;
    leastActiveDay: string | null;
  }> {
    return withLogging(
      'getHouseholdAnalytics',
      async () => {
        const [memberActivity] = await Promise.all([
          householdRepository.getMemberActivity(householdId, days),
          householdRepository.getHouseholdStats(householdId),
        ]);

        const totalSessions = memberActivity.reduce(
          (sum, member) => sum + member.sessionCount,
          0
        );
        const avgSessionsPerMember =
          memberActivity.length > 0 ? totalSessions / memberActivity.length : 0;

        // Simplified day analysis (would need more complex querying for real implementation)
        const mostActiveDay = null; // Would need session timestamps grouped by day
        const leastActiveDay = null;

        return {
          memberActivity,
          totalSessions,
          avgSessionsPerMember,
          mostActiveDay,
          leastActiveDay,
        };
      },
      { householdId, days }
    );
  }

  // Create household with first member
  async createHouseholdWithMember(
    householdData: CreateHouseholdData,
    ownerData: {
      email: string;
      name: string;
      passwordHash: string;
    }
  ): Promise<{
    household: Household;
    owner: { id: string; name: string; email: string };
  }> {
    return withLogging(
      'createHouseholdWithMember',
      async () => {
        return db.$transaction(async tx => {
          // Create household
          const household = await tx.household.create({
            data: {
              ...householdData,
              settings: {
                ...householdData.settings,
                ownerId: null, // Will be set after user creation
              },
            },
          });

          // Create owner user
          const owner = await tx.user.create({
            data: {
              ...ownerData,
              householdId: household.id,
            },
          });

          // Update household settings with owner ID
          await tx.household.update({
            where: { id: household.id },
            data: {
              settings: {
                ...household.settings,
                ownerId: owner.id,
              },
            },
          });

          logger.info('Household created with owner', {
            householdId: household.id,
            ownerId: owner.id,
            householdName: household.name,
          });

          return { household, owner };
        });
      },
      { householdName: householdData.name, ownerEmail: ownerData.email }
    );
  }
}

// Export singleton instance
export const householdService = new HouseholdService();

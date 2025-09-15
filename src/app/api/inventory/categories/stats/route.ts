import { NextRequest, NextResponse } from 'next/server';
import { auth } from '@/lib/auth';
import { prisma } from '@/lib/db';

// eslint-disable-next-line @typescript-eslint/no-unused-vars
export async function GET(_request: NextRequest) {
  try {
    const session = await auth();
    if (!session?.user?.householdId) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }

    // Get current date for expiration calculations
    const today = new Date();
    const oneWeekFromNow = new Date();
    oneWeekFromNow.setDate(today.getDate() + 7);

    // Get predefined categories
    const predefinedCategories = [
      'proteins',
      'vegetables',
      'fruits',
      'grains',
      'dairy',
      'spices',
      'condiments',
      'beverages',
      'baking',
      'frozen',
    ];

    // Get custom categories
    const customCategories = await prisma.customCategory.findMany({
      where: {
        householdId: session.user.householdId,
      },
      select: {
        id: true,
        name: true,
      },
    });

    // Build all categories list
    const allCategories = [
      ...predefinedCategories,
      ...customCategories.map(c => c.id),
    ];

    // Get statistics for each category
    const categoryStats = await Promise.all(
      allCategories.map(async category => {
        const items = await prisma.inventoryItem.findMany({
          where: {
            householdId: session.user.householdId,
            category,
          },
          select: {
            id: true,
            expirationDate: true,
            estimatedCost: true,
          },
        });

        const itemCount = items.length;
        const expiringToday = items.filter(
          item => item.expirationDate && item.expirationDate <= today
        ).length;

        const expiringThisWeek = items.filter(
          item =>
            item.expirationDate &&
            item.expirationDate > today &&
            item.expirationDate <= oneWeekFromNow
        ).length;

        const totalValue = items.reduce(
          (sum, item) =>
            sum + (item.estimatedCost ? Number(item.estimatedCost) : 0),
          0
        );

        return {
          category,
          itemCount,
          expiringThisWeek,
          expiringToday,
          totalValue,
          lastUpdated: new Date(),
        };
      })
    );

    // Filter out categories with no items
    const filteredStats = categoryStats.filter(stat => stat.itemCount > 0);

    return NextResponse.json(filteredStats);
  } catch (error) {
    console.error('Failed to fetch category statistics:', error);
    return NextResponse.json(
      { error: 'Failed to fetch category statistics' },
      { status: 500 }
    );
  }
}

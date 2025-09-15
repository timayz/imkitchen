import { NextRequest, NextResponse } from 'next/server';
import { auth } from '@/lib/auth';
import { prisma } from '@/lib/db';
import { z } from 'zod';

const bulkUpdateSchema = z.object({
  itemIds: z.array(z.string().uuid()),
  updates: z.object({
    category: z.string().optional(),
    location: z.enum(['pantry', 'refrigerator', 'freezer']).optional(),
  }),
});

export async function PUT(request: NextRequest) {
  try {
    const session = await auth();
    if (!session?.user?.householdId) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }

    const body = await request.json();
    const validatedData = bulkUpdateSchema.parse(body);

    if (validatedData.itemIds.length === 0) {
      return NextResponse.json(
        { error: 'No items specified for update' },
        { status: 400 }
      );
    }

    if (validatedData.itemIds.length > 100) {
      return NextResponse.json(
        { error: 'Too many items. Maximum 100 items per bulk update.' },
        { status: 400 }
      );
    }

    // Verify all items belong to user's household
    const itemsCount = await prisma.inventoryItem.count({
      where: {
        id: { in: validatedData.itemIds },
        householdId: session.user.householdId,
      },
    });

    if (itemsCount !== validatedData.itemIds.length) {
      return NextResponse.json(
        { error: 'Some items not found or unauthorized' },
        { status: 404 }
      );
    }

    // If updating category to a custom category, verify it exists and belongs to household
    if (validatedData.updates.category) {
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

      if (!predefinedCategories.includes(validatedData.updates.category)) {
        // It's a custom category, verify it exists
        const customCategory = await prisma.customCategory.findFirst({
          where: {
            id: validatedData.updates.category,
            householdId: session.user.householdId,
          },
        });

        if (!customCategory) {
          return NextResponse.json(
            { error: 'Custom category not found' },
            { status: 404 }
          );
        }
      }
    }

    // Perform bulk update
    const updateData: Record<string, string> = {};
    if (validatedData.updates.category) {
      updateData.category = validatedData.updates.category;
    }
    if (validatedData.updates.location) {
      updateData.location = validatedData.updates.location;
    }

    await prisma.inventoryItem.updateMany({
      where: {
        id: { in: validatedData.itemIds },
        householdId: session.user.householdId,
      },
      data: updateData as any, // eslint-disable-line @typescript-eslint/no-explicit-any
    });

    // Fetch and return updated items
    const updatedItems = await prisma.inventoryItem.findMany({
      where: {
        id: { in: validatedData.itemIds },
        householdId: session.user.householdId,
      },
      include: {
        addedByUser: {
          select: { name: true },
        },
      },
    });

    return NextResponse.json(updatedItems);
  } catch (error) {
    if (error instanceof z.ZodError) {
      return NextResponse.json(
        { error: 'Invalid request data', details: error.issues },
        { status: 400 }
      );
    }

    console.error('Failed to bulk update items:', error);
    return NextResponse.json(
      { error: 'Failed to bulk update items' },
      { status: 500 }
    );
  }
}

import { NextRequest, NextResponse } from 'next/server';
import { z } from 'zod';
import { auth } from '@/lib/auth';
import { prisma } from '@/lib/db';
import type { Prisma } from '@prisma/client';

const UpdateInventoryItemSchema = z.object({
  name: z.string().min(1).max(255).optional(),
  quantity: z.number().positive().optional(),
  unit: z.string().min(1).max(50).optional(),
  category: z
    .enum([
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
    ])
    .optional(),
  location: z.enum(['pantry', 'refrigerator', 'freezer']).optional(),
  expirationDate: z
    .string()
    .optional()
    .transform(str => (str ? new Date(str) : null)),
  estimatedCost: z.number().optional().nullable(),
});

export async function PUT(
  request: NextRequest,
  { params }: { params: Promise<{ itemId: string }> }
) {
  try {
    const session = await auth();
    if (!session?.user?.householdId) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }

    const { itemId } = await params;
    const body = await request.json();
    const validatedData = UpdateInventoryItemSchema.parse(body);

    // Verify item belongs to user's household
    const existingItem = await prisma.inventoryItem.findFirst({
      where: {
        id: itemId,
        householdId: session.user.householdId,
      },
    });

    if (!existingItem) {
      return NextResponse.json(
        { error: 'Item not found or access denied' },
        { status: 404 }
      );
    }

    // Check for duplicate name if name is being updated
    if (validatedData.name && validatedData.name !== existingItem.name) {
      const duplicateItem = await prisma.inventoryItem.findFirst({
        where: {
          name: {
            equals: validatedData.name,
            mode: 'insensitive',
          },
          householdId: session.user.householdId,
          id: {
            not: itemId,
          },
        },
      });

      if (duplicateItem) {
        return NextResponse.json(
          { error: 'Item with this name already exists' },
          { status: 400 }
        );
      }
    }

    const updateData: Prisma.InventoryItemUpdateInput = {
      ...Object.fromEntries(
        Object.entries(validatedData).filter(([, value]) => value !== undefined)
      ),
      updatedAt: new Date(),
    };

    const updatedItem = await prisma.inventoryItem.update({
      where: { id: itemId },
      data: updateData,
      include: {
        addedByUser: {
          select: { name: true },
        },
      },
    });

    return NextResponse.json(updatedItem);
  } catch (error) {
    console.error('Error updating inventory item:', error);

    if (error instanceof z.ZodError) {
      return NextResponse.json(
        { error: 'Validation failed', details: error.issues },
        { status: 400 }
      );
    }

    return NextResponse.json(
      { error: 'Internal server error' },
      { status: 500 }
    );
  }
}

export async function DELETE(
  _request: NextRequest,
  { params }: { params: Promise<{ itemId: string }> }
) {
  try {
    const session = await auth();
    if (!session?.user?.householdId) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }

    const { itemId } = await params;

    // Verify item belongs to user's household
    const existingItem = await prisma.inventoryItem.findFirst({
      where: {
        id: itemId,
        householdId: session.user.householdId,
      },
    });

    if (!existingItem) {
      return NextResponse.json(
        { error: 'Item not found or access denied' },
        { status: 404 }
      );
    }

    await prisma.inventoryItem.delete({
      where: { id: itemId },
    });

    return NextResponse.json(
      { message: 'Item deleted successfully' },
      { status: 204 }
    );
  } catch (error) {
    console.error('Error deleting inventory item:', error);
    return NextResponse.json(
      { error: 'Internal server error' },
      { status: 500 }
    );
  }
}

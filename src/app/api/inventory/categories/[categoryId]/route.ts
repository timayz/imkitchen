import { NextRequest, NextResponse } from 'next/server';
import { auth } from '@/lib/auth';
import { prisma } from '@/lib/db';
import { z } from 'zod';

const updateCategorySchema = z.object({
  name: z.string().min(1).max(50).optional(),
  color: z
    .string()
    .regex(/^#[0-9A-F]{6}$/i)
    .optional(),
  icon: z.string().min(1).max(50).optional(),
});

export async function PUT(
  request: NextRequest,
  { params }: { params: Promise<{ categoryId: string }> }
) {
  try {
    const session = await auth();
    if (!session?.user?.householdId) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }

    const { categoryId } = await params;
    const body = await request.json();
    const validatedData = updateCategorySchema.parse(body);

    // Verify category belongs to user's household
    const existingCategory = await prisma.customCategory.findFirst({
      where: {
        id: categoryId,
        householdId: session.user.householdId,
      },
    });

    if (!existingCategory) {
      return NextResponse.json(
        { error: 'Category not found' },
        { status: 404 }
      );
    }

    // Check if new name already exists (if name is being updated)
    if (validatedData.name && validatedData.name !== existingCategory.name) {
      const nameExists = await prisma.customCategory.findFirst({
        where: {
          name: validatedData.name,
          householdId: session.user.householdId,
          id: { not: categoryId },
        },
      });

      if (nameExists) {
        return NextResponse.json(
          { error: 'Category name already exists' },
          { status: 400 }
        );
      }
    }

    // Build update data object with only defined properties
    const updateData: { name?: string; color?: string; icon?: string } = {};
    if (validatedData.name !== undefined) updateData.name = validatedData.name;
    if (validatedData.color !== undefined)
      updateData.color = validatedData.color;
    if (validatedData.icon !== undefined) updateData.icon = validatedData.icon;

    const category = await prisma.customCategory.update({
      where: { id: categoryId },
      data: updateData,
    });

    return NextResponse.json(category);
  } catch (error) {
    if (error instanceof z.ZodError) {
      return NextResponse.json(
        { error: 'Invalid request data', details: error.issues },
        { status: 400 }
      );
    }

    console.error('Failed to update category:', error);
    return NextResponse.json(
      { error: 'Failed to update category' },
      { status: 500 }
    );
  }
}

export async function DELETE(
  _request: NextRequest,
  { params }: { params: Promise<{ categoryId: string }> }
) {
  try {
    const session = await auth();
    if (!session?.user?.householdId) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }

    const { categoryId } = await params;

    // Verify category belongs to user's household
    const existingCategory = await prisma.customCategory.findFirst({
      where: {
        id: categoryId,
        householdId: session.user.householdId,
      },
    });

    if (!existingCategory) {
      return NextResponse.json(
        { error: 'Category not found' },
        { status: 404 }
      );
    }

    // Check if any items are using this category
    const itemsUsingCategory = await prisma.inventoryItem.count({
      where: {
        category: categoryId,
        householdId: session.user.householdId,
      },
    });

    if (itemsUsingCategory > 0) {
      return NextResponse.json(
        {
          error:
            'Cannot delete category with items. Please move items to another category first.',
          itemCount: itemsUsingCategory,
        },
        { status: 400 }
      );
    }

    await prisma.customCategory.delete({
      where: { id: categoryId },
    });

    return NextResponse.json({ success: true }, { status: 204 });
  } catch (error) {
    console.error('Failed to delete category:', error);
    return NextResponse.json(
      { error: 'Failed to delete category' },
      { status: 500 }
    );
  }
}

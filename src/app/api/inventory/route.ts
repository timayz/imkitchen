import { NextRequest, NextResponse } from 'next/server';
import { z } from 'zod';
import { auth } from '@/lib/auth';
import { prisma } from '@/lib/db';
import type { Prisma } from '@prisma/client';

const CreateInventoryItemSchema = z.object({
  name: z.string().min(1).max(255),
  quantity: z.number().positive(),
  unit: z.string().min(1).max(50),
  category: z.string().min(1).max(50), // Can be predefined enum or custom category ID
  location: z.enum(['pantry', 'refrigerator', 'freezer']),
  expirationDate: z
    .string()
    .optional()
    .transform(str => (str ? new Date(str) : null)),
  purchaseDate: z
    .string()
    .optional()
    .transform(str => (str ? new Date(str) : new Date())),
  estimatedCost: z.number().optional().nullable(),
});

export async function GET(request: NextRequest) {
  try {
    const session = await auth();
    if (!session?.user?.householdId) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }

    const { searchParams } = new URL(request.url);
    const location = searchParams.get('location');
    const category = searchParams.get('category');
    const expiringSoon = searchParams.get('expiringSoon') === 'true';
    const search = searchParams.get('search');
    const sortBy = searchParams.get('sortBy') || 'recently_added';
    const sortDirection = searchParams.get('sortDirection') || 'desc';
    const showOnlyExpiring = searchParams.get('showOnlyExpiring') === 'true';

    const where: Record<string, unknown> = {
      householdId: session.user.householdId,
    };

    if (location) {
      where.location = location;
    }

    if (category) {
      where.category = category;
    }

    if (search) {
      where.name = {
        contains: search,
        mode: 'insensitive',
      };
    }

    if (expiringSoon || showOnlyExpiring) {
      const futureDate = new Date();
      futureDate.setDate(futureDate.getDate() + 7);
      where.expirationDate = {
        lte: futureDate,
        gte: new Date(),
      };
    }

    // Build order by clause based on sortBy and sortDirection
    let orderBy: Prisma.InventoryItemOrderByWithRelationInput[] = [];

    switch (sortBy) {
      case 'alphabetical':
        orderBy = [{ name: sortDirection as 'asc' | 'desc' }];
        break;
      case 'expiration':
        orderBy = [
          { expirationDate: sortDirection as 'asc' | 'desc' },
          { name: 'asc' },
        ];
        break;
      case 'quantity':
        orderBy = [{ quantity: sortDirection as 'asc' | 'desc' }];
        break;
      case 'recently_added':
      default:
        orderBy = [{ createdAt: sortDirection as 'asc' | 'desc' }];
        break;
    }

    const items = await prisma.inventoryItem.findMany({
      where,
      orderBy,
      include: {
        addedByUser: {
          select: { name: true },
        },
      },
    });

    return NextResponse.json(items);
  } catch (error) {
    console.error('Error fetching inventory items:', error);
    return NextResponse.json(
      { error: 'Internal server error' },
      { status: 500 }
    );
  }
}

export async function POST(request: NextRequest) {
  try {
    const session = await auth();
    if (!session?.user?.householdId) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }

    const body = await request.json();
    const validatedData = CreateInventoryItemSchema.parse(body);

    // Check for duplicate entries
    const existingItem = await prisma.inventoryItem.findFirst({
      where: {
        name: {
          equals: validatedData.name,
          mode: 'insensitive',
        },
        householdId: session.user.householdId,
      },
    });

    if (existingItem) {
      return NextResponse.json(
        { error: 'Item with this name already exists' },
        { status: 400 }
      );
    }

    const createData: Prisma.InventoryItemUncheckedCreateInput = {
      ...validatedData,
      householdId: session.user.householdId,
      addedBy: session.user.id,
      estimatedCost: validatedData.estimatedCost ?? null,
    };

    const item = await prisma.inventoryItem.create({
      data: createData,
      include: {
        addedByUser: {
          select: { name: true },
        },
      },
    });

    return NextResponse.json(item, { status: 201 });
  } catch (error) {
    console.error('Error creating inventory item:', error);

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

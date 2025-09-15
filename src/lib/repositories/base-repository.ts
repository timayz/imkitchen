import { PrismaClient } from '@prisma/client';
import { logger, logDatabaseOperation } from '../logger';

// Base repository interface
export interface IRepository<T, TCreate, TUpdate> {
  findById(id: string): Promise<T | null>;
  findMany(options?: Record<string, unknown>): Promise<T[]>;
  create(data: TCreate): Promise<T>;
  update(id: string, data: TUpdate): Promise<T>;
  delete(id: string): Promise<void>;
  count(options?: Record<string, unknown>): Promise<number>;
}

// Base repository implementation

export abstract class BaseRepository<T, TCreate, TUpdate>
  implements IRepository<T, TCreate, TUpdate>
{
  protected db: PrismaClient;
  protected modelName: string;

  constructor(db: PrismaClient, modelName: string) {
    this.db = db;
    this.modelName = modelName;
  }

  // Abstract method to get the Prisma model delegate
  protected abstract getModel(): {
    findUnique: (args: Record<string, unknown>) => Promise<T | null>;
    findMany: (args?: Record<string, unknown>) => Promise<T[]>;
    create: (args: Record<string, unknown>) => Promise<T>;
    update: (args: Record<string, unknown>) => Promise<T>;
    delete: (args: Record<string, unknown>) => Promise<T>;
    count: (args?: Record<string, unknown>) => Promise<number>;
    createMany: (args: Record<string, unknown>) => Promise<{ count: number }>;
    updateMany: (args: Record<string, unknown>) => Promise<{ count: number }>;
    deleteMany: (args: Record<string, unknown>) => Promise<{ count: number }>;
    findFirst: (args: Record<string, unknown>) => Promise<T | null>;
    upsert: (args: Record<string, unknown>) => Promise<T>;
  };

  // Find by ID with logging
  async findById(id: string): Promise<T | null> {
    return logDatabaseOperation(
      'findById',
      this.modelName,
      async () => {
        const result = await this.getModel().findUnique({
          where: { id },
        });

        if (!result) {
          logger.debug(`${this.modelName} not found`, { id });
        }


        return result;
      },
      { id }
    );
  }

  // Find many with options
  async findMany(options: Record<string, unknown> = {}): Promise<T[]> {
    return logDatabaseOperation(
      'findMany',
      this.modelName,
      async () => {
        return this.getModel().findMany(options);
      },
      { options }
    );
  }

  // Create new record
  async create(data: TCreate): Promise<T> {
    return logDatabaseOperation('create', this.modelName, async () => {
      return this.getModel().create({
        data,
      });
    });
  }

  // Update existing record
  async update(id: string, data: TUpdate): Promise<T> {
    return logDatabaseOperation(
      'update',
      this.modelName,
      async () => {
        return this.getModel().update({
          where: { id },
          data,
        });
      },
      { id }
    );
  }

  // Delete record
  async delete(id: string): Promise<void> {
    return logDatabaseOperation(
      'delete',
      this.modelName,
      async () => {
        await this.getModel().delete({
          where: { id },
        });
      },
      { id }
    );
  }

  // Count records
  async count(options: Record<string, unknown> = {}): Promise<number> {
    return logDatabaseOperation(
      'count',
      this.modelName,
      async () => {
        return this.getModel().count(options);
      },
      { options }
    );
  }

  // Batch operations
  async createMany(data: TCreate[]): Promise<{ count: number }> {
    return logDatabaseOperation(
      'createMany',
      this.modelName,
      async () => {
        return this.getModel().createMany({
          data,
          skipDuplicates: true,
        });
      },
      { count: data.length }
    );
  }

  // Find with pagination
  async findWithPagination(options: {
    where?: Record<string, unknown>;
    orderBy?: Record<string, unknown>;
    page?: number;
    limit?: number;
    include?: Record<string, unknown>;
  }) {
    const page = options.page || 1;
    const limit = options.limit || 10;
    const skip = (page - 1) * limit;

    return logDatabaseOperation(
      'findWithPagination',
      this.modelName,
      async () => {
        const [data, total] = await Promise.all([
          this.getModel().findMany({
            where: options.where,
            orderBy: options.orderBy,
            skip,
            take: limit,
            include: options.include,
          }),
          this.getModel().count({
            where: options.where,
          }),
        ]);

        const totalPages = Math.ceil(total / limit);
        const hasNext = page < totalPages;
        const hasPrev = page > 1;

        return {
          data,
          pagination: {
            page,
            limit,
            total,
            totalPages,
            hasNext,
            hasPrev,
          },
        };
      },
      { page, limit }
    );
  }

  // Transaction support
  async executeInTransaction<R>(
    operation: (
      tx: Omit<
        PrismaClient,
        '$connect' | '$disconnect' | '$on' | '$transaction' | '$extends'
      >
    ) => Promise<R>
  ): Promise<R> {
    return logDatabaseOperation('transaction', this.modelName, async () => {
      return this.db.$transaction(operation);
    });
  }

  // Bulk update
  async updateMany(
    where: Record<string, unknown>,
    data: Record<string, unknown>
  ): Promise<{ count: number }> {

    return logDatabaseOperation(
      'updateMany',
      this.modelName,
      async () => {
        return this.getModel().updateMany({
          where,
          data,
        });
      },
      { where }
    );
  }

  // Bulk delete
  async deleteMany(where: Record<string, unknown>): Promise<{ count: number }> {
    return logDatabaseOperation(
      'deleteMany',
      this.modelName,
      async () => {
        return this.getModel().deleteMany({
          where,
        });
      },
      { where }
    );
  }

  // Check if record exists
  async exists(where: Record<string, unknown>): Promise<boolean> {
    return logDatabaseOperation(
      'exists',
      this.modelName,
      async () => {
        const count = await this.getModel().count({
          where,
          take: 1,
        });
        return count > 0;
      },
      { where }
    );
  }

  // Find first record matching criteria
  async findFirst(options: Record<string, unknown>): Promise<T | null> {
    return logDatabaseOperation(
      'findFirst',
      this.modelName,
      async () => {
        return this.getModel().findFirst(options);
      },
      { options }
    );
  }

  // Upsert operation
  async upsert(
    where: Record<string, unknown>,
    create: TCreate,
    update: TUpdate
  ): Promise<T> {
    return logDatabaseOperation(
      'upsert',
      this.modelName,
      async () => {
        return this.getModel().upsert({
          where,
          create,
          update,
        });
      },
      { where }
    );
  }
}


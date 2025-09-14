import { PrismaClient } from '@prisma/client';
import { databaseConfig } from './config';

declare global {
  var __prisma: PrismaClient | undefined;
}

// Prisma Client Configuration
const prismaConfig = {
  // Connection pooling configuration
  datasources: {
    db: {
      url: databaseConfig.url,
    },
  },
  // Logging configuration based on environment
  // log: databaseConfig.enableLogging 
  //   ? ['query', 'error', 'warn'] as const
  //   : ['error'] as const,
  // Error formatting
  errorFormat: 'pretty' as const,
};

// Database connection with connection pooling
export const db = globalThis.__prisma || new PrismaClient(prismaConfig);

// Prevent multiple instances in development
if (process.env.NODE_ENV === 'development') {
  globalThis.__prisma = db;
}

// Connection health check function
export async function checkDatabaseHealth(): Promise<{ status: string; message: string }> {
  try {
    await db.$queryRaw`SELECT 1`;
    return { status: 'healthy', message: 'Database connection is working' };
  } catch (error) {
    console.error('Database health check failed:', error);
    return { 
      status: 'unhealthy', 
      message: error instanceof Error ? error.message : 'Unknown database error' 
    };
  }
}

// Graceful shutdown handler
export async function disconnectDatabase(): Promise<void> {
  try {
    await db.$disconnect();
    console.log('Database disconnected successfully');
  } catch (error) {
    console.error('Error disconnecting from database:', error);
  }
}

// Connection retry logic with exponential backoff
export async function connectWithRetry(maxRetries: number = 5): Promise<boolean> {
  let retries = 0;
  
  while (retries < maxRetries) {
    try {
      await db.$connect();
      console.log('Database connected successfully');
      return true;
    } catch (error) {
      retries++;
      console.error(`Database connection attempt ${retries} failed:`, error);
      
      if (retries >= maxRetries) {
        console.error('Max connection retries reached');
        return false;
      }
      
      // Exponential backoff: 1s, 2s, 4s, 8s, 16s
      const delay = Math.pow(2, retries - 1) * 1000;
      console.log(`Retrying in ${delay}ms...`);
      await new Promise(resolve => setTimeout(resolve, delay));
    }
  }
  
  return false;
}

// Database provider abstraction layer
export interface DatabaseProvider {
  query: (sql: string, params?: unknown[]) => Promise<unknown>;
  transaction: <T>(fn: (tx: Omit<PrismaClient, '$connect' | '$disconnect' | '$on' | '$transaction' | '$extends'>) => Promise<T>) => Promise<T>;
  disconnect: () => Promise<void>;
}

export class PrismaDatabaseProvider implements DatabaseProvider {
  private client: PrismaClient;
  
  constructor(client: PrismaClient) {
    this.client = client;
  }
  
  async query(sql: string, params?: unknown[]): Promise<unknown> {
    return this.client.$queryRawUnsafe(sql, ...(params || []));
  }
  
  async transaction<T>(fn: (tx: Omit<PrismaClient, '$connect' | '$disconnect' | '$on' | '$transaction' | '$extends'>) => Promise<T>): Promise<T> {
    return this.client.$transaction(fn);
  }
  
  async disconnect(): Promise<void> {
    await this.client.$disconnect();
  }
}

// Export provider instance for vendor independence
export const databaseProvider = new PrismaDatabaseProvider(db);

// Environment-specific connection configuration
export const connectionConfig = {
  development: {
    connectionTimeout: 10000,
    maxConnections: 10,
    idleTimeout: 30000,
  },
  staging: {
    connectionTimeout: 15000,
    maxConnections: 20,
    idleTimeout: 60000,
  },
  production: {
    connectionTimeout: 20000,
    maxConnections: 50,
    idleTimeout: 300000,
  },
};

// Get current environment connection configuration
export function getConnectionConfig() {
  const env = process.env.NODE_ENV || 'development';
  return connectionConfig[env as keyof typeof connectionConfig] || connectionConfig.development;
}
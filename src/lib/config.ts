import { z } from 'zod';

// Environment variable schema validation
const envSchema = z.object({
  // Database Configuration
  DATABASE_URL: z
    .string()
    .url('DATABASE_URL must be a valid PostgreSQL connection string'),

  // Redis Configuration
  REDIS_URL: z
    .string()
    .url('REDIS_URL must be a valid Redis connection string')
    .optional(),

  // Authentication Configuration
  NEXTAUTH_SECRET: z
    .string()
    .min(32, 'NEXTAUTH_SECRET must be at least 32 characters'),
  NEXTAUTH_URL: z.string().url('NEXTAUTH_URL must be a valid URL'),

  // External API Keys
  SPOONACULAR_API_KEY: z.string().optional(),
  OPENAI_API_KEY: z.string().optional(),
  SENDGRID_API_KEY: z.string().optional(),

  // File Storage Configuration
  S3_BUCKET_NAME: z.string().optional(),
  S3_ACCESS_KEY_ID: z.string().optional(),
  S3_SECRET_ACCESS_KEY: z.string().optional(),
  S3_REGION: z.string().optional(),

  // Application Configuration
  NODE_ENV: z
    .enum(['development', 'staging', 'production'])
    .default('development'),
  LOG_LEVEL: z.enum(['debug', 'info', 'warn', 'error']).default('info'),

  // Public Environment Variables (for client-side access)
  NEXT_PUBLIC_APP_URL: z
    .string()
    .url('NEXT_PUBLIC_APP_URL must be a valid URL'),
  NEXT_PUBLIC_API_URL: z
    .string()
    .url('NEXT_PUBLIC_API_URL must be a valid URL'),
});

// Parse and validate environment variables
function parseEnv() {
  try {
    return envSchema.parse(process.env);
  } catch (error) {
    if (error instanceof z.ZodError) {
      const missingVars = error.issues
        .map((err: z.ZodIssue) => `${err.path.join('.')}: ${err.message}`)
        .join('\n');

      throw new Error(`Environment validation failed:\n${missingVars}`);
    }
    throw error;
  }
}

// Export validated environment configuration
export const env = parseEnv();

// Database configuration with environment-specific settings
export const databaseConfig = {
  url: env.DATABASE_URL,
  // Connection pool settings based on environment
  connectionLimit: env.NODE_ENV === 'production' ? 50 : 10,
  connectionTimeout: env.NODE_ENV === 'production' ? 20000 : 10000,
  idleTimeout: env.NODE_ENV === 'production' ? 300000 : 30000,
  // Enable query logging in development
  enableLogging: env.NODE_ENV === 'development',
  logLevel: env.LOG_LEVEL,
};

// Redis configuration
export const redisConfig = {
  url: env.REDIS_URL,
  retryAttempts: 3,
  retryDelay: 1000,
};

// Authentication configuration
export const authConfig = {
  secret: env.NEXTAUTH_SECRET,
  url: env.NEXTAUTH_URL,
  sessionMaxAge: 30 * 24 * 60 * 60, // 30 days
};

// External API configuration
export const apiConfig = {
  spoonacular: {
    apiKey: env.SPOONACULAR_API_KEY,
    baseUrl: 'https://api.spoonacular.com',
    rateLimit: 150, // requests per day for free tier
  },
  openai: {
    apiKey: env.OPENAI_API_KEY,
    baseUrl: 'https://api.openai.com/v1',
  },
  sendgrid: {
    apiKey: env.SENDGRID_API_KEY,
    baseUrl: 'https://api.sendgrid.com/v3',
  },
};

// File storage configuration
export const storageConfig = {
  s3: {
    bucketName: env.S3_BUCKET_NAME,
    accessKeyId: env.S3_ACCESS_KEY_ID,
    secretAccessKey: env.S3_SECRET_ACCESS_KEY,
    region: env.S3_REGION || 'us-east-1',
  },
};

// Application configuration
export const appConfig = {
  env: env.NODE_ENV,
  logLevel: env.LOG_LEVEL,
  publicUrl: env.NEXT_PUBLIC_APP_URL,
  apiUrl: env.NEXT_PUBLIC_API_URL,
  // Feature flags based on environment
  features: {
    enableVoiceCommands: env.NODE_ENV !== 'production' || !!env.OPENAI_API_KEY,
    enableRecipeImport: !!env.SPOONACULAR_API_KEY,
    enableEmailNotifications: !!env.SENDGRID_API_KEY,
    enableFileUploads: !!(env.S3_BUCKET_NAME && env.S3_ACCESS_KEY_ID),
  },
};

// Validation helper for optional configurations
export function validateOptionalConfig<T>(
  config: T | undefined,
  name: string,
  required: boolean = false
): T | null {
  if (required && !config) {
    throw new Error(`Required configuration missing: ${name}`);
  }
  return config || null;
}

// Environment helpers
export const isDevelopment = env.NODE_ENV === 'development';
export const isStaging = env.NODE_ENV === 'staging';
export const isProduction = env.NODE_ENV === 'production';

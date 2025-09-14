import { NextRequest, NextResponse } from 'next/server';
import { Prisma } from '@prisma/client';
import { ZodError, ZodIssue } from 'zod';
import { logger } from '../logger';

// Standard error response format
export interface ErrorResponse {
  error: string;
  message: string;
  statusCode: number;
  timestamp: string;
  path?: string | undefined;
  details?: Record<string, unknown>;
}

// Database-specific error types
export enum DatabaseError {
  CONNECTION_FAILED = 'CONNECTION_FAILED',
  UNIQUE_CONSTRAINT = 'UNIQUE_CONSTRAINT',
  FOREIGN_KEY_CONSTRAINT = 'FOREIGN_KEY_CONSTRAINT',
  NOT_FOUND = 'NOT_FOUND',
  VALIDATION_ERROR = 'VALIDATION_ERROR',
  TIMEOUT = 'TIMEOUT',
  UNKNOWN = 'UNKNOWN',
}

// Error handler middleware for API routes
export function withErrorHandler<T extends unknown[]>(
  handler: (...args: T) => Promise<NextResponse>
) {
  return async (...args: T): Promise<NextResponse> => {
    try {
      return await handler(...args);
    } catch (error) {
      return handleError(error, args[0] as NextRequest);
    }
  };
}

// Main error handling function
export function handleError(error: unknown, request?: NextRequest): NextResponse {
  const timestamp = new Date().toISOString();
  const path = request?.url;

  // Log the error for monitoring
  logger.error('API Error occurred', {
    error: error instanceof Error ? error.message : 'Unknown error',
    stack: error instanceof Error ? error.stack : undefined,
    path,
    timestamp,
  });

  // Handle different error types
  if (error instanceof Prisma.PrismaClientKnownRequestError) {
    return handlePrismaError(error, timestamp, path);
  }

  if (error instanceof Prisma.PrismaClientUnknownRequestError) {
    return handleUnknownPrismaError(error, timestamp, path);
  }

  if (error instanceof Prisma.PrismaClientRustPanicError) {
    return handlePrismaRustPanic(error, timestamp, path);
  }

  if (error instanceof Prisma.PrismaClientInitializationError) {
    return handlePrismaInitializationError(error, timestamp, path);
  }

  if (error instanceof Prisma.PrismaClientValidationError) {
    return handlePrismaValidationError(error, timestamp, path);
  }

  if (error instanceof ZodError) {
    return handleZodError(error, timestamp, path);
  }

  if (error instanceof Error) {
    return handleGenericError(error, timestamp, path);
  }

  // Unknown error type
  return createErrorResponse(
    DatabaseError.UNKNOWN,
    'An unexpected error occurred',
    500,
    timestamp,
    path
  );
}

// Handle Prisma known request errors
function handlePrismaError(
  error: Prisma.PrismaClientKnownRequestError,
  timestamp: string,
  path?: string
): NextResponse {
  switch (error.code) {
    case 'P2002': // Unique constraint failed
      return createErrorResponse(
        DatabaseError.UNIQUE_CONSTRAINT,
        `Unique constraint violation: ${getConstraintField(error)}`,
        409,
        timestamp,
        path,
        { field: getConstraintField(error) }
      );

    case 'P2003': // Foreign key constraint failed
      return createErrorResponse(
        DatabaseError.FOREIGN_KEY_CONSTRAINT,
        'Foreign key constraint violation',
        400,
        timestamp,
        path
      );

    case 'P2025': // Record not found
      return createErrorResponse(
        DatabaseError.NOT_FOUND,
        'Record not found',
        404,
        timestamp,
        path
      );

    case 'P1008': // Operations timed out
      return createErrorResponse(
        DatabaseError.TIMEOUT,
        'Database operation timed out',
        408,
        timestamp,
        path
      );

    case 'P1001': // Can't reach database server
    case 'P1002': // Database server reached but timed out
      return createErrorResponse(
        DatabaseError.CONNECTION_FAILED,
        'Database connection failed',
        503,
        timestamp,
        path
      );

    default:
      logger.error('Unhandled Prisma error code', { code: error.code, error: error.message });
      return createErrorResponse(
        DatabaseError.UNKNOWN,
        'Database operation failed',
        500,
        timestamp,
        path,
        { code: error.code }
      );
  }
}

// Handle unknown Prisma errors
function handleUnknownPrismaError(
  error: Prisma.PrismaClientUnknownRequestError,
  timestamp: string,
  path?: string
): NextResponse {
  logger.error('Unknown Prisma error', { error: error.message });
  
  return createErrorResponse(
    DatabaseError.UNKNOWN,
    'Database operation failed',
    500,
    timestamp,
    path
  );
}

// Handle Prisma Rust panic errors
function handlePrismaRustPanic(
  error: Prisma.PrismaClientRustPanicError,
  timestamp: string,
  path?: string
): NextResponse {
  logger.error('Prisma Rust panic', { error: error.message });
  
  return createErrorResponse(
    DatabaseError.UNKNOWN,
    'Internal database error',
    500,
    timestamp,
    path
  );
}

// Handle Prisma initialization errors
function handlePrismaInitializationError(
  error: Prisma.PrismaClientInitializationError,
  timestamp: string,
  path?: string
): NextResponse {
  logger.error('Prisma initialization error', { error: error.message });
  
  return createErrorResponse(
    DatabaseError.CONNECTION_FAILED,
    'Database initialization failed',
    503,
    timestamp,
    path
  );
}

// Handle Prisma validation errors
function handlePrismaValidationError(
  error: Prisma.PrismaClientValidationError,
  timestamp: string,
  path?: string
): NextResponse {
  logger.error('Prisma validation error', { error: error.message });
  
  return createErrorResponse(
    DatabaseError.VALIDATION_ERROR,
    'Invalid data provided',
    400,
    timestamp,
    path
  );
}

// Handle Zod validation errors
function handleZodError(
  error: ZodError,
  timestamp: string,
  path?: string
): NextResponse {
  const validationErrors = error.issues.map((err: ZodIssue) => ({
    field: err.path.join('.'),
    message: err.message,
  }));

  logger.warn('Validation error', { errors: validationErrors });

  return createErrorResponse(
    DatabaseError.VALIDATION_ERROR,
    'Validation failed',
    400,
    timestamp,
    path,
    { errors: validationErrors }
  );
}

// Handle generic errors
function handleGenericError(
  error: Error,
  timestamp: string,
  path?: string
): NextResponse {
  logger.error('Generic error', { error: error.message, stack: error.stack });
  
  return createErrorResponse(
    DatabaseError.UNKNOWN,
    'Internal server error',
    500,
    timestamp,
    path
  );
}

// Create standardized error response
function createErrorResponse(
  error: DatabaseError,
  message: string,
  statusCode: number,
  timestamp: string,
  path?: string | undefined,
  details?: Record<string, unknown>
): NextResponse {
  const errorResponse: ErrorResponse = {
    error,
    message,
    statusCode,
    timestamp,
    ...(path !== undefined && { path }),
    ...(details !== undefined && { details }),
  };

  return NextResponse.json(errorResponse, { status: statusCode });
}

// Extract constraint field from Prisma error
function getConstraintField(error: Prisma.PrismaClientKnownRequestError): string {
  if (error.meta?.target) {
    const target = error.meta.target as string | string[];
    return Array.isArray(target) ? target.join(', ') : target;
  }
  return 'unknown field';
}

// Utility function for handling async operations with error catching
export async function safeDbOperation<T>(
  operation: () => Promise<T>,
  context?: string
): Promise<{ data?: T; error?: ErrorResponse }> {
  try {
    const data = await operation();
    return { data };
  } catch (error) {
    logger.error(`Database operation failed: ${context}`, { error });
    
    if (error instanceof Prisma.PrismaClientKnownRequestError) {
      const response = handlePrismaError(error, new Date().toISOString());
      return { error: await response.json() };
    }
    
    return {
      error: {
        error: DatabaseError.UNKNOWN,
        message: 'Operation failed',
        statusCode: 500,
        timestamp: new Date().toISOString(),
      },
    };
  }
}

// Connection retry helper with exponential backoff
export async function retryDatabaseOperation<T>(
  operation: () => Promise<T>,
  maxRetries: number = 3,
  baseDelay: number = 1000
): Promise<T> {
  let lastError: Error;
  
  for (let attempt = 1; attempt <= maxRetries; attempt++) {
    try {
      return await operation();
    } catch (error) {
      lastError = error instanceof Error ? error : new Error('Unknown error');
      
      if (attempt === maxRetries) {
        throw lastError;
      }
      
      // Check if error is retryable
      if (!isRetryableError(error)) {
        throw lastError;
      }
      
      const delay = Math.min(baseDelay * Math.pow(2, attempt - 1), 10000);
      logger.warn(`Database operation failed, retrying in ${delay}ms`, {
        attempt,
        maxRetries,
        error: lastError.message,
      });
      
      await new Promise(resolve => setTimeout(resolve, delay));
    }
  }
  
  throw lastError!;
}

// Check if error is retryable
function isRetryableError(error: unknown): boolean {
  if (error instanceof Prisma.PrismaClientKnownRequestError) {
    // Retry on connection issues and timeouts
    return ['P1001', 'P1002', 'P1008', 'P1017'].includes(error.code);
  }
  
  if (error instanceof Prisma.PrismaClientUnknownRequestError) {
    return true; // Retry unknown request errors
  }
  
  return false;
}
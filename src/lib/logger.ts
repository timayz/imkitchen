import winston from 'winston';
import { env, isDevelopment, isProduction } from '@/lib/config';

// Custom log levels for imkitchen application
const customLevels = {
  levels: {
    error: 0,
    warn: 1,
    info: 2,
    http: 3,
    debug: 4,
  },
  colors: {
    error: 'red',
    warn: 'yellow',
    info: 'green',
    http: 'magenta',
    debug: 'white',
  },
};

// Custom format for structured logging
const logFormat = winston.format.combine(
  winston.format.timestamp({
    format: 'YYYY-MM-DD HH:mm:ss',
  }),
  winston.format.errors({ stack: true }),
  winston.format.json(),
  winston.format.printf(info => {
    const { timestamp, level, message, ...meta } = info;

    return JSON.stringify({
      timestamp,
      level,
      message,
      environment: env.NODE_ENV,
      service: 'imkitchen',
      ...meta,
    });
  })
);

// Console format for development
const consoleFormat = winston.format.combine(
  winston.format.colorize({ all: true }),
  winston.format.timestamp({
    format: 'HH:mm:ss',
  }),
  winston.format.errors({ stack: true }),
  winston.format.printf(info => {
    const { timestamp, level, message, ...meta } = info;
    const metaStr =
      Object.keys(meta).length > 0 ? `\n${JSON.stringify(meta, null, 2)}` : '';
    return `${timestamp} [${level}]: ${message}${metaStr}`;
  })
);

// Configure transports based on environment
const transports: winston.transport[] = [];

// Always add console transport
transports.push(
  new winston.transports.Console({
    level: isDevelopment ? 'debug' : 'info',
    format: isDevelopment ? consoleFormat : logFormat,
  })
);

// Add file transports for production
if (isProduction) {
  // Error log file
  transports.push(
    new winston.transports.File({
      filename: 'logs/error.log',
      level: 'error',
      format: logFormat,
      maxsize: 5242880, // 5MB
      maxFiles: 5,
    })
  );

  // Combined log file
  transports.push(
    new winston.transports.File({
      filename: 'logs/combined.log',
      format: logFormat,
      maxsize: 5242880, // 5MB
      maxFiles: 5,
    })
  );
}

// Create the logger instance
export const logger = winston.createLogger({
  levels: customLevels.levels,
  level: env.LOG_LEVEL || 'info',
  format: logFormat,
  transports,
  exitOnError: false,
});

// Add colors to winston
winston.addColors(customLevels.colors);

// Specialized logging functions for different categories

/**
 * Log database operations
 */
export function logDatabaseOperation(
  operation: string,
  model: string,
  duration?: number,
  metadata?: Record<string, unknown>
) {
  logger.info('Database operation', {
    category: 'database',
    operation,
    model,
    duration,
    ...metadata,
  });
}

/**
 * Log API requests
 */
export function logApiRequest(
  method: string,
  path: string,
  statusCode: number,
  duration: number,
  userId?: string,
  metadata?: Record<string, unknown>
) {
  logger.http('API request', {
    category: 'api',
    method,
    path,
    statusCode,
    duration,
    userId,
    ...metadata,
  });
}

/**
 * Log authentication events
 */
export function logAuthEvent(
  event: 'login' | 'logout' | 'register' | 'password_reset' | 'failed_login',
  userId?: string,
  metadata?: Record<string, unknown>
) {
  logger.info('Authentication event', {
    category: 'auth',
    event,
    userId,
    ...metadata,
  });
}

/**
 * Log performance metrics
 */
export function logPerformance(
  operation: string,
  duration: number,
  metadata?: Record<string, unknown>
) {
  logger.info('Performance metric', {
    category: 'performance',
    operation,
    duration,
    ...metadata,
  });
}

/**
 * Log security events
 */
export function logSecurityEvent(
  event: 'rate_limit_exceeded' | 'invalid_token' | 'suspicious_activity',
  severity: 'low' | 'medium' | 'high' = 'medium',
  metadata?: Record<string, unknown>
) {
  logger.warn('Security event', {
    category: 'security',
    event,
    severity,
    ...metadata,
  });
}

/**
 * Log external service interactions
 */
export function logExternalService(
  service: string,
  operation: string,
  success: boolean,
  duration?: number,
  metadata?: Record<string, unknown>
) {
  logger.info('External service call', {
    category: 'external',
    service,
    operation,
    success,
    duration,
    ...metadata,
  });
}

/**
 * Wrapper for logging function execution with automatic timing
 */
export async function withLogging<T>(
  operation: string,
  fn: () => Promise<T> | T,
  metadata?: Record<string, unknown>
): Promise<T> {
  const startTime = Date.now();

  try {
    const result = await fn();
    const duration = Date.now() - startTime;

    logger.info('Operation completed', {
      operation,
      duration,
      success: true,
      ...metadata,
    });

    return result;
  } catch (error) {
    const duration = Date.now() - startTime;

    logger.error('Operation failed', {
      operation,
      duration,
      success: false,
      error: error instanceof Error ? error.message : 'Unknown error',
      stack: error instanceof Error ? error.stack : undefined,
      ...metadata,
    });

    throw error;
  }
}

/**
 * Create a child logger with additional context
 */
export function createChildLogger(context: Record<string, unknown>) {
  return logger.child(context);
}

/**
 * Flush all log transports (useful for serverless)
 */
export function flushLogs(): Promise<void> {
  return new Promise(resolve => {
    logger.on('finish', resolve);
    logger.end();
  });
}

// Export the main logger as default
export default logger;

import { env } from './config';

// Log levels
export enum LogLevel {
  DEBUG = 'debug',
  INFO = 'info',
  WARN = 'warn',
  ERROR = 'error',
}

// Log context interface
export interface LogContext {
  userId?: string;
  householdId?: string;
  operation?: string;
  duration?: number;
  error?: string;
  stack?: string | undefined;
  path?: string | undefined;
  method?: string;
  statusCode?: number;
  [key: string]: unknown;
}

// Logger class for structured logging
class Logger {
  private level: LogLevel;
  private isDevelopment: boolean;

  constructor() {
    this.level = this.parseLogLevel(env.LOG_LEVEL);
    this.isDevelopment = env.NODE_ENV === 'development';
  }

  // Parse log level from string
  private parseLogLevel(level: string): LogLevel {
    switch (level.toLowerCase()) {
      case 'debug':
        return LogLevel.DEBUG;
      case 'info':
        return LogLevel.INFO;
      case 'warn':
        return LogLevel.WARN;
      case 'error':
        return LogLevel.ERROR;
      default:
        return LogLevel.INFO;
    }
  }

  // Check if level should be logged
  private shouldLog(level: LogLevel): boolean {
    const levels = [LogLevel.DEBUG, LogLevel.INFO, LogLevel.WARN, LogLevel.ERROR];
    const currentLevelIndex = levels.indexOf(this.level);
    const targetLevelIndex = levels.indexOf(level);
    return targetLevelIndex >= currentLevelIndex;
  }

  // Format log entry
  private formatLog(level: LogLevel, message: string, context?: LogContext) {
    const timestamp = new Date().toISOString();
    const logEntry = {
      timestamp,
      level,
      message,
      ...context,
      environment: env.NODE_ENV,
    };

    if (this.isDevelopment) {
      // Pretty print for development
      console.log(`[${timestamp}] ${level.toUpperCase()}: ${message}`);
      if (context && Object.keys(context).length > 0) {
        console.log('Context:', context);
      }
    } else {
      // Structured JSON for production
      console.log(JSON.stringify(logEntry));
    }

    return logEntry;
  }

  // Debug logging
  debug(message: string, context?: LogContext): void {
    if (this.shouldLog(LogLevel.DEBUG)) {
      this.formatLog(LogLevel.DEBUG, message, context);
    }
  }

  // Info logging
  info(message: string, context?: LogContext): void {
    if (this.shouldLog(LogLevel.INFO)) {
      this.formatLog(LogLevel.INFO, message, context);
    }
  }

  // Warning logging
  warn(message: string, context?: LogContext): void {
    if (this.shouldLog(LogLevel.WARN)) {
      this.formatLog(LogLevel.WARN, message, context);
    }
  }

  // Error logging
  error(message: string, context?: LogContext): void {
    if (this.shouldLog(LogLevel.ERROR)) {
      this.formatLog(LogLevel.ERROR, message, context);
    }
  }

  // Database operation logging
  dbOperation(
    operation: string,
    model: string,
    duration?: number,
    context?: Omit<LogContext, 'operation' | 'duration'>
  ) {
    this.info(`Database operation: ${operation} on ${model}`, {
      operation,
      model,
      ...(duration !== undefined && { duration }),
      ...context,
    });
  }

  // Database error logging
  dbError(
    operation: string,
    model: string,
    error: string,
    context?: LogContext
  ) {
    this.error(`Database error: ${operation} on ${model}`, {
      operation,
      model,
      error,
      ...context,
    });
  }

  // API request logging
  apiRequest(
    method: string,
    path: string,
    statusCode: number,
    duration: number,
    context?: LogContext
  ) {
    const level = statusCode >= 400 ? LogLevel.WARN : LogLevel.INFO;
    const message = `${method} ${path} - ${statusCode} (${duration}ms)`;
    
    if (level === LogLevel.WARN) {
      this.warn(message, { method, path, statusCode, duration, ...context });
    } else {
      this.info(message, { method, path, statusCode, duration, ...context });
    }
  }

  // Authentication logging
  authEvent(
    event: string,
    userId?: string,
    success: boolean = true,
    context?: Omit<LogContext, 'userId' | 'event'>
  ) {
    const level = success ? LogLevel.INFO : LogLevel.WARN;
    const message = `Auth ${event}: ${success ? 'success' : 'failed'}`;
    
    const logContext = {
      event,
      success,
      ...(userId !== undefined && { userId }),
      ...context,
    };
    
    if (level === LogLevel.WARN) {
      this.warn(message, logContext);
    } else {
      this.info(message, logContext);
    }
  }

  // Performance monitoring
  performance(
    operation: string,
    duration: number,
    threshold: number = 1000,
    context?: LogContext
  ) {
    const level = duration > threshold ? LogLevel.WARN : LogLevel.DEBUG;
    const message = `Performance: ${operation} took ${duration}ms`;
    
    if (level === LogLevel.WARN) {
      this.warn(message, { operation, duration, threshold, ...context });
    } else {
      this.debug(message, { operation, duration, threshold, ...context });
    }
  }

  // Security event logging
  security(
    event: string,
    severity: 'low' | 'medium' | 'high' | 'critical',
    context?: LogContext
  ) {
    const level = severity === 'critical' || severity === 'high' 
      ? LogLevel.ERROR 
      : LogLevel.WARN;
    
    const message = `Security ${severity}: ${event}`;
    
    if (level === LogLevel.ERROR) {
      this.error(message, { event, severity, ...context });
    } else {
      this.warn(message, { event, severity, ...context });
    }
  }
}

// Create singleton logger instance
export const logger = new Logger();

// Utility function to measure execution time
export async function withLogging<T>(
  operation: string,
  fn: () => Promise<T>,
  context?: LogContext
): Promise<T> {
  const startTime = Date.now();
  
  try {
    logger.debug(`Starting ${operation}`, context);
    const result = await fn();
    const duration = Date.now() - startTime;
    
    logger.debug(`Completed ${operation}`, { ...context, duration });
    logger.performance(operation, duration);
    
    return result;
  } catch (error) {
    const duration = Date.now() - startTime;
    const errorMessage = error instanceof Error ? error.message : 'Unknown error';
    
    logger.error(`Failed ${operation}`, {
      ...context,
      duration,
      error: errorMessage,
      stack: error instanceof Error ? error.stack : undefined,
    });
    
    throw error;
  }
}

// Database operation logger with automatic timing
export async function logDatabaseOperation<T>(
  operation: string,
  model: string,
  fn: () => Promise<T>,
  context?: LogContext
): Promise<T> {
  const startTime = Date.now();
  
  try {
    const result = await fn();
    const duration = Date.now() - startTime;
    
    logger.dbOperation(operation, model, duration, context);
    
    // Log slow queries
    if (duration > 1000) {
      logger.performance(`Slow database query: ${operation} on ${model}`, duration);
    }
    
    return result;
  } catch (error) {
    const duration = Date.now() - startTime;
    const errorMessage = error instanceof Error ? error.message : 'Unknown error';
    
    logger.dbError(operation, model, errorMessage, { ...context, duration });
    throw error;
  }
}

// Express/Next.js request logger middleware
export function createRequestLogger() {
  return (req: any, res: any, next: any) => {
    const startTime = Date.now();
    const { method, url } = req;
    
    // Log request start
    logger.debug(`${method} ${url} - Request started`);
    
    // Override res.end to log completion
    const originalEnd = res.end;
    res.end = function(...args: any[]) {
      const duration = Date.now() - startTime;
      logger.apiRequest(method, url, res.statusCode, duration);
      
      originalEnd.apply(this, args);
    };
    
    next();
  };
}

// Export logger for direct use
export default logger;
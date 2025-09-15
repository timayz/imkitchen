import { db } from '@/lib/db';
import { databaseConfig, redisConfig, appConfig } from '@/lib/config';

export interface HealthCheckResult {
  status: 'healthy' | 'unhealthy' | 'degraded';
  message: string;
  timestamp: string;
  responseTime?: number;
  details?: Record<string, unknown>;
}

export interface DetailedHealthCheck {
  status: 'healthy' | 'unhealthy' | 'degraded';
  timestamp: string;
  version: string;
  environment: string;
  uptime: number;
  services: {
    database: HealthCheckResult;
    redis?: HealthCheckResult;
    external_apis?: HealthCheckResult;
  };
  system: {
    memory: {
      used: number;
      total: number;
      percentage: number;
    };
    cpu?: {
      usage: number;
    };
  };
}

/**
 * Basic health check - returns simple status
 */
export async function basicHealthCheck(): Promise<HealthCheckResult> {
  const startTime = Date.now();

  try {
    // Simple application check
    const timestamp = new Date().toISOString();
    const responseTime = Date.now() - startTime;

    return {
      status: 'healthy',
      message: 'Application is running',
      timestamp,
      responseTime,
      details: {
        environment: appConfig.env,
        version: process.env.npm_package_version || '1.0.0',
      },
    };
  } catch (error) {
    return {
      status: 'unhealthy',
      message: 'Application health check failed',
      timestamp: new Date().toISOString(),
      responseTime: Date.now() - startTime,
      details: {
        error: error instanceof Error ? error.message : 'Unknown error',
      },
    };
  }
}

/**
 * Check database connectivity and basic query performance
 */
export async function checkDatabase(): Promise<HealthCheckResult> {
  const startTime = Date.now();

  try {
    // Test database connection with a simple query
    await db.$queryRaw`SELECT 1 as health_check`;

    const responseTime = Date.now() - startTime;

    return {
      status: responseTime < 1000 ? 'healthy' : 'degraded',
      message:
        responseTime < 1000
          ? 'Database connection healthy'
          : 'Database responding slowly',
      timestamp: new Date().toISOString(),
      responseTime,
      details: {
        connectionLimit: databaseConfig.connectionLimit,
        slowQuery: responseTime > 1000,
      },
    };
  } catch (error) {
    return {
      status: 'unhealthy',
      message: 'Database connection failed',
      timestamp: new Date().toISOString(),
      responseTime: Date.now() - startTime,
      details: {
        error:
          error instanceof Error ? error.message : 'Database connection error',
      },
    };
  }
}

/**
 * Check Redis connectivity (if configured)
 */
export async function checkRedis(): Promise<HealthCheckResult> {
  const startTime = Date.now();

  if (!redisConfig.url) {
    return {
      status: 'healthy',
      message: 'Redis not configured (optional)',
      timestamp: new Date().toISOString(),
      responseTime: 0,
    };
  }

  try {
    // Import Redis client dynamically to avoid errors if not configured
    const { Redis } = await import('ioredis');
    const redis = new Redis(redisConfig.url, {
      maxRetriesPerRequest: 1,
      connectTimeout: 2000,
    });

    await redis.ping();
    await redis.disconnect();

    const responseTime = Date.now() - startTime;

    return {
      status: 'healthy',
      message: 'Redis connection healthy',
      timestamp: new Date().toISOString(),
      responseTime,
    };
  } catch (error) {
    return {
      status: 'degraded',
      message: 'Redis connection failed (non-critical)',
      timestamp: new Date().toISOString(),
      responseTime: Date.now() - startTime,
      details: {
        error:
          error instanceof Error ? error.message : 'Redis connection error',
      },
    };
  }
}

/**
 * Check external API connectivity
 */
export async function checkExternalAPIs(): Promise<HealthCheckResult> {
  const startTime = Date.now();
  const checks = [];

  try {
    // Check critical external APIs if configured
    const { apiConfig } = await import('@/lib/config');

    // Check Spoonacular API if configured
    if (apiConfig.spoonacular.apiKey) {
      try {
        const response = await fetch(
          `${apiConfig.spoonacular.baseUrl}/recipes/complexSearch?query=test&number=1&apiKey=${apiConfig.spoonacular.apiKey}`,
          {
            signal: AbortSignal.timeout(5000),
          }
        );
        checks.push({
          service: 'spoonacular',
          status: response.ok ? 'healthy' : 'degraded',
          responseTime: Date.now() - startTime,
        });
      } catch {
        checks.push({
          service: 'spoonacular',
          status: 'degraded',
          responseTime: Date.now() - startTime,
        });
      }
    }

    const allHealthy = checks.every(check => check.status === 'healthy');
    const anyUnhealthy = checks.some(check => check.status === 'unhealthy');

    return {
      status: anyUnhealthy ? 'degraded' : allHealthy ? 'healthy' : 'degraded',
      message: `External APIs: ${checks.length} checked`,
      timestamp: new Date().toISOString(),
      responseTime: Date.now() - startTime,
      details: { checks },
    };
  } catch (error) {
    return {
      status: 'degraded',
      message: 'External API checks failed (non-critical)',
      timestamp: new Date().toISOString(),
      responseTime: Date.now() - startTime,
      details: {
        error:
          error instanceof Error ? error.message : 'External API check error',
      },
    };
  }
}

/**
 * Get system metrics
 */
export function getSystemMetrics() {
  const memUsage = process.memoryUsage();

  return {
    memory: {
      used: Math.round(memUsage.heapUsed / 1024 / 1024), // MB
      total: Math.round(memUsage.heapTotal / 1024 / 1024), // MB
      percentage: Math.round((memUsage.heapUsed / memUsage.heapTotal) * 100),
    },
    uptime: Math.round(process.uptime()),
  };
}

/**
 * Comprehensive health check with all services and system metrics
 */
export async function detailedHealthCheck(): Promise<DetailedHealthCheck> {
  try {
    // Run all health checks in parallel
    const [databaseCheck, redisCheck, externalAPICheck] = await Promise.all([
      checkDatabase(),
      checkRedis(),
      checkExternalAPIs(),
    ]);

    const systemMetrics = getSystemMetrics();

    // Determine overall status
    const services = [databaseCheck, redisCheck, externalAPICheck];
    const hasUnhealthy = services.some(
      service => service.status === 'unhealthy'
    );
    const hasDegraded = services.some(service => service.status === 'degraded');

    let overallStatus: 'healthy' | 'unhealthy' | 'degraded' = 'healthy';
    if (hasUnhealthy) {
      overallStatus = 'unhealthy';
    } else if (hasDegraded) {
      overallStatus = 'degraded';
    }

    return {
      status: overallStatus,
      timestamp: new Date().toISOString(),
      version: process.env.npm_package_version || '1.0.0',
      environment: appConfig.env,
      uptime: systemMetrics.uptime,
      services: {
        database: databaseCheck,
        redis: redisCheck,
        external_apis: externalAPICheck,
      },
      system: {
        memory: systemMetrics.memory,
      },
    };
  } catch {
    return {
      status: 'unhealthy',
      timestamp: new Date().toISOString(),
      version: process.env.npm_package_version || '1.0.0',
      environment: appConfig.env,
      uptime: Math.round(process.uptime()),
      services: {
        database: {
          status: 'unhealthy',
          message: 'Health check failed',
          timestamp: new Date().toISOString(),
        },
      },
      system: getSystemMetrics(),
    };
  }
}

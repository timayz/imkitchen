import { NextResponse } from 'next/server';
import { detailedHealthCheck } from '@/lib/health-checks';

/**
 * Detailed health check endpoint
 * GET /api/health/detailed
 *
 * Returns comprehensive health status including all services and system metrics
 * This endpoint may take longer to respond as it checks multiple services
 * Should be used for monitoring dashboards and troubleshooting
 */
export async function GET() {
  try {
    const healthResult = await detailedHealthCheck();

    // Return appropriate HTTP status based on overall health
    let statusCode = 200;
    if (healthResult.status === 'unhealthy') {
      statusCode = 503;
    } else if (healthResult.status === 'degraded') {
      statusCode = 207; // Multi-Status - some services may be degraded
    }

    return NextResponse.json(healthResult, {
      status: statusCode,
      headers: {
        'Cache-Control': 'no-cache, no-store, must-revalidate',
        'X-Health-Status': healthResult.status,
        'X-Response-Time': Date.now().toString(),
      },
    });
  } catch (error) {
    console.error('Detailed health check failed:', error);

    const errorResponse = {
      status: 'unhealthy',
      message: 'Detailed health check failed',
      timestamp: new Date().toISOString(),
      version: process.env.npm_package_version || '1.0.0',
      environment: process.env.NODE_ENV || 'unknown',
      uptime: Math.round(process.uptime()),
      services: {
        database: {
          status: 'unknown',
          message: 'Could not check database health',
          timestamp: new Date().toISOString(),
        },
      },
      system: {
        memory: {
          used: 0,
          total: 0,
          percentage: 0,
        },
      },
      error: {
        message: error instanceof Error ? error.message : 'Unknown error',
        stack:
          process.env.NODE_ENV === 'development' && error instanceof Error
            ? error.stack
            : undefined,
      },
    };

    return NextResponse.json(errorResponse, {
      status: 503,
      headers: {
        'Cache-Control': 'no-cache, no-store, must-revalidate',
        'X-Health-Status': 'unhealthy',
      },
    });
  }
}

// Support OPTIONS for CORS preflight in monitoring tools
export async function OPTIONS() {
  return new Response(null, {
    status: 200,
    headers: {
      'Access-Control-Allow-Origin': '*',
      'Access-Control-Allow-Methods': 'GET, HEAD, OPTIONS',
      'Access-Control-Allow-Headers': 'Content-Type, Authorization',
      'Access-Control-Max-Age': '86400',
    },
  });
}

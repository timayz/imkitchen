import { NextResponse } from 'next/server';
import { basicHealthCheck } from '@/lib/health-checks';

/**
 * Basic health check endpoint
 * GET /api/health
 *
 * Returns simple health status for load balancers and monitoring systems
 * This endpoint should respond quickly (< 2 seconds) and not require authentication
 */
export async function GET() {
  try {
    const healthResult = await basicHealthCheck();

    // Return appropriate HTTP status based on health check result
    const statusCode = healthResult.status === 'healthy' ? 200 : 503;

    return NextResponse.json(
      {
        status: healthResult.status,
        message: healthResult.message,
        timestamp: healthResult.timestamp,
        responseTime: healthResult.responseTime,
      },
      { status: statusCode }
    );
  } catch (error) {
    console.error('Health check failed:', error);

    return NextResponse.json(
      {
        status: 'unhealthy',
        message: 'Health check failed',
        timestamp: new Date().toISOString(),
        error: error instanceof Error ? error.message : 'Unknown error',
      },
      { status: 503 }
    );
  }
}

// Support HEAD requests for simple connectivity checks
export async function HEAD() {
  try {
    const healthResult = await basicHealthCheck();
    const statusCode = healthResult.status === 'healthy' ? 200 : 503;

    return new Response(null, {
      status: statusCode,
      headers: {
        'Content-Type': 'application/json',
        'Cache-Control': 'no-cache, no-store, must-revalidate',
        'X-Health-Status': healthResult.status,
      },
    });
  } catch {
    return new Response(null, {
      status: 503,
      headers: {
        'Content-Type': 'application/json',
        'Cache-Control': 'no-cache, no-store, must-revalidate',
        'X-Health-Status': 'unhealthy',
      },
    });
  }
}

/**
 * @jest-environment node
 */

import { GET, OPTIONS } from '@/app/api/health/detailed/route';
import { detailedHealthCheck } from '@/lib/health-checks';

// Mock health check functions
jest.mock('@/lib/health-checks');

const mockDetailedHealthCheck = detailedHealthCheck as jest.MockedFunction<
  typeof detailedHealthCheck
>;

describe('/api/health/detailed', () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });

  describe('GET /api/health/detailed', () => {
    it('returns 200 status when all services are healthy', async () => {
      const mockHealthResult = {
        status: 'healthy' as const,
        timestamp: new Date().toISOString(),
        version: '1.0.0',
        environment: 'test',
        uptime: 3600,
        services: {
          database: {
            status: 'healthy' as const,
            message: 'Database connection healthy',
            timestamp: new Date().toISOString(),
            responseTime: 50,
          },
          redis: {
            status: 'healthy' as const,
            message: 'Redis connection healthy',
            timestamp: new Date().toISOString(),
            responseTime: 10,
          },
          external_apis: {
            status: 'healthy' as const,
            message: 'External APIs: 0 checked',
            timestamp: new Date().toISOString(),
            responseTime: 0,
          },
        },
        system: {
          memory: {
            used: 100,
            total: 500,
            percentage: 20,
          },
        },
      };

      mockDetailedHealthCheck.mockResolvedValue(mockHealthResult);

      const response = await GET();
      const data = await response.json();

      expect(response.status).toBe(200);
      expect(data.status).toBe('healthy');
      expect(data.services.database.status).toBe('healthy');
      expect(data.system.memory.percentage).toBe(20);
      expect(response.headers.get('X-Health-Status')).toBe('healthy');
    });

    it('returns 207 status when some services are degraded', async () => {
      const mockHealthResult = {
        status: 'degraded' as const,
        timestamp: new Date().toISOString(),
        version: '1.0.0',
        environment: 'test',
        uptime: 3600,
        services: {
          database: {
            status: 'healthy' as const,
            message: 'Database connection healthy',
            timestamp: new Date().toISOString(),
            responseTime: 50,
          },
          redis: {
            status: 'degraded' as const,
            message: 'Redis responding slowly',
            timestamp: new Date().toISOString(),
            responseTime: 2000,
          },
          external_apis: {
            status: 'healthy' as const,
            message: 'External APIs: 0 checked',
            timestamp: new Date().toISOString(),
            responseTime: 0,
          },
        },
        system: {
          memory: {
            used: 400,
            total: 500,
            percentage: 80,
          },
        },
      };

      mockDetailedHealthCheck.mockResolvedValue(mockHealthResult);

      const response = await GET();
      const data = await response.json();

      expect(response.status).toBe(207); // Multi-Status
      expect(data.status).toBe('degraded');
      expect(data.services.redis.status).toBe('degraded');
      expect(response.headers.get('X-Health-Status')).toBe('degraded');
    });

    it('returns 503 status when critical services are unhealthy', async () => {
      const mockHealthResult = {
        status: 'unhealthy' as const,
        timestamp: new Date().toISOString(),
        version: '1.0.0',
        environment: 'test',
        uptime: 3600,
        services: {
          database: {
            status: 'unhealthy' as const,
            message: 'Database connection failed',
            timestamp: new Date().toISOString(),
            responseTime: 5000,
            details: {
              error: 'Connection timeout',
            },
          },
        },
        system: {
          memory: {
            used: 100,
            total: 500,
            percentage: 20,
          },
        },
      };

      mockDetailedHealthCheck.mockResolvedValue(mockHealthResult);

      const response = await GET();
      const data = await response.json();

      expect(response.status).toBe(503);
      expect(data.status).toBe('unhealthy');
      expect(data.services.database.status).toBe('unhealthy');
      expect(data.services.database.details.error).toBe('Connection timeout');
    });

    it('returns 503 status when detailed health check throws error', async () => {
      mockDetailedHealthCheck.mockRejectedValue(
        new Error('Health check system failure')
      );

      const response = await GET();
      const data = await response.json();

      expect(response.status).toBe(503);
      expect(data.status).toBe('unhealthy');
      expect(data.message).toBe('Detailed health check failed');
      expect(data.error.message).toBe('Health check system failure');
      expect(response.headers.get('X-Health-Status')).toBe('unhealthy');
    });

    it('includes system metrics in response', async () => {
      const mockHealthResult = {
        status: 'healthy' as const,
        timestamp: new Date().toISOString(),
        version: '1.0.0',
        environment: 'test',
        uptime: 7200,
        services: {
          database: {
            status: 'healthy' as const,
            message: 'Database connection healthy',
            timestamp: new Date().toISOString(),
            responseTime: 50,
          },
        },
        system: {
          memory: {
            used: 250,
            total: 1000,
            percentage: 25,
          },
        },
      };

      mockDetailedHealthCheck.mockResolvedValue(mockHealthResult);

      const response = await GET();
      const data = await response.json();

      expect(data.uptime).toBe(7200);
      expect(data.version).toBe('1.0.0');
      expect(data.environment).toBe('test');
      expect(data.system.memory.used).toBe(250);
      expect(data.system.memory.total).toBe(1000);
      expect(data.system.memory.percentage).toBe(25);
    });
  });

  describe('OPTIONS /api/health/detailed', () => {
    it('returns CORS headers for preflight requests', async () => {
      const response = await OPTIONS();

      expect(response.status).toBe(200);
      expect(response.headers.get('Access-Control-Allow-Origin')).toBe('*');
      expect(response.headers.get('Access-Control-Allow-Methods')).toBe(
        'GET, HEAD, OPTIONS'
      );
      expect(response.headers.get('Access-Control-Allow-Headers')).toBe(
        'Content-Type, Authorization'
      );
      expect(response.headers.get('Access-Control-Max-Age')).toBe('86400');
    });
  });
});

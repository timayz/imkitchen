/**
 * @jest-environment node
 */

import { GET, HEAD } from '@/app/api/health/route';
import { basicHealthCheck } from '@/lib/health-checks';

// Mock health check functions
jest.mock('@/lib/health-checks');

const mockBasicHealthCheck = basicHealthCheck as jest.MockedFunction<
  typeof basicHealthCheck
>;

describe('/api/health', () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });

  describe('GET /api/health', () => {
    it('returns 200 status when application is healthy', async () => {
      const mockHealthResult = {
        status: 'healthy' as const,
        message: 'Application is running',
        timestamp: new Date().toISOString(),
        responseTime: 50,
      };

      mockBasicHealthCheck.mockResolvedValue(mockHealthResult);

      const response = await GET();
      const data = await response.json();

      expect(response.status).toBe(200);
      expect(data).toEqual({
        status: 'healthy',
        message: 'Application is running',
        timestamp: mockHealthResult.timestamp,
        responseTime: 50,
      });
    });

    it('returns 503 status when application is unhealthy', async () => {
      const mockHealthResult = {
        status: 'unhealthy' as const,
        message: 'Database connection failed',
        timestamp: new Date().toISOString(),
        responseTime: 1000,
      };

      mockBasicHealthCheck.mockResolvedValue(mockHealthResult);

      const response = await GET();
      const data = await response.json();

      expect(response.status).toBe(503);
      expect(data.status).toBe('unhealthy');
      expect(data.message).toBe('Database connection failed');
    });

    it('returns 503 status when health check throws error', async () => {
      mockBasicHealthCheck.mockRejectedValue(new Error('Health check failed'));

      const response = await GET();
      const data = await response.json();

      expect(response.status).toBe(503);
      expect(data.status).toBe('unhealthy');
      expect(data.message).toBe('Health check failed');
      expect(data.error).toBe('Health check failed');
    });

    it('returns degraded status with 503 when system is degraded', async () => {
      const mockHealthResult = {
        status: 'degraded' as const,
        message: 'Some services responding slowly',
        timestamp: new Date().toISOString(),
        responseTime: 2000,
      };

      mockBasicHealthCheck.mockResolvedValue(mockHealthResult);

      const response = await GET();
      const data = await response.json();

      expect(response.status).toBe(503);
      expect(data.status).toBe('degraded');
      expect(data.responseTime).toBe(2000);
    });
  });

  describe('HEAD /api/health', () => {
    it('returns 200 status with headers when healthy', async () => {
      const mockHealthResult = {
        status: 'healthy' as const,
        message: 'Application is running',
        timestamp: new Date().toISOString(),
        responseTime: 50,
      };

      mockBasicHealthCheck.mockResolvedValue(mockHealthResult);

      const response = await HEAD();

      expect(response.status).toBe(200);
      expect(response.headers.get('X-Health-Status')).toBe('healthy');
      expect(response.headers.get('Content-Type')).toBe('application/json');
      expect(response.headers.get('Cache-Control')).toBe(
        'no-cache, no-store, must-revalidate'
      );
    });

    it('returns 503 status with unhealthy header when degraded', async () => {
      const mockHealthResult = {
        status: 'degraded' as const,
        message: 'Some issues detected',
        timestamp: new Date().toISOString(),
        responseTime: 1500,
      };

      mockBasicHealthCheck.mockResolvedValue(mockHealthResult);

      const response = await HEAD();

      expect(response.status).toBe(503);
      expect(response.headers.get('X-Health-Status')).toBe('degraded');
    });

    it('returns 503 status when health check throws error', async () => {
      mockBasicHealthCheck.mockRejectedValue(new Error('Health check failed'));

      const response = await HEAD();

      expect(response.status).toBe(503);
      expect(response.headers.get('X-Health-Status')).toBe('unhealthy');
    });
  });
});

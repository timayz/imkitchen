/**
 * @jest-environment node
 */

import { NextRequest, NextResponse } from 'next/server';
import { POST } from '@/app/api/auth/reset-password/route';
import { AuthService } from '@/lib/services/auth-service';
import { EmailService } from '@/lib/services/email-service';
import { passwordResetRateLimiter } from '@/lib/middleware/rate-limiter';
import { Language } from '@prisma/client';

// Mock dependencies
jest.mock('@/lib/services/auth-service');
jest.mock('@/lib/services/email-service');
jest.mock('@/lib/middleware/rate-limiter');
jest.mock('@/lib/db', () => ({
  prisma: {
    passwordResetToken: {
      create: jest.fn(),
    },
  },
}));

const mockAuthService = AuthService as jest.Mocked<typeof AuthService>;
const mockEmailService = EmailService as jest.Mocked<typeof EmailService>;
const mockRateLimiter = passwordResetRateLimiter as jest.MockedFunction<
  typeof passwordResetRateLimiter
>;

describe('/api/auth/reset-password', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    mockRateLimiter.mockResolvedValue(null); // No rate limiting by default
    mockEmailService.generateResetToken.mockReturnValue('mock-reset-token');
  });

  describe('POST /api/auth/reset-password', () => {
    it('initiates password reset for existing user', async () => {
      const resetData = {
        email: 'existing@example.com',
      };

      const mockUser = {
        id: 'user-123',
        email: resetData.email,
        name: 'Test User',
        householdId: 'household-456',
        language: 'EN' as Language,
        timezone: 'UTC',
      };

      mockAuthService.getUserByEmail.mockResolvedValue(
        mockUser as typeof mockUser
      );
      mockEmailService.sendPasswordResetEmail.mockResolvedValue(true);

      const request = new NextRequest(
        'http://localhost:3000/api/auth/reset-password',
        {
          method: 'POST',
          body: JSON.stringify(resetData),
          headers: { 'Content-Type': 'application/json' },
        }
      );

      const response = await POST(request);
      const data = await response.json();

      expect(response.status).toBe(200);
      expect(data.success).toBe(true);
      expect(data.message).toContain('you will receive a password reset email');
      expect(mockAuthService.getUserByEmail).toHaveBeenCalledWith(
        resetData.email
      );
      expect(mockEmailService.sendPasswordResetEmail).toHaveBeenCalledWith(
        resetData.email,
        'mock-reset-token'
      );
    });

    it('returns same response for non-existing user (security)', async () => {
      const resetData = {
        email: 'nonexistent@example.com',
      };

      mockAuthService.getUserByEmail.mockResolvedValue(null);

      const request = new NextRequest(
        'http://localhost:3000/api/auth/reset-password',
        {
          method: 'POST',
          body: JSON.stringify(resetData),
          headers: { 'Content-Type': 'application/json' },
        }
      );

      const response = await POST(request);
      const data = await response.json();

      expect(response.status).toBe(200);
      expect(data.success).toBe(true);
      expect(data.message).toContain('you will receive a password reset email');
      expect(mockEmailService.sendPasswordResetEmail).not.toHaveBeenCalled();
    });

    it('returns validation error for invalid email', async () => {
      const invalidData = {
        email: 'invalid-email',
      };

      const request = new NextRequest(
        'http://localhost:3000/api/auth/reset-password',
        {
          method: 'POST',
          body: JSON.stringify(invalidData),
          headers: { 'Content-Type': 'application/json' },
        }
      );

      const response = await POST(request);
      const data = await response.json();

      expect(response.status).toBe(400);
      expect(data.success).toBe(false);
      expect(data.error).toBe('Validation failed');
      expect(data.details).toEqual(
        expect.arrayContaining([
          expect.objectContaining({
            field: 'email',
            message: expect.any(String),
          }),
        ])
      );
    });

    it('applies rate limiting when threshold exceeded', async () => {
      const rateLimitResponse = NextResponse.json(
        { error: 'Too many requests' },
        { status: 429 }
      );
      mockRateLimiter.mockResolvedValue(rateLimitResponse);

      const resetData = {
        email: 'test@example.com',
      };

      const request = new NextRequest(
        'http://localhost:3000/api/auth/reset-password',
        {
          method: 'POST',
          body: JSON.stringify(resetData),
          headers: { 'Content-Type': 'application/json' },
        }
      );

      const response = await POST(request);

      expect(response.status).toBe(429);
      expect(mockAuthService.getUserByEmail).not.toHaveBeenCalled();
    });

    it('handles errors gracefully when email service fails', async () => {
      const resetData = {
        email: 'test@example.com',
      };

      const mockUser = {
        id: 'user-123',
        email: resetData.email,
        name: 'Test User',
        householdId: 'household-456',
        language: 'EN' as Language,
        timezone: 'UTC',
      };

      mockAuthService.getUserByEmail.mockResolvedValue(
        mockUser as typeof mockUser
      );
      mockEmailService.sendPasswordResetEmail.mockResolvedValue(false);

      const request = new NextRequest(
        'http://localhost:3000/api/auth/reset-password',
        {
          method: 'POST',
          body: JSON.stringify(resetData),
          headers: { 'Content-Type': 'application/json' },
        }
      );

      const response = await POST(request);
      const data = await response.json();

      // Should still return success response even if email fails (security)
      expect(response.status).toBe(200);
      expect(data.success).toBe(true);
    });

    it('handles unexpected errors gracefully', async () => {
      const resetData = {
        email: 'test@example.com',
      };

      mockAuthService.getUserByEmail.mockRejectedValue(
        new Error('Database error')
      );

      const request = new NextRequest(
        'http://localhost:3000/api/auth/reset-password',
        {
          method: 'POST',
          body: JSON.stringify(resetData),
          headers: { 'Content-Type': 'application/json' },
        }
      );

      const response = await POST(request);
      const data = await response.json();

      expect(response.status).toBe(500);
      expect(data.success).toBe(false);
      expect(data.error).toBe('Internal server error');
    });
  });
});

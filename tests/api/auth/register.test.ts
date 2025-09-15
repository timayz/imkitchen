/**
 * @jest-environment node
 */

import { NextRequest, NextResponse } from 'next/server';
import { POST } from '@/app/api/auth/register/route';
import { AuthService } from '@/lib/services/auth-service';
import { registrationRateLimiter } from '@/lib/middleware/rate-limiter';
import { Language } from '@prisma/client';

// Mock dependencies
jest.mock('@/lib/services/auth-service');
jest.mock('@/lib/middleware/rate-limiter');

const mockAuthService = AuthService as jest.Mocked<typeof AuthService>;
const mockRateLimiter = registrationRateLimiter as jest.MockedFunction<
  typeof registrationRateLimiter
>;

describe('/api/auth/register', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    mockRateLimiter.mockResolvedValue(null); // No rate limiting by default
  });

  describe('POST /api/auth/register', () => {
    it('successfully registers a new user with valid data', async () => {
      const userData = {
        email: 'test@example.com',
        password: 'SecurePassword123',
        name: 'Test User',
        householdName: 'Test Household',
        language: 'EN',
        timezone: 'America/New_York',
        dietaryPreferences: ['VEGETARIAN'],
        allergies: ['nuts'],
      };

      const mockUser = {
        id: 'user-123',
        email: userData.email,
        name: userData.name,
        householdId: 'household-456',
        language: userData.language as Language,
        timezone: userData.timezone,
      };

      mockAuthService.registerUser.mockResolvedValue(
        mockUser as typeof mockUser
      );

      const request = new NextRequest(
        'http://localhost:3000/api/auth/register',
        {
          method: 'POST',
          body: JSON.stringify(userData),
          headers: { 'Content-Type': 'application/json' },
        }
      );

      const response = await POST(request);
      const data = await response.json();

      expect(response.status).toBe(201);
      expect(data).toEqual({
        success: true,
        user: mockUser,
        message: 'Registration successful',
      });
      expect(mockAuthService.registerUser).toHaveBeenCalledWith(userData);
    });

    it('returns validation error for invalid email', async () => {
      const invalidData = {
        email: 'invalid-email',
        password: 'SecurePassword123',
        name: 'Test User',
        householdName: 'Test Household',
        language: 'EN',
        timezone: 'America/New_York',
      };

      const request = new NextRequest(
        'http://localhost:3000/api/auth/register',
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
            message: 'Please enter a valid email address',
          }),
        ])
      );
    });

    it('returns error when email already exists', async () => {
      const userData = {
        email: 'existing@example.com',
        password: 'SecurePassword123',
        name: 'Test User',
        householdName: 'Test Household',
        language: 'EN',
        timezone: 'America/New_York',
      };

      mockAuthService.registerUser.mockRejectedValue(
        new Error('EMAIL_ALREADY_EXISTS')
      );

      const request = new NextRequest(
        'http://localhost:3000/api/auth/register',
        {
          method: 'POST',
          body: JSON.stringify(userData),
          headers: { 'Content-Type': 'application/json' },
        }
      );

      const response = await POST(request);
      const data = await response.json();

      expect(response.status).toBe(409);
      expect(data.success).toBe(false);
      expect(data.error).toBe('Email already exists');
    });

    it('returns error when password requirements not met', async () => {
      const userData = {
        email: 'test@example.com',
        password: 'weak',
        name: 'Test User',
        householdName: 'Test Household',
        language: 'EN',
        timezone: 'America/New_York',
      };

      mockAuthService.registerUser.mockRejectedValue(
        new Error('PASSWORD_REQUIREMENTS_NOT_MET')
      );

      const request = new NextRequest(
        'http://localhost:3000/api/auth/register',
        {
          method: 'POST',
          body: JSON.stringify(userData),
          headers: { 'Content-Type': 'application/json' },
        }
      );

      const response = await POST(request);
      const data = await response.json();

      expect(response.status).toBe(400);
      expect(data.success).toBe(false);
      expect(data.error).toBe('Validation failed');
    });

    it('applies rate limiting when threshold exceeded', async () => {
      const rateLimitResponse = NextResponse.json(
        { error: 'Too many requests' },
        { status: 429 }
      );
      mockRateLimiter.mockResolvedValue(rateLimitResponse);

      const userData = {
        email: 'test@example.com',
        password: 'SecurePassword123',
        name: 'Test User',
        householdName: 'Test Household',
        language: 'EN',
        timezone: 'America/New_York',
      };

      const request = new NextRequest(
        'http://localhost:3000/api/auth/register',
        {
          method: 'POST',
          body: JSON.stringify(userData),
          headers: { 'Content-Type': 'application/json' },
        }
      );

      const response = await POST(request);

      expect(response.status).toBe(429);
      expect(mockAuthService.registerUser).not.toHaveBeenCalled();
    });

    it('handles unexpected errors gracefully', async () => {
      const userData = {
        email: 'test@example.com',
        password: 'SecurePassword123',
        name: 'Test User',
        householdName: 'Test Household',
        language: 'EN',
        timezone: 'America/New_York',
      };

      mockAuthService.registerUser.mockRejectedValue(
        new Error('Unexpected database error')
      );

      const request = new NextRequest(
        'http://localhost:3000/api/auth/register',
        {
          method: 'POST',
          body: JSON.stringify(userData),
          headers: { 'Content-Type': 'application/json' },
        }
      );

      const response = await POST(request);
      const data = await response.json();

      expect(response.status).toBe(500);
      expect(data.success).toBe(false);
      expect(data.error).toBe('Registration failed');
    });
  });
});

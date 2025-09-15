/**
 * @jest-environment node
 */

// Mock dependencies before importing
jest.mock('@/lib/services/auth-service');
jest.mock('@/lib/middleware/rate-limiter', () => ({
  registrationRateLimiter: jest.fn(),
}));
jest.mock('@/lib/db', () => ({
  prisma: {
    user: {
      findUnique: jest.fn(),
      create: jest.fn(),
    },
    household: {
      create: jest.fn(),
    },
    $transaction: jest.fn(),
  },
}));

// Mock bcrypt
jest.mock('bcrypt', () => ({
  hash: jest.fn().mockResolvedValue('mocked-hash'),
  compare: jest.fn().mockResolvedValue(true),
}));

import { POST } from '@/app/api/auth/register/route';
import { AuthService } from '@/lib/services/auth-service';
import { registrationRateLimiter } from '@/lib/middleware/rate-limiter';

const mockAuthService = AuthService as jest.Mocked<typeof AuthService>;
const mockRateLimiter = registrationRateLimiter as jest.MockedFunction<
  typeof registrationRateLimiter
>;

// Helper function to create a mock request
function createMockRequest(body: unknown) {
  return {
    json: () => Promise.resolve(body),
    headers: new Headers({
      'Content-Type': 'application/json',
    }),
    method: 'POST',
    url: 'http://localhost:3000/api/auth/register',
  } as {
    json: () => Promise<unknown>;
    headers: Headers;
    method: string;
    url: string;
  };
}

describe('/api/auth/register', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    mockRateLimiter.mockResolvedValue(null); // Allow requests by default
  });

  it('should register a new user successfully', async () => {
    const registrationData = {
      email: 'test@example.com',
      name: 'Test User',
      password: 'Password123',
      householdName: 'Test Household',
      dietaryPreferences: [],
      allergies: [],
      language: 'EN',
      timezone: 'UTC',
    };

    const mockUser = {
      id: 'user-id',
      email: 'test@example.com',
      name: 'Test User',
      householdId: 'household-id',
      language: 'EN',
      timezone: 'UTC',
    };

    mockAuthService.registerUser.mockResolvedValue(mockUser);

    const request = createMockRequest(registrationData);
    const response = await POST(request);
    const data = await response.json();

    expect(response.status).toBe(201);
    expect(data.success).toBe(true);
    expect(data.user).toEqual(mockUser);
    expect(mockAuthService.registerUser).toHaveBeenCalledWith(registrationData);
  });

  it('should return validation error for invalid data', async () => {
    const invalidData = {
      email: 'invalid-email',
      name: '',
      password: '123', // Too short
      householdName: '',
    };

    const request = createMockRequest(invalidData);
    const response = await POST(request);
    const data = await response.json();

    expect(response.status).toBe(400);
    expect(data.success).toBe(false);
    expect(data.error).toBe('Validation failed');
    expect(data.details).toBeDefined();
    expect(Array.isArray(data.details)).toBe(true);
  });

  it('should return error when email already exists', async () => {
    const registrationData = {
      email: 'existing@example.com',
      name: 'Test User',
      password: 'Password123',
      householdName: 'Test Household',
    };

    mockAuthService.registerUser.mockRejectedValue(
      new Error('EMAIL_ALREADY_EXISTS')
    );

    const request = createMockRequest(registrationData);
    const response = await POST(request);
    const data = await response.json();

    expect(response.status).toBe(409);
    expect(data.success).toBe(false);
    expect(data.error).toBe('Email already exists');
  });

  it('should respect rate limiting', async () => {
    const rateLimitResponse = new Response(
      JSON.stringify({
        success: false,
        error: 'Rate limit exceeded',
        message: 'Too many attempts. Please try again later.',
      }),
      { status: 429 }
    );

    mockRateLimiter.mockResolvedValue(rateLimitResponse);

    const registrationData = {
      email: 'test@example.com',
      name: 'Test User',
      password: 'Password123',
      householdName: 'Test Household',
    };

    const request = createMockRequest(registrationData);
    const response = await POST(request);

    expect(response.status).toBe(429);
    expect(mockRateLimiter).toHaveBeenCalledWith(request);
    expect(mockAuthService.registerUser).not.toHaveBeenCalled();
  });
});

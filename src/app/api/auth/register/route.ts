import { NextRequest, NextResponse } from 'next/server';
import { ZodError } from 'zod';
import { registerSchema } from '@/lib/validators/auth-schemas';
import { AuthService } from '@/lib/services/auth-service';
import { AuthError } from '@/types/auth';
import { registrationRateLimiter } from '@/lib/middleware/rate-limiter';

export async function POST(request: NextRequest) {
  // Apply rate limiting
  const rateLimitResponse = await registrationRateLimiter(request);
  if (rateLimitResponse) {
    return rateLimitResponse;
  }
  try {
    const body = await request.json();

    // Validate request data
    const validatedData = registerSchema.parse(body);

    // Register user
    const user = await AuthService.registerUser(validatedData);

    // Return user data (without sensitive information)
    return NextResponse.json(
      {
        success: true,
        user: {
          id: user.id,
          email: user.email,
          name: user.name,
          householdId: user.householdId,
          language: user.language,
          timezone: user.timezone,
        },
        message: 'Registration successful',
      },
      { status: 201 }
    );
  } catch (error) {
    console.error('Registration error:', error);

    // Handle validation errors
    if (error instanceof ZodError) {
      return NextResponse.json(
        {
          success: false,
          error: 'Validation failed',
          details: error.issues.map(err => ({
            field: err.path.join('.'),
            message: err.message,
          })),
        },
        { status: 400 }
      );
    }

    // Handle authentication errors
    if (error instanceof Error) {
      switch (error.message) {
        case AuthError.EMAIL_ALREADY_EXISTS:
          return NextResponse.json(
            {
              success: false,
              error: 'Email already exists',
              message: 'An account with this email address already exists',
            },
            { status: 409 }
          );
        case AuthError.PASSWORD_REQUIREMENTS_NOT_MET:
          return NextResponse.json(
            {
              success: false,
              error: 'Password requirements not met',
              message: 'Password does not meet security requirements',
            },
            { status: 400 }
          );
        default:
          return NextResponse.json(
            {
              success: false,
              error: 'Registration failed',
              message: 'An error occurred during registration',
            },
            { status: 500 }
          );
      }
    }

    // Generic error response
    return NextResponse.json(
      {
        success: false,
        error: 'Internal server error',
        message: 'An unexpected error occurred',
      },
      { status: 500 }
    );
  }
}

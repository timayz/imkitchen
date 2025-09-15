import { NextRequest, NextResponse } from 'next/server';
import { ZodError } from 'zod';
import { passwordResetRequestSchema } from '@/lib/validators/auth-schemas';
import { AuthService } from '@/lib/services/auth-service';
import { EmailService } from '@/lib/services/email-service';
import { prisma } from '@/lib/db';
import { passwordResetRateLimiter } from '@/lib/middleware/rate-limiter';

export async function POST(request: NextRequest) {
  // Apply rate limiting
  const rateLimitResponse = await passwordResetRateLimiter(request);
  if (rateLimitResponse) {
    return rateLimitResponse;
  }
  try {
    const body = await request.json();
    const { email } = passwordResetRequestSchema.parse(body);

    // Check if user exists
    const user = await AuthService.getUserByEmail(email);

    if (!user) {
      // Don't reveal if email exists for security
      return NextResponse.json(
        {
          success: true,
          message:
            'If an account with that email exists, you will receive a password reset email.',
        },
        { status: 200 }
      );
    }

    // Generate reset token
    const resetToken = EmailService.generateResetToken();
    const expiresAt = new Date(Date.now() + 60 * 60 * 1000); // 1 hour from now

    // Store reset token in database
    await prisma.passwordResetToken.create({
      data: {
        userId: user.id,
        token: resetToken,
        expiresAt,
      },
    });

    // Send reset email
    const emailSent = await EmailService.sendPasswordResetEmail(
      email,
      resetToken
    );

    if (!emailSent) {
      console.error('Failed to send password reset email to:', email);
    }

    return NextResponse.json(
      {
        success: true,
        message:
          'If an account with that email exists, you will receive a password reset email.',
      },
      { status: 200 }
    );
  } catch (error) {
    console.error('Password reset request error:', error);

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

import { NextRequest, NextResponse } from 'next/server';
import { ZodError } from 'zod';
import { passwordResetConfirmSchema } from '@/lib/validators/auth-schemas';
import { AuthService } from '@/lib/services/auth-service';
import { prisma } from '@/lib/db';

export async function POST(request: NextRequest) {
  try {
    const body = await request.json();
    const { token, password } = passwordResetConfirmSchema.parse(body);

    // Find valid reset token
    const resetToken = await prisma.passwordResetToken.findFirst({
      where: {
        token,
        used: false,
        expiresAt: {
          gt: new Date(),
        },
      },
      include: {
        user: true,
      },
    });

    if (!resetToken) {
      return NextResponse.json(
        {
          success: false,
          error: 'Invalid or expired reset token',
          message: 'The password reset link is invalid or has expired',
        },
        { status: 400 }
      );
    }

    // Hash new password
    const passwordHash = await AuthService.hashPassword(password);

    // Update user password and mark token as used in a transaction
    await prisma.$transaction(async tx => {
      // Update user password
      await tx.user.update({
        where: { id: resetToken.userId },
        data: { passwordHash },
      });

      // Mark token as used
      await tx.passwordResetToken.update({
        where: { id: resetToken.id },
        data: { used: true },
      });

      // Clean up expired tokens for this user
      await tx.passwordResetToken.deleteMany({
        where: {
          userId: resetToken.userId,
          OR: [{ used: true }, { expiresAt: { lt: new Date() } }],
        },
      });
    });

    return NextResponse.json(
      {
        success: true,
        message:
          'Password reset successful. You can now log in with your new password.',
      },
      { status: 200 }
    );
  } catch (error) {
    console.error('Password reset confirm error:', error);

    if (error instanceof ZodError) {
      return NextResponse.json(
        {
          success: false,
          error: 'Validation failed',
          details: error.errors.map(err => ({
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

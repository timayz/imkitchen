import { NextRequest, NextResponse } from 'next/server';

interface RateLimitStore {
  [key: string]: {
    attempts: number;
    resetTime: number;
  };
}

// Simple in-memory rate limiter (in production, use Redis)
const rateLimitStore: RateLimitStore = {};

export interface RateLimitOptions {
  windowMs: number; // Time window in milliseconds
  maxAttempts: number; // Maximum attempts per window
  keyGenerator?: (request: NextRequest) => string;
}

export function createRateLimiter(options: RateLimitOptions) {
  const { windowMs, maxAttempts, keyGenerator } = options;

  return async function rateLimiter(
    request: NextRequest
  ): Promise<NextResponse | null> {
    const key = keyGenerator ? keyGenerator(request) : getClientIP(request);
    const now = Date.now();

    // Clean up expired entries
    Object.keys(rateLimitStore).forEach(k => {
      if (rateLimitStore[k].resetTime < now) {
        delete rateLimitStore[k];
      }
    });

    // Get or create rate limit entry
    if (!rateLimitStore[key]) {
      rateLimitStore[key] = {
        attempts: 0,
        resetTime: now + windowMs,
      };
    }

    const entry = rateLimitStore[key];

    // Reset if window has expired
    if (entry.resetTime < now) {
      entry.attempts = 0;
      entry.resetTime = now + windowMs;
    }

    // Check if limit exceeded
    if (entry.attempts >= maxAttempts) {
      const resetInSeconds = Math.ceil((entry.resetTime - now) / 1000);

      return NextResponse.json(
        {
          success: false,
          error: 'Rate limit exceeded',
          message: `Too many attempts. Please try again in ${resetInSeconds} seconds.`,
          retryAfter: resetInSeconds,
        },
        {
          status: 429,
          headers: {
            'Retry-After': resetInSeconds.toString(),
            'X-RateLimit-Limit': maxAttempts.toString(),
            'X-RateLimit-Remaining': '0',
            'X-RateLimit-Reset': entry.resetTime.toString(),
          },
        }
      );
    }

    // Increment attempt count
    entry.attempts++;

    return null; // Allow request to proceed
  };
}

function getClientIP(request: NextRequest): string {
  const forwarded = request.headers.get('x-forwarded-for');
  const real = request.headers.get('x-real-ip');
  const ip = forwarded?.split(',')[0] || real || 'unknown';
  return ip;
}

// Pre-configured rate limiters for common use cases
export const authRateLimiter = createRateLimiter({
  windowMs: 15 * 60 * 1000, // 15 minutes
  maxAttempts: 5, // 5 attempts per 15 minutes
});

export const passwordResetRateLimiter = createRateLimiter({
  windowMs: 60 * 60 * 1000, // 1 hour
  maxAttempts: 3, // 3 attempts per hour
});

export const registrationRateLimiter = createRateLimiter({
  windowMs: 60 * 60 * 1000, // 1 hour
  maxAttempts: 3, // 3 registrations per hour per IP
});

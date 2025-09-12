import { describe, it, expect, vi, beforeEach } from 'vitest';
import bcrypt from 'bcrypt';
import type { NextAuthOptions } from 'next-auth';

// Mock dependencies
vi.mock('./prisma', () => ({
  prisma: {
    user: {
      findUnique: vi.fn(),
    },
    account: {
      findFirst: vi.fn(),
    },
  },
}));

vi.mock('bcrypt', () => ({
  default: {
    compare: vi.fn(),
  },
}));

describe('NextAuth Configuration', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should export authOptions', async () => {
    const { authOptions } = await import('./auth');
    expect(authOptions).toBeDefined();
    expect(authOptions.providers).toBeDefined();
    expect(authOptions.session).toBeDefined();
    expect(authOptions.callbacks).toBeDefined();
  });

  it('should have correct session strategy', async () => {
    const { authOptions } = await import('./auth');
    expect(authOptions.session?.strategy).toBe('jwt');
    expect(authOptions.session?.maxAge).toBe(30 * 24 * 60 * 60); // 30 days
  });

  it('should have correct JWT configuration', async () => {
    const { authOptions } = await import('./auth');
    expect(authOptions.jwt?.maxAge).toBe(30 * 24 * 60 * 60); // 30 days
  });

  it('should have credentials provider configured', async () => {
    const { authOptions } = await import('./auth');
    const credentialsProvider = authOptions.providers?.find(
      (provider: any) => provider.id === 'credentials'
    );
    expect(credentialsProvider).toBeDefined();
    expect(credentialsProvider?.name).toBe('Credentials');
  });

  it('should have correct page redirects', async () => {
    const { authOptions } = await import('./auth');
    expect(authOptions.pages?.signIn).toBe('/auth/signin');
    expect(authOptions.pages?.error).toBe('/auth/error');
    expect(authOptions.pages?.verifyRequest).toBe('/auth/verify-request');
  });
});
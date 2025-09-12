import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { generateVerificationToken, sendVerificationEmail } from './email';

// Mock console.log to capture email outputs in development
const mockConsoleLog = vi.spyOn(console, 'log').mockImplementation(() => {});

describe('Email Utilities', () => {
  const originalNodeEnv = process.env.NODE_ENV;
  
  beforeEach(() => {
    vi.clearAllMocks();
    // Mock NODE_ENV to development for testing
    vi.stubEnv('NODE_ENV', 'development');
  });
  
  afterEach(() => {
    vi.unstubAllEnvs();
  });

  describe('generateVerificationToken', () => {
    it('should generate a token', () => {
      const token = generateVerificationToken();
      expect(token).toBeDefined();
      expect(typeof token).toBe('string');
      expect(token.length).toBeGreaterThan(0);
    });

    it('should generate unique tokens', () => {
      const token1 = generateVerificationToken();
      const token2 = generateVerificationToken();
      expect(token1).not.toBe(token2);
    });

    it('should generate tokens of consistent length', () => {
      const token1 = generateVerificationToken();
      const token2 = generateVerificationToken();
      expect(token1.length).toBe(token2.length);
      expect(token1.length).toBe(64); // 32 bytes = 64 hex characters
    });
  });

  describe('sendVerificationEmail', () => {
    it('should log email in development environment', async () => {
      const email = 'test@example.com';
      const token = 'test-token';

      await sendVerificationEmail(email, token);

      expect(mockConsoleLog).toHaveBeenCalled();
      // Check that the console.log was called with email verification content
      const logCalls = mockConsoleLog.mock.calls;
      const emailLogCall = logCalls.find(call => 
        call.some(arg => typeof arg === 'string' && arg.includes('EMAIL VERIFICATION'))
      );
      expect(emailLogCall).toBeDefined();
    });

    it('should include email address in development log', async () => {
      const email = 'user@test.com';
      const token = 'verification-token';

      await sendVerificationEmail(email, token);

      const logCalls = mockConsoleLog.mock.calls;
      const emailInLog = logCalls.some(call =>
        call.some(arg => typeof arg === 'string' && arg.includes(email))
      );
      expect(emailInLog).toBe(true);
    });

    it('should include verification URL in development log', async () => {
      const email = 'user@test.com';
      const token = 'verification-token';
      process.env.NEXTAUTH_URL = 'http://localhost:3000';

      await sendVerificationEmail(email, token);

      const logCalls = mockConsoleLog.mock.calls;
      const urlInLog = logCalls.some(call =>
        call.some(arg => typeof arg === 'string' && arg.includes(`http://localhost:3000/auth/verify-email?token=${token}`))
      );
      expect(urlInLog).toBe(true);
    });

    it('should not throw in development environment', async () => {
      const email = 'test@example.com';
      const token = 'test-token';

      await expect(sendVerificationEmail(email, token)).resolves.toBeUndefined();
    });
  });
});
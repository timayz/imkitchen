/* eslint-disable @typescript-eslint/no-explicit-any */
// Environment variables must be set before any modules import them
process.env = {
  ...process.env,
  NODE_ENV: 'development',
  DATABASE_URL: 'postgresql://test:test@localhost:5432/imkitchen_test',
  REDIS_URL: 'redis://localhost:6379',
  NEXTAUTH_SECRET:
    'test-secret-for-testing-environment-which-is-at-least-32-chars',
  NEXTAUTH_URL: 'http://localhost:3000',
  NEXT_PUBLIC_APP_URL: 'http://localhost:3000',
  NEXT_PUBLIC_API_URL: 'http://localhost:3000/api',
  LOG_LEVEL: 'error',
};

// Polyfills for Web API globals used by Next.js API routes
import { TextEncoder, TextDecoder } from 'util';
import { webcrypto } from 'crypto';

global.TextEncoder = TextEncoder as any;
global.TextDecoder = TextDecoder as any;
global.crypto = webcrypto as any;

// Mock Request and Response for API route testing
global.Request = jest.fn().mockImplementation((input, init) => ({
  url: typeof input === 'string' ? input : input.url,
  method: init?.method || 'GET',
  headers: new Headers(init?.headers),
  body: init?.body,
  json: jest.fn().mockResolvedValue(init?.body ? JSON.parse(init.body) : {}),
}));

global.Response = jest.fn().mockImplementation((body, init) => ({
  status: init?.status || 200,
  statusText: init?.statusText || 'OK',
  headers: new Headers(init?.headers),
  body,
  json: jest.fn().mockResolvedValue(body ? JSON.parse(body) : {}),
  error: () => new Response(),
  redirect: () => new Response(),
})) as any;

global.Headers = class Headers {
  private headers: Map<string, string> = new Map();

  constructor(init?: HeadersInit) {
    if (init) {
      if (Array.isArray(init)) {
        init.forEach(([key, value]) => this.set(key, value));
      } else if (init instanceof Headers) {
        // Copy from another Headers instance
        (init as any).forEach((value: string, key: string) => {
          this.set(key, value);
        });
      } else {
        Object.entries(init).forEach(([key, value]) => this.set(key, value));
      }
    }
  }

  set(key: string, value: string) {
    this.headers.set(key.toLowerCase(), value);
  }

  get(key: string) {
    return this.headers.get(key.toLowerCase()) || null;
  }

  has(key: string) {
    return this.headers.has(key.toLowerCase());
  }

  delete(key: string) {
    this.headers.delete(key.toLowerCase());
  }

  append(key: string, value: string) {
    const existing = this.get(key);
    if (existing) {
      this.set(key, `${existing}, ${value}`);
    } else {
      this.set(key, value);
    }
  }

  entries() {
    return this.headers.entries();
  }

  forEach(callback: (value: string, key: string, parent: Headers) => void) {
    this.headers.forEach((value, key) => callback(value, key, this));
  }

  keys() {
    return this.headers.keys();
  }

  values() {
    return this.headers.values();
  }

  [Symbol.iterator]() {
    return this.headers.entries();
  }
} as any;

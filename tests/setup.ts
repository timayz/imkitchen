import '@testing-library/jest-dom';

// Only setup browser mocks in jsdom environment
if (typeof window !== 'undefined') {
  // Mock IntersectionObserver
  global.IntersectionObserver = class IntersectionObserver {
    constructor() {}
    disconnect() {}
    observe() {}
    unobserve() {}
    root = null;
    rootMargin = '';
    thresholds = [];
    takeRecords() {
      return [];
    }
  } as unknown as typeof IntersectionObserver;

  // Mock window.matchMedia
  Object.defineProperty(window, 'matchMedia', {
    writable: true,
    value: jest.fn().mockImplementation(query => ({
      matches: false,
      media: query,
      onchange: null,
      addListener: jest.fn(),
      removeListener: jest.fn(),
      addEventListener: jest.fn(),
      removeEventListener: jest.fn(),
      dispatchEvent: jest.fn(),
    })),
  });

  // Mock window.scrollTo
  Object.defineProperty(window, 'scrollTo', {
    writable: true,
    value: jest.fn(),
  });

  // Mock localStorage
  const localStorageMock = {
    getItem: jest.fn(),
    setItem: jest.fn(),
    removeItem: jest.fn(),
    clear: jest.fn(),
    length: 0,
    key: jest.fn(),
  };

  global.localStorage = localStorageMock as unknown as Storage;
}

// Mock Prisma client for all tests
jest.mock('../src/lib/db', () => ({
  db: {
    user: {
      findUnique: jest.fn(),
      findMany: jest.fn(),
      create: jest.fn(),
      update: jest.fn(),
      delete: jest.fn(),
      count: jest.fn(),
      findFirst: jest.fn(),
      updateMany: jest.fn(),
      deleteMany: jest.fn(),
      upsert: jest.fn(),
    },
    household: {
      findUnique: jest.fn(),
      findMany: jest.fn(),
      create: jest.fn(),
      update: jest.fn(),
      delete: jest.fn(),
      count: jest.fn(),
      findFirst: jest.fn(),
      updateMany: jest.fn(),
      deleteMany: jest.fn(),
      upsert: jest.fn(),
    },
    session: {
      findUnique: jest.fn(),
      findMany: jest.fn(),
      create: jest.fn(),
      update: jest.fn(),
      delete: jest.fn(),
      count: jest.fn(),
      findFirst: jest.fn(),
      updateMany: jest.fn(),
      deleteMany: jest.fn(),
      upsert: jest.fn(),
    },
    $transaction: jest.fn(),
    $connect: jest.fn(),
    $disconnect: jest.fn(),
    $queryRaw: jest.fn(),
    $queryRawUnsafe: jest.fn(),
  },
  checkDatabaseHealth: jest.fn(),
  disconnectDatabase: jest.fn(),
  connectWithRetry: jest.fn(),
  databaseProvider: {
    query: jest.fn(),
    transaction: jest.fn(),
    disconnect: jest.fn(),
  },
}));

// Mock bcryptjs
jest.mock('bcryptjs', () => ({
  hash: jest.fn().mockResolvedValue('mocked-hash'),
  compare: jest.fn().mockResolvedValue(true),
}));

// Mock logger
jest.mock('../src/lib/logger', () => ({
  logger: {
    debug: jest.fn(),
    info: jest.fn(),
    warn: jest.fn(),
    error: jest.fn(),
    dbOperation: jest.fn(),
    dbError: jest.fn(),
    apiRequest: jest.fn(),
    authEvent: jest.fn(),
    performance: jest.fn(),
    security: jest.fn(),
  },
  withLogging: jest.fn((_operation, fn) => fn()),
  logDatabaseOperation: jest.fn((_operation, _model, fn) => fn()),
}));

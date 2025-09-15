/* eslint-disable @typescript-eslint/no-explicit-any */
// Test utilities for mocking
export type MockedPrismaClient = {
  inventoryItem: {
    findFirst: jest.MockedFunction<any>;
    findMany: jest.MockedFunction<any>;
    create: jest.MockedFunction<any>;
    update: jest.MockedFunction<any>;
    delete: jest.MockedFunction<any>;
    [key: string]: jest.MockedFunction<any>;
  };
  [key: string]: any;
};

export const createMockPrisma = (): MockedPrismaClient => {
  return {
    inventoryItem: {
      findFirst: jest.fn(),
      findMany: jest.fn(),
      create: jest.fn(),
      update: jest.fn(),
      delete: jest.fn(),
    },
  } as any;
};

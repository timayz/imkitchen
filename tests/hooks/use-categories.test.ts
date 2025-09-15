import React from 'react';
import { renderHook, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import {
  useCategories,
  useCreateCategory,
  useDeleteCategory,
} from '@/hooks/use-categories';
import { apiClient } from '@/lib/api-client';

// Mock API client
jest.mock('@/lib/api-client');
const mockApiClient = apiClient as jest.Mocked<typeof apiClient>;

const createWrapper = () => {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
      mutations: { retry: false },
    },
  });

  function TestWrapper({ children }: { children: React.ReactNode }) {
    return React.createElement(
      QueryClientProvider,
      { client: queryClient },
      children
    );
  }

  return TestWrapper;
};

const mockCategories = [
  {
    id: 'cat-1',
    name: 'Supplements',
    color: '#8b5cf6',
    icon: 'pill',
    householdId: 'household-1',
    createdBy: 'user-1',
    createdAt: new Date(),
    updatedAt: new Date(),
  },
];

describe('useCategories', () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });

  it('fetches categories successfully', async () => {
    mockApiClient.get.mockResolvedValue(mockCategories);

    const { result } = renderHook(() => useCategories(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.isSuccess).toBe(true);
    });

    expect(result.current.data).toEqual(mockCategories);
    expect(mockApiClient.get).toHaveBeenCalledWith('/inventory/categories');
  });

  it('handles fetch error', async () => {
    mockApiClient.get.mockRejectedValue(new Error('Network error'));

    const { result } = renderHook(() => useCategories(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.isError).toBe(true);
    });

    expect(result.current.error).toBeTruthy();
  });
});

describe('useCreateCategory', () => {
  it('creates category successfully', async () => {
    const newCategory = {
      name: 'Test Category',
      color: '#ff0000',
      icon: 'test',
    };

    const createdCategory = {
      id: 'cat-2',
      ...newCategory,
      householdId: 'household-1',
      createdBy: 'user-1',
      createdAt: new Date(),
      updatedAt: new Date(),
    };

    mockApiClient.post.mockResolvedValue(createdCategory);

    const { result } = renderHook(() => useCreateCategory(), {
      wrapper: createWrapper(),
    });

    const response = await result.current.mutateAsync(newCategory);

    expect(response).toEqual(createdCategory);
    expect(mockApiClient.post).toHaveBeenCalledWith(
      '/inventory/categories',
      newCategory
    );
  });
});

describe('useDeleteCategory', () => {
  it('deletes category successfully', async () => {
    mockApiClient.delete.mockResolvedValue(undefined);

    const { result } = renderHook(() => useDeleteCategory(), {
      wrapper: createWrapper(),
    });

    await result.current.mutateAsync('cat-1');

    expect(mockApiClient.delete).toHaveBeenCalledWith(
      '/inventory/categories/cat-1'
    );
  });
});

'use client';

import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import {
  CustomCategory,
  CustomCategoryCreate,
  CustomCategoryUpdate,
  CategoryStats,
} from '@/types/inventory';
import { apiClient } from '@/lib/api-client';

// API functions
async function getCategories(): Promise<CustomCategory[]> {
  return apiClient.get<CustomCategory[]>('/inventory/categories');
}

async function createCategory(
  data: CustomCategoryCreate
): Promise<CustomCategory> {
  return apiClient.post<CustomCategory>('/inventory/categories', data);
}

async function updateCategory(
  id: string,
  data: CustomCategoryUpdate
): Promise<CustomCategory> {
  return apiClient.put<CustomCategory>(`/inventory/categories/${id}`, data);
}

async function deleteCategory(id: string): Promise<void> {
  return apiClient.delete<void>(`/inventory/categories/${id}`);
}

async function getCategoryStats(): Promise<CategoryStats[]> {
  return apiClient.get<CategoryStats[]>('/inventory/categories/stats');
}

// Hooks
export function useCategories() {
  return useQuery({
    queryKey: ['categories'],
    queryFn: getCategories,
    staleTime: 1000 * 60 * 5, // 5 minutes
  });
}

export function useCategoryStats() {
  return useQuery({
    queryKey: ['categoryStats'],
    queryFn: getCategoryStats,
    staleTime: 1000 * 60 * 2, // 2 minutes
  });
}

export function useCreateCategory() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: createCategory,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['categories'] });
      queryClient.invalidateQueries({ queryKey: ['categoryStats'] });
    },
  });
}

export function useUpdateCategory() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ id, data }: { id: string; data: CustomCategoryUpdate }) =>
      updateCategory(id, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['categories'] });
      queryClient.invalidateQueries({ queryKey: ['categoryStats'] });
    },
  });
}

export function useDeleteCategory() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: deleteCategory,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['categories'] });
      queryClient.invalidateQueries({ queryKey: ['categoryStats'] });
      queryClient.invalidateQueries({ queryKey: ['inventory'] });
    },
  });
}

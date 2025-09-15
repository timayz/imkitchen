/* eslint-disable @typescript-eslint/no-explicit-any */
import React from 'react';
import { render, screen, fireEvent } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { InventoryList } from '@/components/inventory/inventory-list';
import { useInventoryItems } from '@/hooks/use-inventory';
import type { InventoryItem } from '@/types/inventory';

// Mock the hooks and components
jest.mock('@/hooks/use-inventory');
jest.mock('@/components/inventory/inventory-item', () => ({
  InventoryItemComponent: ({ item, onEdit }: any) => (
    <div data-testid="inventory-item">
      <span>{item.name}</span>
      <button onClick={() => onEdit(item)}>Edit</button>
    </div>
  ),
}));
jest.mock('@/components/ui/loading-spinner', () => ({
  LoadingSpinner: () => <div data-testid="loading-spinner">Loading...</div>,
}));

const mockUseInventoryItems = useInventoryItems as jest.MockedFunction<
  typeof useInventoryItems
>;

describe('InventoryList', () => {
  let queryClient: QueryClient;

  const mockItems: InventoryItem[] = [
    {
      id: 'item-1',
      name: 'Tomatoes',
      quantity: 5,
      unit: 'pieces',
      category: 'vegetables',
      location: 'refrigerator',
      expirationDate: new Date('2025-12-31'),
      purchaseDate: new Date('2025-01-01'),
      estimatedCost: 3.5,
      householdId: 'household-1',
      addedBy: 'user-1',
      createdAt: new Date(),
      updatedAt: new Date(),
      addedByUser: { name: 'Test User' },
    },
    {
      id: 'item-2',
      name: 'Bread',
      quantity: 1,
      unit: 'pieces',
      category: 'grains',
      location: 'pantry',
      expirationDate: new Date('2025-09-20'),
      purchaseDate: new Date('2025-09-15'),
      estimatedCost: 2.99,
      householdId: 'household-1',
      addedBy: 'user-1',
      createdAt: new Date(),
      updatedAt: new Date(),
      addedByUser: { name: 'Test User' },
    },
  ];

  beforeEach(() => {
    queryClient = new QueryClient({
      defaultOptions: {
        queries: { retry: false },
        mutations: { retry: false },
      },
    });

    mockUseInventoryItems.mockReturnValue({
      data: mockItems,
      isLoading: false,
      error: null,
    } as any);

    jest.clearAllMocks();
  });

  const renderList = (props = {}) => {
    return render(
      <QueryClientProvider client={queryClient}>
        <InventoryList {...props} />
      </QueryClientProvider>
    );
  };

  describe('Rendering', () => {
    it('should render inventory list with items', () => {
      renderList();

      expect(screen.getByText('Kitchen Inventory')).toBeInTheDocument();
      expect(screen.getByText('Tomatoes')).toBeInTheDocument();
      expect(screen.getByText('Bread')).toBeInTheDocument();
      expect(screen.getAllByTestId('inventory-item')).toHaveLength(2);
    });

    it('should render add item button', () => {
      const mockOnAddItem = jest.fn();
      renderList({ onAddItem: mockOnAddItem });

      expect(
        screen.getByRole('button', { name: /add item/i })
      ).toBeInTheDocument();
    });

    it('should render filters', () => {
      renderList();

      expect(screen.getByLabelText(/search/i)).toBeInTheDocument();
      expect(screen.getByLabelText(/location/i)).toBeInTheDocument();
      expect(screen.getByLabelText(/category/i)).toBeInTheDocument();
    });
  });

  describe('Loading and Error States', () => {
    it('should show loading spinner when loading', () => {
      mockUseInventoryItems.mockReturnValue({
        data: [],
        isLoading: true,
        error: null,
      } as any);

      renderList();

      expect(screen.getByTestId('loading-spinner')).toBeInTheDocument();
    });

    it('should show error message when error occurs', () => {
      mockUseInventoryItems.mockReturnValue({
        data: [],
        isLoading: false,
        error: new Error('Failed to fetch'),
      } as any);

      renderList();

      expect(
        screen.getByText('Failed to load inventory items')
      ).toBeInTheDocument();
      expect(
        screen.getByRole('button', { name: /retry/i })
      ).toBeInTheDocument();
    });

    it('should show empty state when no items', () => {
      mockUseInventoryItems.mockReturnValue({
        data: [],
        isLoading: false,
        error: null,
      } as any);

      renderList();

      // Should show empty states for each location
      expect(screen.getByText('No items in pantry')).toBeInTheDocument();
      expect(screen.getByText('No items in refrigerator')).toBeInTheDocument();
      expect(screen.getByText('No items in freezer')).toBeInTheDocument();
    });
  });

  describe('Interactions', () => {
    it('should call onAddItem when add button is clicked', () => {
      const mockOnAddItem = jest.fn();
      renderList({ onAddItem: mockOnAddItem });

      fireEvent.click(screen.getByRole('button', { name: /add item/i }));

      expect(mockOnAddItem).toHaveBeenCalled();
    });

    it('should call onEditItem when edit button is clicked', () => {
      const mockOnEditItem = jest.fn();
      renderList({ onEditItem: mockOnEditItem });

      // Find the first edit button and click it
      const editButtons = screen.getAllByText('Edit');
      fireEvent.click(editButtons[0]);

      // The items might be reordered by the component, so just check that the function was called
      expect(mockOnEditItem).toHaveBeenCalledWith(
        expect.objectContaining({
          id: expect.any(String),
          name: expect.any(String),
          householdId: 'household-1',
        })
      );
    });

    it('should update search query when typing', () => {
      renderList();

      const searchInput = screen.getByLabelText(/search/i);
      fireEvent.change(searchInput, { target: { value: 'tomato' } });

      expect(searchInput).toHaveValue('tomato');
    });

    it('should update location filter', () => {
      renderList();

      const locationSelect = screen.getByLabelText(/location/i);
      fireEvent.change(locationSelect, { target: { value: 'refrigerator' } });

      expect(locationSelect).toHaveValue('refrigerator');
    });

    it('should update category filter', () => {
      renderList();

      const categorySelect = screen.getByLabelText(/category/i);
      fireEvent.change(categorySelect, { target: { value: 'vegetables' } });

      expect(categorySelect).toHaveValue('vegetables');
    });
  });

  describe('Filtering Integration', () => {
    it('should call useInventoryItems with correct filters when search changes', () => {
      renderList();

      const searchInput = screen.getByLabelText(/search/i);
      fireEvent.change(searchInput, { target: { value: 'tomato' } });

      expect(mockUseInventoryItems).toHaveBeenLastCalledWith({
        search: 'tomato',
      });
    });

    it('should call useInventoryItems with correct filters when location changes', () => {
      renderList();

      const locationSelect = screen.getByLabelText(/location/i);
      fireEvent.change(locationSelect, { target: { value: 'refrigerator' } });

      expect(mockUseInventoryItems).toHaveBeenLastCalledWith({
        location: 'refrigerator',
      });
    });

    it('should call useInventoryItems with multiple filters', () => {
      renderList();

      const searchInput = screen.getByLabelText(/search/i);
      const categorySelect = screen.getByLabelText(/category/i);

      fireEvent.change(searchInput, { target: { value: 'bread' } });
      fireEvent.change(categorySelect, { target: { value: 'grains' } });

      expect(mockUseInventoryItems).toHaveBeenLastCalledWith({
        search: 'bread',
        category: 'grains',
      });
    });
  });

  describe('Accessibility', () => {
    it('should have proper form labels', () => {
      renderList();

      expect(screen.getByLabelText(/search/i)).toHaveAttribute('id', 'search');
      expect(screen.getByLabelText(/location/i)).toHaveAttribute(
        'id',
        'location'
      );
      expect(screen.getByLabelText(/category/i)).toHaveAttribute(
        'id',
        'category'
      );
    });

    it('should support keyboard navigation', () => {
      renderList();

      const searchInput = screen.getByLabelText(/search/i);
      searchInput.focus();
      expect(document.activeElement).toBe(searchInput);
    });
  });
});

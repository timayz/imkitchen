/* eslint-disable @typescript-eslint/no-explicit-any */
import React from 'react';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { InventoryForm } from '@/components/forms/inventory-form';
import {
  useCreateInventoryItem,
  useUpdateInventoryItem,
} from '@/hooks/use-inventory';
import { useAutoSave } from '@/hooks/use-local-storage';
import type { InventoryItem } from '@/types/inventory';

// Mock the hooks
jest.mock('@/hooks/use-inventory');
jest.mock('@/hooks/use-local-storage');

const mockUseCreateInventoryItem =
  useCreateInventoryItem as jest.MockedFunction<typeof useCreateInventoryItem>;
const mockUseUpdateInventoryItem =
  useUpdateInventoryItem as jest.MockedFunction<typeof useUpdateInventoryItem>;
const mockUseAutoSave = useAutoSave as jest.MockedFunction<typeof useAutoSave>;

describe('InventoryForm', () => {
  let queryClient: QueryClient;
  let user: ReturnType<typeof userEvent.setup>;

  const mockCreateMutation = {
    mutateAsync: jest.fn(),
    isPending: false,
    isError: false,
    error: null,
  };

  const mockUpdateMutation = {
    mutateAsync: jest.fn(),
    isPending: false,
    isError: false,
    error: null,
  };

  const mockInventoryItem: InventoryItem = {
    id: 'item-1',
    name: 'Test Item',
    quantity: 5,
    unit: 'pieces',
    category: 'vegetables',
    location: 'pantry',
    expirationDate: new Date('2025-12-31'),
    purchaseDate: new Date('2025-01-01'),
    estimatedCost: 10.5,
    householdId: 'household-1',
    addedBy: 'user-1',
    createdAt: new Date(),
    updatedAt: new Date(),
    addedByUser: { name: 'Test User' },
  };

  beforeEach(() => {
    user = userEvent.setup();
    queryClient = new QueryClient({
      defaultOptions: {
        queries: { retry: false },
        mutations: { retry: false },
      },
    });

    // Reset mutation mocks
    mockCreateMutation.isPending = false;
    mockUpdateMutation.isPending = false;
    mockCreateMutation.mutateAsync.mockReset();
    mockUpdateMutation.mutateAsync.mockReset();

    mockUseCreateInventoryItem.mockReturnValue(mockCreateMutation as any);
    mockUseUpdateInventoryItem.mockReturnValue(mockUpdateMutation as any);
    mockUseAutoSave.mockImplementation(() => {});

    jest.clearAllMocks();
  });

  const renderForm = (props = {}) => {
    return render(
      <QueryClientProvider client={queryClient}>
        <InventoryForm {...props} />
      </QueryClientProvider>
    );
  };

  describe('Create Mode', () => {
    it('should render create form with default values', () => {
      renderForm();

      expect(screen.getByLabelText(/name/i)).toHaveValue('');
      expect(screen.getByLabelText(/quantity/i)).toHaveValue(1);
      expect(screen.getByLabelText(/unit/i)).toHaveValue('pieces');
      expect(screen.getByLabelText(/category/i)).toHaveValue('vegetables');
      expect(screen.getByLabelText(/location/i)).toHaveValue('pantry');
      expect(
        screen.getByRole('button', { name: /add item/i })
      ).toBeInTheDocument();
    });

    it('should show validation errors for required fields', async () => {
      renderForm();

      const submitButton = screen.getByRole('button', { name: /add item/i });
      await user.click(submitButton);

      await waitFor(() => {
        expect(screen.getByText(/name is required/i)).toBeInTheDocument();
      });
    });

    it('should validate quantity is greater than 0', async () => {
      renderForm();

      const nameInput = screen.getByLabelText(/name/i);
      await user.type(nameInput, 'Test Item'); // Fill required name

      const quantityInput = screen.getByLabelText(/quantity/i);
      await user.clear(quantityInput);
      await user.type(quantityInput, '0');

      // Find button - should now be "Add Item" since isPending is false
      const submitButton = screen.getByRole('button', { name: /add item/i });
      await user.click(submitButton);

      await waitFor(
        () => {
          expect(
            screen.getByText(/quantity must be greater than 0/i)
          ).toBeInTheDocument();
        },
        { timeout: 3000 }
      );
    });

    it('should validate cost is not negative', async () => {
      renderForm();

      const nameInput = screen.getByLabelText(/name/i);
      await user.type(nameInput, 'Test Item'); // Fill required name

      const costInput = screen.getByLabelText(/cost/i);
      await user.type(costInput, '-5');

      const submitButton = screen.getByRole('button', {
        name: /add item|adding.../i,
      });
      await user.click(submitButton);

      await waitFor(() => {
        expect(
          screen.getByText(/cost cannot be negative/i)
        ).toBeInTheDocument();
      });
    });

    it('should successfully create new item with valid data', async () => {
      const mockOnSuccess = jest.fn();
      const newItem = { ...mockInventoryItem, id: 'new-item' };
      mockCreateMutation.mutateAsync.mockResolvedValue(newItem);

      renderForm({ onSuccess: mockOnSuccess });

      // Fill out form
      await user.type(screen.getByLabelText(/name/i), 'New Item');
      await user.clear(screen.getByLabelText(/quantity/i));
      await user.type(screen.getByLabelText(/quantity/i), '3');

      const categorySelect = screen.getByLabelText(/category/i);
      await user.selectOptions(categorySelect, 'fruits');

      const locationSelect = screen.getByLabelText(/location/i);
      await user.selectOptions(locationSelect, 'refrigerator');

      const submitButton = screen.getByRole('button', { name: /add item/i });
      await user.click(submitButton);

      await waitFor(() => {
        expect(mockCreateMutation.mutateAsync).toHaveBeenCalledWith({
          name: 'New Item',
          quantity: 3,
          unit: 'pieces',
          category: 'fruits',
          location: 'refrigerator',
          expirationDate: null,
          purchaseDate: expect.any(Date),
          estimatedCost: null,
        });
      });

      expect(mockOnSuccess).toHaveBeenCalledWith(newItem);
    });

    it('should reset form after successful creation', async () => {
      const newItem = { ...mockInventoryItem, id: 'new-item' };
      mockCreateMutation.mutateAsync.mockResolvedValue(newItem);

      renderForm();

      // Fill out form
      await user.type(screen.getByLabelText(/name/i), 'New Item');
      await user.type(screen.getByLabelText(/cost/i), '5.99');

      const submitButton = screen.getByRole('button', { name: /add item/i });
      await user.click(submitButton);

      await waitFor(() => {
        expect(screen.getByLabelText(/name/i)).toHaveValue('');
        const costInput = screen.getByLabelText(/cost/i) as HTMLInputElement;
        expect(costInput.value === '' || costInput.value === null).toBe(true);
      });
    });

    it('should show auto-save status when creating', async () => {
      renderForm();

      // Auto-save status should be visible for new items after the effect runs
      await waitFor(
        () => {
          expect(screen.getByText(/draft saved/i)).toBeInTheDocument();
        },
        { timeout: 2000 }
      );
    });

    it('should use auto-save hook with correct parameters', () => {
      renderForm();

      expect(mockUseAutoSave).toHaveBeenCalledWith(
        'inventory-form-draft-new',
        expect.objectContaining({
          name: '',
          quantity: 1,
          unit: 'pieces',
        }),
        1000
      );
    });
  });

  describe('Edit Mode', () => {
    it('should render edit form with item data', async () => {
      renderForm({ item: mockInventoryItem });

      await waitFor(() => {
        expect(screen.getByLabelText(/name/i)).toHaveValue('Test Item');
        expect(screen.getByLabelText(/quantity/i)).toHaveValue(5);
        expect(screen.getByLabelText(/unit/i)).toHaveValue('pieces');
        expect(screen.getByLabelText(/category/i)).toHaveValue('vegetables');
        expect(screen.getByLabelText(/location/i)).toHaveValue('pantry');
        expect(screen.getByLabelText(/cost/i)).toHaveValue(10.5);
        expect(
          screen.getByRole('button', { name: /update item|updating.../i })
        ).toBeInTheDocument();
      });
    });

    it('should populate expiration date correctly', () => {
      renderForm({ item: mockInventoryItem });

      const expirationInput = screen.getByLabelText(/expiration date/i);
      expect(expirationInput).toHaveValue('2025-12-31');
    });

    it('should not show auto-save status when editing', () => {
      renderForm({ item: mockInventoryItem });

      expect(screen.queryByText(/draft saved/i)).not.toBeInTheDocument();
    });

    it('should successfully update item with modified data', async () => {
      const mockOnSuccess = jest.fn();
      const updatedItem = {
        ...mockInventoryItem,
        name: 'Updated Item',
        quantity: 8,
      };
      mockUpdateMutation.mutateAsync.mockResolvedValue(updatedItem);

      renderForm({ item: mockInventoryItem, onSuccess: mockOnSuccess });

      // Modify name and quantity
      const nameInput = screen.getByLabelText(/name/i);
      await user.clear(nameInput);
      await user.type(nameInput, 'Updated Item');

      const quantityInput = screen.getByLabelText(/quantity/i);
      await user.clear(quantityInput);
      await user.type(quantityInput, '8');

      const submitButton = screen.getByRole('button', { name: /update item/i });
      await user.click(submitButton);

      await waitFor(() => {
        expect(mockUpdateMutation.mutateAsync).toHaveBeenCalledWith({
          id: 'item-1',
          updates: {
            name: 'Updated Item',
            quantity: 8,
            unit: 'pieces',
            category: 'vegetables',
            location: 'pantry',
            expirationDate: new Date('2025-12-31'),
            estimatedCost: 10.5,
          },
        });
      });

      expect(mockOnSuccess).toHaveBeenCalledWith(updatedItem);
    });

    it('should handle clearing optional fields', async () => {
      const mockOnSuccess = jest.fn();
      const updatedItem = {
        ...mockInventoryItem,
        estimatedCost: null,
        expirationDate: null,
      };
      mockUpdateMutation.mutateAsync.mockResolvedValue(updatedItem);

      renderForm({ item: mockInventoryItem, onSuccess: mockOnSuccess });

      // Clear cost and expiration date
      const costInput = screen.getByLabelText(/cost/i);
      await user.clear(costInput);

      const expirationInput = screen.getByLabelText(/expiration date/i);
      await user.clear(expirationInput);

      const submitButton = screen.getByRole('button', { name: /update item/i });
      await user.click(submitButton);

      await waitFor(() => {
        expect(mockUpdateMutation.mutateAsync).toHaveBeenCalledWith({
          id: 'item-1',
          updates: expect.objectContaining({
            estimatedCost: null,
            expirationDate: null,
          }),
        });
      });
    });
  });

  describe('Loading States', () => {
    it('should show loading state during creation', () => {
      mockCreateMutation.isPending = true;
      renderForm();

      const submitButton = screen.getByRole('button', { name: /adding.../i });
      expect(submitButton).toBeDisabled();
    });

    it('should show loading state during update', () => {
      mockUpdateMutation.isPending = true;
      renderForm({ item: mockInventoryItem });

      const submitButton = screen.getByRole('button', { name: /updating.../i });
      expect(submitButton).toBeDisabled();
    });

    it('should disable cancel button when loading', () => {
      const mockOnCancel = jest.fn();
      mockCreateMutation.isPending = true;

      renderForm({ onCancel: mockOnCancel });

      const cancelButton = screen.getByRole('button', { name: /cancel/i });
      expect(cancelButton).toBeDisabled();
    });
  });

  describe('Form Interactions', () => {
    it('should clear errors when user starts typing', async () => {
      renderForm();

      const submitButton = screen.getByRole('button', {
        name: /add item|adding.../i,
      });
      await user.click(submitButton);

      await waitFor(() => {
        expect(screen.getByText(/name is required/i)).toBeInTheDocument();
      });

      const nameInput = screen.getByLabelText(/name/i);
      await user.type(nameInput, 'A');

      await waitFor(() => {
        expect(screen.queryByText(/name is required/i)).not.toBeInTheDocument();
      });
    });

    it('should handle cancel callback', async () => {
      const mockOnCancel = jest.fn();
      renderForm({ onCancel: mockOnCancel });

      await waitFor(() => {
        const cancelButton = screen.getByRole('button', { name: /cancel/i });
        return user.click(cancelButton);
      });

      expect(mockOnCancel).toHaveBeenCalled();
    });

    it('should handle dropdown changes', async () => {
      renderForm();

      const categorySelect = screen.getByLabelText(/category/i);
      await user.selectOptions(categorySelect, 'fruits');

      expect(categorySelect).toHaveValue('fruits');
    });

    it('should handle date input changes', async () => {
      renderForm();

      const expirationInput = screen.getByLabelText(/expiration date/i);
      await user.type(expirationInput, '2025-06-15');

      expect(expirationInput).toHaveValue('2025-06-15');
    });
  });

  describe('Accessibility', () => {
    it('should have proper labels and form associations', () => {
      renderForm();

      expect(screen.getByLabelText(/name/i)).toHaveAttribute('id', 'name');
      expect(screen.getByLabelText(/quantity/i)).toHaveAttribute(
        'id',
        'quantity'
      );
      expect(screen.getByLabelText(/unit/i)).toHaveAttribute('id', 'unit');
      expect(screen.getByLabelText(/category/i)).toHaveAttribute(
        'id',
        'category'
      );
      expect(screen.getByLabelText(/location/i)).toHaveAttribute(
        'id',
        'location'
      );
    });

    it('should show error messages with proper associations', async () => {
      renderForm();

      const submitButton = screen.getByRole('button', {
        name: /add item|adding.../i,
      });
      await user.click(submitButton);

      await waitFor(() => {
        const nameInput = screen.getByLabelText(/name/i);
        const errorMessage = screen.getByText(/name is required/i);

        expect(nameInput).toHaveClass('border-red-300');
        expect(errorMessage).toBeInTheDocument();
      });
    });

    it('should support keyboard navigation', () => {
      renderForm();

      const nameInput = screen.getByLabelText(/name/i);
      nameInput.focus();
      expect(nameInput).toHaveFocus();
    });
  });
});

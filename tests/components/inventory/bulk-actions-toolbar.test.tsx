import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { BulkActionsToolbar } from '@/components/inventory/bulk-actions-toolbar';
import { InventoryService } from '@/lib/services/inventory-service';
import { CustomCategory } from '@/types/inventory';

// Mock InventoryService
jest.mock('@/lib/services/inventory-service');
const mockInventoryService = InventoryService as jest.Mocked<
  typeof InventoryService
>;

const mockCustomCategories: CustomCategory[] = [
  {
    id: 'custom-1',
    name: 'Supplements',
    color: '#8b5cf6',
    icon: 'pill',
    householdId: 'household-1',
    createdBy: 'user-1',
    createdAt: new Date(),
    updatedAt: new Date(),
  },
];

const createWrapper = () => {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
      mutations: { retry: false },
    },
  });

  const TestWrapper = ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  );
  TestWrapper.displayName = 'TestWrapper';

  return TestWrapper;
};

describe('BulkActionsToolbar', () => {
  const defaultProps = {
    selectedItemIds: ['item-1', 'item-2'],
    onClearSelection: jest.fn(),
  };

  beforeEach(() => {
    jest.clearAllMocks();
    mockInventoryService.bulkUpdateItems = jest.fn().mockResolvedValue({});
  });

  it('renders null when no items selected', () => {
    const { container } = render(
      <BulkActionsToolbar {...defaultProps} selectedItemIds={[]} />,
      { wrapper: createWrapper() }
    );

    expect(container.firstChild).toBeNull();
  });

  it('displays correct selection count', () => {
    render(<BulkActionsToolbar {...defaultProps} />, {
      wrapper: createWrapper(),
    });

    expect(screen.getByText('2 items selected')).toBeInTheDocument();
  });

  it('displays singular form for single selection', () => {
    render(
      <BulkActionsToolbar {...defaultProps} selectedItemIds={['item-1']} />,
      { wrapper: createWrapper() }
    );

    expect(screen.getByText('1 item selected')).toBeInTheDocument();
  });

  it('shows category dropdown when category button clicked', () => {
    render(<BulkActionsToolbar {...defaultProps} />, {
      wrapper: createWrapper(),
    });

    fireEvent.click(screen.getByText('Change Category'));

    expect(screen.getByText('Predefined')).toBeInTheDocument();
    expect(screen.getByText('Proteins')).toBeInTheDocument();
    expect(screen.getByText('Vegetables')).toBeInTheDocument();
  });

  it('shows location dropdown when location button clicked', () => {
    render(<BulkActionsToolbar {...defaultProps} />, {
      wrapper: createWrapper(),
    });

    fireEvent.click(screen.getByText('Change Location'));

    expect(screen.getByText('Pantry')).toBeInTheDocument();
    expect(screen.getByText('Refrigerator')).toBeInTheDocument();
    expect(screen.getByText('Freezer')).toBeInTheDocument();
  });

  it('displays custom categories when provided', () => {
    render(
      <BulkActionsToolbar
        {...defaultProps}
        customCategories={mockCustomCategories}
      />,
      { wrapper: createWrapper() }
    );

    fireEvent.click(screen.getByText('Change Category'));

    expect(screen.getByText('Custom')).toBeInTheDocument();
    expect(screen.getByText('Supplements')).toBeInTheDocument();
  });

  it('calls bulkUpdateItems when category selected', async () => {
    render(<BulkActionsToolbar {...defaultProps} />, {
      wrapper: createWrapper(),
    });

    fireEvent.click(screen.getByText('Change Category'));
    fireEvent.click(screen.getByText('Proteins'));

    await waitFor(() => {
      expect(mockInventoryService.bulkUpdateItems).toHaveBeenCalledWith({
        itemIds: ['item-1', 'item-2'],
        updates: { category: 'proteins' },
      });
    });
  });

  it('calls bulkUpdateItems when location selected', async () => {
    render(<BulkActionsToolbar {...defaultProps} />, {
      wrapper: createWrapper(),
    });

    fireEvent.click(screen.getByText('Change Location'));
    fireEvent.click(screen.getByText('Pantry'));

    await waitFor(() => {
      expect(mockInventoryService.bulkUpdateItems).toHaveBeenCalledWith({
        itemIds: ['item-1', 'item-2'],
        updates: { location: 'pantry' },
      });
    });
  });

  it('calls onClearSelection when clear button clicked', () => {
    render(<BulkActionsToolbar {...defaultProps} />, {
      wrapper: createWrapper(),
    });

    fireEvent.click(screen.getByTitle('Clear selection'));

    expect(defaultProps.onClearSelection).toHaveBeenCalled();
  });

  it('shows loading state during bulk update', async () => {
    mockInventoryService.bulkUpdateItems = jest.fn(
      () => new Promise(resolve => setTimeout(resolve, 100))
    );

    render(<BulkActionsToolbar {...defaultProps} />, {
      wrapper: createWrapper(),
    });

    fireEvent.click(screen.getByText('Change Category'));
    fireEvent.click(screen.getByText('Proteins'));

    await waitFor(() => {
      expect(screen.getByText('Updating...')).toBeInTheDocument();
    });
  });

  it('disables buttons during bulk update', async () => {
    mockInventoryService.bulkUpdateItems = jest.fn(
      () => new Promise(resolve => setTimeout(resolve, 100))
    );

    render(<BulkActionsToolbar {...defaultProps} />, {
      wrapper: createWrapper(),
    });

    fireEvent.click(screen.getByText('Change Category'));
    fireEvent.click(screen.getByText('Proteins'));

    await waitFor(() => {
      expect(screen.getByText('Change Category')).toBeDisabled();
      expect(screen.getByText('Change Location')).toBeDisabled();
    });
  });

  it('handles bulk update errors gracefully', async () => {
    const consoleSpy = jest.spyOn(console, 'error').mockImplementation();
    mockInventoryService.bulkUpdateItems = jest
      .fn()
      .mockRejectedValue(new Error('Update failed'));

    render(<BulkActionsToolbar {...defaultProps} />, {
      wrapper: createWrapper(),
    });

    fireEvent.click(screen.getByText('Change Category'));
    fireEvent.click(screen.getByText('Proteins'));

    await waitFor(() => {
      expect(consoleSpy).toHaveBeenCalledWith(
        'Failed to update categories:',
        expect.any(Error)
      );
    });

    consoleSpy.mockRestore();
  });
});

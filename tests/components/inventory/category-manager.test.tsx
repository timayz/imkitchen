import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { CategoryManager } from '@/components/inventory/category-manager';
import {
  useCategories,
  useCreateCategory,
  useUpdateCategory,
  useDeleteCategory,
} from '@/hooks/use-categories';
import { CustomCategory } from '@/types/inventory';

// Mock the hooks
jest.mock('@/hooks/use-categories');
const mockUseCategories = useCategories as jest.MockedFunction<
  typeof useCategories
>;
const mockUseCreateCategory = useCreateCategory as jest.MockedFunction<
  typeof useCreateCategory
>;
const mockUseUpdateCategory = useUpdateCategory as jest.MockedFunction<
  typeof useUpdateCategory
>;
const mockUseDeleteCategory = useDeleteCategory as jest.MockedFunction<
  typeof useDeleteCategory
>;

const mockCategories: CustomCategory[] = [
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
  {
    id: 'cat-2',
    name: 'Snacks',
    color: '#f59e0b',
    icon: 'star',
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

describe('CategoryManager', () => {
  const mockMutateAsync = jest.fn();
  const defaultMutationReturn = {
    mutateAsync: mockMutateAsync,
    isPending: false,
    isError: false,
    error: null,
  };

  beforeEach(() => {
    jest.clearAllMocks();

    mockUseCategories.mockReturnValue({
      data: mockCategories,
      isLoading: false,
      error: null,
    });

    mockUseCreateCategory.mockReturnValue(defaultMutationReturn);
    mockUseUpdateCategory.mockReturnValue(defaultMutationReturn);
    mockUseDeleteCategory.mockReturnValue(defaultMutationReturn);

    // Mock window.confirm
    global.confirm = jest.fn(() => true);
  });

  afterEach(() => {
    delete (global as typeof global & { confirm?: unknown }).confirm;
  });

  it('displays loading state', () => {
    mockUseCategories.mockReturnValue({
      data: [],
      isLoading: true,
      error: null,
    });

    render(<CategoryManager />, { wrapper: createWrapper() });

    expect(document.querySelector('.animate-spin')).toBeInTheDocument();
  });

  it('displays error state', () => {
    mockUseCategories.mockReturnValue({
      data: [],
      isLoading: false,
      error: new Error('Failed to fetch'),
    });

    render(<CategoryManager />, { wrapper: createWrapper() });

    expect(
      screen.getByText('Failed to load categories. Please try again.')
    ).toBeInTheDocument();
  });

  it('displays existing categories', () => {
    render(<CategoryManager />, { wrapper: createWrapper() });

    expect(screen.getByText('Manage Categories')).toBeInTheDocument();
    expect(screen.getByText('Custom Categories (2)')).toBeInTheDocument();
    expect(screen.getByText('Supplements')).toBeInTheDocument();
    expect(screen.getByText('Snacks')).toBeInTheDocument();
  });

  it('shows empty state when no categories exist', () => {
    mockUseCategories.mockReturnValue({
      data: [],
      isLoading: false,
      error: null,
    });

    render(<CategoryManager />, { wrapper: createWrapper() });

    expect(screen.getByText('No custom categories yet.')).toBeInTheDocument();
    expect(
      screen.getByText('Create your first custom category to get started.')
    ).toBeInTheDocument();
  });

  it('shows create form when add button clicked', () => {
    render(<CategoryManager />, { wrapper: createWrapper() });

    fireEvent.click(screen.getByText('Add Custom Category'));

    expect(screen.getByText('Create New Category')).toBeInTheDocument();
    expect(
      screen.getByPlaceholderText('Enter category name')
    ).toBeInTheDocument();
    expect(screen.getByText('Color')).toBeInTheDocument();
    expect(screen.getByText('Icon')).toBeInTheDocument();
  });

  it('allows entering category name', () => {
    render(<CategoryManager />, { wrapper: createWrapper() });

    fireEvent.click(screen.getByText('Add Custom Category'));

    const nameInput = screen.getByPlaceholderText('Enter category name');
    fireEvent.change(nameInput, { target: { value: 'Test Category' } });

    expect(nameInput).toHaveValue('Test Category');
  });

  it('allows selecting color', () => {
    render(<CategoryManager />, { wrapper: createWrapper() });

    fireEvent.click(screen.getByText('Add Custom Category'));

    // Click on a color button (they are styled with background color)
    const colorButtons = screen
      .getAllByRole('button')
      .filter(btn => btn.style.backgroundColor);
    fireEvent.click(colorButtons[1]); // Click second color

    // The color selection should be visually indicated
    expect(colorButtons[1]).toHaveClass('border-gray-900');
  });

  it('allows selecting icon', () => {
    render(<CategoryManager />, { wrapper: createWrapper() });

    fireEvent.click(screen.getByText('Add Custom Category'));

    // Find and click an icon button
    const leafButton = screen.getByText('leaf');
    fireEvent.click(leafButton);

    expect(leafButton).toHaveClass('border-orange-500', 'bg-orange-50');
  });

  it('creates category when form submitted', async () => {
    render(<CategoryManager />, { wrapper: createWrapper() });

    fireEvent.click(screen.getByText('Add Custom Category'));

    const nameInput = screen.getByPlaceholderText('Enter category name');
    fireEvent.change(nameInput, { target: { value: 'New Category' } });

    fireEvent.click(screen.getByText('Create'));

    await waitFor(() => {
      expect(mockMutateAsync).toHaveBeenCalledWith({
        name: 'New Category',
        color: '#ef4444', // First default color
        icon: 'utensils', // First default icon
      });
    });
  });

  it('disables create button when name is empty', () => {
    render(<CategoryManager />, { wrapper: createWrapper() });

    fireEvent.click(screen.getByText('Add Custom Category'));

    const createButton = screen.getByText('Create');
    expect(createButton).toBeDisabled();
  });

  it('cancels creation and resets form', () => {
    render(<CategoryManager />, { wrapper: createWrapper() });

    fireEvent.click(screen.getByText('Add Custom Category'));

    const nameInput = screen.getByPlaceholderText('Enter category name');
    fireEvent.change(nameInput, { target: { value: 'Test' } });

    fireEvent.click(screen.getByText('Cancel'));

    expect(screen.queryByText('Create New Category')).not.toBeInTheDocument();
    expect(screen.getByText('Add Custom Category')).toBeInTheDocument();
  });

  it('enters edit mode when edit button clicked', () => {
    render(<CategoryManager />, { wrapper: createWrapper() });

    // Find edit button by looking for square-pen icon (the actual edit icon)
    const editButtons = screen.getAllByRole('button').filter(btn => {
      const svg = btn.querySelector('svg');
      return svg?.classList.contains('lucide-square-pen');
    });

    expect(editButtons).toHaveLength(2); // Two categories, so two edit buttons
    fireEvent.click(editButtons[0]); // Click first edit button (Supplements)

    expect(screen.getByDisplayValue('Supplements')).toBeInTheDocument();
  });

  it('updates category when edit form submitted', async () => {
    render(<CategoryManager />, { wrapper: createWrapper() });

    // Find edit button by looking for square-pen icon (the actual edit icon)
    const editButtons = screen.getAllByRole('button').filter(btn => {
      const svg = btn.querySelector('svg');
      return svg?.classList.contains('lucide-square-pen');
    });

    fireEvent.click(editButtons[0]); // Click first edit button (Supplements)

    const nameInput = screen.getByDisplayValue('Supplements');
    fireEvent.change(nameInput, { target: { value: 'Updated Name' } });

    // Find save button by its icon (green button with Save icon)
    const saveButtons = screen.getAllByRole('button').filter(btn => {
      const svg = btn.querySelector('svg');
      return (
        svg?.classList.contains('lucide-save') &&
        btn.className.includes('text-green-600')
      );
    });
    fireEvent.click(saveButtons[0]);

    await waitFor(() => {
      expect(mockMutateAsync).toHaveBeenCalledWith({
        id: 'cat-1',
        data: {
          name: 'Updated Name',
          color: '#8b5cf6',
          icon: 'pill',
        },
      });
    });
  });

  it('cancels editing when cancel button clicked', () => {
    render(<CategoryManager />, { wrapper: createWrapper() });

    // Find edit button by looking for square-pen icon (the actual edit icon)
    const editButtons = screen.getAllByRole('button').filter(btn => {
      const svg = btn.querySelector('svg');
      return svg?.classList.contains('lucide-square-pen');
    });

    fireEvent.click(editButtons[0]); // Click first edit button (Supplements)

    // Find cancel button (X icon with gray styling) after edit mode is activated
    const cancelButtons = screen.getAllByRole('button').filter(btn => {
      const svg = btn.querySelector('svg');
      return (
        svg?.classList.contains('lucide-x') &&
        btn.className.includes('text-gray-600')
      );
    });

    expect(cancelButtons.length).toBeGreaterThan(0);
    fireEvent.click(cancelButtons[0]);

    expect(screen.queryByDisplayValue('Supplements')).not.toBeInTheDocument();
    expect(screen.getByText('Supplements')).toBeInTheDocument();
  });

  it('deletes category when confirmed', async () => {
    render(<CategoryManager />, { wrapper: createWrapper() });

    const deleteButtons = screen.getAllByRole('button').filter(btn => {
      const svg = btn.querySelector('svg');
      return svg?.classList.contains('lucide-trash-2');
    });

    fireEvent.click(deleteButtons[0]);

    await waitFor(() => {
      expect(mockMutateAsync).toHaveBeenCalledWith('cat-1');
    });
  });

  it('does not delete category when not confirmed', async () => {
    global.confirm = jest.fn(() => false);

    render(<CategoryManager />, { wrapper: createWrapper() });

    const deleteButtons = screen.getAllByRole('button').filter(btn => {
      const svg = btn.querySelector('svg');
      return svg?.classList.contains('lucide-trash-2');
    });

    fireEvent.click(deleteButtons[0]);

    expect(mockMutateAsync).not.toHaveBeenCalled();
  });

  it('calls onClose when close button clicked', () => {
    const onClose = jest.fn();
    render(<CategoryManager onClose={onClose} />, { wrapper: createWrapper() });

    const closeButtons = screen.getAllByRole('button').filter(btn => {
      const svg = btn.querySelector('svg');
      return svg?.classList.contains('lucide-x');
    });

    fireEvent.click(closeButtons[0]);

    expect(onClose).toHaveBeenCalled();
  });

  it('shows loading states during mutations', () => {
    mockUseCreateCategory.mockReturnValue({
      ...defaultMutationReturn,
      isPending: true,
    });

    render(<CategoryManager />, { wrapper: createWrapper() });

    fireEvent.click(screen.getByText('Add Custom Category'));

    expect(screen.getByText('Add Custom Category')).toBeDisabled();
  });
});

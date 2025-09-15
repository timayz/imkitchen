import { render, screen } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { CategoryStatsWidget } from '@/components/inventory/category-stats-widget';
import { useCategoryStats } from '@/hooks/use-categories';
import { CategoryStats, CustomCategory } from '@/types/inventory';

// Mock the hook
jest.mock('@/hooks/use-categories');
const mockUseCategoryStats = useCategoryStats as jest.MockedFunction<
  typeof useCategoryStats
>;

const mockStats: CategoryStats[] = [
  {
    category: 'proteins',
    itemCount: 5,
    expiringThisWeek: 2,
    expiringToday: 1,
    totalValue: 50.5,
    lastUpdated: new Date(),
  },
  {
    category: 'vegetables',
    itemCount: 8,
    expiringThisWeek: 3,
    expiringToday: 0,
    totalValue: 25.0,
    lastUpdated: new Date(),
  },
  {
    category: 'custom-1',
    itemCount: 3,
    expiringThisWeek: 1,
    expiringToday: 0,
    totalValue: 15.0,
    lastUpdated: new Date(),
  },
];

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

describe('CategoryStatsWidget', () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });

  it('displays loading state', () => {
    mockUseCategoryStats.mockReturnValue({
      data: [],
      isLoading: true,
      error: null,
    });

    render(<CategoryStatsWidget />, { wrapper: createWrapper() });

    expect(document.querySelector('.animate-spin')).toBeInTheDocument();
  });

  it('displays error state', () => {
    mockUseCategoryStats.mockReturnValue({
      data: [],
      isLoading: false,
      error: new Error('Failed to fetch'),
    });

    render(<CategoryStatsWidget />, { wrapper: createWrapper() });

    expect(screen.getByText('Failed to load statistics')).toBeInTheDocument();
  });

  it('displays overall inventory stats', () => {
    mockUseCategoryStats.mockReturnValue({
      data: mockStats,
      isLoading: false,
      error: null,
    });

    render(<CategoryStatsWidget />, { wrapper: createWrapper() });

    expect(screen.getByText('Inventory Overview')).toBeInTheDocument();
    expect(screen.getByText('16')).toBeInTheDocument(); // Total items
    expect(screen.getByText('6')).toBeInTheDocument(); // Expiring this week
    expect(screen.getByText('1')).toBeInTheDocument(); // Expiring today
    expect(screen.getByText('$91')).toBeInTheDocument(); // Total value
  });

  it('displays category breakdown for all view', () => {
    mockUseCategoryStats.mockReturnValue({
      data: mockStats,
      isLoading: false,
      error: null,
    });

    render(<CategoryStatsWidget customCategories={mockCustomCategories} />, {
      wrapper: createWrapper(),
    });

    expect(screen.getByText('Category Breakdown')).toBeInTheDocument();
    expect(screen.getByText('Vegetables')).toBeInTheDocument();
    expect(screen.getByText('Proteins')).toBeInTheDocument();
    expect(screen.getByText('Supplements')).toBeInTheDocument(); // Custom category
  });

  it('displays filtered stats for specific category', () => {
    mockUseCategoryStats.mockReturnValue({
      data: mockStats,
      isLoading: false,
      error: null,
    });

    render(<CategoryStatsWidget selectedCategory="proteins" />, {
      wrapper: createWrapper(),
    });

    expect(screen.getByText('Proteins Stats')).toBeInTheDocument();
    expect(screen.getByText('5')).toBeInTheDocument(); // Total items for proteins
    expect(screen.getByText('2')).toBeInTheDocument(); // Expiring this week for proteins
    expect(screen.getByText('$51')).toBeInTheDocument(); // Total value for proteins
  });

  it('displays custom category name correctly', () => {
    mockUseCategoryStats.mockReturnValue({
      data: mockStats,
      isLoading: false,
      error: null,
    });

    render(
      <CategoryStatsWidget
        selectedCategory="custom-1"
        customCategories={mockCustomCategories}
      />,
      { wrapper: createWrapper() }
    );

    expect(screen.getByText('Supplements Stats')).toBeInTheDocument();
  });

  it('shows expiring items warnings in breakdown', () => {
    mockUseCategoryStats.mockReturnValue({
      data: mockStats,
      isLoading: false,
      error: null,
    });

    render(<CategoryStatsWidget />, { wrapper: createWrapper() });

    expect(screen.getByText('3 expiring')).toBeInTheDocument(); // Vegetables
    expect(screen.getByText('2 expiring')).toBeInTheDocument(); // Proteins
    expect(screen.getByText('1 urgent')).toBeInTheDocument(); // Proteins today
  });

  it('sorts categories by item count in breakdown', () => {
    mockUseCategoryStats.mockReturnValue({
      data: mockStats,
      isLoading: false,
      error: null,
    });

    render(<CategoryStatsWidget />, { wrapper: createWrapper() });

    const categoryElements = screen.getAllByText(/\d+ items?/);
    expect(categoryElements[0]).toHaveTextContent('8 items'); // Vegetables first (highest count)
    expect(categoryElements[1]).toHaveTextContent('5 items'); // Proteins second
    expect(categoryElements[2]).toHaveTextContent('3 items'); // Custom last
  });

  it('displays empty state when no data', () => {
    mockUseCategoryStats.mockReturnValue({
      data: [],
      isLoading: false,
      error: null,
    });

    render(<CategoryStatsWidget />, { wrapper: createWrapper() });

    expect(screen.getByText('No data available')).toBeInTheDocument();
    expect(screen.getByText('Add items to see statistics')).toBeInTheDocument();
  });

  it('handles single item count correctly', () => {
    const singleItemStats: CategoryStats[] = [
      {
        category: 'proteins',
        itemCount: 1,
        expiringThisWeek: 0,
        expiringToday: 0,
        totalValue: 10.0,
        lastUpdated: new Date(),
      },
    ];

    mockUseCategoryStats.mockReturnValue({
      data: singleItemStats,
      isLoading: false,
      error: null,
    });

    render(<CategoryStatsWidget />, { wrapper: createWrapper() });

    expect(screen.getByText('1 item')).toBeInTheDocument(); // Singular form
  });

  it('applies custom className', () => {
    mockUseCategoryStats.mockReturnValue({
      data: [],
      isLoading: false,
      error: null,
    });

    const { container } = render(
      <CategoryStatsWidget className="custom-class" />,
      { wrapper: createWrapper() }
    );

    expect(container.firstChild).toHaveClass('custom-class');
  });
});

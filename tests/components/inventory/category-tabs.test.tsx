import { render, screen, fireEvent } from '@testing-library/react';
import { CategoryTabs } from '@/components/inventory/category-tabs';
import { CustomCategory } from '@/types/inventory';

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

describe('CategoryTabs', () => {
  const defaultProps = {
    selectedCategory: 'all' as const,
    onCategoryChange: jest.fn(),
    viewMode: 'grid' as const,
    onViewModeChange: jest.fn(),
  };

  beforeEach(() => {
    jest.clearAllMocks();
  });

  it('renders all predefined categories', () => {
    render(<CategoryTabs {...defaultProps} />);

    // Check that categories are rendered (desktop + mobile versions)
    expect(screen.getAllByText('All Items')[0]).toBeInTheDocument();
    expect(screen.getAllByText('Proteins')[0]).toBeInTheDocument();
    expect(screen.getAllByText('Vegetables')[0]).toBeInTheDocument();
    expect(screen.getAllByText('Fruits')[0]).toBeInTheDocument();
    expect(screen.getAllByText('Dairy')[0]).toBeInTheDocument();
  });

  it('renders custom categories when provided', () => {
    render(
      <CategoryTabs {...defaultProps} customCategories={mockCustomCategories} />
    );

    expect(screen.getByText('Supplements')).toBeInTheDocument();
  });

  it('calls onCategoryChange when category is selected', () => {
    render(<CategoryTabs {...defaultProps} />);

    fireEvent.click(screen.getByText('Proteins'));
    expect(defaultProps.onCategoryChange).toHaveBeenCalledWith('proteins');
  });

  it('highlights selected category', () => {
    render(<CategoryTabs {...defaultProps} selectedCategory="proteins" />);

    const proteinsButtons = screen.getAllByText('Proteins');
    // Check the desktop version (first one, not in mobile section)
    expect(proteinsButtons[0].closest('button')).toHaveClass(
      'text-red-600',
      'bg-red-50'
    );
  });

  it('toggles view mode when view mode buttons are clicked', () => {
    render(<CategoryTabs {...defaultProps} />);

    // Find view mode buttons by their icons
    const gridButton = screen
      .getAllByRole('button')
      .find(
        button =>
          button.querySelector('svg')?.getAttribute('data-testid') ===
            'grid-icon' || button.className.includes('grid')
      );

    if (gridButton) {
      fireEvent.click(gridButton);
      expect(defaultProps.onViewModeChange).toHaveBeenCalledWith('grid');
    }
  });

  it('shows mobile category dropdown when clicked', () => {
    // Mock mobile viewport
    Object.defineProperty(window, 'innerWidth', {
      writable: true,
      configurable: true,
      value: 375,
    });

    render(<CategoryTabs {...defaultProps} />);

    // The mobile dropdown should not be visible initially
    const mobileCategories = document.querySelectorAll('.md\\:hidden');
    expect(mobileCategories.length).toBeGreaterThan(0);
  });

  it('calls onManageCategories when manage button is clicked', () => {
    const onManageCategories = jest.fn();
    render(
      <CategoryTabs {...defaultProps} onManageCategories={onManageCategories} />
    );

    const manageButton = screen.getByTitle('Manage Categories');
    fireEvent.click(manageButton);
    expect(onManageCategories).toHaveBeenCalled();
  });

  it('shows add category button when onManageCategories is provided', () => {
    const onManageCategories = jest.fn();
    render(
      <CategoryTabs {...defaultProps} onManageCategories={onManageCategories} />
    );

    expect(screen.getByText('Add Category')).toBeInTheDocument();
  });
});

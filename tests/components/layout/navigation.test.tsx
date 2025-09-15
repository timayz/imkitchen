import { render, screen, fireEvent } from '@testing-library/react';
import { useSession } from 'next-auth/react';
import { usePathname } from 'next/navigation';
import { useTranslations } from 'next-intl';
import { Navigation } from '@/components/layout/navigation';

// Mock dependencies
jest.mock('next-auth/react');
jest.mock('next/navigation');
jest.mock('next-intl');
jest.mock('@/hooks/use-navigation');
jest.mock('@/components/ui/language-selector');
jest.mock('@/components/layout/mobile-menu');
jest.mock('@/components/layout/user-dropdown');

const mockUseSession = useSession as jest.MockedFunction<typeof useSession>;
const mockUsePathname = usePathname as jest.MockedFunction<typeof usePathname>;
const mockUseTranslations = useTranslations as jest.MockedFunction<
  typeof useTranslations
>;

// Mock navigation hook
const mockNavigation = {
  isMenuOpen: false,
  currentSection: 'dashboard' as const,
  breadcrumbs: [],
  userDropdownOpen: false,
  toggleMobileMenu: jest.fn(),
  closeMobileMenu: jest.fn(),
  toggleUserDropdown: jest.fn(),
  closeUserDropdown: jest.fn(),
  setCurrentSection: jest.fn(),
  setBreadcrumbs: jest.fn(),
  closeAllMenus: jest.fn(),
};

jest.mock('@/hooks/use-navigation', () => ({
  useNavigation: () => mockNavigation,
}));

// Mock child components
jest.mock('@/components/ui/language-selector', () => ({
  LanguageSelector: () => (
    <div data-testid="language-selector">Language Selector</div>
  ),
}));

jest.mock('@/components/layout/mobile-menu', () => ({
  MobileMenu: ({ isOpen }: { isOpen: boolean }) =>
    isOpen ? <div data-testid="mobile-menu">Mobile Menu</div> : null,
}));

jest.mock('@/components/layout/user-dropdown', () => ({
  UserDropdown: () => <div data-testid="user-dropdown">User Dropdown</div>,
}));

describe('Navigation', () => {
  const mockT = Object.assign(
    jest.fn((key: string) => key),
    {
      rich: jest.fn((key: string) => key),
      markup: jest.fn((key: string) => key),
      raw: jest.fn((key: string) => key),
      has: jest.fn(() => true),
    }
  );

  beforeEach(() => {
    mockUseSession.mockReturnValue({
      data: null,
      status: 'unauthenticated',
      update: jest.fn(),
    });
    mockUsePathname.mockReturnValue('/en/dashboard');
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    mockUseTranslations.mockReturnValue(mockT as any);
    jest.clearAllMocks();
  });

  it('renders navigation header with logo', () => {
    render(<Navigation />);

    expect(screen.getByText('iK')).toBeInTheDocument();
    expect(screen.getByText('imkitchen')).toBeInTheDocument();
  });

  it('renders main navigation sections', () => {
    render(<Navigation />);

    expect(screen.getByText('navigation.dashboard')).toBeInTheDocument();
    expect(screen.getByText('navigation.inventory')).toBeInTheDocument();
    expect(screen.getByText('navigation.recipes')).toBeInTheDocument();
    expect(screen.getByText('navigation.mealPlanning')).toBeInTheDocument();
    expect(screen.getByText('navigation.shopping')).toBeInTheDocument();
  });

  it('renders language selector', () => {
    render(<Navigation />);

    expect(screen.getByTestId('language-selector')).toBeInTheDocument();
  });

  it('renders auth links when not authenticated', () => {
    render(<Navigation />);

    expect(screen.getByText('auth.login')).toBeInTheDocument();
    expect(screen.getByText('auth.register')).toBeInTheDocument();
  });

  it('renders user dropdown when authenticated', () => {
    mockUseSession.mockReturnValue({
      data: {
        user: {
          id: '1',
          name: 'Test User',
          email: 'test@example.com',
          householdId: 'household-1',
          language: 'EN',
          timezone: 'UTC',
        },
        expires: '2024-01-01',
      },
      status: 'authenticated',
      update: jest.fn(),
    });

    render(<Navigation />);

    expect(screen.getByTestId('user-dropdown')).toBeInTheDocument();
    expect(screen.queryByText('auth.login')).not.toBeInTheDocument();
  });

  it('toggles mobile menu when hamburger button is clicked', () => {
    render(<Navigation />);

    const menuButton = screen.getByLabelText('navigation.toggleMenu');
    fireEvent.click(menuButton);

    expect(mockNavigation.toggleMobileMenu).toHaveBeenCalled();
  });

  it('highlights current navigation section', () => {
    mockUsePathname.mockReturnValue('/en/inventory');

    render(<Navigation />);

    const inventoryLink = screen.getByText('navigation.inventory').closest('a');
    expect(inventoryLink).toHaveClass('bg-orange-100', 'text-orange-700');
  });

  it('shows mobile menu when isMenuOpen is true', () => {
    mockNavigation.isMenuOpen = true;

    render(<Navigation />);

    expect(screen.getByTestId('mobile-menu')).toBeInTheDocument();
  });

  it('has proper accessibility attributes', () => {
    render(<Navigation />);

    const menuButton = screen.getByLabelText('navigation.toggleMenu');
    expect(menuButton).toHaveAttribute('aria-expanded', 'false');
    expect(menuButton).toHaveAttribute('aria-controls', 'mobile-menu');
  });
});

export interface NavigationState {
  isMenuOpen: boolean;
  currentSection:
    | 'dashboard'
    | 'inventory'
    | 'recipes'
    | 'meal-planning'
    | 'shopping';
  breadcrumbs: BreadcrumbItem[];
  userDropdownOpen: boolean;
}

export interface BreadcrumbItem {
  label: string;
  href: string;
  isCurrentPage?: boolean;
}

export interface NavigationSection {
  key: string;
  href: string;
  translationKey: string;
  icon?: string;
}

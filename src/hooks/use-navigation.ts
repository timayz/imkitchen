'use client';

import { useState, useCallback } from 'react';
import { NavigationState } from '@/types/navigation';

const initialState: NavigationState = {
  isMenuOpen: false,
  currentSection: 'dashboard',
  breadcrumbs: [],
  userDropdownOpen: false,
};

export function useNavigation() {
  const [state, setState] = useState<NavigationState>(initialState);

  const toggleMobileMenu = useCallback(() => {
    setState(prev => ({
      ...prev,
      isMenuOpen: !prev.isMenuOpen,
      userDropdownOpen: false, // Close user dropdown when opening menu
    }));
  }, []);

  const closeMobileMenu = useCallback(() => {
    setState(prev => ({
      ...prev,
      isMenuOpen: false,
    }));
  }, []);

  const toggleUserDropdown = useCallback(() => {
    setState(prev => ({
      ...prev,
      userDropdownOpen: !prev.userDropdownOpen,
      isMenuOpen: false, // Close mobile menu when opening dropdown
    }));
  }, []);

  const closeUserDropdown = useCallback(() => {
    setState(prev => ({
      ...prev,
      userDropdownOpen: false,
    }));
  }, []);

  const setCurrentSection = useCallback(
    (section: NavigationState['currentSection']) => {
      setState(prev => ({
        ...prev,
        currentSection: section,
      }));
    },
    []
  );

  const setBreadcrumbs = useCallback(
    (breadcrumbs: NavigationState['breadcrumbs']) => {
      setState(prev => ({
        ...prev,
        breadcrumbs,
      }));
    },
    []
  );

  const closeAllMenus = useCallback(() => {
    setState(prev => ({
      ...prev,
      isMenuOpen: false,
      userDropdownOpen: false,
    }));
  }, []);

  return {
    ...state,
    toggleMobileMenu,
    closeMobileMenu,
    toggleUserDropdown,
    closeUserDropdown,
    setCurrentSection,
    setBreadcrumbs,
    closeAllMenus,
  };
}

'use client';

import { useRef, useEffect } from 'react';
import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { useSession, signOut } from 'next-auth/react';
import { useTranslations } from 'next-intl';
import { ChevronDown, User, Settings, LogOut } from 'lucide-react';
import { useNavigation } from '@/hooks/use-navigation';
import { LanguageSelector } from '@/components/ui/language-selector';

export function UserDropdown() {
  const { data: session } = useSession();
  const pathname = usePathname();
  const t = useTranslations();
  const navigation = useNavigation();
  const dropdownRef = useRef<HTMLDivElement>(null);

  // Extract locale from pathname
  const locale = pathname.split('/')[1] || 'en';

  // Close dropdown when clicking outside
  useEffect(() => {
    function handleClickOutside(event: MouseEvent) {
      if (
        dropdownRef.current &&
        !dropdownRef.current.contains(event.target as Node)
      ) {
        navigation.closeUserDropdown();
      }
    }

    if (navigation.userDropdownOpen) {
      document.addEventListener('mousedown', handleClickOutside);
      return () =>
        document.removeEventListener('mousedown', handleClickOutside);
    }

    return undefined;
  }, [navigation.userDropdownOpen, navigation]);

  // Close dropdown on escape key
  useEffect(() => {
    function handleEscape(event: KeyboardEvent) {
      if (event.key === 'Escape') {
        navigation.closeUserDropdown();
      }
    }

    if (navigation.userDropdownOpen) {
      document.addEventListener('keydown', handleEscape);
      return () => document.removeEventListener('keydown', handleEscape);
    }

    return undefined;
  }, [navigation.userDropdownOpen, navigation]);

  const handleSignOut = async () => {
    navigation.closeUserDropdown();
    await signOut({ callbackUrl: `/${locale}/` });
  };

  const handleLinkClick = () => {
    navigation.closeUserDropdown();
  };

  if (!session?.user) return null;

  return (
    <div className="relative" ref={dropdownRef}>
      {/* Trigger Button */}
      <button
        onClick={navigation.toggleUserDropdown}
        className="flex items-center space-x-2 px-3 py-2 rounded-md text-sm font-medium text-gray-700 hover:text-gray-900 hover:bg-gray-100 focus:outline-none focus:ring-2 focus:ring-orange-500 focus:ring-offset-2 transition-colors duration-200"
        aria-expanded={navigation.userDropdownOpen}
        aria-haspopup="true"
        aria-label={t('navigation.userMenu')}
        style={{ minHeight: '44px' }} // Touch-friendly target
      >
        <div className="h-8 w-8 rounded-full bg-orange-500 flex items-center justify-center">
          <User className="h-5 w-5 text-white" />
        </div>
        <span className="hidden lg:block max-w-32 truncate">
          {session.user.name || session.user.email}
        </span>
        <ChevronDown
          className={`h-4 w-4 transition-transform duration-200 ${
            navigation.userDropdownOpen ? 'rotate-180' : ''
          }`}
        />
      </button>

      {/* Dropdown Menu */}
      {navigation.userDropdownOpen && (
        <div className="absolute right-0 mt-2 w-64 bg-white rounded-md shadow-lg ring-1 ring-black ring-opacity-5 focus:outline-none z-50">
          <div className="py-1" role="menu" aria-orientation="vertical">
            {/* User Info */}
            <div className="px-4 py-3 border-b border-gray-100">
              <p className="text-sm font-medium text-gray-900 truncate">
                {session.user.name || t('navigation.user')}
              </p>
              <p className="text-sm text-gray-500 truncate">
                {session.user.email}
              </p>
            </div>

            {/* Navigation Links */}
            <div className="py-1">
              <Link
                href={`/${locale}/settings/profile`}
                onClick={handleLinkClick}
                className="flex items-center px-4 py-2 text-sm text-gray-700 hover:bg-gray-100 hover:text-gray-900 focus:outline-none focus:ring-2 focus:ring-orange-500 focus:ring-offset-2 focus:ring-offset-white"
                role="menuitem"
                style={{ minHeight: '44px' }}
              >
                <Settings className="h-4 w-4 mr-3" />
                {t('navigation.settings')}
              </Link>
            </div>

            {/* Language Selection */}
            <div className="px-4 py-2 border-t border-gray-100">
              <div className="flex items-center justify-between">
                <span className="text-sm text-gray-700">
                  {t('navigation.language')}
                </span>
                <LanguageSelector />
              </div>
            </div>

            {/* Sign Out */}
            <div className="py-1 border-t border-gray-100">
              <button
                onClick={handleSignOut}
                className="flex items-center w-full px-4 py-2 text-sm text-gray-700 hover:bg-gray-100 hover:text-gray-900 focus:outline-none focus:ring-2 focus:ring-orange-500 focus:ring-offset-2 focus:ring-offset-white"
                role="menuitem"
                style={{ minHeight: '44px' }}
              >
                <LogOut className="h-4 w-4 mr-3" />
                {t('auth.logout')}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

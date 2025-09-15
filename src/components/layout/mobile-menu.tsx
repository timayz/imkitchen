'use client';

import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { useTranslations } from 'next-intl';
import { Session } from 'next-auth';
import { signOut } from 'next-auth/react';
import { X, User, Settings, LogOut } from 'lucide-react';
import { LanguageSelector } from '@/components/ui/language-selector';
import { NavigationSection } from '@/types/navigation';

interface MobileMenuProps {
  isOpen: boolean;
  onClose: () => void;
  navigationSections: NavigationSection[];
  currentLocale: string;
  session: Session | null;
}

export function MobileMenu({
  isOpen,
  onClose,
  navigationSections,
  currentLocale,
  session,
}: MobileMenuProps) {
  const pathname = usePathname();
  const t = useTranslations();

  const isCurrentSection = (href: string) => {
    const pathWithoutLocale = pathname.replace(`/${currentLocale}`, '') || '/';
    return (
      pathWithoutLocale.startsWith(href) ||
      (href === '/dashboard' && pathWithoutLocale === '/')
    );
  };

  const handleLinkClick = () => {
    onClose();
  };

  const handleSignOut = async () => {
    onClose();
    await signOut({ callbackUrl: `/${currentLocale}/` });
  };

  if (!isOpen) return null;

  return (
    <>
      {/* Backdrop */}
      <div
        className="fixed inset-0 z-40 bg-black bg-opacity-50 md:hidden"
        onClick={onClose}
        aria-hidden="true"
      />

      {/* Mobile menu panel */}
      <div
        id="mobile-menu"
        className="fixed inset-y-0 right-0 z-50 w-full max-w-sm bg-white shadow-xl md:hidden transform transition-transform duration-300 ease-in-out"
        style={{ transform: isOpen ? 'translateX(0)' : 'translateX(100%)' }}
      >
        <div className="flex flex-col h-full">
          {/* Header */}
          <div className="flex items-center justify-between p-4 border-b">
            <h2 className="text-lg font-semibold text-gray-900">
              {t('navigation.menu')}
            </h2>
            <button
              onClick={onClose}
              className="p-2 rounded-md text-gray-600 hover:text-gray-900 hover:bg-gray-100 focus:outline-none focus:ring-2 focus:ring-orange-500 focus:ring-offset-2"
              aria-label={t('navigation.closeMenu')}
              style={{ minHeight: '44px', minWidth: '44px' }}
            >
              <X className="h-6 w-6" aria-hidden="true" />
            </button>
          </div>

          {/* User Section */}
          {session?.user && (
            <div className="p-4 border-b bg-gray-50">
              <div className="flex items-center space-x-3">
                <div className="h-10 w-10 rounded-full bg-orange-500 flex items-center justify-center">
                  <User className="h-6 w-6 text-white" />
                </div>
                <div className="flex-1 min-w-0">
                  <p className="text-sm font-medium text-gray-900 truncate">
                    {session.user.name || session.user.email}
                  </p>
                  <p className="text-sm text-gray-500 truncate">
                    {session.user.email}
                  </p>
                </div>
              </div>
            </div>
          )}

          {/* Navigation Links */}
          <nav className="flex-1 px-4 py-6 space-y-1 overflow-y-auto">
            {navigationSections.map(section => (
              <Link
                key={section.key}
                href={`/${currentLocale}${section.href}`}
                onClick={handleLinkClick}
                className={`block px-3 py-3 rounded-md text-base font-medium transition-colors duration-200 focus:outline-none focus:ring-2 focus:ring-orange-500 focus:ring-offset-2 ${
                  isCurrentSection(section.href)
                    ? 'bg-orange-100 text-orange-700'
                    : 'text-gray-600 hover:text-gray-900 hover:bg-gray-100'
                }`}
                style={{ minHeight: '44px' }} // Touch-friendly target
                aria-current={
                  isCurrentSection(section.href) ? 'page' : undefined
                }
              >
                {t(section.translationKey)}
              </Link>
            ))}

            {/* Auth Links for non-authenticated users */}
            {!session?.user && (
              <div className="pt-4 space-y-1">
                <Link
                  href={`/${currentLocale}/login`}
                  onClick={handleLinkClick}
                  className="block px-3 py-3 rounded-md text-base font-medium text-gray-600 hover:text-gray-900 hover:bg-gray-100 focus:outline-none focus:ring-2 focus:ring-orange-500 focus:ring-offset-2"
                  style={{ minHeight: '44px' }}
                >
                  {t('auth.login')}
                </Link>
                <Link
                  href={`/${currentLocale}/register`}
                  onClick={handleLinkClick}
                  className="block px-3 py-3 rounded-md text-base font-medium bg-orange-500 text-white hover:bg-orange-600 focus:outline-none focus:ring-2 focus:ring-orange-500 focus:ring-offset-2"
                  style={{ minHeight: '44px' }}
                >
                  {t('auth.register')}
                </Link>
              </div>
            )}

            {/* User Actions for authenticated users */}
            {session?.user && (
              <div className="pt-4 space-y-1 border-t">
                <Link
                  href={`/${currentLocale}/settings/profile`}
                  onClick={handleLinkClick}
                  className="flex items-center px-3 py-3 rounded-md text-base font-medium text-gray-600 hover:text-gray-900 hover:bg-gray-100 focus:outline-none focus:ring-2 focus:ring-orange-500 focus:ring-offset-2"
                  style={{ minHeight: '44px' }}
                >
                  <Settings className="h-5 w-5 mr-3" />
                  {t('navigation.settings')}
                </Link>
                <button
                  onClick={handleSignOut}
                  className="flex items-center w-full px-3 py-3 rounded-md text-base font-medium text-gray-600 hover:text-gray-900 hover:bg-gray-100 focus:outline-none focus:ring-2 focus:ring-orange-500 focus:ring-offset-2"
                  style={{ minHeight: '44px' }}
                >
                  <LogOut className="h-5 w-5 mr-3" />
                  {t('auth.logout')}
                </button>
              </div>
            )}
          </nav>

          {/* Footer */}
          <div className="p-4 border-t">
            <div className="flex items-center justify-between">
              <span className="text-sm text-gray-500">
                {t('navigation.language')}
              </span>
              <LanguageSelector />
            </div>
          </div>
        </div>
      </div>
    </>
  );
}

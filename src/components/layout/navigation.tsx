'use client';

import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { useSession } from 'next-auth/react';
import { useTranslations } from 'next-intl';
import { Menu, X } from 'lucide-react';
import { useNavigation } from '@/hooks/use-navigation';
import { LanguageSelector } from '@/components/ui/language-selector';
import { MobileMenu } from './mobile-menu';
import { UserDropdown } from './user-dropdown';
import { NavigationSection } from '@/types/navigation';

const navigationSections: NavigationSection[] = [
  {
    key: 'dashboard',
    href: '/dashboard',
    translationKey: 'navigation.dashboard',
  },
  {
    key: 'inventory',
    href: '/inventory',
    translationKey: 'navigation.inventory',
  },
  { key: 'recipes', href: '/recipes', translationKey: 'navigation.recipes' },
  {
    key: 'meal-planning',
    href: '/meal-planning',
    translationKey: 'navigation.mealPlanning',
  },
  { key: 'shopping', href: '/shopping', translationKey: 'navigation.shopping' },
];

export function Navigation() {
  const pathname = usePathname();
  const { data: session } = useSession();
  const t = useTranslations();
  const navigation = useNavigation();

  // Extract locale from pathname
  const locale = pathname.split('/')[1] || 'en';

  const isCurrentSection = (href: string) => {
    // Remove locale prefix for comparison
    const pathWithoutLocale = pathname.replace(`/${locale}`, '') || '/';
    return (
      pathWithoutLocale.startsWith(href) ||
      (href === '/dashboard' && pathWithoutLocale === '/')
    );
  };

  return (
    <>
      <header className="sticky top-0 z-50 w-full border-b bg-white/95 backdrop-blur supports-[backdrop-filter]:bg-white/60">
        <div className="container mx-auto px-4">
          <div className="flex h-16 items-center justify-between">
            {/* Logo */}
            <div className="flex items-center space-x-4">
              <Link
                href={`/${locale}/dashboard`}
                className="flex items-center space-x-2"
              >
                <div className="h-8 w-8 rounded-lg bg-orange-500 flex items-center justify-center">
                  <span className="text-white font-bold text-lg">iK</span>
                </div>
                <span className="text-xl font-bold text-gray-900 hidden sm:block">
                  imkitchen
                </span>
              </Link>
            </div>

            {/* Desktop Navigation */}
            <nav className="hidden md:flex items-center space-x-1">
              {navigationSections.map(section => (
                <Link
                  key={section.key}
                  href={`/${locale}${section.href}`}
                  className={`px-3 py-2 rounded-md text-sm font-medium transition-colors duration-200 focus:outline-none focus:ring-2 focus:ring-orange-500 focus:ring-offset-2 ${
                    isCurrentSection(section.href)
                      ? 'bg-orange-100 text-orange-700'
                      : 'text-gray-600 hover:text-gray-900 hover:bg-gray-100'
                  }`}
                  aria-current={
                    isCurrentSection(section.href) ? 'page' : undefined
                  }
                >
                  {t(section.translationKey)}
                </Link>
              ))}
            </nav>

            {/* Right Side Actions */}
            <div className="flex items-center space-x-2">
              {/* Language Selector */}
              <div className="hidden sm:block">
                <LanguageSelector />
              </div>

              {/* User Dropdown or Auth Links */}
              {session?.user ? (
                <UserDropdown />
              ) : (
                <div className="hidden sm:flex items-center space-x-2">
                  <Link
                    href={`/${locale}/login`}
                    className="px-3 py-2 text-sm font-medium text-gray-600 hover:text-gray-900 focus:outline-none focus:ring-2 focus:ring-orange-500 focus:ring-offset-2 rounded-md"
                  >
                    {t('auth.login')}
                  </Link>
                  <Link
                    href={`/${locale}/register`}
                    className="px-3 py-2 text-sm font-medium bg-orange-500 text-white hover:bg-orange-600 focus:outline-none focus:ring-2 focus:ring-orange-500 focus:ring-offset-2 rounded-md transition-colors duration-200"
                  >
                    {t('auth.register')}
                  </Link>
                </div>
              )}

              {/* Mobile Menu Button */}
              <button
                onClick={navigation.toggleMobileMenu}
                className="md:hidden inline-flex items-center justify-center p-2 rounded-md text-gray-600 hover:text-gray-900 hover:bg-gray-100 focus:outline-none focus:ring-2 focus:ring-orange-500 focus:ring-offset-2 transition-colors duration-200"
                aria-expanded={navigation.isMenuOpen}
                aria-controls="mobile-menu"
                aria-label={t('navigation.toggleMenu')}
                style={{ minHeight: '44px', minWidth: '44px' }} // Touch-friendly target
              >
                {navigation.isMenuOpen ? (
                  <X className="h-6 w-6" aria-hidden="true" />
                ) : (
                  <Menu className="h-6 w-6" aria-hidden="true" />
                )}
              </button>
            </div>
          </div>
        </div>
      </header>

      {/* Mobile Menu */}
      <MobileMenu
        isOpen={navigation.isMenuOpen}
        onClose={navigation.closeMobileMenu}
        navigationSections={navigationSections}
        currentLocale={locale}
        session={session}
      />
    </>
  );
}

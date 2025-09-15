'use client';

import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { useTranslations } from 'next-intl';
import { ChevronRight, Home } from 'lucide-react';
import { BreadcrumbItem } from '@/types/navigation';

interface BreadcrumbsProps {
  items?: BreadcrumbItem[];
  showHome?: boolean;
}

export function Breadcrumbs({ items, showHome = true }: BreadcrumbsProps) {
  const pathname = usePathname();
  const t = useTranslations();

  // Extract locale from pathname
  const locale = pathname.split('/')[1] || 'en';

  // Generate breadcrumbs from pathname if items not provided
  const breadcrumbItems =
    items || generateBreadcrumbsFromPath(pathname, locale, t);

  if (!breadcrumbItems.length && !showHome) {
    return null;
  }

  return (
    <nav className="flex" aria-label={t('navigation.breadcrumbs')}>
      <ol className="inline-flex items-center space-x-1 md:space-x-3">
        {showHome && (
          <li className="inline-flex items-center">
            <Link
              href={`/${locale}/dashboard`}
              className="inline-flex items-center text-sm font-medium text-gray-700 hover:text-orange-600 focus:outline-none focus:ring-2 focus:ring-orange-500 focus:ring-offset-2 rounded"
              aria-label={t('navigation.home')}
            >
              <Home className="h-4 w-4" />
            </Link>
          </li>
        )}

        {breadcrumbItems.map(item => (
          <li key={item.href} className="inline-flex items-center">
            <ChevronRight className="h-4 w-4 text-gray-400 mx-1" />
            {item.isCurrentPage ? (
              <span
                className="text-sm font-medium text-gray-500"
                aria-current="page"
              >
                {item.label}
              </span>
            ) : (
              <Link
                href={item.href}
                className="text-sm font-medium text-gray-700 hover:text-orange-600 focus:outline-none focus:ring-2 focus:ring-orange-500 focus:ring-offset-2 rounded"
              >
                {item.label}
              </Link>
            )}
          </li>
        ))}
      </ol>
    </nav>
  );
}

function generateBreadcrumbsFromPath(
  pathname: string,
  locale: string,
  t: (key: string) => string
): BreadcrumbItem[] {
  // Remove locale from pathname
  const pathWithoutLocale = pathname.replace(`/${locale}`, '') || '/';

  // Split path into segments
  const segments = pathWithoutLocale.split('/').filter(Boolean);

  if (!segments.length) {
    return [];
  }

  const breadcrumbs: BreadcrumbItem[] = [];
  let currentPath = `/${locale}`;

  segments.forEach((segment, index) => {
    currentPath += `/${segment}`;
    const isLast = index === segments.length - 1;

    // Map segments to human-readable labels
    let label = segment;
    try {
      // Try to get translation for common navigation items
      const translationKey = `navigation.${segment}`;
      const translated = t(translationKey);
      if (translated !== translationKey) {
        label = translated;
      } else {
        // Format segment as title case
        label = segment
          .split('-')
          .map(word => word.charAt(0).toUpperCase() + word.slice(1))
          .join(' ');
      }
    } catch {
      // Fallback to formatted segment
      label = segment
        .split('-')
        .map(word => word.charAt(0).toUpperCase() + word.slice(1))
        .join(' ');
    }

    breadcrumbs.push({
      label,
      href: currentPath,
      isCurrentPage: isLast,
    });
  });

  return breadcrumbs;
}

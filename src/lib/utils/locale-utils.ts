import { cookies } from 'next/headers';
import {
  type Locale,
  defaultLocale,
  isValidLocale,
  getLocaleConfig,
} from '@/lib/i18n';
import type { LocaleDetectionResult } from '@/types/i18n';

/**
 * Detects the user's preferred locale from various sources
 */
export async function detectUserLocale(
  userLanguage?: string,
  acceptLanguage?: string
): Promise<LocaleDetectionResult> {
  // 1. Check user database preference
  if (userLanguage && isValidLocale(userLanguage)) {
    return {
      locale: userLanguage,
      source: 'user',
    };
  }

  // 2. Check cookie preference
  const cookieStore = await cookies();
  const cookieLocale = cookieStore.get('NEXT_LOCALE')?.value;
  if (cookieLocale && isValidLocale(cookieLocale)) {
    return {
      locale: cookieLocale,
      source: 'cookie',
    };
  }

  // 3. Check Accept-Language header
  if (acceptLanguage) {
    const browserLocales = acceptLanguage
      .split(',')
      .map(lang => lang.split(';')[0].trim().slice(0, 2));

    for (const browserLocale of browserLocales) {
      if (isValidLocale(browserLocale)) {
        return {
          locale: browserLocale,
          source: 'browser',
        };
      }
    }
  }

  // 4. Fallback to default locale
  return {
    locale: defaultLocale,
    source: 'default',
  };
}

/**
 * Sets the locale preference in a cookie
 */
export function setLocaleCookie(locale: Locale) {
  if (typeof document !== 'undefined') {
    document.cookie = `NEXT_LOCALE=${locale}; Path=/; Max-Age=31536000; SameSite=Lax`;
  }
}

/**
 * Formats a date according to the locale's format
 */
export function formatDate(date: Date, locale: Locale): string {
  getLocaleConfig(locale); // Keep for future use with custom date formats
  return new Intl.DateTimeFormat(locale, {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
  }).format(date);
}

/**
 * Formats a number according to the locale's format
 */
export function formatNumber(number: number, locale: Locale): string {
  const config = getLocaleConfig(locale);
  return new Intl.NumberFormat(locale, config.numberFormat).format(number);
}

/**
 * Formats a currency according to the locale's format
 */
export function formatCurrency(
  amount: number,
  locale: Locale,
  currency = 'USD'
): string {
  return new Intl.NumberFormat(locale, {
    style: 'currency',
    currency,
  }).format(amount);
}

/**
 * Formats a relative time (e.g., "2 hours ago")
 */
export function formatRelativeTime(date: Date, locale: Locale): string {
  const now = new Date();
  const diffInSeconds = Math.floor((now.getTime() - date.getTime()) / 1000);

  const rtf = new Intl.RelativeTimeFormat(locale, { numeric: 'auto' });

  if (diffInSeconds < 60) {
    return rtf.format(-diffInSeconds, 'second');
  } else if (diffInSeconds < 3600) {
    return rtf.format(-Math.floor(diffInSeconds / 60), 'minute');
  } else if (diffInSeconds < 86400) {
    return rtf.format(-Math.floor(diffInSeconds / 3600), 'hour');
  } else {
    return rtf.format(-Math.floor(diffInSeconds / 86400), 'day');
  }
}

/**
 * Validates and sanitizes a locale parameter for security
 */
export function sanitizeLocale(locale: string): Locale {
  // Remove any potential path traversal attempts
  const cleanLocale = locale.replace(/[^a-z]/gi, '').toLowerCase();

  // Validate against allowed locales
  if (isValidLocale(cleanLocale)) {
    return cleanLocale;
  }

  return defaultLocale;
}

/**
 * Gets the direction (LTR/RTL) for a locale
 */
export function getTextDirection(locale: Locale): 'ltr' | 'rtl' {
  const config = getLocaleConfig(locale);
  return config.direction;
}

/**
 * Creates a localized URL path
 */
export function createLocalizedPath(path: string, locale: Locale): string {
  // Remove leading slash if present
  const cleanPath = path.startsWith('/') ? path.slice(1) : path;

  // For default locale at root, don't add locale prefix
  if (cleanPath === '' && locale === defaultLocale) {
    return `/${locale}`;
  }

  return `/${locale}/${cleanPath}`;
}

/**
 * Extracts locale from a pathname
 */
export function extractLocaleFromPath(pathname: string): {
  locale: Locale;
  path: string;
} {
  const segments = pathname.split('/').filter(Boolean);

  if (segments.length > 0 && isValidLocale(segments[0])) {
    return {
      locale: segments[0],
      path: '/' + segments.slice(1).join('/'),
    };
  }

  return {
    locale: defaultLocale,
    path: pathname,
  };
}

import { cookies } from 'next/headers';
import { defaultLocale, isValidLocale } from '@/lib/i18n';
import type { LocaleDetectionResult } from '@/types/i18n';

/**
 * Detects the user's preferred locale from various sources (server-side only)
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

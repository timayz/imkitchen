import { getRequestConfig } from 'next-intl/server';
import { notFound } from 'next/navigation';

// Supported locales
export const locales = ['en', 'es', 'fr', 'de'] as const;
export type Locale = (typeof locales)[number];

// Default locale
export const defaultLocale: Locale = 'en';

// Locale configuration
export const localeConfig = {
  en: {
    code: 'en' as const,
    name: 'English',
    nativeName: 'English',
    direction: 'ltr' as const,
    dateFormat: 'MM/dd/yyyy',
    numberFormat: {
      style: 'decimal',
      minimumFractionDigits: 0,
      maximumFractionDigits: 2,
    } as Intl.NumberFormatOptions,
  },
  es: {
    code: 'es' as const,
    name: 'Spanish',
    nativeName: 'Español',
    direction: 'ltr' as const,
    dateFormat: 'dd/MM/yyyy',
    numberFormat: {
      style: 'decimal',
      minimumFractionDigits: 0,
      maximumFractionDigits: 2,
    } as Intl.NumberFormatOptions,
  },
  fr: {
    code: 'fr' as const,
    name: 'French',
    nativeName: 'Français',
    direction: 'ltr' as const,
    dateFormat: 'dd/MM/yyyy',
    numberFormat: {
      style: 'decimal',
      minimumFractionDigits: 0,
      maximumFractionDigits: 2,
    } as Intl.NumberFormatOptions,
  },
  de: {
    code: 'de' as const,
    name: 'German',
    nativeName: 'Deutsch',
    direction: 'ltr' as const,
    dateFormat: 'dd.MM.yyyy',
    numberFormat: {
      style: 'decimal',
      minimumFractionDigits: 0,
      maximumFractionDigits: 2,
    } as Intl.NumberFormatOptions,
  },
} as const;

// Check if a locale is valid
export function isValidLocale(locale: string): locale is Locale {
  return locales.includes(locale as Locale);
}

// Get locale configuration
export function getLocaleConfig(locale: Locale) {
  return localeConfig[locale];
}

// Get user locale from database language field
export function mapUserLanguageToLocale(language: string): Locale {
  if (isValidLocale(language)) {
    return language;
  }
  return defaultLocale;
}

// RTL language detection (for future use)
export function isRTLLocale(): boolean {
  // Currently all supported languages are LTR
  // This will be extended when Arabic/Hebrew support is added
  return false;
}

// next-intl configuration
export default getRequestConfig(async ({ locale }) => {
  const validLocale = locale && isValidLocale(locale) ? locale : defaultLocale;

  const { messages } = await import('../../messages');

  return {
    locale: validLocale,
    messages: messages[validLocale as keyof typeof messages],
  };
});

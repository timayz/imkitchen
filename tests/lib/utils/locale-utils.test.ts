import {
  formatDate,
  formatNumber,
  formatCurrency,
  formatRelativeTime,
  sanitizeLocale,
  getTextDirection,
  createLocalizedPath,
  extractLocaleFromPath,
} from '@/lib/utils/locale-utils';
import { defaultLocale } from '@/lib/i18n';

// Mock cookies for testing
jest.mock('next/headers', () => ({
  cookies: jest.fn(() => ({
    get: jest.fn(() => undefined),
  })),
}));

describe('locale-utils', () => {
  describe('formatDate', () => {
    const testDate = new Date('2025-09-15T10:30:00Z');

    it('should format date according to locale', () => {
      // Test different locale date formats
      expect(formatDate(testDate, 'en')).toMatch(/\d{2}\/\d{2}\/\d{4}/); // MM/dd/yyyy
      expect(formatDate(testDate, 'de')).toMatch(/\d{2}\.\d{2}\.\d{4}/); // dd.MM.yyyy
    });
  });

  describe('formatNumber', () => {
    it('should format numbers according to locale', () => {
      const number = 1234.56;

      // All current locales use similar number formatting
      expect(formatNumber(number, 'en')).toContain('1,234');
      expect(formatNumber(number, 'de')).toBeDefined();
    });
  });

  describe('formatCurrency', () => {
    it('should format currency according to locale', () => {
      const amount = 123.45;

      expect(formatCurrency(amount, 'en')).toContain('$');
      expect(formatCurrency(amount, 'de')).toContain('$');
    });

    it('should handle different currencies', () => {
      const amount = 100;

      expect(formatCurrency(amount, 'en', 'EUR')).toContain('€');
    });
  });

  describe('formatRelativeTime', () => {
    it('should format relative time correctly', () => {
      const now = new Date();
      const oneHourAgo = new Date(now.getTime() - 60 * 60 * 1000);

      const formatted = formatRelativeTime(oneHourAgo, 'en');
      expect(formatted).toContain('hour');
    });
  });

  describe('sanitizeLocale', () => {
    it('should return valid locale for clean input', () => {
      expect(sanitizeLocale('en')).toBe('en');
      expect(sanitizeLocale('es')).toBe('es');
    });

    it('should sanitize and return default for invalid input', () => {
      expect(sanitizeLocale('../en')).toBe(defaultLocale);
      expect(sanitizeLocale('en/../../')).toBe(defaultLocale);
      expect(sanitizeLocale('invalid123')).toBe(defaultLocale);
    });

    it('should handle empty input', () => {
      expect(sanitizeLocale('')).toBe(defaultLocale);
    });
  });

  describe('getTextDirection', () => {
    it('should return ltr for all current locales', () => {
      expect(getTextDirection('en')).toBe('ltr');
      expect(getTextDirection('es')).toBe('ltr');
      expect(getTextDirection('fr')).toBe('ltr');
      expect(getTextDirection('de')).toBe('ltr');
    });
  });

  describe('createLocalizedPath', () => {
    it('should create correct localized paths', () => {
      expect(createLocalizedPath('/dashboard', 'en')).toBe('/en/dashboard');
      expect(createLocalizedPath('dashboard', 'es')).toBe('/es/dashboard');
      expect(createLocalizedPath('/settings/profile', 'fr')).toBe(
        '/fr/settings/profile'
      );
    });

    it('should handle root path', () => {
      expect(createLocalizedPath('', 'en')).toBe('/en');
      expect(createLocalizedPath('/', 'de')).toBe('/de/');
    });
  });

  describe('extractLocaleFromPath', () => {
    it('should extract locale from valid paths', () => {
      expect(extractLocaleFromPath('/en/dashboard')).toEqual({
        locale: 'en',
        path: '/dashboard',
      });

      expect(extractLocaleFromPath('/es/settings/profile')).toEqual({
        locale: 'es',
        path: '/settings/profile',
      });
    });

    it('should return default locale for paths without locale', () => {
      expect(extractLocaleFromPath('/dashboard')).toEqual({
        locale: defaultLocale,
        path: '/dashboard',
      });

      expect(extractLocaleFromPath('/invalid/path')).toEqual({
        locale: defaultLocale,
        path: '/invalid/path',
      });
    });

    it('should handle root paths', () => {
      expect(extractLocaleFromPath('/en')).toEqual({
        locale: 'en',
        path: '/',
      });

      expect(extractLocaleFromPath('/')).toEqual({
        locale: defaultLocale,
        path: '/',
      });
    });
  });
});

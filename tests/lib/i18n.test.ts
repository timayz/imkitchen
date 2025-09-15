import {
  locales,
  defaultLocale,
  isValidLocale,
  getLocaleConfig,
  mapUserLanguageToLocale,
  isRTLLocale,
} from '@/lib/i18n';

describe('i18n utilities', () => {
  describe('isValidLocale', () => {
    it('should return true for valid locales', () => {
      locales.forEach(locale => {
        expect(isValidLocale(locale)).toBe(true);
      });
    });

    it('should return false for invalid locales', () => {
      expect(isValidLocale('invalid')).toBe(false);
      expect(isValidLocale('zh')).toBe(false);
      expect(isValidLocale('')).toBe(false);
    });
  });

  describe('getLocaleConfig', () => {
    it('should return config for each supported locale', () => {
      locales.forEach(locale => {
        const config = getLocaleConfig(locale);
        expect(config).toBeDefined();
        expect(config.code).toBe(locale);
        expect(config.name).toBeTruthy();
        expect(config.nativeName).toBeTruthy();
        expect(['ltr', 'rtl']).toContain(config.direction);
      });
    });

    it('should return correct native names', () => {
      expect(getLocaleConfig('en').nativeName).toBe('English');
      expect(getLocaleConfig('es').nativeName).toBe('Español');
      expect(getLocaleConfig('fr').nativeName).toBe('Français');
      expect(getLocaleConfig('de').nativeName).toBe('Deutsch');
    });
  });

  describe('mapUserLanguageToLocale', () => {
    it('should return valid locale for valid language', () => {
      expect(mapUserLanguageToLocale('en')).toBe('en');
      expect(mapUserLanguageToLocale('es')).toBe('es');
      expect(mapUserLanguageToLocale('fr')).toBe('fr');
      expect(mapUserLanguageToLocale('de')).toBe('de');
    });

    it('should return default locale for invalid language', () => {
      expect(mapUserLanguageToLocale('invalid')).toBe(defaultLocale);
      expect(mapUserLanguageToLocale('zh')).toBe(defaultLocale);
      expect(mapUserLanguageToLocale('')).toBe(defaultLocale);
    });
  });

  describe('isRTLLocale', () => {
    it('should return false for all current locales (LTR)', () => {
      locales.forEach(locale => {
        expect(isRTLLocale(locale)).toBe(false);
      });
    });
  });

  describe('default locale', () => {
    it('should be English', () => {
      expect(defaultLocale).toBe('en');
    });

    it('should be in the locales array', () => {
      expect(locales).toContain(defaultLocale);
    });
  });
});

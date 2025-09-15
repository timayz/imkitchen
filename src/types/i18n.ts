import { Locale } from '@/lib/i18n';

// Translation interfaces
export interface AuthTranslations {
  login: {
    title: string;
    email: string;
    password: string;
    submit: string;
    forgotPassword: string;
    noAccount: string;
    signUp: string;
    invalidCredentials: string;
  };
  register: {
    title: string;
    name: string;
    email: string;
    password: string;
    confirmPassword: string;
    language: string;
    dietaryPreferences: string;
    allergies: string;
    submit: string;
    hasAccount: string;
    signIn: string;
    passwordMismatch: string;
  };
  profile: {
    title: string;
    personalInfo: string;
    language: string;
    preferences: string;
    save: string;
    logout: string;
  };
  passwordReset: {
    title: string;
    email: string;
    submit: string;
    checkEmail: string;
    newPassword: string;
    confirmPassword: string;
    reset: string;
  };
}

export interface NavigationTranslations {
  dashboard: string;
  inventory: string;
  recipes: string;
  mealPlanning: string;
  shopping: string;
  cooking: string;
  settings: string;
  profile: string;
  logout: string;
}

export interface CommonTranslations {
  loading: string;
  error: string;
  success: string;
  cancel: string;
  save: string;
  delete: string;
  edit: string;
  add: string;
  search: string;
  filter: string;
  clear: string;
  close: string;
  back: string;
  next: string;
  previous: string;
  submit: string;
  required: string;
  optional: string;
  yes: string;
  no: string;
  confirm: string;
  language: {
    en: string;
    es: string;
    fr: string;
    de: string;
  };
}

export interface TranslationKeys {
  auth: AuthTranslations;
  navigation: NavigationTranslations;
  common: CommonTranslations;
}

export interface LocaleConfig {
  code: Locale;
  name: string;
  nativeName: string;
  direction: 'ltr' | 'rtl';
  dateFormat: string;
  numberFormat: Intl.NumberFormatOptions;
}

// Language selection option
export interface LanguageOption {
  code: Locale;
  name: string;
  nativeName: string;
  flag?: string;
}

// Locale detection types
export interface LocaleDetectionResult {
  locale: Locale;
  source: 'user' | 'cookie' | 'browser' | 'default';
}

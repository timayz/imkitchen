'use client';

import { createContext, useContext, ReactNode } from 'react';
import { type Locale, getLocaleConfig } from '@/lib/i18n';
import type { LocaleConfig } from '@/types/i18n';

interface LocaleContextType {
  locale: Locale;
  localeConfig: LocaleConfig;
}

const LocaleContext = createContext<LocaleContextType | undefined>(undefined);

interface LocaleProviderProps {
  locale: Locale;
  children: ReactNode;
}

export function LocaleProvider({ locale, children }: LocaleProviderProps) {
  const localeConfig = getLocaleConfig(locale);

  const value: LocaleContextType = {
    locale,
    localeConfig,
  };

  return (
    <LocaleContext.Provider value={value}>{children}</LocaleContext.Provider>
  );
}

export function useLocaleContext() {
  const context = useContext(LocaleContext);

  if (context === undefined) {
    throw new Error('useLocaleContext must be used within a LocaleProvider');
  }

  return context;
}

// Hook for getting locale-aware formatting functions
export function useLocaleFormatting() {
  const { locale, localeConfig } = useLocaleContext();

  const formatDate = (date: Date): string => {
    return new Intl.DateTimeFormat(locale, {
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
    }).format(date);
  };

  const formatNumber = (number: number): string => {
    return new Intl.NumberFormat(locale, localeConfig.numberFormat).format(
      number
    );
  };

  const formatCurrency = (amount: number, currency = 'USD'): string => {
    return new Intl.NumberFormat(locale, {
      style: 'currency',
      currency,
    }).format(amount);
  };

  const formatRelativeTime = (date: Date): string => {
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
  };

  return {
    locale,
    localeConfig,
    formatDate,
    formatNumber,
    formatCurrency,
    formatRelativeTime,
  };
}

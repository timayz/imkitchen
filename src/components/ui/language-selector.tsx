'use client';

import { useState, useTransition } from 'react';
import { useRouter, usePathname } from 'next/navigation';
import { useLocale, useTranslations } from 'next-intl';
import { locales, type Locale, getLocaleConfig } from '@/lib/i18n';
import { setLocaleCookie } from '@/lib/utils/locale-utils';

interface LanguageSelectorProps {
  className?: string;
}

export function LanguageSelector({ className = '' }: LanguageSelectorProps) {
  const [isOpen, setIsOpen] = useState(false);
  const [isPending, startTransition] = useTransition();
  const router = useRouter();
  const pathname = usePathname();
  const locale = useLocale() as Locale;
  const t = useTranslations('common');

  const currentLocaleConfig = getLocaleConfig(locale);

  const handleLanguageChange = (newLocale: Locale) => {
    if (newLocale === locale) {
      setIsOpen(false);
      return;
    }

    startTransition(() => {
      // Set the locale cookie
      setLocaleCookie(newLocale);

      // Create the new pathname with the new locale
      const segments = pathname.split('/').filter(Boolean);
      const newPathname = `/${newLocale}/${segments.slice(1).join('/')}`;

      // Navigate to the new locale
      router.push(newPathname);
      setIsOpen(false);
    });
  };

  return (
    <div className={`relative ${className}`}>
      <button
        type="button"
        className="flex items-center gap-2 px-3 py-2 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md shadow-sm hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-orange-500 disabled:opacity-50"
        onClick={() => setIsOpen(!isOpen)}
        disabled={isPending}
        aria-expanded={isOpen}
        aria-haspopup="listbox"
        aria-label="Select language"
      >
        <span className="flex items-center">
          <span className="block truncate">
            {currentLocaleConfig.nativeName}
          </span>
        </span>
        <svg
          className={`w-4 h-4 transition-transform duration-200 ${
            isOpen ? 'rotate-180' : ''
          }`}
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2}
            d="M19 9l-7 7-7-7"
          />
        </svg>
      </button>

      {isOpen && (
        <>
          {/* Backdrop */}
          <div
            className="fixed inset-0 z-10"
            onClick={() => setIsOpen(false)}
            onKeyDown={e => {
              if (e.key === 'Escape') {
                setIsOpen(false);
              }
            }}
            role="button"
            tabIndex={0}
            aria-label="Close language selector"
          />

          {/* Dropdown */}
          <div className="absolute right-0 z-20 w-48 mt-1 bg-white border border-gray-300 rounded-md shadow-lg">
            <div className="py-1" role="listbox" aria-label="Language options">
              {locales.map(localeOption => {
                const localeConfigOption = getLocaleConfig(localeOption);
                const isSelected = localeOption === locale;

                return (
                  <button
                    key={localeOption}
                    type="button"
                    className={`w-full text-left px-4 py-2 text-sm hover:bg-gray-100 focus:outline-none focus:bg-gray-100 ${
                      isSelected
                        ? 'bg-orange-50 text-orange-900'
                        : 'text-gray-900'
                    }`}
                    onClick={() => handleLanguageChange(localeOption)}
                    role="option"
                    aria-selected={isSelected}
                  >
                    <div className="flex items-center justify-between">
                      <div>
                        <div className="font-medium">
                          {localeConfigOption.nativeName}
                        </div>
                        <div className="text-xs text-gray-500">
                          {t(`language.${localeOption}`)}
                        </div>
                      </div>
                      {isSelected && (
                        <svg
                          className="w-4 h-4 text-orange-600"
                          fill="currentColor"
                          viewBox="0 0 20 20"
                        >
                          <path
                            fillRule="evenodd"
                            d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z"
                            clipRule="evenodd"
                          />
                        </svg>
                      )}
                    </div>
                  </button>
                );
              })}
            </div>
          </div>
        </>
      )}
    </div>
  );
}

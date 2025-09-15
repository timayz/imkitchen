'use client';

import { useTranslations } from 'next-intl';
import { ReactNode, ElementType } from 'react';

interface TranslatedTextProps {
  namespace?: string;
  translationKey: string;
  fallback?: string;
  values?: Record<string, string | number>;
  children?: ReactNode;
  className?: string;
  as?: ElementType;
}

export function TranslatedText({
  namespace = 'common',
  translationKey,
  fallback,
  values,
  children,
  className,
  as: Component = 'span',
}: TranslatedTextProps) {
  const t = useTranslations(namespace);

  try {
    const translatedText = t(translationKey, values);

    return (
      <Component className={className}>
        {translatedText}
        {children}
      </Component>
    );
  } catch {
    // Fallback handling for missing translations
    const displayText = fallback || translationKey;

    return (
      <Component className={className}>
        {displayText}
        {children}
      </Component>
    );
  }
}

// Convenience components for common HTML elements
export const TranslatedHeading = (props: Omit<TranslatedTextProps, 'as'>) => (
  <TranslatedText {...props} as="h1" />
);

export const TranslatedParagraph = (props: Omit<TranslatedTextProps, 'as'>) => (
  <TranslatedText {...props} as="p" />
);

export const TranslatedButton = (props: Omit<TranslatedTextProps, 'as'>) => (
  <TranslatedText {...props} as="button" />
);

export const TranslatedLabel = (props: Omit<TranslatedTextProps, 'as'>) => (
  <TranslatedText {...props} as="label" />
);

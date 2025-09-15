'use client';

import { ReactNode } from 'react';
import { useLocale } from 'next-intl';
import { getTextDirection } from '@/lib/utils/locale-utils';
import type { Locale } from '@/lib/i18n';

interface RTLWrapperProps {
  children: ReactNode;
  className?: string;
  style?: React.CSSProperties;
}

export function RTLWrapper({
  children,
  className = '',
  style = {},
}: RTLWrapperProps) {
  const locale = useLocale() as Locale;
  const direction = getTextDirection(locale);

  return (
    <div
      className={`${className}`}
      style={{
        direction,
        ...style,
      }}
      dir={direction}
    >
      {children}
    </div>
  );
}

// Hook for getting RTL-aware CSS classes
export function useRTLClasses() {
  const locale = useLocale() as Locale;
  const direction = getTextDirection(locale);
  const isRTL = direction === 'rtl';

  return {
    direction,
    isRTL,
    isLTR: !isRTL,
    // Conditional classes for RTL layouts
    textAlign: isRTL ? 'text-right' : 'text-left',
    marginLeft: isRTL ? 'mr-auto' : 'ml-auto',
    marginRight: isRTL ? 'ml-auto' : 'mr-auto',
    paddingLeft: isRTL ? 'pr-4' : 'pl-4',
    paddingRight: isRTL ? 'pl-4' : 'pr-4',
    borderLeft: isRTL ? 'border-r' : 'border-l',
    borderRight: isRTL ? 'border-l' : 'border-r',
    roundedLeft: isRTL ? 'rounded-r' : 'rounded-l',
    roundedRight: isRTL ? 'rounded-l' : 'rounded-r',
    // Flexbox direction helpers
    flexRow: isRTL ? 'flex-row-reverse' : 'flex-row',
    // Text direction utilities
    float: isRTL ? 'float-right' : 'float-left',
    clear: isRTL ? 'clear-right' : 'clear-left',
  };
}

// Component variants for RTL-compatible UI elements
interface RTLFlexProps {
  children: ReactNode;
  className?: string;
  reverse?: boolean;
}

export function RTLFlex({
  children,
  className = '',
  reverse = false,
}: RTLFlexProps) {
  const { flexRow } = useRTLClasses();
  const directionClass = reverse
    ? flexRow === 'flex-row'
      ? 'flex-row-reverse'
      : 'flex-row'
    : flexRow;

  return (
    <div className={`flex ${directionClass} ${className}`}>{children}</div>
  );
}

interface RTLTextProps {
  children: ReactNode;
  className?: string;
  align?: 'left' | 'right' | 'center' | 'justify';
}

export function RTLText({
  children,
  className = '',
  align = 'left',
}: RTLTextProps) {
  const { isRTL } = useRTLClasses();

  let textAlignClass = 'text-left';

  if (align === 'left') {
    textAlignClass = isRTL ? 'text-right' : 'text-left';
  } else if (align === 'right') {
    textAlignClass = isRTL ? 'text-left' : 'text-right';
  } else if (align === 'center') {
    textAlignClass = 'text-center';
  } else if (align === 'justify') {
    textAlignClass = 'text-justify';
  }

  return <div className={`${textAlignClass} ${className}`}>{children}</div>;
}

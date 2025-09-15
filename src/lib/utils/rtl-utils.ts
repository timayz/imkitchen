import type { Locale } from '@/lib/i18n';
import { getTextDirection } from './locale-utils';

/**
 * RTL-aware CSS utilities for consistent directional layouts
 */

export interface RTLAwareStyles {
  marginStart: string;
  marginEnd: string;
  paddingStart: string;
  paddingEnd: string;
  borderStart: string;
  borderEnd: string;
  roundedStart: string;
  roundedEnd: string;
  textAlign: string;
  float: string;
  clear: string;
}

/**
 * Get RTL-aware CSS classes based on locale
 */
export function getRTLAwareClasses(locale: Locale): RTLAwareStyles {
  const direction = getTextDirection(locale);
  const isRTL = direction === 'rtl';

  return {
    marginStart: isRTL ? 'mr-4' : 'ml-4',
    marginEnd: isRTL ? 'ml-4' : 'mr-4',
    paddingStart: isRTL ? 'pr-4' : 'pl-4',
    paddingEnd: isRTL ? 'pl-4' : 'pr-4',
    borderStart: isRTL ? 'border-r' : 'border-l',
    borderEnd: isRTL ? 'border-l' : 'border-r',
    roundedStart: isRTL ? 'rounded-r' : 'rounded-l',
    roundedEnd: isRTL ? 'rounded-l' : 'rounded-r',
    textAlign: isRTL ? 'text-right' : 'text-left',
    float: isRTL ? 'float-right' : 'float-left',
    clear: isRTL ? 'clear-right' : 'clear-left',
  };
}

/**
 * Get RTL-aware flex direction
 */
export function getRTLFlexDirection(locale: Locale, reverse = false): string {
  const direction = getTextDirection(locale);
  const isRTL = direction === 'rtl';

  if (reverse) {
    return isRTL ? 'flex-row' : 'flex-row-reverse';
  }

  return isRTL ? 'flex-row-reverse' : 'flex-row';
}

/**
 * Get RTL-aware positioning classes
 */
export function getRTLPositioning(locale: Locale) {
  const direction = getTextDirection(locale);
  const isRTL = direction === 'rtl';

  return {
    left: isRTL ? 'right-0' : 'left-0',
    right: isRTL ? 'left-0' : 'right-0',
    translateX: isRTL ? 'translate-x-full' : '-translate-x-full',
    dropdownAlign: isRTL ? 'left-0' : 'right-0',
  };
}

/**
 * Convert logical CSS properties to physical ones based on direction
 */
export function convertLogicalToPhysical(
  logicalProperty: string,
  value: string,
  locale: Locale
): Record<string, string> {
  const direction = getTextDirection(locale);
  const isRTL = direction === 'rtl';

  const logicalMap: Record<string, Record<string, string>> = {
    'margin-inline-start': {
      ltr: 'margin-left',
      rtl: 'margin-right',
    },
    'margin-inline-end': {
      ltr: 'margin-right',
      rtl: 'margin-left',
    },
    'padding-inline-start': {
      ltr: 'padding-left',
      rtl: 'padding-right',
    },
    'padding-inline-end': {
      ltr: 'padding-right',
      rtl: 'padding-left',
    },
    'border-inline-start': {
      ltr: 'border-left',
      rtl: 'border-right',
    },
    'border-inline-end': {
      ltr: 'border-right',
      rtl: 'border-left',
    },
    'border-start-start-radius': {
      ltr: 'border-top-left-radius',
      rtl: 'border-top-right-radius',
    },
    'border-start-end-radius': {
      ltr: 'border-top-right-radius',
      rtl: 'border-top-left-radius',
    },
    'border-end-start-radius': {
      ltr: 'border-bottom-left-radius',
      rtl: 'border-bottom-right-radius',
    },
    'border-end-end-radius': {
      ltr: 'border-bottom-right-radius',
      rtl: 'border-bottom-left-radius',
    },
  };

  const mapping = logicalMap[logicalProperty];
  if (!mapping) {
    return { [logicalProperty]: value };
  }

  const physicalProperty = isRTL ? mapping.rtl : mapping.ltr;
  return { [physicalProperty]: value };
}

/**
 * Helper to create RTL-aware animation keyframes
 */
export function createRTLAnimation(
  name: string,
  keyframes: Record<string, Record<string, string>>,
  locale: Locale
): string {
  const direction = getTextDirection(locale);
  const isRTL = direction === 'rtl';

  const processedKeyframes = Object.entries(keyframes)
    .map(([key, styles]) => {
      const processedStyles = Object.entries(styles)
        .map(([prop, value]) => {
          // Convert transform translateX for RTL
          if (
            prop === 'transform' &&
            typeof value === 'string' &&
            value.includes('translateX')
          ) {
            if (isRTL) {
              // Flip translateX values for RTL
              return `${prop}: ${value.replace(
                /translateX\(([^)]+)\)/,
                (_match, val) => {
                  const numericValue = parseFloat(val);
                  return `translateX(${-numericValue}${val.replace(/[-\d.]/g, '')})`;
                }
              )}`;
            }
          }

          // Convert logical properties
          const converted = convertLogicalToPhysical(prop, value, locale);
          return Object.entries(converted)
            .map(([p, v]) => `${p}: ${v}`)
            .join('; ');
        })
        .join('; ');

      return `${key} { ${processedStyles} }`;
    })
    .join(' ');

  return `@keyframes ${name}-${direction} { ${processedKeyframes} }`;
}

/**
 * Get icon rotation for RTL support (for directional icons like arrows)
 */
export function getRTLIconRotation(
  locale: Locale,
  iconType: 'arrow' | 'chevron' | 'caret' = 'arrow'
): string {
  const direction = getTextDirection(locale);
  const isRTL = direction === 'rtl';

  if (!isRTL) {
    return '';
  }

  // For RTL, rotate directional icons 180 degrees
  const rotationMap = {
    arrow: 'transform: scaleX(-1)',
    chevron: 'transform: scaleX(-1)',
    caret: 'transform: scaleX(-1)',
  };

  return rotationMap[iconType];
}

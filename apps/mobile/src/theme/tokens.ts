/**
 * Design System Tokens
 * Standardized color palette, typography, and spacing tokens
 * Supporting both light and dark themes with accessibility compliance (WCAG 2.1 AA)
 */

export interface ColorTokens {
  // Primary Colors
  primary: string;
  primaryLight: string;
  primaryDark: string;
  
  // Secondary Colors
  secondary: string;
  secondaryLight: string;
  secondaryDark: string;
  
  // Background Colors
  background: string;
  backgroundSecondary: string;
  backgroundTertiary: string;
  surface: string;
  surfaceVariant: string;
  
  // Text Colors
  text: string;
  textSecondary: string;
  textTertiary: string;
  textInverse: string;
  
  // Action Colors
  success: string;
  error: string;
  warning: string;
  info: string;
  
  // Border Colors
  border: string;
  borderLight: string;
  borderFocus: string;
  
  // Interactive States
  pressed: string;
  hover: string;
  disabled: string;
  
  // Overlay Colors
  overlay: string;
  scrim: string;
  
  // Status Colors
  online: string;
  offline: string;
  syncing: string;
}

export interface TypographyTokens {
  // Font Families
  fontFamily: string;
  fontFamilyMono: string;
  
  // Font Sizes
  fontSize: {
    xs: number;
    sm: number;
    base: number;
    lg: number;
    xl: number;
    '2xl': number;
    '3xl': number;
    '4xl': number;
  };
  
  // Line Heights
  lineHeight: {
    tight: number;
    normal: number;
    relaxed: number;
  };
  
  // Font Weights
  fontWeight: {
    normal: string;
    medium: string;
    semibold: string;
    bold: string;
  };
}

export interface SpacingTokens {
  xs: number;
  sm: number;
  md: number;
  lg: number;
  xl: number;
  '2xl': number;
  '3xl': number;
  '4xl': number;
}

export interface BorderRadiusTokens {
  none: number;
  sm: number;
  md: number;
  lg: number;
  xl: number;
  full: number;
}

export interface ShadowTokens {
  none: string;
  sm: string;
  md: string;
  lg: string;
  xl: string;
}

// Light Theme Colors
export const lightColors: ColorTokens = {
  // Primary - Warm Kitchen Green
  primary: '#4CAF50',
  primaryLight: '#81C784',
  primaryDark: '#388E3C',
  
  // Secondary - Warm Orange
  secondary: '#FF9800',
  secondaryLight: '#FFB74D',
  secondaryDark: '#F57C00',
  
  // Backgrounds
  background: '#FFFFFF',
  backgroundSecondary: '#F8F9FA',
  backgroundTertiary: '#F0F2F5',
  surface: '#FFFFFF',
  surfaceVariant: '#F5F5F5',
  
  // Text
  text: '#1A1A1A',
  textSecondary: '#4A4A4A',
  textTertiary: '#6B6B6B',
  textInverse: '#FFFFFF',
  
  // Actions
  success: '#4CAF50',
  error: '#F44336',
  warning: '#FF9800',
  info: '#2196F3',
  
  // Borders
  border: '#E0E0E0',
  borderLight: '#F0F0F0',
  borderFocus: '#2196F3',
  
  // Interactive
  pressed: 'rgba(0, 0, 0, 0.08)',
  hover: 'rgba(0, 0, 0, 0.04)',
  disabled: '#E0E0E0',
  
  // Overlays
  overlay: 'rgba(0, 0, 0, 0.5)',
  scrim: 'rgba(0, 0, 0, 0.3)',
  
  // Status
  online: '#4CAF50',
  offline: '#9E9E9E',
  syncing: '#FF9800',
};

// Dark Theme Colors
export const darkColors: ColorTokens = {
  // Primary - Softer Kitchen Green for dark mode
  primary: '#66BB6A',
  primaryLight: '#81C784',
  primaryDark: '#4CAF50',
  
  // Secondary - Warmer Orange for dark mode  
  secondary: '#FFB74D',
  secondaryLight: '#FFCC02',
  secondaryDark: '#FF9800',
  
  // Backgrounds
  background: '#121212',
  backgroundSecondary: '#1E1E1E',
  backgroundTertiary: '#2A2A2A',
  surface: '#1E1E1E',
  surfaceVariant: '#2A2A2A',
  
  // Text
  text: '#FFFFFF',
  textSecondary: '#B0B0B0',
  textTertiary: '#808080',
  textInverse: '#000000',
  
  // Actions
  success: '#66BB6A',
  error: '#EF5350',
  warning: '#FFB74D',
  info: '#42A5F5',
  
  // Borders
  border: '#404040',
  borderLight: '#303030',
  borderFocus: '#42A5F5',
  
  // Interactive
  pressed: 'rgba(255, 255, 255, 0.08)',
  hover: 'rgba(255, 255, 255, 0.04)',
  disabled: '#404040',
  
  // Overlays
  overlay: 'rgba(0, 0, 0, 0.7)',
  scrim: 'rgba(0, 0, 0, 0.5)',
  
  // Status
  online: '#66BB6A',
  offline: '#757575',
  syncing: '#FFB74D',
};

// Typography Tokens
export const typography: TypographyTokens = {
  fontFamily: 'System', // Will use system font for better performance
  fontFamilyMono: 'Courier New',
  
  fontSize: {
    xs: 12,
    sm: 14,
    base: 16,
    lg: 18,
    xl: 20,
    '2xl': 24,
    '3xl': 30,
    '4xl': 36,
  },
  
  lineHeight: {
    tight: 1.25,
    normal: 1.5,
    relaxed: 1.75,
  },
  
  fontWeight: {
    normal: '400',
    medium: '500',
    semibold: '600',
    bold: '700',
  },
};

// Spacing Tokens
export const spacing: SpacingTokens = {
  xs: 4,
  sm: 8,
  md: 16,
  lg: 24,
  xl: 32,
  '2xl': 48,
  '3xl': 64,
  '4xl': 96,
};

// Border Radius Tokens
export const borderRadius: BorderRadiusTokens = {
  none: 0,
  sm: 4,
  md: 8,
  lg: 12,
  xl: 16,
  full: 9999,
};

// Shadow Tokens
export const shadows: ShadowTokens = {
  none: 'none',
  sm: '0px 1px 2px rgba(0, 0, 0, 0.1)',
  md: '0px 2px 4px rgba(0, 0, 0, 0.15)',
  lg: '0px 4px 8px rgba(0, 0, 0, 0.2)',
  xl: '0px 8px 16px rgba(0, 0, 0, 0.25)',
};

// Animation Timing Tokens
export const animations = {
  duration: {
    fast: 150,
    normal: 250,
    slow: 350,
  },
  
  easing: {
    easeIn: 'ease-in',
    easeOut: 'ease-out',
    easeInOut: 'ease-in-out',
    linear: 'linear',
  },
  
  curves: {
    // React Native easing curves
    bezier: [0.4, 0.0, 0.2, 1.0],
    easeInOut: [0.4, 0.0, 0.2, 1.0],
    easeOut: [0.0, 0.0, 0.2, 1.0],
    easeIn: [0.4, 0.0, 1.0, 1.0],
  },
};

// Accessibility tokens for compliance
export const accessibility = {
  // WCAG 2.1 AA compliant contrast ratios
  minContrastRatio: 4.5,
  largeTextContrastRatio: 3.0,
  
  // Touch target minimums
  minTouchTarget: 44,
  
  // Focus indicators
  focusRingWidth: 2,
  focusRingOffset: 2,
};

export default {
  light: lightColors,
  dark: darkColors,
  typography,
  spacing,
  borderRadius,
  shadows,
  animations,
  accessibility,
};
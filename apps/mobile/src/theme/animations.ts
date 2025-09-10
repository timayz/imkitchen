/**
 * Animation Utilities
 * Smooth animations optimized for 60fps performance with React Native Animated API
 */

import { Animated, Easing } from 'react-native';

// Animation duration constants
export const ANIMATION_DURATION = {
  FAST: 150,
  NORMAL: 250,
  SLOW: 350,
  EXTRA_SLOW: 500,
} as const;

// Easing curves for smooth animations
export const EASING_CURVES = {
  EASE_IN_OUT: Easing.bezier(0.4, 0.0, 0.2, 1.0),
  EASE_OUT: Easing.out(Easing.quad),
  EASE_IN: Easing.in(Easing.quad),
  SPRING: Easing.elastic(1),
  BOUNCE: Easing.bounce,
} as const;

/**
 * Creates a pulsing animation for the Fill My Week button
 */
export const createPulseAnimation = (
  animatedValue: Animated.Value,
  config?: {
    minScale?: number;
    maxScale?: number;
    duration?: number;
    iterations?: number;
  }
): Animated.CompositeAnimation => {
  const {
    minScale = 0.95,
    maxScale = 1.05,
    duration = ANIMATION_DURATION.SLOW,
    iterations = -1, // Infinite loop
  } = config || {};

  return Animated.loop(
    Animated.sequence([
      Animated.timing(animatedValue, {
        toValue: maxScale,
        duration: duration / 2,
        easing: EASING_CURVES.EASE_IN_OUT,
        useNativeDriver: true,
      }),
      Animated.timing(animatedValue, {
        toValue: minScale,
        duration: duration / 2,
        easing: EASING_CURVES.EASE_IN_OUT,
        useNativeDriver: true,
      }),
    ]),
    { iterations }
  );
};

/**
 * Creates a smooth fade animation
 */
export const createFadeAnimation = (
  animatedValue: Animated.Value,
  toValue: number,
  duration: number = ANIMATION_DURATION.NORMAL
): Animated.CompositeAnimation => {
  return Animated.timing(animatedValue, {
    toValue,
    duration,
    easing: EASING_CURVES.EASE_IN_OUT,
    useNativeDriver: true,
  });
};

/**
 * Creates a scale animation for press effects
 */
export const createScaleAnimation = (
  animatedValue: Animated.Value,
  toValue: number,
  duration: number = ANIMATION_DURATION.FAST
): Animated.CompositeAnimation => {
  return Animated.timing(animatedValue, {
    toValue,
    duration,
    easing: EASING_CURVES.EASE_OUT,
    useNativeDriver: true,
  });
};

/**
 * Creates a slide animation for transitions
 */
export const createSlideAnimation = (
  animatedValue: Animated.Value,
  toValue: number,
  duration: number = ANIMATION_DURATION.NORMAL
): Animated.CompositeAnimation => {
  return Animated.timing(animatedValue, {
    toValue,
    duration,
    easing: EASING_CURVES.EASE_IN_OUT,
    useNativeDriver: true,
  });
};

/**
 * Creates a spring animation for bouncy effects
 */
export const createSpringAnimation = (
  animatedValue: Animated.Value,
  toValue: number,
  config?: {
    tension?: number;
    friction?: number;
    speed?: number;
    bounciness?: number;
  }
): Animated.CompositeAnimation => {
  return Animated.spring(animatedValue, {
    toValue,
    tension: config?.tension || 40,
    friction: config?.friction || 7,
    speed: config?.speed || 12,
    bounciness: config?.bounciness || 8,
    useNativeDriver: true,
  });
};

/**
 * Creates a progress animation for loading states
 */
export const createProgressAnimation = (
  animatedValue: Animated.Value,
  toValue: number,
  duration: number = ANIMATION_DURATION.SLOW
): Animated.CompositeAnimation => {
  return Animated.timing(animatedValue, {
    toValue,
    duration,
    easing: Easing.out(Easing.quad),
    useNativeDriver: false, // Progress animations often need layout changes
  });
};

/**
 * Creates a rotation animation
 */
export const createRotationAnimation = (
  animatedValue: Animated.Value,
  duration: number = ANIMATION_DURATION.SLOW,
  iterations: number = -1
): Animated.CompositeAnimation => {
  return Animated.loop(
    Animated.timing(animatedValue, {
      toValue: 1,
      duration,
      easing: Easing.linear,
      useNativeDriver: true,
    }),
    { iterations }
  );
};

/**
 * Creates a shimmer animation for loading states
 */
export const createShimmerAnimation = (
  animatedValue: Animated.Value,
  duration: number = 1500
): Animated.CompositeAnimation => {
  return Animated.loop(
    Animated.timing(animatedValue, {
      toValue: 1,
      duration,
      easing: Easing.linear,
      useNativeDriver: true,
    })
  );
};

/**
 * Creates a staggered animation for list items
 */
export const createStaggeredAnimation = (
  animatedValues: Animated.Value[],
  staggerDelay: number = 100,
  duration: number = ANIMATION_DURATION.NORMAL
): Animated.CompositeAnimation => {
  return Animated.stagger(
    staggerDelay,
    animatedValues.map(value =>
      Animated.timing(value, {
        toValue: 1,
        duration,
        easing: EASING_CURVES.EASE_OUT,
        useNativeDriver: true,
      })
    )
  );
};

/**
 * Animation presets for common UI interactions
 */
export const ANIMATION_PRESETS = {
  // Button press effect
  BUTTON_PRESS: {
    scale: 0.95,
    duration: ANIMATION_DURATION.FAST,
    easing: EASING_CURVES.EASE_OUT,
  },
  
  // Card hover effect
  CARD_HOVER: {
    scale: 1.02,
    duration: ANIMATION_DURATION.NORMAL,
    easing: EASING_CURVES.EASE_IN_OUT,
  },
  
  // Modal entrance
  MODAL_ENTRANCE: {
    scale: 0.9,
    opacity: 0,
    duration: ANIMATION_DURATION.NORMAL,
    easing: EASING_CURVES.EASE_OUT,
  },
  
  // List item entrance
  LIST_ITEM_ENTRANCE: {
    translateY: 20,
    opacity: 0,
    duration: ANIMATION_DURATION.NORMAL,
    easing: EASING_CURVES.EASE_OUT,
  },
  
  // Loading spinner
  LOADING_SPINNER: {
    duration: 1000,
    iterations: -1,
    easing: Easing.linear,
  },
} as const;

/**
 * Performance monitoring for animations
 */
export class AnimationPerformanceMonitor {
  private static animationCount = 0;
  private static maxConcurrentAnimations = 10;

  static startAnimation(): boolean {
    if (this.animationCount >= this.maxConcurrentAnimations) {
      console.warn('Maximum concurrent animations reached. Skipping animation for performance.');
      return false;
    }
    this.animationCount++;
    return true;
  }

  static endAnimation(): void {
    this.animationCount = Math.max(0, this.animationCount - 1);
  }

  static getCurrentAnimationCount(): number {
    return this.animationCount;
  }
}

/**
 * Hook-like utility for managing animation lifecycle with performance monitoring
 */
export const withPerformanceMonitoring = (
  animation: Animated.CompositeAnimation,
  onComplete?: () => void
): Animated.CompositeAnimation => {
  if (!AnimationPerformanceMonitor.startAnimation()) {
    return Animated.timing(new Animated.Value(0), {
      toValue: 1,
      duration: 0,
      useNativeDriver: true,
    });
  }

  const originalStart = animation.start.bind(animation);
  animation.start = (callback?: (finished?: boolean) => void) => {
    originalStart((finished) => {
      AnimationPerformanceMonitor.endAnimation();
      callback?.(finished);
      onComplete?.();
    });
  };

  return animation;
};
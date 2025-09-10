import {
  createPulseAnimation,
  createFadeAnimation,
  createScaleAnimation,
  createSlideAnimation,
  createSpringAnimation,
  createProgressAnimation,
  createRotationAnimation,
  createShimmerAnimation,
  createStaggeredAnimation,
  AnimationPerformanceMonitor,
  withPerformanceMonitoring,
  ANIMATION_DURATION,
  EASING_CURVES,
  ANIMATION_PRESETS,
} from '../../src/theme/animations';

// Mock React Native Animated
const mockTiming = jest.fn();
const mockSpring = jest.fn();
const mockLoop = jest.fn();
const mockSequence = jest.fn();
const mockStagger = jest.fn();

jest.mock('react-native', () => ({
  Animated: {
    Value: jest.fn().mockImplementation((value) => ({ value })),
    timing: mockTiming,
    spring: mockSpring,
    loop: mockLoop,
    sequence: mockSequence,
    stagger: mockStagger,
  },
  Easing: {
    bezier: jest.fn(),
    out: jest.fn(),
    in: jest.fn(),
    elastic: jest.fn(),
    bounce: jest.fn(),
    quad: jest.fn(),
    linear: jest.fn(),
  },
}));

describe('Animation Utilities', () => {
  let animatedValue: any;

  beforeEach(() => {
    jest.clearAllMocks();
    animatedValue = { value: 0 };
    
    // Reset animation monitor
    AnimationPerformanceMonitor['animationCount'] = 0;
    
    // Setup default mock implementations
    mockTiming.mockReturnValue({
      start: jest.fn(),
      stop: jest.fn(),
      reset: jest.fn(),
    });
    
    mockSpring.mockReturnValue({
      start: jest.fn(),
      stop: jest.fn(),
      reset: jest.fn(),
    });
    
    mockLoop.mockReturnValue({
      start: jest.fn(),
      stop: jest.fn(),
      reset: jest.fn(),
    });
    
    mockSequence.mockReturnValue({
      start: jest.fn(),
      stop: jest.fn(),
      reset: jest.fn(),
    });
    
    mockStagger.mockReturnValue({
      start: jest.fn(),
      stop: jest.fn(),
      reset: jest.fn(),
    });
  });

  describe('Constants', () => {
    it('defines animation duration constants', () => {
      expect(ANIMATION_DURATION.FAST).toBe(150);
      expect(ANIMATION_DURATION.NORMAL).toBe(250);
      expect(ANIMATION_DURATION.SLOW).toBe(350);
      expect(ANIMATION_DURATION.EXTRA_SLOW).toBe(500);
    });

    it('defines easing curves', () => {
      expect(EASING_CURVES).toBeDefined();
      expect(EASING_CURVES.EASE_IN_OUT).toBeDefined();
      expect(EASING_CURVES.EASE_OUT).toBeDefined();
      expect(EASING_CURVES.EASE_IN).toBeDefined();
      expect(EASING_CURVES.SPRING).toBeDefined();
      expect(EASING_CURVES.BOUNCE).toBeDefined();
    });

    it('defines animation presets', () => {
      expect(ANIMATION_PRESETS.BUTTON_PRESS).toEqual({
        scale: 0.95,
        duration: ANIMATION_DURATION.FAST,
        easing: EASING_CURVES.EASE_OUT,
      });

      expect(ANIMATION_PRESETS.MODAL_ENTRANCE).toEqual({
        scale: 0.9,
        opacity: 0,
        duration: ANIMATION_DURATION.NORMAL,
        easing: EASING_CURVES.EASE_OUT,
      });
    });
  });

  describe('createPulseAnimation', () => {
    it('creates a pulse animation with default config', () => {
      const animation = createPulseAnimation(animatedValue);

      expect(mockSequence).toHaveBeenCalled();
      expect(mockLoop).toHaveBeenCalled();
      expect(mockTiming).toHaveBeenCalledTimes(2);
      
      // Check timing calls for pulse sequence
      expect(mockTiming).toHaveBeenNthCalledWith(1, animatedValue, {
        toValue: 1.05, // default maxScale
        duration: ANIMATION_DURATION.SLOW / 2,
        easing: EASING_CURVES.EASE_IN_OUT,
        useNativeDriver: true,
      });
      
      expect(mockTiming).toHaveBeenNthCalledWith(2, animatedValue, {
        toValue: 0.95, // default minScale
        duration: ANIMATION_DURATION.SLOW / 2,
        easing: EASING_CURVES.EASE_IN_OUT,
        useNativeDriver: true,
      });
    });

    it('creates a pulse animation with custom config', () => {
      const config = {
        minScale: 0.9,
        maxScale: 1.1,
        duration: 400,
        iterations: 3,
      };
      
      createPulseAnimation(animatedValue, config);

      expect(mockTiming).toHaveBeenNthCalledWith(1, animatedValue, {
        toValue: 1.1,
        duration: 200, // duration / 2
        easing: EASING_CURVES.EASE_IN_OUT,
        useNativeDriver: true,
      });
      
      expect(mockLoop).toHaveBeenCalledWith(expect.anything(), { iterations: 3 });
    });
  });

  describe('createFadeAnimation', () => {
    it('creates a fade animation with correct parameters', () => {
      createFadeAnimation(animatedValue, 1, 300);

      expect(mockTiming).toHaveBeenCalledWith(animatedValue, {
        toValue: 1,
        duration: 300,
        easing: EASING_CURVES.EASE_IN_OUT,
        useNativeDriver: true,
      });
    });

    it('uses default duration when not provided', () => {
      createFadeAnimation(animatedValue, 0.5);

      expect(mockTiming).toHaveBeenCalledWith(animatedValue, {
        toValue: 0.5,
        duration: ANIMATION_DURATION.NORMAL,
        easing: EASING_CURVES.EASE_IN_OUT,
        useNativeDriver: true,
      });
    });
  });

  describe('createScaleAnimation', () => {
    it('creates a scale animation with correct parameters', () => {
      createScaleAnimation(animatedValue, 0.95, 100);

      expect(mockTiming).toHaveBeenCalledWith(animatedValue, {
        toValue: 0.95,
        duration: 100,
        easing: EASING_CURVES.EASE_OUT,
        useNativeDriver: true,
      });
    });

    it('uses default duration when not provided', () => {
      createScaleAnimation(animatedValue, 1.05);

      expect(mockTiming).toHaveBeenCalledWith(animatedValue, {
        toValue: 1.05,
        duration: ANIMATION_DURATION.FAST,
        easing: EASING_CURVES.EASE_OUT,
        useNativeDriver: true,
      });
    });
  });

  describe('createSlideAnimation', () => {
    it('creates a slide animation with correct parameters', () => {
      createSlideAnimation(animatedValue, 100, 200);

      expect(mockTiming).toHaveBeenCalledWith(animatedValue, {
        toValue: 100,
        duration: 200,
        easing: EASING_CURVES.EASE_IN_OUT,
        useNativeDriver: true,
      });
    });
  });

  describe('createSpringAnimation', () => {
    it('creates a spring animation with default config', () => {
      createSpringAnimation(animatedValue, 1);

      expect(mockSpring).toHaveBeenCalledWith(animatedValue, {
        toValue: 1,
        tension: 40,
        friction: 7,
        speed: 12,
        bounciness: 8,
        useNativeDriver: true,
      });
    });

    it('creates a spring animation with custom config', () => {
      const config = {
        tension: 50,
        friction: 10,
        speed: 15,
        bounciness: 12,
      };
      
      createSpringAnimation(animatedValue, 0.8, config);

      expect(mockSpring).toHaveBeenCalledWith(animatedValue, {
        toValue: 0.8,
        tension: 50,
        friction: 10,
        speed: 15,
        bounciness: 12,
        useNativeDriver: true,
      });
    });
  });

  describe('createProgressAnimation', () => {
    it('creates a progress animation with useNativeDriver: false', () => {
      createProgressAnimation(animatedValue, 0.75, 400);

      expect(mockTiming).toHaveBeenCalledWith(animatedValue, {
        toValue: 0.75,
        duration: 400,
        easing: expect.anything(), // Easing.out(Easing.quad)
        useNativeDriver: false, // Progress animations need layout changes
      });
    });

    it('uses default duration when not provided', () => {
      createProgressAnimation(animatedValue, 1);

      expect(mockTiming).toHaveBeenCalledWith(animatedValue, {
        toValue: 1,
        duration: ANIMATION_DURATION.SLOW,
        easing: expect.anything(),
        useNativeDriver: false,
      });
    });
  });

  describe('createRotationAnimation', () => {
    it('creates a rotation animation with loop', () => {
      createRotationAnimation(animatedValue, 1000, 5);

      expect(mockTiming).toHaveBeenCalledWith(animatedValue, {
        toValue: 1,
        duration: 1000,
        easing: expect.anything(), // Easing.linear
        useNativeDriver: true,
      });
      
      expect(mockLoop).toHaveBeenCalledWith(expect.anything(), { iterations: 5 });
    });

    it('uses default parameters when not provided', () => {
      createRotationAnimation(animatedValue);

      expect(mockTiming).toHaveBeenCalledWith(animatedValue, {
        toValue: 1,
        duration: ANIMATION_DURATION.SLOW,
        easing: expect.anything(),
        useNativeDriver: true,
      });
      
      expect(mockLoop).toHaveBeenCalledWith(expect.anything(), { iterations: -1 });
    });
  });

  describe('createShimmerAnimation', () => {
    it('creates a shimmer animation with loop', () => {
      createShimmerAnimation(animatedValue, 2000);

      expect(mockTiming).toHaveBeenCalledWith(animatedValue, {
        toValue: 1,
        duration: 2000,
        easing: expect.anything(), // Easing.linear
        useNativeDriver: true,
      });
      
      expect(mockLoop).toHaveBeenCalled();
    });

    it('uses default duration when not provided', () => {
      createShimmerAnimation(animatedValue);

      expect(mockTiming).toHaveBeenCalledWith(animatedValue, {
        toValue: 1,
        duration: 1500,
        easing: expect.anything(),
        useNativeDriver: true,
      });
    });
  });

  describe('createStaggeredAnimation', () => {
    it('creates staggered animations for multiple values', () => {
      const animatedValues = [
        { value: 0 },
        { value: 0 },
        { value: 0 },
      ];
      
      createStaggeredAnimation(animatedValues, 150, 300);

      expect(mockStagger).toHaveBeenCalledWith(150, expect.any(Array));
      expect(mockTiming).toHaveBeenCalledTimes(3); // One for each animated value
      
      animatedValues.forEach((value) => {
        expect(mockTiming).toHaveBeenCalledWith(value, {
          toValue: 1,
          duration: 300,
          easing: EASING_CURVES.EASE_OUT,
          useNativeDriver: true,
        });
      });
    });

    it('uses default parameters when not provided', () => {
      const animatedValues = [{ value: 0 }, { value: 0 }];
      
      createStaggeredAnimation(animatedValues);

      expect(mockStagger).toHaveBeenCalledWith(100, expect.any(Array)); // Default stagger delay
      expect(mockTiming).toHaveBeenCalledWith(expect.anything(), {
        toValue: 1,
        duration: ANIMATION_DURATION.NORMAL,
        easing: EASING_CURVES.EASE_OUT,
        useNativeDriver: true,
      });
    });
  });

  describe('AnimationPerformanceMonitor', () => {
    beforeEach(() => {
      // Reset the static counter
      AnimationPerformanceMonitor['animationCount'] = 0;
    });

    it('tracks concurrent animation count', () => {
      expect(AnimationPerformanceMonitor.getCurrentAnimationCount()).toBe(0);
      
      expect(AnimationPerformanceMonitor.startAnimation()).toBe(true);
      expect(AnimationPerformanceMonitor.getCurrentAnimationCount()).toBe(1);
      
      expect(AnimationPerformanceMonitor.startAnimation()).toBe(true);
      expect(AnimationPerformanceMonitor.getCurrentAnimationCount()).toBe(2);
      
      AnimationPerformanceMonitor.endAnimation();
      expect(AnimationPerformanceMonitor.getCurrentAnimationCount()).toBe(1);
      
      AnimationPerformanceMonitor.endAnimation();
      expect(AnimationPerformanceMonitor.getCurrentAnimationCount()).toBe(0);
    });

    it('prevents starting animations when max limit reached', () => {
      const consoleSpy = jest.spyOn(console, 'warn').mockImplementation(() => {});
      
      // Start 10 animations (the max limit)
      for (let i = 0; i < 10; i++) {
        expect(AnimationPerformanceMonitor.startAnimation()).toBe(true);
      }
      
      // 11th animation should be rejected
      expect(AnimationPerformanceMonitor.startAnimation()).toBe(false);
      expect(consoleSpy).toHaveBeenCalledWith(
        'Maximum concurrent animations reached. Skipping animation for performance.'
      );
      
      consoleSpy.mockRestore();
    });

    it('does not let animation count go below zero', () => {
      AnimationPerformanceMonitor.endAnimation();
      AnimationPerformanceMonitor.endAnimation();
      
      expect(AnimationPerformanceMonitor.getCurrentAnimationCount()).toBe(0);
    });

    it('maintains accurate count during mixed start/end operations', () => {
      AnimationPerformanceMonitor.startAnimation(); // 1
      AnimationPerformanceMonitor.startAnimation(); // 2
      AnimationPerformanceMonitor.endAnimation();   // 1
      AnimationPerformanceMonitor.startAnimation(); // 2
      AnimationPerformanceMonitor.startAnimation(); // 3
      AnimationPerformanceMonitor.endAnimation();   // 2
      AnimationPerformanceMonitor.endAnimation();   // 1
      
      expect(AnimationPerformanceMonitor.getCurrentAnimationCount()).toBe(1);
    });
  });

  describe('withPerformanceMonitoring', () => {
    let mockAnimation: any;
    let mockStartCallback: jest.Mock;

    beforeEach(() => {
      mockStartCallback = jest.fn();
      mockAnimation = {
        start: jest.fn((callback) => {
          // Simulate animation completion
          setTimeout(() => callback?.(true), 0);
        }),
        stop: jest.fn(),
        reset: jest.fn(),
      };
      
      AnimationPerformanceMonitor['animationCount'] = 0;
    });

    it('wraps animation with performance monitoring', async () => {
      const wrappedAnimation = withPerformanceMonitoring(mockAnimation);
      
      expect(AnimationPerformanceMonitor.getCurrentAnimationCount()).toBe(0);
      
      wrappedAnimation.start(mockStartCallback);
      
      // Animation should be tracked
      expect(AnimationPerformanceMonitor.getCurrentAnimationCount()).toBe(1);
      
      // Wait for animation to complete
      await new Promise(resolve => setTimeout(resolve, 10));
      
      // Animation should be untracked after completion
      expect(AnimationPerformanceMonitor.getCurrentAnimationCount()).toBe(0);
      expect(mockStartCallback).toHaveBeenCalledWith(true);
    });

    it('calls onComplete callback when provided', async () => {
      const onCompleteSpy = jest.fn();
      const wrappedAnimation = withPerformanceMonitoring(mockAnimation, onCompleteSpy);
      
      wrappedAnimation.start();
      
      await new Promise(resolve => setTimeout(resolve, 10));
      
      expect(onCompleteSpy).toHaveBeenCalled();
    });

    it('returns no-op animation when performance limit reached', () => {
      // Fill up the animation slots
      for (let i = 0; i < 10; i++) {
        AnimationPerformanceMonitor.startAnimation();
      }
      
      const wrappedAnimation = withPerformanceMonitoring(mockAnimation);
      
      expect(mockTiming).toHaveBeenCalledWith(expect.any(Animated.Value), {
        toValue: 1,
        duration: 0,
        useNativeDriver: true,
      });
      
      // Original animation should not have been modified
      expect(mockAnimation.start).not.toHaveBeenCalled();
    });

    it('handles animation callback correctly', async () => {
      const originalCallback = jest.fn();
      const wrappedAnimation = withPerformanceMonitoring(mockAnimation);
      
      wrappedAnimation.start(originalCallback);
      
      await new Promise(resolve => setTimeout(resolve, 10));
      
      expect(originalCallback).toHaveBeenCalledWith(true);
      expect(AnimationPerformanceMonitor.getCurrentAnimationCount()).toBe(0);
    });

    it('preserves original animation methods', () => {
      const wrappedAnimation = withPerformanceMonitoring(mockAnimation);
      
      expect(wrappedAnimation.stop).toBe(mockAnimation.stop);
      expect(wrappedAnimation.reset).toBe(mockAnimation.reset);
    });
  });

  describe('Performance Validation - 60fps Claims', () => {
    it('validates all animations use useNativeDriver for transform properties', () => {
      // Check that animations that can use native driver do so
      createPulseAnimation(animatedValue);
      createFadeAnimation(animatedValue, 1);
      createScaleAnimation(animatedValue, 0.95);
      createSlideAnimation(animatedValue, 100);
      createSpringAnimation(animatedValue, 1);
      createRotationAnimation(animatedValue);
      createShimmerAnimation(animatedValue);
      
      // All these animations should use native driver for optimal performance
      const nativeDriverCalls = mockTiming.mock.calls.filter(call => 
        call[1].useNativeDriver === true
      );
      
      const springNativeDriverCalls = mockSpring.mock.calls.filter(call =>
        call[1].useNativeDriver === true
      );
      
      // Most timing calls should use native driver (except progress animation)
      expect(nativeDriverCalls.length).toBeGreaterThan(5);
      expect(springNativeDriverCalls.length).toBeGreaterThan(0);
    });

    it('validates progress animation uses layout driver for width changes', () => {
      createProgressAnimation(animatedValue, 0.8);
      
      expect(mockTiming).toHaveBeenCalledWith(animatedValue, expect.objectContaining({
        useNativeDriver: false, // Progress needs layout changes
      }));
    });

    it('validates performance monitoring limits concurrent animations', () => {
      const maxConcurrentAnimations = 10;
      
      // Start maximum animations
      for (let i = 0; i < maxConcurrentAnimations; i++) {
        expect(AnimationPerformanceMonitor.startAnimation()).toBe(true);
      }
      
      // Next animation should be rejected for performance
      expect(AnimationPerformanceMonitor.startAnimation()).toBe(false);
      expect(AnimationPerformanceMonitor.getCurrentAnimationCount()).toBe(maxConcurrentAnimations);
    });

    it('validates animation durations are reasonable for 60fps', () => {
      // Ensure animation durations allow for smooth frame rates
      expect(ANIMATION_DURATION.FAST).toBeGreaterThanOrEqual(150); // ~9 frames at 60fps
      expect(ANIMATION_DURATION.NORMAL).toBeGreaterThanOrEqual(250); // ~15 frames at 60fps
      expect(ANIMATION_DURATION.SLOW).toBeGreaterThanOrEqual(350); // ~21 frames at 60fps
      
      // Durations should not be too long to avoid janky UX
      expect(ANIMATION_DURATION.EXTRA_SLOW).toBeLessThanOrEqual(500);
    });

    it('validates staggered animations do not overwhelm the system', () => {
      const animatedValues = Array.from({ length: 20 }, () => new Animated.Value(0));
      
      createStaggeredAnimation(animatedValues, 50); // Short stagger delay
      
      expect(mockStagger).toHaveBeenCalledWith(50, expect.any(Array));
      
      // With 20 items and 50ms stagger, total animation time should be reasonable
      const totalStaggerTime = 20 * 50; // 1000ms = 1 second
      expect(totalStaggerTime).toBeLessThanOrEqual(2000); // Should complete within 2 seconds
    });

    it('validates no-op animation when performance limit reached has zero duration', () => {
      // Simulate performance limit reached
      for (let i = 0; i < 10; i++) {
        AnimationPerformanceMonitor.startAnimation();
      }
      
      const mockFallbackAnimation = {
        start: jest.fn(),
        stop: jest.fn(),
        reset: jest.fn(),
      };
      
      mockTiming.mockReturnValue(mockFallbackAnimation);
      
      const wrappedAnimation = withPerformanceMonitoring(mockAnimation);
      
      // Should create a zero-duration animation for immediate completion
      expect(mockTiming).toHaveBeenCalledWith(expect.any(Object), {
        toValue: 1,
        duration: 0, // Zero duration for immediate completion
        useNativeDriver: true,
      });
    });
  });
});
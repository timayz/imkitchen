import { useRef, useEffect } from 'react';

interface SwipeHandlers {
  onSwipeLeft?: () => void;
  onSwipeRight?: () => void;
  onSwipeUp?: () => void;
  onSwipeDown?: () => void;
}

interface SwipeOptions {
  threshold?: number; // Minimum distance for a swipe (px)
  preventDefaultTouchmoveEvent?: boolean;
}

export function useSwipe(handlers: SwipeHandlers, options: SwipeOptions = {}) {
  const { threshold = 50, preventDefaultTouchmoveEvent = false } = options;

  const elementRef = useRef<HTMLElement>(null);

  useEffect(() => {
    const element = elementRef.current;
    if (!element) return;

    let startX = 0;
    let startY = 0;
    let endX = 0;
    let endY = 0;

    const handleTouchStart = (e: TouchEvent) => {
      const touch = e.touches[0];
      startX = touch.clientX;
      startY = touch.clientY;
    };

    const handleTouchMove = (e: TouchEvent) => {
      if (preventDefaultTouchmoveEvent) {
        e.preventDefault();
      }
    };

    const handleTouchEnd = (e: TouchEvent) => {
      const touch = e.changedTouches[0];
      endX = touch.clientX;
      endY = touch.clientY;

      const deltaX = endX - startX;
      const deltaY = endY - startY;

      // Check if the swipe distance exceeds the threshold
      if (Math.abs(deltaX) > threshold || Math.abs(deltaY) > threshold) {
        // Determine swipe direction
        if (Math.abs(deltaX) > Math.abs(deltaY)) {
          // Horizontal swipe
          if (deltaX > 0) {
            handlers.onSwipeRight?.();
          } else {
            handlers.onSwipeLeft?.();
          }
        } else {
          // Vertical swipe
          if (deltaY > 0) {
            handlers.onSwipeDown?.();
          } else {
            handlers.onSwipeUp?.();
          }
        }
      }
    };

    element.addEventListener('touchstart', handleTouchStart, { passive: true });
    element.addEventListener('touchmove', handleTouchMove, {
      passive: !preventDefaultTouchmoveEvent,
    });
    element.addEventListener('touchend', handleTouchEnd, { passive: true });

    return () => {
      element.removeEventListener('touchstart', handleTouchStart);
      element.removeEventListener('touchmove', handleTouchMove);
      element.removeEventListener('touchend', handleTouchEnd);
    };
  }, [handlers, threshold, preventDefaultTouchmoveEvent]);

  return elementRef;
}

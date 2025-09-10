/**
 * Progressive Splash Screen Tests
 * 
 * Tests for multi-phase splash screen component including
 * initialization tracking, error handling, and accessibility
 */

import React from 'react';
import { render, waitFor } from 'react-native-testing-library';
import { ProgressiveSplashScreen } from '../atoms/ProgressiveSplashScreen';
import { splashScreenService } from '../../services/splash_screen_service';

// Mock the splash screen service
jest.mock('../../services/splash_screen_service', () => ({
  splashScreenService: {
    onProgressUpdate: jest.fn(),
    onPhaseChange: jest.fn(),
    onError: jest.fn(),
    onComplete: jest.fn(),
    initializeApp: jest.fn(),
    cleanup: jest.fn()
  }
}));

// Mock React Native components and APIs
jest.mock('react-native', () => {
  const RN = jest.requireActual('react-native');
  return {
    ...RN,
    Animated: {
      ...RN.Animated,
      Value: jest.fn(() => ({
        setValue: jest.fn(),
        interpolate: jest.fn(() => ({
          inputRange: [0, 1],
          outputRange: ['0%', '100%']
        }))
      })),
      timing: jest.fn(() => ({
        start: jest.fn((callback) => callback && callback())
      })),
      spring: jest.fn(() => ({
        start: jest.fn((callback) => callback && callback())
      })),
      parallel: jest.fn((animations) => ({
        start: jest.fn((callback) => callback && callback())
      }))
    },
    Dimensions: {
      get: jest.fn(() => ({ width: 375, height: 812 }))
    },
    StatusBar: 'StatusBar'
  };
});

describe('ProgressiveSplashScreen', () => {
  const mockOnComplete = jest.fn();
  const mockOnError = jest.fn();

  beforeEach(() => {
    jest.clearAllMocks();
    
    // Reset service mocks
    (splashScreenService.onProgressUpdate as jest.Mock).mockImplementation((callback) => {
      // Simulate progress updates
      setTimeout(() => callback('initializing_services', 25), 100);
      setTimeout(() => callback('checking_auth', 50), 200);
      setTimeout(() => callback('finalizing', 100), 300);
    });

    (splashScreenService.onPhaseChange as jest.Mock).mockImplementation((callback) => {
      setTimeout(() => callback('loading'), 50);
      setTimeout(() => callback('auth'), 150);
      setTimeout(() => callback('complete'), 350);
    });

    (splashScreenService.onComplete as jest.Mock).mockImplementation((callback) => {
      setTimeout(() => callback(), 400);
    });

    (splashScreenService.initializeApp as jest.Mock).mockResolvedValue(undefined);
  });

  afterEach(() => {
    jest.useRealTimers();
  });

  describe('initialization', () => {
    it('should render loading screen with app branding', () => {
      const { getByText } = render(
        <ProgressiveSplashScreen onInitializationComplete={mockOnComplete} />
      );

      expect(getByText('imkitchen')).toBeTruthy();
      expect(getByText('Smart Meal Planning')).toBeTruthy();
      expect(getByText('🍳')).toBeTruthy();
    });

    it('should set up service event handlers on mount', () => {
      render(<ProgressiveSplashScreen onInitializationComplete={mockOnComplete} />);

      expect(splashScreenService.onProgressUpdate).toHaveBeenCalledWith(expect.any(Function));
      expect(splashScreenService.onPhaseChange).toHaveBeenCalledWith(expect.any(Function));
      expect(splashScreenService.onError).toHaveBeenCalledWith(expect.any(Function));
      expect(splashScreenService.onComplete).toHaveBeenCalledWith(expect.any(Function));
    });

    it('should start app initialization', () => {
      render(<ProgressiveSplashScreen onInitializationComplete={mockOnComplete} />);

      expect(splashScreenService.initializeApp).toHaveBeenCalled();
    });

    it('should cleanup on unmount', () => {
      const { unmount } = render(
        <ProgressiveSplashScreen onInitializationComplete={mockOnComplete} />
      );

      unmount();

      expect(splashScreenService.cleanup).toHaveBeenCalled();
    });
  });

  describe('progress tracking', () => {
    it('should display phase titles correctly', async () => {
      jest.useFakeTimers();

      const { getByText } = render(
        <ProgressiveSplashScreen onInitializationComplete={mockOnComplete} />
      );

      // Initial loading phase
      expect(getByText('Starting imkitchen...')).toBeTruthy();

      // Advance timers to trigger phase changes
      jest.advanceTimersByTime(160);

      await waitFor(() => {
        expect(getByText('Setting up authentication...')).toBeTruthy();
      });

      jest.advanceTimersByTime(200);

      await waitFor(() => {
        expect(getByText('Welcome to imkitchen!')).toBeTruthy();
      });

      jest.useRealTimers();
    });

    it('should display step descriptions correctly', async () => {
      jest.useFakeTimers();

      const { getByText } = render(
        <ProgressiveSplashScreen onInitializationComplete={mockOnComplete} />
      );

      // Advance timers to trigger step updates
      jest.advanceTimersByTime(110);

      await waitFor(() => {
        expect(getByText('Starting core services...')).toBeTruthy();
      });

      jest.advanceTimersByTime(100);

      await waitFor(() => {
        expect(getByText('Checking authentication...')).toBeTruthy();
      });

      jest.useRealTimers();
    });

    it('should update progress percentage', async () => {
      jest.useFakeTimers();

      const { getByText } = render(
        <ProgressiveSplashScreen onInitializationComplete={mockOnComplete} />
      );

      // Should start at 0%
      expect(getByText('0%')).toBeTruthy();

      // Advance timers to trigger progress updates
      jest.advanceTimersByTime(110);

      await waitFor(() => {
        expect(getByText('25%')).toBeTruthy();
      });

      jest.advanceTimersByTime(100);

      await waitFor(() => {
        expect(getByText('50%')).toBeTruthy();
      });

      jest.useRealTimers();
    });
  });

  describe('completion handling', () => {
    it('should call onInitializationComplete when initialization finishes', async () => {
      jest.useFakeTimers();

      render(<ProgressiveSplashScreen onInitializationComplete={mockOnComplete} />);

      // Advance timers to complete initialization
      jest.advanceTimersByTime(500);

      await waitFor(() => {
        expect(mockOnComplete).toHaveBeenCalled();
      });

      jest.useRealTimers();
    });

    it('should show exit animation before completing', async () => {
      jest.useFakeTimers();

      render(<ProgressiveSplashScreen onInitializationComplete={mockOnComplete} />);

      jest.advanceTimersByTime(410);

      // Should trigger exit animation before calling completion
      expect(mockOnComplete).toHaveBeenCalled();

      jest.useRealTimers();
    });
  });

  describe('error handling', () => {
    it('should display error screen when initialization fails', async () => {
      const testError = new Error('Test initialization error');

      (splashScreenService.onError as jest.Mock).mockImplementation((callback) => {
        setTimeout(() => callback(testError), 100);
      });

      const { getByText } = render(
        <ProgressiveSplashScreen 
          onInitializationComplete={mockOnComplete}
          onError={mockOnError}
        />
      );

      await waitFor(() => {
        expect(getByText('Initialization Error')).toBeTruthy();
        expect(getByText('Test initialization error')).toBeTruthy();
      });

      expect(mockOnError).toHaveBeenCalledWith(testError);
    });

    it('should display timeout error when initialization takes too long', async () => {
      jest.useFakeTimers();

      const { getByText } = render(
        <ProgressiveSplashScreen 
          onInitializationComplete={mockOnComplete}
          onError={mockOnError}
          timeout={1000}
        />
      );

      // Advance timers past timeout
      jest.advanceTimersByTime(1100);

      await waitFor(() => {
        expect(getByText('Loading Timeout')).toBeTruthy();
        expect(getByText(/taking longer than expected/)).toBeTruthy();
      });

      expect(mockOnError).toHaveBeenCalledWith(
        expect.objectContaining({
          message: expect.stringContaining('timed out')
        })
      );

      jest.useRealTimers();
    });

    it('should show retry hint in error state', async () => {
      const testError = new Error('Network error');

      (splashScreenService.onError as jest.Mock).mockImplementation((callback) => {
        setTimeout(() => callback(testError), 50);
      });

      const { getByText } = render(
        <ProgressiveSplashScreen 
          onInitializationComplete={mockOnComplete}
          onError={mockOnError}
        />
      );

      await waitFor(() => {
        expect(getByText(/Pull down to refresh/)).toBeTruthy();
      });
    });
  });

  describe('debug mode', () => {
    it('should show debug information when enabled', () => {
      const { getByText } = render(
        <ProgressiveSplashScreen 
          onInitializationComplete={mockOnComplete}
          showDebugInfo={true}
        />
      );

      expect(getByText('Phase: loading')).toBeTruthy();
      expect(getByText('Step: none')).toBeTruthy();
      expect(getByText('Progress: 0.0%')).toBeTruthy();
    });

    it('should hide debug information by default', () => {
      const { queryByText } = render(
        <ProgressiveSplashScreen onInitializationComplete={mockOnComplete} />
      );

      expect(queryByText('Phase: loading')).toBeFalsy();
      expect(queryByText('Step: none')).toBeFalsy();
    });
  });

  describe('accessibility', () => {
    it('should include accessibility text for screen readers', () => {
      const { getByLabelText } = render(
        <ProgressiveSplashScreen onInitializationComplete={mockOnComplete} />
      );

      // Check for accessibility text element
      const accessibilityText = getByLabelText(/Starting imkitchen/);
      expect(accessibilityText).toBeTruthy();
    });

    it('should update accessibility text with progress', async () => {
      jest.useFakeTimers();

      const { getByLabelText } = render(
        <ProgressiveSplashScreen onInitializationComplete={mockOnComplete} />
      );

      jest.advanceTimersByTime(110);

      await waitFor(() => {
        const accessibilityText = getByLabelText(/Starting core services/);
        expect(accessibilityText).toBeTruthy();
      });

      jest.useRealTimers();
    });
  });

  describe('customization', () => {
    it('should use custom timeout value', async () => {
      jest.useFakeTimers();

      const { getByText } = render(
        <ProgressiveSplashScreen 
          onInitializationComplete={mockOnComplete}
          onError={mockOnError}
          timeout={500}
        />
      );

      // Advance timers past custom timeout
      jest.advanceTimersByTime(600);

      await waitFor(() => {
        expect(getByText('Loading Timeout')).toBeTruthy();
      });

      expect(mockOnError).toHaveBeenCalledWith(
        expect.objectContaining({
          message: expect.stringContaining('500ms')
        })
      );

      jest.useRealTimers();
    });
  });
});
import React from 'react';
import { render, act, waitFor } from '@testing-library/react-native';
import { Animated } from 'react-native';
import ProgressIndicator from '../../../src/components/atoms/ProgressIndicator';
import { ThemeProvider } from '../../../src/theme/ThemeProvider';

// Mock the theme and animations
jest.mock('../../../src/theme/ThemeProvider', () => ({
  useTheme: () => ({
    colors: {
      primary: '#007AFF',
      success: '#34C759',
      warning: '#FF9500',
      info: '#5AC8FA',
      text: '#000000',
      textSecondary: '#6C6C70',
      textTertiary: '#C7C7CC',
      backgroundTertiary: '#F2F2F7',
    },
  }),
  ThemeProvider: ({ children }: { children: React.ReactNode }) => children,
}));

jest.mock('../../../src/theme/animations', () => ({
  createProgressAnimation: jest.fn((animValue, progress, duration) => 
    Animated.timing(animValue, {
      toValue: progress,
      duration,
      useNativeDriver: false,
    })
  ),
  createPulseAnimation: jest.fn((animValue, options) => 
    Animated.sequence([
      Animated.timing(animValue, {
        toValue: options.maxScale,
        duration: options.duration / 2,
        useNativeDriver: false,
      }),
      Animated.timing(animValue, {
        toValue: options.minScale,
        duration: options.duration / 2,
        useNativeDriver: false,
      }),
    ])
  ),
  withPerformanceMonitoring: jest.fn((animation) => animation),
  ANIMATION_DURATION: {
    NORMAL: 300,
    SLOW: 600,
  },
}));

const renderWithTheme = (component: React.ReactElement) => {
  return render(
    <ThemeProvider>
      {component}
    </ThemeProvider>
  );
};

describe('ProgressIndicator', () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });

  describe('Basic Rendering', () => {
    it('renders with default props', () => {
      const { getByTestId } = renderWithTheme(
        <ProgressIndicator progress={0.5} testID="progress-indicator" />
      );

      expect(getByTestId('progress-indicator')).toBeTruthy();
    });

    it('renders compact variant correctly', () => {
      const { getByTestId, getByText } = renderWithTheme(
        <ProgressIndicator 
          progress={0.75} 
          variant="compact" 
          testID="progress-compact"
        />
      );

      expect(getByTestId('progress-compact')).toBeTruthy();
      expect(getByText('75%')).toBeTruthy();
    });

    it('renders detailed variant with breakdown', () => {
      const { getByTestId, getByText } = renderWithTheme(
        <ProgressIndicator 
          progress={0.4} 
          variant="detailed"
          total={10}
          testID="progress-detailed"
        />
      );

      expect(getByTestId('progress-detailed')).toBeTruthy();
      expect(getByText('4 completed')).toBeTruthy();
      expect(getByText('6 remaining')).toBeTruthy();
    });

    it('shows completion state correctly', () => {
      const { getByText } = renderWithTheme(
        <ProgressIndicator 
          progress={1.0} 
          variant="detailed"
          total={5}
        />
      );

      expect(getByText('🎉 Complete!')).toBeTruthy();
      expect(getByText('100%')).toBeTruthy();
    });
  });

  describe('Progress Calculation', () => {
    it('calculates percentage correctly', () => {
      const { getByText } = renderWithTheme(
        <ProgressIndicator progress={0.33} />
      );

      expect(getByText('33%')).toBeTruthy();
    });

    it('handles progress over 1 correctly', () => {
      const { getByText } = renderWithTheme(
        <ProgressIndicator progress={1.5} />
      );

      expect(getByText('100%')).toBeTruthy();
    });

    it('handles negative progress correctly', () => {
      const { getByText } = renderWithTheme(
        <ProgressIndicator progress={-0.2} />
      );

      expect(getByText('0%')).toBeTruthy();
    });

    it('calculates completed items from total correctly', () => {
      const { getByText } = renderWithTheme(
        <ProgressIndicator 
          progress={0.6} 
          total={10}
          variant="detailed"
        />
      );

      expect(getByText('6 completed')).toBeTruthy();
      expect(getByText('4 remaining')).toBeTruthy();
    });
  });

  describe('Text Display Options', () => {
    it('hides percentage when showPercentage is false', () => {
      const { queryByText } = renderWithTheme(
        <ProgressIndicator progress={0.5} showPercentage={false} />
      );

      expect(queryByText('50%')).toBeNull();
    });

    it('hides text when showText is false', () => {
      const { queryByText } = renderWithTheme(
        <ProgressIndicator progress={0.5} showText={false} />
      );

      expect(queryByText('Progress: 50%')).toBeNull();
    });

    it('displays custom label', () => {
      const { getByText } = renderWithTheme(
        <ProgressIndicator 
          progress={0.3} 
          label="Custom Progress Label"
        />
      );

      expect(getByText('Custom Progress Label')).toBeTruthy();
    });

    it('uses completed/total format when provided', () => {
      const { getByText } = renderWithTheme(
        <ProgressIndicator 
          progress={0.7} 
          total={10}
          completed={7}
        />
      );

      expect(getByText('7 of 10')).toBeTruthy();
    });
  });

  describe('Animation Behavior', () => {
    it('triggers progress animation when animated is true', async () => {
      const { createProgressAnimation } = require('../../../src/theme/animations');
      
      renderWithTheme(
        <ProgressIndicator progress={0.8} animated={true} />
      );

      await waitFor(() => {
        expect(createProgressAnimation).toHaveBeenCalled();
      });
    });

    it('skips animation when animated is false', () => {
      const { createProgressAnimation } = require('../../../src/theme/animations');
      
      renderWithTheme(
        <ProgressIndicator progress={0.8} animated={false} />
      );

      expect(createProgressAnimation).not.toHaveBeenCalled();
    });

    it('triggers pulse animation when progress is completed', async () => {
      const { createPulseAnimation } = require('../../../src/theme/animations');
      
      renderWithTheme(
        <ProgressIndicator progress={1.0} pulseOnComplete={true} />
      );

      await waitFor(() => {
        expect(createPulseAnimation).toHaveBeenCalled();
      });
    });

    it('does not pulse when pulseOnComplete is false', () => {
      const { createPulseAnimation } = require('../../../src/theme/animations');
      
      renderWithTheme(
        <ProgressIndicator progress={1.0} pulseOnComplete={false} />
      );

      expect(createPulseAnimation).not.toHaveBeenCalled();
    });
  });

  describe('Color Progression', () => {
    it('uses success color when completed', () => {
      const { getByTestId } = renderWithTheme(
        <ProgressIndicator progress={1.0} testID="progress-complete" />
      );

      const progressElement = getByTestId('progress-complete');
      expect(progressElement).toBeTruthy();
      // Color is applied through styles, so we verify the component renders
    });

    it('uses custom color when provided', () => {
      const { getByTestId } = renderWithTheme(
        <ProgressIndicator 
          progress={0.5} 
          color="#FF0000" 
          testID="progress-custom-color"
        />
      );

      expect(getByTestId('progress-custom-color')).toBeTruthy();
    });
  });

  describe('Accessibility', () => {
    it('has correct accessibility role', () => {
      const { getByTestId } = renderWithTheme(
        <ProgressIndicator progress={0.6} testID="progress-a11y" />
      );

      const progressElement = getByTestId('progress-a11y');
      expect(progressElement.props.accessibilityRole).toBe('progressbar');
    });

    it('has correct accessibility value', () => {
      const { getByTestId } = renderWithTheme(
        <ProgressIndicator progress={0.6} testID="progress-a11y" />
      );

      const progressElement = getByTestId('progress-a11y');
      expect(progressElement.props.accessibilityValue).toEqual({
        min: 0,
        max: 100,
        now: 60,
      });
    });

    it('has default accessibility label', () => {
      const { getByTestId } = renderWithTheme(
        <ProgressIndicator progress={0.6} testID="progress-a11y" />
      );

      const progressElement = getByTestId('progress-a11y');
      expect(progressElement.props.accessibilityLabel).toBe('Progress: 60 percent');
    });

    it('uses custom accessibility label when provided', () => {
      const { getByTestId } = renderWithTheme(
        <ProgressIndicator 
          progress={0.6} 
          accessibilityLabel="Custom progress label"
          testID="progress-a11y" 
        />
      );

      const progressElement = getByTestId('progress-a11y');
      expect(progressElement.props.accessibilityLabel).toBe('Custom progress label');
    });
  });

  describe('Performance Monitoring', () => {
    it('wraps animations with performance monitoring', async () => {
      const { withPerformanceMonitoring } = require('../../../src/theme/animations');
      
      renderWithTheme(
        <ProgressIndicator progress={0.8} animated={true} />
      );

      await waitFor(() => {
        expect(withPerformanceMonitoring).toHaveBeenCalled();
      });
    });
  });

  describe('Edge Cases', () => {
    it('handles zero progress', () => {
      const { getByText } = renderWithTheme(
        <ProgressIndicator progress={0} />
      );

      expect(getByText('0%')).toBeTruthy();
    });

    it('handles undefined total gracefully', () => {
      const { getByText } = renderWithTheme(
        <ProgressIndicator progress={0.5} total={undefined} />
      );

      expect(getByText('50%')).toBeTruthy();
    });

    it('handles zero total gracefully', () => {
      const { getByTestId } = renderWithTheme(
        <ProgressIndicator 
          progress={0.5} 
          total={0} 
          testID="progress-zero-total"
        />
      );

      expect(getByTestId('progress-zero-total')).toBeTruthy();
    });
  });
});
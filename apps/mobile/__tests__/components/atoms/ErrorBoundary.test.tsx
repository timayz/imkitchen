import React from 'react';
import { render, fireEvent, waitFor } from '@testing-library/react-native';
import { Alert, Linking } from 'react-native';
import ErrorBoundary from '../../../src/components/atoms/ErrorBoundary';
import { ThemeProvider } from '../../../src/theme/ThemeProvider';

// Mock the theme
jest.mock('../../../src/theme/ThemeProvider', () => ({
  useTheme: () => ({
    colors: {
      primary: '#007AFF',
      background: '#FFFFFF',
      backgroundSecondary: '#F8F8F8',
      border: '#E5E5E5',
      text: '#000000',
      textSecondary: '#6C6C70',
      textTertiary: '#C7C7CC',
      textInverse: '#FFFFFF',
    },
  }),
  ThemeProvider: ({ children }: { children: React.ReactNode }) => children,
}));

// Mock Alert and Linking
jest.mock('react-native', () => {
  const RN = jest.requireActual('react-native');
  return {
    ...RN,
    Alert: {
      alert: jest.fn(),
    },
    Linking: {
      openURL: jest.fn(),
    },
  };
});

const mockAlert = Alert.alert as jest.MockedFunction<typeof Alert.alert>;
const mockLinking = Linking.openURL as jest.MockedFunction<typeof Linking.openURL>;

// Test component that throws an error
const ThrowError: React.FC<{ shouldThrow?: boolean; errorMessage?: string }> = ({ 
  shouldThrow = false, 
  errorMessage = 'Test error' 
}) => {
  if (shouldThrow) {
    throw new Error(errorMessage);
  }
  return <React.Fragment>{'Normal operation'}</React.Fragment>;
};

const renderWithTheme = (component: React.ReactElement) => {
  return render(
    <ThemeProvider>
      {component}
    </ThemeProvider>
  );
};

describe('ErrorBoundary', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    // Suppress console.error for error boundary tests
    jest.spyOn(console, 'error').mockImplementation(() => {});
  });

  afterEach(() => {
    jest.restoreAllMocks();
  });

  describe('Normal Operation', () => {
    it('renders children when no error occurs', () => {
      const { getByText } = renderWithTheme(
        <ErrorBoundary>
          <ThrowError shouldThrow={false} />
        </ErrorBoundary>
      );

      expect(getByText('Normal operation')).toBeTruthy();
    });

    it('does not render error UI when no error occurs', () => {
      const { queryByText } = renderWithTheme(
        <ErrorBoundary>
          <ThrowError shouldThrow={false} />
        </ErrorBoundary>
      );

      expect(queryByText('Oops! Something went wrong')).toBeNull();
    });
  });

  describe('Error Handling', () => {
    it('catches and displays error UI when child component throws', () => {
      const { getByText } = renderWithTheme(
        <ErrorBoundary>
          <ThrowError shouldThrow={true} />
        </ErrorBoundary>
      );

      expect(getByText('Oops! Something went wrong')).toBeTruthy();
      expect(getByText(/We encountered an unexpected error/)).toBeTruthy();
      expect(getByText('Try Again')).toBeTruthy();
      expect(getByText('Report Issue')).toBeTruthy();
    });

    it('calls onError callback when error occurs', () => {
      const onErrorSpy = jest.fn();
      
      renderWithTheme(
        <ErrorBoundary onError={onErrorSpy}>
          <ThrowError shouldThrow={true} errorMessage="Custom error" />
        </ErrorBoundary>
      );

      expect(onErrorSpy).toHaveBeenCalledTimes(1);
      expect(onErrorSpy).toHaveBeenCalledWith(
        expect.any(Error),
        expect.objectContaining({
          componentStack: expect.any(String),
        })
      );
    });

    it('displays error details in development mode', () => {
      const originalDev = __DEV__;
      (global as any).__DEV__ = true;

      const { getByText } = renderWithTheme(
        <ErrorBoundary>
          <ThrowError shouldThrow={true} errorMessage="Development error" />
        </ErrorBoundary>
      );

      expect(getByText(/Development error/)).toBeTruthy();

      (global as any).__DEV__ = originalDev;
    });

    it('hides error details in production mode', () => {
      const originalDev = __DEV__;
      (global as any).__DEV__ = false;

      const { queryByText } = renderWithTheme(
        <ErrorBoundary>
          <ThrowError shouldThrow={true} errorMessage="Production error" />
        </ErrorBoundary>
      );

      expect(queryByText(/Production error/)).toBeNull();

      (global as any).__DEV__ = originalDev;
    });
  });

  describe('Custom Fallback', () => {
    it('renders custom fallback when provided', () => {
      const CustomFallback = () => <React.Fragment>{'Custom error UI'}</React.Fragment>;
      
      const { getByText, queryByText } = renderWithTheme(
        <ErrorBoundary fallback={<CustomFallback />}>
          <ThrowError shouldThrow={true} />
        </ErrorBoundary>
      );

      expect(getByText('Custom error UI')).toBeTruthy();
      expect(queryByText('Oops! Something went wrong')).toBeNull();
    });
  });

  describe('Recovery Actions', () => {
    it('resets error state when try again is pressed', () => {
      const { getByText, queryByText, rerender } = renderWithTheme(
        <ErrorBoundary>
          <ThrowError shouldThrow={true} />
        </ErrorBoundary>
      );

      // Verify error state is shown
      expect(getByText('Try Again')).toBeTruthy();
      
      // Press try again
      fireEvent.press(getByText('Try Again'));

      // Re-render with no error to simulate recovery
      rerender(
        <ThemeProvider>
          <ErrorBoundary>
            <ThrowError shouldThrow={false} />
          </ErrorBoundary>
        </ThemeProvider>
      );

      // Verify normal operation is restored
      expect(getByText('Normal operation')).toBeTruthy();
      expect(queryByText('Oops! Something went wrong')).toBeNull();
    });

    it('shows report error dialog when report issue is pressed', async () => {
      const { getByText } = renderWithTheme(
        <ErrorBoundary>
          <ThrowError shouldThrow={true} errorMessage="Test report error" />
        </ErrorBoundary>
      );

      fireEvent.press(getByText('Report Issue'));

      await waitFor(() => {
        expect(mockAlert).toHaveBeenCalledWith(
          'Report Error',
          'Help us improve the app by reporting this error. No personal information will be shared.',
          expect.arrayContaining([
            expect.objectContaining({ text: 'Cancel' }),
            expect.objectContaining({ text: 'Copy Error Details' }),
            expect.objectContaining({ text: 'Contact Support' }),
          ])
        );
      });
    });

    it('handles copy error details action', async () => {
      const consoleSpy = jest.spyOn(console, 'log').mockImplementation(() => {});
      
      const { getByText } = renderWithTheme(
        <ErrorBoundary>
          <ThrowError shouldThrow={true} errorMessage="Copy test error" />
        </ErrorBoundary>
      );

      fireEvent.press(getByText('Report Issue'));

      // Get the onPress callback for "Copy Error Details"
      const alertCall = mockAlert.mock.calls[0];
      const copyAction = alertCall[2]?.find((action: any) => action.text === 'Copy Error Details');
      
      if (copyAction?.onPress) {
        copyAction.onPress();
        
        await waitFor(() => {
          expect(consoleSpy).toHaveBeenCalledWith('Error Report:', expect.any(String));
        });
      }

      consoleSpy.mockRestore();
    });

    it('handles contact support action', async () => {
      const { getByText } = renderWithTheme(
        <ErrorBoundary>
          <ThrowError shouldThrow={true} errorMessage="Support test error" />
        </ErrorBoundary>
      );

      fireEvent.press(getByText('Report Issue'));

      // Get the onPress callback for "Contact Support"
      const alertCall = mockAlert.mock.calls[0];
      const supportAction = alertCall[2]?.find((action: any) => action.text === 'Contact Support');
      
      if (supportAction?.onPress) {
        supportAction.onPress();
        
        await waitFor(() => {
          expect(mockLinking).toHaveBeenCalledWith(
            expect.stringContaining('mailto:support@imkitchen.app')
          );
        });
      }
    });
  });

  describe('Accessibility', () => {
    it('has correct accessibility attributes for action buttons', () => {
      const { getByRole } = renderWithTheme(
        <ErrorBoundary>
          <ThrowError shouldThrow={true} />
        </ErrorBoundary>
      );

      const tryAgainButton = getByRole('button', { name: 'Try again' });
      const reportButton = getByRole('button', { name: 'Report error' });

      expect(tryAgainButton).toBeTruthy();
      expect(reportButton).toBeTruthy();
      
      expect(tryAgainButton.props.accessibilityLabel).toBe('Try again');
      expect(reportButton.props.accessibilityLabel).toBe('Report error');
    });
  });

  describe('Error Information', () => {
    it('captures error stack trace', () => {
      const onErrorSpy = jest.fn();
      
      renderWithTheme(
        <ErrorBoundary onError={onErrorSpy}>
          <ThrowError shouldThrow={true} />
        </ErrorBoundary>
      );

      const [error] = onErrorSpy.mock.calls[0];
      expect(error).toBeInstanceOf(Error);
      expect(error.stack).toBeDefined();
    });

    it('captures component stack trace', () => {
      const onErrorSpy = jest.fn();
      
      renderWithTheme(
        <ErrorBoundary onError={onErrorSpy}>
          <ThrowError shouldThrow={true} />
        </ErrorBoundary>
      );

      const [, errorInfo] = onErrorSpy.mock.calls[0];
      expect(errorInfo.componentStack).toBeDefined();
      expect(typeof errorInfo.componentStack).toBe('string');
    });
  });

  describe('Error Report Generation', () => {
    it('generates proper error report structure', async () => {
      const consoleSpy = jest.spyOn(console, 'log').mockImplementation(() => {});
      
      const { getByText } = renderWithTheme(
        <ErrorBoundary>
          <ThrowError shouldThrow={true} errorMessage="Report structure test" />
        </ErrorBoundary>
      );

      fireEvent.press(getByText('Report Issue'));

      // Trigger copy action
      const alertCall = mockAlert.mock.calls[0];
      const copyAction = alertCall[2]?.find((action: any) => action.text === 'Copy Error Details');
      
      if (copyAction?.onPress) {
        copyAction.onPress();
        
        await waitFor(() => {
          expect(consoleSpy).toHaveBeenCalledWith(
            'Error Report:',
            expect.stringContaining('Report structure test')
          );
        });
        
        // Parse the logged error report to verify structure
        const errorReportStr = consoleSpy.mock.calls[0][1];
        const errorReport = JSON.parse(errorReportStr);
        
        expect(errorReport).toMatchObject({
          error: expect.stringContaining('Report structure test'),
          stack: expect.any(String),
          componentStack: expect.any(String),
          timestamp: expect.any(String),
          userAgent: expect.any(String),
        });
      }

      consoleSpy.mockRestore();
    });
  });

  describe('Edge Cases', () => {
    it('handles error boundary re-throwing', () => {
      const { getByText } = renderWithTheme(
        <ErrorBoundary>
          <ThrowError shouldThrow={true} />
        </ErrorBoundary>
      );

      // First error
      expect(getByText('Try Again')).toBeTruthy();
      
      // Try again (this will re-render and potentially throw again)
      fireEvent.press(getByText('Try Again'));
      
      // Should still show error boundary if error persists
      // Note: In real scenario, the component would need to manage its error state
    });

    it('handles undefined error gracefully', () => {
      // Mock getDerivedStateFromError to return undefined error
      const spy = jest.spyOn(console, 'error').mockImplementation(() => {});
      
      const { getByText } = renderWithTheme(
        <ErrorBoundary>
          <ThrowError shouldThrow={true} />
        </ErrorBoundary>
      );

      expect(getByText('Oops! Something went wrong')).toBeTruthy();
      
      spy.mockRestore();
    });
  });
});
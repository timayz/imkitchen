import React from 'react';
import { render, waitFor, act } from '@testing-library/react-native';
import { useColorScheme } from 'react-native';
import AsyncStorage from '@react-native-async-storage/async-storage';
import { ThemeProvider, useTheme, useThemeStyles } from '../../src/theme/ThemeProvider';

// Mock React Native's useColorScheme
jest.mock('react-native', () => {
  const RN = jest.requireActual('react-native');
  return {
    ...RN,
    useColorScheme: jest.fn(),
  };
});

const mockUseColorScheme = useColorScheme as jest.MockedFunction<typeof useColorScheme>;

// Mock AsyncStorage
const mockAsyncStorage = AsyncStorage as jest.Mocked<typeof AsyncStorage>;

// Test component to access theme
const TestComponent: React.FC<{ onThemeUpdate?: (theme: any) => void }> = ({ onThemeUpdate }) => {
  const theme = useTheme();
  
  React.useEffect(() => {
    onThemeUpdate?.(theme);
  }, [theme, onThemeUpdate]);

  return (
    <React.Fragment>
      {`Theme: ${theme.isDarkMode ? 'dark' : 'light'}, Mode: ${theme.themeMode}`}
    </React.Fragment>
  );
};

// Test component for useThemeStyles hook
const StyledTestComponent: React.FC = () => {
  const styles = useThemeStyles((colors, isDarkMode) => ({
    container: {
      backgroundColor: colors.background,
      borderColor: isDarkMode ? colors.border : colors.backgroundSecondary,
    },
  }));

  return (
    <React.Fragment>
      {`Background: ${styles.container.backgroundColor}`}
    </React.Fragment>
  );
};

describe('ThemeProvider', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    mockUseColorScheme.mockReturnValue('light');
    mockAsyncStorage.getItem.mockResolvedValue(null);
    mockAsyncStorage.setItem.mockResolvedValue();
  });

  describe('Initial Setup', () => {
    it('provides default theme context', async () => {
      let capturedTheme: any;
      
      render(
        <ThemeProvider>
          <TestComponent onThemeUpdate={(theme) => { capturedTheme = theme; }} />
        </ThemeProvider>
      );

      await waitFor(() => {
        expect(capturedTheme).toBeDefined();
        expect(capturedTheme.themeMode).toBe('system');
        expect(capturedTheme.isDarkMode).toBe(false);
        expect(capturedTheme.colors).toBeDefined();
      });
    });

    it('follows system color scheme by default', async () => {
      mockUseColorScheme.mockReturnValue('dark');
      let capturedTheme: any;
      
      render(
        <ThemeProvider>
          <TestComponent onThemeUpdate={(theme) => { capturedTheme = theme; }} />
        </ThemeProvider>
      );

      await waitFor(() => {
        expect(capturedTheme.isDarkMode).toBe(true);
        expect(capturedTheme.themeMode).toBe('system');
      });
    });

    it('loads saved theme preference from AsyncStorage', async () => {
      mockAsyncStorage.getItem.mockResolvedValue('dark');
      let capturedTheme: any;
      
      render(
        <ThemeProvider>
          <TestComponent onThemeUpdate={(theme) => { capturedTheme = theme; }} />
        </ThemeProvider>
      );

      await waitFor(() => {
        expect(capturedTheme.themeMode).toBe('dark');
        expect(capturedTheme.isDarkMode).toBe(true);
      });
    });

    it('ignores invalid saved theme preference', async () => {
      mockAsyncStorage.getItem.mockResolvedValue('invalid');
      let capturedTheme: any;
      
      render(
        <ThemeProvider>
          <TestComponent onThemeUpdate={(theme) => { capturedTheme = theme; }} />
        </ThemeProvider>
      );

      await waitFor(() => {
        expect(capturedTheme.themeMode).toBe('system');
      });
    });

    it('handles AsyncStorage load errors gracefully', async () => {
      mockAsyncStorage.getItem.mockRejectedValue(new Error('Storage error'));
      const consoleSpy = jest.spyOn(console, 'warn').mockImplementation(() => {});
      let capturedTheme: any;
      
      render(
        <ThemeProvider>
          <TestComponent onThemeUpdate={(theme) => { capturedTheme = theme; }} />
        </ThemeProvider>
      );

      await waitFor(() => {
        expect(capturedTheme.themeMode).toBe('system');
        expect(consoleSpy).toHaveBeenCalledWith('Failed to load theme preference:', expect.any(Error));
      });

      consoleSpy.mockRestore();
    });
  });

  describe('Theme Mode Changes', () => {
    it('changes to light mode when setThemeMode is called', async () => {
      let capturedTheme: any;
      
      render(
        <ThemeProvider>
          <TestComponent onThemeUpdate={(theme) => { capturedTheme = theme; }} />
        </ThemeProvider>
      );

      await waitFor(() => {
        expect(capturedTheme).toBeDefined();
      });

      await act(async () => {
        await capturedTheme.setThemeMode('light');
      });

      expect(capturedTheme.themeMode).toBe('light');
      expect(capturedTheme.isDarkMode).toBe(false);
      expect(mockAsyncStorage.setItem).toHaveBeenCalledWith('@imkitchen_theme_mode', 'light');
    });

    it('changes to dark mode when setThemeMode is called', async () => {
      let capturedTheme: any;
      
      render(
        <ThemeProvider>
          <TestComponent onThemeUpdate={(theme) => { capturedTheme = theme; }} />
        </ThemeProvider>
      );

      await waitFor(() => {
        expect(capturedTheme).toBeDefined();
      });

      await act(async () => {
        await capturedTheme.setThemeMode('dark');
      });

      expect(capturedTheme.themeMode).toBe('dark');
      expect(capturedTheme.isDarkMode).toBe(true);
      expect(mockAsyncStorage.setItem).toHaveBeenCalledWith('@imkitchen_theme_mode', 'dark');
    });

    it('switches back to system mode', async () => {
      mockUseColorScheme.mockReturnValue('dark');
      let capturedTheme: any;
      
      render(
        <ThemeProvider>
          <TestComponent onThemeUpdate={(theme) => { capturedTheme = theme; }} />
        </ThemeProvider>
      );

      await waitFor(() => {
        expect(capturedTheme).toBeDefined();
      });

      // First set to light mode
      await act(async () => {
        await capturedTheme.setThemeMode('light');
      });

      expect(capturedTheme.isDarkMode).toBe(false);

      // Then switch back to system (which is dark)
      await act(async () => {
        await capturedTheme.setThemeMode('system');
      });

      expect(capturedTheme.themeMode).toBe('system');
      expect(capturedTheme.isDarkMode).toBe(true); // Should follow system dark mode
    });

    it('handles AsyncStorage save errors gracefully', async () => {
      mockAsyncStorage.setItem.mockRejectedValue(new Error('Save error'));
      const consoleSpy = jest.spyOn(console, 'warn').mockImplementation(() => {});
      let capturedTheme: any;
      
      render(
        <ThemeProvider>
          <TestComponent onThemeUpdate={(theme) => { capturedTheme = theme; }} />
        </ThemeProvider>
      );

      await waitFor(() => {
        expect(capturedTheme).toBeDefined();
      });

      await act(async () => {
        await capturedTheme.setThemeMode('dark');
      });

      expect(capturedTheme.themeMode).toBe('dark');
      expect(consoleSpy).toHaveBeenCalledWith('Failed to save theme preference:', expect.any(Error));

      consoleSpy.mockRestore();
    });
  });

  describe('Theme Toggle', () => {
    it('toggles from light to dark', async () => {
      let capturedTheme: any;
      
      render(
        <ThemeProvider>
          <TestComponent onThemeUpdate={(theme) => { capturedTheme = theme; }} />
        </ThemeProvider>
      );

      await waitFor(() => {
        expect(capturedTheme).toBeDefined();
      });

      // Start with light mode
      await act(async () => {
        await capturedTheme.setThemeMode('light');
      });

      expect(capturedTheme.isDarkMode).toBe(false);

      // Toggle to dark
      await act(async () => {
        capturedTheme.toggleTheme();
      });

      await waitFor(() => {
        expect(capturedTheme.isDarkMode).toBe(true);
        expect(capturedTheme.themeMode).toBe('dark');
      });
    });

    it('toggles from dark to light', async () => {
      let capturedTheme: any;
      
      render(
        <ThemeProvider>
          <TestComponent onThemeUpdate={(theme) => { capturedTheme = theme; }} />
        </ThemeProvider>
      );

      await waitFor(() => {
        expect(capturedTheme).toBeDefined();
      });

      // Start with dark mode
      await act(async () => {
        await capturedTheme.setThemeMode('dark');
      });

      expect(capturedTheme.isDarkMode).toBe(true);

      // Toggle to light
      await act(async () => {
        capturedTheme.toggleTheme();
      });

      await waitFor(() => {
        expect(capturedTheme.isDarkMode).toBe(false);
        expect(capturedTheme.themeMode).toBe('light');
      });
    });

    it('toggles from system dark to light', async () => {
      mockUseColorScheme.mockReturnValue('dark');
      let capturedTheme: any;
      
      render(
        <ThemeProvider>
          <TestComponent onThemeUpdate={(theme) => { capturedTheme = theme; }} />
        </ThemeProvider>
      );

      await waitFor(() => {
        expect(capturedTheme).toBeDefined();
        expect(capturedTheme.themeMode).toBe('system');
        expect(capturedTheme.isDarkMode).toBe(true);
      });

      // Toggle from system dark to light
      await act(async () => {
        capturedTheme.toggleTheme();
      });

      await waitFor(() => {
        expect(capturedTheme.isDarkMode).toBe(false);
        expect(capturedTheme.themeMode).toBe('light');
      });
    });
  });

  describe('System Color Scheme Changes', () => {
    it('responds to system color scheme changes when in system mode', async () => {
      let capturedTheme: any;
      
      const { rerender } = render(
        <ThemeProvider>
          <TestComponent onThemeUpdate={(theme) => { capturedTheme = theme; }} />
        </ThemeProvider>
      );

      await waitFor(() => {
        expect(capturedTheme.isDarkMode).toBe(false);
      });

      // Simulate system changing to dark mode
      mockUseColorScheme.mockReturnValue('dark');
      
      rerender(
        <ThemeProvider>
          <TestComponent onThemeUpdate={(theme) => { capturedTheme = theme; }} />
        </ThemeProvider>
      );

      await waitFor(() => {
        expect(capturedTheme.isDarkMode).toBe(true);
        expect(capturedTheme.themeMode).toBe('system');
      });
    });

    it('ignores system color scheme changes when not in system mode', async () => {
      let capturedTheme: any;
      
      const { rerender } = render(
        <ThemeProvider>
          <TestComponent onThemeUpdate={(theme) => { capturedTheme = theme; }} />
        </ThemeProvider>
      );

      await waitFor(() => {
        expect(capturedTheme).toBeDefined();
      });

      // Set to light mode explicitly
      await act(async () => {
        await capturedTheme.setThemeMode('light');
      });

      expect(capturedTheme.isDarkMode).toBe(false);

      // Simulate system changing to dark mode
      mockUseColorScheme.mockReturnValue('dark');
      
      rerender(
        <ThemeProvider>
          <TestComponent onThemeUpdate={(theme) => { capturedTheme = theme; }} />
        </ThemeProvider>
      );

      // Should remain light since we're not in system mode
      expect(capturedTheme.isDarkMode).toBe(false);
      expect(capturedTheme.themeMode).toBe('light');
    });
  });

  describe('Color Tokens', () => {
    it('provides light colors when in light mode', async () => {
      let capturedTheme: any;
      
      render(
        <ThemeProvider>
          <TestComponent onThemeUpdate={(theme) => { capturedTheme = theme; }} />
        </ThemeProvider>
      );

      await waitFor(() => {
        expect(capturedTheme).toBeDefined();
      });

      await act(async () => {
        await capturedTheme.setThemeMode('light');
      });

      expect(capturedTheme.colors).toBeDefined();
      expect(capturedTheme.colors.background).toBe('#FFFFFF');
    });

    it('provides dark colors when in dark mode', async () => {
      let capturedTheme: any;
      
      render(
        <ThemeProvider>
          <TestComponent onThemeUpdate={(theme) => { capturedTheme = theme; }} />
        </ThemeProvider>
      );

      await waitFor(() => {
        expect(capturedTheme).toBeDefined();
      });

      await act(async () => {
        await capturedTheme.setThemeMode('dark');
      });

      expect(capturedTheme.colors).toBeDefined();
      expect(capturedTheme.colors.background).toBe('#000000');
    });
  });

  describe('Loading State', () => {
    it('shows loading state while theme is being determined', async () => {
      let resolveStorage: (value: string | null) => void;
      const storagePromise = new Promise<string | null>((resolve) => {
        resolveStorage = resolve;
      });
      
      mockAsyncStorage.getItem.mockReturnValue(storagePromise);

      const { queryByText } = render(
        <ThemeProvider>
          <TestComponent />
        </ThemeProvider>
      );

      // Should not render children while loading
      expect(queryByText(/Theme:/)).toBeNull();

      // Resolve storage
      resolveStorage!(null);

      await waitFor(() => {
        expect(queryByText(/Theme:/)).toBeTruthy();
      });
    });
  });

  describe('Error Handling', () => {
    it('throws error when useTheme is used outside ThemeProvider', () => {
      const TestComponentOutside = () => {
        useTheme();
        return null;
      };

      expect(() => render(<TestComponentOutside />)).toThrow(
        'useTheme must be used within a ThemeProvider'
      );
    });
  });

  describe('useThemeStyles Hook', () => {
    it('provides theme-aware styles', async () => {
      const { getByText, rerender } = render(
        <ThemeProvider>
          <StyledTestComponent />
        </ThemeProvider>
      );

      await waitFor(() => {
        expect(getByText(/Background: #FFFFFF/)).toBeTruthy();
      });

      // Force dark mode system preference
      mockUseColorScheme.mockReturnValue('dark');
      
      rerender(
        <ThemeProvider>
          <StyledTestComponent />
        </ThemeProvider>
      );

      await waitFor(() => {
        expect(getByText(/Background: #000000/)).toBeTruthy();
      });
    });

    it('updates styles when theme changes', async () => {
      let capturedTheme: any;
      
      const CombinedTestComponent = () => {
        const theme = useTheme();
        const styles = useThemeStyles((colors) => ({
          container: { backgroundColor: colors.background },
        }));
        
        React.useEffect(() => {
          capturedTheme = theme;
        }, [theme]);

        return (
          <React.Fragment>
            {`Background: ${styles.container.backgroundColor}`}
          </React.Fragment>
        );
      };

      const { getByText } = render(
        <ThemeProvider>
          <CombinedTestComponent />
        </ThemeProvider>
      );

      await waitFor(() => {
        expect(capturedTheme).toBeDefined();
        expect(getByText(/Background: #FFFFFF/)).toBeTruthy();
      });

      await act(async () => {
        await capturedTheme.setThemeMode('dark');
      });

      await waitFor(() => {
        expect(getByText(/Background: #000000/)).toBeTruthy();
      });
    });
  });

  describe('Persistence Integration', () => {
    it('persists theme changes across app restarts', async () => {
      let capturedTheme: any;
      
      // First app session
      const { unmount } = render(
        <ThemeProvider>
          <TestComponent onThemeUpdate={(theme) => { capturedTheme = theme; }} />
        </ThemeProvider>
      );

      await waitFor(() => {
        expect(capturedTheme).toBeDefined();
      });

      await act(async () => {
        await capturedTheme.setThemeMode('dark');
      });

      expect(mockAsyncStorage.setItem).toHaveBeenCalledWith('@imkitchen_theme_mode', 'dark');
      
      unmount();

      // Simulate app restart - AsyncStorage returns saved value
      mockAsyncStorage.getItem.mockResolvedValue('dark');
      
      render(
        <ThemeProvider>
          <TestComponent onThemeUpdate={(theme) => { capturedTheme = theme; }} />
        </ThemeProvider>
      );

      await waitFor(() => {
        expect(capturedTheme.themeMode).toBe('dark');
        expect(capturedTheme.isDarkMode).toBe(true);
      });
    });
  });
});
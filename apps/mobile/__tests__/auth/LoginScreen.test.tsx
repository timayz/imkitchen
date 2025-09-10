import React from 'react';
import { render, fireEvent, waitFor } from '@testing-library/react-native';
import { Alert } from 'react-native';
import LoginScreen from '../../src/screens/auth/LoginScreen';
import { useAuthStore } from '../../src/store/auth_store';

// Mock dependencies
jest.mock('../../src/store/auth_store');
jest.mock('@react-navigation/native', () => ({
  useNavigation: () => ({
    navigate: jest.fn(),
  }),
}));

// Mock Alert
jest.spyOn(Alert, 'alert');

const mockUseAuthStore = useAuthStore as jest.MockedFunction<typeof useAuthStore>;

describe('LoginScreen', () => {
  const mockLogin = jest.fn();
  const mockClearError = jest.fn();

  beforeEach(() => {
    jest.clearAllMocks();
    mockUseAuthStore.mockReturnValue({
      login: mockLogin,
      isLoading: false,
      error: null,
      clearError: mockClearError,
      // Add other required properties
      user: null,
      isAuthenticated: false,
      token: null,
      refreshToken: null,
      sessionExpiry: null,
      register: jest.fn(),
      logout: jest.fn(),
      refreshAuth: jest.fn(),
      forgotPassword: jest.fn(),
      resetPassword: jest.fn(),
      restoreSession: jest.fn(),
      setLoading: jest.fn(),
    });
  });

  it('renders correctly', () => {
    const { getByText, getByPlaceholderText } = render(<LoginScreen />);

    expect(getByText('Welcome Back')).toBeTruthy();
    expect(getByText('Sign in to your imkitchen account')).toBeTruthy();
    expect(getByPlaceholderText('Enter your email')).toBeTruthy();
    expect(getByPlaceholderText('Enter your password')).toBeTruthy();
    expect(getByText('Sign In')).toBeTruthy();
  });

  it('handles successful login', async () => {
    mockLogin.mockResolvedValue(undefined);

    const { getByPlaceholderText, getByText } = render(<LoginScreen />);

    const emailInput = getByPlaceholderText('Enter your email');
    const passwordInput = getByPlaceholderText('Enter your password');
    const loginButton = getByText('Sign In');

    fireEvent.changeText(emailInput, 'test@example.com');
    fireEvent.changeText(passwordInput, 'password123');
    fireEvent.press(loginButton);

    await waitFor(() => {
      expect(mockClearError).toHaveBeenCalled();
      expect(mockLogin).toHaveBeenCalledWith({
        email: 'test@example.com',
        password: 'password123',
      });
    });
  });

  it('shows validation error for empty fields', () => {
    const { getByText } = render(<LoginScreen />);

    const loginButton = getByText('Sign In');
    fireEvent.press(loginButton);

    expect(Alert.alert).toHaveBeenCalledWith('Error', 'Please fill in all fields');
  });

  it('shows validation error for empty email', () => {
    const { getByPlaceholderText, getByText } = render(<LoginScreen />);

    const passwordInput = getByPlaceholderText('Enter your password');
    const loginButton = getByText('Sign In');

    fireEvent.changeText(passwordInput, 'password123');
    fireEvent.press(loginButton);

    expect(Alert.alert).toHaveBeenCalledWith('Error', 'Please fill in all fields');
  });

  it('shows validation error for empty password', () => {
    const { getByPlaceholderText, getByText } = render(<LoginScreen />);

    const emailInput = getByPlaceholderText('Enter your email');
    const loginButton = getByText('Sign In');

    fireEvent.changeText(emailInput, 'test@example.com');
    fireEvent.press(loginButton);

    expect(Alert.alert).toHaveBeenCalledWith('Error', 'Please fill in all fields');
  });

  it('trims email whitespace', async () => {
    mockLogin.mockResolvedValue(undefined);

    const { getByPlaceholderText, getByText } = render(<LoginScreen />);

    const emailInput = getByPlaceholderText('Enter your email');
    const passwordInput = getByPlaceholderText('Enter your password');
    const loginButton = getByText('Sign In');

    fireEvent.changeText(emailInput, '  test@example.com  ');
    fireEvent.changeText(passwordInput, 'password123');
    fireEvent.press(loginButton);

    await waitFor(() => {
      expect(mockLogin).toHaveBeenCalledWith({
        email: 'test@example.com',
        password: 'password123',
      });
    });
  });

  it('handles login error', async () => {
    const mockError = new Error('Invalid credentials');
    mockLogin.mockRejectedValue(mockError);

    const { getByPlaceholderText, getByText } = render(<LoginScreen />);

    const emailInput = getByPlaceholderText('Enter your email');
    const passwordInput = getByPlaceholderText('Enter your password');
    const loginButton = getByText('Sign In');

    fireEvent.changeText(emailInput, 'test@example.com');
    fireEvent.changeText(passwordInput, 'wrongpassword');
    fireEvent.press(loginButton);

    await waitFor(() => {
      expect(Alert.alert).toHaveBeenCalledWith('Login Failed', 'Invalid credentials');
    });
  });

  it('displays loading state', () => {
    mockUseAuthStore.mockReturnValue({
      login: mockLogin,
      isLoading: true,
      error: null,
      clearError: mockClearError,
      // Add other required properties
      user: null,
      isAuthenticated: false,
      token: null,
      refreshToken: null,
      sessionExpiry: null,
      register: jest.fn(),
      logout: jest.fn(),
      refreshAuth: jest.fn(),
      forgotPassword: jest.fn(),
      resetPassword: jest.fn(),
      restoreSession: jest.fn(),
      setLoading: jest.fn(),
    });

    const { getByText } = render(<LoginScreen />);

    expect(getByText('Signing In...')).toBeTruthy();
  });

  it('displays error message', () => {
    mockUseAuthStore.mockReturnValue({
      login: mockLogin,
      isLoading: false,
      error: 'Invalid credentials',
      clearError: mockClearError,
      // Add other required properties
      user: null,
      isAuthenticated: false,
      token: null,
      refreshToken: null,
      sessionExpiry: null,
      register: jest.fn(),
      logout: jest.fn(),
      refreshAuth: jest.fn(),
      forgotPassword: jest.fn(),
      resetPassword: jest.fn(),
      restoreSession: jest.fn(),
      setLoading: jest.fn(),
    });

    const { getByText } = render(<LoginScreen />);

    expect(getByText('Invalid credentials')).toBeTruthy();
  });

  it('toggles password visibility', () => {
    const { getByPlaceholderText, getByText } = render(<LoginScreen />);

    const passwordInput = getByPlaceholderText('Enter your password');
    const eyeButton = getByText('👁️');

    expect(passwordInput.props.secureTextEntry).toBe(true);

    fireEvent.press(eyeButton);

    expect(passwordInput.props.secureTextEntry).toBe(false);
  });

  it('disables inputs and buttons when loading', () => {
    mockUseAuthStore.mockReturnValue({
      login: mockLogin,
      isLoading: true,
      error: null,
      clearError: mockClearError,
      // Add other required properties
      user: null,
      isAuthenticated: false,
      token: null,
      refreshToken: null,
      sessionExpiry: null,
      register: jest.fn(),
      logout: jest.fn(),
      refreshAuth: jest.fn(),
      forgotPassword: jest.fn(),
      resetPassword: jest.fn(),
      restoreSession: jest.fn(),
      setLoading: jest.fn(),
    });

    const { getByPlaceholderText, getByText } = render(<LoginScreen />);

    const emailInput = getByPlaceholderText('Enter your email');
    const passwordInput = getByPlaceholderText('Enter your password');
    const loginButton = getByText('Signing In...');

    expect(emailInput.props.editable).toBe(false);
    expect(passwordInput.props.editable).toBe(false);
    expect(loginButton.props.accessibilityState?.disabled).toBe(true);
  });
});
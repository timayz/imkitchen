/**
 * Error Boundary Component
 * Improved error handling with clear user-friendly messages and recovery options
 */

import React, { Component, ErrorInfo, ReactNode } from 'react';
import { View, Text, TouchableOpacity, StyleSheet, Alert, Linking } from 'react-native';
import { useTheme } from '../../theme/ThemeProvider';

interface Props {
  children: ReactNode;
  fallback?: ReactNode;
  onError?: (error: Error, errorInfo: ErrorInfo) => void;
}

interface State {
  hasError: boolean;
  error?: Error;
  errorInfo?: ErrorInfo;
}

class ErrorBoundaryClass extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = { hasError: false };
  }

  static getDerivedStateFromError(error: Error): State {
    return {
      hasError: true,
      error,
    };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    console.error('ErrorBoundary caught an error:', error, errorInfo);
    this.setState({ errorInfo });
    this.props.onError?.(error, errorInfo);
  }

  private handleRetry = () => {
    this.setState({ hasError: false, error: undefined, errorInfo: undefined });
  };

  private handleReportError = async () => {
    const { error, errorInfo } = this.state;
    
    // Create error report
    const errorReport = {
      error: error?.toString(),
      stack: error?.stack,
      componentStack: errorInfo?.componentStack,
      timestamp: new Date().toISOString(),
      userAgent: navigator.userAgent || 'React Native App',
    };

    // Show options to user
    Alert.alert(
      'Report Error',
      'Help us improve the app by reporting this error. No personal information will be shared.',
      [
        {
          text: 'Cancel',
          style: 'cancel',
        },
        {
          text: 'Copy Error Details',
          onPress: () => {
            // In a real app, you'd copy to clipboard
            console.log('Error Report:', JSON.stringify(errorReport, null, 2));
            Alert.alert('Error Details', 'Error details copied to console.');
          },
        },
        {
          text: 'Contact Support',
          onPress: () => {
            // Open email client with pre-filled error report
            const subject = encodeURIComponent('App Error Report');
            const body = encodeURIComponent(
              `Error Report:\n\n${JSON.stringify(errorReport, null, 2)}`
            );
            Linking.openURL(`mailto:support@imkitchen.app?subject=${subject}&body=${body}`);
          },
        },
      ]
    );
  };

  render() {
    if (this.state.hasError) {
      if (this.props.fallback) {
        return this.props.fallback;
      }

      return <ErrorBoundaryFallback 
        error={this.state.error}
        onRetry={this.handleRetry}
        onReport={this.handleReportError}
      />;
    }

    return this.props.children;
  }
}

// Themed fallback component
const ErrorBoundaryFallback: React.FC<{
  error?: Error;
  onRetry: () => void;
  onReport: () => void;
}> = ({ error, onRetry, onReport }) => {
  const { colors } = useTheme();

  const styles = StyleSheet.create({
    container: {
      flex: 1,
      justifyContent: 'center',
      alignItems: 'center',
      padding: 20,
      backgroundColor: colors.background,
    },
    icon: {
      fontSize: 64,
      marginBottom: 16,
    },
    title: {
      fontSize: 24,
      fontWeight: '700',
      color: colors.text,
      marginBottom: 8,
      textAlign: 'center',
    },
    message: {
      fontSize: 16,
      color: colors.textSecondary,
      marginBottom: 24,
      textAlign: 'center',
      lineHeight: 24,
    },
    errorDetails: {
      fontSize: 14,
      color: colors.textTertiary,
      marginBottom: 24,
      textAlign: 'center',
      fontFamily: 'monospace',
    },
    buttonContainer: {
      flexDirection: 'row',
      gap: 12,
    },
    button: {
      backgroundColor: colors.primary,
      paddingHorizontal: 24,
      paddingVertical: 12,
      borderRadius: 8,
      minWidth: 100,
    },
    buttonSecondary: {
      backgroundColor: colors.backgroundSecondary,
      borderWidth: 1,
      borderColor: colors.border,
    },
    buttonText: {
      color: colors.textInverse,
      fontSize: 16,
      fontWeight: '600',
      textAlign: 'center',
    },
    buttonTextSecondary: {
      color: colors.text,
    },
  });

  return (
    <View style={styles.container}>
      <Text style={styles.icon}>⚠️</Text>
      <Text style={styles.title}>Oops! Something went wrong</Text>
      <Text style={styles.message}>
        We encountered an unexpected error. Don't worry, your data is safe.
        {'\n\n'}
        Try refreshing the app, or contact support if the problem persists.
      </Text>
      
      {__DEV__ && error && (
        <Text style={styles.errorDetails} numberOfLines={3}>
          {error.toString()}
        </Text>
      )}
      
      <View style={styles.buttonContainer}>
        <TouchableOpacity
          style={styles.button}
          onPress={onRetry}
          accessibilityRole="button"
          accessibilityLabel="Try again"
        >
          <Text style={styles.buttonText}>Try Again</Text>
        </TouchableOpacity>
        
        <TouchableOpacity
          style={[styles.button, styles.buttonSecondary]}
          onPress={onReport}
          accessibilityRole="button"
          accessibilityLabel="Report error"
        >
          <Text style={[styles.buttonText, styles.buttonTextSecondary]}>Report Issue</Text>
        </TouchableOpacity>
      </View>
    </View>
  );
};

// Main export with theme support
export const ErrorBoundary: React.FC<Props> = (props) => {
  return <ErrorBoundaryClass {...props} />;
};

export default ErrorBoundary;
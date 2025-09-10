/**
 * Sync Error Banner Component
 * 
 * Displays user-friendly error messages for sync failures with
 * retry actions and detailed error information.
 * 
 * Features:
 * - Clear, user-friendly error messages
 * - Retry actions for recoverable errors
 * - Error categorization and severity indicators
 * - Dismissible notifications with persistence
 * - Accessibility support for error announcements
 * - Integration with sync service error states
 */

import React, { useState, useEffect } from 'react';
import {
  View,
  Text,
  TouchableOpacity,
  StyleSheet,
  Animated,
  Alert,
  AccessibilityInfo,
} from 'react-native';
import { backgroundSyncService } from '../../services/background_sync_service';

export interface SyncError {
  id: string;
  type: SyncErrorType;
  message: string;
  userMessage: string;
  severity: 'low' | 'medium' | 'high';
  isRecoverable: boolean;
  timestamp: Date;
  itemId?: string;
  itemType?: string;
  retryCount: number;
  maxRetries: number;
  details?: any;
}

export enum SyncErrorType {
  NETWORK_ERROR = 'network_error',
  AUTH_ERROR = 'auth_error',
  CONFLICT_ERROR = 'conflict_error',
  VALIDATION_ERROR = 'validation_error',
  STORAGE_ERROR = 'storage_error',
  TIMEOUT_ERROR = 'timeout_error',
  SERVER_ERROR = 'server_error',
  UNKNOWN_ERROR = 'unknown_error'
}

export interface SyncErrorBannerProps {
  error: SyncError | null;
  onRetry?: (error: SyncError) => Promise<void>;
  onDismiss?: (error: SyncError) => void;
  onDetails?: (error: SyncError) => void;
  autoHide?: boolean;
  autoHideDuration?: number;
  style?: any;
}

const SyncErrorBanner: React.FC<SyncErrorBannerProps> = ({
  error,
  onRetry,
  onDismiss,
  onDetails,
  autoHide = false,
  autoHideDuration = 5000,
  style
}) => {
  const [isVisible, setIsVisible] = useState(false);
  const [isRetrying, setIsRetrying] = useState(false);
  const [slideAnim] = useState(new Animated.Value(-100));
  const [opacityAnim] = useState(new Animated.Value(0));

  useEffect(() => {
    if (error) {
      setIsVisible(true);
      showBanner();
      
      // Announce error to screen readers
      AccessibilityInfo.announceForAccessibility(
        `Sync error: ${error.userMessage}`
      );
      
      // Auto-hide for low severity errors
      if (autoHide && error.severity === 'low') {
        setTimeout(() => {
          hideBanner();
        }, autoHideDuration);
      }
    } else {
      hideBanner();
    }
  }, [error]);

  const showBanner = () => {
    Animated.parallel([
      Animated.timing(slideAnim, {
        toValue: 0,
        duration: 300,
        useNativeDriver: true,
      }),
      Animated.timing(opacityAnim, {
        toValue: 1,
        duration: 300,
        useNativeDriver: true,
      }),
    ]).start();
  };

  const hideBanner = () => {
    Animated.parallel([
      Animated.timing(slideAnim, {
        toValue: -100,
        duration: 250,
        useNativeDriver: true,
      }),
      Animated.timing(opacityAnim, {
        toValue: 0,
        duration: 250,
        useNativeDriver: true,
      }),
    ]).start(() => {
      setIsVisible(false);
    });
  };

  const handleRetry = async () => {
    if (!error || !onRetry || isRetrying) return;

    if (error.retryCount >= error.maxRetries) {
      Alert.alert(
        'Maximum Retries Exceeded',
        'This error has been retried multiple times. Please check your connection or try again later.',
        [{ text: 'OK' }]
      );
      return;
    }

    setIsRetrying(true);
    try {
      await onRetry(error);
      hideBanner();
    } catch (retryError) {
      console.error('[SyncErrorBanner] Retry failed:', retryError);
      // Error will be updated by parent component
    } finally {
      setIsRetrying(false);
    }
  };

  const handleDismiss = () => {
    if (error && onDismiss) {
      onDismiss(error);
    }
    hideBanner();
  };

  const handleDetails = () => {
    if (error && onDetails) {
      onDetails(error);
    } else if (error) {
      // Show default details alert
      Alert.alert(
        'Error Details',
        `Type: ${error.type}\n\nMessage: ${error.message}\n\nTime: ${error.timestamp.toLocaleString()}${
          error.itemId ? `\n\nItem: ${error.itemType} ${error.itemId}` : ''
        }`,
        [{ text: 'OK' }]
      );
    }
  };

  const getSeverityColor = (severity: string): string => {
    switch (severity) {
      case 'high':
        return '#DC3545';
      case 'medium':
        return '#FFC107';
      case 'low':
        return '#17A2B8';
      default:
        return '#6C757D';
    }
  };

  const getErrorIcon = (type: SyncErrorType): string => {
    switch (type) {
      case SyncErrorType.NETWORK_ERROR:
        return '📡';
      case SyncErrorType.AUTH_ERROR:
        return '🔐';
      case SyncErrorType.CONFLICT_ERROR:
        return '⚡';
      case SyncErrorType.VALIDATION_ERROR:
        return '⚠️';
      case SyncErrorType.STORAGE_ERROR:
        return '💾';
      case SyncErrorType.TIMEOUT_ERROR:
        return '⏰';
      case SyncErrorType.SERVER_ERROR:
        return '🔧';
      default:
        return '❗';
    }
  };

  const getUserFriendlyMessage = (error: SyncError): string => {
    if (error.userMessage) {
      return error.userMessage;
    }

    switch (error.type) {
      case SyncErrorType.NETWORK_ERROR:
        return 'Unable to connect to the server. Please check your internet connection.';
      case SyncErrorType.AUTH_ERROR:
        return 'Authentication failed. Please sign in again.';
      case SyncErrorType.CONFLICT_ERROR:
        return 'Your data conflicts with changes made elsewhere. Please resolve conflicts.';
      case SyncErrorType.VALIDATION_ERROR:
        return 'Invalid data detected. Please check your entries.';
      case SyncErrorType.STORAGE_ERROR:
        return 'Storage error occurred. Please free up some space.';
      case SyncErrorType.TIMEOUT_ERROR:
        return 'Request timed out. Please try again.';
      case SyncErrorType.SERVER_ERROR:
        return 'Server error occurred. Please try again later.';
      default:
        return 'An unexpected error occurred during synchronization.';
    }
  };

  if (!isVisible || !error) {
    return null;
  }

  const severityColor = getSeverityColor(error.severity);
  const errorIcon = getErrorIcon(error.type);
  const userMessage = getUserFriendlyMessage(error);
  const canRetry = error.isRecoverable && error.retryCount < error.maxRetries;

  return (
    <Animated.View
      style={[
        styles.container,
        {
          backgroundColor: severityColor,
          transform: [{ translateY: slideAnim }],
          opacity: opacityAnim,
        },
        style,
      ]}
      accessibilityRole="alert"
      accessibilityLiveRegion="polite"
    >
      <View style={styles.content}>
        <View style={styles.iconContainer}>
          <Text style={styles.icon}>{errorIcon}</Text>
        </View>
        
        <View style={styles.messageContainer}>
          <Text style={styles.message} numberOfLines={2}>
            {userMessage}
          </Text>
          
          {error.itemId && (
            <Text style={styles.subMessage}>
              {error.itemType}: {error.itemId}
            </Text>
          )}
          
          <View style={styles.metaInfo}>
            <Text style={styles.timestamp}>
              {error.timestamp.toLocaleTimeString()}
            </Text>
            
            {error.retryCount > 0 && (
              <Text style={styles.retryCount}>
                Retry {error.retryCount}/{error.maxRetries}
              </Text>
            )}
          </View>
        </View>
      </View>

      <View style={styles.actions}>
        {canRetry && (
          <TouchableOpacity
            style={styles.actionButton}
            onPress={handleRetry}
            disabled={isRetrying}
            accessibilityLabel="Retry sync operation"
            accessibilityHint="Attempts to retry the failed sync operation"
          >
            {isRetrying ? (
              <ActivityIndicator size="small" color="#FFFFFF" />
            ) : (
              <Text style={styles.actionButtonText}>Retry</Text>
            )}
          </TouchableOpacity>
        )}
        
        <TouchableOpacity
          style={styles.actionButton}
          onPress={handleDetails}
          accessibilityLabel="View error details"
          accessibilityHint="Shows detailed information about this error"
        >
          <Text style={styles.actionButtonText}>Details</Text>
        </TouchableOpacity>
        
        <TouchableOpacity
          style={styles.dismissButton}
          onPress={handleDismiss}
          accessibilityLabel="Dismiss error notification"
          accessibilityHint="Hides this error notification"
        >
          <Text style={styles.dismissButtonText}>✕</Text>
        </TouchableOpacity>
      </View>
    </Animated.View>
  );
};

const styles = StyleSheet.create({
  container: {
    flexDirection: 'row',
    alignItems: 'center',
    paddingHorizontal: 16,
    paddingVertical: 12,
    marginHorizontal: 16,
    marginVertical: 4,
    borderRadius: 8,
    shadowColor: '#000',
    shadowOffset: {
      width: 0,
      height: 2,
    },
    shadowOpacity: 0.25,
    shadowRadius: 3.84,
    elevation: 5,
  },
  content: {
    flex: 1,
    flexDirection: 'row',
    alignItems: 'center',
  },
  iconContainer: {
    marginRight: 12,
  },
  icon: {
    fontSize: 24,
  },
  messageContainer: {
    flex: 1,
  },
  message: {
    color: '#FFFFFF',
    fontSize: 14,
    fontWeight: '500',
    lineHeight: 20,
  },
  subMessage: {
    color: 'rgba(255, 255, 255, 0.8)',
    fontSize: 12,
    marginTop: 2,
  },
  metaInfo: {
    flexDirection: 'row',
    marginTop: 4,
    justifyContent: 'space-between',
  },
  timestamp: {
    color: 'rgba(255, 255, 255, 0.7)',
    fontSize: 10,
  },
  retryCount: {
    color: 'rgba(255, 255, 255, 0.7)',
    fontSize: 10,
    fontWeight: '600',
  },
  actions: {
    flexDirection: 'row',
    marginLeft: 8,
  },
  actionButton: {
    paddingHorizontal: 12,
    paddingVertical: 6,
    backgroundColor: 'rgba(255, 255, 255, 0.2)',
    borderRadius: 4,
    marginLeft: 8,
    minWidth: 60,
    alignItems: 'center',
  },
  actionButtonText: {
    color: '#FFFFFF',
    fontSize: 12,
    fontWeight: '600',
  },
  dismissButton: {
    paddingHorizontal: 8,
    paddingVertical: 6,
    marginLeft: 8,
    alignItems: 'center',
    justifyContent: 'center',
  },
  dismissButtonText: {
    color: 'rgba(255, 255, 255, 0.8)',
    fontSize: 16,
    fontWeight: 'bold',
  },
});

export default SyncErrorBanner;
import React, { useState, useEffect, useRef } from 'react';
import {
  View,
  Text,
  TouchableOpacity,
  Animated,
  StyleSheet,
  Dimensions,
  AccessibilityInfo,
} from 'react-native';

interface ShoppingListUpdateNotificationProps {
  visible: boolean;
  message: string;
  type: 'success' | 'info' | 'warning' | 'error';
  onDismiss?: () => void;
  onAction?: () => void;
  actionText?: string;
  autoHideDuration?: number;
  position?: 'top' | 'bottom';
  persistent?: boolean;
}

const { width: screenWidth } = Dimensions.get('window');

export const ShoppingListUpdateNotification: React.FC<ShoppingListUpdateNotificationProps> = ({
  visible,
  message,
  type,
  onDismiss,
  onAction,
  actionText,
  autoHideDuration = 4000,
  position = 'top',
  persistent = false,
}) => {
  const [isVisible, setIsVisible] = useState(visible);
  const slideAnim = useRef(new Animated.Value(0)).current;
  const opacityAnim = useRef(new Animated.Value(0)).current;
  const autoHideTimer = useRef<NodeJS.Timeout | null>(null);

  useEffect(() => {
    if (visible) {
      showNotification();
    } else {
      hideNotification();
    }
  }, [visible]);

  useEffect(() => {
    // Clean up timer on unmount
    return () => {
      if (autoHideTimer.current) {
        clearTimeout(autoHideTimer.current);
      }
    };
  }, []);

  const showNotification = () => {
    setIsVisible(true);
    
    // Animate in
    Animated.parallel([
      Animated.spring(slideAnim, {
        toValue: 1,
        useNativeDriver: true,
        tension: 100,
        friction: 8,
      }),
      Animated.timing(opacityAnim, {
        toValue: 1,
        duration: 300,
        useNativeDriver: true,
      }),
    ]).start();

    // Announce to screen readers
    AccessibilityInfo.announceForAccessibility(`Notification: ${message}`);

    // Auto-hide if not persistent
    if (!persistent && autoHideDuration > 0) {
      autoHideTimer.current = setTimeout(() => {
        handleDismiss();
      }, autoHideDuration);
    }
  };

  const hideNotification = () => {
    if (autoHideTimer.current) {
      clearTimeout(autoHideTimer.current);
      autoHideTimer.current = null;
    }

    Animated.parallel([
      Animated.spring(slideAnim, {
        toValue: 0,
        useNativeDriver: true,
        tension: 100,
        friction: 8,
      }),
      Animated.timing(opacityAnim, {
        toValue: 0,
        duration: 200,
        useNativeDriver: true,
      }),
    ]).start(() => {
      setIsVisible(false);
    });
  };

  const handleDismiss = () => {
    hideNotification();
    onDismiss?.();
  };

  const handleAction = () => {
    onAction?.();
    handleDismiss();
  };

  const getNotificationStyles = () => {
    const baseStyles = [styles.notification];
    
    switch (type) {
      case 'success':
        baseStyles.push(styles.successNotification);
        break;
      case 'info':
        baseStyles.push(styles.infoNotification);
        break;
      case 'warning':
        baseStyles.push(styles.warningNotification);
        break;
      case 'error':
        baseStyles.push(styles.errorNotification);
        break;
    }

    return baseStyles;
  };

  const getIcon = () => {
    switch (type) {
      case 'success':
        return '✅';
      case 'info':
        return 'ℹ️';
      case 'warning':
        return '⚠️';
      case 'error':
        return '❌';
      default:
        return 'ℹ️';
    }
  };

  const getTransformStyle = () => {
    const slideDistance = position === 'top' ? -100 : 100;
    
    return {
      transform: [
        {
          translateY: slideAnim.interpolate({
            inputRange: [0, 1],
            outputRange: [slideDistance, 0],
          }),
        },
      ],
    };
  };

  if (!isVisible) {
    return null;
  }

  return (
    <Animated.View
      style={[
        styles.container,
        position === 'top' ? styles.topContainer : styles.bottomContainer,
        { opacity: opacityAnim },
        getTransformStyle(),
      ]}
      accessibilityRole="alert"
      accessibilityLabel={`${type} notification: ${message}`}
      accessibilityLiveRegion="assertive"
    >
      <View style={getNotificationStyles()}>
        {/* Icon and Message */}
        <View style={styles.contentContainer}>
          <Text style={styles.icon} accessibilityHidden={true}>
            {getIcon()}
          </Text>
          <Text style={styles.message}>{message}</Text>
        </View>

        {/* Actions */}
        <View style={styles.actionsContainer}>
          {actionText && onAction && (
            <TouchableOpacity
              style={styles.actionButton}
              onPress={handleAction}
              accessibilityRole="button"
              accessibilityLabel={actionText}
              accessibilityHint="Tap to perform the action"
            >
              <Text style={styles.actionText}>{actionText}</Text>
            </TouchableOpacity>
          )}

          <TouchableOpacity
            style={styles.dismissButton}
            onPress={handleDismiss}
            accessibilityRole="button"
            accessibilityLabel="Dismiss notification"
            accessibilityHint="Tap to close this notification"
          >
            <Text style={styles.dismissText}>✕</Text>
          </TouchableOpacity>
        </View>
      </View>
    </Animated.View>
  );
};

const styles = StyleSheet.create({
  container: {
    position: 'absolute',
    left: 16,
    right: 16,
    zIndex: 9999,
    elevation: 10, // Android shadow
    shadowColor: '#000', // iOS shadow
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.25,
    shadowRadius: 4,
  },
  topContainer: {
    top: 60, // Below status bar and nav
  },
  bottomContainer: {
    bottom: 100, // Above bottom navigation
  },
  notification: {
    flexDirection: 'row',
    alignItems: 'center',
    padding: 16,
    borderRadius: 12,
    backgroundColor: '#FFFFFF',
    borderLeftWidth: 4,
    minHeight: 60,
  },
  successNotification: {
    borderLeftColor: '#34C759',
    backgroundColor: '#F0FFF4',
  },
  infoNotification: {
    borderLeftColor: '#007AFF',
    backgroundColor: '#F0F8FF',
  },
  warningNotification: {
    borderLeftColor: '#FF9500',
    backgroundColor: '#FFF8DC',
  },
  errorNotification: {
    borderLeftColor: '#FF3B30',
    backgroundColor: '#FFF0F0',
  },
  contentContainer: {
    flex: 1,
    flexDirection: 'row',
    alignItems: 'center',
  },
  icon: {
    fontSize: 20,
    marginRight: 12,
  },
  message: {
    flex: 1,
    fontSize: 14,
    lineHeight: 20,
    color: '#333333',
    fontWeight: '500',
  },
  actionsContainer: {
    flexDirection: 'row',
    alignItems: 'center',
    marginLeft: 12,
  },
  actionButton: {
    paddingVertical: 8,
    paddingHorizontal: 12,
    backgroundColor: '#007AFF',
    borderRadius: 6,
    marginRight: 8,
  },
  actionText: {
    color: '#FFFFFF',
    fontSize: 12,
    fontWeight: '600',
  },
  dismissButton: {
    padding: 8,
    borderRadius: 6,
    backgroundColor: 'rgba(0, 0, 0, 0.1)',
  },
  dismissText: {
    fontSize: 16,
    color: '#666666',
    fontWeight: '600',
  },
});

// Hook for managing notification state
export const useShoppingListNotifications = () => {
  const [notifications, setNotifications] = useState<Array<{
    id: string;
    message: string;
    type: 'success' | 'info' | 'warning' | 'error';
    actionText?: string;
    onAction?: () => void;
    persistent?: boolean;
  }>>([]);

  const showNotification = (
    message: string,
    type: 'success' | 'info' | 'warning' | 'error' = 'info',
    options?: {
      actionText?: string;
      onAction?: () => void;
      persistent?: boolean;
    }
  ) => {
    const id = `notification-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
    
    const notification = {
      id,
      message,
      type,
      actionText: options?.actionText,
      onAction: options?.onAction,
      persistent: options?.persistent,
    };

    setNotifications(prev => [...prev, notification]);

    // Auto-remove non-persistent notifications
    if (!options?.persistent) {
      setTimeout(() => {
        removeNotification(id);
      }, 4500); // Slightly longer than auto-hide to ensure cleanup
    }

    return id;
  };

  const removeNotification = (id: string) => {
    setNotifications(prev => prev.filter(n => n.id !== id));
  };

  const clearAll = () => {
    setNotifications([]);
  };

  return {
    notifications,
    showNotification,
    removeNotification,
    clearAll,
  };
};

// Shopping list specific notification helpers
export const showShoppingListUpdate = (
  showNotification: (message: string, type: any, options?: any) => string,
  changeType: 'added' | 'removed' | 'modified',
  itemCount: number,
  onViewChanges?: () => void
) => {
  let message: string;
  let type: 'success' | 'info' | 'warning' | 'error' = 'info';

  switch (changeType) {
    case 'added':
      message = `${itemCount} item${itemCount !== 1 ? 's' : ''} added to shopping list`;
      type = 'success';
      break;
    case 'removed':
      message = `${itemCount} item${itemCount !== 1 ? 's' : ''} removed from shopping list`;
      type = 'info';
      break;
    case 'modified':
      message = `${itemCount} item${itemCount !== 1 ? 's' : ''} updated in shopping list`;
      type = 'info';
      break;
  }

  return showNotification(message, type, {
    actionText: onViewChanges ? 'View Changes' : undefined,
    onAction: onViewChanges,
  });
};
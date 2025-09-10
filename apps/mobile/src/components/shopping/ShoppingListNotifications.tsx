import React, { useEffect } from 'react';
import {
  View,
  Text,
  TouchableOpacity,
  StyleSheet,
  Animated,
  Dimensions,
} from 'react-native';
import { useMealPlanStore } from '../../store/meal_plan_store';

interface NotificationItemProps {
  message: string;
  type: 'success' | 'info' | 'warning';
  onDismiss: () => void;
}

const NotificationItem: React.FC<NotificationItemProps> = ({ 
  message, 
  type, 
  onDismiss 
}) => {
  const fadeAnim = React.useRef(new Animated.Value(0)).current;
  const slideAnim = React.useRef(new Animated.Value(-100)).current;

  useEffect(() => {
    // Animate in
    Animated.parallel([
      Animated.timing(fadeAnim, {
        toValue: 1,
        duration: 300,
        useNativeDriver: true,
      }),
      Animated.timing(slideAnim, {
        toValue: 0,
        duration: 300,
        useNativeDriver: true,
      }),
    ]).start();

    // Auto-dismiss after 4 seconds
    const timer = setTimeout(() => {
      handleDismiss();
    }, 4000);

    return () => clearTimeout(timer);
  }, []);

  const handleDismiss = () => {
    Animated.parallel([
      Animated.timing(fadeAnim, {
        toValue: 0,
        duration: 200,
        useNativeDriver: true,
      }),
      Animated.timing(slideAnim, {
        toValue: -100,
        duration: 200,
        useNativeDriver: true,
      }),
    ]).start(() => {
      onDismiss();
    });
  };

  const getNotificationStyle = () => {
    switch (type) {
      case 'success':
        return { backgroundColor: '#d4edda', borderColor: '#c3e6cb', icon: '✅' };
      case 'info':
        return { backgroundColor: '#d1ecf1', borderColor: '#bee5eb', icon: 'ℹ️' };
      case 'warning':
        return { backgroundColor: '#fff3cd', borderColor: '#ffeaa7', icon: '⚠️' };
      default:
        return { backgroundColor: '#f8f9fa', borderColor: '#dee2e6', icon: 'ℹ️' };
    }
  };

  const notificationStyle = getNotificationStyle();

  return (
    <Animated.View
      style={[
        styles.notification,
        {
          backgroundColor: notificationStyle.backgroundColor,
          borderColor: notificationStyle.borderColor,
          opacity: fadeAnim,
          transform: [{ translateY: slideAnim }],
        },
      ]}
    >
      <View style={styles.notificationContent}>
        <Text style={styles.notificationIcon}>{notificationStyle.icon}</Text>
        <Text style={styles.notificationMessage}>{message}</Text>
        <TouchableOpacity
          style={styles.dismissButton}
          onPress={handleDismiss}
          activeOpacity={0.7}
        >
          <Text style={styles.dismissButtonText}>✕</Text>
        </TouchableOpacity>
      </View>
    </Animated.View>
  );
};

export const ShoppingListNotifications: React.FC = () => {
  const { 
    shoppingListNotifications, 
    clearShoppingListNotifications 
  } = useMealPlanStore();

  const handleDismissNotification = (index: number) => {
    // Remove specific notification (in a real implementation, this would be more sophisticated)
    if (index === 0 && shoppingListNotifications.length === 1) {
      clearShoppingListNotifications();
    }
  };

  if (shoppingListNotifications.length === 0) {
    return null;
  }

  return (
    <View style={styles.container}>
      {shoppingListNotifications.map((notification, index) => (
        <NotificationItem
          key={`${notification.timestamp}-${index}`}
          message={notification.message}
          type={notification.type}
          onDismiss={() => handleDismissNotification(index)}
        />
      ))}
    </View>
  );
};

const styles = StyleSheet.create({
  container: {
    position: 'absolute',
    top: 60, // Below status bar and nav
    left: 16,
    right: 16,
    zIndex: 1000,
    pointerEvents: 'box-none', // Allow touches to pass through to underlying content
  },
  notification: {
    borderRadius: 12,
    borderWidth: 1,
    marginBottom: 8,
    shadowColor: '#000',
    shadowOffset: {
      width: 0,
      height: 2,
    },
    shadowOpacity: 0.1,
    shadowRadius: 4,
    elevation: 3,
  },
  notificationContent: {
    flexDirection: 'row',
    alignItems: 'center',
    paddingHorizontal: 16,
    paddingVertical: 12,
  },
  notificationIcon: {
    fontSize: 16,
    marginRight: 12,
  },
  notificationMessage: {
    flex: 1,
    fontSize: 14,
    color: '#2d3436',
    fontWeight: '500',
    lineHeight: 20,
  },
  dismissButton: {
    width: 24,
    height: 24,
    borderRadius: 12,
    backgroundColor: 'rgba(0, 0, 0, 0.1)',
    alignItems: 'center',
    justifyContent: 'center',
    marginLeft: 8,
  },
  dismissButtonText: {
    fontSize: 12,
    color: '#636e72',
    fontWeight: '600',
  },
});
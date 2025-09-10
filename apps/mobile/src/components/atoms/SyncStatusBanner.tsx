import React from 'react';
import { View, Text, TouchableOpacity, StyleSheet, ActivityIndicator } from 'react-native';
import type { NetworkStatus } from '../../services/offline_recipe_repository';

interface SyncStatusBannerProps {
  networkStatus: NetworkStatus;
  syncInProgress: boolean;
  syncErrors: string[];
  lastSyncAttempt: Date | null;
  onRetrySync?: () => void;
  onDismissErrors?: () => void;
}

export const SyncStatusBanner: React.FC<SyncStatusBannerProps> = ({
  networkStatus,
  syncInProgress,
  syncErrors,
  lastSyncAttempt,
  onRetrySync,
  onDismissErrors,
}) => {
  const isOnline = networkStatus.isConnected && networkStatus.isInternetReachable !== false;
  const hasErrors = syncErrors.length > 0;
  
  // Don't show banner if online and no issues
  if (isOnline && !syncInProgress && !hasErrors) {
    return null;
  }

  const getBannerStyle = () => {
    if (syncInProgress) return styles.syncingBanner;
    if (hasErrors) return styles.errorBanner;
    if (!isOnline) return styles.offlineBanner;
    return styles.defaultBanner;
  };

  const getMainMessage = () => {
    if (syncInProgress) return 'Syncing your recipes...';
    if (hasErrors) return `Sync failed (${syncErrors.length} error${syncErrors.length > 1 ? 's' : ''})`;
    if (!isOnline) return 'You\'re offline. Some features may be limited.';
    return '';
  };

  const getSubMessage = () => {
    if (syncInProgress) return 'Please wait while we update your recipes';
    if (hasErrors) return 'Your recipes may not be up to date';
    if (!isOnline && lastSyncAttempt) {
      const timeSince = new Date().getTime() - lastSyncAttempt.getTime();
      const minutes = Math.floor(timeSince / (1000 * 60));
      const hours = Math.floor(minutes / 60);
      
      if (hours > 0) {
        return `Last synced ${hours} hour${hours > 1 ? 's' : ''} ago`;
      } else if (minutes > 0) {
        return `Last synced ${minutes} minute${minutes > 1 ? 's' : ''} ago`;
      } else {
        return 'Just synced';
      }
    }
    return '';
  };

  return (
    <View style={[styles.banner, getBannerStyle()]}>
      <View style={styles.content}>
        <View style={styles.messageContainer}>
          {syncInProgress && (
            <ActivityIndicator 
              size="small" 
              color="#FFF" 
              style={styles.spinner}
            />
          )}
          <View style={styles.textContainer}>
            <Text style={styles.mainMessage}>{getMainMessage()}</Text>
            {getSubMessage() && (
              <Text style={styles.subMessage}>{getSubMessage()}</Text>
            )}
          </View>
        </View>
        
        <View style={styles.actions}>
          {hasErrors && onRetrySync && !syncInProgress && (
            <TouchableOpacity 
              style={styles.retryButton} 
              onPress={onRetrySync}
            >
              <Text style={styles.retryButtonText}>Retry</Text>
            </TouchableOpacity>
          )}
          
          {hasErrors && onDismissErrors && (
            <TouchableOpacity 
              style={styles.dismissButton} 
              onPress={onDismissErrors}
            >
              <Text style={styles.dismissButtonText}>×</Text>
            </TouchableOpacity>
          )}
        </View>
      </View>
      
      {hasErrors && syncErrors.length > 0 && (
        <View style={styles.errorDetails}>
          <Text style={styles.errorText}>
            {syncErrors[0]} {syncErrors.length > 1 ? `(+${syncErrors.length - 1} more)` : ''}
          </Text>
        </View>
      )}
    </View>
  );
};

const styles = StyleSheet.create({
  banner: {
    paddingHorizontal: 16,
    paddingVertical: 12,
    marginHorizontal: 16,
    marginVertical: 8,
    borderRadius: 8,
    elevation: 2,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.1,
    shadowRadius: 4,
  },
  defaultBanner: {
    backgroundColor: '#4A90E2',
  },
  syncingBanner: {
    backgroundColor: '#FFA500',
  },
  errorBanner: {
    backgroundColor: '#FF4444',
  },
  offlineBanner: {
    backgroundColor: '#666666',
  },
  content: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
  },
  messageContainer: {
    flexDirection: 'row',
    alignItems: 'center',
    flex: 1,
  },
  spinner: {
    marginRight: 8,
  },
  textContainer: {
    flex: 1,
  },
  mainMessage: {
    color: '#FFFFFF',
    fontSize: 14,
    fontWeight: '600',
    marginBottom: 2,
  },
  subMessage: {
    color: '#FFFFFF',
    fontSize: 12,
    opacity: 0.9,
  },
  actions: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 8,
  },
  retryButton: {
    backgroundColor: 'rgba(255, 255, 255, 0.2)',
    paddingHorizontal: 12,
    paddingVertical: 6,
    borderRadius: 4,
    borderWidth: 1,
    borderColor: 'rgba(255, 255, 255, 0.3)',
  },
  retryButtonText: {
    color: '#FFFFFF',
    fontSize: 12,
    fontWeight: '600',
  },
  dismissButton: {
    width: 24,
    height: 24,
    borderRadius: 12,
    backgroundColor: 'rgba(255, 255, 255, 0.2)',
    alignItems: 'center',
    justifyContent: 'center',
  },
  dismissButtonText: {
    color: '#FFFFFF',
    fontSize: 16,
    fontWeight: 'bold',
    lineHeight: 20,
  },
  errorDetails: {
    marginTop: 8,
    paddingTop: 8,
    borderTopWidth: 1,
    borderTopColor: 'rgba(255, 255, 255, 0.2)',
  },
  errorText: {
    color: '#FFFFFF',
    fontSize: 11,
    opacity: 0.9,
  },
});
import React from 'react';
import { View, Text, StyleSheet } from 'react-native';
import type { NetworkStatus } from '../../services/offline_recipe_repository';

interface OfflineStatusIndicatorProps {
  networkStatus: NetworkStatus;
  syncInProgress: boolean;
  syncErrors: string[];
  lastSyncAttempt: Date | null;
  showLabel?: boolean;
  size?: 'small' | 'medium' | 'large';
}

export const OfflineStatusIndicator: React.FC<OfflineStatusIndicatorProps> = ({
  networkStatus,
  syncInProgress,
  syncErrors,
  lastSyncAttempt,
  showLabel = true,
  size = 'medium',
}) => {
  const isOnline = networkStatus.isConnected && networkStatus.isInternetReachable !== false;
  const hasErrors = syncErrors.length > 0;
  
  const getStatusColor = () => {
    if (syncInProgress) return '#FFA500'; // Orange for syncing
    if (hasErrors) return '#FF4444'; // Red for errors
    if (!isOnline) return '#888888'; // Gray for offline
    return '#44AA44'; // Green for online
  };

  const getStatusText = () => {
    if (syncInProgress) return 'Syncing...';
    if (hasErrors) return 'Sync Failed';
    if (!isOnline) return 'Offline';
    return 'Online';
  };

  const getIndicatorSize = () => {
    switch (size) {
      case 'small': return 8;
      case 'large': return 16;
      default: return 12;
    }
  };

  const getTextSize = () => {
    switch (size) {
      case 'small': return 10;
      case 'large': return 16;
      default: return 12;
    }
  };

  return (
    <View style={[styles.container, styles[`container_${size}`]]}>
      <View
        style={[
          styles.indicator,
          {
            backgroundColor: getStatusColor(),
            width: getIndicatorSize(),
            height: getIndicatorSize(),
            borderRadius: getIndicatorSize() / 2,
          },
        ]}
      />
      {showLabel && (
        <Text style={[styles.text, { fontSize: getTextSize(), color: getStatusColor() }]}>
          {getStatusText()}
        </Text>
      )}
      {lastSyncAttempt && size === 'large' && (
        <Text style={styles.lastSync}>
          Last sync: {lastSyncAttempt.toLocaleTimeString()}
        </Text>
      )}
    </View>
  );
};

const styles = StyleSheet.create({
  container: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'center',
  },
  container_small: {
    gap: 4,
  },
  container_medium: {
    gap: 6,
  },
  container_large: {
    gap: 8,
    flexDirection: 'column',
    alignItems: 'flex-start',
  },
  indicator: {
    // Dynamic styles applied inline
  },
  text: {
    fontWeight: '500',
  },
  lastSync: {
    fontSize: 10,
    color: '#888888',
    marginTop: 2,
  },
});
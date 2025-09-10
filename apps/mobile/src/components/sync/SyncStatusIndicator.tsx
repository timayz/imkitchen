/**
 * Sync Status Indicator Component
 * 
 * Displays comprehensive synchronization status with real-time updates,
 * visual feedback, and user interaction capabilities.
 * 
 * Features:
 * - Real-time sync status monitoring and display
 * - Visual sync progress indicators with animations
 * - Conflict notification and resolution prompts
 * - Manual sync trigger with feedback
 * - Sync history and statistics display
 * - Offline/online status integration
 * - Accessibility support with screen reader announcements
 */

import React, { useState, useEffect, useRef } from 'react';
import {
  View,
  Text,
  TouchableOpacity,
  StyleSheet,
  Animated,
  ActivityIndicator,
  Alert,
  AccessibilityInfo,
} from 'react-native';
import { backgroundSyncService } from '../../services/background_sync_service';
import { conflictResolutionService } from '../../services/conflict_resolution_service';
import { syncCoordinationService } from '../../services/sync_coordination_service';

export interface SyncStatusIndicatorProps {
  style?: any;
  showDetails?: boolean;
  onConflictPress?: (conflictCount: number) => void;
  onSyncPress?: () => void;
  compact?: boolean;
}

interface SyncDisplayState {
  status: 'synced' | 'syncing' | 'offline' | 'conflicts' | 'error';
  message: string;
  progress: number;
  conflictCount: number;
  queueSize: number;
  lastSync?: Date;
  canManualSync: boolean;
}

export const SyncStatusIndicator: React.FC<SyncStatusIndicatorProps> = ({
  style,
  showDetails = false,
  onConflictPress,
  onSyncPress,
  compact = false
}) => {
  const [displayState, setDisplayState] = useState<SyncDisplayState>({
    status: 'synced',
    message: 'All up to date',
    progress: 0,
    conflictCount: 0,
    queueSize: 0,
    canManualSync: true
  });

  const [isManualSyncing, setIsManualSyncing] = useState(false);
  const [expandedStats, setExpandedStats] = useState(false);

  // Animation values
  const pulseAnim = useRef(new Animated.Value(1)).current;
  const progressAnim = useRef(new Animated.Value(0)).current;
  const slideAnim = useRef(new Animated.Value(0)).current;

  useEffect(() => {
    const updateInterval = setInterval(updateSyncStatus, 1000);
    return () => clearInterval(updateInterval);
  }, []);

  useEffect(() => {
    startStatusAnimation();
    announceStatusChange();
  }, [displayState.status]);

  const updateSyncStatus = async () => {
    try {
      // Get sync service status
      const syncStatus = backgroundSyncService.getSyncStatus();
      const coordinationStatus = syncCoordinationService.getCoordinationStatus();
      const conflicts = conflictResolutionService.getPendingConflicts();
      const syncStats = backgroundSyncService.getStatistics();

      // Determine display status and message
      const newState = calculateDisplayState(
        syncStatus,
        coordinationStatus,
        conflicts,
        syncStats
      );

      setDisplayState(newState);
      
      // Update progress animation
      Animated.timing(progressAnim, {
        toValue: newState.progress / 100,
        duration: 300,
        useNativeDriver: false,
      }).start();

    } catch (error) {
      console.error('[SyncStatusIndicator] Failed to update sync status:', error);
      setDisplayState({
        status: 'error',
        message: 'Sync status unavailable',
        progress: 0,
        conflictCount: 0,
        queueSize: 0,
        canManualSync: false
      });
    }
  };

  const calculateDisplayState = (
    syncStatus: any,
    coordinationStatus: any,
    conflicts: any[],
    stats: any
  ): SyncDisplayState => {
    // Priority: Conflicts > Offline > Syncing > Error > Synced
    
    if (conflicts.length > 0) {
      return {
        status: 'conflicts',
        message: `${conflicts.length} conflict${conflicts.length > 1 ? 's' : ''} need${conflicts.length === 1 ? 's' : ''} attention`,
        progress: 0,
        conflictCount: conflicts.length,
        queueSize: syncStatus.queueSize,
        lastSync: syncStatus.lastSync,
        canManualSync: true
      };
    }

    if (!syncStatus.isOnline) {
      return {
        status: 'offline',
        message: syncStatus.queueSize > 0 ? 
          `${syncStatus.queueSize} item${syncStatus.queueSize > 1 ? 's' : ''} pending sync` : 
          'Working offline',
        progress: 0,
        conflictCount: 0,
        queueSize: syncStatus.queueSize,
        lastSync: syncStatus.lastSync,
        canManualSync: false
      };
    }

    if (syncStatus.activeSyncs > 0 || isManualSyncing) {
      const progress = syncStatus.queueSize > 0 ? 
        Math.round(((syncStatus.queueSize - syncStatus.activeSyncs) / syncStatus.queueSize) * 100) : 
        50;
      
      return {
        status: 'syncing',
        message: `Syncing ${syncStatus.activeSyncs} item${syncStatus.activeSyncs > 1 ? 's' : ''}...`,
        progress,
        conflictCount: 0,
        queueSize: syncStatus.queueSize,
        lastSync: syncStatus.lastSync,
        canManualSync: false
      };
    }

    if (syncStatus.queueSize > 0) {
      return {
        status: 'syncing',
        message: `${syncStatus.queueSize} item${syncStatus.queueSize > 1 ? 's' : ''} queued`,
        progress: 0,
        conflictCount: 0,
        queueSize: syncStatus.queueSize,
        lastSync: syncStatus.lastSync,
        canManualSync: true
      };
    }

    // All synced
    const lastSyncMessage = syncStatus.lastSync ? 
      `Last sync: ${formatLastSyncTime(syncStatus.lastSync)}` : 
      'All up to date';

    return {
      status: 'synced',
      message: lastSyncMessage,
      progress: 100,
      conflictCount: 0,
      queueSize: 0,
      lastSync: syncStatus.lastSync,
      canManualSync: true
    };
  };

  const formatLastSyncTime = (date: Date): string => {
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / 60000);
    const diffHours = Math.floor(diffMins / 60);
    const diffDays = Math.floor(diffHours / 24);

    if (diffMins < 1) return 'just now';
    if (diffMins < 60) return `${diffMins}m ago`;
    if (diffHours < 24) return `${diffHours}h ago`;
    if (diffDays < 7) return `${diffDays}d ago`;
    return date.toLocaleDateString();
  };

  const startStatusAnimation = () => {
    if (displayState.status === 'syncing') {
      // Pulse animation for syncing
      Animated.loop(
        Animated.sequence([
          Animated.timing(pulseAnim, {
            toValue: 1.2,
            duration: 800,
            useNativeDriver: true,
          }),
          Animated.timing(pulseAnim, {
            toValue: 1,
            duration: 800,
            useNativeDriver: true,
          }),
        ])
      ).start();
    } else {
      // Stop pulsing
      Animated.timing(pulseAnim, {
        toValue: 1,
        duration: 200,
        useNativeDriver: true,
      }).start();
    }
  };

  const announceStatusChange = () => {
    // Announce status changes for accessibility
    const announcement = `Sync status: ${displayState.message}`;
    AccessibilityInfo.announceForAccessibility(announcement);
  };

  const handleManualSync = async () => {
    if (!displayState.canManualSync || isManualSyncing) return;

    try {
      setIsManualSyncing(true);
      
      const result = await backgroundSyncService.manualSync();
      
      if (onSyncPress) {
        onSyncPress();
      }

      // Show success feedback
      if (result.triggered === 0) {
        Alert.alert('Sync Complete', 'Everything is already up to date');
      } else {
        Alert.alert(
          'Sync Started', 
          `Syncing ${result.triggered} item${result.triggered > 1 ? 's' : ''}...`
        );
      }

    } catch (error) {
      console.error('[SyncStatusIndicator] Manual sync failed:', error);
      Alert.alert('Sync Failed', 'Please check your connection and try again');
    } finally {
      setTimeout(() => setIsManualSyncing(false), 2000); // Minimum visual feedback time
    }
  };

  const handleConflictPress = () => {
    if (onConflictPress) {
      onConflictPress(displayState.conflictCount);
    }
  };

  const toggleExpandedStats = () => {
    setExpandedStats(!expandedStats);
    
    Animated.timing(slideAnim, {
      toValue: expandedStats ? 0 : 1,
      duration: 300,
      useNativeDriver: false,
    }).start();
  };

  const getStatusIcon = () => {
    switch (displayState.status) {
      case 'synced':
        return '✓';
      case 'syncing':
        return <ActivityIndicator size="small" color={getStatusColor()} />;
      case 'offline':
        return '⚠';
      case 'conflicts':
        return '!';
      case 'error':
        return '✗';
      default:
        return '?';
    }
  };

  const getStatusColor = () => {
    switch (displayState.status) {
      case 'synced':
        return '#34C759';
      case 'syncing':
        return '#007AFF';
      case 'offline':
        return '#FF9500';
      case 'conflicts':
        return '#FF3B30';
      case 'error':
        return '#FF3B30';
      default:
        return '#8E8E93';
    }
  };

  if (compact) {
    return (
      <TouchableOpacity 
        style={[styles.compactContainer, style]}
        onPress={displayState.status === 'conflicts' ? handleConflictPress : handleManualSync}
        disabled={!displayState.canManualSync && displayState.status !== 'conflicts'}
        accessibilityLabel={`Sync status: ${displayState.message}`}
        accessibilityRole="button"
      >
        <Animated.View style={[styles.compactIcon, { transform: [{ scale: pulseAnim }] }]}>
          <Text style={[styles.iconText, { color: getStatusColor() }]}>
            {typeof getStatusIcon() === 'string' ? getStatusIcon() : ''}
          </Text>
          {typeof getStatusIcon() !== 'string' && getStatusIcon()}
        </Animated.View>
        
        {displayState.conflictCount > 0 && (
          <View style={styles.conflictBadge}>
            <Text style={styles.conflictBadgeText}>{displayState.conflictCount}</Text>
          </View>
        )}
      </TouchableOpacity>
    );
  }

  return (
    <View style={[styles.container, style]}>
      {/* Main Status Row */}
      <View style={styles.statusRow}>
        <Animated.View style={[styles.iconContainer, { transform: [{ scale: pulseAnim }] }]}>
          <View style={[styles.iconBackground, { backgroundColor: getStatusColor() + '20' }]}>
            {typeof getStatusIcon() === 'string' ? (
              <Text style={[styles.iconText, { color: getStatusColor() }]}>
                {getStatusIcon()}
              </Text>
            ) : (
              getStatusIcon()
            )}
          </View>
        </Animated.View>

        <View style={styles.statusContent}>
          <Text style={styles.statusMessage}>{displayState.message}</Text>
          
          {displayState.status === 'syncing' && displayState.progress > 0 && (
            <View style={styles.progressContainer}>
              <View style={styles.progressTrack}>
                <Animated.View 
                  style={[
                    styles.progressBar, 
                    { 
                      width: progressAnim.interpolate({
                        inputRange: [0, 1],
                        outputRange: ['0%', '100%']
                      }),
                      backgroundColor: getStatusColor()
                    }
                  ]} 
                />
              </View>
              <Text style={styles.progressText}>{displayState.progress}%</Text>
            </View>
          )}
        </View>

        {/* Action Buttons */}
        <View style={styles.actionButtons}>
          {displayState.status === 'conflicts' && (
            <TouchableOpacity
              style={[styles.actionButton, styles.conflictButton]}
              onPress={handleConflictPress}
              accessibilityLabel={`View ${displayState.conflictCount} conflicts`}
              accessibilityRole="button"
            >
              <Text style={styles.conflictButtonText}>
                {displayState.conflictCount}
              </Text>
            </TouchableOpacity>
          )}

          {displayState.canManualSync && displayState.status !== 'conflicts' && (
            <TouchableOpacity
              style={[styles.actionButton, styles.syncButton]}
              onPress={handleManualSync}
              disabled={isManualSyncing}
              accessibilityLabel="Sync now"
              accessibilityRole="button"
            >
              {isManualSyncing ? (
                <ActivityIndicator size="small" color="#007AFF" />
              ) : (
                <Text style={styles.syncButtonText}>↻</Text>
              )}
            </TouchableOpacity>
          )}

          {showDetails && (
            <TouchableOpacity
              style={[styles.actionButton, styles.detailsButton]}
              onPress={toggleExpandedStats}
              accessibilityLabel={expandedStats ? 'Hide details' : 'Show details'}
              accessibilityRole="button"
            >
              <Text style={styles.detailsButtonText}>
                {expandedStats ? '−' : '+'}
              </Text>
            </TouchableOpacity>
          )}
        </View>
      </View>

      {/* Expanded Stats */}
      {showDetails && (
        <Animated.View 
          style={[
            styles.expandedStats,
            {
              opacity: slideAnim,
              maxHeight: slideAnim.interpolate({
                inputRange: [0, 1],
                outputRange: [0, 100]
              })
            }
          ]}
        >
          <SyncStatisticsDisplay />
        </Animated.View>
      )}
    </View>
  );
};

// Statistics display component
const SyncStatisticsDisplay: React.FC = () => {
  const [stats, setStats] = useState<any>(null);

  useEffect(() => {
    const updateStats = () => {
      const syncStats = backgroundSyncService.getStatistics();
      const conflictAnalytics = conflictResolutionService.getAnalytics();
      
      setStats({
        totalSyncs: syncStats.totalSyncs,
        successRate: Math.round(syncStats.syncEfficiency),
        totalConflicts: conflictAnalytics.totalConflicts,
        autoResolved: conflictAnalytics.autoResolvedConflicts
      });
    };

    updateStats();
    const interval = setInterval(updateStats, 5000);
    return () => clearInterval(interval);
  }, []);

  if (!stats) return null;

  return (
    <View style={styles.statsGrid}>
      <View style={styles.statItem}>
        <Text style={styles.statValue}>{stats.totalSyncs}</Text>
        <Text style={styles.statLabel}>Total Syncs</Text>
      </View>
      
      <View style={styles.statItem}>
        <Text style={styles.statValue}>{stats.successRate}%</Text>
        <Text style={styles.statLabel}>Success Rate</Text>
      </View>
      
      <View style={styles.statItem}>
        <Text style={styles.statValue}>{stats.totalConflicts}</Text>
        <Text style={styles.statLabel}>Conflicts</Text>
      </View>
      
      <View style={styles.statItem}>
        <Text style={styles.statValue}>{stats.autoResolved}</Text>
        <Text style={styles.statLabel}>Auto-Resolved</Text>
      </View>
    </View>
  );
};

const styles = StyleSheet.create({
  container: {
    backgroundColor: '#FFFFFF',
    borderRadius: 12,
    padding: 16,
    margin: 8,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.1,
    shadowRadius: 4,
    elevation: 3,
  },
  compactContainer: {
    flexDirection: 'row',
    alignItems: 'center',
    padding: 8,
    borderRadius: 20,
    backgroundColor: '#F8F9FA',
  },
  statusRow: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  iconContainer: {
    marginRight: 12,
  },
  compactIcon: {
    position: 'relative',
  },
  iconBackground: {
    width: 32,
    height: 32,
    borderRadius: 16,
    justifyContent: 'center',
    alignItems: 'center',
  },
  iconText: {
    fontSize: 16,
    fontWeight: 'bold',
  },
  statusContent: {
    flex: 1,
    marginRight: 12,
  },
  statusMessage: {
    fontSize: 14,
    fontWeight: '500',
    color: '#1C1C1E',
    marginBottom: 4,
  },
  progressContainer: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  progressTrack: {
    flex: 1,
    height: 4,
    backgroundColor: '#E5E5EA',
    borderRadius: 2,
    marginRight: 8,
  },
  progressBar: {
    height: '100%',
    borderRadius: 2,
  },
  progressText: {
    fontSize: 12,
    color: '#8E8E93',
    minWidth: 35,
    textAlign: 'right',
  },
  actionButtons: {
    flexDirection: 'row',
    gap: 8,
  },
  actionButton: {
    width: 32,
    height: 32,
    borderRadius: 16,
    justifyContent: 'center',
    alignItems: 'center',
  },
  conflictButton: {
    backgroundColor: '#FF3B30',
  },
  conflictButtonText: {
    color: '#FFFFFF',
    fontSize: 12,
    fontWeight: 'bold',
  },
  conflictBadge: {
    position: 'absolute',
    top: -4,
    right: -4,
    backgroundColor: '#FF3B30',
    borderRadius: 10,
    minWidth: 20,
    height: 20,
    justifyContent: 'center',
    alignItems: 'center',
  },
  conflictBadgeText: {
    color: '#FFFFFF',
    fontSize: 12,
    fontWeight: 'bold',
  },
  syncButton: {
    backgroundColor: '#007AFF',
  },
  syncButtonText: {
    color: '#FFFFFF',
    fontSize: 16,
    fontWeight: 'bold',
  },
  detailsButton: {
    backgroundColor: '#E5E5EA',
  },
  detailsButtonText: {
    color: '#1C1C1E',
    fontSize: 16,
    fontWeight: 'bold',
  },
  expandedStats: {
    marginTop: 16,
    paddingTop: 16,
    borderTopWidth: 1,
    borderTopColor: '#E5E5EA',
    overflow: 'hidden',
  },
  statsGrid: {
    flexDirection: 'row',
    justifyContent: 'space-around',
  },
  statItem: {
    alignItems: 'center',
  },
  statValue: {
    fontSize: 16,
    fontWeight: 'bold',
    color: '#1C1C1E',
  },
  statLabel: {
    fontSize: 12,
    color: '#8E8E93',
    marginTop: 2,
  },
});
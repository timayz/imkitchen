/**
 * Conflict History Screen Component
 * 
 * Shows past resolution decisions and audit logging with timestamps 
 * and user actions for conflict resolution tracking and analysis.
 * 
 * Features:
 * - Comprehensive conflict resolution history
 * - Audit trail with timestamps and user actions
 * - Resolution outcome tracking
 * - Conflict pattern analysis
 * - Resolution effectiveness metrics
 * - Rollback capabilities for recent resolutions
 */

import React, { useState, useEffect } from 'react';
import {
  View,
  Text,
  FlatList,
  TouchableOpacity,
  StyleSheet,
  RefreshControl,
  Alert,
  ActivityIndicator
} from 'react-native';
import { 
  ConflictAnalytics, 
  ConflictResolutionRecord, 
  ResolutionType,
  ConflictPattern
} from '../../services/conflict_resolution_service';
import { conflictResolutionService } from '../../services/conflict_resolution_service';

interface ConflictHistoryEntry {
  id: string;
  itemType: string;
  itemId: string;
  conflictDetectedAt: Date;
  resolutionRecord: ConflictResolutionRecord;
  canRollback: boolean;
  rollbackId?: string;
}

interface ConflictHistoryScreenProps {
  navigation?: any;
}

const ConflictHistoryScreen: React.FC<ConflictHistoryScreenProps> = ({ navigation }) => {
  const [historyEntries, setHistoryEntries] = useState<ConflictHistoryEntry[]>([]);
  const [analytics, setAnalytics] = useState<ConflictAnalytics | null>(null);
  const [conflictPatterns, setConflictPatterns] = useState<ConflictPattern[]>([]);
  const [loading, setLoading] = useState(true);
  const [refreshing, setRefreshing] = useState(false);
  const [selectedTab, setSelectedTab] = useState<'history' | 'analytics' | 'patterns'>('history');

  useEffect(() => {
    loadConflictHistory();
  }, []);

  const loadConflictHistory = async () => {
    try {
      setLoading(true);
      
      // Load conflict resolution history
      const history = await conflictResolutionService.getResolutionHistory();
      const formattedHistory = history.map((entry, index) => ({
        id: `conflict_${index}`,
        itemType: entry.itemType || 'unknown',
        itemId: entry.itemId || `item_${index}`,
        conflictDetectedAt: new Date(entry.detectedAt || Date.now()),
        resolutionRecord: entry,
        canRollback: entry.outcome === 'success' && 
                     (Date.now() - new Date(entry.resolvedAt).getTime()) < (24 * 60 * 60 * 1000), // 24 hours
        rollbackId: entry.rollbackId
      }));
      
      setHistoryEntries(formattedHistory);
      
      // Load analytics
      const analyticsData = await conflictResolutionService.getAnalytics();
      setAnalytics(analyticsData);
      
      // Load conflict patterns
      const patterns = await conflictResolutionService.getConflictPatterns();
      setConflictPatterns(patterns);
      
    } catch (error) {
      console.error('[ConflictHistory] Failed to load conflict history:', error);
      Alert.alert('Error', 'Failed to load conflict history');
    } finally {
      setLoading(false);
    }
  };

  const handleRefresh = async () => {
    setRefreshing(true);
    await loadConflictHistory();
    setRefreshing(false);
  };

  const handleRollback = async (entry: ConflictHistoryEntry) => {
    Alert.alert(
      'Rollback Resolution',
      `Are you sure you want to rollback the resolution for ${entry.itemType} ${entry.itemId}?`,
      [
        { text: 'Cancel', style: 'cancel' },
        {
          text: 'Rollback',
          style: 'destructive',
          onPress: async () => {
            try {
              await conflictResolutionService.rollbackResolution(entry.rollbackId!);
              Alert.alert('Success', 'Resolution has been rolled back');
              loadConflictHistory();
            } catch (error) {
              console.error('[ConflictHistory] Rollback failed:', error);
              Alert.alert('Error', 'Failed to rollback resolution');
            }
          }
        }
      ]
    );
  };

  const formatDate = (date: Date): string => {
    return new Intl.DateTimeFormat('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    }).format(date);
  };

  const formatDuration = (startDate: Date, endDate: Date): string => {
    const durationMs = endDate.getTime() - startDate.getTime();
    const seconds = Math.floor(durationMs / 1000);
    const minutes = Math.floor(seconds / 60);
    
    if (minutes > 0) {
      return `${minutes}m ${seconds % 60}s`;
    }
    return `${seconds}s`;
  };

  const getResolutionTypeDisplay = (type: ResolutionType): string => {
    switch (type) {
      case ResolutionType.LOCAL_WINS:
        return 'Keep Local';
      case ResolutionType.REMOTE_WINS:
        return 'Keep Remote';
      case ResolutionType.LAST_WRITE_WINS:
        return 'Most Recent';
      case ResolutionType.FIELD_LEVEL_MERGE:
        return 'Field Merge';
      case ResolutionType.SEMANTIC_MERGE:
        return 'Smart Merge';
      case ResolutionType.USER_GUIDED:
        return 'Manual';
      case ResolutionType.CUSTOM_MERGE:
        return 'Custom';
      default:
        return 'Unknown';
    }
  };

  const getOutcomeColor = (outcome: string): string => {
    switch (outcome) {
      case 'success':
        return '#28A745';
      case 'failed':
        return '#DC3545';
      case 'partial':
        return '#FFC107';
      default:
        return '#6C757D';
    }
  };

  const renderHistoryItem = ({ item }: { item: ConflictHistoryEntry }) => (
    <View style={styles.historyItem}>
      <View style={styles.historyHeader}>
        <Text style={styles.itemType}>{item.itemType}</Text>
        <Text style={styles.itemId} numberOfLines={1}>
          {item.itemId}
        </Text>
        <View style={[styles.outcomeIndicator, { backgroundColor: getOutcomeColor(item.resolutionRecord.outcome) }]} />
      </View>
      
      <View style={styles.historyDetails}>
        <Text style={styles.resolutionStrategy}>
          {getResolutionTypeDisplay(item.resolutionRecord.strategy)}
        </Text>
        <Text style={styles.resolvedBy}>
          by {item.resolutionRecord.resolvedBy}
        </Text>
      </View>
      
      <View style={styles.historyTimeline}>
        <Text style={styles.timelineText}>
          Detected: {formatDate(item.conflictDetectedAt)}
        </Text>
        <Text style={styles.timelineText}>
          Resolved: {formatDate(new Date(item.resolutionRecord.resolvedAt))}
        </Text>
        <Text style={styles.durationText}>
          Duration: {formatDuration(item.conflictDetectedAt, new Date(item.resolutionRecord.resolvedAt))}
        </Text>
      </View>
      
      <View style={styles.historyMetrics}>
        <View style={styles.metricItem}>
          <Text style={styles.metricLabel}>Confidence</Text>
          <Text style={styles.metricValue}>{item.resolutionRecord.confidence}%</Text>
        </View>
        
        {item.resolutionRecord.userFeedback && (
          <View style={styles.feedbackContainer}>
            <Text style={styles.feedbackLabel}>Feedback:</Text>
            <Text style={styles.feedbackText}>{item.resolutionRecord.userFeedback}</Text>
          </View>
        )}
      </View>
      
      {item.canRollback && (
        <TouchableOpacity
          style={styles.rollbackButton}
          onPress={() => handleRollback(item)}
        >
          <Text style={styles.rollbackButtonText}>Rollback</Text>
        </TouchableOpacity>
      )}
    </View>
  );

  const renderAnalytics = () => {
    if (!analytics) {
      return (
        <View style={styles.loadingContainer}>
          <ActivityIndicator size="large" color="#007AFF" />
        </View>
      );
    }

    return (
      <View style={styles.analyticsContainer}>
        <View style={styles.analyticsGrid}>
          <View style={styles.analyticsItem}>
            <Text style={styles.analyticsValue}>{analytics.totalConflicts}</Text>
            <Text style={styles.analyticsLabel}>Total Conflicts</Text>
          </View>
          
          <View style={styles.analyticsItem}>
            <Text style={styles.analyticsValue}>{analytics.autoResolvedConflicts}</Text>
            <Text style={styles.analyticsLabel}>Auto Resolved</Text>
          </View>
          
          <View style={styles.analyticsItem}>
            <Text style={styles.analyticsValue}>{analytics.userResolvedConflicts}</Text>
            <Text style={styles.analyticsLabel}>User Resolved</Text>
          </View>
          
          <View style={styles.analyticsItem}>
            <Text style={styles.analyticsValue}>{analytics.resolutionSuccessRate.toFixed(1)}%</Text>
            <Text style={styles.analyticsLabel}>Success Rate</Text>
          </View>
        </View>
        
        <View style={styles.averageTimeContainer}>
          <Text style={styles.sectionTitle}>Average Resolution Time</Text>
          <Text style={styles.averageTimeValue}>
            {Math.floor(analytics.averageResolutionTime / 1000)}s
          </Text>
        </View>
        
        <View style={styles.effectivenessContainer}>
          <Text style={styles.sectionTitle}>Resolution Effectiveness</Text>
          {Object.entries(analytics.resolutionEffectiveness).map(([type, effectiveness]) => (
            <View key={type} style={styles.effectivenessItem}>
              <Text style={styles.effectivenessType}>
                {getResolutionTypeDisplay(type as ResolutionType)}
              </Text>
              <View style={styles.effectivenessBar}>
                <View 
                  style={[
                    styles.effectivenessProgress, 
                    { width: `${effectiveness}%` }
                  ]} 
                />
              </View>
              <Text style={styles.effectivenessValue}>{effectiveness}%</Text>
            </View>
          ))}
        </View>
      </View>
    );
  };

  const renderPatterns = () => (
    <FlatList
      data={conflictPatterns}
      keyExtractor={(item, index) => `pattern_${index}`}
      renderItem={({ item }) => (
        <View style={styles.patternItem}>
          <Text style={styles.patternDescription}>{item.description}</Text>
          <View style={styles.patternMetrics}>
            <Text style={styles.patternFrequency}>
              Frequency: {item.frequency}
            </Text>
            <Text style={styles.patternSuccess}>
              Success Rate: {(item.successRate * 100).toFixed(1)}%
            </Text>
          </View>
          <Text style={styles.recommendedStrategy}>
            Recommended: {getResolutionTypeDisplay(item.recommendedStrategy)}
          </Text>
        </View>
      )}
      ListEmptyComponent={
        <View style={styles.emptyContainer}>
          <Text style={styles.emptyText}>No conflict patterns found</Text>
        </View>
      }
    />
  );

  const renderTabContent = () => {
    switch (selectedTab) {
      case 'history':
        return (
          <FlatList
            data={historyEntries}
            keyExtractor={(item) => item.id}
            renderItem={renderHistoryItem}
            refreshControl={
              <RefreshControl refreshing={refreshing} onRefresh={handleRefresh} />
            }
            ListEmptyComponent={
              <View style={styles.emptyContainer}>
                <Text style={styles.emptyText}>No conflict history found</Text>
              </View>
            }
          />
        );
      case 'analytics':
        return renderAnalytics();
      case 'patterns':
        return renderPatterns();
      default:
        return null;
    }
  };

  if (loading) {
    return (
      <View style={styles.loadingContainer}>
        <ActivityIndicator size="large" color="#007AFF" />
        <Text style={styles.loadingText}>Loading conflict history...</Text>
      </View>
    );
  }

  return (
    <View style={styles.container}>
      <View style={styles.tabContainer}>
        <TouchableOpacity
          style={[styles.tab, selectedTab === 'history' && styles.activeTab]}
          onPress={() => setSelectedTab('history')}
        >
          <Text style={[styles.tabText, selectedTab === 'history' && styles.activeTabText]}>
            History
          </Text>
        </TouchableOpacity>
        
        <TouchableOpacity
          style={[styles.tab, selectedTab === 'analytics' && styles.activeTab]}
          onPress={() => setSelectedTab('analytics')}
        >
          <Text style={[styles.tabText, selectedTab === 'analytics' && styles.activeTabText]}>
            Analytics
          </Text>
        </TouchableOpacity>
        
        <TouchableOpacity
          style={[styles.tab, selectedTab === 'patterns' && styles.activeTab]}
          onPress={() => setSelectedTab('patterns')}
        >
          <Text style={[styles.tabText, selectedTab === 'patterns' && styles.activeTabText]}>
            Patterns
          </Text>
        </TouchableOpacity>
      </View>
      
      <View style={styles.content}>
        {renderTabContent()}
      </View>
    </View>
  );
};

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#FFFFFF'
  },
  loadingContainer: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center'
  },
  loadingText: {
    marginTop: 16,
    fontSize: 16,
    color: '#666666'
  },
  tabContainer: {
    flexDirection: 'row',
    borderBottomWidth: 1,
    borderBottomColor: '#E0E0E0'
  },
  tab: {
    flex: 1,
    paddingVertical: 16,
    alignItems: 'center',
    backgroundColor: '#F8F9FA'
  },
  activeTab: {
    backgroundColor: '#FFFFFF',
    borderBottomWidth: 2,
    borderBottomColor: '#007AFF'
  },
  tabText: {
    fontSize: 16,
    color: '#666666',
    fontWeight: '500'
  },
  activeTabText: {
    color: '#007AFF',
    fontWeight: '600'
  },
  content: {
    flex: 1
  },
  historyItem: {
    padding: 16,
    marginHorizontal: 16,
    marginVertical: 8,
    backgroundColor: '#F8F9FA',
    borderRadius: 8,
    borderWidth: 1,
    borderColor: '#E0E0E0'
  },
  historyHeader: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 8
  },
  itemType: {
    fontSize: 16,
    fontWeight: '600',
    color: '#333333'
  },
  itemId: {
    flex: 1,
    fontSize: 14,
    color: '#666666',
    marginLeft: 8
  },
  outcomeIndicator: {
    width: 12,
    height: 12,
    borderRadius: 6
  },
  historyDetails: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 8
  },
  resolutionStrategy: {
    fontSize: 14,
    fontWeight: '500',
    color: '#007AFF'
  },
  resolvedBy: {
    fontSize: 12,
    color: '#666666',
    marginLeft: 8
  },
  historyTimeline: {
    marginBottom: 8
  },
  timelineText: {
    fontSize: 12,
    color: '#666666',
    marginBottom: 2
  },
  durationText: {
    fontSize: 12,
    color: '#28A745',
    fontWeight: '500'
  },
  historyMetrics: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 8
  },
  metricItem: {
    marginRight: 16
  },
  metricLabel: {
    fontSize: 10,
    color: '#666666',
    textTransform: 'uppercase'
  },
  metricValue: {
    fontSize: 14,
    fontWeight: '600',
    color: '#333333'
  },
  feedbackContainer: {
    flex: 1,
    marginTop: 8
  },
  feedbackLabel: {
    fontSize: 10,
    color: '#666666',
    textTransform: 'uppercase',
    marginBottom: 2
  },
  feedbackText: {
    fontSize: 12,
    color: '#333333',
    fontStyle: 'italic'
  },
  rollbackButton: {
    alignSelf: 'flex-start',
    paddingHorizontal: 12,
    paddingVertical: 6,
    backgroundColor: '#FFC107',
    borderRadius: 4,
    marginTop: 8
  },
  rollbackButtonText: {
    fontSize: 12,
    fontWeight: '600',
    color: '#FFFFFF'
  },
  analyticsContainer: {
    padding: 16
  },
  analyticsGrid: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    marginBottom: 24
  },
  analyticsItem: {
    width: '50%',
    padding: 16,
    alignItems: 'center',
    backgroundColor: '#F8F9FA',
    borderRadius: 8,
    marginBottom: 8
  },
  analyticsValue: {
    fontSize: 24,
    fontWeight: 'bold',
    color: '#007AFF'
  },
  analyticsLabel: {
    fontSize: 12,
    color: '#666666',
    textAlign: 'center',
    marginTop: 4
  },
  averageTimeContainer: {
    padding: 16,
    backgroundColor: '#F0F8FF',
    borderRadius: 8,
    alignItems: 'center',
    marginBottom: 24
  },
  sectionTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#333333',
    marginBottom: 8
  },
  averageTimeValue: {
    fontSize: 32,
    fontWeight: 'bold',
    color: '#007AFF'
  },
  effectivenessContainer: {
    backgroundColor: '#F8F9FA',
    padding: 16,
    borderRadius: 8
  },
  effectivenessItem: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 12
  },
  effectivenessType: {
    width: 100,
    fontSize: 12,
    color: '#666666'
  },
  effectivenessBar: {
    flex: 1,
    height: 8,
    backgroundColor: '#E0E0E0',
    borderRadius: 4,
    marginHorizontal: 12
  },
  effectivenessProgress: {
    height: '100%',
    backgroundColor: '#28A745',
    borderRadius: 4
  },
  effectivenessValue: {
    width: 40,
    fontSize: 12,
    fontWeight: '600',
    color: '#333333',
    textAlign: 'right'
  },
  patternItem: {
    padding: 16,
    marginHorizontal: 16,
    marginVertical: 8,
    backgroundColor: '#F8F9FA',
    borderRadius: 8,
    borderWidth: 1,
    borderColor: '#E0E0E0'
  },
  patternDescription: {
    fontSize: 14,
    color: '#333333',
    marginBottom: 8
  },
  patternMetrics: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    marginBottom: 8
  },
  patternFrequency: {
    fontSize: 12,
    color: '#666666'
  },
  patternSuccess: {
    fontSize: 12,
    color: '#28A745',
    fontWeight: '500'
  },
  recommendedStrategy: {
    fontSize: 12,
    color: '#007AFF',
    fontWeight: '500'
  },
  emptyContainer: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    padding: 32
  },
  emptyText: {
    fontSize: 16,
    color: '#666666',
    textAlign: 'center'
  }
});

export default ConflictHistoryScreen;
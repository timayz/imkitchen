import React, { useState, useEffect } from 'react';
import { View, Text, StyleSheet, ScrollView, TouchableOpacity, RefreshControl, Alert } from 'react-native';
import { analyticsService } from '../../services/analytics_service';
import type { RotationDebugLog } from '../../types/analytics';

export const RotationDebugViewer: React.FC = () => {
  const [debugLogs, setDebugLogs] = useState<RotationDebugLog[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [isRefreshing, setIsRefreshing] = useState(false);
  const [selectedDecisionType, setSelectedDecisionType] = useState<'all' | 'recipe_selection' | 'constraint_violation' | 'fallback_triggered'>('all');
  const [expandedLogId, setExpandedLogId] = useState<string | null>(null);

  const loadDebugLogs = async (showLoader: boolean = true) => {
    if (showLoader) {
      setIsLoading(true);
    }

    try {
      const logs = await analyticsService.getDebugLogs(100);
      setDebugLogs(logs);
    } catch (error) {
      console.error('Failed to load debug logs:', error);
      Alert.alert(
        'Error',
        'Failed to load debug logs. Please try again.',
        [{ text: 'OK' }]
      );
    } finally {
      setIsLoading(false);
      setIsRefreshing(false);
    }
  };

  const handleRefresh = () => {
    setIsRefreshing(true);
    loadDebugLogs(false);
  };

  const filteredLogs = debugLogs.filter(log => 
    selectedDecisionType === 'all' || log.decisionType === selectedDecisionType
  );

  const getDecisionTypeColor = (type: string) => {
    switch (type) {
      case 'recipe_selection': return '#3498db';
      case 'constraint_violation': return '#e74c3c';
      case 'fallback_triggered': return '#f39c12';
      default: return '#95a5a6';
    }
  };

  const getDecisionTypeLabel = (type: string) => {
    switch (type) {
      case 'recipe_selection': return 'Recipe Selection';
      case 'constraint_violation': return 'Constraint Violation';
      case 'fallback_triggered': return 'Fallback Triggered';
      default: return 'Unknown';
    }
  };

  const renderLogItem = (log: RotationDebugLog) => {
    const isExpanded = expandedLogId === log.id;
    const decisionColor = getDecisionTypeColor(log.decisionType);

    return (
      <View key={log.id} style={styles.logItem}>
        <TouchableOpacity
          style={styles.logHeader}
          onPress={() => setExpandedLogId(isExpanded ? null : log.id)}
        >
          <View style={styles.logHeaderLeft}>
            <View style={[styles.decisionTypeIndicator, { backgroundColor: decisionColor }]} />
            <View style={styles.logHeaderContent}>
              <Text style={styles.logTitle}>
                {getDecisionTypeLabel(log.decisionType)}
              </Text>
              <Text style={styles.logTimestamp}>
                {new Date(log.timestamp).toLocaleString()}
              </Text>
            </View>
          </View>
          <Text style={styles.expandIcon}>
            {isExpanded ? '▼' : '▶'}
          </Text>
        </TouchableOpacity>

        {isExpanded && (
          <View style={styles.logDetails}>
            <View style={styles.logField}>
              <Text style={styles.fieldLabel}>Algorithm Version:</Text>
              <Text style={styles.fieldValue}>{log.algorithmVersion}</Text>
            </View>

            {log.recipeName && (
              <View style={styles.logField}>
                <Text style={styles.fieldLabel}>Recipe:</Text>
                <Text style={styles.fieldValue}>{log.recipeName}</Text>
              </View>
            )}

            {log.constraintViolated && (
              <View style={styles.logField}>
                <Text style={styles.fieldLabel}>Constraint Violated:</Text>
                <Text style={[styles.fieldValue, styles.errorText]}>{log.constraintViolated}</Text>
              </View>
            )}

            {log.fallbackReason && (
              <View style={styles.logField}>
                <Text style={styles.fieldLabel}>Fallback Reason:</Text>
                <Text style={[styles.fieldValue, styles.warningText]}>{log.fallbackReason}</Text>
              </View>
            )}

            <View style={styles.logActions}>
              <TouchableOpacity
                style={styles.actionButton}
                onPress={() => handleShareLog(log)}
              >
                <Text style={styles.actionButtonText}>Share Log</Text>
              </TouchableOpacity>
            </View>
          </View>
        )}
      </View>
    );
  };

  const handleShareLog = (log: RotationDebugLog) => {
    // This would integrate with React Native's Share API
    const logText = `
Rotation Debug Log
Type: ${getDecisionTypeLabel(log.decisionType)}
Time: ${new Date(log.timestamp).toLocaleString()}
Algorithm: ${log.algorithmVersion}
${log.recipeName ? `Recipe: ${log.recipeName}` : ''}
${log.constraintViolated ? `Constraint: ${log.constraintViolated}` : ''}
${log.fallbackReason ? `Fallback: ${log.fallbackReason}` : ''}
    `.trim();
    
    console.log('Sharing debug log:', logText);
    // Implementation would use react-native-share or similar
  };

  useEffect(() => {
    loadDebugLogs();
  }, []);

  if (isLoading && debugLogs.length === 0) {
    return (
      <View style={styles.loadingContainer}>
        <Text style={styles.loadingText}>Loading debug logs...</Text>
      </View>
    );
  }

  return (
    <View style={styles.container}>
      <View style={styles.header}>
        <Text style={styles.title}>Rotation Debug Logs</Text>
        <Text style={styles.subtitle}>
          Algorithm decision history and troubleshooting information
        </Text>
      </View>

      {/* Decision Type Filter */}
      <View style={styles.filterSection}>
        <Text style={styles.filterLabel}>Filter by type:</Text>
        <ScrollView horizontal showsHorizontalScrollIndicator={false}>
          <View style={styles.filterOptions}>
            {['all', 'recipe_selection', 'constraint_violation', 'fallback_triggered'].map(type => (
              <TouchableOpacity
                key={type}
                style={[
                  styles.filterOption,
                  selectedDecisionType === type && styles.filterOptionSelected
                ]}
                onPress={() => setSelectedDecisionType(type as any)}
              >
                <Text style={[
                  styles.filterOptionText,
                  selectedDecisionType === type && styles.filterOptionTextSelected
                ]}>
                  {type === 'all' ? 'All' : getDecisionTypeLabel(type)}
                </Text>
              </TouchableOpacity>
            ))}
          </View>
        </ScrollView>
      </View>

      {/* Stats Summary */}
      <View style={styles.statsSection}>
        <View style={styles.statItem}>
          <Text style={styles.statValue}>{debugLogs.length}</Text>
          <Text style={styles.statLabel}>Total Logs</Text>
        </View>
        <View style={styles.statItem}>
          <Text style={[styles.statValue, { color: '#e74c3c' }]}>
            {debugLogs.filter(log => log.decisionType === 'constraint_violation').length}
          </Text>
          <Text style={styles.statLabel}>Violations</Text>
        </View>
        <View style={styles.statItem}>
          <Text style={[styles.statValue, { color: '#f39c12' }]}>
            {debugLogs.filter(log => log.decisionType === 'fallback_triggered').length}
          </Text>
          <Text style={styles.statLabel}>Fallbacks</Text>
        </View>
      </View>

      {/* Debug Logs List */}
      <ScrollView
        style={styles.logsList}
        refreshControl={
          <RefreshControl
            refreshing={isRefreshing}
            onRefresh={handleRefresh}
            colors={['#4CAF50']}
          />
        }
      >
        {filteredLogs.length === 0 ? (
          <View style={styles.emptyState}>
            <Text style={styles.emptyText}>No debug logs found</Text>
            <Text style={styles.emptySubtext}>
              {selectedDecisionType === 'all' 
                ? 'Generate some meal plans to see algorithm decisions here'
                : `No ${getDecisionTypeLabel(selectedDecisionType).toLowerCase()} logs found`
              }
            </Text>
          </View>
        ) : (
          filteredLogs.map(renderLogItem)
        )}
      </ScrollView>

      <View style={styles.footer}>
        <Text style={styles.footerText}>
          Showing {filteredLogs.length} of {debugLogs.length} logs
        </Text>
        <TouchableOpacity
          style={styles.refreshButton}
          onPress={handleRefresh}
        >
          <Text style={styles.refreshButtonText}>🔄 Refresh</Text>
        </TouchableOpacity>
      </View>
    </View>
  );
};

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#f5f5f5',
  },
  loadingContainer: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    backgroundColor: '#f5f5f5',
  },
  loadingText: {
    fontSize: 16,
    color: '#666',
  },
  header: {
    padding: 16,
    backgroundColor: '#fff',
    borderBottomWidth: 1,
    borderBottomColor: '#e0e0e0',
  },
  title: {
    fontSize: 18,
    fontWeight: 'bold',
    color: '#2c3e50',
    marginBottom: 4,
  },
  subtitle: {
    fontSize: 14,
    color: '#7f8c8d',
  },
  filterSection: {
    padding: 16,
    backgroundColor: '#fff',
    borderBottomWidth: 1,
    borderBottomColor: '#f1f2f6',
  },
  filterLabel: {
    fontSize: 14,
    fontWeight: '600',
    color: '#2c3e50',
    marginBottom: 8,
  },
  filterOptions: {
    flexDirection: 'row',
    gap: 8,
  },
  filterOption: {
    paddingHorizontal: 12,
    paddingVertical: 6,
    borderRadius: 16,
    backgroundColor: '#f8f9fa',
    borderWidth: 1,
    borderColor: '#e9ecef',
  },
  filterOptionSelected: {
    backgroundColor: '#3498db',
    borderColor: '#3498db',
  },
  filterOptionText: {
    fontSize: 12,
    color: '#6c757d',
  },
  filterOptionTextSelected: {
    color: '#fff',
    fontWeight: '600',
  },
  statsSection: {
    flexDirection: 'row',
    padding: 16,
    backgroundColor: '#fff',
    borderBottomWidth: 1,
    borderBottomColor: '#f1f2f6',
  },
  statItem: {
    flex: 1,
    alignItems: 'center',
  },
  statValue: {
    fontSize: 20,
    fontWeight: 'bold',
    color: '#3498db',
  },
  statLabel: {
    fontSize: 12,
    color: '#7f8c8d',
    marginTop: 2,
  },
  logsList: {
    flex: 1,
  },
  logItem: {
    backgroundColor: '#fff',
    marginHorizontal: 16,
    marginVertical: 4,
    borderRadius: 8,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 1 },
    shadowOpacity: 0.1,
    shadowRadius: 2,
    elevation: 2,
  },
  logHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    padding: 12,
  },
  logHeaderLeft: {
    flexDirection: 'row',
    alignItems: 'center',
    flex: 1,
  },
  decisionTypeIndicator: {
    width: 8,
    height: 8,
    borderRadius: 4,
    marginRight: 12,
  },
  logHeaderContent: {
    flex: 1,
  },
  logTitle: {
    fontSize: 14,
    fontWeight: '600',
    color: '#2c3e50',
  },
  logTimestamp: {
    fontSize: 12,
    color: '#7f8c8d',
    marginTop: 2,
  },
  expandIcon: {
    fontSize: 12,
    color: '#95a5a6',
  },
  logDetails: {
    paddingHorizontal: 32,
    paddingBottom: 12,
    borderTopWidth: 1,
    borderTopColor: '#f1f2f6',
  },
  logField: {
    marginBottom: 8,
  },
  fieldLabel: {
    fontSize: 12,
    fontWeight: '600',
    color: '#7f8c8d',
    marginBottom: 2,
  },
  fieldValue: {
    fontSize: 14,
    color: '#2c3e50',
  },
  errorText: {
    color: '#e74c3c',
  },
  warningText: {
    color: '#f39c12',
  },
  logActions: {
    flexDirection: 'row',
    justifyContent: 'flex-end',
    marginTop: 8,
  },
  actionButton: {
    paddingHorizontal: 12,
    paddingVertical: 6,
    borderRadius: 4,
    backgroundColor: '#f8f9fa',
    borderWidth: 1,
    borderColor: '#e9ecef',
  },
  actionButtonText: {
    fontSize: 12,
    color: '#6c757d',
  },
  emptyState: {
    alignItems: 'center',
    padding: 32,
    marginTop: 32,
  },
  emptyText: {
    fontSize: 16,
    color: '#7f8c8d',
    marginBottom: 8,
  },
  emptySubtext: {
    fontSize: 14,
    color: '#95a5a6',
    textAlign: 'center',
  },
  footer: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    padding: 16,
    backgroundColor: '#fff',
    borderTopWidth: 1,
    borderTopColor: '#e0e0e0',
  },
  footerText: {
    fontSize: 12,
    color: '#7f8c8d',
  },
  refreshButton: {
    paddingHorizontal: 12,
    paddingVertical: 6,
    borderRadius: 4,
    backgroundColor: '#3498db',
  },
  refreshButtonText: {
    fontSize: 12,
    color: '#fff',
    fontWeight: '600',
  },
});
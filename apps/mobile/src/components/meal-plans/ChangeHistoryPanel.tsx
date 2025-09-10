import React, { useState, useEffect } from 'react';
import {
  View,
  Text,
  ScrollView,
  TouchableOpacity,
  Modal,
  StyleSheet,
  SafeAreaView,
  RefreshControl,
} from 'react-native';
import { UndoRedoControls } from '../atoms/UndoRedoControls';
import type { MealPlanChange } from '../../services/ChangeHistoryTracker';

interface ChangeHistoryPanelProps {
  visible: boolean;
  onClose: () => void;
  changes: MealPlanChange[];
  currentIndex: number;
  canUndo: boolean;
  canRedo: boolean;
  onUndo: () => void;
  onRedo: () => void;
  onJumpToChange?: (index: number) => void;
  onClearHistory?: () => void;
  refreshing?: boolean;
  onRefresh?: () => void;
}

export const ChangeHistoryPanel: React.FC<ChangeHistoryPanelProps> = ({
  visible,
  onClose,
  changes,
  currentIndex,
  canUndo,
  canRedo,
  onUndo,
  onRedo,
  onJumpToChange,
  onClearHistory,
  refreshing = false,
  onRefresh,
}) => {
  const [showDetails, setShowDetails] = useState<string | null>(null);

  const formatTime = (date: Date) => {
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMinutes = Math.floor(diffMs / (1000 * 60));
    const diffHours = Math.floor(diffMs / (1000 * 60 * 60));
    const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));

    if (diffMinutes < 1) return 'Just now';
    if (diffMinutes < 60) return `${diffMinutes}m ago`;
    if (diffHours < 24) return `${diffHours}h ago`;
    if (diffDays < 7) return `${diffDays}d ago`;
    
    return date.toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  const getChangeIcon = (type: MealPlanChange['type']): string => {
    switch (type) {
      case 'substitution':
      case 'swap':
        return '🔄';
      case 'reorder':
        return '↕️';
      case 'lock':
        return '🔒';
      case 'unlock':
        return '🔓';
      case 'add':
        return '➕';
      case 'remove':
        return '➖';
      case 'batch':
        return '📦';
      default:
        return '✏️';
    }
  };

  const getChangeColor = (type: MealPlanChange['type']): string => {
    switch (type) {
      case 'substitution':
      case 'swap':
        return '#007AFF';
      case 'reorder':
        return '#FF9500';
      case 'lock':
        return '#FF3B30';
      case 'unlock':
        return '#34C759';
      case 'add':
        return '#30D158';
      case 'remove':
        return '#FF453A';
      case 'batch':
        return '#5856D6';
      default:
        return '#8E8E93';
    }
  };

  const isCurrentChange = (index: number): boolean => {
    return index === currentIndex;
  };

  const isFutureChange = (index: number): boolean => {
    return index > currentIndex;
  };

  const handleJumpToChange = (index: number) => {
    if (onJumpToChange) {
      onJumpToChange(index);
    }
  };

  const toggleDetails = (changeId: string) => {
    setShowDetails(showDetails === changeId ? null : changeId);
  };

  const renderChangeItem = (change: MealPlanChange, index: number) => {
    const isSelected = isCurrentChange(index);
    const isFuture = isFutureChange(index);
    const changeColor = getChangeColor(change.type);

    return (
      <TouchableOpacity
        key={change.id}
        style={[
          styles.changeItem,
          isSelected && styles.currentChangeItem,
          isFuture && styles.futureChangeItem,
        ]}
        onPress={() => handleJumpToChange(index)}
        disabled={!onJumpToChange}
        accessibilityRole="button"
        accessibilityLabel={`Change: ${change.description}`}
        accessibilityHint={
          isSelected 
            ? "Current state" 
            : isFuture 
              ? "Future change, can be redone" 
              : "Past change, can be undone"
        }
      >
        <View style={styles.changeHeader}>
          <View style={styles.changeInfo}>
            <View style={styles.changeTitleRow}>
              <Text style={[styles.changeIcon, { color: changeColor }]}>
                {getChangeIcon(change.type)}
              </Text>
              <Text 
                style={[
                  styles.changeDescription,
                  isFuture && styles.futureText
                ]}
                numberOfLines={showDetails === change.id ? undefined : 1}
              >
                {change.description}
              </Text>
            </View>
            
            <View style={styles.changeMetadata}>
              <Text style={[styles.changeTime, isFuture && styles.futureText]}>
                {formatTime(change.timestamp)}
              </Text>
              {change.type === 'batch' && (
                <Text style={[styles.batchIndicator, isFuture && styles.futureText]}>
                  BATCH
                </Text>
              )}
            </View>
          </View>

          {/* State indicator */}
          <View style={styles.stateIndicator}>
            {isSelected && (
              <View style={styles.currentIndicator}>
                <Text style={styles.currentIndicatorText}>●</Text>
              </View>
            )}
          </View>
        </View>

        {/* Expandable details */}
        {change.metadata && (
          <TouchableOpacity 
            style={styles.detailsToggle}
            onPress={() => toggleDetails(change.id)}
            accessibilityRole="button"
            accessibilityLabel="Toggle change details"
          >
            <Text style={styles.detailsToggleText}>
              {showDetails === change.id ? 'Hide Details ↑' : 'Show Details ↓'}
            </Text>
          </TouchableOpacity>
        )}

        {showDetails === change.id && change.metadata && (
          <View style={styles.changeDetails}>
            {change.metadata.reason && (
              <Text style={styles.detailText}>
                <Text style={styles.detailLabel}>Reason:</Text> {change.metadata.reason}
              </Text>
            )}
            
            {change.metadata.affectedSlots && change.metadata.affectedSlots.length > 0 && (
              <Text style={styles.detailText}>
                <Text style={styles.detailLabel}>Affected slots:</Text>{' '}
                {change.metadata.affectedSlots
                  .map(slot => `${slot.day} ${slot.mealType}`)
                  .join(', ')}
              </Text>
            )}
            
            {typeof change.metadata.cost === 'number' && (
              <Text style={styles.detailText}>
                <Text style={styles.detailLabel}>Cost impact:</Text> ${change.metadata.cost.toFixed(2)}
              </Text>
            )}

            {change.metadata.batchId && (
              <Text style={styles.detailText}>
                <Text style={styles.detailLabel}>Batch ID:</Text> {change.metadata.batchId.slice(-8)}
              </Text>
            )}
          </View>
        )}
      </TouchableOpacity>
    );
  };

  const renderEmptyState = () => (
    <View style={styles.emptyState}>
      <Text style={styles.emptyStateIcon}>📋</Text>
      <Text style={styles.emptyStateTitle}>No Changes Yet</Text>
      <Text style={styles.emptyStateMessage}>
        Start making changes to your meal plan and they'll appear here
      </Text>
    </View>
  );

  const renderHeader = () => (
    <View style={styles.header}>
      <View style={styles.headerLeft}>
        <TouchableOpacity onPress={onClose} style={styles.closeButton}>
          <Text style={styles.closeButtonText}>✕</Text>
        </TouchableOpacity>
      </View>

      <View style={styles.headerCenter}>
        <Text style={styles.headerTitle}>Change History</Text>
        <Text style={styles.headerSubtitle}>
          {changes.length} change{changes.length !== 1 ? 's' : ''}
          {currentIndex >= 0 && ` • Position ${currentIndex + 1}`}
        </Text>
      </View>

      <View style={styles.headerRight}>
        {onClearHistory && changes.length > 0 && (
          <TouchableOpacity 
            onPress={onClearHistory}
            style={styles.clearButton}
            accessibilityRole="button"
            accessibilityLabel="Clear all history"
          >
            <Text style={styles.clearButtonText}>Clear</Text>
          </TouchableOpacity>
        )}
      </View>
    </View>
  );

  const renderControls = () => (
    <View style={styles.controls}>
      <UndoRedoControls
        canUndo={canUndo}
        canRedo={canRedo}
        onUndo={onUndo}
        onRedo={onRedo}
        undoDescription={
          canUndo && changes[currentIndex] 
            ? changes[currentIndex].description 
            : undefined
        }
        redoDescription={
          canRedo && changes[currentIndex + 1] 
            ? changes[currentIndex + 1].description 
            : undefined
        }
        size="large"
        style="filled"
        orientation="horizontal"
        showLabels
      />
      
      {(canUndo || canRedo) && (
        <Text style={styles.controlsHint}>
          {canUndo ? `Can undo: ${changes[currentIndex]?.description}` : ''}
          {canUndo && canRedo ? ' • ' : ''}
          {canRedo ? `Can redo: ${changes[currentIndex + 1]?.description}` : ''}
        </Text>
      )}
    </View>
  );

  if (!visible) return null;

  return (
    <Modal
      visible={visible}
      animationType="slide"
      presentationStyle="pageSheet"
      onRequestClose={onClose}
    >
      <SafeAreaView style={styles.container}>
        {renderHeader()}
        
        {renderControls()}

        <ScrollView 
          style={styles.scrollView}
          showsVerticalScrollIndicator={false}
          refreshControl={
            onRefresh ? (
              <RefreshControl
                refreshing={refreshing}
                onRefresh={onRefresh}
                tintColor="#007AFF"
              />
            ) : undefined
          }
        >
          {changes.length === 0 ? (
            renderEmptyState()
          ) : (
            <View style={styles.changesContainer}>
              {changes.map((change, index) => renderChangeItem(change, index))}
            </View>
          )}
        </ScrollView>
      </SafeAreaView>
    </Modal>
  );
};

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#F2F2F7',
  },
  header: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    padding: 16,
    backgroundColor: '#FFFFFF',
    borderBottomWidth: 0.5,
    borderBottomColor: '#C6C6C8',
  },
  headerLeft: {
    flex: 1,
    alignItems: 'flex-start',
  },
  headerCenter: {
    flex: 2,
    alignItems: 'center',
  },
  headerRight: {
    flex: 1,
    alignItems: 'flex-end',
  },
  closeButton: {
    padding: 8,
  },
  closeButtonText: {
    fontSize: 18,
    color: '#007AFF',
  },
  clearButton: {
    paddingVertical: 6,
    paddingHorizontal: 12,
    backgroundColor: '#FF3B30',
    borderRadius: 16,
  },
  clearButtonText: {
    color: '#FFFFFF',
    fontSize: 14,
    fontWeight: '600',
  },
  headerTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#000000',
  },
  headerSubtitle: {
    fontSize: 12,
    color: '#8E8E93',
    marginTop: 2,
  },
  controls: {
    backgroundColor: '#FFFFFF',
    padding: 20,
    alignItems: 'center',
    borderBottomWidth: 0.5,
    borderBottomColor: '#C6C6C8',
  },
  controlsHint: {
    fontSize: 12,
    color: '#8E8E93',
    textAlign: 'center',
    marginTop: 12,
    lineHeight: 16,
  },
  scrollView: {
    flex: 1,
  },
  changesContainer: {
    padding: 16,
  },
  changeItem: {
    backgroundColor: '#FFFFFF',
    borderRadius: 12,
    padding: 16,
    marginBottom: 12,
    borderLeftWidth: 4,
    borderLeftColor: '#E5E5EA',
    shadowColor: '#000000',
    shadowOffset: { width: 0, height: 1 },
    shadowOpacity: 0.1,
    shadowRadius: 2,
    elevation: 2,
  },
  currentChangeItem: {
    borderLeftColor: '#007AFF',
    backgroundColor: '#F0F9FF',
  },
  futureChangeItem: {
    opacity: 0.6,
    borderLeftColor: '#C6C6C8',
  },
  changeHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'flex-start',
  },
  changeInfo: {
    flex: 1,
  },
  changeTitleRow: {
    flexDirection: 'row',
    alignItems: 'flex-start',
    marginBottom: 4,
  },
  changeIcon: {
    fontSize: 16,
    marginRight: 8,
    marginTop: 2,
  },
  changeDescription: {
    flex: 1,
    fontSize: 16,
    fontWeight: '500',
    color: '#000000',
    lineHeight: 20,
  },
  changeMetadata: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 8,
  },
  changeTime: {
    fontSize: 12,
    color: '#8E8E93',
  },
  batchIndicator: {
    fontSize: 10,
    color: '#5856D6',
    fontWeight: '600',
    backgroundColor: '#F0F0FF',
    paddingHorizontal: 6,
    paddingVertical: 2,
    borderRadius: 8,
  },
  futureText: {
    opacity: 0.7,
  },
  stateIndicator: {
    marginLeft: 8,
  },
  currentIndicator: {
    width: 12,
    height: 12,
    borderRadius: 6,
    backgroundColor: '#007AFF',
    justifyContent: 'center',
    alignItems: 'center',
  },
  currentIndicatorText: {
    color: '#FFFFFF',
    fontSize: 8,
  },
  detailsToggle: {
    marginTop: 12,
    paddingVertical: 4,
  },
  detailsToggleText: {
    fontSize: 12,
    color: '#007AFF',
    textAlign: 'center',
  },
  changeDetails: {
    marginTop: 12,
    padding: 12,
    backgroundColor: '#F8F9FA',
    borderRadius: 8,
    gap: 6,
  },
  detailText: {
    fontSize: 12,
    color: '#495057',
    lineHeight: 16,
  },
  detailLabel: {
    fontWeight: '600',
    color: '#212529',
  },
  emptyState: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    padding: 40,
  },
  emptyStateIcon: {
    fontSize: 48,
    marginBottom: 16,
  },
  emptyStateTitle: {
    fontSize: 20,
    fontWeight: '600',
    color: '#000000',
    marginBottom: 8,
  },
  emptyStateMessage: {
    fontSize: 16,
    color: '#8E8E93',
    textAlign: 'center',
    lineHeight: 22,
  },
});
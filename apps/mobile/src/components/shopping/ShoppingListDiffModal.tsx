import React from 'react';
import {
  View,
  Text,
  TouchableOpacity,
  StyleSheet,
  Modal,
  ScrollView,
  Platform,
} from 'react-native';

interface ShoppingListChange {
  type: 'added' | 'removed' | 'modified';
  ingredientName: string;
  oldAmount?: number;
  newAmount?: number;
  unit: string;
}

interface ShoppingListDiffModalProps {
  isVisible: boolean;
  onClose: () => void;
  onAcceptChanges: () => void;
  onRejectChanges: () => void;
  changes: ShoppingListChange[];
  title?: string;
}

export const ShoppingListDiffModal: React.FC<ShoppingListDiffModalProps> = ({
  isVisible,
  onClose,
  onAcceptChanges,
  onRejectChanges,
  changes,
  title = 'Shopping List Changes',
}) => {
  const addedItems = changes.filter(change => change.type === 'added');
  const removedItems = changes.filter(change => change.type === 'removed');
  const modifiedItems = changes.filter(change => change.type === 'modified');

  const getChangeIcon = (type: 'added' | 'removed' | 'modified') => {
    switch (type) {
      case 'added':
        return { icon: '➕', color: '#28a745', bgColor: '#e8f5e8' };
      case 'removed':
        return { icon: '➖', color: '#dc3545', bgColor: '#fde8e8' };
      case 'modified':
        return { icon: '🔄', color: '#ffc107', bgColor: '#fff8e1' };
    }
  };

  const formatAmount = (amount: number, unit: string) => {
    const formattedAmount = amount % 1 === 0 ? amount.toString() : amount.toFixed(2);
    return `${formattedAmount} ${unit}`;
  };

  const renderChangeItem = (change: ShoppingListChange) => {
    const { icon, color, bgColor } = getChangeIcon(change.type);

    return (
      <View key={`${change.type}-${change.ingredientName}`} style={styles.changeItem}>
        <View style={[styles.changeIcon, { backgroundColor: bgColor }]}>
          <Text style={[styles.changeIconText, { color }]}>{icon}</Text>
        </View>
        
        <View style={styles.changeContent}>
          <Text style={styles.ingredientName}>{change.ingredientName}</Text>
          
          {change.type === 'added' && (
            <Text style={styles.changeDetails}>
              Add {formatAmount(change.newAmount!, change.unit)}
            </Text>
          )}
          
          {change.type === 'removed' && (
            <Text style={styles.changeDetails}>
              Remove {formatAmount(change.oldAmount!, change.unit)}
            </Text>
          )}
          
          {change.type === 'modified' && (
            <Text style={styles.changeDetails}>
              {formatAmount(change.oldAmount!, change.unit)} → {formatAmount(change.newAmount!, change.unit)}
            </Text>
          )}
        </View>
      </View>
    );
  };

  const renderChangeSection = (
    title: string,
    items: ShoppingListChange[],
    sectionColor: string,
    emptyMessage: string
  ) => {
    if (items.length === 0) return null;

    return (
      <View style={styles.changeSection}>
        <View style={[styles.sectionHeader, { backgroundColor: sectionColor }]}>
          <Text style={styles.sectionTitle}>{title}</Text>
          <Text style={styles.sectionCount}>{items.length}</Text>
        </View>
        
        <View style={styles.sectionContent}>
          {items.map(renderChangeItem)}
        </View>
      </View>
    );
  };

  if (changes.length === 0) {
    return null;
  }

  return (
    <Modal
      visible={isVisible}
      transparent
      animationType="slide"
      onRequestClose={onClose}
    >
      <View style={styles.modalOverlay}>
        <View style={styles.modalContent}>
          {/* Header */}
          <View style={styles.header}>
            <View style={styles.headerContent}>
              <Text style={styles.modalTitle}>{title}</Text>
              <Text style={styles.modalSubtitle}>
                Review changes to your shopping list
              </Text>
            </View>
            <TouchableOpacity
              style={styles.closeButton}
              onPress={onClose}
              activeOpacity={0.7}
            >
              <Text style={styles.closeButtonText}>✕</Text>
            </TouchableOpacity>
          </View>

          {/* Changes summary */}
          <View style={styles.summary}>
            <View style={styles.summaryItem}>
              <Text style={styles.summaryNumber}>{addedItems.length}</Text>
              <Text style={styles.summaryLabel}>Added</Text>
            </View>
            <View style={styles.summaryItem}>
              <Text style={styles.summaryNumber}>{modifiedItems.length}</Text>
              <Text style={styles.summaryLabel}>Modified</Text>
            </View>
            <View style={styles.summaryItem}>
              <Text style={styles.summaryNumber}>{removedItems.length}</Text>
              <Text style={styles.summaryLabel}>Removed</Text>
            </View>
          </View>

          {/* Changes list */}
          <ScrollView 
            style={styles.changesList}
            showsVerticalScrollIndicator={false}
          >
            {renderChangeSection(
              'Added Items',
              addedItems,
              '#e8f5e8',
              'No new items'
            )}
            
            {renderChangeSection(
              'Modified Items',
              modifiedItems,
              '#fff8e1',
              'No modified items'
            )}
            
            {renderChangeSection(
              'Removed Items',
              removedItems,
              '#fde8e8',
              'No removed items'
            )}
          </ScrollView>

          {/* Actions */}
          <View style={styles.actions}>
            <TouchableOpacity
              style={[styles.actionButton, styles.rejectButton]}
              onPress={onRejectChanges}
              activeOpacity={0.7}
            >
              <Text style={styles.rejectButtonText}>Keep Current List</Text>
            </TouchableOpacity>
            
            <TouchableOpacity
              style={[styles.actionButton, styles.acceptButton]}
              onPress={onAcceptChanges}
              activeOpacity={0.7}
            >
              <Text style={styles.acceptButtonText}>Update Shopping List</Text>
            </TouchableOpacity>
          </View>
        </View>
      </View>
    </Modal>
  );
};

const styles = StyleSheet.create({
  modalOverlay: {
    flex: 1,
    backgroundColor: 'rgba(0, 0, 0, 0.5)',
    justifyContent: 'flex-end',
  },
  modalContent: {
    backgroundColor: '#ffffff',
    borderTopLeftRadius: 20,
    borderTopRightRadius: 20,
    maxHeight: '85%',
    paddingBottom: Platform.OS === 'ios' ? 34 : 20,
  },
  header: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    paddingHorizontal: 20,
    paddingVertical: 16,
    borderBottomWidth: 1,
    borderBottomColor: '#f1f3f4',
  },
  headerContent: {
    flex: 1,
  },
  modalTitle: {
    fontSize: 20,
    fontWeight: '600',
    color: '#2d3436',
    marginBottom: 2,
  },
  modalSubtitle: {
    fontSize: 14,
    color: '#636e72',
  },
  closeButton: {
    width: 32,
    height: 32,
    borderRadius: 16,
    backgroundColor: '#f8f9fa',
    alignItems: 'center',
    justifyContent: 'center',
    marginLeft: 12,
  },
  closeButtonText: {
    fontSize: 16,
    color: '#6c757d',
    fontWeight: '600',
  },
  summary: {
    flexDirection: 'row',
    paddingHorizontal: 20,
    paddingVertical: 16,
    backgroundColor: '#f8f9fa',
    justifyContent: 'space-around',
  },
  summaryItem: {
    alignItems: 'center',
  },
  summaryNumber: {
    fontSize: 24,
    fontWeight: '700',
    color: '#2d3436',
  },
  summaryLabel: {
    fontSize: 12,
    color: '#636e72',
    textTransform: 'uppercase',
    fontWeight: '500',
    marginTop: 2,
  },
  changesList: {
    flex: 1,
    paddingHorizontal: 20,
  },
  changeSection: {
    marginBottom: 16,
  },
  sectionHeader: {
    paddingHorizontal: 12,
    paddingVertical: 8,
    borderRadius: 8,
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    marginBottom: 8,
  },
  sectionTitle: {
    fontSize: 14,
    fontWeight: '600',
    color: '#2d3436',
  },
  sectionCount: {
    fontSize: 12,
    fontWeight: '600',
    color: '#636e72',
    backgroundColor: '#ffffff',
    paddingHorizontal: 8,
    paddingVertical: 2,
    borderRadius: 10,
  },
  sectionContent: {
    paddingLeft: 8,
  },
  changeItem: {
    flexDirection: 'row',
    alignItems: 'center',
    paddingVertical: 8,
    paddingHorizontal: 12,
    backgroundColor: '#ffffff',
    borderRadius: 8,
    marginBottom: 4,
    borderWidth: 1,
    borderColor: '#f1f3f4',
  },
  changeIcon: {
    width: 32,
    height: 32,
    borderRadius: 16,
    alignItems: 'center',
    justifyContent: 'center',
    marginRight: 12,
  },
  changeIconText: {
    fontSize: 14,
    fontWeight: '600',
  },
  changeContent: {
    flex: 1,
  },
  ingredientName: {
    fontSize: 16,
    fontWeight: '500',
    color: '#2d3436',
    marginBottom: 2,
  },
  changeDetails: {
    fontSize: 14,
    color: '#636e72',
  },
  actions: {
    flexDirection: 'row',
    paddingHorizontal: 20,
    paddingTop: 16,
    gap: 12,
  },
  actionButton: {
    flex: 1,
    paddingVertical: 14,
    borderRadius: 12,
    alignItems: 'center',
  },
  rejectButton: {
    backgroundColor: '#f8f9fa',
    borderWidth: 1,
    borderColor: '#dee2e6',
  },
  acceptButton: {
    backgroundColor: '#007bff',
  },
  rejectButtonText: {
    color: '#6c757d',
    fontSize: 16,
    fontWeight: '600',
  },
  acceptButtonText: {
    color: '#ffffff',
    fontSize: 16,
    fontWeight: '600',
  },
});
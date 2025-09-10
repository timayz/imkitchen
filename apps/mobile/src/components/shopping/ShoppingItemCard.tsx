import React, { useState } from 'react';
import {
  View,
  Text,
  TouchableOpacity,
  StyleSheet,
  Alert,
  TextInput,
  Modal,
} from 'react-native';
import type { ShoppingItemCardProps } from '../../types/shopping';

export const ShoppingItemCard: React.FC<ShoppingItemCardProps> = ({
  item,
  onToggleCompleted,
  onShowRecipeSources,
  disabled = false,
}) => {
  const [isNotesModalVisible, setIsNotesModalVisible] = useState(false);
  const [notesText, setNotesText] = useState(item.notes || '');

  const formatAmount = (amount: number, unit: string) => {
    // Format amount with appropriate decimal places
    const formattedAmount = amount % 1 === 0 ? amount.toString() : amount.toFixed(2);
    return `${formattedAmount} ${unit}`;
  };

  const handleToggleCompleted = async () => {
    if (disabled) return;

    try {
      await onToggleCompleted(!item.isCompleted, item.notes);
    } catch (error) {
      Alert.alert(
        'Error',
        'Failed to update item. Please try again.',
        [{ text: 'OK' }]
      );
    }
  };

  const handleNotesPress = () => {
    setIsNotesModalVisible(true);
  };

  const handleSaveNotes = async () => {
    if (disabled) return;

    try {
      await onToggleCompleted(item.isCompleted, notesText.trim() || undefined);
      setIsNotesModalVisible(false);
    } catch (error) {
      Alert.alert(
        'Error',
        'Failed to save notes. Please try again.',
        [{ text: 'OK' }]
      );
    }
  };

  const handleCancelNotes = () => {
    setNotesText(item.notes || '');
    setIsNotesModalVisible(false);
  };

  const getCategoryIcon = (category: string) => {
    switch (category) {
      case 'produce':
        return '🥕';
      case 'dairy':
        return '🥛';
      case 'protein':
        return '🍗';
      case 'pantry':
        return '🏺';
      default:
        return '📦';
    }
  };

  return (
    <>
      <TouchableOpacity
        style={[
          styles.container,
          item.isCompleted && styles.completedContainer,
          disabled && styles.disabledContainer,
        ]}
        onPress={handleToggleCompleted}
        disabled={disabled}
        activeOpacity={0.7}
      >
        {/* Checkbox and main content */}
        <View style={styles.mainContent}>
          {/* Checkbox */}
          <View style={[
            styles.checkbox,
            item.isCompleted && styles.checkedCheckbox,
          ]}>
            {item.isCompleted && (
              <Text style={styles.checkmark}>✓</Text>
            )}
          </View>

          {/* Item details */}
          <View style={styles.itemDetails}>
            <View style={styles.itemHeader}>
              <Text style={[
                styles.itemName,
                item.isCompleted && styles.completedText,
              ]}>
                {item.ingredientName}
              </Text>
              <Text style={styles.categoryIcon}>
                {getCategoryIcon(item.category)}
              </Text>
            </View>

            <View style={styles.itemMeta}>
              <Text style={[
                styles.itemAmount,
                item.isCompleted && styles.completedText,
              ]}>
                {formatAmount(item.amount, item.unit)}
              </Text>
              
              {item.estimatedCost && (
                <Text style={styles.estimatedCost}>
                  ~${item.estimatedCost.toFixed(2)}
                </Text>
              )}
            </View>

            {/* Notes indicator */}
            {item.notes && (
              <Text style={[
                styles.notes,
                item.isCompleted && styles.completedText,
              ]} numberOfLines={1}>
                📝 {item.notes}
              </Text>
            )}
          </View>
        </View>

        {/* Action buttons */}
        <View style={styles.actions}>
          {/* Recipe sources button */}
          {item.recipeSources && item.recipeSources.length > 0 && (
            <TouchableOpacity
              style={styles.actionButton}
              onPress={(e) => {
                e.stopPropagation();
                onShowRecipeSources();
              }}
              disabled={disabled}
            >
              <Text style={styles.actionButtonText}>
                🍽️ {item.recipeSources.length}
              </Text>
            </TouchableOpacity>
          )}

          {/* Notes button */}
          <TouchableOpacity
            style={styles.actionButton}
            onPress={(e) => {
              e.stopPropagation();
              handleNotesPress();
            }}
            disabled={disabled}
          >
            <Text style={styles.actionButtonText}>
              {item.notes ? '📝' : '📝+'}
            </Text>
          </TouchableOpacity>
        </View>
      </TouchableOpacity>

      {/* Notes modal */}
      <Modal
        visible={isNotesModalVisible}
        transparent
        animationType="fade"
        onRequestClose={handleCancelNotes}
      >
        <View style={styles.modalOverlay}>
          <View style={styles.modalContent}>
            <Text style={styles.modalTitle}>
              Notes for {item.ingredientName}
            </Text>
            
            <TextInput
              style={styles.notesInput}
              value={notesText}
              onChangeText={setNotesText}
              placeholder="Add notes about this item..."
              multiline
              numberOfLines={4}
              maxLength={200}
              autoFocus
            />
            
            <View style={styles.modalActions}>
              <TouchableOpacity
                style={[styles.modalButton, styles.cancelButton]}
                onPress={handleCancelNotes}
              >
                <Text style={styles.cancelButtonText}>Cancel</Text>
              </TouchableOpacity>
              
              <TouchableOpacity
                style={[styles.modalButton, styles.saveButton]}
                onPress={handleSaveNotes}
              >
                <Text style={styles.saveButtonText}>Save</Text>
              </TouchableOpacity>
            </View>
          </View>
        </View>
      </Modal>
    </>
  );
};

const styles = StyleSheet.create({
  container: {
    backgroundColor: '#ffffff',
    paddingHorizontal: 16,
    paddingVertical: 12,
    borderBottomWidth: 1,
    borderBottomColor: '#f1f3f4',
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
  },
  completedContainer: {
    backgroundColor: '#f8f9fa',
    opacity: 0.7,
  },
  disabledContainer: {
    opacity: 0.5,
  },
  mainContent: {
    flexDirection: 'row',
    alignItems: 'center',
    flex: 1,
  },
  checkbox: {
    width: 24,
    height: 24,
    borderRadius: 12,
    borderWidth: 2,
    borderColor: '#dee2e6',
    backgroundColor: '#ffffff',
    alignItems: 'center',
    justifyContent: 'center',
    marginRight: 12,
  },
  checkedCheckbox: {
    backgroundColor: '#28a745',
    borderColor: '#28a745',
  },
  checkmark: {
    color: '#ffffff',
    fontSize: 14,
    fontWeight: '600',
  },
  itemDetails: {
    flex: 1,
  },
  itemHeader: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
  },
  itemName: {
    fontSize: 16,
    fontWeight: '500',
    color: '#2d3436',
    flex: 1,
  },
  categoryIcon: {
    fontSize: 16,
    marginLeft: 8,
  },
  itemMeta: {
    flexDirection: 'row',
    alignItems: 'center',
    marginTop: 2,
  },
  itemAmount: {
    fontSize: 14,
    color: '#636e72',
    marginRight: 12,
  },
  estimatedCost: {
    fontSize: 12,
    color: '#28a745',
    fontWeight: '500',
  },
  notes: {
    fontSize: 12,
    color: '#6c757d',
    marginTop: 4,
    fontStyle: 'italic',
  },
  completedText: {
    textDecorationLine: 'line-through',
    color: '#adb5bd',
  },
  actions: {
    flexDirection: 'row',
    alignItems: 'center',
    marginLeft: 8,
  },
  actionButton: {
    paddingHorizontal: 8,
    paddingVertical: 4,
    backgroundColor: '#f8f9fa',
    borderRadius: 12,
    marginLeft: 4,
  },
  actionButtonText: {
    fontSize: 12,
    color: '#495057',
  },
  modalOverlay: {
    flex: 1,
    backgroundColor: 'rgba(0, 0, 0, 0.5)',
    justifyContent: 'center',
    alignItems: 'center',
    padding: 20,
  },
  modalContent: {
    backgroundColor: '#ffffff',
    borderRadius: 12,
    padding: 20,
    width: '100%',
    maxWidth: 400,
  },
  modalTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#2d3436',
    marginBottom: 16,
    textAlign: 'center',
  },
  notesInput: {
    borderWidth: 1,
    borderColor: '#dee2e6',
    borderRadius: 8,
    padding: 12,
    fontSize: 16,
    color: '#2d3436',
    textAlignVertical: 'top',
    marginBottom: 16,
    minHeight: 80,
  },
  modalActions: {
    flexDirection: 'row',
    justifyContent: 'space-between',
  },
  modalButton: {
    flex: 1,
    paddingVertical: 12,
    borderRadius: 8,
    alignItems: 'center',
  },
  cancelButton: {
    backgroundColor: '#f8f9fa',
    marginRight: 8,
  },
  saveButton: {
    backgroundColor: '#007bff',
    marginLeft: 8,
  },
  cancelButtonText: {
    color: '#6c757d',
    fontSize: 16,
    fontWeight: '500',
  },
  saveButtonText: {
    color: '#ffffff',
    fontSize: 16,
    fontWeight: '500',
  },
});
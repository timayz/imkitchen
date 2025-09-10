import React, { useState } from 'react';
import {
  View,
  Text,
  TouchableOpacity,
  StyleSheet,
  Alert,
  Modal,
  Platform,
  Share,
} from 'react-native';
import type { ShoppingListExportButtonProps, ShoppingListExportOptions } from '../../types/shopping';
import { useShoppingStore } from '../../store/shopping_store';
import { shoppingService } from '../../services/shopping_service';

export const ShoppingListExportButton: React.FC<ShoppingListExportButtonProps> = ({
  listId,
  disabled = false,
}) => {
  const [isExportModalVisible, setIsExportModalVisible] = useState(false);
  const [isExporting, setIsExporting] = useState(false);
  const { exportShoppingList } = useShoppingStore();

  const exportOptions: Array<{
    format: 'json' | 'csv' | 'txt';
    label: string;
    description: string;
    icon: string;
  }> = [
    {
      format: 'txt',
      label: 'Text File',
      description: 'Simple text format for sharing',
      icon: '📄',
    },
    {
      format: 'csv',
      label: 'Spreadsheet (CSV)',
      description: 'For grocery apps and budgeting',
      icon: '📊',
    },
    {
      format: 'json',
      label: 'Data Export (JSON)',
      description: 'Complete data with recipe sources',
      icon: '💾',
    },
  ];

  const handleExport = async (format: 'json' | 'csv' | 'txt', includeRecipeSources = false) => {
    setIsExporting(true);
    
    try {
      const options: ShoppingListExportOptions = {
        format,
        includeRecipeSources,
      };

      const blob = await exportShoppingList(listId, options);
      
      // Get filename with timestamp
      const timestamp = new Date().toISOString().slice(0, 10);
      const filename = `shopping-list-${timestamp}.${format}`;

      if (Platform.OS === 'web') {
        // Web: Download file
        shoppingService.downloadBlob(blob, filename);
      } else {
        // Mobile: Share file
        const fileReader = new FileReader();
        fileReader.onload = async () => {
          const base64 = fileReader.result as string;
          
          try {
            await Share.share({
              title: 'Shopping List',
              message: `Here's your shopping list exported as ${format.toUpperCase()}`,
              url: base64,
            });
          } catch (shareError) {
            Alert.alert(
              'Share Failed',
              'Could not share the shopping list. Please try again.',
              [{ text: 'OK' }]
            );
          }
        };
        fileReader.readAsDataURL(blob);
      }

      setIsExportModalVisible(false);
      
      Alert.alert(
        'Export Successful',
        `Shopping list exported as ${format.toUpperCase()}`,
        [{ text: 'OK' }]
      );

    } catch (error) {
      Alert.alert(
        'Export Failed',
        'Failed to export shopping list. Please try again.',
        [{ text: 'OK' }]
      );
    } finally {
      setIsExporting(false);
    }
  };

  const handleExportPress = () => {
    if (disabled) return;
    setIsExportModalVisible(true);
  };

  const handleCloseModal = () => {
    if (isExporting) return;
    setIsExportModalVisible(false);
  };

  return (
    <>
      <TouchableOpacity
        style={[
          styles.exportButton,
          disabled && styles.disabledButton,
        ]}
        onPress={handleExportPress}
        disabled={disabled}
        activeOpacity={0.7}
      >
        <Text style={[
          styles.exportButtonText,
          disabled && styles.disabledButtonText,
        ]}>
          📤 Export
        </Text>
      </TouchableOpacity>

      {/* Export options modal */}
      <Modal
        visible={isExportModalVisible}
        transparent
        animationType="slide"
        onRequestClose={handleCloseModal}
      >
        <View style={styles.modalOverlay}>
          <View style={styles.modalContent}>
            <View style={styles.modalHeader}>
              <Text style={styles.modalTitle}>Export Shopping List</Text>
              <Text style={styles.modalSubtitle}>
                Choose your preferred export format
              </Text>
            </View>

            <View style={styles.exportOptions}>
              {exportOptions.map((option) => (
                <View key={option.format} style={styles.exportOptionSection}>
                  <TouchableOpacity
                    style={[
                      styles.exportOption,
                      isExporting && styles.disabledOption,
                    ]}
                    onPress={() => handleExport(option.format, false)}
                    disabled={isExporting}
                  >
                    <View style={styles.optionLeft}>
                      <Text style={styles.optionIcon}>{option.icon}</Text>
                      <View style={styles.optionText}>
                        <Text style={styles.optionLabel}>{option.label}</Text>
                        <Text style={styles.optionDescription}>
                          {option.description}
                        </Text>
                      </View>
                    </View>
                    <Text style={styles.optionArrow}>→</Text>
                  </TouchableOpacity>

                  {/* Option to include recipe sources for JSON/CSV */}
                  {(option.format === 'json' || option.format === 'csv') && (
                    <TouchableOpacity
                      style={[
                        styles.exportOptionVariant,
                        isExporting && styles.disabledOption,
                      ]}
                      onPress={() => handleExport(option.format, true)}
                      disabled={isExporting}
                    >
                      <View style={styles.variantLeft}>
                        <Text style={styles.variantIcon}>📄 + 🍽️</Text>
                        <Text style={styles.variantLabel}>
                          With recipe sources
                        </Text>
                      </View>
                      <Text style={styles.optionArrow}>→</Text>
                    </TouchableOpacity>
                  )}
                </View>
              ))}
            </View>

            <View style={styles.modalActions}>
              <TouchableOpacity
                style={[
                  styles.modalButton,
                  styles.cancelButton,
                  isExporting && styles.disabledButton,
                ]}
                onPress={handleCloseModal}
                disabled={isExporting}
              >
                <Text style={[
                  styles.cancelButtonText,
                  isExporting && styles.disabledButtonText,
                ]}>
                  {isExporting ? 'Exporting...' : 'Cancel'}
                </Text>
              </TouchableOpacity>
            </View>
          </View>
        </View>
      </Modal>
    </>
  );
};

const styles = StyleSheet.create({
  exportButton: {
    backgroundColor: '#007bff',
    paddingHorizontal: 12,
    paddingVertical: 8,
    borderRadius: 8,
    alignItems: 'center',
    justifyContent: 'center',
  },
  disabledButton: {
    backgroundColor: '#e9ecef',
  },
  exportButtonText: {
    color: '#ffffff',
    fontSize: 14,
    fontWeight: '500',
  },
  disabledButtonText: {
    color: '#adb5bd',
  },
  modalOverlay: {
    flex: 1,
    backgroundColor: 'rgba(0, 0, 0, 0.5)',
    justifyContent: 'flex-end',
  },
  modalContent: {
    backgroundColor: '#ffffff',
    borderTopLeftRadius: 20,
    borderTopRightRadius: 20,
    paddingTop: 20,
    paddingBottom: Platform.OS === 'ios' ? 34 : 20,
    maxHeight: '80%',
  },
  modalHeader: {
    paddingHorizontal: 20,
    paddingBottom: 20,
    borderBottomWidth: 1,
    borderBottomColor: '#f1f3f4',
  },
  modalTitle: {
    fontSize: 20,
    fontWeight: '600',
    color: '#2d3436',
    textAlign: 'center',
    marginBottom: 4,
  },
  modalSubtitle: {
    fontSize: 14,
    color: '#636e72',
    textAlign: 'center',
  },
  exportOptions: {
    paddingHorizontal: 20,
    paddingVertical: 20,
  },
  exportOptionSection: {
    marginBottom: 16,
  },
  exportOption: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    paddingVertical: 16,
    paddingHorizontal: 16,
    backgroundColor: '#f8f9fa',
    borderRadius: 12,
    marginBottom: 8,
  },
  exportOptionVariant: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    paddingVertical: 12,
    paddingHorizontal: 16,
    backgroundColor: '#e3f2fd',
    borderRadius: 8,
    marginLeft: 16,
  },
  disabledOption: {
    opacity: 0.5,
  },
  optionLeft: {
    flexDirection: 'row',
    alignItems: 'center',
    flex: 1,
  },
  optionIcon: {
    fontSize: 24,
    marginRight: 12,
  },
  optionText: {
    flex: 1,
  },
  optionLabel: {
    fontSize: 16,
    fontWeight: '500',
    color: '#2d3436',
    marginBottom: 2,
  },
  optionDescription: {
    fontSize: 12,
    color: '#636e72',
  },
  optionArrow: {
    fontSize: 16,
    color: '#636e72',
  },
  variantLeft: {
    flexDirection: 'row',
    alignItems: 'center',
    flex: 1,
  },
  variantIcon: {
    fontSize: 16,
    marginRight: 12,
  },
  variantLabel: {
    fontSize: 14,
    color: '#495057',
  },
  modalActions: {
    paddingHorizontal: 20,
    paddingTop: 10,
    borderTopWidth: 1,
    borderTopColor: '#f1f3f4',
  },
  modalButton: {
    paddingVertical: 12,
    borderRadius: 8,
    alignItems: 'center',
  },
  cancelButton: {
    backgroundColor: '#f8f9fa',
  },
  cancelButtonText: {
    color: '#6c757d',
    fontSize: 16,
    fontWeight: '500',
  },
});
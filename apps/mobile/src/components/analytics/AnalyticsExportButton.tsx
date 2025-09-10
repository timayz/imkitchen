import React, { useState } from 'react';
import { View, Text, StyleSheet, TouchableOpacity, Modal, Alert, ActivityIndicator } from 'react-native';
import { analyticsService } from '../../services/analytics_service';
import type { RotationAnalytics, AnalyticsExportOptions } from '../../types/analytics';

interface AnalyticsExportButtonProps {
  analytics: RotationAnalytics;
}

export const AnalyticsExportButton: React.FC<AnalyticsExportButtonProps> = ({
  analytics,
}) => {
  const [isModalVisible, setIsModalVisible] = useState(false);
  const [isExporting, setIsExporting] = useState(false);
  const [selectedFormat, setSelectedFormat] = useState<'json' | 'csv'>('json');
  const [includeLogs, setIncludeLogs] = useState(false);

  const handleExport = async () => {
    if (isExporting) return;
    
    setIsExporting(true);
    
    try {
      const options: AnalyticsExportOptions = {
        format: selectedFormat,
        includeDebugLogs: includeLogs,
        dateRange: {
          startDate: new Date(Date.now() - analytics.weeksAnalyzed * 7 * 24 * 60 * 60 * 1000).toISOString().split('T')[0],
          endDate: new Date().toISOString().split('T')[0],
        },
      };

      const blob = await analyticsService.exportRotationData(options);
      
      // For React Native, we need to handle file downloads differently
      // This would typically involve a file system operation or sharing
      Alert.alert(
        'Export Ready',
        `Your rotation data has been exported as ${selectedFormat.toUpperCase()}. ${blob.size} bytes ready for download.`,
        [
          { text: 'Share', onPress: () => handleShare(blob) },
          { text: 'Save', onPress: () => handleSave(blob) },
          { text: 'OK' }
        ]
      );
      
      setIsModalVisible(false);
    } catch (error) {
      console.error('Export failed:', error);
      Alert.alert(
        'Export Failed',
        'Failed to export rotation data. Please try again.',
        [{ text: 'OK' }]
      );
    } finally {
      setIsExporting(false);
    }
  };

  const handleShare = async (blob: Blob) => {
    // This would integrate with React Native's Share API
    console.log('Sharing export data:', blob.size);
    // Implementation would use react-native-share or similar
  };

  const handleSave = async (blob: Blob) => {
    // This would integrate with React Native's file system
    console.log('Saving export data:', blob.size);
    // Implementation would use react-native-fs or similar
  };

  return (
    <>
      <TouchableOpacity style={styles.exportButton} onPress={() => setIsModalVisible(true)}>
        <Text style={styles.exportButtonText}>📊 Export Analytics</Text>
      </TouchableOpacity>

      <Modal
        visible={isModalVisible}
        transparent
        animationType="slide"
        onRequestClose={() => setIsModalVisible(false)}
      >
        <View style={styles.modalOverlay}>
          <View style={styles.modalContent}>
            <View style={styles.modalHeader}>
              <Text style={styles.modalTitle}>Export Rotation Analytics</Text>
              <TouchableOpacity
                style={styles.closeButton}
                onPress={() => setIsModalVisible(false)}
              >
                <Text style={styles.closeButtonText}>×</Text>
              </TouchableOpacity>
            </View>

            <View style={styles.optionSection}>
              <Text style={styles.optionLabel}>Export Format</Text>
              <View style={styles.formatOptions}>
                <TouchableOpacity
                  style={[
                    styles.formatOption,
                    selectedFormat === 'json' && styles.formatOptionSelected
                  ]}
                  onPress={() => setSelectedFormat('json')}
                >
                  <Text style={[
                    styles.formatOptionText,
                    selectedFormat === 'json' && styles.formatOptionTextSelected
                  ]}>
                    JSON
                  </Text>
                  <Text style={styles.formatDescription}>
                    Structured data for analysis tools
                  </Text>
                </TouchableOpacity>

                <TouchableOpacity
                  style={[
                    styles.formatOption,
                    selectedFormat === 'csv' && styles.formatOptionSelected
                  ]}
                  onPress={() => setSelectedFormat('csv')}
                >
                  <Text style={[
                    styles.formatOptionText,
                    selectedFormat === 'csv' && styles.formatOptionTextSelected
                  ]}>
                    CSV
                  </Text>
                  <Text style={styles.formatDescription}>
                    Spreadsheet-compatible format
                  </Text>
                </TouchableOpacity>
              </View>
            </View>

            <View style={styles.optionSection}>
              <TouchableOpacity
                style={styles.checkboxOption}
                onPress={() => setIncludeLogs(!includeLogs)}
              >
                <View style={[styles.checkbox, includeLogs && styles.checkboxSelected]}>
                  {includeLogs && <Text style={styles.checkmark}>✓</Text>}
                </View>
                <View style={styles.checkboxContent}>
                  <Text style={styles.checkboxLabel}>Include Debug Logs</Text>
                  <Text style={styles.checkboxDescription}>
                    Add algorithm decision logs for detailed analysis
                  </Text>
                </View>
              </TouchableOpacity>
            </View>

            <View style={styles.exportPreview}>
              <Text style={styles.previewTitle}>Export Summary</Text>
              <Text style={styles.previewText}>
                • {analytics.weeksAnalyzed} weeks of rotation data
              </Text>
              <Text style={styles.previewText}>
                • Analytics from {new Date(analytics.calculatedAt).toLocaleDateString()}
              </Text>
              <Text style={styles.previewText}>
                • Format: {selectedFormat.toUpperCase()}
              </Text>
              {includeLogs && (
                <Text style={styles.previewText}>
                  • Debug logs included
                </Text>
              )}
            </View>

            <View style={styles.modalActions}>
              <TouchableOpacity
                style={styles.cancelButton}
                onPress={() => setIsModalVisible(false)}
                disabled={isExporting}
              >
                <Text style={styles.cancelButtonText}>Cancel</Text>
              </TouchableOpacity>

              <TouchableOpacity
                style={[styles.exportActionButton, isExporting && styles.exportActionButtonDisabled]}
                onPress={handleExport}
                disabled={isExporting}
              >
                {isExporting ? (
                  <ActivityIndicator size="small" color="#fff" />
                ) : (
                  <Text style={styles.exportActionButtonText}>Export Data</Text>
                )}
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
    backgroundColor: '#3498db',
    paddingVertical: 12,
    paddingHorizontal: 20,
    borderRadius: 8,
    alignItems: 'center',
  },
  exportButtonText: {
    color: '#fff',
    fontSize: 16,
    fontWeight: '600',
  },
  modalOverlay: {
    flex: 1,
    backgroundColor: 'rgba(0, 0, 0, 0.5)',
    justifyContent: 'flex-end',
  },
  modalContent: {
    backgroundColor: '#fff',
    borderTopLeftRadius: 20,
    borderTopRightRadius: 20,
    paddingBottom: 34, // Safe area padding
  },
  modalHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    padding: 20,
    borderBottomWidth: 1,
    borderBottomColor: '#e0e0e0',
  },
  modalTitle: {
    fontSize: 18,
    fontWeight: 'bold',
    color: '#2c3e50',
  },
  closeButton: {
    width: 32,
    height: 32,
    borderRadius: 16,
    backgroundColor: '#f8f9fa',
    justifyContent: 'center',
    alignItems: 'center',
  },
  closeButtonText: {
    fontSize: 20,
    color: '#6c757d',
  },
  optionSection: {
    padding: 20,
    borderBottomWidth: 1,
    borderBottomColor: '#f1f2f6',
  },
  optionLabel: {
    fontSize: 16,
    fontWeight: '600',
    color: '#2c3e50',
    marginBottom: 12,
  },
  formatOptions: {
    flexDirection: 'row',
    gap: 12,
  },
  formatOption: {
    flex: 1,
    padding: 16,
    borderRadius: 8,
    borderWidth: 1,
    borderColor: '#e0e0e0',
    alignItems: 'center',
  },
  formatOptionSelected: {
    borderColor: '#3498db',
    backgroundColor: '#f8fcff',
  },
  formatOptionText: {
    fontSize: 16,
    fontWeight: '600',
    color: '#6c757d',
    marginBottom: 4,
  },
  formatOptionTextSelected: {
    color: '#3498db',
  },
  formatDescription: {
    fontSize: 12,
    color: '#95a5a6',
    textAlign: 'center',
  },
  checkboxOption: {
    flexDirection: 'row',
    alignItems: 'flex-start',
  },
  checkbox: {
    width: 20,
    height: 20,
    borderRadius: 4,
    borderWidth: 2,
    borderColor: '#e0e0e0',
    marginRight: 12,
    marginTop: 2,
    justifyContent: 'center',
    alignItems: 'center',
  },
  checkboxSelected: {
    borderColor: '#3498db',
    backgroundColor: '#3498db',
  },
  checkmark: {
    color: '#fff',
    fontSize: 12,
    fontWeight: 'bold',
  },
  checkboxContent: {
    flex: 1,
  },
  checkboxLabel: {
    fontSize: 16,
    fontWeight: '600',
    color: '#2c3e50',
    marginBottom: 4,
  },
  checkboxDescription: {
    fontSize: 14,
    color: '#7f8c8d',
    lineHeight: 18,
  },
  exportPreview: {
    padding: 20,
    backgroundColor: '#f8f9fa',
  },
  previewTitle: {
    fontSize: 14,
    fontWeight: '600',
    color: '#2c3e50',
    marginBottom: 8,
  },
  previewText: {
    fontSize: 13,
    color: '#5a6c7d',
    marginBottom: 4,
  },
  modalActions: {
    flexDirection: 'row',
    padding: 20,
    gap: 12,
  },
  cancelButton: {
    flex: 1,
    paddingVertical: 12,
    borderRadius: 8,
    borderWidth: 1,
    borderColor: '#e0e0e0',
    alignItems: 'center',
  },
  cancelButtonText: {
    fontSize: 16,
    color: '#6c757d',
    fontWeight: '600',
  },
  exportActionButton: {
    flex: 2,
    paddingVertical: 12,
    borderRadius: 8,
    backgroundColor: '#3498db',
    alignItems: 'center',
  },
  exportActionButtonDisabled: {
    backgroundColor: '#95a5a6',
  },
  exportActionButtonText: {
    fontSize: 16,
    color: '#fff',
    fontWeight: '600',
  },
});
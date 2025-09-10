/**
 * Conflict Resolution Modal Component
 * 
 * Provides a side-by-side diff view for resolving sync conflicts
 * with user choice options (keep mine/keep theirs/merge manually).
 * 
 * Features:
 * - Side-by-side diff visualization
 * - Field-level conflict resolution
 * - User-friendly conflict descriptions
 * - Multiple resolution strategies
 * - Rollback capability
 * - Resolution preview
 */

import React, { useState, useEffect, useMemo } from 'react';
import {
  View,
  Text,
  Modal,
  TouchableOpacity,
  ScrollView,
  StyleSheet,
  Alert,
  ActivityIndicator
} from 'react-native';
import { ConflictData, ConflictResolutionResult, ResolutionType } from '../../services/conflict_resolution_service';
import { conflictResolutionService } from '../../services/conflict_resolution_service';

interface ConflictResolutionModalProps {
  visible: boolean;
  conflictData: ConflictData | null;
  onResolve: (result: ConflictResolutionResult) => void;
  onCancel: () => void;
}

interface DiffViewItem {
  fieldPath: string;
  localValue: any;
  remoteValue: any;
  baseValue?: any;
  isConflicted: boolean;
  userChoice?: 'local' | 'remote' | 'custom';
  customValue?: any;
}

const ConflictResolutionModal: React.FC<ConflictResolutionModalProps> = ({
  visible,
  conflictData,
  onResolve,
  onCancel
}) => {
  const [loading, setLoading] = useState(false);
  const [resolutionStrategy, setResolutionStrategy] = useState<ResolutionType>(ResolutionType.USER_GUIDED);
  const [fieldResolutions, setFieldResolutions] = useState<Map<string, any>>(new Map());
  const [previewData, setPreviewData] = useState<any>(null);

  // Generate diff view data
  const diffViewItems = useMemo((): DiffViewItem[] => {
    if (!conflictData) return [];

    return conflictData.conflictingFields.map(field => ({
      fieldPath: field.fieldPath,
      localValue: field.localValue,
      remoteValue: field.remoteValue,
      baseValue: field.baseValue,
      isConflicted: true,
      userChoice: fieldResolutions.get(field.fieldPath)?.choice,
      customValue: fieldResolutions.get(field.fieldPath)?.customValue
    }));
  }, [conflictData, fieldResolutions]);

  // Reset state when conflict data changes
  useEffect(() => {
    if (conflictData) {
      setFieldResolutions(new Map());
      setPreviewData(null);
      setResolutionStrategy(ResolutionType.USER_GUIDED);
    }
  }, [conflictData]);

  const handleFieldChoice = (fieldPath: string, choice: 'local' | 'remote' | 'custom', customValue?: any) => {
    const newResolutions = new Map(fieldResolutions);
    newResolutions.set(fieldPath, { choice, customValue });
    setFieldResolutions(newResolutions);
    
    // Generate preview
    generatePreview(newResolutions);
  };

  const generatePreview = (resolutions: Map<string, any>) => {
    if (!conflictData) return;

    const preview = { ...conflictData.localVersion };
    
    resolutions.forEach((resolution, fieldPath) => {
      const field = conflictData.conflictingFields.find(f => f.fieldPath === fieldPath);
      if (!field) return;

      switch (resolution.choice) {
        case 'local':
          setNestedValue(preview, fieldPath, field.localValue);
          break;
        case 'remote':
          setNestedValue(preview, fieldPath, field.remoteValue);
          break;
        case 'custom':
          setNestedValue(preview, fieldPath, resolution.customValue);
          break;
      }
    });

    setPreviewData(preview);
  };

  const setNestedValue = (obj: any, path: string, value: any) => {
    const keys = path.split('.');
    const lastKey = keys.pop()!;
    const target = keys.reduce((current, key) => current[key], obj);
    target[lastKey] = value;
  };

  const handleAutoResolve = async (strategy: ResolutionType) => {
    if (!conflictData) return;

    setLoading(true);
    try {
      const result = await conflictResolutionService.resolveConflict(conflictData, strategy);
      onResolve(result);
    } catch (error) {
      console.error('[ConflictResolutionModal] Auto-resolve failed:', error);
      Alert.alert('Resolution Failed', 'Could not automatically resolve the conflict. Please try manual resolution.');
    } finally {
      setLoading(false);
    }
  };

  const handleManualResolve = async () => {
    if (!conflictData || fieldResolutions.size === 0) {
      Alert.alert('Incomplete Resolution', 'Please make choices for all conflicted fields.');
      return;
    }

    setLoading(true);
    try {
      // Build resolution data based on user choices
      const resolutionData = { ...conflictData.localVersion };
      
      fieldResolutions.forEach((resolution, fieldPath) => {
        const field = conflictData.conflictingFields.find(f => f.fieldPath === fieldPath);
        if (!field) return;

        let valueToUse;
        switch (resolution.choice) {
          case 'local':
            valueToUse = field.localValue;
            break;
          case 'remote':
            valueToUse = field.remoteValue;
            break;
          case 'custom':
            valueToUse = resolution.customValue;
            break;
        }
        
        setNestedValue(resolutionData, fieldPath, valueToUse);
      });

      const result: ConflictResolutionResult = {
        success: true,
        resolvedData: resolutionData,
        strategy: ResolutionType.USER_GUIDED,
        confidence: 100,
        fieldsResolved: Array.from(fieldResolutions.keys()),
        fieldsRequiringUserInput: [],
        resolutionSummary: `Manually resolved ${fieldResolutions.size} conflicted fields`
      };

      onResolve(result);
    } catch (error) {
      console.error('[ConflictResolutionModal] Manual resolve failed:', error);
      Alert.alert('Resolution Failed', 'Could not apply the manual resolution.');
    } finally {
      setLoading(false);
    }
  };

  const renderValue = (value: any, label: string, isSelected?: boolean) => {
    const displayValue = typeof value === 'object' ? JSON.stringify(value, null, 2) : String(value);
    
    return (
      <View style={[styles.valueContainer, isSelected && styles.selectedValue]}>
        <Text style={styles.valueLabel}>{label}</Text>
        <Text style={styles.valueText} numberOfLines={3}>
          {displayValue}
        </Text>
      </View>
    );
  };

  const renderFieldDiff = (item: DiffViewItem) => {
    const hasUserChoice = item.userChoice !== undefined;
    
    return (
      <View key={item.fieldPath} style={styles.fieldDiffContainer}>
        <Text style={styles.fieldPath}>{item.fieldPath}</Text>
        
        <View style={styles.diffRow}>
          {/* Local Version */}
          <TouchableOpacity
            style={[styles.diffColumn, item.userChoice === 'local' && styles.selectedColumn]}
            onPress={() => handleFieldChoice(item.fieldPath, 'local')}
          >
            {renderValue(item.localValue, 'Your Version', item.userChoice === 'local')}
          </TouchableOpacity>

          {/* Remote Version */}
          <TouchableOpacity
            style={[styles.diffColumn, item.userChoice === 'remote' && styles.selectedColumn]}
            onPress={() => handleFieldChoice(item.fieldPath, 'remote')}
          >
            {renderValue(item.remoteValue, 'Server Version', item.userChoice === 'remote')}
          </TouchableOpacity>
        </View>

        {/* Base Version (if available) */}
        {item.baseValue !== undefined && (
          <View style={styles.baseVersionContainer}>
            <Text style={styles.baseVersionLabel}>Original Version:</Text>
            <Text style={styles.baseVersionText}>
              {typeof item.baseValue === 'object' 
                ? JSON.stringify(item.baseValue, null, 2) 
                : String(item.baseValue)}
            </Text>
          </View>
        )}

        {/* Custom Input Option */}
        <TouchableOpacity
          style={styles.customOptionButton}
          onPress={() => {
            // For simplicity, this would show a text input modal
            // In a real implementation, this would be more sophisticated
            Alert.prompt(
              'Custom Value',
              `Enter custom value for ${item.fieldPath}:`,
              [
                { text: 'Cancel', style: 'cancel' },
                {
                  text: 'Save',
                  onPress: (text) => {
                    let customValue = text;
                    try {
                      // Try to parse as JSON if it looks like an object
                      if (text?.startsWith('{') || text?.startsWith('[')) {
                        customValue = JSON.parse(text);
                      }
                    } catch {
                      // Keep as string if not valid JSON
                    }
                    handleFieldChoice(item.fieldPath, 'custom', customValue);
                  }
                }
              ],
              'plain-text',
              String(item.localValue)
            );
          }}
        >
          <Text style={styles.customOptionText}>Enter Custom Value</Text>
        </TouchableOpacity>
      </View>
    );
  };

  if (!conflictData) return null;

  return (
    <Modal
      visible={visible}
      animationType="slide"
      presentationStyle="pageSheet"
      onRequestClose={onCancel}
    >
      <View style={styles.container}>
        <View style={styles.header}>
          <Text style={styles.title}>Resolve Conflict</Text>
          <Text style={styles.subtitle}>
            {conflictData.itemType} - {conflictData.itemId}
          </Text>
          <TouchableOpacity style={styles.closeButton} onPress={onCancel}>
            <Text style={styles.closeButtonText}>✕</Text>
          </TouchableOpacity>
        </View>

        <ScrollView style={styles.content}>
          {/* Quick Resolution Buttons */}
          <View style={styles.quickResolutionContainer}>
            <Text style={styles.sectionTitle}>Quick Resolution:</Text>
            <View style={styles.quickButtonsRow}>
              <TouchableOpacity
                style={styles.quickButton}
                onPress={() => handleAutoResolve(ResolutionType.LOCAL_WINS)}
                disabled={loading}
              >
                <Text style={styles.quickButtonText}>Keep Mine</Text>
              </TouchableOpacity>
              
              <TouchableOpacity
                style={styles.quickButton}
                onPress={() => handleAutoResolve(ResolutionType.REMOTE_WINS)}
                disabled={loading}
              >
                <Text style={styles.quickButtonText}>Keep Theirs</Text>
              </TouchableOpacity>
              
              <TouchableOpacity
                style={styles.quickButton}
                onPress={() => handleAutoResolve(ResolutionType.LAST_WRITE_WINS)}
                disabled={loading}
              >
                <Text style={styles.quickButtonText}>Most Recent</Text>
              </TouchableOpacity>
            </View>
          </View>

          {/* Field-by-Field Resolution */}
          <View style={styles.fieldResolutionContainer}>
            <Text style={styles.sectionTitle}>Field-by-Field Resolution:</Text>
            {diffViewItems.map(renderFieldDiff)}
          </View>

          {/* Preview */}
          {previewData && (
            <View style={styles.previewContainer}>
              <Text style={styles.sectionTitle}>Preview:</Text>
              <Text style={styles.previewText}>
                {JSON.stringify(previewData, null, 2)}
              </Text>
            </View>
          )}
        </ScrollView>

        <View style={styles.footer}>
          <TouchableOpacity
            style={styles.cancelButton}
            onPress={onCancel}
            disabled={loading}
          >
            <Text style={styles.cancelButtonText}>Cancel</Text>
          </TouchableOpacity>
          
          <TouchableOpacity
            style={[styles.resolveButton, (fieldResolutions.size === 0 || loading) && styles.disabledButton]}
            onPress={handleManualResolve}
            disabled={fieldResolutions.size === 0 || loading}
          >
            {loading ? (
              <ActivityIndicator color="#FFFFFF" />
            ) : (
              <Text style={styles.resolveButtonText}>Apply Resolution</Text>
            )}
          </TouchableOpacity>
        </View>
      </View>
    </Modal>
  );
};

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#FFFFFF'
  },
  header: {
    padding: 16,
    borderBottomWidth: 1,
    borderBottomColor: '#E0E0E0',
    position: 'relative'
  },
  title: {
    fontSize: 20,
    fontWeight: 'bold',
    color: '#333333'
  },
  subtitle: {
    fontSize: 14,
    color: '#666666',
    marginTop: 4
  },
  closeButton: {
    position: 'absolute',
    right: 16,
    top: 16,
    width: 32,
    height: 32,
    borderRadius: 16,
    backgroundColor: '#F5F5F5',
    justifyContent: 'center',
    alignItems: 'center'
  },
  closeButtonText: {
    fontSize: 18,
    color: '#666666'
  },
  content: {
    flex: 1,
    padding: 16
  },
  quickResolutionContainer: {
    marginBottom: 24
  },
  sectionTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#333333',
    marginBottom: 12
  },
  quickButtonsRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    gap: 12
  },
  quickButton: {
    flex: 1,
    padding: 12,
    backgroundColor: '#007AFF',
    borderRadius: 8,
    alignItems: 'center'
  },
  quickButtonText: {
    color: '#FFFFFF',
    fontWeight: '600'
  },
  fieldResolutionContainer: {
    marginBottom: 24
  },
  fieldDiffContainer: {
    marginBottom: 20,
    padding: 16,
    backgroundColor: '#F8F9FA',
    borderRadius: 8,
    borderWidth: 1,
    borderColor: '#E0E0E0'
  },
  fieldPath: {
    fontSize: 16,
    fontWeight: '600',
    color: '#333333',
    marginBottom: 12
  },
  diffRow: {
    flexDirection: 'row',
    gap: 12,
    marginBottom: 12
  },
  diffColumn: {
    flex: 1,
    borderWidth: 1,
    borderColor: '#D0D0D0',
    borderRadius: 6,
    padding: 8
  },
  selectedColumn: {
    borderColor: '#007AFF',
    borderWidth: 2,
    backgroundColor: '#F0F8FF'
  },
  valueContainer: {
    minHeight: 60
  },
  selectedValue: {
    backgroundColor: 'rgba(0, 122, 255, 0.1)'
  },
  valueLabel: {
    fontSize: 12,
    fontWeight: '600',
    color: '#666666',
    marginBottom: 4
  },
  valueText: {
    fontSize: 14,
    color: '#333333'
  },
  baseVersionContainer: {
    marginTop: 8,
    padding: 8,
    backgroundColor: '#FFFBF0',
    borderRadius: 4
  },
  baseVersionLabel: {
    fontSize: 12,
    fontWeight: '600',
    color: '#B8860B',
    marginBottom: 4
  },
  baseVersionText: {
    fontSize: 12,
    color: '#8B7355'
  },
  customOptionButton: {
    marginTop: 8,
    padding: 8,
    backgroundColor: '#F5F5F5',
    borderRadius: 4,
    alignItems: 'center'
  },
  customOptionText: {
    fontSize: 14,
    color: '#007AFF',
    fontWeight: '500'
  },
  previewContainer: {
    marginBottom: 24,
    padding: 16,
    backgroundColor: '#F0F8F0',
    borderRadius: 8
  },
  previewText: {
    fontSize: 12,
    color: '#2D5A2D',
    fontFamily: 'monospace'
  },
  footer: {
    flexDirection: 'row',
    padding: 16,
    borderTopWidth: 1,
    borderTopColor: '#E0E0E0',
    gap: 12
  },
  cancelButton: {
    flex: 1,
    padding: 16,
    backgroundColor: '#F5F5F5',
    borderRadius: 8,
    alignItems: 'center'
  },
  cancelButtonText: {
    fontSize: 16,
    color: '#666666',
    fontWeight: '600'
  },
  resolveButton: {
    flex: 2,
    padding: 16,
    backgroundColor: '#28A745',
    borderRadius: 8,
    alignItems: 'center'
  },
  disabledButton: {
    backgroundColor: '#CCCCCC'
  },
  resolveButtonText: {
    fontSize: 16,
    color: '#FFFFFF',
    fontWeight: '600'
  }
});

export default ConflictResolutionModal;
import React, { useState } from 'react';
import { View, Text, StyleSheet, TouchableOpacity, Modal, Alert, ActivityIndicator } from 'react-native';
import { analyticsService } from '../../services/analytics_service';

interface RotationResetButtonProps {
  onResetComplete: () => void;
}

export const RotationResetButton: React.FC<RotationResetButtonProps> = ({
  onResetComplete,
}) => {
  const [isModalVisible, setIsModalVisible] = useState(false);
  const [isResetting, setIsResetting] = useState(false);
  const [preservePatterns, setPreservePatterns] = useState(true);
  const [preserveFavorites, setPreserveFavorites] = useState(true);
  const [showImpactPreview, setShowImpactPreview] = useState(false);

  const handleReset = async () => {
    if (isResetting) return;

    // Show confirmation alert
    Alert.alert(
      'Reset Rotation Cycle',
      'Are you sure you want to reset your rotation cycle? This action cannot be undone.',
      [
        { text: 'Cancel', style: 'cancel' },
        { 
          text: 'Reset', 
          style: 'destructive',
          onPress: performReset 
        }
      ]
    );
  };

  const performReset = async () => {
    setIsResetting(true);
    
    try {
      await analyticsService.resetRotationCycle({
        confirmReset: true,
        preservePatterns,
        preserveFavorites,
      });

      setIsModalVisible(false);
      
      Alert.alert(
        'Reset Complete',
        'Your rotation cycle has been reset successfully. Fresh meal planning starts now!',
        [{ text: 'OK', onPress: onResetComplete }]
      );
      
    } catch (error) {
      console.error('Reset failed:', error);
      Alert.alert(
        'Reset Failed',
        'Failed to reset rotation cycle. Please try again.',
        [{ text: 'OK' }]
      );
    } finally {
      setIsResetting(false);
    }
  };

  const toggleImpactPreview = () => {
    setShowImpactPreview(!showImpactPreview);
  };

  const getImpactDescription = () => {
    let impacts = [];
    
    impacts.push('• All rotation history will be cleared');
    impacts.push('• Week-to-week meal memory will reset');
    
    if (preservePatterns) {
      impacts.push('• Your weekly patterns will be kept');
    } else {
      impacts.push('• Weekly patterns will be reset to defaults');
    }
    
    if (preserveFavorites) {
      impacts.push('• Your favorite recipes will remain marked');
    } else {
      impacts.push('• All favorite markings will be removed');
    }
    
    impacts.push('• Analytics data will be archived');
    
    return impacts;
  };

  return (
    <>
      <TouchableOpacity style={styles.resetButton} onPress={() => setIsModalVisible(true)}>
        <Text style={styles.resetButtonText}>🔄 Reset Rotation</Text>
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
              <Text style={styles.modalTitle}>Reset Rotation Cycle</Text>
              <TouchableOpacity
                style={styles.closeButton}
                onPress={() => setIsModalVisible(false)}
              >
                <Text style={styles.closeButtonText}>×</Text>
              </TouchableOpacity>
            </View>

            <View style={styles.warningSection}>
              <View style={styles.warningIcon}>
                <Text style={styles.warningEmoji}>⚠️</Text>
              </View>
              <View style={styles.warningContent}>
                <Text style={styles.warningTitle}>This will clear your rotation history</Text>
                <Text style={styles.warningText}>
                  Resetting will give you a fresh start with meal planning, but your 
                  current rotation patterns will be lost.
                </Text>
              </View>
            </View>

            <View style={styles.optionSection}>
              <Text style={styles.optionLabel}>What to preserve?</Text>
              
              <TouchableOpacity
                style={styles.checkboxOption}
                onPress={() => setPreservePatterns(!preservePatterns)}
              >
                <View style={[styles.checkbox, preservePatterns && styles.checkboxSelected]}>
                  {preservePatterns && <Text style={styles.checkmark}>✓</Text>}
                </View>
                <View style={styles.checkboxContent}>
                  <Text style={styles.checkboxLabel}>Keep Weekly Patterns</Text>
                  <Text style={styles.checkboxDescription}>
                    Maintain your cooking schedule and availability settings
                  </Text>
                </View>
              </TouchableOpacity>

              <TouchableOpacity
                style={styles.checkboxOption}
                onPress={() => setPreserveFavorites(!preserveFavorites)}
              >
                <View style={[styles.checkbox, preserveFavorites && styles.checkboxSelected]}>
                  {preserveFavorites && <Text style={styles.checkmark}>✓</Text>}
                </View>
                <View style={styles.checkboxContent}>
                  <Text style={styles.checkboxLabel}>Keep Favorite Recipes</Text>
                  <Text style={styles.checkboxDescription}>
                    Maintain your recipe favorites and preference weights
                  </Text>
                </View>
              </TouchableOpacity>
            </View>

            <View style={styles.impactSection}>
              <TouchableOpacity
                style={styles.impactToggle}
                onPress={toggleImpactPreview}
              >
                <Text style={styles.impactToggleText}>
                  {showImpactPreview ? '▼' : '▶'} What will happen?
                </Text>
              </TouchableOpacity>
              
              {showImpactPreview && (
                <View style={styles.impactPreview}>
                  {getImpactDescription().map((impact, index) => (
                    <Text key={index} style={styles.impactItem}>
                      {impact}
                    </Text>
                  ))}
                </View>
              )}
            </View>

            <View style={styles.modalActions}>
              <TouchableOpacity
                style={styles.cancelButton}
                onPress={() => setIsModalVisible(false)}
                disabled={isResetting}
              >
                <Text style={styles.cancelButtonText}>Cancel</Text>
              </TouchableOpacity>

              <TouchableOpacity
                style={[styles.resetActionButton, isResetting && styles.resetActionButtonDisabled]}
                onPress={handleReset}
                disabled={isResetting}
              >
                {isResetting ? (
                  <ActivityIndicator size="small" color="#fff" />
                ) : (
                  <Text style={styles.resetActionButtonText}>Reset Cycle</Text>
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
  resetButton: {
    backgroundColor: '#e74c3c',
    paddingVertical: 12,
    paddingHorizontal: 20,
    borderRadius: 8,
    alignItems: 'center',
    marginTop: 8,
  },
  resetButtonText: {
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
  warningSection: {
    flexDirection: 'row',
    padding: 20,
    backgroundColor: '#fff3cd',
    borderBottomWidth: 1,
    borderBottomColor: '#f1f2f6',
  },
  warningIcon: {
    marginRight: 12,
  },
  warningEmoji: {
    fontSize: 24,
  },
  warningContent: {
    flex: 1,
  },
  warningTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#856404',
    marginBottom: 4,
  },
  warningText: {
    fontSize: 14,
    color: '#856404',
    lineHeight: 18,
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
    marginBottom: 16,
  },
  checkboxOption: {
    flexDirection: 'row',
    alignItems: 'flex-start',
    marginBottom: 16,
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
  impactSection: {
    padding: 20,
    borderBottomWidth: 1,
    borderBottomColor: '#f1f2f6',
  },
  impactToggle: {
    paddingVertical: 8,
  },
  impactToggleText: {
    fontSize: 14,
    fontWeight: '600',
    color: '#3498db',
  },
  impactPreview: {
    marginTop: 12,
    paddingTop: 12,
    borderTopWidth: 1,
    borderTopColor: '#f1f2f6',
  },
  impactItem: {
    fontSize: 13,
    color: '#5a6c7d',
    marginBottom: 6,
    lineHeight: 18,
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
  resetActionButton: {
    flex: 2,
    paddingVertical: 12,
    borderRadius: 8,
    backgroundColor: '#e74c3c',
    alignItems: 'center',
  },
  resetActionButtonDisabled: {
    backgroundColor: '#95a5a6',
  },
  resetActionButtonText: {
    fontSize: 16,
    color: '#fff',
    fontWeight: '600',
  },
});
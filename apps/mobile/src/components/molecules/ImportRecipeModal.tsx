import React, { useState, useCallback } from 'react';
import {
  View,
  Text,
  Modal,
  TextInput,
  TouchableOpacity,
  StyleSheet,
  Alert,
  ActivityIndicator,
  ScrollView,
  Switch,
} from 'react-native';
import type { 
  CommunityRecipe, 
  RecipeImportRequest,
  RecipeImportResponse,
  ImportConflict
} from '@imkitchen/shared-types';

interface ImportRecipeModalProps {
  recipe: CommunityRecipe | null;
  visible: boolean;
  onClose: () => void;
  onImportSuccess: (response: RecipeImportResponse) => void;
  onImportError: (error: string) => void;
  conflict?: ImportConflict | null;
  isLoading?: boolean;
}

export const ImportRecipeModal: React.FC<ImportRecipeModalProps> = ({
  recipe,
  visible,
  onClose,
  onImportSuccess,
  onImportError,
  conflict,
  isLoading = false,
}) => {
  const [customTitle, setCustomTitle] = useState('');
  const [notes, setNotes] = useState('');
  const [servingAdjustment, setServingAdjustment] = useState('');
  const [preserveAttribution, setPreserveAttribution] = useState(true);
  const [isSubmitting, setIsSubmitting] = useState(false);

  // Reset form when modal opens/closes
  React.useEffect(() => {
    if (visible && recipe) {
      setCustomTitle(recipe.title);
      setNotes('');
      setServingAdjustment(recipe.servings.toString());
      setPreserveAttribution(true);
    }
  }, [visible, recipe]);

  const handleImport = useCallback(async () => {
    if (!recipe) return;

    setIsSubmitting(true);

    try {
      const request: RecipeImportRequest = {
        communityRecipeId: recipe.id,
        preserveAttribution,
        customizations: {
          ...(customTitle !== recipe.title && { title: customTitle }),
          ...(notes.trim() && { notes: notes.trim() }),
          ...(servingAdjustment && parseInt(servingAdjustment) !== recipe.servings && {
            servingAdjustment: parseInt(servingAdjustment),
          }),
        },
      };

      // This would call the import service
      // For now, we'll simulate the response
      const response: RecipeImportResponse = {
        success: true,
        personalRecipeId: 'new-recipe-id',
        message: 'Recipe successfully imported to your collection',
        attribution: preserveAttribution ? {
          originalContributor: recipe.contributorName || 'Unknown',
          importDate: new Date(),
          communityMetrics: {
            totalImports: recipe.importCount + 1,
            averageRating: recipe.averageRating,
          },
        } : undefined,
      };

      onImportSuccess(response);
      onClose();
    } catch (error) {
      onImportError(error instanceof Error ? error.message : 'Failed to import recipe');
    } finally {
      setIsSubmitting(false);
    }
  }, [
    recipe,
    customTitle,
    notes,
    servingAdjustment,
    preserveAttribution,
    onImportSuccess,
    onImportError,
    onClose,
  ]);

  const handleConflictResolution = useCallback((resolution: string) => {
    switch (resolution) {
      case 'rename':
        setCustomTitle(`${recipe?.title} (Copy)`);
        break;
      case 'replace':
        Alert.alert(
          'Replace Existing Recipe',
          'This will replace your existing recipe with this community version. This action cannot be undone.',
          [
            { text: 'Cancel', style: 'cancel' },
            { text: 'Replace', style: 'destructive', onPress: handleImport },
          ]
        );
        return;
      case 'cancel':
        onClose();
        return;
      default:
        break;
    }
  }, [recipe?.title, handleImport, onClose]);

  const validateForm = () => {
    if (!customTitle.trim()) {
      Alert.alert('Error', 'Recipe title is required');
      return false;
    }
    if (servingAdjustment && (isNaN(parseInt(servingAdjustment)) || parseInt(servingAdjustment) <= 0)) {
      Alert.alert('Error', 'Serving adjustment must be a positive number');
      return false;
    }
    return true;
  };

  const handleSubmit = () => {
    if (!validateForm()) return;
    handleImport();
  };

  if (!recipe) return null;

  return (
    <Modal
      visible={visible}
      animationType="slide"
      presentationStyle="pageSheet"
      onRequestClose={onClose}
    >
      <View style={styles.container}>
        <View style={styles.header}>
          <Text style={styles.headerTitle}>Import Recipe</Text>
          <TouchableOpacity
            style={styles.closeButton}
            onPress={onClose}
            disabled={isSubmitting}
          >
            <Text style={styles.closeButtonText}>✕</Text>
          </TouchableOpacity>
        </View>

        <ScrollView style={styles.content} showsVerticalScrollIndicator={false}>
          {/* Conflict Resolution */}
          {conflict && (
            <View style={styles.conflictSection}>
              <Text style={styles.conflictTitle}>Import Conflict</Text>
              <Text style={styles.conflictMessage}>
                You already have a recipe called "{conflict.existingRecipeTitle}" 
                imported from this community recipe.
              </Text>
              <View style={styles.conflictOptions}>
                {conflict.resolution.options.map((option) => (
                  <TouchableOpacity
                    key={option}
                    style={[
                      styles.conflictButton,
                      option === conflict.resolution.recommended && styles.recommendedButton,
                    ]}
                    onPress={() => handleConflictResolution(option)}
                  >
                    <Text style={[
                      styles.conflictButtonText,
                      option === conflict.resolution.recommended && styles.recommendedButtonText,
                    ]}>
                      {option.charAt(0).toUpperCase() + option.slice(1)}
                      {option === conflict.resolution.recommended && ' (Recommended)'}
                    </Text>
                  </TouchableOpacity>
                ))}
              </View>
            </View>
          )}

          {/* Recipe Info */}
          <View style={styles.recipeInfo}>
            <Text style={styles.recipeTitle}>{recipe.title}</Text>
            {recipe.description && (
              <Text style={styles.recipeDescription}>{recipe.description}</Text>
            )}
            <View style={styles.recipeStats}>
              <Text style={styles.statText}>
                ⭐ {recipe.averageRating.toFixed(1)} ({recipe.totalRatings} ratings)
              </Text>
              <Text style={styles.statText}>
                📥 {recipe.importCount} imports
              </Text>
            </View>
          </View>

          {/* Customization Options */}
          <View style={styles.customizationSection}>
            <Text style={styles.sectionTitle}>Customize Import</Text>

            {/* Custom Title */}
            <View style={styles.inputGroup}>
              <Text style={styles.inputLabel}>Recipe Title</Text>
              <TextInput
                style={styles.textInput}
                value={customTitle}
                onChangeText={setCustomTitle}
                placeholder="Enter custom title"
                maxLength={255}
              />
            </View>

            {/* Notes */}
            <View style={styles.inputGroup}>
              <Text style={styles.inputLabel}>Personal Notes (Optional)</Text>
              <TextInput
                style={[styles.textInput, styles.notesInput]}
                value={notes}
                onChangeText={setNotes}
                placeholder="Add your personal notes about this recipe"
                multiline={true}
                numberOfLines={3}
                maxLength={1000}
              />
              <Text style={styles.charCount}>{notes.length}/1000</Text>
            </View>

            {/* Serving Adjustment */}
            <View style={styles.inputGroup}>
              <Text style={styles.inputLabel}>
                Servings (Original: {recipe.servings})
              </Text>
              <TextInput
                style={[styles.textInput, styles.numberInput]}
                value={servingAdjustment}
                onChangeText={setServingAdjustment}
                placeholder={recipe.servings.toString()}
                keyboardType="numeric"
                maxLength={2}
              />
              <Text style={styles.helpText}>
                Ingredient quantities will be adjusted automatically
              </Text>
            </View>
          </View>

          {/* Attribution Settings */}
          <View style={styles.attributionSection}>
            <Text style={styles.sectionTitle}>Attribution</Text>
            <View style={styles.switchRow}>
              <View style={styles.switchInfo}>
                <Text style={styles.switchLabel}>Preserve Attribution</Text>
                <Text style={styles.switchDescription}>
                  Keep track of the original contributor and community metrics
                </Text>
              </View>
              <Switch
                value={preserveAttribution}
                onValueChange={setPreserveAttribution}
                trackColor={{ false: '#d1d5db', true: '#3b82f6' }}
                thumbColor={preserveAttribution ? '#ffffff' : '#ffffff'}
              />
            </View>

            {preserveAttribution && recipe.contributorName && (
              <View style={styles.attributionPreview}>
                <Text style={styles.attributionText}>
                  Originally by: {recipe.contributorName}
                </Text>
                <Text style={styles.attributionText}>
                  Community rating: {recipe.averageRating.toFixed(1)}/5.0
                </Text>
              </View>
            )}
          </View>
        </ScrollView>

        {/* Action Buttons */}
        <View style={styles.actionButtons}>
          <TouchableOpacity
            style={styles.cancelButton}
            onPress={onClose}
            disabled={isSubmitting}
          >
            <Text style={styles.cancelButtonText}>Cancel</Text>
          </TouchableOpacity>

          <TouchableOpacity
            style={[styles.importButton, (isSubmitting || isLoading) && styles.disabledButton]}
            onPress={handleSubmit}
            disabled={isSubmitting || isLoading}
          >
            {isSubmitting || isLoading ? (
              <>
                <ActivityIndicator size="small" color="#fff" style={styles.buttonLoader} />
                <Text style={styles.importButtonText}>Importing...</Text>
              </>
            ) : (
              <Text style={styles.importButtonText}>Import Recipe</Text>
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
    backgroundColor: '#fff',
  },
  header: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    paddingHorizontal: 20,
    paddingTop: 16,
    paddingBottom: 12,
    borderBottomWidth: 1,
    borderBottomColor: '#f0f0f0',
  },
  headerTitle: {
    fontSize: 20,
    fontWeight: '700',
    color: '#333',
  },
  closeButton: {
    width: 32,
    height: 32,
    borderRadius: 16,
    backgroundColor: '#f0f0f0',
    justifyContent: 'center',
    alignItems: 'center',
  },
  closeButtonText: {
    fontSize: 18,
    color: '#666',
    fontWeight: '600',
  },
  content: {
    flex: 1,
    paddingHorizontal: 20,
  },
  conflictSection: {
    backgroundColor: '#fef3c7',
    borderColor: '#f59e0b',
    borderWidth: 1,
    borderRadius: 8,
    padding: 16,
    marginVertical: 16,
  },
  conflictTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#92400e',
    marginBottom: 8,
  },
  conflictMessage: {
    fontSize: 14,
    color: '#92400e',
    marginBottom: 12,
    lineHeight: 20,
  },
  conflictOptions: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: 8,
  },
  conflictButton: {
    paddingHorizontal: 12,
    paddingVertical: 6,
    borderRadius: 6,
    borderWidth: 1,
    borderColor: '#f59e0b',
    backgroundColor: '#fff',
  },
  recommendedButton: {
    backgroundColor: '#f59e0b',
  },
  conflictButtonText: {
    fontSize: 12,
    color: '#f59e0b',
    fontWeight: '600',
  },
  recommendedButtonText: {
    color: '#fff',
  },
  recipeInfo: {
    paddingVertical: 16,
    borderBottomWidth: 1,
    borderBottomColor: '#f0f0f0',
  },
  recipeTitle: {
    fontSize: 20,
    fontWeight: '700',
    color: '#333',
    marginBottom: 8,
  },
  recipeDescription: {
    fontSize: 14,
    color: '#666',
    lineHeight: 20,
    marginBottom: 12,
  },
  recipeStats: {
    flexDirection: 'row',
    gap: 16,
  },
  statText: {
    fontSize: 12,
    color: '#666',
  },
  customizationSection: {
    paddingVertical: 20,
    borderBottomWidth: 1,
    borderBottomColor: '#f0f0f0',
  },
  sectionTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#333',
    marginBottom: 16,
  },
  inputGroup: {
    marginBottom: 20,
  },
  inputLabel: {
    fontSize: 14,
    fontWeight: '600',
    color: '#333',
    marginBottom: 8,
  },
  textInput: {
    borderWidth: 1,
    borderColor: '#d1d5db',
    borderRadius: 8,
    paddingHorizontal: 12,
    paddingVertical: 10,
    fontSize: 16,
    backgroundColor: '#f9fafb',
  },
  notesInput: {
    height: 80,
    textAlignVertical: 'top',
  },
  numberInput: {
    width: 80,
  },
  charCount: {
    fontSize: 12,
    color: '#666',
    textAlign: 'right',
    marginTop: 4,
  },
  helpText: {
    fontSize: 12,
    color: '#666',
    marginTop: 4,
    fontStyle: 'italic',
  },
  attributionSection: {
    paddingVertical: 20,
  },
  switchRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 12,
  },
  switchInfo: {
    flex: 1,
    marginRight: 12,
  },
  switchLabel: {
    fontSize: 16,
    fontWeight: '600',
    color: '#333',
    marginBottom: 2,
  },
  switchDescription: {
    fontSize: 12,
    color: '#666',
    lineHeight: 16,
  },
  attributionPreview: {
    backgroundColor: '#f0f9ff',
    borderRadius: 6,
    padding: 12,
  },
  attributionText: {
    fontSize: 12,
    color: '#0369a1',
    marginBottom: 2,
  },
  actionButtons: {
    flexDirection: 'row',
    gap: 12,
    paddingHorizontal: 20,
    paddingVertical: 16,
    borderTopWidth: 1,
    borderTopColor: '#f0f0f0',
  },
  cancelButton: {
    flex: 1,
    paddingVertical: 14,
    borderRadius: 8,
    borderWidth: 1,
    borderColor: '#d1d5db',
    alignItems: 'center',
  },
  importButton: {
    flex: 2,
    backgroundColor: '#007AFF',
    paddingVertical: 14,
    borderRadius: 8,
    alignItems: 'center',
    flexDirection: 'row',
    justifyContent: 'center',
  },
  disabledButton: {
    backgroundColor: '#9ca3af',
  },
  cancelButtonText: {
    fontSize: 16,
    fontWeight: '600',
    color: '#374151',
  },
  importButtonText: {
    fontSize: 16,
    fontWeight: '600',
    color: '#fff',
  },
  buttonLoader: {
    marginRight: 8,
  },
});
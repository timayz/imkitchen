import React, { useEffect, useState } from 'react';
import {
  View,
  Text,
  ScrollView,
  TouchableOpacity,
  StyleSheet,
  Alert,
  ActivityIndicator,
  Platform,
  SafeAreaView,
} from 'react-native';
import { usePreferenceStore } from '../store/preference_store';
import { TimeConstraintSlider } from '../components/atoms/TimeConstraintSlider';
import { ComplexityPreferenceSelector } from '../components/atoms/ComplexityPreferenceSelector';

interface PreferenceSettingsScreenProps {
  navigation?: any; // Navigation prop if using React Navigation
}

export const PreferenceSettingsScreen: React.FC<PreferenceSettingsScreenProps> = ({
  navigation,
}) => {
  const {
    preferences,
    isLoading,
    error,
    lastUpdated,
    loadPreferences,
    updatePreferences,
    resetPreferences,
    clearError,
    setMaxCookTime,
    setPreferredComplexity,
  } = usePreferenceStore();

  const [hasChanges, setHasChanges] = useState(false);
  const [initialPreferences, setInitialPreferences] = useState(preferences);

  // Load preferences on mount
  useEffect(() => {
    loadPreferences();
  }, [loadPreferences]);

  // Track changes
  useEffect(() => {
    const changed = 
      preferences.maxCookTime !== initialPreferences.maxCookTime ||
      preferences.preferredComplexity !== initialPreferences.preferredComplexity;
    setHasChanges(changed);
  }, [preferences, initialPreferences]);

  // Update initial preferences when loaded
  useEffect(() => {
    if (!isLoading && !error) {
      setInitialPreferences(preferences);
      setHasChanges(false);
    }
  }, [preferences, isLoading, error]);

  const handleMaxCookTimeChange = (time: number) => {
    setMaxCookTime(time);
  };

  const handleComplexityChange = (complexity: 'simple' | 'moderate' | 'complex') => {
    setPreferredComplexity(complexity);
  };

  const handleSave = async () => {
    try {
      await updatePreferences(preferences);
      setInitialPreferences(preferences);
      setHasChanges(false);
      
      Alert.alert(
        'Success',
        'Your preferences have been saved successfully!',
        [{ text: 'OK' }]
      );
    } catch (err) {
      // Error is handled by the store
      Alert.alert(
        'Error',
        'Failed to save preferences. Please try again.',
        [{ text: 'OK' }]
      );
    }
  };

  const handleReset = () => {
    Alert.alert(
      'Reset Preferences',
      'Are you sure you want to reset your preferences to default values?',
      [
        {
          text: 'Cancel',
          style: 'cancel',
        },
        {
          text: 'Reset',
          style: 'destructive',
          onPress: async () => {
            try {
              await resetPreferences();
              setHasChanges(false);
              Alert.alert(
                'Success',
                'Your preferences have been reset to defaults!',
                [{ text: 'OK' }]
              );
            } catch (err) {
              Alert.alert(
                'Error',
                'Failed to reset preferences. Please try again.',
                [{ text: 'OK' }]
              );
            }
          },
        },
      ]
    );
  };

  const handleDiscard = () => {
    if (!hasChanges) return;

    Alert.alert(
      'Discard Changes',
      'Are you sure you want to discard your changes?',
      [
        {
          text: 'Keep Editing',
          style: 'cancel',
        },
        {
          text: 'Discard',
          style: 'destructive',
          onPress: () => {
            setMaxCookTime(initialPreferences.maxCookTime);
            setPreferredComplexity(initialPreferences.preferredComplexity);
            setHasChanges(false);
          },
        },
      ]
    );
  };

  const handleGoBack = () => {
    if (hasChanges) {
      Alert.alert(
        'Unsaved Changes',
        'You have unsaved changes. What would you like to do?',
        [
          {
            text: 'Save & Exit',
            onPress: async () => {
              await handleSave();
              navigation?.goBack();
            },
          },
          {
            text: 'Discard Changes',
            style: 'destructive',
            onPress: () => {
              navigation?.goBack();
            },
          },
          {
            text: 'Keep Editing',
            style: 'cancel',
          },
        ]
      );
    } else {
      navigation?.goBack();
    }
  };

  if (error) {
    return (
      <SafeAreaView style={styles.container}>
        <View style={styles.errorContainer}>
          <Text style={styles.errorText}>Error: {error}</Text>
          <TouchableOpacity style={styles.retryButton} onPress={() => {
            clearError();
            loadPreferences();
          }}>
            <Text style={styles.retryButtonText}>Retry</Text>
          </TouchableOpacity>
        </View>
      </SafeAreaView>
    );
  }

  return (
    <SafeAreaView style={styles.container}>
      <View style={styles.header}>
        <TouchableOpacity onPress={handleGoBack} style={styles.backButton}>
          <Text style={styles.backButtonText}>← Back</Text>
        </TouchableOpacity>
        <Text style={styles.headerTitle}>Cooking Preferences</Text>
        <View style={styles.placeholder} />
      </View>

      <ScrollView style={styles.scrollView} showsVerticalScrollIndicator={false}>
        <View style={styles.content}>
          <Text style={styles.sectionTitle}>Customize Your Cooking Experience</Text>
          <Text style={styles.sectionDescription}>
            Set your preferences to get meal plans tailored to your cooking style and available time.
          </Text>

          <View style={styles.preferenceSection}>
            <TimeConstraintSlider
              value={preferences.maxCookTime}
              onValueChange={handleMaxCookTimeChange}
              disabled={isLoading}
            />

            <ComplexityPreferenceSelector
              value={preferences.preferredComplexity}
              onValueChange={handleComplexityChange}
              disabled={isLoading}
            />
          </View>

          {lastUpdated && (
            <Text style={styles.lastUpdated}>
              Last updated: {new Date(lastUpdated).toLocaleDateString()} at{' '}
              {new Date(lastUpdated).toLocaleTimeString()}
            </Text>
          )}
        </View>
      </ScrollView>

      <View style={styles.footer}>
        {hasChanges && (
          <View style={styles.changesIndicator}>
            <Text style={styles.changesText}>You have unsaved changes</Text>
          </View>
        )}

        <View style={styles.buttonContainer}>
          <TouchableOpacity
            style={[styles.button, styles.resetButton]}
            onPress={handleReset}
            disabled={isLoading}
          >
            <Text style={styles.resetButtonText}>Reset to Defaults</Text>
          </TouchableOpacity>

          <TouchableOpacity
            style={[styles.button, styles.discardButton]}
            onPress={handleDiscard}
            disabled={!hasChanges || isLoading}
          >
            <Text style={[styles.discardButtonText, (!hasChanges || isLoading) && styles.disabledButtonText]}>
              Discard
            </Text>
          </TouchableOpacity>

          <TouchableOpacity
            style={[styles.button, styles.saveButton, (!hasChanges || isLoading) && styles.disabledButton]}
            onPress={handleSave}
            disabled={!hasChanges || isLoading}
          >
            {isLoading ? (
              <ActivityIndicator color="#FFF" size="small" />
            ) : (
              <Text style={styles.saveButtonText}>Save Changes</Text>
            )}
          </TouchableOpacity>
        </View>
      </View>
    </SafeAreaView>
  );
};

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#FFF',
  },
  header: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    paddingHorizontal: 16,
    paddingVertical: 12,
    borderBottomWidth: 1,
    borderBottomColor: '#E0E0E0',
  },
  backButton: {
    padding: 8,
  },
  backButtonText: {
    fontSize: 16,
    color: '#4CAF50',
    fontWeight: '600',
  },
  headerTitle: {
    fontSize: 18,
    fontWeight: '700',
    color: '#333',
  },
  placeholder: {
    width: 60, // To center the title
  },
  scrollView: {
    flex: 1,
  },
  content: {
    padding: 16,
  },
  sectionTitle: {
    fontSize: 24,
    fontWeight: '700',
    color: '#333',
    marginBottom: 8,
  },
  sectionDescription: {
    fontSize: 16,
    color: '#666',
    marginBottom: 32,
    lineHeight: 22,
  },
  preferenceSection: {
    marginBottom: 32,
  },
  lastUpdated: {
    fontSize: 12,
    color: '#999',
    textAlign: 'center',
    fontStyle: 'italic',
  },
  footer: {
    borderTopWidth: 1,
    borderTopColor: '#E0E0E0',
    backgroundColor: '#FAFAFA',
    paddingHorizontal: 16,
    paddingVertical: 12,
    ...Platform.select({
      ios: {
        shadowColor: '#000',
        shadowOffset: { width: 0, height: -2 },
        shadowOpacity: 0.1,
        shadowRadius: 4,
      },
      android: {
        elevation: 8,
      },
    }),
  },
  changesIndicator: {
    backgroundColor: '#FFF3CD',
    borderColor: '#FFEAA7',
    borderWidth: 1,
    borderRadius: 8,
    padding: 12,
    marginBottom: 16,
    alignItems: 'center',
  },
  changesText: {
    fontSize: 14,
    color: '#856404',
    fontWeight: '600',
  },
  buttonContainer: {
    flexDirection: 'row',
    gap: 12,
  },
  button: {
    flex: 1,
    paddingVertical: 16,
    borderRadius: 8,
    alignItems: 'center',
    justifyContent: 'center',
    minHeight: 48,
  },
  saveButton: {
    backgroundColor: '#4CAF50',
  },
  disabledButton: {
    backgroundColor: '#CCC',
  },
  saveButtonText: {
    color: '#FFF',
    fontSize: 16,
    fontWeight: '600',
  },
  resetButton: {
    backgroundColor: 'transparent',
    borderWidth: 1,
    borderColor: '#F44336',
  },
  resetButtonText: {
    color: '#F44336',
    fontSize: 14,
    fontWeight: '600',
  },
  discardButton: {
    backgroundColor: 'transparent',
    borderWidth: 1,
    borderColor: '#999',
  },
  discardButtonText: {
    color: '#999',
    fontSize: 14,
    fontWeight: '600',
  },
  disabledButtonText: {
    color: '#CCC',
  },
  errorContainer: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    padding: 32,
  },
  errorText: {
    fontSize: 16,
    color: '#F44336',
    textAlign: 'center',
    marginBottom: 24,
  },
  retryButton: {
    backgroundColor: '#4CAF50',
    paddingHorizontal: 24,
    paddingVertical: 12,
    borderRadius: 8,
  },
  retryButtonText: {
    color: '#FFF',
    fontSize: 16,
    fontWeight: '600',
  },
});
import React, { useEffect } from 'react';
import { View, Text, ScrollView, StyleSheet, Alert, ActivityIndicator } from 'react-native';
import { WeeklyAvailabilityGrid, WeeklyPattern } from '../../components/patterns/WeeklyAvailabilityGrid';
import { usePreferenceStore } from '../../store/preference_store';

export const WeeklyPatternsScreen: React.FC = () => {
  const {
    weeklyPatterns,
    isLoading,
    error,
    loadWeeklyPatterns,
    updateDayPattern,
    clearError
  } = usePreferenceStore();

  useEffect(() => {
    loadWeeklyPatterns();
  }, [loadWeeklyPatterns]);

  useEffect(() => {
    if (error) {
      Alert.alert(
        'Error',
        error,
        [
          { text: 'OK', onPress: clearError }
        ]
      );
    }
  }, [error, clearError]);

  const handlePatternUpdate = async (dayOfWeek: number, pattern: Partial<WeeklyPattern>) => {
    try {
      await updateDayPattern(dayOfWeek, pattern);
    } catch (err) {
      console.error('Failed to update pattern:', err);
      Alert.alert(
        'Update Failed',
        'Failed to save your cooking pattern. Please try again.',
        [{ text: 'OK' }]
      );
    }
  };

  if (isLoading && weeklyPatterns.length === 0) {
    return (
      <View style={styles.loadingContainer}>
        <ActivityIndicator size="large" color="#3B82F6" />
        <Text style={styles.loadingText}>Loading your cooking patterns...</Text>
      </View>
    );
  }

  return (
    <ScrollView style={styles.container} showsVerticalScrollIndicator={false}>
      <View style={styles.header}>
        <Text style={styles.title}>Weekly Cooking Patterns</Text>
        <Text style={styles.subtitle}>
          Configure your cooking preferences for each day of the week. 
          Weekend patterns allow for more elaborate meals.
        </Text>
      </View>

      <WeeklyAvailabilityGrid
        patterns={weeklyPatterns}
        onPatternUpdate={handlePatternUpdate}
        disabled={isLoading}
      />

      <View style={styles.helpSection}>
        <Text style={styles.helpTitle}>How it works:</Text>
        <Text style={styles.helpText}>
          • <Text style={styles.boldText}>Weekend Pattern:</Text> Enables longer cooking times and complex recipes
        </Text>
        <Text style={styles.helpText}>
          • <Text style={styles.boldText}>Weekday Pattern:</Text> Focuses on quick, simple meals
        </Text>
        <Text style={styles.helpText}>
          • <Text style={styles.boldText}>Time Limits:</Text> Maximum prep time you're comfortable with
        </Text>
        <Text style={styles.helpText}>
          • <Text style={styles.boldText}>Complexity:</Text> Preferred recipe difficulty level
        </Text>
      </View>

      <View style={styles.bottomSpacer} />
    </ScrollView>
  );
};

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#F9FAFB',
  },
  loadingContainer: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    backgroundColor: '#F9FAFB',
  },
  loadingText: {
    marginTop: 16,
    fontSize: 16,
    color: '#6B7280',
    textAlign: 'center',
  },
  header: {
    padding: 20,
    backgroundColor: '#FFFFFF',
    borderBottomWidth: 1,
    borderBottomColor: '#E5E7EB',
  },
  title: {
    fontSize: 24,
    fontWeight: '700',
    color: '#1F2937',
    marginBottom: 8,
  },
  subtitle: {
    fontSize: 16,
    color: '#6B7280',
    lineHeight: 24,
  },
  helpSection: {
    margin: 16,
    padding: 16,
    backgroundColor: '#FFFFFF',
    borderRadius: 12,
    borderWidth: 1,
    borderColor: '#E5E7EB',
  },
  helpTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#1F2937',
    marginBottom: 12,
  },
  helpText: {
    fontSize: 14,
    color: '#6B7280',
    marginBottom: 8,
    lineHeight: 20,
  },
  boldText: {
    fontWeight: '600',
    color: '#374151',
  },
  bottomSpacer: {
    height: 32,
  },
});
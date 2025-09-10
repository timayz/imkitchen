import React from 'react';
import { View, Text, TouchableOpacity, StyleSheet } from 'react-native';
import { WeeklyPattern } from './WeeklyAvailabilityGrid';

interface DayPatternCardProps {
  dayName: string;
  pattern: WeeklyPattern;
  onPatternUpdate: (pattern: Partial<WeeklyPattern>) => void;
  disabled?: boolean;
}

const TIME_OPTIONS = [15, 30, 45, 60, 90, 120, 180];
const COMPLEXITY_OPTIONS = [
  { value: 'simple' as const, label: 'Simple', color: '#10B981' },
  { value: 'moderate' as const, label: 'Moderate', color: '#F59E0B' },
  { value: 'complex' as const, label: 'Complex', color: '#EF4444' }
];

export const DayPatternCard: React.FC<DayPatternCardProps> = ({
  dayName,
  pattern,
  onPatternUpdate,
  disabled = false
}) => {
  const handleTimeChange = (time: number) => {
    onPatternUpdate({ ...pattern, maxPrepTime: time });
  };

  const handleComplexityChange = (complexity: 'simple' | 'moderate' | 'complex') => {
    onPatternUpdate({ ...pattern, preferredComplexity: complexity });
  };

  const toggleWeekendPattern = () => {
    onPatternUpdate({ ...pattern, isWeekendPattern: !pattern.isWeekendPattern });
  };

  return (
    <View style={[
      styles.container,
      pattern.isWeekendPattern && styles.weekendContainer,
      disabled && styles.disabledContainer
    ]}>
      <View style={styles.header}>
        <Text style={[styles.dayName, pattern.isWeekendPattern && styles.weekendText]}>
          {dayName}
        </Text>
        
        <TouchableOpacity
          onPress={toggleWeekendPattern}
          disabled={disabled}
          style={[
            styles.weekendToggle,
            pattern.isWeekendPattern && styles.weekendToggleActive
          ]}
        >
          <Text style={styles.weekendToggleText}>
            {pattern.isWeekendPattern ? '🏠' : '⏰'}
          </Text>
        </TouchableOpacity>
      </View>

      {/* Time Availability */}
      <View style={styles.section}>
        <Text style={styles.sectionLabel}>Available Time</Text>
        <View style={styles.timeSlider}>
          {TIME_OPTIONS.map(time => (
            <TouchableOpacity
              key={time}
              onPress={() => handleTimeChange(time)}
              disabled={disabled}
              style={[
                styles.timeOption,
                pattern.maxPrepTime === time && styles.selectedTimeOption,
                disabled && styles.disabledOption
              ]}
            >
              <Text style={[
                styles.timeOptionText,
                pattern.maxPrepTime === time && styles.selectedTimeOptionText
              ]}>
                {time}m
              </Text>
            </TouchableOpacity>
          ))}
        </View>
      </View>

      {/* Complexity Preference */}
      <View style={styles.section}>
        <Text style={styles.sectionLabel}>Preferred Complexity</Text>
        <View style={styles.complexityOptions}>
          {COMPLEXITY_OPTIONS.map(option => (
            <TouchableOpacity
              key={option.value}
              onPress={() => handleComplexityChange(option.value)}
              disabled={disabled}
              style={[
                styles.complexityOption,
                pattern.preferredComplexity === option.value && {
                  backgroundColor: option.color,
                  borderColor: option.color
                },
                disabled && styles.disabledOption
              ]}
            >
              <Text style={[
                styles.complexityOptionText,
                pattern.preferredComplexity === option.value && styles.selectedComplexityText
              ]}>
                {option.label}
              </Text>
            </TouchableOpacity>
          ))}
        </View>
      </View>

      {/* Pattern Preview */}
      <View style={styles.previewSection}>
        <Text style={styles.previewLabel}>Pattern Preview:</Text>
        <Text style={styles.previewText}>
          {pattern.isWeekendPattern ? '🏠 Weekend cooking' : '⏰ Weekday cooking'} • 
          {pattern.maxPrepTime}min max • 
          {pattern.preferredComplexity} meals
        </Text>
      </View>
    </View>
  );
};

const styles = StyleSheet.create({
  container: {
    backgroundColor: '#FFFFFF',
    borderRadius: 12,
    padding: 16,
    marginBottom: 12,
    borderWidth: 1,
    borderColor: '#E5E7EB',
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 1 },
    shadowOpacity: 0.05,
    shadowRadius: 2,
    elevation: 2,
  },
  weekendContainer: {
    backgroundColor: '#FFFBEB',
    borderColor: '#F59E0B',
  },
  disabledContainer: {
    opacity: 0.6,
  },
  header: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 16,
  },
  dayName: {
    fontSize: 18,
    fontWeight: '600',
    color: '#1F2937',
  },
  weekendText: {
    color: '#92400E',
  },
  weekendToggle: {
    width: 36,
    height: 36,
    borderRadius: 18,
    backgroundColor: '#F3F4F6',
    justifyContent: 'center',
    alignItems: 'center',
    borderWidth: 2,
    borderColor: '#E5E7EB',
  },
  weekendToggleActive: {
    backgroundColor: '#FEF3C7',
    borderColor: '#F59E0B',
  },
  weekendToggleText: {
    fontSize: 16,
  },
  section: {
    marginBottom: 16,
  },
  sectionLabel: {
    fontSize: 14,
    fontWeight: '500',
    color: '#374151',
    marginBottom: 8,
  },
  timeSlider: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: 8,
  },
  timeOption: {
    paddingHorizontal: 12,
    paddingVertical: 6,
    borderRadius: 16,
    backgroundColor: '#F3F4F6',
    borderWidth: 1,
    borderColor: '#E5E7EB',
  },
  selectedTimeOption: {
    backgroundColor: '#3B82F6',
    borderColor: '#3B82F6',
  },
  disabledOption: {
    opacity: 0.5,
  },
  timeOptionText: {
    fontSize: 12,
    fontWeight: '500',
    color: '#6B7280',
  },
  selectedTimeOptionText: {
    color: '#FFFFFF',
  },
  complexityOptions: {
    flexDirection: 'row',
    gap: 8,
  },
  complexityOption: {
    flex: 1,
    paddingVertical: 8,
    paddingHorizontal: 12,
    borderRadius: 8,
    backgroundColor: '#F3F4F6',
    borderWidth: 1,
    borderColor: '#E5E7EB',
    alignItems: 'center',
  },
  complexityOptionText: {
    fontSize: 12,
    fontWeight: '500',
    color: '#6B7280',
  },
  selectedComplexityText: {
    color: '#FFFFFF',
    fontWeight: '600',
  },
  previewSection: {
    backgroundColor: '#F9FAFB',
    borderRadius: 8,
    padding: 12,
    marginTop: 4,
  },
  previewLabel: {
    fontSize: 12,
    fontWeight: '500',
    color: '#6B7280',
    marginBottom: 4,
  },
  previewText: {
    fontSize: 12,
    color: '#374151',
    lineHeight: 16,
  },
});
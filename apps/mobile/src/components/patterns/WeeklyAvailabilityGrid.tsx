import React from 'react';
import { View, Text, TouchableOpacity, StyleSheet } from 'react-native';

export interface WeeklyPattern {
  dayOfWeek: number;         // 0=Sunday, 6=Saturday
  maxPrepTime: number;       // Minutes
  preferredComplexity: 'simple' | 'moderate' | 'complex';
  isWeekendPattern: boolean;
}

interface WeeklyAvailabilityGridProps {
  patterns: WeeklyPattern[];
  onPatternUpdate: (dayOfWeek: number, pattern: Partial<WeeklyPattern>) => void;
  disabled?: boolean;
}

const DAYS = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'];
const COMPLEXITY_LEVELS = [
  { value: 'simple' as const, label: 'Simple', color: '#10B981' },
  { value: 'moderate' as const, label: 'Moderate', color: '#F59E0B' },
  { value: 'complex' as const, label: 'Complex', color: '#EF4444' }
];

export const WeeklyAvailabilityGrid: React.FC<WeeklyAvailabilityGridProps> = ({
  patterns,
  onPatternUpdate,
  disabled = false
}) => {
  const getPatternForDay = (dayOfWeek: number): WeeklyPattern => {
    return patterns.find(p => p.dayOfWeek === dayOfWeek) || {
      dayOfWeek,
      maxPrepTime: 60,
      preferredComplexity: 'moderate',
      isWeekendPattern: dayOfWeek === 0 || dayOfWeek === 6
    };
  };

  const handleComplexityChange = (dayOfWeek: number, complexity: 'simple' | 'moderate' | 'complex') => {
    const pattern = getPatternForDay(dayOfWeek);
    onPatternUpdate(dayOfWeek, { ...pattern, preferredComplexity: complexity });
  };

  const handleTimeChange = (dayOfWeek: number, time: number) => {
    const pattern = getPatternForDay(dayOfWeek);
    onPatternUpdate(dayOfWeek, { ...pattern, maxPrepTime: time });
  };

  const toggleWeekendPattern = (dayOfWeek: number) => {
    const pattern = getPatternForDay(dayOfWeek);
    onPatternUpdate(dayOfWeek, { ...pattern, isWeekendPattern: !pattern.isWeekendPattern });
  };

  return (
    <View style={styles.container}>
      <Text style={styles.title}>Weekly Cooking Patterns</Text>
      
      <View style={styles.grid}>
        {DAYS.map((dayName, index) => {
          const pattern = getPatternForDay(index);
          const isWeekend = index === 0 || index === 6;
          
          return (
            <View 
              key={index} 
              style={[
                styles.dayCard,
                pattern.isWeekendPattern && styles.weekendCard,
                disabled && styles.disabledCard
              ]}
            >
              <View style={styles.dayHeader}>
                <Text style={[styles.dayName, pattern.isWeekendPattern && styles.weekendText]}>
                  {dayName}
                </Text>
                {isWeekend && (
                  <TouchableOpacity
                    onPress={() => toggleWeekendPattern(index)}
                    disabled={disabled}
                    style={[styles.weekendToggle, pattern.isWeekendPattern && styles.weekendToggleActive]}
                  >
                    <Text style={styles.weekendToggleText}>
                      {pattern.isWeekendPattern ? '🏠' : '⏰'}
                    </Text>
                  </TouchableOpacity>
                )}
              </View>

              {/* Time Selection */}
              <View style={styles.timeContainer}>
                <Text style={styles.label}>Max Time</Text>
                <View style={styles.timeButtons}>
                  {[30, 60, 90, 120].map(time => (
                    <TouchableOpacity
                      key={time}
                      onPress={() => handleTimeChange(index, time)}
                      disabled={disabled}
                      style={[
                        styles.timeButton,
                        pattern.maxPrepTime === time && styles.selectedTimeButton,
                        disabled && styles.disabledButton
                      ]}
                    >
                      <Text style={[
                        styles.timeButtonText,
                        pattern.maxPrepTime === time && styles.selectedTimeButtonText
                      ]}>
                        {time}m
                      </Text>
                    </TouchableOpacity>
                  ))}
                </View>
              </View>

              {/* Complexity Selection */}
              <View style={styles.complexityContainer}>
                <Text style={styles.label}>Complexity</Text>
                <View style={styles.complexityButtons}>
                  {COMPLEXITY_LEVELS.map(level => (
                    <TouchableOpacity
                      key={level.value}
                      onPress={() => handleComplexityChange(index, level.value)}
                      disabled={disabled}
                      style={[
                        styles.complexityButton,
                        pattern.preferredComplexity === level.value && { backgroundColor: level.color },
                        disabled && styles.disabledButton
                      ]}
                    >
                      <Text style={[
                        styles.complexityButtonText,
                        pattern.preferredComplexity === level.value && styles.selectedComplexityText
                      ]}>
                        {level.label}
                      </Text>
                    </TouchableOpacity>
                  ))}
                </View>
              </View>
            </View>
          );
        })}
      </View>
    </View>
  );
};

const styles = StyleSheet.create({
  container: {
    padding: 16,
    backgroundColor: '#FFFFFF',
  },
  title: {
    fontSize: 20,
    fontWeight: '600',
    color: '#1F2937',
    marginBottom: 16,
    textAlign: 'center',
  },
  grid: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: 8,
    justifyContent: 'space-between',
  },
  dayCard: {
    width: '48%',
    backgroundColor: '#F9FAFB',
    borderRadius: 12,
    padding: 12,
    marginBottom: 8,
    borderWidth: 1,
    borderColor: '#E5E7EB',
  },
  weekendCard: {
    backgroundColor: '#FEF3C7',
    borderColor: '#F59E0B',
  },
  disabledCard: {
    opacity: 0.6,
  },
  dayHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 12,
  },
  dayName: {
    fontSize: 16,
    fontWeight: '600',
    color: '#1F2937',
  },
  weekendText: {
    color: '#92400E',
  },
  weekendToggle: {
    width: 28,
    height: 28,
    borderRadius: 14,
    backgroundColor: '#E5E7EB',
    justifyContent: 'center',
    alignItems: 'center',
  },
  weekendToggleActive: {
    backgroundColor: '#F59E0B',
  },
  weekendToggleText: {
    fontSize: 14,
  },
  timeContainer: {
    marginBottom: 12,
  },
  label: {
    fontSize: 12,
    fontWeight: '500',
    color: '#6B7280',
    marginBottom: 6,
  },
  timeButtons: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: 4,
  },
  timeButton: {
    paddingHorizontal: 8,
    paddingVertical: 4,
    borderRadius: 6,
    backgroundColor: '#E5E7EB',
    minWidth: 32,
    alignItems: 'center',
  },
  selectedTimeButton: {
    backgroundColor: '#3B82F6',
  },
  disabledButton: {
    opacity: 0.5,
  },
  timeButtonText: {
    fontSize: 11,
    fontWeight: '500',
    color: '#6B7280',
  },
  selectedTimeButtonText: {
    color: '#FFFFFF',
  },
  complexityContainer: {
    marginTop: 8,
  },
  complexityButtons: {
    flexDirection: 'row',
    gap: 4,
  },
  complexityButton: {
    flex: 1,
    paddingVertical: 6,
    paddingHorizontal: 4,
    borderRadius: 6,
    backgroundColor: '#E5E7EB',
    alignItems: 'center',
  },
  complexityButtonText: {
    fontSize: 10,
    fontWeight: '500',
    color: '#6B7280',
  },
  selectedComplexityText: {
    color: '#FFFFFF',
    fontWeight: '600',
  },
});
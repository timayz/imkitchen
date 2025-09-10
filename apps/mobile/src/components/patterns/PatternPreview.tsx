import React from 'react';
import { View, Text, StyleSheet } from 'react-native';
import { WeeklyPattern } from './WeeklyAvailabilityGrid';

interface PatternPreviewProps {
  patterns: WeeklyPattern[];
  selectedDay?: number;
}

const DAYS = ['Sunday', 'Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday'];

export const PatternPreview: React.FC<PatternPreviewProps> = ({
  patterns,
  selectedDay
}) => {
  const getPreviewForDay = (dayOfWeek: number) => {
    const pattern = patterns.find(p => p.dayOfWeek === dayOfWeek);
    if (!pattern) return null;

    const complexity = pattern.preferredComplexity;
    const time = pattern.maxPrepTime;
    const isWeekend = pattern.isWeekendPattern;

    return {
      day: DAYS[dayOfWeek],
      pattern,
      description: `${isWeekend ? '🏠' : '⏰'} ${complexity} meals (${time}min max)`,
      color: isWeekend ? '#F59E0B' : '#6B7280',
      backgroundColor: isWeekend ? '#FEF3C7' : '#F3F4F6'
    };
  };

  const previewData = patterns.map(p => getPreviewForDay(p.dayOfWeek)).filter(Boolean);

  if (selectedDay !== undefined) {
    const dayPreview = getPreviewForDay(selectedDay);
    if (!dayPreview) return null;

    return (
      <View style={styles.singlePreview}>
        <Text style={styles.previewTitle}>Pattern Preview</Text>
        <View style={[styles.dayPreview, { backgroundColor: dayPreview.backgroundColor }]}>
          <Text style={[styles.dayName, { color: dayPreview.color }]}>
            {dayPreview.day}
          </Text>
          <Text style={[styles.dayDescription, { color: dayPreview.color }]}>
            {dayPreview.description}
          </Text>
        </View>
      </View>
    );
  }

  return (
    <View style={styles.container}>
      <Text style={styles.title}>Weekly Pattern Overview</Text>
      
      <View style={styles.weekPreview}>
        {previewData.map((preview, index) => (
          <View
            key={index}
            style={[
              styles.dayPreview,
              { backgroundColor: preview!.backgroundColor }
            ]}
          >
            <Text style={[styles.dayName, { color: preview!.color }]}>
              {preview!.day.slice(0, 3)}
            </Text>
            <Text style={[styles.dayDescription, { color: preview!.color }]}>
              {preview!.description}
            </Text>
          </View>
        ))}
      </View>

      <View style={styles.summary}>
        <Text style={styles.summaryTitle}>Your Weekly Pattern:</Text>
        <View style={styles.summaryStats}>
          <View style={styles.statItem}>
            <Text style={styles.statNumber}>
              {patterns.filter(p => p.isWeekendPattern).length}
            </Text>
            <Text style={styles.statLabel}>Weekend Days</Text>
          </View>
          <View style={styles.statItem}>
            <Text style={styles.statNumber}>
              {Math.round(patterns.reduce((sum, p) => sum + p.maxPrepTime, 0) / patterns.length)}
            </Text>
            <Text style={styles.statLabel}>Avg Time (min)</Text>
          </View>
          <View style={styles.statItem}>
            <Text style={styles.statNumber}>
              {patterns.filter(p => p.preferredComplexity === 'complex').length}
            </Text>
            <Text style={styles.statLabel}>Complex Days</Text>
          </View>
        </View>
      </View>
    </View>
  );
};

const styles = StyleSheet.create({
  container: {
    backgroundColor: '#FFFFFF',
    borderRadius: 12,
    padding: 16,
    margin: 16,
    borderWidth: 1,
    borderColor: '#E5E7EB',
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 1 },
    shadowOpacity: 0.05,
    shadowRadius: 2,
    elevation: 2,
  },
  singlePreview: {
    backgroundColor: '#FFFFFF',
    borderRadius: 12,
    padding: 16,
    borderWidth: 1,
    borderColor: '#E5E7EB',
  },
  title: {
    fontSize: 18,
    fontWeight: '600',
    color: '#1F2937',
    marginBottom: 16,
    textAlign: 'center',
  },
  previewTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#1F2937',
    marginBottom: 12,
  },
  weekPreview: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: 8,
    marginBottom: 20,
  },
  dayPreview: {
    flex: 1,
    minWidth: '30%',
    padding: 8,
    borderRadius: 8,
    alignItems: 'center',
  },
  dayName: {
    fontSize: 12,
    fontWeight: '600',
    marginBottom: 4,
  },
  dayDescription: {
    fontSize: 10,
    textAlign: 'center',
    lineHeight: 14,
  },
  summary: {
    borderTopWidth: 1,
    borderTopColor: '#E5E7EB',
    paddingTop: 16,
  },
  summaryTitle: {
    fontSize: 14,
    fontWeight: '600',
    color: '#1F2937',
    marginBottom: 12,
    textAlign: 'center',
  },
  summaryStats: {
    flexDirection: 'row',
    justifyContent: 'space-around',
  },
  statItem: {
    alignItems: 'center',
  },
  statNumber: {
    fontSize: 20,
    fontWeight: '700',
    color: '#3B82F6',
    marginBottom: 4,
  },
  statLabel: {
    fontSize: 12,
    color: '#6B7280',
    textAlign: 'center',
  },
});
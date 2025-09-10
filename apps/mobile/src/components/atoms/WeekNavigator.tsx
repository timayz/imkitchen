import React from 'react';
import {
  View,
  Text,
  TouchableOpacity,
  StyleSheet,
} from 'react-native';
import type { WeekNavigatorProps } from '@imkitchen/shared-types';

export const WeekNavigator: React.FC<WeekNavigatorProps> = ({
  currentWeek,
  onPreviousWeek,
  onNextWeek,
  onWeekSelect,
  showWeekSelector = false,
  minDate,
  maxDate,
}) => {
  const formatWeekRange = (weekStart: Date): string => {
    const weekEnd = new Date(weekStart);
    weekEnd.setDate(weekEnd.getDate() + 6);
    
    const startMonth = weekStart.toLocaleDateString('en-US', { month: 'short' });
    const startDay = weekStart.getDate();
    const endMonth = weekEnd.toLocaleDateString('en-US', { month: 'short' });
    const endDay = weekEnd.getDate();
    
    if (startMonth === endMonth) {
      return `${startMonth} ${startDay}-${endDay}`;
    } else {
      return `${startMonth} ${startDay} - ${endMonth} ${endDay}`;
    }
  };

  const formatYear = (date: Date): string => {
    return date.getFullYear().toString();
  };

  const isCurrentWeek = (date: Date): boolean => {
    const today = new Date();
    const startOfWeek = new Date(today);
    startOfWeek.setDate(today.getDate() - today.getDay() + 1); // Monday
    startOfWeek.setHours(0, 0, 0, 0);
    
    const weekStart = new Date(date);
    weekStart.setHours(0, 0, 0, 0);
    
    return weekStart.getTime() === startOfWeek.getTime();
  };

  const canNavigateBack = (): boolean => {
    if (!minDate) return true;
    const prevWeek = new Date(currentWeek);
    prevWeek.setDate(prevWeek.getDate() - 7);
    return prevWeek >= minDate;
  };

  const canNavigateForward = (): boolean => {
    if (!maxDate) return true;
    const nextWeek = new Date(currentWeek);
    nextWeek.setDate(nextWeek.getDate() + 7);
    return nextWeek <= maxDate;
  };

  const handlePreviousWeek = () => {
    if (canNavigateBack()) {
      onPreviousWeek();
    }
  };

  const handleNextWeek = () => {
    if (canNavigateForward()) {
      onNextWeek();
    }
  };

  const handleWeekSelect = () => {
    if (showWeekSelector && onWeekSelect) {
      // In a real implementation, this would open a date picker
      // For now, we'll just log the action
      console.log('Week selector pressed');
    }
  };

  return (
    <View style={styles.container}>
      {/* Navigation Controls */}
      <View style={styles.navigationRow}>
        {/* Previous Week Button */}
        <TouchableOpacity
          style={[
            styles.navButton,
            !canNavigateBack() && styles.navButtonDisabled
          ]}
          onPress={handlePreviousWeek}
          disabled={!canNavigateBack()}
          activeOpacity={0.7}
        >
          <Text style={[
            styles.navButtonText,
            !canNavigateBack() && styles.navButtonTextDisabled
          ]}>
            ‹
          </Text>
        </TouchableOpacity>

        {/* Week Display */}
        <TouchableOpacity
          style={styles.weekDisplay}
          onPress={handleWeekSelect}
          disabled={!showWeekSelector}
          activeOpacity={showWeekSelector ? 0.7 : 1}
        >
          <Text style={styles.weekRangeText}>
            {formatWeekRange(currentWeek)}
          </Text>
          <Text style={styles.yearText}>
            {formatYear(currentWeek)}
          </Text>
          {isCurrentWeek(currentWeek) && (
            <View style={styles.currentWeekIndicator}>
              <Text style={styles.currentWeekText}>This Week</Text>
            </View>
          )}
        </TouchableOpacity>

        {/* Next Week Button */}
        <TouchableOpacity
          style={[
            styles.navButton,
            !canNavigateForward() && styles.navButtonDisabled
          ]}
          onPress={handleNextWeek}
          disabled={!canNavigateForward()}
          activeOpacity={0.7}
        >
          <Text style={[
            styles.navButtonText,
            !canNavigateForward() && styles.navButtonTextDisabled
          ]}>
            ›
          </Text>
        </TouchableOpacity>
      </View>

      {/* Week Indicators */}
      <View style={styles.weekIndicators}>
        {Array.from({ length: 7 }, (_, index) => {
          const date = new Date(currentWeek);
          date.setDate(currentWeek.getDate() + index);
          const isToday = date.toDateString() === new Date().toDateString();
          
          return (
            <View
              key={index}
              style={[
                styles.dayIndicator,
                isToday && styles.todayIndicator
              ]}
            >
              <Text style={[
                styles.dayIndicatorText,
                isToday && styles.todayIndicatorText
              ]}>
                {date.getDate()}
              </Text>
            </View>
          );
        })}
      </View>
    </View>
  );
};

const styles = StyleSheet.create({
  container: {
    backgroundColor: '#fff',
    paddingVertical: 16,
    paddingHorizontal: 16,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.1,
    shadowRadius: 4,
    elevation: 3,
  },
  navigationRow: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    marginBottom: 16,
  },
  navButton: {
    width: 40,
    height: 40,
    borderRadius: 20,
    backgroundColor: '#f0f0f0',
    justifyContent: 'center',
    alignItems: 'center',
  },
  navButtonDisabled: {
    backgroundColor: '#f8f8f8',
  },
  navButtonText: {
    fontSize: 20,
    fontWeight: '600',
    color: '#333',
  },
  navButtonTextDisabled: {
    color: '#ccc',
  },
  weekDisplay: {
    flex: 1,
    alignItems: 'center',
    marginHorizontal: 16,
    position: 'relative',
  },
  weekRangeText: {
    fontSize: 18,
    fontWeight: '700',
    color: '#333',
    textAlign: 'center',
  },
  yearText: {
    fontSize: 14,
    color: '#666',
    marginTop: 2,
  },
  currentWeekIndicator: {
    position: 'absolute',
    top: -8,
    right: -16,
    backgroundColor: '#007AFF',
    paddingHorizontal: 6,
    paddingVertical: 2,
    borderRadius: 8,
  },
  currentWeekText: {
    fontSize: 10,
    color: '#fff',
    fontWeight: '600',
  },
  weekIndicators: {
    flexDirection: 'row',
    justifyContent: 'space-between',
  },
  dayIndicator: {
    width: 32,
    height: 32,
    borderRadius: 16,
    backgroundColor: '#f8f9fa',
    justifyContent: 'center',
    alignItems: 'center',
  },
  todayIndicator: {
    backgroundColor: '#007AFF',
  },
  dayIndicatorText: {
    fontSize: 12,
    fontWeight: '600',
    color: '#666',
  },
  todayIndicatorText: {
    color: '#fff',
  },
});
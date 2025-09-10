import React from 'react';
import { View, Text, TouchableOpacity, StyleSheet, Switch } from 'react-native';

interface WeekendToggleProps {
  isWeekendPattern: boolean;
  onToggle: (enabled: boolean) => void;
  disabled?: boolean;
  dayName?: string;
}

export const WeekendToggle: React.FC<WeekendToggleProps> = ({
  isWeekendPattern,
  onToggle,
  disabled = false,
  dayName
}) => {
  return (
    <View style={[styles.container, disabled && styles.disabledContainer]}>
      <TouchableOpacity
        onPress={() => onToggle(!isWeekendPattern)}
        disabled={disabled}
        style={[
          styles.toggleButton,
          isWeekendPattern && styles.weekendActive,
          disabled && styles.disabledButton
        ]}
        activeOpacity={0.7}
      >
        <View style={styles.iconContainer}>
          <Text style={styles.icon}>
            {isWeekendPattern ? '🏠' : '⏰'}
          </Text>
        </View>
        
        <View style={styles.textContainer}>
          <Text style={[
            styles.modeText,
            isWeekendPattern && styles.weekendModeText
          ]}>
            {isWeekendPattern ? 'Weekend Cooking' : 'Weekday Cooking'}
          </Text>
          {dayName && (
            <Text style={styles.dayText}>
              {dayName}
            </Text>
          )}
          <Text style={[
            styles.descriptionText,
            isWeekendPattern && styles.weekendDescriptionText
          ]}>
            {isWeekendPattern 
              ? 'More time for elaborate meals'
              : 'Quick and efficient cooking'
            }
          </Text>
        </View>

        <View style={styles.switchContainer}>
          <Switch
            value={isWeekendPattern}
            onValueChange={onToggle}
            disabled={disabled}
            trackColor={{ 
              false: '#E5E7EB', 
              true: '#F59E0B' 
            }}
            thumbColor={isWeekendPattern ? '#FFFFFF' : '#9CA3AF'}
            style={styles.switch}
          />
        </View>
      </TouchableOpacity>

      {/* Visual Indicator */}
      <View style={[
        styles.indicator,
        isWeekendPattern && styles.weekendIndicator
      ]} />
    </View>
  );
};

export default WeekendToggle;

const styles = StyleSheet.create({
  container: {
    position: 'relative',
  },
  disabledContainer: {
    opacity: 0.6,
  },
  toggleButton: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: '#FFFFFF',
    borderRadius: 12,
    padding: 16,
    borderWidth: 2,
    borderColor: '#E5E7EB',
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 1 },
    shadowOpacity: 0.05,
    shadowRadius: 2,
    elevation: 2,
  },
  weekendActive: {
    backgroundColor: '#FFFBEB',
    borderColor: '#F59E0B',
  },
  disabledButton: {
    opacity: 0.6,
  },
  iconContainer: {
    width: 40,
    height: 40,
    borderRadius: 20,
    backgroundColor: '#F3F4F6',
    justifyContent: 'center',
    alignItems: 'center',
    marginRight: 12,
  },
  icon: {
    fontSize: 18,
  },
  textContainer: {
    flex: 1,
  },
  modeText: {
    fontSize: 16,
    fontWeight: '600',
    color: '#1F2937',
    marginBottom: 2,
  },
  weekendModeText: {
    color: '#92400E',
  },
  dayText: {
    fontSize: 12,
    fontWeight: '500',
    color: '#6B7280',
    marginBottom: 2,
  },
  descriptionText: {
    fontSize: 12,
    color: '#6B7280',
    lineHeight: 16,
  },
  weekendDescriptionText: {
    color: '#78350F',
  },
  switchContainer: {
    marginLeft: 12,
  },
  switch: {
    transform: [{ scale: 0.9 }],
  },
  indicator: {
    position: 'absolute',
    left: 0,
    top: 0,
    bottom: 0,
    width: 4,
    backgroundColor: '#E5E7EB',
    borderTopLeftRadius: 12,
    borderBottomLeftRadius: 12,
  },
  weekendIndicator: {
    backgroundColor: '#F59E0B',
  },
});
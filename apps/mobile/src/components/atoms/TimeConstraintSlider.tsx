import React, { useState } from 'react';
import {
  View,
  Text,
  StyleSheet,
  Platform,
} from 'react-native';
import Slider from '@react-native-community/slider';

interface TimeConstraintSliderProps {
  value: number;
  onValueChange: (value: number) => void;
  onSlidingComplete?: (value: number) => void;
  disabled?: boolean;
  minimumValue?: number;
  maximumValue?: number;
}

export const TimeConstraintSlider: React.FC<TimeConstraintSliderProps> = ({
  value,
  onValueChange,
  onSlidingComplete,
  disabled = false,
  minimumValue = 15,
  maximumValue = 180,
}) => {
  const [localValue, setLocalValue] = useState(value);

  const handleValueChange = (newValue: number) => {
    const roundedValue = Math.round(newValue / 5) * 5; // Round to nearest 5 minutes
    setLocalValue(roundedValue);
    onValueChange(roundedValue);
  };

  const handleSlidingComplete = (newValue: number) => {
    const roundedValue = Math.round(newValue / 5) * 5;
    setLocalValue(roundedValue);
    onSlidingComplete?.(roundedValue);
  };

  const formatTime = (minutes: number): string => {
    if (minutes >= 60) {
      const hours = Math.floor(minutes / 60);
      const remainingMinutes = minutes % 60;
      if (remainingMinutes === 0) {
        return `${hours}h`;
      }
      return `${hours}h ${remainingMinutes}m`;
    }
    return `${minutes}m`;
  };

  const getTimeDescription = (minutes: number): string => {
    if (minutes <= 20) return 'Quick meals';
    if (minutes <= 45) return 'Standard cooking';
    if (minutes <= 90) return 'Longer recipes';
    return 'Complex dishes';
  };

  return (
    <View style={styles.container}>
      <View style={styles.header}>
        <Text style={styles.label}>Maximum Cook Time</Text>
        <Text style={styles.value}>{formatTime(localValue)}</Text>
      </View>
      
      <Text style={styles.description}>
        {getTimeDescription(localValue)}
      </Text>

      <View style={styles.sliderContainer}>
        <Slider
          style={styles.slider}
          value={localValue}
          minimumValue={minimumValue}
          maximumValue={maximumValue}
          onValueChange={handleValueChange}
          onSlidingComplete={handleSlidingComplete}
          disabled={disabled}
          minimumTrackTintColor="#4CAF50"
          maximumTrackTintColor="#E0E0E0"
          thumbStyle={styles.thumb}
          trackStyle={styles.track}
          step={5}
        />
        
        <View style={styles.rangeLabels}>
          <Text style={styles.rangeLabel}>{formatTime(minimumValue)}</Text>
          <Text style={styles.rangeLabel}>{formatTime(maximumValue)}</Text>
        </View>
      </View>
    </View>
  );
};

const styles = StyleSheet.create({
  container: {
    marginVertical: 16,
  },
  header: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 4,
  },
  label: {
    fontSize: 16,
    fontWeight: '600',
    color: '#333',
  },
  value: {
    fontSize: 16,
    fontWeight: 'bold',
    color: '#4CAF50',
  },
  description: {
    fontSize: 14,
    color: '#666',
    marginBottom: 16,
  },
  sliderContainer: {
    paddingHorizontal: 8,
  },
  slider: {
    width: '100%',
    height: 40,
  },
  thumb: {
    backgroundColor: '#4CAF50',
    borderRadius: 15,
    height: 20,
    width: 20,
    ...Platform.select({
      ios: {
        shadowColor: '#000',
        shadowOffset: { width: 0, height: 2 },
        shadowOpacity: 0.25,
        shadowRadius: 3.84,
      },
      android: {
        elevation: 5,
      },
    }),
  },
  track: {
    height: 4,
    borderRadius: 2,
  },
  rangeLabels: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    marginTop: 8,
  },
  rangeLabel: {
    fontSize: 12,
    color: '#999',
  },
});
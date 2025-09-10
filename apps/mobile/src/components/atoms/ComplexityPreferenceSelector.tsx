import React from 'react';
import {
  View,
  Text,
  TouchableOpacity,
  StyleSheet,
} from 'react-native';

type ComplexityLevel = 'simple' | 'moderate' | 'complex';

interface ComplexityOption {
  value: ComplexityLevel;
  label: string;
  description: string;
  color: string;
  icon: string;
}

const COMPLEXITY_OPTIONS: ComplexityOption[] = [
  {
    value: 'simple',
    label: 'Simple',
    description: 'Quick & easy recipes',
    color: '#4CAF50',
    icon: '⚡',
  },
  {
    value: 'moderate',
    label: 'Moderate',
    description: 'Balanced complexity',
    color: '#FF9800',
    icon: '🍳',
  },
  {
    value: 'complex',
    label: 'Complex',
    description: 'Advanced techniques',
    color: '#F44336',
    icon: '👨‍🍳',
  },
];

interface ComplexityPreferenceSelectorProps {
  value: ComplexityLevel;
  onValueChange: (complexity: ComplexityLevel) => void;
  disabled?: boolean;
}

export const ComplexityPreferenceSelector: React.FC<ComplexityPreferenceSelectorProps> = ({
  value,
  onValueChange,
  disabled = false,
}) => {
  const handlePress = (complexity: ComplexityLevel) => {
    if (!disabled) {
      onValueChange(complexity);
    }
  };

  return (
    <View style={styles.container}>
      <Text style={styles.label}>Preferred Complexity</Text>
      <Text style={styles.subtitle}>Choose your cooking comfort level</Text>
      
      <View style={styles.optionsContainer}>
        {COMPLEXITY_OPTIONS.map((option) => {
          const isSelected = value === option.value;
          
          return (
            <TouchableOpacity
              key={option.value}
              style={[
                styles.option,
                isSelected && styles.selectedOption,
                isSelected && { borderColor: option.color },
                disabled && styles.disabledOption,
              ]}
              onPress={() => handlePress(option.value)}
              disabled={disabled}
              accessibilityRole="radio"
              accessibilityState={{ selected: isSelected }}
              accessibilityLabel={`${option.label}: ${option.description}`}
            >
              <View style={styles.optionContent}>
                <View style={styles.iconContainer}>
                  <Text style={styles.icon}>{option.icon}</Text>
                </View>
                
                <View style={styles.textContainer}>
                  <Text
                    style={[
                      styles.optionLabel,
                      isSelected && styles.selectedLabel,
                      isSelected && { color: option.color },
                    ]}
                  >
                    {option.label}
                  </Text>
                  <Text
                    style={[
                      styles.optionDescription,
                      isSelected && styles.selectedDescription,
                    ]}
                  >
                    {option.description}
                  </Text>
                </View>
                
                <View style={styles.radioContainer}>
                  <View
                    style={[
                      styles.radio,
                      isSelected && styles.selectedRadio,
                      isSelected && { backgroundColor: option.color },
                    ]}
                  >
                    {isSelected && <View style={styles.radioInner} />}
                  </View>
                </View>
              </View>
            </TouchableOpacity>
          );
        })}
      </View>
    </View>
  );
};

const styles = StyleSheet.create({
  container: {
    marginVertical: 16,
  },
  label: {
    fontSize: 16,
    fontWeight: '600',
    color: '#333',
    marginBottom: 4,
  },
  subtitle: {
    fontSize: 14,
    color: '#666',
    marginBottom: 16,
  },
  optionsContainer: {
    gap: 12,
  },
  option: {
    borderWidth: 2,
    borderColor: '#E0E0E0',
    borderRadius: 12,
    padding: 16,
    backgroundColor: '#FAFAFA',
  },
  selectedOption: {
    backgroundColor: '#F8F9FA',
    borderWidth: 2,
  },
  disabledOption: {
    opacity: 0.6,
  },
  optionContent: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  iconContainer: {
    marginRight: 12,
  },
  icon: {
    fontSize: 24,
  },
  textContainer: {
    flex: 1,
  },
  optionLabel: {
    fontSize: 16,
    fontWeight: '600',
    color: '#333',
    marginBottom: 2,
  },
  selectedLabel: {
    fontWeight: '700',
  },
  optionDescription: {
    fontSize: 14,
    color: '#666',
  },
  selectedDescription: {
    color: '#555',
  },
  radioContainer: {
    marginLeft: 12,
  },
  radio: {
    width: 20,
    height: 20,
    borderRadius: 10,
    borderWidth: 2,
    borderColor: '#DDD',
    backgroundColor: '#FFF',
    alignItems: 'center',
    justifyContent: 'center',
  },
  selectedRadio: {
    borderColor: 'transparent',
  },
  radioInner: {
    width: 8,
    height: 8,
    borderRadius: 4,
    backgroundColor: '#FFF',
  },
});
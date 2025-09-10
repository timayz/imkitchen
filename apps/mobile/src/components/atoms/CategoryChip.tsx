import React from 'react';
import { TouchableOpacity, Text, StyleSheet, ViewStyle } from 'react-native';
import type { RecipeCategory } from '@imkitchen/shared-types';

interface CategoryChipProps {
  category: RecipeCategory;
  label: string;
  isSelected: boolean;
  onPress: (category: RecipeCategory) => void;
  style?: ViewStyle;
}

export const CategoryChip: React.FC<CategoryChipProps> = ({
  category,
  label,
  isSelected,
  onPress,
  style,
}) => {
  const handlePress = () => {
    onPress(category);
  };

  return (
    <TouchableOpacity
      style={[
        styles.chip,
        isSelected && styles.selectedChip,
        style,
      ]}
      onPress={handlePress}
      activeOpacity={0.7}
      accessibilityRole="button"
      accessibilityLabel={`${label} category filter`}
      accessibilityState={{ selected: isSelected }}
    >
      <Text style={[
        styles.chipText,
        isSelected && styles.selectedChipText,
      ]}>
        {label}
      </Text>
    </TouchableOpacity>
  );
};

const styles = StyleSheet.create({
  chip: {
    paddingHorizontal: 16,
    paddingVertical: 8,
    borderRadius: 20,
    borderWidth: 1,
    borderColor: '#e0e0e0',
    backgroundColor: '#f5f5f5',
    marginRight: 8,
    marginBottom: 8,
  },
  selectedChip: {
    backgroundColor: '#007AFF',
    borderColor: '#007AFF',
  },
  chipText: {
    fontSize: 14,
    fontWeight: '600',
    color: '#666',
  },
  selectedChipText: {
    color: '#fff',
  },
});
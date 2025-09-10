import React from 'react';
import {
  View,
  Text,
  ScrollView,
  StyleSheet,
} from 'react-native';
import { CategoryChip } from '../atoms/CategoryChip';
import type { RecipeCategory } from '@imkitchen/shared-types';

interface CategoryDiscoverySectionProps {
  selectedCategories: RecipeCategory[];
  onCategoryToggle: (category: RecipeCategory) => void;
  onClearCategories: () => void;
}

const CATEGORY_OPTIONS: { category: RecipeCategory; label: string; emoji: string }[] = [
  { category: 'vegetarian', label: 'Vegetarian', emoji: '🥬' },
  { category: 'vegan', label: 'Vegan', emoji: '🌱' },
  { category: 'quick_meals', label: 'Quick Meals', emoji: '⚡' },
  { category: 'comfort_food', label: 'Comfort Food', emoji: '🍲' },
  { category: 'healthy', label: 'Healthy', emoji: '💚' },
  { category: 'budget_friendly', label: 'Budget Friendly', emoji: '💰' },
  { category: 'family_friendly', label: 'Family Friendly', emoji: '👨‍👩‍👧‍👦' },
];

export const CategoryDiscoverySection: React.FC<CategoryDiscoverySectionProps> = ({
  selectedCategories,
  onCategoryToggle,
  onClearCategories,
}) => {
  return (
    <View style={styles.container}>
      <View style={styles.header}>
        <Text style={styles.title}>Discover by Category</Text>
        {selectedCategories.length > 0 && (
          <Text style={styles.clearButton} onPress={onClearCategories}>
            Clear All
          </Text>
        )}
      </View>
      
      <ScrollView
        horizontal
        showsHorizontalScrollIndicator={false}
        contentContainerStyle={styles.scrollContent}
      >
        {CATEGORY_OPTIONS.map(({ category, label, emoji }) => (
          <View key={category} style={styles.categoryItem}>
            <CategoryChip
              category={category}
              label={`${emoji} ${label}`}
              isSelected={selectedCategories.includes(category)}
              onPress={onCategoryToggle}
            />
          </View>
        ))}
      </ScrollView>
      
      {selectedCategories.length > 0 && (
        <View style={styles.selectedInfo}>
          <Text style={styles.selectedText}>
            {selectedCategories.length} category{selectedCategories.length > 1 ? 'ies' : 'y'} selected
          </Text>
        </View>
      )}
    </View>
  );
};

const styles = StyleSheet.create({
  container: {
    backgroundColor: '#fff',
    paddingVertical: 16,
    borderBottomWidth: 1,
    borderBottomColor: '#f0f0f0',
  },
  header: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    paddingHorizontal: 20,
    marginBottom: 12,
  },
  title: {
    fontSize: 18,
    fontWeight: '600',
    color: '#333',
  },
  clearButton: {
    fontSize: 14,
    color: '#007AFF',
    fontWeight: '600',
  },
  scrollContent: {
    paddingHorizontal: 20,
  },
  categoryItem: {
    marginRight: 0, // CategoryChip handles its own margin
  },
  selectedInfo: {
    paddingHorizontal: 20,
    marginTop: 8,
  },
  selectedText: {
    fontSize: 12,
    color: '#666',
    fontStyle: 'italic',
  },
});
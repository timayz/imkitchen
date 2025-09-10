import React from 'react';
import {
  View,
  Text,
  TouchableOpacity,
  StyleSheet,
} from 'react-native';
import type { GrocerySectionHeaderProps } from '../../types/shopping';

export const GrocerySectionHeader: React.FC<GrocerySectionHeaderProps> = ({
  category,
  itemCount,
  completedCount,
  isExpanded,
  onToggleExpanded,
}) => {
  const getCategoryDisplay = (category: string) => {
    switch (category) {
      case 'produce':
        return { name: 'Produce', icon: '🥕', color: '#28a745' };
      case 'dairy':
        return { name: 'Dairy', icon: '🥛', color: '#007bff' };
      case 'protein':
        return { name: 'Protein', icon: '🍗', color: '#dc3545' };
      case 'pantry':
        return { name: 'Pantry', icon: '🏺', color: '#fd7e14' };
      case 'other':
        return { name: 'Other', icon: '📦', color: '#6c757d' };
      default:
        return { name: category.charAt(0).toUpperCase() + category.slice(1), icon: '📦', color: '#6c757d' };
    }
  };

  const categoryDisplay = getCategoryDisplay(category);
  const completionPercentage = itemCount > 0 ? Math.round((completedCount / itemCount) * 100) : 0;
  const isCompleted = completedCount === itemCount && itemCount > 0;

  return (
    <TouchableOpacity
      style={[
        styles.container,
        isCompleted && styles.completedContainer,
      ]}
      onPress={onToggleExpanded}
      activeOpacity={0.7}
    >
      {/* Left side: Category info */}
      <View style={styles.leftContent}>
        <View style={styles.categoryHeader}>
          <Text style={styles.categoryIcon}>{categoryDisplay.icon}</Text>
          <Text style={[
            styles.categoryName,
            isCompleted && styles.completedText,
          ]}>
            {categoryDisplay.name}
          </Text>
          
          {/* Completion badge */}
          {isCompleted && (
            <View style={styles.completedBadge}>
              <Text style={styles.completedBadgeText}>✓</Text>
            </View>
          )}
        </View>

        {/* Progress info */}
        <View style={styles.progressInfo}>
          <Text style={[
            styles.progressText,
            isCompleted && styles.completedText,
          ]}>
            {completedCount} of {itemCount} items
          </Text>
          
          {completionPercentage > 0 && (
            <Text style={[
              styles.percentageText,
              { color: isCompleted ? '#28a745' : categoryDisplay.color },
            ]}>
              {completionPercentage}%
            </Text>
          )}
        </View>

        {/* Progress bar */}
        <View style={styles.progressBarContainer}>
          <View style={styles.progressBarBackground}>
            <View
              style={[
                styles.progressBarFill,
                {
                  width: `${completionPercentage}%`,
                  backgroundColor: isCompleted ? '#28a745' : categoryDisplay.color,
                },
              ]}
            />
          </View>
        </View>
      </View>

      {/* Right side: Expand/collapse indicator */}
      <View style={styles.rightContent}>
        <Text style={[
          styles.expandIcon,
          isExpanded && styles.expandedIcon,
        ]}>
          ▼
        </Text>
      </View>
    </TouchableOpacity>
  );
};

const styles = StyleSheet.create({
  container: {
    backgroundColor: '#ffffff',
    paddingHorizontal: 16,
    paddingVertical: 14,
    borderBottomWidth: 1,
    borderBottomColor: '#e9ecef',
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    shadowColor: '#000',
    shadowOffset: {
      width: 0,
      height: 1,
    },
    shadowOpacity: 0.05,
    shadowRadius: 2,
    elevation: 1,
  },
  completedContainer: {
    backgroundColor: '#f8fff9',
  },
  leftContent: {
    flex: 1,
  },
  categoryHeader: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 4,
  },
  categoryIcon: {
    fontSize: 20,
    marginRight: 8,
  },
  categoryName: {
    fontSize: 18,
    fontWeight: '600',
    color: '#2d3436',
    flex: 1,
  },
  completedText: {
    color: '#28a745',
  },
  completedBadge: {
    backgroundColor: '#28a745',
    borderRadius: 10,
    width: 20,
    height: 20,
    alignItems: 'center',
    justifyContent: 'center',
    marginLeft: 8,
  },
  completedBadgeText: {
    color: '#ffffff',
    fontSize: 12,
    fontWeight: '600',
  },
  progressInfo: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 6,
  },
  progressText: {
    fontSize: 14,
    color: '#636e72',
    flex: 1,
  },
  percentageText: {
    fontSize: 14,
    fontWeight: '600',
  },
  progressBarContainer: {
    marginTop: 2,
  },
  progressBarBackground: {
    height: 4,
    backgroundColor: '#e9ecef',
    borderRadius: 2,
    overflow: 'hidden',
  },
  progressBarFill: {
    height: '100%',
    borderRadius: 2,
  },
  rightContent: {
    alignItems: 'center',
    justifyContent: 'center',
    paddingLeft: 12,
  },
  expandIcon: {
    fontSize: 14,
    color: '#6c757d',
    transform: [{ rotate: '0deg' }],
  },
  expandedIcon: {
    transform: [{ rotate: '180deg' }],
  },
});
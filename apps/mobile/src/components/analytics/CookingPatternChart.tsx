import React from 'react';
import { View, Text, StyleSheet, ScrollView, Dimensions } from 'react-native';
import type { ComplexityTrendData } from '../../types/analytics';

interface CookingPatternChartProps {
  complexityDistribution: Record<string, number>;
  complexityTrends: ComplexityTrendData[];
}

const { width } = Dimensions.get('window');

export const CookingPatternChart: React.FC<CookingPatternChartProps> = ({
  complexityDistribution,
  complexityTrends,
}) => {
  const complexityLevels = ['Very Easy', 'Easy', 'Medium', 'Hard', 'Very Hard'];
  const complexityColors = ['#4CAF50', '#8BC34A', '#FF9800', '#FF5722', '#9C27B0'];

  const renderComplexityDistribution = () => {
    const total = Object.values(complexityDistribution).reduce((sum, count) => sum + count, 0);
    
    return (
      <View style={styles.section}>
        <Text style={styles.sectionTitle}>Recipe Complexity Distribution</Text>
        <View style={styles.distributionContainer}>
          {complexityLevels.map((level, index) => {
            const count = complexityDistribution[level] || 0;
            const percentage = total > 0 ? (count / total) * 100 : 0;
            
            return (
              <View key={level} style={styles.distributionItem}>
                <View style={styles.distributionHeader}>
                  <View style={[styles.colorIndicator, { backgroundColor: complexityColors[index] }]} />
                  <Text style={styles.distributionLabel}>{level}</Text>
                  <Text style={styles.distributionPercentage}>{Math.round(percentage)}%</Text>
                </View>
                <View style={styles.distributionBar}>
                  <View 
                    style={[
                      styles.distributionFill, 
                      { 
                        width: `${percentage}%`,
                        backgroundColor: complexityColors[index]
                      }
                    ]} 
                  />
                </View>
                <Text style={styles.distributionCount}>{count} recipes</Text>
              </View>
            );
          })}
        </View>
      </View>
    );
  };

  const renderTrendChart = () => {
    if (complexityTrends.length === 0) return null;

    const maxComplexity = Math.max(...complexityTrends.map(t => t.averageComplexity));
    const maxPrepTime = Math.max(...complexityTrends.map(t => t.prepTimeMinutes));

    return (
      <View style={styles.section}>
        <Text style={styles.sectionTitle}>Weekly Cooking Trends</Text>
        <ScrollView horizontal showsHorizontalScrollIndicator={false}>
          <View style={styles.trendsContainer}>
            <View style={styles.trendsChart}>
              {complexityTrends.map((trend, index) => (
                <View key={trend.week} style={styles.trendColumn}>
                  <View style={styles.trendBars}>
                    {/* Complexity bar */}
                    <View 
                      style={[
                        styles.complexityBar,
                        { 
                          height: (trend.averageComplexity / maxComplexity) * 80,
                          backgroundColor: '#3498db'
                        }
                      ]} 
                    />
                    {/* Prep time bar */}
                    <View 
                      style={[
                        styles.prepTimeBar,
                        { 
                          height: (trend.prepTimeMinutes / maxPrepTime) * 80,
                          backgroundColor: '#e74c3c'
                        }
                      ]} 
                    />
                  </View>
                  <Text style={styles.weekLabel}>{trend.week}</Text>
                  <Text style={styles.recipeCount}>{trend.recipeCount}</Text>
                </View>
              ))}
            </View>
            
            <View style={styles.legend}>
              <View style={styles.legendItem}>
                <View style={[styles.legendColor, { backgroundColor: '#3498db' }]} />
                <Text style={styles.legendText}>Avg Complexity</Text>
              </View>
              <View style={styles.legendItem}>
                <View style={[styles.legendColor, { backgroundColor: '#e74c3c' }]} />
                <Text style={styles.legendText}>Prep Time (min)</Text>
              </View>
            </View>
          </View>
        </ScrollView>
      </View>
    );
  };

  return (
    <View style={styles.container}>
      <View style={styles.header}>
        <Text style={styles.title}>Cooking Pattern Analysis</Text>
        <Text style={styles.subtitle}>Recipe complexity and preparation trends</Text>
      </View>

      {renderComplexityDistribution()}
      {renderTrendChart()}
    </View>
  );
};

const styles = StyleSheet.create({
  container: {
    backgroundColor: '#fff',
    margin: 16,
    borderRadius: 12,
    padding: 20,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.1,
    shadowRadius: 4,
    elevation: 3,
  },
  header: {
    marginBottom: 20,
  },
  title: {
    fontSize: 18,
    fontWeight: 'bold',
    color: '#2c3e50',
    marginBottom: 4,
  },
  subtitle: {
    fontSize: 14,
    color: '#7f8c8d',
  },
  section: {
    marginBottom: 24,
  },
  sectionTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#2c3e50',
    marginBottom: 16,
  },
  distributionContainer: {
    gap: 12,
  },
  distributionItem: {
    marginBottom: 8,
  },
  distributionHeader: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 4,
  },
  colorIndicator: {
    width: 12,
    height: 12,
    borderRadius: 2,
    marginRight: 8,
  },
  distributionLabel: {
    flex: 1,
    fontSize: 14,
    color: '#2c3e50',
  },
  distributionPercentage: {
    fontSize: 14,
    fontWeight: '600',
    color: '#2c3e50',
  },
  distributionBar: {
    height: 8,
    backgroundColor: '#f1f2f6',
    borderRadius: 4,
    marginBottom: 2,
  },
  distributionFill: {
    height: '100%',
    borderRadius: 4,
  },
  distributionCount: {
    fontSize: 12,
    color: '#7f8c8d',
  },
  trendsContainer: {
    minWidth: width - 60,
  },
  trendsChart: {
    flexDirection: 'row',
    alignItems: 'flex-end',
    height: 120,
    paddingHorizontal: 10,
    marginBottom: 16,
  },
  trendColumn: {
    alignItems: 'center',
    marginHorizontal: 8,
    width: 40,
  },
  trendBars: {
    flexDirection: 'row',
    alignItems: 'flex-end',
    height: 80,
    gap: 2,
  },
  complexityBar: {
    width: 8,
    borderRadius: 2,
  },
  prepTimeBar: {
    width: 8,
    borderRadius: 2,
  },
  weekLabel: {
    fontSize: 10,
    color: '#7f8c8d',
    marginTop: 4,
    textAlign: 'center',
  },
  recipeCount: {
    fontSize: 10,
    color: '#95a5a6',
    marginTop: 2,
  },
  legend: {
    flexDirection: 'row',
    justifyContent: 'center',
    gap: 20,
  },
  legendItem: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  legendColor: {
    width: 12,
    height: 12,
    borderRadius: 2,
    marginRight: 6,
  },
  legendText: {
    fontSize: 12,
    color: '#7f8c8d',
  },
});
import React, { useState } from 'react';
import { View, Text, StyleSheet, ScrollView, TouchableOpacity } from 'react-native';
import type { WeeklyAnalysisData } from '../../types/analytics';

interface WeeklyTrendAnalysisProps {
  weeklyPatterns: WeeklyAnalysisData[];
  selectedWeeks: number;
  onWeeksChange: (weeks: number) => void;
}

const weekOptions = [4, 8, 12, 24, 52];

export const WeeklyTrendAnalysis: React.FC<WeeklyTrendAnalysisProps> = ({
  weeklyPatterns,
  selectedWeeks,
  onWeeksChange,
}) => {
  const [selectedMetric, setSelectedMetric] = useState<'variety' | 'adherence' | 'favorites'>('variety');

  const getMetricValue = (data: WeeklyAnalysisData, metric: string) => {
    switch (metric) {
      case 'variety': return data.varietyScore;
      case 'adherence': return data.patternAdherence;
      case 'favorites': return (data.favoritesUsed / data.totalMeals) * 100;
      default: return 0;
    }
  };

  const getMetricColor = (metric: string) => {
    switch (metric) {
      case 'variety': return '#3498db';
      case 'adherence': return '#2ecc71';
      case 'favorites': return '#e74c3c';
      default: return '#95a5a6';
    }
  };

  const getMetricLabel = (metric: string) => {
    switch (metric) {
      case 'variety': return 'Variety Score';
      case 'adherence': return 'Pattern Adherence';
      case 'favorites': return 'Favorites Usage %';
      default: return '';
    }
  };

  const calculateTrend = () => {
    if (weeklyPatterns.length < 2) return null;
    
    const recent = weeklyPatterns.slice(-4);
    const earlier = weeklyPatterns.slice(-8, -4);
    
    if (earlier.length === 0 || recent.length === 0) return null;
    
    const recentAvg = recent.reduce((sum, week) => sum + getMetricValue(week, selectedMetric), 0) / recent.length;
    const earlierAvg = earlier.reduce((sum, week) => sum + getMetricValue(week, selectedMetric), 0) / earlier.length;
    
    const change = recentAvg - earlierAvg;
    const changePercent = earlierAvg > 0 ? (change / earlierAvg) * 100 : 0;
    
    return { change, changePercent, isImproving: change > 0 };
  };

  const trend = calculateTrend();
  const maxValue = Math.max(...weeklyPatterns.map(w => getMetricValue(w, selectedMetric)));
  const minValue = Math.min(...weeklyPatterns.map(w => getMetricValue(w, selectedMetric)));

  return (
    <View style={styles.container}>
      <View style={styles.header}>
        <Text style={styles.title}>Weekly Trend Analysis</Text>
        <Text style={styles.subtitle}>Track your meal planning patterns over time</Text>
      </View>

      {/* Week Selection */}
      <View style={styles.weekSelector}>
        <Text style={styles.selectorLabel}>Analysis Period:</Text>
        <ScrollView horizontal showsHorizontalScrollIndicator={false}>
          <View style={styles.weekOptions}>
            {weekOptions.map(weeks => (
              <TouchableOpacity
                key={weeks}
                style={[
                  styles.weekOption,
                  selectedWeeks === weeks && styles.weekOptionSelected
                ]}
                onPress={() => onWeeksChange(weeks)}
              >
                <Text style={[
                  styles.weekOptionText,
                  selectedWeeks === weeks && styles.weekOptionTextSelected
                ]}>
                  {weeks}w
                </Text>
              </TouchableOpacity>
            ))}
          </View>
        </ScrollView>
      </View>

      {/* Metric Selection */}
      <View style={styles.metricSelector}>
        <TouchableOpacity
          style={[
            styles.metricOption,
            selectedMetric === 'variety' && styles.metricOptionSelected
          ]}
          onPress={() => setSelectedMetric('variety')}
        >
          <Text style={[
            styles.metricOptionText,
            selectedMetric === 'variety' && styles.metricOptionTextSelected
          ]}>
            Variety
          </Text>
        </TouchableOpacity>

        <TouchableOpacity
          style={[
            styles.metricOption,
            selectedMetric === 'adherence' && styles.metricOptionSelected
          ]}
          onPress={() => setSelectedMetric('adherence')}
        >
          <Text style={[
            styles.metricOptionText,
            selectedMetric === 'adherence' && styles.metricOptionTextSelected
          ]}>
            Patterns
          </Text>
        </TouchableOpacity>

        <TouchableOpacity
          style={[
            styles.metricOption,
            selectedMetric === 'favorites' && styles.metricOptionSelected
          ]}
          onPress={() => setSelectedMetric('favorites')}
        >
          <Text style={[
            styles.metricOptionText,
            selectedMetric === 'favorites' && styles.metricOptionTextSelected
          ]}>
            Favorites
          </Text>
        </TouchableOpacity>
      </View>

      {/* Trend Summary */}
      {trend && (
        <View style={[styles.trendSummary, { backgroundColor: trend.isImproving ? '#e8f5e8' : '#fff3cd' }]}>
          <Text style={styles.trendTitle}>Recent Trend</Text>
          <Text style={[styles.trendValue, { color: trend.isImproving ? '#28a745' : '#ffc107' }]}>
            {trend.isImproving ? '↗' : '↘'} {Math.abs(trend.changePercent).toFixed(1)}%
          </Text>
          <Text style={styles.trendDescription}>
            {getMetricLabel(selectedMetric)} is {trend.isImproving ? 'improving' : 'declining'} 
            over the last 4 weeks
          </Text>
        </View>
      )}

      {/* Weekly Chart */}
      <View style={styles.chartSection}>
        <Text style={styles.chartTitle}>{getMetricLabel(selectedMetric)} Over Time</Text>
        
        {weeklyPatterns.length === 0 ? (
          <View style={styles.emptyChart}>
            <Text style={styles.emptyText}>No data available for the selected period</Text>
          </View>
        ) : (
          <ScrollView horizontal showsHorizontalScrollIndicator={false}>
            <View style={styles.chart}>
              {weeklyPatterns.map((week, index) => {
                const value = getMetricValue(week, selectedMetric);
                const height = maxValue > 0 ? (value / maxValue) * 80 : 0;
                
                return (
                  <View key={week.weekNumber} style={styles.chartBar}>
                    <Text style={styles.barValue}>
                      {Math.round(value)}{selectedMetric === 'favorites' ? '%' : ''}
                    </Text>
                    <View 
                      style={[
                        styles.bar, 
                        { 
                          height: Math.max(height, 2),
                          backgroundColor: getMetricColor(selectedMetric)
                        }
                      ]} 
                    />
                    <Text style={styles.weekLabel}>
                      W{week.weekNumber}
                    </Text>
                    <Text style={styles.weekDate}>
                      {new Date(week.weekStartDate).toLocaleDateString('en-US', { 
                        month: 'short', 
                        day: 'numeric' 
                      })}
                    </Text>
                  </View>
                );
              })}
            </View>
          </ScrollView>
        )}
      </View>

      {/* Insights */}
      {weeklyPatterns.length > 0 && (
        <View style={styles.insights}>
          <Text style={styles.insightsTitle}>Key Insights</Text>
          <View style={styles.insightsList}>
            <Text style={styles.insightItem}>
              • Average {getMetricLabel(selectedMetric).toLowerCase()}: {
                Math.round(weeklyPatterns.reduce((sum, w) => sum + getMetricValue(w, selectedMetric), 0) / weeklyPatterns.length)
              }{selectedMetric === 'favorites' ? '%' : ''}
            </Text>
            <Text style={styles.insightItem}>
              • Best week: {Math.round(maxValue)}{selectedMetric === 'favorites' ? '%' : ''}
            </Text>
            <Text style={styles.insightItem}>
              • Consistency: {Math.round(((maxValue - minValue) / maxValue) * 100)}% variation
            </Text>
          </View>
        </View>
      )}
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
  weekSelector: {
    marginBottom: 20,
  },
  selectorLabel: {
    fontSize: 14,
    fontWeight: '600',
    color: '#2c3e50',
    marginBottom: 8,
  },
  weekOptions: {
    flexDirection: 'row',
    gap: 8,
  },
  weekOption: {
    paddingHorizontal: 16,
    paddingVertical: 8,
    borderRadius: 20,
    backgroundColor: '#f8f9fa',
    borderWidth: 1,
    borderColor: '#e9ecef',
  },
  weekOptionSelected: {
    backgroundColor: '#3498db',
    borderColor: '#3498db',
  },
  weekOptionText: {
    fontSize: 14,
    color: '#6c757d',
  },
  weekOptionTextSelected: {
    color: '#fff',
    fontWeight: '600',
  },
  metricSelector: {
    flexDirection: 'row',
    marginBottom: 20,
    backgroundColor: '#f8f9fa',
    borderRadius: 8,
    padding: 4,
  },
  metricOption: {
    flex: 1,
    paddingVertical: 8,
    alignItems: 'center',
    borderRadius: 6,
  },
  metricOptionSelected: {
    backgroundColor: '#fff',
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 1 },
    shadowOpacity: 0.1,
    shadowRadius: 2,
    elevation: 1,
  },
  metricOptionText: {
    fontSize: 14,
    color: '#6c757d',
  },
  metricOptionTextSelected: {
    color: '#2c3e50',
    fontWeight: '600',
  },
  trendSummary: {
    padding: 16,
    borderRadius: 8,
    marginBottom: 20,
    flexDirection: 'row',
    alignItems: 'center',
  },
  trendTitle: {
    fontSize: 14,
    fontWeight: '600',
    color: '#2c3e50',
    marginRight: 8,
  },
  trendValue: {
    fontSize: 18,
    fontWeight: 'bold',
    marginRight: 8,
  },
  trendDescription: {
    flex: 1,
    fontSize: 12,
    color: '#5a6c7d',
  },
  chartSection: {
    marginBottom: 20,
  },
  chartTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#2c3e50',
    marginBottom: 16,
  },
  emptyChart: {
    height: 120,
    justifyContent: 'center',
    alignItems: 'center',
  },
  emptyText: {
    fontSize: 14,
    color: '#7f8c8d',
  },
  chart: {
    flexDirection: 'row',
    alignItems: 'flex-end',
    height: 120,
    paddingHorizontal: 10,
  },
  chartBar: {
    alignItems: 'center',
    marginHorizontal: 4,
    width: 40,
  },
  barValue: {
    fontSize: 10,
    color: '#2c3e50',
    marginBottom: 4,
    fontWeight: '600',
  },
  bar: {
    width: 24,
    borderRadius: 2,
    marginBottom: 4,
  },
  weekLabel: {
    fontSize: 10,
    color: '#7f8c8d',
    fontWeight: '600',
  },
  weekDate: {
    fontSize: 8,
    color: '#95a5a6',
    marginTop: 2,
  },
  insights: {
    backgroundColor: '#f8f9fa',
    padding: 16,
    borderRadius: 8,
  },
  insightsTitle: {
    fontSize: 14,
    fontWeight: '600',
    color: '#2c3e50',
    marginBottom: 8,
  },
  insightsList: {
    gap: 4,
  },
  insightItem: {
    fontSize: 13,
    color: '#5a6c7d',
    lineHeight: 18,
  },
});
import React, { useEffect, useState } from 'react';
import {
  View,
  Text,
  ScrollView,
  StyleSheet,
  RefreshControl,
  Alert,
  ActivityIndicator,
} from 'react-native';
import { VarietyScoreCard } from '../../components/analytics/VarietyScoreCard';
import { CookingPatternChart } from '../../components/analytics/CookingPatternChart';
import { FavoritesFrequencyChart } from '../../components/analytics/FavoritesFrequencyChart';
import { WeeklyTrendAnalysis } from '../../components/analytics/WeeklyTrendAnalysis';
import { AnalyticsExportButton } from '../../components/analytics/AnalyticsExportButton';
import { RotationResetButton } from '../../components/analytics/RotationResetButton';
import { analyticsService } from '../../services/analytics_service';
import type { RotationAnalytics } from '../../types/analytics';

export const RotationStatsScreen: React.FC = () => {
  const [analytics, setAnalytics] = useState<RotationAnalytics | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [isRefreshing, setIsRefreshing] = useState(false);
  const [selectedWeeks, setSelectedWeeks] = useState(12);

  const loadAnalytics = async (weeks: number = selectedWeeks, showLoader: boolean = true) => {
    if (showLoader) {
      setIsLoading(true);
    }
    
    try {
      const data = await analyticsService.getRotationAnalytics(weeks);
      setAnalytics(data);
    } catch (error) {
      console.error('Failed to load analytics:', error);
      Alert.alert(
        'Error',
        'Failed to load rotation analytics. Please try again.',
        [{ text: 'OK' }]
      );
    } finally {
      setIsLoading(false);
      setIsRefreshing(false);
    }
  };

  const handleRefresh = () => {
    setIsRefreshing(true);
    loadAnalytics(selectedWeeks, false);
  };

  const handleWeeksChange = (weeks: number) => {
    setSelectedWeeks(weeks);
    loadAnalytics(weeks);
  };

  const handleResetComplete = () => {
    // Reload analytics after reset
    loadAnalytics();
  };

  useEffect(() => {
    loadAnalytics();
  }, []);

  if (isLoading && !analytics) {
    return (
      <View style={styles.loadingContainer}>
        <ActivityIndicator size="large" color="#4CAF50" />
        <Text style={styles.loadingText}>Loading analytics...</Text>
      </View>
    );
  }

  return (
    <ScrollView
      style={styles.container}
      refreshControl={
        <RefreshControl
          refreshing={isRefreshing}
          onRefresh={handleRefresh}
          colors={['#4CAF50']}
        />
      }
    >
      <View style={styles.header}>
        <Text style={styles.title}>Rotation Analytics</Text>
        <Text style={styles.subtitle}>
          Analyzing {analytics?.weeksAnalyzed || 0} weeks of meal planning data
        </Text>
      </View>

      {analytics && (
        <>
          <VarietyScoreCard
            varietyScore={analytics.varietyScore}
            rotationEfficiency={analytics.rotationEfficiency}
            weeksAnalyzed={analytics.weeksAnalyzed}
          />

          <CookingPatternChart
            complexityDistribution={analytics.complexityDistribution}
            complexityTrends={analytics.complexityTrends}
          />

          <FavoritesFrequencyChart
            favoritesFrequency={analytics.favoritesFrequency}
            favoritesImpact={analytics.favoritesImpact}
          />

          <WeeklyTrendAnalysis
            weeklyPatterns={analytics.weeklyPatterns}
            selectedWeeks={selectedWeeks}
            onWeeksChange={handleWeeksChange}
          />

          <View style={styles.actionsContainer}>
            <AnalyticsExportButton analytics={analytics} />
            <RotationResetButton onResetComplete={handleResetComplete} />
          </View>
        </>
      )}

      <View style={styles.footer}>
        <Text style={styles.footerText}>
          Last updated: {analytics?.calculatedAt ? 
            new Date(analytics.calculatedAt).toLocaleDateString() : 'Never'}
        </Text>
      </View>
    </ScrollView>
  );
};

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#f5f5f5',
  },
  loadingContainer: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    backgroundColor: '#f5f5f5',
  },
  loadingText: {
    marginTop: 16,
    fontSize: 16,
    color: '#666',
  },
  header: {
    padding: 16,
    backgroundColor: '#fff',
    borderBottomWidth: 1,
    borderBottomColor: '#e0e0e0',
  },
  title: {
    fontSize: 24,
    fontWeight: 'bold',
    color: '#2c3e50',
    marginBottom: 4,
  },
  subtitle: {
    fontSize: 14,
    color: '#7f8c8d',
  },
  actionsContainer: {
    padding: 16,
    gap: 12,
  },
  footer: {
    padding: 16,
    alignItems: 'center',
  },
  footerText: {
    fontSize: 12,
    color: '#95a5a6',
  },
});
import React, { useState, useEffect } from 'react';
import {
  View,
  Text,
  ScrollView,
  TouchableOpacity,
  StyleSheet,
  ActivityIndicator,
  RefreshControl,
} from 'react-native';
import { format, subDays, subWeeks, subMonths } from 'date-fns';
import { AttributionService } from '../../services/attribution_service';
import type { 
  CommunityMetricsData, 
  MetricsTimeframe, 
  ContributorAchievement 
} from '@imkitchen/shared-types';

interface CommunityMetricsDashboardProps {
  recipeId?: string;
  contributorId?: string;
  showPersonalMetrics?: boolean;
  onAchievementPress?: (achievement: ContributorAchievement) => void;
  style?: any;
}

export const CommunityMetricsDashboard: React.FC<CommunityMetricsDashboardProps> = ({
  recipeId,
  contributorId,
  showPersonalMetrics = false,
  onAchievementPress,
  style,
}) => {
  const [metrics, setMetrics] = useState<CommunityMetricsData | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [refreshing, setRefreshing] = useState(false);
  const [selectedTimeframe, setSelectedTimeframe] = useState<MetricsTimeframe>('week');
  const [error, setError] = useState<string | null>(null);

  const attributionService = new AttributionService();

  useEffect(() => {
    loadMetrics();
  }, [recipeId, contributorId, selectedTimeframe]);

  const loadMetrics = async () => {
    try {
      setError(null);
      let data: CommunityMetricsData;

      if (recipeId) {
        data = await attributionService.getRecipeMetrics(recipeId, selectedTimeframe);
      } else if (contributorId) {
        data = await attributionService.getContributorMetrics(contributorId, selectedTimeframe);
      } else if (showPersonalMetrics) {
        data = await attributionService.getPersonalMetrics(selectedTimeframe);
      } else {
        data = await attributionService.getCommunityOverviewMetrics(selectedTimeframe);
      }

      setMetrics(data);
    } catch (err) {
      console.error('Failed to load metrics:', err);
      setError('Failed to load metrics. Please try again.');
    } finally {
      setIsLoading(false);
      setRefreshing(false);
    }
  };

  const handleRefresh = () => {
    setRefreshing(true);
    loadMetrics();
  };

  const handleTimeframeChange = (timeframe: MetricsTimeframe) => {
    setSelectedTimeframe(timeframe);
    setIsLoading(true);
  };

  const formatMetricValue = (value: number, type: 'count' | 'percentage' | 'rating' | 'time') => {
    switch (type) {
      case 'count':
        return value.toLocaleString();
      case 'percentage':
        return `${value.toFixed(1)}%`;
      case 'rating':
        return value.toFixed(1);
      case 'time':
        return `${Math.round(value)}min`;
      default:
        return value.toString();
    }
  };

  const getTimeframePeriod = () => {
    const now = new Date();
    switch (selectedTimeframe) {
      case 'day':
        return format(now, 'MMM d');
      case 'week':
        return `${format(subWeeks(now, 1), 'MMM d')} - ${format(now, 'MMM d')}`;
      case 'month':
        return format(now, 'MMMM yyyy');
      case 'quarter':
        return `Q${Math.floor(now.getMonth() / 3) + 1} ${now.getFullYear()}`;
      case 'year':
        return format(now, 'yyyy');
      default:
        return 'All time';
    }
  };

  const renderTimeframeSelector = () => (
    <View style={styles.timeframeSelector}>
      {['day', 'week', 'month', 'quarter', 'year'].map((timeframe) => (
        <TouchableOpacity
          key={timeframe}
          style={[
            styles.timeframeButton,
            selectedTimeframe === timeframe && styles.activeTimeframeButton,
          ]}
          onPress={() => handleTimeframeChange(timeframe as MetricsTimeframe)}
          accessibilityLabel={`Select ${timeframe} timeframe`}
          accessibilityRole="button"
        >
          <Text
            style={[
              styles.timeframeButtonText,
              selectedTimeframe === timeframe && styles.activeTimeframeButtonText,
            ]}
          >
            {timeframe.charAt(0).toUpperCase() + timeframe.slice(1)}
          </Text>
        </TouchableOpacity>
      ))}
    </View>
  );

  const renderOverviewCards = () => {
    if (!metrics?.overview) return null;

    const cards = [
      {
        title: 'Total Imports',
        value: metrics.overview.totalImports,
        type: 'count' as const,
        icon: '📥',
        trend: metrics.overview.importTrend,
      },
      {
        title: 'Average Rating',
        value: metrics.overview.averageRating,
        type: 'rating' as const,
        icon: '⭐',
        trend: metrics.overview.ratingTrend,
      },
      {
        title: 'Community Reach',
        value: metrics.overview.uniqueUsers,
        type: 'count' as const,
        icon: '👥',
        trend: metrics.overview.reachTrend,
      },
      {
        title: 'Engagement Rate',
        value: metrics.overview.engagementRate,
        type: 'percentage' as const,
        icon: '🎯',
        trend: metrics.overview.engagementTrend,
      },
    ];

    return (
      <View style={styles.overviewSection}>
        <Text style={styles.sectionTitle}>Overview</Text>
        <Text style={styles.sectionSubtitle}>{getTimeframePeriod()}</Text>
        
        <View style={styles.cardsGrid}>
          {cards.map((card, index) => (
            <View key={index} style={styles.metricCard}>
              <View style={styles.cardHeader}>
                <Text style={styles.cardIcon}>{card.icon}</Text>
                {card.trend !== undefined && (
                  <View
                    style={[
                      styles.trendIndicator,
                      card.trend > 0 ? styles.positiveeTrend : styles.negativeTrend,
                    ]}
                  >
                    <Text style={styles.trendText}>
                      {card.trend > 0 ? '↗' : '↘'} {Math.abs(card.trend).toFixed(1)}%
                    </Text>
                  </View>
                )}
              </View>
              <Text style={styles.cardValue}>
                {formatMetricValue(card.value, card.type)}
              </Text>
              <Text style={styles.cardTitle}>{card.title}</Text>
            </View>
          ))}
        </View>
      </View>
    );
  };

  const renderPopularityMetrics = () => {
    if (!metrics?.popularity) return null;

    return (
      <View style={styles.popularitySection}>
        <Text style={styles.sectionTitle}>Popularity Metrics</Text>
        
        <View style={styles.popularityGrid}>
          <View style={styles.popularityItem}>
            <Text style={styles.popularityValue}>
              #{metrics.popularity.weeklyRank}
            </Text>
            <Text style={styles.popularityLabel}>Weekly Rank</Text>
          </View>
          
          <View style={styles.popularityItem}>
            <Text style={styles.popularityValue}>
              {formatMetricValue(metrics.popularity.trendingScore, 'rating')}
            </Text>
            <Text style={styles.popularityLabel}>Trending Score</Text>
          </View>
          
          <View style={styles.popularityItem}>
            <Text style={styles.popularityValue}>
              {formatMetricValue(metrics.popularity.viralityIndex, 'rating')}
            </Text>
            <Text style={styles.popularityLabel}>Virality Index</Text>
          </View>
        </View>
        
        {metrics.popularity.featuredIn && metrics.popularity.featuredIn.length > 0 && (
          <View style={styles.featuredSection}>
            <Text style={styles.featuredTitle}>Featured In</Text>
            {metrics.popularity.featuredIn.map((feature, index) => (
              <Text key={index} style={styles.featuredItem}>
                🏆 {feature.title} ({format(new Date(feature.date), 'MMM d')})
              </Text>
            ))}
          </View>
        )}
      </View>
    );
  };

  const renderEngagementBreakdown = () => {
    if (!metrics?.engagement) return null;

    const engagementItems = [
      { label: 'Views', value: metrics.engagement.views, icon: '👀' },
      { label: 'Saves', value: metrics.engagement.saves, icon: '💾' },
      { label: 'Shares', value: metrics.engagement.shares, icon: '📤' },
      { label: 'Comments', value: metrics.engagement.comments, icon: '💬' },
      { label: 'Ratings', value: metrics.engagement.ratings, icon: '⭐' },
    ];

    return (
      <View style={styles.engagementSection}>
        <Text style={styles.sectionTitle}>Engagement Breakdown</Text>
        
        <View style={styles.engagementList}>
          {engagementItems.map((item, index) => (
            <View key={index} style={styles.engagementItem}>
              <View style={styles.engagementItemLeft}>
                <Text style={styles.engagementIcon}>{item.icon}</Text>
                <Text style={styles.engagementLabel}>{item.label}</Text>
              </View>
              <Text style={styles.engagementValue}>
                {formatMetricValue(item.value, 'count')}
              </Text>
            </View>
          ))}
        </View>
      </View>
    );
  };

  const renderAchievements = () => {
    if (!metrics?.achievements || metrics.achievements.length === 0) return null;

    return (
      <View style={styles.achievementsSection}>
        <Text style={styles.sectionTitle}>Recent Achievements</Text>
        
        <ScrollView 
          horizontal 
          showsHorizontalScrollIndicator={false}
          style={styles.achievementsList}
        >
          {metrics.achievements.map((achievement, index) => (
            <TouchableOpacity
              key={index}
              style={styles.achievementCard}
              onPress={() => onAchievementPress?.(achievement)}
              accessibilityLabel={`Achievement: ${achievement.title}`}
              accessibilityRole="button"
            >
              <Text style={styles.achievementEmoji}>{achievement.emoji}</Text>
              <Text style={styles.achievementTitle}>{achievement.title}</Text>
              <Text style={styles.achievementDate}>
                {format(new Date(achievement.earnedAt), 'MMM d')}
              </Text>
            </TouchableOpacity>
          ))}
        </ScrollView>
      </View>
    );
  };

  const renderGeographicReach = () => {
    if (!metrics?.geographic) return null;

    return (
      <View style={styles.geographicSection}>
        <Text style={styles.sectionTitle}>Geographic Reach</Text>
        
        <View style={styles.geographicStats}>
          <View style={styles.geographicItem}>
            <Text style={styles.geographicValue}>
              {metrics.geographic.countries}
            </Text>
            <Text style={styles.geographicLabel}>Countries</Text>
          </View>
          
          <View style={styles.geographicItem}>
            <Text style={styles.geographicValue}>
              {metrics.geographic.cities}
            </Text>
            <Text style={styles.geographicLabel}>Cities</Text>
          </View>
        </View>
        
        {metrics.geographic.topRegions && (
          <View style={styles.topRegions}>
            <Text style={styles.topRegionsTitle}>Top Regions</Text>
            {metrics.geographic.topRegions.slice(0, 5).map((region, index) => (
              <View key={index} style={styles.regionItem}>
                <Text style={styles.regionName}>
                  {region.flag} {region.name}
                </Text>
                <Text style={styles.regionCount}>
                  {formatMetricValue(region.count, 'count')}
                </Text>
              </View>
            ))}
          </View>
        )}
      </View>
    );
  };

  if (isLoading && !refreshing) {
    return (
      <View style={[styles.container, styles.loadingContainer, style]}>
        <ActivityIndicator size="large" color="#007AFF" />
        <Text style={styles.loadingText}>Loading metrics...</Text>
      </View>
    );
  }

  if (error) {
    return (
      <View style={[styles.container, styles.errorContainer, style]}>
        <Text style={styles.errorText}>{error}</Text>
        <TouchableOpacity style={styles.retryButton} onPress={loadMetrics}>
          <Text style={styles.retryButtonText}>Retry</Text>
        </TouchableOpacity>
      </View>
    );
  }

  return (
    <ScrollView
      style={[styles.container, style]}
      refreshControl={
        <RefreshControl refreshing={refreshing} onRefresh={handleRefresh} />
      }
    >
      {renderTimeframeSelector()}
      {renderOverviewCards()}
      {renderPopularityMetrics()}
      {renderEngagementBreakdown()}
      {renderAchievements()}
      {renderGeographicReach()}
    </ScrollView>
  );
};

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#f8f9fa',
  },
  loadingContainer: {
    justifyContent: 'center',
    alignItems: 'center',
    padding: 40,
  },
  loadingText: {
    marginTop: 16,
    fontSize: 16,
    color: '#666',
  },
  errorContainer: {
    justifyContent: 'center',
    alignItems: 'center',
    padding: 40,
  },
  errorText: {
    fontSize: 16,
    color: '#e74c3c',
    textAlign: 'center',
    marginBottom: 16,
  },
  retryButton: {
    backgroundColor: '#007AFF',
    paddingHorizontal: 20,
    paddingVertical: 10,
    borderRadius: 8,
  },
  retryButtonText: {
    color: '#fff',
    fontSize: 16,
    fontWeight: '600',
  },
  timeframeSelector: {
    flexDirection: 'row',
    backgroundColor: '#fff',
    marginBottom: 16,
    paddingHorizontal: 16,
    paddingVertical: 12,
  },
  timeframeButton: {
    flex: 1,
    paddingVertical: 8,
    paddingHorizontal: 12,
    borderRadius: 6,
    alignItems: 'center',
    marginHorizontal: 2,
  },
  activeTimeframeButton: {
    backgroundColor: '#007AFF',
  },
  timeframeButtonText: {
    fontSize: 14,
    color: '#666',
    fontWeight: '500',
  },
  activeTimeframeButtonText: {
    color: '#fff',
  },
  overviewSection: {
    backgroundColor: '#fff',
    padding: 16,
    marginBottom: 16,
  },
  sectionTitle: {
    fontSize: 20,
    fontWeight: '700',
    color: '#333',
    marginBottom: 4,
  },
  sectionSubtitle: {
    fontSize: 14,
    color: '#666',
    marginBottom: 16,
  },
  cardsGrid: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    justifyContent: 'space-between',
  },
  metricCard: {
    width: '48%',
    backgroundColor: '#f8f9fa',
    padding: 16,
    borderRadius: 12,
    marginBottom: 12,
  },
  cardHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 8,
  },
  cardIcon: {
    fontSize: 24,
  },
  trendIndicator: {
    paddingHorizontal: 6,
    paddingVertical: 2,
    borderRadius: 4,
  },
  positiveeTrend: {
    backgroundColor: '#e8f5e8',
  },
  negativeTrend: {
    backgroundColor: '#ffeaea',
  },
  trendText: {
    fontSize: 12,
    fontWeight: '600',
    color: '#4CAF50',
  },
  cardValue: {
    fontSize: 24,
    fontWeight: '700',
    color: '#333',
    marginBottom: 4,
  },
  cardTitle: {
    fontSize: 14,
    color: '#666',
  },
  popularitySection: {
    backgroundColor: '#fff',
    padding: 16,
    marginBottom: 16,
  },
  popularityGrid: {
    flexDirection: 'row',
    justifyContent: 'space-around',
    marginBottom: 16,
  },
  popularityItem: {
    alignItems: 'center',
  },
  popularityValue: {
    fontSize: 20,
    fontWeight: '700',
    color: '#007AFF',
    marginBottom: 4,
  },
  popularityLabel: {
    fontSize: 12,
    color: '#666',
  },
  featuredSection: {
    borderTopWidth: 1,
    borderTopColor: '#f0f0f0',
    paddingTop: 16,
  },
  featuredTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#333',
    marginBottom: 8,
  },
  featuredItem: {
    fontSize: 14,
    color: '#666',
    marginBottom: 4,
  },
  engagementSection: {
    backgroundColor: '#fff',
    padding: 16,
    marginBottom: 16,
  },
  engagementList: {
    marginTop: 8,
  },
  engagementItem: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    paddingVertical: 12,
    borderBottomWidth: 1,
    borderBottomColor: '#f0f0f0',
  },
  engagementItemLeft: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  engagementIcon: {
    fontSize: 18,
    marginRight: 12,
  },
  engagementLabel: {
    fontSize: 16,
    color: '#333',
  },
  engagementValue: {
    fontSize: 16,
    fontWeight: '600',
    color: '#007AFF',
  },
  achievementsSection: {
    backgroundColor: '#fff',
    padding: 16,
    marginBottom: 16,
  },
  achievementsList: {
    marginTop: 12,
  },
  achievementCard: {
    backgroundColor: '#f8f9fa',
    padding: 16,
    borderRadius: 12,
    alignItems: 'center',
    marginRight: 12,
    width: 120,
  },
  achievementEmoji: {
    fontSize: 32,
    marginBottom: 8,
  },
  achievementTitle: {
    fontSize: 14,
    fontWeight: '600',
    color: '#333',
    textAlign: 'center',
    marginBottom: 4,
  },
  achievementDate: {
    fontSize: 12,
    color: '#666',
  },
  geographicSection: {
    backgroundColor: '#fff',
    padding: 16,
    marginBottom: 16,
  },
  geographicStats: {
    flexDirection: 'row',
    justifyContent: 'space-around',
    marginBottom: 16,
  },
  geographicItem: {
    alignItems: 'center',
  },
  geographicValue: {
    fontSize: 20,
    fontWeight: '700',
    color: '#333',
    marginBottom: 4,
  },
  geographicLabel: {
    fontSize: 12,
    color: '#666',
  },
  topRegions: {
    borderTopWidth: 1,
    borderTopColor: '#f0f0f0',
    paddingTop: 16,
  },
  topRegionsTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#333',
    marginBottom: 8,
  },
  regionItem: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    paddingVertical: 8,
  },
  regionName: {
    fontSize: 14,
    color: '#333',
  },
  regionCount: {
    fontSize: 14,
    fontWeight: '600',
    color: '#666',
  },
});
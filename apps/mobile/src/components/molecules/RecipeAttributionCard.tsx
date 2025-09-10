import React from 'react';
import {
  View,
  Text,
  TouchableOpacity,
  Image,
  StyleSheet,
} from 'react-native';
import { format } from 'date-fns';
import type { RecipeAttribution, ContributorProfile } from '@imkitchen/shared-types';

interface RecipeAttributionCardProps {
  attribution: RecipeAttribution;
  contributor?: ContributorProfile;
  showFullDetails?: boolean;
  onContributorPress?: (contributorId: string) => void;
  onViewMetrics?: () => void;
  style?: any;
}

export const RecipeAttributionCard: React.FC<RecipeAttributionCardProps> = ({
  attribution,
  contributor,
  showFullDetails = false,
  onContributorPress,
  onViewMetrics,
  style,
}) => {
  const handleContributorPress = () => {
    if (onContributorPress && attribution.originalContributorId) {
      onContributorPress(attribution.originalContributorId);
    }
  };

  const renderContributorInfo = () => (
    <TouchableOpacity
      style={styles.contributorSection}
      onPress={handleContributorPress}
      disabled={!onContributorPress}
      accessibilityLabel={`View ${attribution.originalContributor}'s profile`}
      accessibilityRole="button"
    >
      {contributor?.avatarUrl ? (
        <Image 
          source={{ uri: contributor.avatarUrl }} 
          style={styles.contributorAvatar}
          accessibilityLabel={`${attribution.originalContributor}'s avatar`}
        />
      ) : (
        <View style={[styles.contributorAvatar, styles.defaultAvatar]}>
          <Text style={styles.avatarText}>
            {attribution.originalContributor.charAt(0).toUpperCase()}
          </Text>
        </View>
      )}
      
      <View style={styles.contributorDetails}>
        <Text style={styles.contributorName}>
          {attribution.originalContributor}
        </Text>
        {contributor && (
          <View style={styles.contributorStats}>
            <Text style={styles.statText}>
              {contributor.totalRecipes} recipes
            </Text>
            <Text style={styles.statSeparator}>•</Text>
            <Text style={styles.statText}>
              ⭐ {contributor.averageRating.toFixed(1)}
            </Text>
            {contributor.badges && contributor.badges.length > 0 && (
              <>
                <Text style={styles.statSeparator}>•</Text>
                <View style={styles.badgesContainer}>
                  {contributor.badges.slice(0, 2).map((badge, index) => (
                    <Text key={index} style={styles.badge}>
                      {badge.emoji}
                    </Text>
                  ))}
                  {contributor.badges.length > 2 && (
                    <Text style={styles.moreBadges}>
                      +{contributor.badges.length - 2}
                    </Text>
                  )}
                </View>
              </>
            )}
          </View>
        )}
      </View>
      
      {onContributorPress && (
        <Text style={styles.viewProfileText}>View Profile</Text>
      )}
    </TouchableOpacity>
  );

  const renderImportInfo = () => (
    <View style={styles.importSection}>
      <Text style={styles.importLabel}>Imported</Text>
      <Text style={styles.importDate}>
        {format(new Date(attribution.importDate), 'MMM d, yyyy')}
      </Text>
      {attribution.customizations && attribution.customizations.length > 0 && (
        <Text style={styles.customizationsNote}>
          with {attribution.customizations.length} customization{attribution.customizations.length !== 1 ? 's' : ''}
        </Text>
      )}
    </View>
  );

  const renderCommunityMetrics = () => (
    <View style={styles.metricsSection}>
      <Text style={styles.metricsTitle}>Community Metrics</Text>
      <View style={styles.metricsGrid}>
        <View style={styles.metricItem}>
          <Text style={styles.metricValue}>
            {attribution.communityMetrics.totalImports.toLocaleString()}
          </Text>
          <Text style={styles.metricLabel}>Imports</Text>
        </View>
        
        <View style={styles.metricItem}>
          <Text style={styles.metricValue}>
            ⭐ {attribution.communityMetrics.averageRating.toFixed(1)}
          </Text>
          <Text style={styles.metricLabel}>Rating</Text>
        </View>
        
        <View style={styles.metricItem}>
          <Text style={styles.metricValue}>
            {attribution.communityMetrics.totalRatings.toLocaleString()}
          </Text>
          <Text style={styles.metricLabel}>Reviews</Text>
        </View>
        
        {attribution.communityMetrics.trendingScore > 0 && (
          <View style={styles.metricItem}>
            <Text style={[styles.metricValue, styles.trendingValue]}>
              📈 {attribution.communityMetrics.trendingScore.toFixed(1)}
            </Text>
            <Text style={styles.metricLabel}>Trending</Text>
          </View>
        )}
      </View>
      
      {attribution.communityMetrics.popularityRank && (
        <Text style={styles.popularityRank}>
          #{attribution.communityMetrics.popularityRank} most popular this week
        </Text>
      )}
    </View>
  );

  const renderRecipeChain = () => {
    if (!attribution.recipeChain || attribution.recipeChain.length <= 1) {
      return null;
    }

    return (
      <View style={styles.chainSection}>
        <Text style={styles.chainTitle}>Recipe Chain</Text>
        <View style={styles.chainContainer}>
          {attribution.recipeChain.map((link, index) => (
            <View key={index} style={styles.chainItem}>
              <View style={styles.chainNode}>
                <Text style={styles.chainNodeText}>
                  {link.contributorName.charAt(0).toUpperCase()}
                </Text>
              </View>
              <Text style={styles.chainContributor}>
                {link.contributorName}
              </Text>
              {index < attribution.recipeChain.length - 1 && (
                <Text style={styles.chainArrow}>→</Text>
              )}
            </View>
          ))}
        </View>
        <Text style={styles.chainDescription}>
          This recipe has been adapted {attribution.recipeChain.length - 1} time{attribution.recipeChain.length > 2 ? 's' : ''}
        </Text>
      </View>
    );
  };

  const renderEngagementStats = () => {
    if (!attribution.engagementStats || !showFullDetails) {
      return null;
    }

    return (
      <View style={styles.engagementSection}>
        <Text style={styles.engagementTitle}>Community Engagement</Text>
        <View style={styles.engagementGrid}>
          <View style={styles.engagementItem}>
            <Text style={styles.engagementValue}>
              {attribution.engagementStats.weeklyViews}
            </Text>
            <Text style={styles.engagementLabel}>Views this week</Text>
          </View>
          
          <View style={styles.engagementItem}>
            <Text style={styles.engagementValue}>
              {attribution.engagementStats.savesToMealPlans}
            </Text>
            <Text style={styles.engagementLabel}>Meal plan saves</Text>
          </View>
          
          <View style={styles.engagementItem}>
            <Text style={styles.engagementValue}>
              {attribution.engagementStats.socialShares}
            </Text>
            <Text style={styles.engagementLabel}>Shares</Text>
          </View>
        </View>
      </View>
    );
  };

  return (
    <View style={[styles.container, style]}>
      <View style={styles.header}>
        <Text style={styles.title}>Recipe Attribution</Text>
        {onViewMetrics && (
          <TouchableOpacity
            onPress={onViewMetrics}
            accessibilityLabel="View detailed metrics"
            accessibilityRole="button"
          >
            <Text style={styles.viewMetricsButton}>View Details</Text>
          </TouchableOpacity>
        )}
      </View>

      {renderContributorInfo()}
      {renderImportInfo()}
      {renderCommunityMetrics()}
      
      {showFullDetails && (
        <>
          {renderRecipeChain()}
          {renderEngagementStats()}
        </>
      )}
      
      {attribution.preserveAttribution && (
        <View style={styles.attributionNote}>
          <Text style={styles.attributionText}>
            ℹ️ Attribution preserved as requested by original contributor
          </Text>
        </View>
      )}
    </View>
  );
};

const styles = StyleSheet.create({
  container: {
    backgroundColor: '#fff',
    borderRadius: 12,
    padding: 16,
    borderWidth: 1,
    borderColor: '#e0e0e0',
    marginVertical: 8,
  },
  header: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 16,
  },
  title: {
    fontSize: 18,
    fontWeight: '600',
    color: '#333',
  },
  viewMetricsButton: {
    fontSize: 14,
    color: '#007AFF',
    fontWeight: '500',
  },
  contributorSection: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 16,
    paddingBottom: 16,
    borderBottomWidth: 1,
    borderBottomColor: '#f0f0f0',
  },
  contributorAvatar: {
    width: 48,
    height: 48,
    borderRadius: 24,
    marginRight: 12,
  },
  defaultAvatar: {
    backgroundColor: '#007AFF',
    alignItems: 'center',
    justifyContent: 'center',
  },
  avatarText: {
    color: '#fff',
    fontSize: 20,
    fontWeight: '600',
  },
  contributorDetails: {
    flex: 1,
  },
  contributorName: {
    fontSize: 16,
    fontWeight: '600',
    color: '#333',
    marginBottom: 4,
  },
  contributorStats: {
    flexDirection: 'row',
    alignItems: 'center',
    flexWrap: 'wrap',
  },
  statText: {
    fontSize: 14,
    color: '#666',
  },
  statSeparator: {
    fontSize: 14,
    color: '#666',
    marginHorizontal: 6,
  },
  badgesContainer: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  badge: {
    fontSize: 14,
    marginRight: 2,
  },
  moreBadges: {
    fontSize: 12,
    color: '#666',
    marginLeft: 2,
  },
  viewProfileText: {
    fontSize: 14,
    color: '#007AFF',
    fontWeight: '500',
  },
  importSection: {
    marginBottom: 16,
    paddingBottom: 16,
    borderBottomWidth: 1,
    borderBottomColor: '#f0f0f0',
  },
  importLabel: {
    fontSize: 14,
    color: '#666',
    marginBottom: 4,
  },
  importDate: {
    fontSize: 16,
    fontWeight: '600',
    color: '#333',
  },
  customizationsNote: {
    fontSize: 12,
    color: '#999',
    marginTop: 2,
    fontStyle: 'italic',
  },
  metricsSection: {
    marginBottom: 16,
  },
  metricsTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#333',
    marginBottom: 12,
  },
  metricsGrid: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    marginBottom: 8,
  },
  metricItem: {
    alignItems: 'center',
    flex: 1,
  },
  metricValue: {
    fontSize: 18,
    fontWeight: '700',
    color: '#333',
    marginBottom: 4,
  },
  trendingValue: {
    color: '#4CAF50',
  },
  metricLabel: {
    fontSize: 12,
    color: '#666',
    textAlign: 'center',
  },
  popularityRank: {
    fontSize: 12,
    color: '#4CAF50',
    fontWeight: '500',
    textAlign: 'center',
    marginTop: 8,
  },
  chainSection: {
    marginBottom: 16,
    paddingBottom: 16,
    borderBottomWidth: 1,
    borderBottomColor: '#f0f0f0',
  },
  chainTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#333',
    marginBottom: 12,
  },
  chainContainer: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 8,
  },
  chainItem: {
    alignItems: 'center',
    marginRight: 12,
  },
  chainNode: {
    width: 32,
    height: 32,
    borderRadius: 16,
    backgroundColor: '#f0f0f0',
    alignItems: 'center',
    justifyContent: 'center',
    marginBottom: 4,
  },
  chainNodeText: {
    fontSize: 14,
    fontWeight: '600',
    color: '#666',
  },
  chainContributor: {
    fontSize: 10,
    color: '#666',
    textAlign: 'center',
    maxWidth: 60,
  },
  chainArrow: {
    fontSize: 16,
    color: '#ccc',
    marginHorizontal: 4,
  },
  chainDescription: {
    fontSize: 12,
    color: '#666',
    fontStyle: 'italic',
  },
  engagementSection: {
    marginBottom: 16,
  },
  engagementTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#333',
    marginBottom: 12,
  },
  engagementGrid: {
    flexDirection: 'row',
    justifyContent: 'space-between',
  },
  engagementItem: {
    alignItems: 'center',
    flex: 1,
  },
  engagementValue: {
    fontSize: 16,
    fontWeight: '700',
    color: '#333',
    marginBottom: 4,
  },
  engagementLabel: {
    fontSize: 12,
    color: '#666',
    textAlign: 'center',
  },
  attributionNote: {
    backgroundColor: '#f8f9fa',
    padding: 12,
    borderRadius: 8,
    marginTop: 8,
  },
  attributionText: {
    fontSize: 12,
    color: '#666',
    fontStyle: 'italic',
  },
});
import React from 'react';
import {
  View,
  Text,
  StyleSheet,
  ViewStyle,
} from 'react-native';
import { RatingStars } from '../atoms/RatingStars';
import type { RatingDistribution as RatingDistributionType } from '@imkitchen/shared-types';

interface RatingDistributionProps {
  distribution: RatingDistributionType;
  totalRatings: number;
  style?: ViewStyle;
}

export const RatingDistribution: React.FC<RatingDistributionProps> = ({
  distribution,
  totalRatings,
  style,
}) => {
  if (totalRatings === 0) {
    return (
      <View style={[styles.container, style]}>
        <Text style={styles.noRatingsText}>No ratings yet</Text>
        <Text style={styles.noRatingsSubtext}>Be the first to rate this recipe!</Text>
      </View>
    );
  }

  const getPercentage = (count: number): number => {
    return totalRatings > 0 ? Math.round((count / totalRatings) * 100) : 0;
  };

  const getBarWidth = (count: number): number => {
    const maxCount = Math.max(
      distribution.oneStar,
      distribution.twoStar,
      distribution.threeStar,
      distribution.fourStar,
      distribution.fiveStar
    );
    return maxCount > 0 ? (count / maxCount) * 100 : 0;
  };

  const ratingData = [
    { stars: 5, count: distribution.fiveStar },
    { stars: 4, count: distribution.fourStar },
    { stars: 3, count: distribution.threeStar },
    { stars: 2, count: distribution.twoStar },
    { stars: 1, count: distribution.oneStar },
  ];

  return (
    <View style={[styles.container, style]}>
      <Text style={styles.title}>Rating Breakdown</Text>
      
      <View style={styles.distributionList}>
        {ratingData.map(({ stars, count }) => {
          const percentage = getPercentage(count);
          const barWidth = getBarWidth(count);

          return (
            <View key={stars} style={styles.ratingRow}>
              {/* Star indicator */}
              <View style={styles.starContainer}>
                <Text style={styles.starNumber}>{stars}</Text>
                <Text style={styles.starSymbol}>★</Text>
              </View>

              {/* Progress bar */}
              <View style={styles.progressBarContainer}>
                <View style={styles.progressBarBackground}>
                  <View
                    style={[
                      styles.progressBar,
                      {
                        width: `${barWidth}%`,
                        backgroundColor: getProgressBarColor(stars),
                      },
                    ]}
                  />
                </View>
              </View>

              {/* Count and percentage */}
              <View style={styles.statsContainer}>
                <Text style={styles.count}>{count}</Text>
                <Text style={styles.percentage}>({percentage}%)</Text>
              </View>
            </View>
          );
        })}
      </View>

      <View style={styles.totalContainer}>
        <Text style={styles.totalText}>
          Total: {totalRatings} rating{totalRatings !== 1 ? 's' : ''}
        </Text>
      </View>
    </View>
  );
};

const getProgressBarColor = (stars: number): string => {
  switch (stars) {
    case 5:
      return '#4CAF50';
    case 4:
      return '#8BC34A';
    case 3:
      return '#FF9800';
    case 2:
      return '#FF5722';
    case 1:
      return '#F44336';
    default:
      return '#ccc';
  }
};

const styles = StyleSheet.create({
  container: {
    backgroundColor: '#fff',
    borderRadius: 12,
    padding: 16,
    shadowColor: '#000',
    shadowOffset: {
      width: 0,
      height: 2,
    },
    shadowOpacity: 0.1,
    shadowRadius: 4,
    elevation: 3,
  },
  title: {
    fontSize: 16,
    fontWeight: '600',
    color: '#333',
    marginBottom: 16,
  },
  distributionList: {
    marginBottom: 12,
  },
  ratingRow: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 8,
  },
  starContainer: {
    flexDirection: 'row',
    alignItems: 'center',
    width: 32,
    marginRight: 12,
  },
  starNumber: {
    fontSize: 14,
    fontWeight: '600',
    color: '#333',
    marginRight: 2,
  },
  starSymbol: {
    fontSize: 14,
    color: '#FFD700',
  },
  progressBarContainer: {
    flex: 1,
    marginRight: 12,
  },
  progressBarBackground: {
    height: 8,
    backgroundColor: '#f0f0f0',
    borderRadius: 4,
    overflow: 'hidden',
  },
  progressBar: {
    height: '100%',
    borderRadius: 4,
    minWidth: 2,
  },
  statsContainer: {
    flexDirection: 'row',
    alignItems: 'center',
    minWidth: 60,
  },
  count: {
    fontSize: 14,
    fontWeight: '600',
    color: '#333',
    marginRight: 4,
  },
  percentage: {
    fontSize: 12,
    color: '#666',
  },
  totalContainer: {
    paddingTop: 12,
    borderTopWidth: 1,
    borderTopColor: '#f0f0f0',
    alignItems: 'center',
  },
  totalText: {
    fontSize: 14,
    color: '#666',
  },
  noRatingsText: {
    fontSize: 16,
    fontWeight: '600',
    color: '#333',
    textAlign: 'center',
    marginBottom: 4,
  },
  noRatingsSubtext: {
    fontSize: 14,
    color: '#666',
    textAlign: 'center',
  },
});
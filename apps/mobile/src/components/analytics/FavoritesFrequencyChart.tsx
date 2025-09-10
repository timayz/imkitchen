import React from 'react';
import { View, Text, StyleSheet, ScrollView } from 'react-native';

interface FavoritesFrequencyChartProps {
  favoritesFrequency: Record<string, number>;
  favoritesImpact: number;
}

export const FavoritesFrequencyChart: React.FC<FavoritesFrequencyChartProps> = ({
  favoritesFrequency,
  favoritesImpact,
}) => {
  const sortedFavorites = Object.entries(favoritesFrequency)
    .sort(([, a], [, b]) => b - a)
    .slice(0, 10); // Show top 10 favorites

  const maxFrequency = Math.max(...Object.values(favoritesFrequency));
  const totalUses = Object.values(favoritesFrequency).reduce((sum, count) => sum + count, 0);

  const getImpactColor = (impact: number) => {
    if (impact >= 0.7) return '#4CAF50'; // High impact - good variety balance
    if (impact >= 0.4) return '#FF9800'; // Medium impact
    return '#F44336'; // Low impact - might be over-using favorites
  };

  const getImpactLabel = (impact: number) => {
    if (impact >= 0.7) return 'Balanced';
    if (impact >= 0.4) return 'Moderate';
    return 'Heavy Usage';
  };

  return (
    <View style={styles.container}>
      <View style={styles.header}>
        <Text style={styles.title}>Favorite Recipe Usage</Text>
        <Text style={styles.subtitle}>How often your favorite recipes appear in meal plans</Text>
      </View>

      <View style={styles.impactSection}>
        <View style={styles.impactContainer}>
          <Text style={styles.impactLabel}>Favorites Impact</Text>
          <View style={styles.impactValue}>
            <Text style={[styles.impactNumber, { color: getImpactColor(favoritesImpact) }]}>
              {Math.round(favoritesImpact * 100)}%
            </Text>
            <Text style={[styles.impactStatus, { color: getImpactColor(favoritesImpact) }]}>
              {getImpactLabel(favoritesImpact)}
            </Text>
          </View>
        </View>
        <Text style={styles.impactDescription}>
          {favoritesImpact >= 0.7 
            ? 'Good balance between favorites and variety exploration'
            : favoritesImpact >= 0.4
            ? 'Moderate use of favorites with room for more variety'
            : 'High reliance on favorites - consider exploring new recipes'}
        </Text>
      </View>

      <View style={styles.frequencySection}>
        <Text style={styles.sectionTitle}>Top Used Favorites</Text>
        {sortedFavorites.length === 0 ? (
          <View style={styles.emptyState}>
            <Text style={styles.emptyText}>No favorite recipes tracked yet</Text>
            <Text style={styles.emptySubtext}>
              Mark recipes as favorites to see usage patterns here
            </Text>
          </View>
        ) : (
          <ScrollView style={styles.frequencyList} showsVerticalScrollIndicator={false}>
            {sortedFavorites.map(([recipeName, frequency], index) => {
              const percentage = totalUses > 0 ? (frequency / totalUses) * 100 : 0;
              const barWidth = maxFrequency > 0 ? (frequency / maxFrequency) * 100 : 0;
              
              return (
                <View key={recipeName} style={styles.frequencyItem}>
                  <View style={styles.frequencyHeader}>
                    <View style={styles.rankContainer}>
                      <Text style={styles.rank}>#{index + 1}</Text>
                    </View>
                    <View style={styles.recipeInfo}>
                      <Text style={styles.recipeName} numberOfLines={1}>
                        {recipeName}
                      </Text>
                      <Text style={styles.usageStats}>
                        {frequency} times ({Math.round(percentage)}% of favorites)
                      </Text>
                    </View>
                    <Text style={styles.frequencyCount}>{frequency}</Text>
                  </View>
                  
                  <View style={styles.frequencyBar}>
                    <View 
                      style={[
                        styles.frequencyFill, 
                        { 
                          width: `${barWidth}%`,
                          backgroundColor: index < 3 ? '#3498db' : '#95a5a6'
                        }
                      ]} 
                    />
                  </View>
                </View>
              );
            })}
          </ScrollView>
        )}
      </View>

      {totalUses > 0 && (
        <View style={styles.summary}>
          <Text style={styles.summaryText}>
            Total favorite recipe uses: {totalUses} across {Object.keys(favoritesFrequency).length} recipes
          </Text>
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
  impactSection: {
    backgroundColor: '#f8f9fa',
    padding: 16,
    borderRadius: 8,
    marginBottom: 20,
  },
  impactContainer: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 8,
  },
  impactLabel: {
    fontSize: 16,
    fontWeight: '600',
    color: '#2c3e50',
  },
  impactValue: {
    alignItems: 'flex-end',
  },
  impactNumber: {
    fontSize: 24,
    fontWeight: 'bold',
  },
  impactStatus: {
    fontSize: 12,
    fontWeight: '500',
  },
  impactDescription: {
    fontSize: 13,
    color: '#5a6c7d',
    lineHeight: 18,
  },
  frequencySection: {
    marginBottom: 16,
  },
  sectionTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#2c3e50',
    marginBottom: 16,
  },
  emptyState: {
    alignItems: 'center',
    padding: 32,
  },
  emptyText: {
    fontSize: 16,
    color: '#7f8c8d',
    marginBottom: 8,
  },
  emptySubtext: {
    fontSize: 14,
    color: '#95a5a6',
    textAlign: 'center',
  },
  frequencyList: {
    maxHeight: 300,
  },
  frequencyItem: {
    marginBottom: 16,
  },
  frequencyHeader: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 6,
  },
  rankContainer: {
    width: 30,
    alignItems: 'center',
  },
  rank: {
    fontSize: 14,
    fontWeight: 'bold',
    color: '#3498db',
  },
  recipeInfo: {
    flex: 1,
    marginLeft: 12,
  },
  recipeName: {
    fontSize: 15,
    fontWeight: '600',
    color: '#2c3e50',
    marginBottom: 2,
  },
  usageStats: {
    fontSize: 12,
    color: '#7f8c8d',
  },
  frequencyCount: {
    fontSize: 18,
    fontWeight: 'bold',
    color: '#2c3e50',
    marginLeft: 12,
  },
  frequencyBar: {
    height: 6,
    backgroundColor: '#f1f2f6',
    borderRadius: 3,
  },
  frequencyFill: {
    height: '100%',
    borderRadius: 3,
  },
  summary: {
    borderTopWidth: 1,
    borderTopColor: '#e0e0e0',
    paddingTop: 12,
    alignItems: 'center',
  },
  summaryText: {
    fontSize: 12,
    color: '#7f8c8d',
  },
});
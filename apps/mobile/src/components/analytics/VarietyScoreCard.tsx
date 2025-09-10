import React from 'react';
import { View, Text, StyleSheet } from 'react-native';

interface VarietyScoreCardProps {
  varietyScore: number;
  rotationEfficiency: number;
  weeksAnalyzed: number;
}

export const VarietyScoreCard: React.FC<VarietyScoreCardProps> = ({
  varietyScore,
  rotationEfficiency,
  weeksAnalyzed,
}) => {
  const getScoreColor = (score: number) => {
    if (score >= 80) return '#4CAF50'; // Green - Excellent
    if (score >= 60) return '#FF9800'; // Orange - Good
    return '#F44336'; // Red - Needs improvement
  };

  const getScoreLabel = (score: number) => {
    if (score >= 80) return 'Excellent';
    if (score >= 60) return 'Good';
    return 'Needs Improvement';
  };

  return (
    <View style={styles.container}>
      <View style={styles.header}>
        <Text style={styles.title}>Meal Variety Analysis</Text>
        <Text style={styles.subtitle}>{weeksAnalyzed} weeks analyzed</Text>
      </View>

      <View style={styles.metricsContainer}>
        <View style={styles.mainMetric}>
          <View style={[styles.scoreCircle, { borderColor: getScoreColor(varietyScore) }]}>
            <Text style={[styles.scoreValue, { color: getScoreColor(varietyScore) }]}>
              {Math.round(varietyScore)}
            </Text>
            <Text style={styles.scoreUnit}>%</Text>
          </View>
          <Text style={styles.mainLabel}>Variety Score</Text>
          <Text style={[styles.scoreLabel, { color: getScoreColor(varietyScore) }]}>
            {getScoreLabel(varietyScore)}
          </Text>
        </View>

        <View style={styles.secondaryMetric}>
          <Text style={styles.secondaryValue}>
            {Math.round(rotationEfficiency * 100)}%
          </Text>
          <Text style={styles.secondaryLabel}>Rotation Efficiency</Text>
          <Text style={styles.secondaryDescription}>
            How well the algorithm uses your recipe collection
          </Text>
        </View>
      </View>

      <View style={styles.explanation}>
        <Text style={styles.explanationText}>
          Variety score measures how diverse your meal planning is across complexity levels, 
          prep times, and cuisine types over time.
        </Text>
      </View>
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
  metricsContainer: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 20,
  },
  mainMetric: {
    flex: 1,
    alignItems: 'center',
  },
  scoreCircle: {
    width: 100,
    height: 100,
    borderRadius: 50,
    borderWidth: 4,
    justifyContent: 'center',
    alignItems: 'center',
    backgroundColor: '#f8f9fa',
    marginBottom: 8,
  },
  scoreValue: {
    fontSize: 28,
    fontWeight: 'bold',
  },
  scoreUnit: {
    fontSize: 14,
    color: '#7f8c8d',
    position: 'absolute',
    right: 20,
    top: 15,
  },
  mainLabel: {
    fontSize: 16,
    fontWeight: '600',
    color: '#2c3e50',
    marginBottom: 4,
  },
  scoreLabel: {
    fontSize: 14,
    fontWeight: '500',
  },
  secondaryMetric: {
    flex: 1,
    paddingLeft: 20,
    borderLeftWidth: 1,
    borderLeftColor: '#e0e0e0',
  },
  secondaryValue: {
    fontSize: 24,
    fontWeight: 'bold',
    color: '#3498db',
    marginBottom: 4,
  },
  secondaryLabel: {
    fontSize: 16,
    fontWeight: '600',
    color: '#2c3e50',
    marginBottom: 4,
  },
  secondaryDescription: {
    fontSize: 12,
    color: '#7f8c8d',
    lineHeight: 16,
  },
  explanation: {
    backgroundColor: '#f8f9fa',
    padding: 12,
    borderRadius: 8,
    borderLeftWidth: 3,
    borderLeftColor: '#3498db',
  },
  explanationText: {
    fontSize: 13,
    color: '#5a6c7d',
    lineHeight: 18,
  },
});
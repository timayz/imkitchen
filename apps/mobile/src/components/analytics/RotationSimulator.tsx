import React, { useState } from 'react';
import { View, Text, StyleSheet, ScrollView, TouchableOpacity, Alert, TextInput } from 'react-native';

interface SimulationConfig {
  favoriteRecipes: string[];
  complexityPreference: number;
  timeConstraint: number;
  dietaryRestrictions: string[];
  weeklyPattern: Record<string, boolean>;
}

interface SimulationResult {
  weekNumber: number;
  meals: Array<{
    day: string;
    recipeName: string;
    complexity: number;
    prepTime: number;
    isFavorite: boolean;
    score: number;
  }>;
  varietyScore: number;
  constraintViolations: string[];
  fallbacksUsed: number;
}

export const RotationSimulator: React.FC = () => {
  const [config, setConfig] = useState<SimulationConfig>({
    favoriteRecipes: ['Spaghetti Carbonara', 'Chicken Stir Fry', 'Veggie Burger'],
    complexityPreference: 3,
    timeConstraint: 45,
    dietaryRestrictions: [],
    weeklyPattern: {
      monday: true,
      tuesday: true,
      wednesday: false,
      thursday: true,
      friday: true,
      saturday: true,
      sunday: true,
    },
  });
  
  const [simulationResults, setSimulationResults] = useState<SimulationResult[]>([]);
  const [isSimulating, setIsSimulating] = useState(false);
  const [weeksToSimulate, setWeeksToSimulate] = useState(4);

  const runSimulation = async () => {
    if (isSimulating) return;
    
    setIsSimulating(true);
    
    try {
      // Mock simulation - in real implementation this would call the backend
      const results = generateMockSimulationResults(config, weeksToSimulate);
      setSimulationResults(results);
      
      Alert.alert(
        'Simulation Complete',
        `Generated ${weeksToSimulate} weeks of meal plans. Review the results below.`,
        [{ text: 'OK' }]
      );
    } catch (error) {
      console.error('Simulation failed:', error);
      Alert.alert(
        'Simulation Failed',
        'Failed to run rotation simulation. Please try again.',
        [{ text: 'OK' }]
      );
    } finally {
      setIsSimulating(false);
    }
  };

  const generateMockSimulationResults = (config: SimulationConfig, weeks: number): SimulationResult[] => {
    const mockRecipes = [
      { name: 'Spaghetti Carbonara', complexity: 3, prepTime: 30, isFavorite: true },
      { name: 'Chicken Stir Fry', complexity: 2, prepTime: 25, isFavorite: true },
      { name: 'Veggie Burger', complexity: 2, prepTime: 20, isFavorite: true },
      { name: 'Beef Tacos', complexity: 3, prepTime: 35, isFavorite: false },
      { name: 'Caesar Salad', complexity: 1, prepTime: 15, isFavorite: false },
      { name: 'Lasagna', complexity: 4, prepTime: 60, isFavorite: false },
      { name: 'Fish & Chips', complexity: 3, prepTime: 40, isFavorite: false },
      { name: 'Thai Curry', complexity: 4, prepTime: 45, isFavorite: false },
    ];

    const days = ['Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday', 'Sunday'];
    const availableDays = days.filter(day => config.weeklyPattern[day.toLowerCase()]);
    
    const results: SimulationResult[] = [];
    
    for (let week = 1; week <= weeks; week++) {
      const weekMeals = availableDays.map(day => {
        // Simple algorithm simulation
        const availableRecipes = mockRecipes.filter(recipe => 
          recipe.prepTime <= config.timeConstraint
        );
        
        const randomRecipe = availableRecipes[Math.floor(Math.random() * availableRecipes.length)];
        
        // Calculate mock score
        let score = 50; // Base score
        if (randomRecipe.isFavorite) score *= 1.5;
        if (Math.abs(randomRecipe.complexity - config.complexityPreference) <= 1) score += 20;
        score += Math.random() * 20 - 10; // Add some randomness
        
        return {
          day,
          recipeName: randomRecipe.name,
          complexity: randomRecipe.complexity,
          prepTime: randomRecipe.prepTime,
          isFavorite: randomRecipe.isFavorite,
          score: Math.round(score),
        };
      });

      // Calculate variety score
      const complexities = weekMeals.map(meal => meal.complexity);
      const uniqueComplexities = new Set(complexities);
      const varietyScore = Math.round((uniqueComplexities.size / 5) * 100);

      // Mock constraint violations
      const violations: string[] = [];
      if (weekMeals.some(meal => meal.prepTime > config.timeConstraint)) {
        violations.push('Time constraint exceeded');
      }

      results.push({
        weekNumber: week,
        meals: weekMeals,
        varietyScore,
        constraintViolations: violations,
        fallbacksUsed: Math.random() > 0.8 ? 1 : 0,
      });
    }

    return results;
  };

  const updateWeeklyPattern = (day: string, available: boolean) => {
    setConfig(prev => ({
      ...prev,
      weeklyPattern: {
        ...prev.weeklyPattern,
        [day]: available,
      },
    }));
  };

  const renderConfigSection = () => (
    <View style={styles.configSection}>
      <Text style={styles.sectionTitle}>Simulation Configuration</Text>
      
      {/* Weeks to Simulate */}
      <View style={styles.configItem}>
        <Text style={styles.configLabel}>Weeks to Simulate:</Text>
        <View style={styles.weeksSelector}>
          {[2, 4, 8, 12].map(weeks => (
            <TouchableOpacity
              key={weeks}
              style={[
                styles.weekOption,
                weeksToSimulate === weeks && styles.weekOptionSelected
              ]}
              onPress={() => setWeeksToSimulate(weeks)}
            >
              <Text style={[
                styles.weekOptionText,
                weeksToSimulate === weeks && styles.weekOptionTextSelected
              ]}>
                {weeks}w
              </Text>
            </TouchableOpacity>
          ))}
        </View>
      </View>

      {/* Complexity Preference */}
      <View style={styles.configItem}>
        <Text style={styles.configLabel}>Complexity Preference (1-5):</Text>
        <View style={styles.complexitySelector}>
          {[1, 2, 3, 4, 5].map(level => (
            <TouchableOpacity
              key={level}
              style={[
                styles.complexityOption,
                config.complexityPreference === level && styles.complexityOptionSelected
              ]}
              onPress={() => setConfig(prev => ({ ...prev, complexityPreference: level }))}
            >
              <Text style={[
                styles.complexityOptionText,
                config.complexityPreference === level && styles.complexityOptionTextSelected
              ]}>
                {level}
              </Text>
            </TouchableOpacity>
          ))}
        </View>
      </View>

      {/* Time Constraint */}
      <View style={styles.configItem}>
        <Text style={styles.configLabel}>Max Prep Time (minutes):</Text>
        <TextInput
          style={styles.timeInput}
          value={config.timeConstraint.toString()}
          onChangeText={(text) => {
            const time = parseInt(text) || 30;
            setConfig(prev => ({ ...prev, timeConstraint: time }));
          }}
          keyboardType="numeric"
          placeholder="45"
        />
      </View>

      {/* Weekly Pattern */}
      <View style={styles.configItem}>
        <Text style={styles.configLabel}>Cooking Days:</Text>
        <View style={styles.weeklyPattern}>
          {Object.entries(config.weeklyPattern).map(([day, available]) => (
            <TouchableOpacity
              key={day}
              style={[
                styles.dayOption,
                available && styles.dayOptionSelected
              ]}
              onPress={() => updateWeeklyPattern(day, !available)}
            >
              <Text style={[
                styles.dayOptionText,
                available && styles.dayOptionTextSelected
              ]}>
                {day.slice(0, 3)}
              </Text>
            </TouchableOpacity>
          ))}
        </View>
      </View>

      <TouchableOpacity
        style={[styles.simulateButton, isSimulating && styles.simulateButtonDisabled]}
        onPress={runSimulation}
        disabled={isSimulating}
      >
        <Text style={styles.simulateButtonText}>
          {isSimulating ? 'Simulating...' : '🎲 Run Simulation'}
        </Text>
      </TouchableOpacity>
    </View>
  );

  const renderSimulationResults = () => {
    if (simulationResults.length === 0) {
      return (
        <View style={styles.emptyResults}>
          <Text style={styles.emptyText}>No simulation results yet</Text>
          <Text style={styles.emptySubtext}>
            Configure your preferences and run a simulation to see results
          </Text>
        </View>
      );
    }

    const avgVarietyScore = simulationResults.reduce((sum, result) => sum + result.varietyScore, 0) / simulationResults.length;
    const totalViolations = simulationResults.reduce((sum, result) => sum + result.constraintViolations.length, 0);
    const totalFallbacks = simulationResults.reduce((sum, result) => sum + result.fallbacksUsed, 0);

    return (
      <View style={styles.resultsSection}>
        <Text style={styles.sectionTitle}>Simulation Results</Text>
        
        {/* Summary Stats */}
        <View style={styles.summaryStats}>
          <View style={styles.statItem}>
            <Text style={styles.statValue}>{Math.round(avgVarietyScore)}</Text>
            <Text style={styles.statLabel}>Avg Variety</Text>
          </View>
          <View style={styles.statItem}>
            <Text style={[styles.statValue, { color: totalViolations > 0 ? '#e74c3c' : '#27ae60' }]}>
              {totalViolations}
            </Text>
            <Text style={styles.statLabel}>Violations</Text>
          </View>
          <View style={styles.statItem}>
            <Text style={[styles.statValue, { color: totalFallbacks > 0 ? '#f39c12' : '#27ae60' }]}>
              {totalFallbacks}
            </Text>
            <Text style={styles.statLabel}>Fallbacks</Text>
          </View>
        </View>

        {/* Week-by-Week Results */}
        <ScrollView style={styles.weeklyResults} nestedScrollEnabled>
          {simulationResults.map(result => (
            <View key={result.weekNumber} style={styles.weekResult}>
              <View style={styles.weekHeader}>
                <Text style={styles.weekTitle}>Week {result.weekNumber}</Text>
                <Text style={styles.weekVariety}>Variety: {result.varietyScore}%</Text>
              </View>
              
              {result.meals.map(meal => (
                <View key={`${result.weekNumber}-${meal.day}`} style={styles.mealItem}>
                  <Text style={styles.mealDay}>{meal.day}:</Text>
                  <View style={styles.mealDetails}>
                    <Text style={[styles.mealName, meal.isFavorite && styles.favoriteMeal]}>
                      {meal.recipeName} {meal.isFavorite && '⭐'}
                    </Text>
                    <Text style={styles.mealInfo}>
                      C{meal.complexity} • {meal.prepTime}min • Score: {meal.score}
                    </Text>
                  </View>
                </View>
              ))}

              {result.constraintViolations.length > 0 && (
                <View style={styles.violationsContainer}>
                  <Text style={styles.violationsTitle}>⚠️ Constraint Violations:</Text>
                  {result.constraintViolations.map((violation, index) => (
                    <Text key={index} style={styles.violationText}>• {violation}</Text>
                  ))}
                </View>
              )}
            </View>
          ))}
        </ScrollView>
      </View>
    );
  };

  return (
    <ScrollView style={styles.container} showsVerticalScrollIndicator={false}>
      <View style={styles.header}>
        <Text style={styles.title}>Rotation Simulator</Text>
        <Text style={styles.subtitle}>
          Test how preference changes affect meal planning outcomes
        </Text>
      </View>

      {renderConfigSection()}
      {renderSimulationResults()}
    </ScrollView>
  );
};

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#f5f5f5',
  },
  header: {
    padding: 16,
    backgroundColor: '#fff',
    borderBottomWidth: 1,
    borderBottomColor: '#e0e0e0',
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
  configSection: {
    backgroundColor: '#fff',
    margin: 16,
    padding: 16,
    borderRadius: 8,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.1,
    shadowRadius: 4,
    elevation: 3,
  },
  sectionTitle: {
    fontSize: 16,
    fontWeight: 'bold',
    color: '#2c3e50',
    marginBottom: 16,
  },
  configItem: {
    marginBottom: 16,
  },
  configLabel: {
    fontSize: 14,
    fontWeight: '600',
    color: '#2c3e50',
    marginBottom: 8,
  },
  weeksSelector: {
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
  complexitySelector: {
    flexDirection: 'row',
    gap: 8,
  },
  complexityOption: {
    width: 40,
    height: 40,
    borderRadius: 20,
    backgroundColor: '#f8f9fa',
    borderWidth: 1,
    borderColor: '#e9ecef',
    justifyContent: 'center',
    alignItems: 'center',
  },
  complexityOptionSelected: {
    backgroundColor: '#e74c3c',
    borderColor: '#e74c3c',
  },
  complexityOptionText: {
    fontSize: 16,
    fontWeight: '600',
    color: '#6c757d',
  },
  complexityOptionTextSelected: {
    color: '#fff',
  },
  timeInput: {
    borderWidth: 1,
    borderColor: '#e0e0e0',
    borderRadius: 8,
    paddingHorizontal: 12,
    paddingVertical: 8,
    fontSize: 14,
    backgroundColor: '#f8f9fa',
    width: 80,
  },
  weeklyPattern: {
    flexDirection: 'row',
    gap: 6,
  },
  dayOption: {
    paddingHorizontal: 8,
    paddingVertical: 6,
    borderRadius: 12,
    backgroundColor: '#f8f9fa',
    borderWidth: 1,
    borderColor: '#e9ecef',
  },
  dayOptionSelected: {
    backgroundColor: '#27ae60',
    borderColor: '#27ae60',
  },
  dayOptionText: {
    fontSize: 12,
    color: '#6c757d',
  },
  dayOptionTextSelected: {
    color: '#fff',
    fontWeight: '600',
  },
  simulateButton: {
    backgroundColor: '#9b59b6',
    paddingVertical: 12,
    borderRadius: 8,
    alignItems: 'center',
    marginTop: 8,
  },
  simulateButtonDisabled: {
    backgroundColor: '#95a5a6',
  },
  simulateButtonText: {
    color: '#fff',
    fontSize: 16,
    fontWeight: '600',
  },
  emptyResults: {
    alignItems: 'center',
    padding: 32,
    margin: 16,
    backgroundColor: '#fff',
    borderRadius: 8,
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
  resultsSection: {
    backgroundColor: '#fff',
    margin: 16,
    padding: 16,
    borderRadius: 8,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.1,
    shadowRadius: 4,
    elevation: 3,
  },
  summaryStats: {
    flexDirection: 'row',
    marginBottom: 16,
    backgroundColor: '#f8f9fa',
    padding: 12,
    borderRadius: 8,
  },
  statItem: {
    flex: 1,
    alignItems: 'center',
  },
  statValue: {
    fontSize: 20,
    fontWeight: 'bold',
    color: '#3498db',
  },
  statLabel: {
    fontSize: 12,
    color: '#7f8c8d',
    marginTop: 2,
  },
  weeklyResults: {
    maxHeight: 400,
  },
  weekResult: {
    marginBottom: 16,
    backgroundColor: '#f8f9fa',
    padding: 12,
    borderRadius: 8,
  },
  weekHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 8,
  },
  weekTitle: {
    fontSize: 14,
    fontWeight: 'bold',
    color: '#2c3e50',
  },
  weekVariety: {
    fontSize: 12,
    color: '#3498db',
    fontWeight: '600',
  },
  mealItem: {
    flexDirection: 'row',
    marginBottom: 4,
    alignItems: 'flex-start',
  },
  mealDay: {
    fontSize: 12,
    fontWeight: '600',
    color: '#7f8c8d',
    width: 60,
  },
  mealDetails: {
    flex: 1,
  },
  mealName: {
    fontSize: 13,
    color: '#2c3e50',
    marginBottom: 2,
  },
  favoriteMeal: {
    fontWeight: '600',
    color: '#e74c3c',
  },
  mealInfo: {
    fontSize: 11,
    color: '#95a5a6',
  },
  violationsContainer: {
    marginTop: 8,
    padding: 8,
    backgroundColor: '#fff3cd',
    borderRadius: 4,
  },
  violationsTitle: {
    fontSize: 12,
    fontWeight: '600',
    color: '#856404',
    marginBottom: 4,
  },
  violationText: {
    fontSize: 11,
    color: '#856404',
  },
});
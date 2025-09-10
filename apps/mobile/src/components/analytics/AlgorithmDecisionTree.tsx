import React, { useState } from 'react';
import { View, Text, StyleSheet, ScrollView, TouchableOpacity } from 'react-native';

interface DecisionNode {
  id: string;
  type: 'constraint' | 'filter' | 'scoring' | 'selection' | 'fallback';
  label: string;
  description: string;
  condition?: string;
  result?: string;
  children?: DecisionNode[];
  isActive?: boolean;
}

interface AlgorithmDecisionTreeProps {
  scenario?: 'preference_heavy' | 'constraint_heavy' | 'balanced' | 'fallback';
}

export const AlgorithmDecisionTree: React.FC<AlgorithmDecisionTreeProps> = ({
  scenario = 'balanced',
}) => {
  const [expandedNodes, setExpandedNodes] = useState<Set<string>>(new Set(['root']));
  const [selectedNode, setSelectedNode] = useState<string | null>(null);

  // Decision tree structure based on rotation algorithm logic
  const getDecisionTree = (scenario: string): DecisionNode => {
    const baseTree: DecisionNode = {
      id: 'root',
      type: 'constraint',
      label: 'Meal Plan Generation',
      description: 'Start rotation algorithm execution',
      children: [
        {
          id: 'load_constraints',
          type: 'constraint',
          label: 'Load User Constraints',
          description: 'Retrieve dietary restrictions, time limits, complexity preferences',
          condition: 'user.preferences.isLoaded',
          children: [
            {
              id: 'filter_recipes',
              type: 'filter',
              label: 'Filter Available Recipes',
              description: 'Remove recipes that violate hard constraints',
              condition: 'recipes.meetsDietaryRestrictions && recipes.withinTimeLimit',
              children: [
                {
                  id: 'apply_patterns',
                  type: 'filter',
                  label: 'Apply Weekly Patterns',
                  description: 'Filter by cooking availability and meal patterns',
                  condition: 'weeklyPattern.allowsCooking(day)',
                  children: [
                    {
                      id: 'calculate_scores',
                      type: 'scoring',
                      label: 'Calculate Recipe Scores',
                      description: 'Score recipes based on preferences, variety, and favorites',
                      children: [
                        {
                          id: 'favorites_boost',
                          type: 'scoring',
                          label: 'Apply Favorites Boost',
                          description: 'Multiply score by 1.5x for favorite recipes',
                          condition: 'recipe.isFavorite',
                          isActive: scenario === 'preference_heavy',
                        },
                        {
                          id: 'variety_penalty',
                          type: 'scoring',
                          label: 'Apply Variety Penalty',
                          description: 'Reduce score for recently used recipes',
                          condition: 'recipe.wasUsedRecently',
                        },
                        {
                          id: 'complexity_matching',
                          type: 'scoring',
                          label: 'Complexity Preference Match',
                          description: 'Boost score for preferred complexity levels',
                          condition: 'recipe.complexity == user.preferredComplexity',
                        }
                      ]
                    },
                    {
                      id: 'weighted_selection',
                      type: 'selection',
                      label: 'Weighted Random Selection',
                      description: 'Select recipes using weighted randomization',
                      condition: 'scoredRecipes.length > 0',
                      children: [
                        {
                          id: 'constraint_check',
                          type: 'constraint',
                          label: 'Final Constraint Check',
                          description: 'Verify selected recipes meet all constraints',
                          children: [
                            {
                              id: 'success',
                              type: 'selection',
                              label: 'Selection Complete',
                              description: 'Successfully generated meal plan',
                              result: 'Meal plan generated successfully',
                            }
                          ]
                        }
                      ]
                    }
                  ]
                }
              ]
            }
          ]
        },
        {
          id: 'fallback_path',
          type: 'fallback',
          label: 'Constraint Violation Fallback',
          description: 'Handle cases where constraints cannot be satisfied',
          condition: 'availableRecipes.length == 0',
          isActive: scenario === 'fallback',
          children: [
            {
              id: 'relax_constraints',
              type: 'fallback',
              label: 'Relax Constraints',
              description: 'Gradually remove non-essential constraints',
              children: [
                {
                  id: 'retry_selection',
                  type: 'selection',
                  label: 'Retry Selection',
                  description: 'Attempt recipe selection with relaxed constraints',
                  result: 'Meal plan generated with relaxed constraints',
                }
              ]
            }
          ]
        }
      ]
    };

    // Activate relevant nodes based on scenario
    const activateNodesForScenario = (node: DecisionNode, scenario: string) => {
      switch (scenario) {
        case 'preference_heavy':
          if (node.id === 'favorites_boost' || node.id === 'complexity_matching') {
            node.isActive = true;
          }
          break;
        case 'constraint_heavy':
          if (node.id === 'filter_recipes' || node.id === 'constraint_check') {
            node.isActive = true;
          }
          break;
        case 'fallback':
          if (node.id === 'fallback_path' || node.id === 'relax_constraints') {
            node.isActive = true;
          }
          break;
      }

      if (node.children) {
        node.children.forEach(child => activateNodesForScenario(child, scenario));
      }
    };

    activateNodesForScenario(baseTree, scenario);
    return baseTree;
  };

  const decisionTree = getDecisionTree(scenario);

  const toggleNode = (nodeId: string) => {
    const newExpanded = new Set(expandedNodes);
    if (newExpanded.has(nodeId)) {
      newExpanded.delete(nodeId);
    } else {
      newExpanded.add(nodeId);
    }
    setExpandedNodes(newExpanded);
  };

  const getNodeColor = (type: string, isActive?: boolean) => {
    const colors = {
      constraint: isActive ? '#e74c3c' : '#f8d7da',
      filter: isActive ? '#3498db' : '#d6eaf8',
      scoring: isActive ? '#f39c12' : '#fdeaa7',
      selection: isActive ? '#27ae60' : '#d5f4e6',
      fallback: isActive ? '#9b59b6' : '#e8daef',
    };
    return colors[type as keyof typeof colors] || '#f8f9fa';
  };

  const getNodeIcon = (type: string) => {
    const icons = {
      constraint: '🚫',
      filter: '🔍',
      scoring: '📊',
      selection: '✅',
      fallback: '🔄',
    };
    return icons[type as keyof typeof icons] || '📋';
  };

  const renderDecisionNode = (node: DecisionNode, level: number = 0) => {
    const isExpanded = expandedNodes.has(node.id);
    const isSelected = selectedNode === node.id;
    const hasChildren = node.children && node.children.length > 0;

    return (
      <View key={node.id} style={[styles.nodeContainer, { marginLeft: level * 20 }]}>
        <TouchableOpacity
          style={[
            styles.nodeHeader,
            { backgroundColor: getNodeColor(node.type, node.isActive) },
            isSelected && styles.nodeSelected,
          ]}
          onPress={() => {
            setSelectedNode(isSelected ? null : node.id);
            if (hasChildren) {
              toggleNode(node.id);
            }
          }}
        >
          <View style={styles.nodeContent}>
            <Text style={styles.nodeIcon}>{getNodeIcon(node.type)}</Text>
            <View style={styles.nodeText}>
              <Text style={[styles.nodeLabel, node.isActive && styles.nodeActiveLabel]}>
                {node.label}
              </Text>
              <Text style={styles.nodeDescription} numberOfLines={1}>
                {node.description}
              </Text>
            </View>
            {hasChildren && (
              <Text style={styles.expandIcon}>
                {isExpanded ? '▼' : '▶'}
              </Text>
            )}
          </View>
        </TouchableOpacity>

        {isSelected && (
          <View style={styles.nodeDetails}>
            <Text style={styles.nodeDetailDescription}>{node.description}</Text>
            {node.condition && (
              <View style={styles.nodeDetailItem}>
                <Text style={styles.nodeDetailLabel}>Condition:</Text>
                <Text style={styles.nodeDetailValue}>{node.condition}</Text>
              </View>
            )}
            {node.result && (
              <View style={styles.nodeDetailItem}>
                <Text style={styles.nodeDetailLabel}>Result:</Text>
                <Text style={styles.nodeDetailValue}>{node.result}</Text>
              </View>
            )}
          </View>
        )}

        {isExpanded && hasChildren && (
          <View style={styles.childrenContainer}>
            {node.children!.map(child => renderDecisionNode(child, level + 1))}
          </View>
        )}
      </View>
    );
  };

  return (
    <View style={styles.container}>
      <View style={styles.header}>
        <Text style={styles.title}>Algorithm Decision Tree</Text>
        <Text style={styles.subtitle}>
          Visualizing the rotation algorithm decision process
        </Text>
      </View>

      {/* Scenario Selector */}
      <View style={styles.scenarioSelector}>
        <Text style={styles.selectorLabel}>Scenario:</Text>
        <ScrollView horizontal showsHorizontalScrollIndicator={false}>
          <View style={styles.scenarioOptions}>
            {[
              { key: 'balanced', label: 'Balanced' },
              { key: 'preference_heavy', label: 'Preference Heavy' },
              { key: 'constraint_heavy', label: 'Constraint Heavy' },
              { key: 'fallback', label: 'Fallback Mode' },
            ].map(({ key, label }) => (
              <TouchableOpacity
                key={key}
                style={[
                  styles.scenarioOption,
                  scenario === key && styles.scenarioOptionSelected
                ]}
                onPress={() => {
                  // This would be passed as a prop in real implementation
                  console.log('Scenario changed to:', key);
                }}
              >
                <Text style={[
                  styles.scenarioOptionText,
                  scenario === key && styles.scenarioOptionTextSelected
                ]}>
                  {label}
                </Text>
              </TouchableOpacity>
            ))}
          </View>
        </ScrollView>
      </View>

      {/* Legend */}
      <View style={styles.legend}>
        <Text style={styles.legendTitle}>Node Types:</Text>
        <View style={styles.legendItems}>
          {[
            { type: 'constraint', label: 'Constraints' },
            { type: 'filter', label: 'Filtering' },
            { type: 'scoring', label: 'Scoring' },
            { type: 'selection', label: 'Selection' },
            { type: 'fallback', label: 'Fallback' },
          ].map(({ type, label }) => (
            <View key={type} style={styles.legendItem}>
              <Text style={styles.legendIcon}>{getNodeIcon(type)}</Text>
              <View style={[styles.legendColor, { backgroundColor: getNodeColor(type, false) }]} />
              <Text style={styles.legendLabel}>{label}</Text>
            </View>
          ))}
        </View>
      </View>

      {/* Decision Tree */}
      <ScrollView style={styles.treeContainer} showsVerticalScrollIndicator={false}>
        {renderDecisionNode(decisionTree)}
      </ScrollView>

      <View style={styles.footer}>
        <Text style={styles.footerText}>
          Tap nodes to expand/collapse. Highlighted nodes are active in current scenario.
        </Text>
      </View>
    </View>
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
  scenarioSelector: {
    padding: 16,
    backgroundColor: '#fff',
    borderBottomWidth: 1,
    borderBottomColor: '#f1f2f6',
  },
  selectorLabel: {
    fontSize: 14,
    fontWeight: '600',
    color: '#2c3e50',
    marginBottom: 8,
  },
  scenarioOptions: {
    flexDirection: 'row',
    gap: 8,
  },
  scenarioOption: {
    paddingHorizontal: 12,
    paddingVertical: 6,
    borderRadius: 16,
    backgroundColor: '#f8f9fa',
    borderWidth: 1,
    borderColor: '#e9ecef',
  },
  scenarioOptionSelected: {
    backgroundColor: '#3498db',
    borderColor: '#3498db',
  },
  scenarioOptionText: {
    fontSize: 12,
    color: '#6c757d',
  },
  scenarioOptionTextSelected: {
    color: '#fff',
    fontWeight: '600',
  },
  legend: {
    padding: 16,
    backgroundColor: '#fff',
    borderBottomWidth: 1,
    borderBottomColor: '#f1f2f6',
  },
  legendTitle: {
    fontSize: 14,
    fontWeight: '600',
    color: '#2c3e50',
    marginBottom: 8,
  },
  legendItems: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: 12,
  },
  legendItem: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 4,
  },
  legendIcon: {
    fontSize: 12,
  },
  legendColor: {
    width: 12,
    height: 12,
    borderRadius: 2,
  },
  legendLabel: {
    fontSize: 11,
    color: '#7f8c8d',
  },
  treeContainer: {
    flex: 1,
    padding: 16,
  },
  nodeContainer: {
    marginBottom: 8,
  },
  nodeHeader: {
    borderRadius: 8,
    borderWidth: 1,
    borderColor: '#e0e0e0',
  },
  nodeSelected: {
    borderColor: '#3498db',
    borderWidth: 2,
  },
  nodeContent: {
    flexDirection: 'row',
    alignItems: 'center',
    padding: 12,
  },
  nodeIcon: {
    fontSize: 16,
    marginRight: 12,
  },
  nodeText: {
    flex: 1,
  },
  nodeLabel: {
    fontSize: 14,
    fontWeight: '600',
    color: '#2c3e50',
  },
  nodeActiveLabel: {
    color: '#fff',
    fontWeight: 'bold',
  },
  nodeDescription: {
    fontSize: 12,
    color: '#7f8c8d',
    marginTop: 2,
  },
  expandIcon: {
    fontSize: 12,
    color: '#95a5a6',
    marginLeft: 8,
  },
  nodeDetails: {
    backgroundColor: '#f8f9fa',
    padding: 12,
    borderRadius: 8,
    marginTop: 4,
    marginHorizontal: 4,
  },
  nodeDetailDescription: {
    fontSize: 13,
    color: '#2c3e50',
    marginBottom: 8,
  },
  nodeDetailItem: {
    marginBottom: 4,
  },
  nodeDetailLabel: {
    fontSize: 11,
    fontWeight: '600',
    color: '#7f8c8d',
  },
  nodeDetailValue: {
    fontSize: 12,
    color: '#2c3e50',
    fontFamily: 'monospace',
  },
  childrenContainer: {
    marginTop: 4,
    marginLeft: 16,
    borderLeftWidth: 1,
    borderLeftColor: '#e0e0e0',
    paddingLeft: 12,
  },
  footer: {
    padding: 16,
    backgroundColor: '#fff',
    borderTopWidth: 1,
    borderTopColor: '#e0e0e0',
  },
  footerText: {
    fontSize: 12,
    color: '#7f8c8d',
    textAlign: 'center',
  },
});
import React, { useState } from 'react';
import {
  View,
  Text,
  ScrollView,
  TextInput,
  TouchableOpacity,
  StyleSheet,
  Alert,
  KeyboardAvoidingView,
  Platform,
} from 'react-native';
import { SafeAreaView } from 'react-native-safe-area-context';
import { useNavigation } from '@react-navigation/native';
import type { NativeStackNavigationProp } from '@react-navigation/native-stack';
import type { CreateRecipeInput, RecipeIngredient, RecipeInstruction } from '@imkitchen/shared-types';
import { useRecipeStore } from '../../store/recipe_store';

type RecipeStackParamList = {
  RecipeList: undefined;
  AddRecipe: undefined;
};

type NavigationProp = NativeStackNavigationProp<RecipeStackParamList>;

const MEAL_TYPES = ['breakfast', 'lunch', 'dinner', 'snack'];
const COMPLEXITY_LEVELS = ['simple', 'moderate', 'complex'];
const INGREDIENT_CATEGORIES = ['produce', 'dairy', 'pantry', 'protein', 'other'];

export const AddRecipeScreen: React.FC = () => {
  const navigation = useNavigation<NavigationProp>();
  const { createRecipe, loading } = useRecipeStore();

  const [recipe, setRecipe] = useState<CreateRecipeInput>({
    title: '',
    description: '',
    prepTime: 0,
    cookTime: 0,
    mealType: [],
    complexity: 'moderate',
    servings: 4,
    ingredients: [{ name: '', amount: 0, unit: '', category: 'other' }],
    instructions: [{ stepNumber: 1, instruction: '' }],
    dietaryLabels: [],
  });

  const handleSave = async () => {
    // Validate required fields
    if (!recipe.title.trim()) {
      Alert.alert('Error', 'Recipe title is required');
      return;
    }

    if (recipe.ingredients.some(ing => !ing.name.trim())) {
      Alert.alert('Error', 'All ingredients must have a name');
      return;
    }

    if (recipe.instructions.some(inst => !inst.instruction.trim())) {
      Alert.alert('Error', 'All instruction steps must have text');
      return;
    }

    if (recipe.mealType.length === 0) {
      Alert.alert('Error', 'Please select at least one meal type');
      return;
    }

    const result = await createRecipe(recipe);
    if (result) {
      Alert.alert('Success', 'Recipe created successfully!', [
        { text: 'OK', onPress: () => navigation.goBack() },
      ]);
    }
  };

  const handleCancel = () => {
    Alert.alert(
      'Discard Changes',
      'Are you sure you want to discard your changes?',
      [
        { text: 'Cancel', style: 'cancel' },
        { text: 'Discard', style: 'destructive', onPress: () => navigation.goBack() },
      ]
    );
  };

  const updateIngredient = (index: number, field: keyof RecipeIngredient, value: any) => {
    const newIngredients = [...recipe.ingredients];
    newIngredients[index] = { ...newIngredients[index], [field]: value };
    setRecipe({ ...recipe, ingredients: newIngredients });
  };

  const addIngredient = () => {
    setRecipe({
      ...recipe,
      ingredients: [...recipe.ingredients, { name: '', amount: 0, unit: '', category: 'other' }],
    });
  };

  const removeIngredient = (index: number) => {
    if (recipe.ingredients.length > 1) {
      const newIngredients = recipe.ingredients.filter((_, i) => i !== index);
      setRecipe({ ...recipe, ingredients: newIngredients });
    }
  };

  const updateInstruction = (index: number, text: string) => {
    const newInstructions = [...recipe.instructions];
    newInstructions[index] = { ...newInstructions[index], instruction: text };
    setRecipe({ ...recipe, instructions: newInstructions });
  };

  const addInstruction = () => {
    setRecipe({
      ...recipe,
      instructions: [
        ...recipe.instructions,
        { stepNumber: recipe.instructions.length + 1, instruction: '' },
      ],
    });
  };

  const removeInstruction = (index: number) => {
    if (recipe.instructions.length > 1) {
      const newInstructions = recipe.instructions
        .filter((_, i) => i !== index)
        .map((inst, i) => ({ ...inst, stepNumber: i + 1 }));
      setRecipe({ ...recipe, instructions: newInstructions });
    }
  };

  const toggleMealType = (mealType: string) => {
    const newMealTypes = recipe.mealType.includes(mealType)
      ? recipe.mealType.filter(type => type !== mealType)
      : [...recipe.mealType, mealType];
    setRecipe({ ...recipe, mealType: newMealTypes });
  };

  return (
    <SafeAreaView style={styles.container}>
      {/* Header */}
      <View style={styles.header}>
        <TouchableOpacity onPress={handleCancel}>
          <Text style={styles.cancelButton}>Cancel</Text>
        </TouchableOpacity>
        <Text style={styles.title}>Add Recipe</Text>
        <TouchableOpacity
          onPress={handleSave}
          disabled={loading}
        >
          <Text style={[styles.saveButton, loading && styles.disabledButton]}>
            {loading ? 'Saving...' : 'Save'}
          </Text>
        </TouchableOpacity>
      </View>

      <KeyboardAvoidingView
        style={styles.content}
        behavior={Platform.OS === 'ios' ? 'padding' : 'height'}
      >
        <ScrollView style={styles.scrollView} showsVerticalScrollIndicator={false}>
          {/* Basic Info */}
          <View style={styles.section}>
            <Text style={styles.sectionTitle}>Basic Information</Text>
            
            <Text style={styles.label}>Title *</Text>
            <TextInput
              style={styles.input}
              value={recipe.title}
              onChangeText={(text) => setRecipe({ ...recipe, title: text })}
              placeholder="Enter recipe title"
              maxLength={255}
            />

            <Text style={styles.label}>Description</Text>
            <TextInput
              style={[styles.input, styles.textArea]}
              value={recipe.description || ''}
              onChangeText={(text) => setRecipe({ ...recipe, description: text })}
              placeholder="Describe your recipe (optional)"
              multiline
              numberOfLines={3}
              textAlignVertical="top"
            />
          </View>

          {/* Timing */}
          <View style={styles.section}>
            <Text style={styles.sectionTitle}>Timing & Servings</Text>
            
            <View style={styles.row}>
              <View style={styles.halfWidth}>
                <Text style={styles.label}>Prep Time (min) *</Text>
                <TextInput
                  style={styles.input}
                  value={recipe.prepTime.toString()}
                  onChangeText={(text) => setRecipe({ ...recipe, prepTime: parseInt(text) || 0 })}
                  placeholder="30"
                  keyboardType="numeric"
                />
              </View>
              
              <View style={styles.halfWidth}>
                <Text style={styles.label}>Cook Time (min) *</Text>
                <TextInput
                  style={styles.input}
                  value={recipe.cookTime.toString()}
                  onChangeText={(text) => setRecipe({ ...recipe, cookTime: parseInt(text) || 0 })}
                  placeholder="45"
                  keyboardType="numeric"
                />
              </View>
            </View>

            <Text style={styles.label}>Servings *</Text>
            <TextInput
              style={[styles.input, styles.shortInput]}
              value={recipe.servings.toString()}
              onChangeText={(text) => setRecipe({ ...recipe, servings: parseInt(text) || 1 })}
              placeholder="4"
              keyboardType="numeric"
            />
          </View>

          {/* Classification */}
          <View style={styles.section}>
            <Text style={styles.sectionTitle}>Classification</Text>
            
            <Text style={styles.label}>Meal Types *</Text>
            <View style={styles.chipContainer}>
              {MEAL_TYPES.map(type => (
                <TouchableOpacity
                  key={type}
                  style={[
                    styles.chip,
                    recipe.mealType.includes(type) && styles.chipSelected,
                  ]}
                  onPress={() => toggleMealType(type)}
                >
                  <Text style={[
                    styles.chipText,
                    recipe.mealType.includes(type) && styles.chipTextSelected,
                  ]}>
                    {type.charAt(0).toUpperCase() + type.slice(1)}
                  </Text>
                </TouchableOpacity>
              ))}
            </View>

            <Text style={styles.label}>Complexity *</Text>
            <View style={styles.chipContainer}>
              {COMPLEXITY_LEVELS.map(level => (
                <TouchableOpacity
                  key={level}
                  style={[
                    styles.chip,
                    recipe.complexity === level && styles.chipSelected,
                  ]}
                  onPress={() => setRecipe({ ...recipe, complexity: level as any })}
                >
                  <Text style={[
                    styles.chipText,
                    recipe.complexity === level && styles.chipTextSelected,
                  ]}>
                    {level.charAt(0).toUpperCase() + level.slice(1)}
                  </Text>
                </TouchableOpacity>
              ))}
            </View>
          </View>

          {/* Ingredients */}
          <View style={styles.section}>
            <View style={styles.sectionHeader}>
              <Text style={styles.sectionTitle}>Ingredients</Text>
              <TouchableOpacity onPress={addIngredient} style={styles.addButton}>
                <Text style={styles.addButtonText}>+ Add</Text>
              </TouchableOpacity>
            </View>
            
            {recipe.ingredients.map((ingredient, index) => (
              <View key={index} style={styles.ingredientRow}>
                <View style={styles.ingredientInputs}>
                  <View style={styles.amountContainer}>
                    <TextInput
                      style={[styles.input, styles.amountInput]}
                      value={ingredient.amount.toString()}
                      onChangeText={(text) => updateIngredient(index, 'amount', parseFloat(text) || 0)}
                      placeholder="1"
                      keyboardType="numeric"
                    />
                    <TextInput
                      style={[styles.input, styles.unitInput]}
                      value={ingredient.unit}
                      onChangeText={(text) => updateIngredient(index, 'unit', text)}
                      placeholder="cup"
                    />
                  </View>
                  <TextInput
                    style={[styles.input, styles.nameInput]}
                    value={ingredient.name}
                    onChangeText={(text) => updateIngredient(index, 'name', text)}
                    placeholder="Ingredient name"
                  />
                </View>
                {recipe.ingredients.length > 1 && (
                  <TouchableOpacity
                    onPress={() => removeIngredient(index)}
                    style={styles.removeButton}
                  >
                    <Text style={styles.removeButtonText}>×</Text>
                  </TouchableOpacity>
                )}
              </View>
            ))}
          </View>

          {/* Instructions */}
          <View style={styles.section}>
            <View style={styles.sectionHeader}>
              <Text style={styles.sectionTitle}>Instructions</Text>
              <TouchableOpacity onPress={addInstruction} style={styles.addButton}>
                <Text style={styles.addButtonText}>+ Add</Text>
              </TouchableOpacity>
            </View>
            
            {recipe.instructions.map((instruction, index) => (
              <View key={index} style={styles.instructionRow}>
                <View style={styles.stepNumber}>
                  <Text style={styles.stepNumberText}>{instruction.stepNumber}</Text>
                </View>
                <TextInput
                  style={[styles.input, styles.instructionInput]}
                  value={instruction.instruction}
                  onChangeText={(text) => updateInstruction(index, text)}
                  placeholder="Describe this step..."
                  multiline
                  textAlignVertical="top"
                />
                {recipe.instructions.length > 1 && (
                  <TouchableOpacity
                    onPress={() => removeInstruction(index)}
                    style={styles.removeButton}
                  >
                    <Text style={styles.removeButtonText}>×</Text>
                  </TouchableOpacity>
                )}
              </View>
            ))}
          </View>
        </ScrollView>
      </KeyboardAvoidingView>
    </SafeAreaView>
  );
};

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#fff',
  },
  header: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    paddingHorizontal: 16,
    paddingVertical: 12,
    borderBottomWidth: 1,
    borderBottomColor: '#e0e0e0',
  },
  cancelButton: {
    color: '#FF3B30',
    fontSize: 16,
  },
  title: {
    fontSize: 18,
    fontWeight: '600',
    color: '#333',
  },
  saveButton: {
    color: '#007AFF',
    fontSize: 16,
    fontWeight: '600',
  },
  disabledButton: {
    opacity: 0.5,
  },
  content: {
    flex: 1,
  },
  scrollView: {
    flex: 1,
  },
  section: {
    paddingHorizontal: 16,
    paddingVertical: 20,
    borderBottomWidth: 1,
    borderBottomColor: '#f0f0f0',
  },
  sectionHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 16,
  },
  sectionTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#333',
    marginBottom: 16,
  },
  label: {
    fontSize: 16,
    fontWeight: '500',
    color: '#333',
    marginBottom: 8,
    marginTop: 12,
  },
  input: {
    height: 44,
    paddingHorizontal: 12,
    backgroundColor: '#f8f8f8',
    borderRadius: 8,
    fontSize: 16,
    color: '#333',
  },
  textArea: {
    height: 80,
    paddingTop: 12,
  },
  shortInput: {
    width: 80,
  },
  row: {
    flexDirection: 'row',
    gap: 12,
  },
  halfWidth: {
    flex: 1,
  },
  chipContainer: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: 8,
    marginTop: 8,
  },
  chip: {
    paddingHorizontal: 16,
    paddingVertical: 8,
    backgroundColor: '#f0f0f0',
    borderRadius: 20,
  },
  chipSelected: {
    backgroundColor: '#007AFF',
  },
  chipText: {
    fontSize: 14,
    color: '#333',
    fontWeight: '500',
  },
  chipTextSelected: {
    color: '#fff',
  },
  addButton: {
    paddingHorizontal: 12,
    paddingVertical: 6,
    backgroundColor: '#007AFF',
    borderRadius: 6,
  },
  addButtonText: {
    color: '#fff',
    fontSize: 14,
    fontWeight: '600',
  },
  ingredientRow: {
    flexDirection: 'row',
    alignItems: 'flex-start',
    marginBottom: 12,
  },
  ingredientInputs: {
    flex: 1,
    gap: 8,
  },
  amountContainer: {
    flexDirection: 'row',
    gap: 8,
  },
  amountInput: {
    flex: 1,
  },
  unitInput: {
    flex: 1,
  },
  nameInput: {
    flex: 1,
  },
  instructionRow: {
    flexDirection: 'row',
    alignItems: 'flex-start',
    marginBottom: 12,
    gap: 12,
  },
  stepNumber: {
    width: 32,
    height: 32,
    borderRadius: 16,
    backgroundColor: '#007AFF',
    justifyContent: 'center',
    alignItems: 'center',
    marginTop: 6,
  },
  stepNumberText: {
    color: '#fff',
    fontSize: 14,
    fontWeight: '600',
  },
  instructionInput: {
    flex: 1,
    height: 60,
    paddingTop: 12,
  },
  removeButton: {
    width: 32,
    height: 32,
    borderRadius: 16,
    backgroundColor: '#FF3B30',
    justifyContent: 'center',
    alignItems: 'center',
    marginTop: 6,
    marginLeft: 8,
  },
  removeButtonText: {
    color: '#fff',
    fontSize: 18,
    fontWeight: '600',
  },
});
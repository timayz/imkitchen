import React, { useState } from 'react';
import {
  View,
  Text,
  TextInput,
  TouchableOpacity,
  StyleSheet,
  Alert,
  ScrollView,
  KeyboardAvoidingView,
  Platform,
  ActivityIndicator,
} from 'react-native';
import { SafeAreaView } from 'react-native-safe-area-context';
import { useNavigation } from '@react-navigation/native';
import type { NativeStackNavigationProp } from '@react-navigation/native-stack';
import type { ImportRecipeInput } from '@imkitchen/shared-types';
import { useRecipeStore } from '../../store/recipe_store';

type RecipeStackParamList = {
  RecipeList: undefined;
  ImportRecipe: undefined;
  RecipeDetail: { recipeId: string };
};

type NavigationProp = NativeStackNavigationProp<RecipeStackParamList>;

const POPULAR_RECIPE_SITES = [
  'allrecipes.com',
  'foodnetwork.com',
  'epicurious.com',
  'seriouseats.com',
  'tasty.co',
  'delish.com',
  'simplyrecipes.com',
];

export const ImportRecipeScreen: React.FC = () => {
  const navigation = useNavigation<NavigationProp>();
  const { importRecipe, loading } = useRecipeStore();

  const [url, setUrl] = useState('');
  const [isImporting, setIsImporting] = useState(false);

  const validateUrl = (urlString: string): boolean => {
    try {
      const parsed = new URL(urlString);
      return parsed.protocol === 'http:' || parsed.protocol === 'https:';
    } catch {
      return false;
    }
  };

  const handleImport = async () => {
    if (!url.trim()) {
      Alert.alert('Error', 'Please enter a recipe URL');
      return;
    }

    if (!validateUrl(url.trim())) {
      Alert.alert('Error', 'Please enter a valid URL (including http:// or https://)');
      return;
    }

    setIsImporting(true);

    const importInput: ImportRecipeInput = {
      url: url.trim(),
    };

    try {
      const result = await importRecipe(importInput);
      
      if (result?.success && result.recipe) {
        let message = 'Recipe imported successfully!';
        if (result.warnings && result.warnings.length > 0) {
          message += `\n\nWarnings:\n${result.warnings.join('\n')}`;
        }
        
        Alert.alert('Success', message, [
          {
            text: 'View Recipe',
            onPress: () => {
              navigation.replace('RecipeDetail', { recipeId: result.recipe!.id });
            },
          },
          {
            text: 'Import Another',
            onPress: () => {
              setUrl('');
            },
          },
        ]);
      } else {
        Alert.alert(
          'Import Failed',
          result?.error || 'Could not import recipe from this URL. You can try:\n\n• Making sure the URL is correct\n• Checking if the website contains recipe data\n• Manually adding the recipe instead',
          [
            { text: 'Try Again' },
            {
              text: 'Add Manually',
              onPress: () => navigation.navigate('AddRecipe'),
            },
          ]
        );
      }
    } catch (error) {
      Alert.alert(
        'Import Error',
        'An error occurred while importing the recipe. Please check your internet connection and try again.',
        [{ text: 'OK' }]
      );
    } finally {
      setIsImporting(false);
    }
  };

  const handleCancel = () => {
    navigation.goBack();
  };

  const fillExampleUrl = (siteUrl: string) => {
    setUrl(`https://${siteUrl}/recipe-example`);
  };

  return (
    <SafeAreaView style={styles.container}>
      {/* Header */}
      <View style={styles.header}>
        <TouchableOpacity onPress={handleCancel}>
          <Text style={styles.cancelButton}>Cancel</Text>
        </TouchableOpacity>
        <Text style={styles.title}>Import Recipe</Text>
        <TouchableOpacity
          onPress={handleImport}
          disabled={isImporting || !url.trim()}
        >
          <Text style={[
            styles.importButton,
            (isImporting || !url.trim()) && styles.disabledButton
          ]}>
            {isImporting ? 'Importing...' : 'Import'}
          </Text>
        </TouchableOpacity>
      </View>

      <KeyboardAvoidingView
        style={styles.content}
        behavior={Platform.OS === 'ios' ? 'padding' : 'height'}
      >
        <ScrollView style={styles.scrollView} showsVerticalScrollIndicator={false}>
          {/* Instructions */}
          <View style={styles.instructionsSection}>
            <Text style={styles.instructionsTitle}>How it works</Text>
            <Text style={styles.instructionsText}>
              Paste a URL from a recipe website below. We'll automatically extract the recipe details including ingredients, instructions, and cooking times.
            </Text>
          </View>

          {/* URL Input */}
          <View style={styles.inputSection}>
            <Text style={styles.label}>Recipe URL</Text>
            <TextInput
              style={styles.urlInput}
              value={url}
              onChangeText={setUrl}
              placeholder="https://example.com/recipe-name"
              placeholderTextColor="#999"
              keyboardType="url"
              autoCapitalize="none"
              autoCorrect={false}
              editable={!isImporting}
            />
            
            {isImporting && (
              <View style={styles.importingIndicator}>
                <ActivityIndicator size="small" color="#007AFF" />
                <Text style={styles.importingText}>Importing recipe...</Text>
              </View>
            )}
          </View>

          {/* Popular Sites */}
          <View style={styles.sitesSection}>
            <Text style={styles.sitesTitle}>Popular Recipe Sites</Text>
            <Text style={styles.sitesSubtitle}>
              Import works best with these popular recipe websites:
            </Text>
            
            <View style={styles.sitesList}>
              {POPULAR_RECIPE_SITES.map((site, index) => (
                <TouchableOpacity
                  key={index}
                  style={styles.siteItem}
                  onPress={() => fillExampleUrl(site)}
                  disabled={isImporting}
                >
                  <Text style={styles.siteText}>{site}</Text>
                  <Text style={styles.siteArrow}>→</Text>
                </TouchableOpacity>
              ))}
            </View>
          </View>

          {/* Tips */}
          <View style={styles.tipsSection}>
            <Text style={styles.tipsTitle}>Tips for better imports</Text>
            <View style={styles.tipsList}>
              <View style={styles.tipItem}>
                <Text style={styles.tipBullet}>•</Text>
                <Text style={styles.tipText}>
                  Make sure you're using the full recipe page URL, not a search results page
                </Text>
              </View>
              <View style={styles.tipItem}>
                <Text style={styles.tipBullet}>•</Text>
                <Text style={styles.tipText}>
                  Some recipe sites may require you to scroll down to the actual recipe
                </Text>
              </View>
              <View style={styles.tipItem}>
                <Text style={styles.tipBullet}>•</Text>
                <Text style={styles.tipText}>
                  If import fails, you can always add the recipe manually
                </Text>
              </View>
              <View style={styles.tipItem}>
                <Text style={styles.tipBullet}>•</Text>
                <Text style={styles.tipText}>
                  After import, you can edit any details that weren't captured correctly
                </Text>
              </View>
            </View>
          </View>

          {/* Manual Alternative */}
          <View style={styles.manualSection}>
            <Text style={styles.manualTitle}>Can't import your recipe?</Text>
            <Text style={styles.manualText}>
              No problem! You can always add recipes manually with full control over all details.
            </Text>
            <TouchableOpacity
              style={styles.manualButton}
              onPress={() => navigation.navigate('AddRecipe')}
              disabled={isImporting}
            >
              <Text style={styles.manualButtonText}>Add Recipe Manually</Text>
            </TouchableOpacity>
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
  importButton: {
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
  instructionsSection: {
    paddingHorizontal: 16,
    paddingVertical: 20,
  },
  instructionsTitle: {
    fontSize: 20,
    fontWeight: '600',
    color: '#333',
    marginBottom: 12,
  },
  instructionsText: {
    fontSize: 16,
    color: '#666',
    lineHeight: 22,
  },
  inputSection: {
    paddingHorizontal: 16,
    paddingVertical: 20,
    borderTopWidth: 1,
    borderTopColor: '#f0f0f0',
  },
  label: {
    fontSize: 16,
    fontWeight: '500',
    color: '#333',
    marginBottom: 8,
  },
  urlInput: {
    height: 50,
    paddingHorizontal: 16,
    backgroundColor: '#f8f8f8',
    borderRadius: 12,
    fontSize: 16,
    color: '#333',
    borderWidth: 1,
    borderColor: '#e0e0e0',
  },
  importingIndicator: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'center',
    marginTop: 16,
    paddingVertical: 12,
    backgroundColor: '#f0f8ff',
    borderRadius: 8,
  },
  importingText: {
    marginLeft: 8,
    fontSize: 14,
    color: '#007AFF',
    fontWeight: '500',
  },
  sitesSection: {
    paddingHorizontal: 16,
    paddingVertical: 20,
    borderTopWidth: 1,
    borderTopColor: '#f0f0f0',
  },
  sitesTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#333',
    marginBottom: 8,
  },
  sitesSubtitle: {
    fontSize: 14,
    color: '#666',
    marginBottom: 16,
    lineHeight: 20,
  },
  sitesList: {
    gap: 8,
  },
  siteItem: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    paddingHorizontal: 16,
    paddingVertical: 12,
    backgroundColor: '#f8f8f8',
    borderRadius: 8,
  },
  siteText: {
    fontSize: 14,
    color: '#333',
    fontWeight: '500',
  },
  siteArrow: {
    fontSize: 16,
    color: '#007AFF',
  },
  tipsSection: {
    paddingHorizontal: 16,
    paddingVertical: 20,
    borderTopWidth: 1,
    borderTopColor: '#f0f0f0',
  },
  tipsTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#333',
    marginBottom: 16,
  },
  tipsList: {
    gap: 12,
  },
  tipItem: {
    flexDirection: 'row',
    alignItems: 'flex-start',
  },
  tipBullet: {
    fontSize: 16,
    color: '#007AFF',
    marginRight: 12,
    marginTop: 2,
  },
  tipText: {
    flex: 1,
    fontSize: 14,
    color: '#666',
    lineHeight: 20,
  },
  manualSection: {
    paddingHorizontal: 16,
    paddingVertical: 24,
    borderTopWidth: 1,
    borderTopColor: '#f0f0f0',
    alignItems: 'center',
  },
  manualTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#333',
    marginBottom: 8,
    textAlign: 'center',
  },
  manualText: {
    fontSize: 14,
    color: '#666',
    lineHeight: 20,
    textAlign: 'center',
    marginBottom: 20,
    paddingHorizontal: 20,
  },
  manualButton: {
    paddingHorizontal: 24,
    paddingVertical: 12,
    backgroundColor: '#007AFF',
    borderRadius: 8,
  },
  manualButtonText: {
    color: '#fff',
    fontSize: 16,
    fontWeight: '600',
  },
});
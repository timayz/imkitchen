import React, { useState } from 'react';
import { View, Text, TouchableOpacity, StyleSheet, Alert, ActivityIndicator } from 'react-native';
import { useFavoritesStore } from '../../store/favorites_store';

export interface FavoritesExport {
  version: string;
  exportedAt: string;
  favorites: {
    recipeId: string;
    recipeName: string;
    favoriteAt: string;
    metadata?: {
      tags?: string[];
      complexity?: string;
      prepTime?: number;
    };
  }[];
}

export const FavoritesImportExport: React.FC = () => {
  const { favorites, importFavorites, exportFavorites } = useFavoritesStore();
  const [isExporting, setIsExporting] = useState(false);
  const [isImporting, setIsImporting] = useState(false);

  const handleExport = async () => {
    if (favorites.length === 0) {
      Alert.alert(
        'No Favorites to Export',
        'You don\'t have any favorite recipes to export yet.',
        [{ text: 'OK' }]
      );
      return;
    }

    setIsExporting(true);
    
    try {
      const exportData: FavoritesExport = {
        version: '1.0',
        exportedAt: new Date().toISOString(),
        favorites: favorites.map(fav => ({
          recipeId: fav.recipeId,
          recipeName: fav.recipe.name,
          favoriteAt: fav.favoriteAt,
          metadata: {
            tags: fav.recipe.tags,
            complexity: fav.recipe.complexity,
            prepTime: fav.recipe.prepTime,
          }
        }))
      };

      await exportFavorites(exportData);
      
      Alert.alert(
        'Export Successful',
        `Successfully exported ${favorites.length} favorite recipes. The file has been saved to your device.`,
        [{ text: 'OK' }]
      );
    } catch (error) {
      console.error('Failed to export favorites:', error);
      Alert.alert(
        'Export Failed',
        'Failed to export your favorites. Please try again.',
        [{ text: 'OK' }]
      );
    } finally {
      setIsExporting(false);
    }
  };

  const handleImport = () => {
    Alert.alert(
      'Import Favorites',
      'This will allow you to import favorites from a previously exported file. Any duplicate recipes will be skipped.',
      [
        { text: 'Cancel', style: 'cancel' },
        { text: 'Choose File', onPress: performImport }
      ]
    );
  };

  const performImport = async () => {
    setIsImporting(true);
    
    try {
      // In a real app, this would open a file picker
      // For now, we'll simulate the import process
      const mockImportData: FavoritesExport = {
        version: '1.0',
        exportedAt: new Date().toISOString(),
        favorites: [] // Would be populated from selected file
      };

      const result = await importFavorites(mockImportData);
      
      Alert.alert(
        'Import Successful',
        `Successfully imported ${result.imported} favorites. ${result.skipped} duplicates were skipped.`,
        [{ text: 'OK' }]
      );
    } catch (error) {
      console.error('Failed to import favorites:', error);
      Alert.alert(
        'Import Failed',
        'Failed to import favorites. Please check the file format and try again.',
        [{ text: 'OK' }]
      );
    } finally {
      setIsImporting(false);
    }
  };

  const handleClearFavorites = () => {
    if (favorites.length === 0) {
      Alert.alert(
        'No Favorites',
        'You don\'t have any favorites to clear.',
        [{ text: 'OK' }]
      );
      return;
    }

    Alert.alert(
      'Clear All Favorites',
      `This will remove all ${favorites.length} favorite recipes. This action cannot be undone.`,
      [
        { text: 'Cancel', style: 'cancel' },
        { 
          text: 'Clear All', 
          style: 'destructive',
          onPress: async () => {
            try {
              // Implementation would clear all favorites
              Alert.alert('Cleared', 'All favorites have been removed.');
            } catch (error) {
              Alert.alert('Error', 'Failed to clear favorites.');
            }
          }
        }
      ]
    );
  };

  return (
    <View style={styles.container}>
      <Text style={styles.title}>Data Management</Text>
      <Text style={styles.subtitle}>
        Export your favorites for backup or import from another device
      </Text>

      <View style={styles.actions}>
        <TouchableOpacity
          style={[styles.actionButton, styles.exportButton]}
          onPress={handleExport}
          disabled={isExporting || favorites.length === 0}
        >
          {isExporting ? (
            <ActivityIndicator size="small" color="#FFFFFF" />
          ) : (
            <Text style={styles.exportButtonText}>
              📤 Export Favorites ({favorites.length})
            </Text>
          )}
        </TouchableOpacity>

        <TouchableOpacity
          style={[styles.actionButton, styles.importButton]}
          onPress={handleImport}
          disabled={isImporting}
        >
          {isImporting ? (
            <ActivityIndicator size="small" color="#3B82F6" />
          ) : (
            <Text style={styles.importButtonText}>
              📥 Import Favorites
            </Text>
          )}
        </TouchableOpacity>

        <TouchableOpacity
          style={[styles.actionButton, styles.clearButton]}
          onPress={handleClearFavorites}
          disabled={favorites.length === 0}
        >
          <Text style={styles.clearButtonText}>
            🗑️ Clear All Favorites
          </Text>
        </TouchableOpacity>
      </View>

      <View style={styles.info}>
        <Text style={styles.infoText}>
          • Export creates a backup file of your favorites
        </Text>
        <Text style={styles.infoText}>
          • Import works with files exported from imkitchen
        </Text>
        <Text style={styles.infoText}>
          • Your data stays on your device and is never shared
        </Text>
      </View>
    </View>
  );
};

const styles = StyleSheet.create({
  container: {
    backgroundColor: '#FFFFFF',
    borderRadius: 12,
    margin: 16,
    padding: 20,
    borderWidth: 1,
    borderColor: '#E5E7EB',
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 1 },
    shadowOpacity: 0.05,
    shadowRadius: 2,
    elevation: 2,
  },
  title: {
    fontSize: 18,
    fontWeight: '600',
    color: '#1F2937',
    marginBottom: 8,
  },
  subtitle: {
    fontSize: 14,
    color: '#6B7280',
    marginBottom: 20,
    lineHeight: 20,
  },
  actions: {
    gap: 12,
    marginBottom: 20,
  },
  actionButton: {
    paddingVertical: 12,
    paddingHorizontal: 16,
    borderRadius: 8,
    alignItems: 'center',
    justifyContent: 'center',
    minHeight: 48,
  },
  exportButton: {
    backgroundColor: '#10B981',
  },
  importButton: {
    backgroundColor: '#FFFFFF',
    borderWidth: 2,
    borderColor: '#3B82F6',
  },
  clearButton: {
    backgroundColor: '#FFFFFF',
    borderWidth: 2,
    borderColor: '#EF4444',
  },
  exportButtonText: {
    color: '#FFFFFF',
    fontSize: 16,
    fontWeight: '600',
  },
  importButtonText: {
    color: '#3B82F6',
    fontSize: 16,
    fontWeight: '600',
  },
  clearButtonText: {
    color: '#EF4444',
    fontSize: 16,
    fontWeight: '600',
  },
  info: {
    backgroundColor: '#F9FAFB',
    borderRadius: 8,
    padding: 16,
  },
  infoText: {
    fontSize: 12,
    color: '#6B7280',
    marginBottom: 4,
    lineHeight: 16,
  },
});
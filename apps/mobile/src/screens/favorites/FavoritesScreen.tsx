import React, { useEffect } from 'react';
import { View, StyleSheet, Alert } from 'react-native';
import { FavoritesManager } from '../../components/favorites/FavoritesManager';
import { useFavoritesStore } from '../../store/favorites_store';

interface FavoritesScreenProps {
  onRecipeSelect?: (recipeId: string) => void;
}

export const FavoritesScreen: React.FC<FavoritesScreenProps> = ({
  onRecipeSelect
}) => {
  const { error, clearError } = useFavoritesStore();

  useEffect(() => {
    if (error) {
      Alert.alert(
        'Favorites Error',
        error,
        [
          { text: 'OK', onPress: clearError }
        ]
      );
    }
  }, [error, clearError]);

  const handleRecipeSelect = (recipeId: string) => {
    if (onRecipeSelect) {
      onRecipeSelect(recipeId);
    } else {
      // Default behavior: navigate to recipe detail
      console.log('Navigate to recipe:', recipeId);
    }
  };

  return (
    <View style={styles.container}>
      <FavoritesManager
        onRecipeSelect={handleRecipeSelect}
        showImportExport={true}
      />
    </View>
  );
};

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#F9FAFB',
  },
});
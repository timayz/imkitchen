/**
 * Enhanced Empty State Component
 * Advanced empty states with helpful guidance and call-to-action prompts
 */

import React from 'react';
import {
  View,
  Text,
  TouchableOpacity,
  StyleSheet,
  Animated,
  ViewStyle,
  TextStyle,
} from 'react-native';
import { useTheme } from '../../theme/ThemeProvider';
import { createFadeAnimation, ANIMATION_DURATION } from '../../theme/animations';

export interface EmptyStateProps {
  // Visual content
  icon?: string | React.ReactNode;
  title: string;
  message: string;
  
  // Actions
  primaryAction?: {
    text: string;
    onPress: () => void;
    icon?: string;
  };
  secondaryAction?: {
    text: string;
    onPress: () => void;
    icon?: string;
  };
  
  // Appearance
  variant?: 'default' | 'centered' | 'compact';
  animated?: boolean;
  
  // Accessibility
  testID?: string;
  
  // Styling
  style?: ViewStyle;
  titleStyle?: TextStyle;
  messageStyle?: TextStyle;
}

export const EmptyState: React.FC<EmptyStateProps> = ({
  icon = '📝',
  title,
  message,
  primaryAction,
  secondaryAction,
  variant = 'default',
  animated = true,
  testID,
  style,
  titleStyle,
  messageStyle,
}) => {
  const { colors } = useTheme();
  const fadeAnim = React.useRef(new Animated.Value(0)).current;

  React.useEffect(() => {
    if (animated) {
      createFadeAnimation(fadeAnim, 1, ANIMATION_DURATION.NORMAL).start();
    } else {
      fadeAnim.setValue(1);
    }
  }, [fadeAnim, animated]);

  const styles = StyleSheet.create({
    container: {
      flex: 1,
      justifyContent: 'center',
      alignItems: 'center',
      padding: 32,
      backgroundColor: colors.background,
    },
    compactContainer: {
      paddingVertical: 24,
      paddingHorizontal: 16,
    },
    centeredContainer: {
      minHeight: 300,
    },
    icon: {
      fontSize: 64,
      marginBottom: 16,
      textAlign: 'center',
    },
    compactIcon: {
      fontSize: 48,
      marginBottom: 12,
    },
    title: {
      fontSize: 20,
      fontWeight: '700',
      color: colors.text,
      marginBottom: 8,
      textAlign: 'center',
    },
    compactTitle: {
      fontSize: 18,
      marginBottom: 6,
    },
    message: {
      fontSize: 16,
      color: colors.textSecondary,
      marginBottom: 24,
      textAlign: 'center',
      lineHeight: 24,
      maxWidth: 300,
    },
    compactMessage: {
      fontSize: 14,
      marginBottom: 16,
      maxWidth: 250,
    },
    actionsContainer: {
      flexDirection: 'column',
      alignItems: 'center',
      gap: 12,
      width: '100%',
      maxWidth: 280,
    },
    compactActionsContainer: {
      flexDirection: 'row',
      justifyContent: 'center',
      gap: 12,
      maxWidth: 300,
    },
    primaryButton: {
      backgroundColor: colors.primary,
      paddingHorizontal: 24,
      paddingVertical: 14,
      borderRadius: 8,
      minWidth: 160,
      alignItems: 'center',
      justifyContent: 'center',
      flexDirection: 'row',
      gap: 8,
    },
    secondaryButton: {
      backgroundColor: colors.backgroundSecondary,
      borderWidth: 1,
      borderColor: colors.border,
      paddingHorizontal: 20,
      paddingVertical: 12,
      borderRadius: 8,
      minWidth: 120,
      alignItems: 'center',
      justifyContent: 'center',
      flexDirection: 'row',
      gap: 8,
    },
    compactButton: {
      paddingHorizontal: 16,
      paddingVertical: 10,
      minWidth: 100,
      flex: 1,
      maxWidth: 140,
    },
    primaryButtonText: {
      color: colors.textInverse,
      fontSize: 16,
      fontWeight: '600',
    },
    secondaryButtonText: {
      color: colors.text,
      fontSize: 16,
      fontWeight: '500',
    },
    compactButtonText: {
      fontSize: 14,
    },
    buttonIcon: {
      fontSize: 16,
    },
  });

  const containerStyle = [
    styles.container,
    variant === 'compact' && styles.compactContainer,
    variant === 'centered' && styles.centeredContainer,
    style,
  ];

  const iconStyle = [
    styles.icon,
    variant === 'compact' && styles.compactIcon,
  ];

  const titleStyleCombined = [
    styles.title,
    variant === 'compact' && styles.compactTitle,
    titleStyle,
  ];

  const messageStyleCombined = [
    styles.message,
    variant === 'compact' && styles.compactMessage,
    messageStyle,
  ];

  const actionsContainerStyle = [
    styles.actionsContainer,
    variant === 'compact' && styles.compactActionsContainer,
  ];

  const buttonStyle = [
    styles.primaryButton,
    variant === 'compact' && styles.compactButton,
  ];

  const secondaryButtonStyle = [
    styles.secondaryButton,
    variant === 'compact' && styles.compactButton,
  ];

  return (
    <Animated.View style={[containerStyle, { opacity: fadeAnim }]} testID={testID}>
      {typeof icon === 'string' ? (
        <Text style={iconStyle}>{icon}</Text>
      ) : (
        <View style={{ marginBottom: variant === 'compact' ? 12 : 16 }}>
          {icon}
        </View>
      )}
      
      <Text style={titleStyleCombined}>{title}</Text>
      <Text style={messageStyleCombined}>{message}</Text>
      
      {(primaryAction || secondaryAction) && (
        <View style={actionsContainerStyle}>
          {primaryAction && (
            <TouchableOpacity
              style={buttonStyle}
              onPress={primaryAction.onPress}
              accessibilityRole="button"
              accessibilityLabel={primaryAction.text}
              activeOpacity={0.8}
            >
              {primaryAction.icon && (
                <Text style={styles.buttonIcon}>{primaryAction.icon}</Text>
              )}
              <Text style={[styles.primaryButtonText, variant === 'compact' && styles.compactButtonText]}>
                {primaryAction.text}
              </Text>
            </TouchableOpacity>
          )}
          
          {secondaryAction && (
            <TouchableOpacity
              style={secondaryButtonStyle}
              onPress={secondaryAction.onPress}
              accessibilityRole="button"
              accessibilityLabel={secondaryAction.text}
              activeOpacity={0.8}
            >
              {secondaryAction.icon && (
                <Text style={styles.buttonIcon}>{secondaryAction.icon}</Text>
              )}
              <Text style={[styles.secondaryButtonText, variant === 'compact' && styles.compactButtonText]}>
                {secondaryAction.text}
              </Text>
            </TouchableOpacity>
          )}
        </View>
      )}
    </Animated.View>
  );
};

// Preset empty states for common scenarios
export const EmptyMealPlanState: React.FC<{
  onCreateMealPlan?: () => void;
  onImportRecipes?: () => void;
}> = ({ onCreateMealPlan, onImportRecipes }) => (
  <EmptyState
    icon="🍽️"
    title="No Meal Plan Yet"
    message="Create your first meal plan to see your weekly schedule. Our AI will help you build a personalized plan with your favorite recipes."
    primaryAction={onCreateMealPlan ? {
      text: "Fill My Week",
      onPress: onCreateMealPlan,
      icon: "✨"
    } : undefined}
    secondaryAction={onImportRecipes ? {
      text: "Add Recipes First",
      onPress: onImportRecipes,
      icon: "📚"
    } : undefined}
    testID="empty-meal-plan-state"
  />
);

export const EmptyRecipeCollectionState: React.FC<{
  onBrowseCommunity?: () => void;
  onAddRecipe?: () => void;
}> = ({ onBrowseCommunity, onAddRecipe }) => (
  <EmptyState
    icon="📚"
    title="No Recipes Yet"
    message="Start building your recipe collection! Browse community favorites or add your own recipes to get started with meal planning."
    primaryAction={onBrowseCommunity ? {
      text: "Browse Community",
      onPress: onBrowseCommunity,
      icon: "🌍"
    } : undefined}
    secondaryAction={onAddRecipe ? {
      text: "Add Recipe",
      onPress: onAddRecipe,
      icon: "➕"
    } : undefined}
    testID="empty-recipe-collection-state"
  />
);

export const EmptyShoppingListState: React.FC<{
  onGenerateFromMealPlan?: () => void;
  onAddItems?: () => void;
}> = ({ onGenerateFromMealPlan, onAddItems }) => (
  <EmptyState
    icon="🛒"
    title="Shopping List is Empty"
    message="Your shopping list will appear here. Generate it automatically from your meal plan or add items manually."
    primaryAction={onGenerateFromMealPlan ? {
      text: "Generate from Meal Plan",
      onPress: onGenerateFromMealPlan,
      icon: "⚡"
    } : undefined}
    secondaryAction={onAddItems ? {
      text: "Add Items",
      onPress: onAddItems,
      icon: "✏️"
    } : undefined}
    variant="compact"
    testID="empty-shopping-list-state"
  />
);

export default EmptyState;
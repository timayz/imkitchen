import React from 'react';
import { EmptyState } from './EmptyState';
import type { EmptyStateConfig } from '@imkitchen/shared-types';

interface EmptyMealPlanStateProps extends EmptyStateConfig {
  onActionPress?: () => void;
  onSecondaryActionPress?: () => void;
}

export const EmptyMealPlanState: React.FC<EmptyMealPlanStateProps> = ({
  title,
  message,
  actionText,
  onActionPress,
  onSecondaryActionPress,
  icon = '📅',
}) => {
  return (
    <EmptyState
      icon={icon}
      title={title}
      message={`${message}\n\nQuick Tips:\n• Browse recipes to find meals you love\n• Plan meals 1 week at a time\n• Mix simple and complex recipes\n• Consider prep time for busy days`}
      primaryAction={actionText && onActionPress ? {
        text: actionText,
        onPress: onActionPress,
        icon: '✨'
      } : undefined}
      secondaryAction={onSecondaryActionPress ? {
        text: 'Browse Recipes',
        onPress: onSecondaryActionPress,
        icon: '📚'
      } : undefined}
      variant="default"
      animated={true}
      testID="empty-meal-plan-state"
    />
  );
};

export default EmptyMealPlanState;
import React from 'react';
import { ProgressIndicator } from '../atoms/ProgressIndicator';
import type { ShoppingProgressBarProps } from '../../types/shopping';

export const ShoppingProgressBar: React.FC<ShoppingProgressBarProps> = ({
  totalItems,
  completedItems,
  showPercentage = false,
}) => {
  const progress = totalItems > 0 ? completedItems / totalItems : 0;
  
  return (
    <ProgressIndicator
      progress={progress}
      total={totalItems}
      completed={completedItems}
      showPercentage={showPercentage}
      variant="detailed"
      animated={true}
      pulseOnComplete={true}
      accessibilityLabel={`Shopping progress: ${completedItems} of ${totalItems} items completed`}
      testID="shopping-progress-bar"
    />
  );
};

export default ShoppingProgressBar;
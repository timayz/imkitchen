/**
 * Animated Progress Indicator Component
 * Smooth progress bars with enhanced animations and accessibility support
 */

import React, { useRef, useEffect } from 'react';
import {
  View,
  Text,
  StyleSheet,
  Animated,
  ViewStyle,
} from 'react-native';
import { useTheme } from '../../theme/ThemeProvider';
import { 
  createProgressAnimation,
  createPulseAnimation,
  ANIMATION_DURATION,
  withPerformanceMonitoring 
} from '../../theme/animations';

export interface ProgressIndicatorProps {
  // Progress data
  progress: number; // 0 to 1
  total?: number;
  completed?: number;
  
  // Display options
  showPercentage?: boolean;
  showText?: boolean;
  label?: string;
  variant?: 'default' | 'compact' | 'detailed';
  
  // Styling
  height?: number;
  color?: string;
  backgroundColor?: string;
  style?: ViewStyle;
  
  // Animation
  animated?: boolean;
  animationDuration?: number;
  pulseOnComplete?: boolean;
  
  // Accessibility
  accessibilityLabel?: string;
  testID?: string;
}

export const ProgressIndicator: React.FC<ProgressIndicatorProps> = ({
  progress,
  total,
  completed,
  showPercentage = true,
  showText = true,
  label,
  variant = 'default',
  height = 8,
  color,
  backgroundColor,
  style,
  animated = true,
  animationDuration = ANIMATION_DURATION.NORMAL,
  pulseOnComplete = true,
  accessibilityLabel,
  testID,
}) => {
  const { colors } = useTheme();
  const progressAnim = useRef(new Animated.Value(0)).current;
  const pulseAnim = useRef(new Animated.Value(1)).current;
  const isCompleted = progress >= 1;
  
  // Calculate display values
  const percentage = Math.max(0, Math.min(100, Math.round(progress * 100)));
  const displayProgress = Math.max(0, Math.min(1, progress));
  
  const completedItems = completed ?? (total ? Math.round(total * progress) : percentage);
  const totalItems = total ?? 100;
  
  // Animate progress changes
  useEffect(() => {
    if (animated) {
      const animation = withPerformanceMonitoring(
        createProgressAnimation(progressAnim, displayProgress, animationDuration)
      );
      animation.start();
    } else {
      progressAnim.setValue(displayProgress);
    }
  }, [displayProgress, animated, animationDuration, progressAnim]);
  
  // Pulse animation when completed
  useEffect(() => {
    if (isCompleted && pulseOnComplete) {
      const pulseAnimation = withPerformanceMonitoring(
        createPulseAnimation(pulseAnim, {
          minScale: 1.0,
          maxScale: 1.02,
          duration: ANIMATION_DURATION.SLOW,
          iterations: 3
        })
      );
      pulseAnimation.start(() => {
        pulseAnim.setValue(1);
      });
    } else {
      pulseAnim.setValue(1);
    }
  }, [isCompleted, pulseOnComplete, pulseAnim]);
  
  const getProgressColor = () => {
    if (color) return color;
    if (isCompleted) return colors.success;
    if (percentage >= 75) return colors.warning;
    if (percentage >= 50) return colors.info;
    if (percentage >= 25) return colors.primary;
    return colors.textTertiary;
  };
  
  const getProgressText = () => {
    if (label) return label;
    if (total && completed !== undefined) {
      if (isCompleted) return '🎉 Complete!';
      return `${completedItems} of ${totalItems}`;
    }
    if (isCompleted) return 'Complete';
    return `Progress: ${percentage}%`;
  };
  
  const getBackgroundColor = () => {
    return backgroundColor || colors.backgroundTertiary;
  };
  
  const renderCompactVariant = () => (
    <View style={[styles.compactContainer, style]}>
      <Animated.View 
        style={[
          styles.progressBarContainer, 
          { height, transform: [{ scale: pulseAnim }] }
        ]}
      >
        <View style={[styles.progressBar, { backgroundColor: getBackgroundColor() }]}>
          <Animated.View
            style={[
              styles.progressFill,
              {
                backgroundColor: getProgressColor(),
                transform: [{ scaleX: progressAnim }],
              },
            ]}
          />
        </View>
      </Animated.View>
      
      {showPercentage && (
        <Text style={[styles.compactPercentage, { color: getProgressColor() }]}>
          {percentage}%
        </Text>
      )}
    </View>
  );
  
  const renderDetailedVariant = () => (
    <View style={[styles.detailedContainer, style]}>
      <View style={styles.headerContainer}>
        {showText && (
          <Text style={[styles.progressText, { color: colors.text }]}>
            {getProgressText()}
          </Text>
        )}
        
        {showPercentage && (
          <Text style={[styles.percentageText, { color: getProgressColor() }]}>
            {percentage}%
          </Text>
        )}
      </View>
      
      <Animated.View 
        style={[
          styles.progressBarContainer, 
          { height, transform: [{ scale: pulseAnim }] }
        ]}
      >
        <View style={[styles.progressBar, { backgroundColor: getBackgroundColor() }]}>
          <Animated.View
            style={[
              styles.progressFill,
              {
                backgroundColor: getProgressColor(),
                transform: [{ scaleX: progressAnim }],
              },
            ]}
          />
          
          {/* Completion sparkle effect */}
          {isCompleted && (
            <View style={styles.completionOverlay}>
              <Text style={styles.completionIcon}>✨</Text>
            </View>
          )}
        </View>
      </Animated.View>
      
      {/* Breakdown */}
      {total && (
        <View style={styles.breakdown}>
          <View style={styles.breakdownItem}>
            <View style={[styles.breakdownDot, { backgroundColor: getProgressColor() }]} />
            <Text style={[styles.breakdownText, { color: colors.textSecondary }]}>
              {completedItems} completed
            </Text>
          </View>
          
          {totalItems - completedItems > 0 && (
            <View style={styles.breakdownItem}>
              <View style={[styles.breakdownDot, { backgroundColor: colors.backgroundTertiary }]} />
              <Text style={[styles.breakdownText, { color: colors.textSecondary }]}>
                {totalItems - completedItems} remaining
              </Text>
            </View>
          )}
        </View>
      )}
    </View>
  );
  
  const renderDefaultVariant = () => (
    <View style={[styles.defaultContainer, style]}>
      {showText && (
        <Text style={[styles.progressText, { color: colors.text }]}>
          {getProgressText()}
        </Text>
      )}
      
      <View style={styles.progressRow}>
        <Animated.View 
          style={[
            styles.progressBarContainer, 
            { height, flex: 1, transform: [{ scale: pulseAnim }] }
          ]}
        >
          <View style={[styles.progressBar, { backgroundColor: getBackgroundColor() }]}>
            <Animated.View
              style={[
                styles.progressFill,
                {
                  backgroundColor: getProgressColor(),
                  transform: [{ scaleX: progressAnim }],
                },
              ]}
            />
          </View>
        </Animated.View>
        
        {showPercentage && (
          <Text style={[styles.percentageText, { color: getProgressColor() }]}>
            {percentage}%
          </Text>
        )}
      </View>
    </View>
  );
  
  const accessibilityProps = {
    accessibilityRole: 'progressbar' as const,
    accessibilityValue: {
      min: 0,
      max: 100,
      now: percentage,
    },
    accessibilityLabel: accessibilityLabel || `Progress: ${percentage} percent`,
  };
  
  return (
    <View {...accessibilityProps} testID={testID}>
      {variant === 'compact' && renderCompactVariant()}
      {variant === 'detailed' && renderDetailedVariant()}
      {variant === 'default' && renderDefaultVariant()}
    </View>
  );
};

const styles = StyleSheet.create({
  defaultContainer: {
    paddingVertical: 4,
  },
  compactContainer: {
    flexDirection: 'row',
    alignItems: 'center',
    paddingVertical: 2,
  },
  detailedContainer: {
    paddingVertical: 8,
  },
  headerContainer: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 8,
  },
  progressRow: {
    flexDirection: 'row',
    alignItems: 'center',
    marginTop: 4,
  },
  progressText: {
    fontSize: 14,
    fontWeight: '500',
    marginBottom: 4,
  },
  percentageText: {
    fontSize: 14,
    fontWeight: '600',
    marginLeft: 12,
    minWidth: 40,
    textAlign: 'right',
  },
  compactPercentage: {
    fontSize: 12,
    fontWeight: '600',
    marginLeft: 8,
    minWidth: 32,
    textAlign: 'right',
  },
  progressBarContainer: {
    position: 'relative',
  },
  progressBar: {
    height: '100%',
    borderRadius: 4,
    overflow: 'hidden',
    position: 'relative',
  },
  progressFill: {
    height: '100%',
    borderRadius: 4,
    transformOrigin: 'left',
  },
  completionOverlay: {
    position: 'absolute',
    top: 0,
    left: 0,
    right: 0,
    bottom: 0,
    alignItems: 'center',
    justifyContent: 'center',
    backgroundColor: 'rgba(76, 175, 80, 0.1)',
  },
  completionIcon: {
    fontSize: 10,
  },
  breakdown: {
    flexDirection: 'row',
    alignItems: 'center',
    flexWrap: 'wrap',
    marginTop: 8,
  },
  breakdownItem: {
    flexDirection: 'row',
    alignItems: 'center',
    marginRight: 16,
    marginTop: 2,
  },
  breakdownDot: {
    width: 8,
    height: 8,
    borderRadius: 4,
    marginRight: 6,
  },
  breakdownText: {
    fontSize: 12,
  },
});

export default ProgressIndicator;
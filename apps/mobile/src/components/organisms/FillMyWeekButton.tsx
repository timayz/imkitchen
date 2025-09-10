import React, { useState, useRef, useEffect } from 'react';
import {
  TouchableOpacity,
  Text,
  View,
  StyleSheet,
  ActivityIndicator,
  Animated,
  Alert,
} from 'react-native';
import type { MealPlanGenerationResponse } from '@imkitchen/shared-types';
import { mealPlanService } from '../../services/meal_plan_service';
import { useMealPlanStore } from '../../store/meal_plan_store';
import { useTheme } from '../../theme/ThemeProvider';
import { 
  createPulseAnimation, 
  createScaleAnimation, 
  createProgressAnimation,
  ANIMATION_DURATION,
  withPerformanceMonitoring 
} from '../../theme/animations';

interface FillMyWeekButtonProps {
  onGenerationComplete?: (response: MealPlanGenerationResponse) => void;
  onGenerationError?: (error: string) => void;
  weekStartDate?: Date;
  style?: any;
  disabled?: boolean;
  testID?: string;
}

export const FillMyWeekButton: React.FC<FillMyWeekButtonProps> = ({
  onGenerationComplete,
  onGenerationError,
  weekStartDate,
  style,
  disabled = false,
  testID,
}) => {
  const { colors } = useTheme();
  const [isGenerating, setIsGenerating] = useState(false);
  const [progress, setProgress] = useState(0);
  
  // Animation values
  const scaleAnim = useRef(new Animated.Value(1)).current;
  const pulseAnim = useRef(new Animated.Value(1)).current;
  const progressAnim = useRef(new Animated.Value(0)).current;
  
  const { loadMealPlan, currentWeek } = useMealPlanStore();
  
  // Start pulsing animation when generating
  useEffect(() => {
    if (isGenerating) {
      const pulseAnimation = withPerformanceMonitoring(
        createPulseAnimation(pulseAnim, {
          minScale: 1.0,
          maxScale: 1.05,
          duration: ANIMATION_DURATION.SLOW,
          iterations: -1
        })
      );
      pulseAnimation.start();
      
      return () => {
        pulseAnimation.stop();
      };
    } else {
      pulseAnim.setValue(1);
    }
  }, [isGenerating, pulseAnim]);
  
  // Animate progress bar
  useEffect(() => {
    const animation = withPerformanceMonitoring(
      createProgressAnimation(progressAnim, progress / 100, ANIMATION_DURATION.NORMAL)
    );
    animation.start();
  }, [progress, progressAnim]);

  const handleGeneration = async () => {
    if (isGenerating || disabled) return;

    try {
      setIsGenerating(true);
      setProgress(0);

      // Animate button press with enhanced animation
      const pressAnimation = withPerformanceMonitoring(
        Animated.sequence([
          createScaleAnimation(scaleAnim, 0.95, ANIMATION_DURATION.FAST),
          createScaleAnimation(scaleAnim, 1, ANIMATION_DURATION.FAST),
        ])
      );
      pressAnimation.start();

      // Simulate progress for user feedback
      const progressInterval = setInterval(() => {
        setProgress(prev => {
          if (prev >= 90) {
            clearInterval(progressInterval);
            return prev;
          }
          return prev + Math.random() * 15;
        });
      }, 200);

      // Call the generation API
      const response = await mealPlanService.generateWeeklyMealPlan({
        weekStartDate: weekStartDate || currentWeek,
      });

      // Complete progress
      setProgress(100);
      clearInterval(progressInterval);

      // Show success feedback
      setTimeout(() => {
        setIsGenerating(false);
        setProgress(0);
        
        // Handle warnings if any
        if (response.warnings && response.warnings.length > 0) {
          Alert.alert(
            'Meal Plan Generated',
            `Your weekly meal plan is ready!\n\n${response.warnings.join('\n')}`,
            [
              {
                text: 'View Plan',
                onPress: () => onGenerationComplete?.(response),
              },
            ]
          );
        } else {
          onGenerationComplete?.(response);
        }

        // Refresh the meal plan in the store
        if (weekStartDate || currentWeek) {
          loadMealPlan(weekStartDate || currentWeek, true);
        }
      }, 500);

    } catch (error) {
      setIsGenerating(false);
      setProgress(0);
      clearInterval(progressInterval);
      
      const errorMessage = error instanceof Error ? error.message : 'Failed to generate meal plan';
      
      Alert.alert(
        'Generation Failed',
        errorMessage,
        [
          {
            text: 'Try Again',
            onPress: () => handleGeneration(),
          },
          {
            text: 'Cancel',
            style: 'cancel',
          },
        ]
      );
      
      onGenerationError?.(errorMessage);
    }
  };

  const handleRegenerateConfirmation = () => {
    Alert.alert(
      'Regenerate Meal Plan?',
      'This will replace your current meal plan with a new selection. This action cannot be undone.',
      [
        {
          text: 'Cancel',
          style: 'cancel',
        },
        {
          text: 'Regenerate',
          style: 'destructive',
          onPress: handleGeneration,
        },
      ]
    );
  };

  return (
    <Animated.View
      style={[
        styles.container,
        style,
        { 
          transform: [
            { scale: scaleAnim },
            { scale: isGenerating ? pulseAnim : 1 }
          ] 
        },
      ]}
    >
      <TouchableOpacity
        style={[
          getButtonStyles(colors, disabled, isGenerating),
        ]}
        onPress={handleGeneration}
        disabled={disabled || isGenerating}
        accessibilityRole="button"
        accessibilityLabel={
          isGenerating
            ? `Generating meal plan, ${Math.round(progress)}% complete`
            : 'Fill My Week - Generate automated meal plan'
        }
        accessibilityHint="Automatically generates a weekly meal plan with recipe variety and rotation"
        accessibilityState={{
          disabled: disabled || isGenerating,
          busy: isGenerating,
        }}
        testID={testID}
      >
        {isGenerating ? (
          <View style={styles.generatingContent}>
            <ActivityIndicator
              size="small"
              color={colors.textInverse}
              style={styles.spinner}
            />
            <Text style={getGeneratingTextStyles(colors)}>
              Generating... {Math.round(progress)}%
            </Text>
            {progress > 0 && (
              <View style={getProgressBarStyles(colors)}>
                <Animated.View
                  style={[
                    getProgressFillStyles(colors),
                    { 
                      transform: [{ scaleX: progressAnim }],
                      transformOrigin: 'left'
                    },
                  ]}
                />
              </View>
            )}
          </View>
        ) : (
          <>
            <Text style={getButtonTextStyles(colors, disabled)}>
              ✨ Fill My Week
            </Text>
            <Text style={getButtonSubtextStyles(colors, disabled)}>
              Instant meal plan generation
            </Text>
          </>
        )}
      </TouchableOpacity>
      
      {/* Accessibility live region for progress updates */}
      <Text
        style={styles.srOnly}
        accessibilityLiveRegion="polite"
        accessibilityRole="text"
      >
        {isGenerating ? `Generating meal plan, ${Math.round(progress)}% complete` : ''}
      </Text>
    </Animated.View>
  );
};

// Themed style functions
const getButtonStyles = (colors: any, disabled: boolean, isGenerating: boolean) => ({
  backgroundColor: disabled 
    ? colors.disabled 
    : isGenerating 
      ? colors.primaryDark 
      : colors.primary,
  paddingHorizontal: 32,
  paddingVertical: 16,
  borderRadius: 12,
  minHeight: 64,
  minWidth: 280,
  justifyContent: 'center' as const,
  alignItems: 'center' as const,
  shadowColor: colors.text,
  shadowOffset: {
    width: 0,
    height: 2,
  },
  shadowOpacity: disabled ? 0.1 : 0.25,
  shadowRadius: 3.84,
  elevation: disabled ? 1 : 5,
});

const getButtonTextStyles = (colors: any, disabled: boolean) => ({
  color: disabled ? colors.textTertiary : colors.textInverse,
  fontSize: 18,
  fontWeight: '700' as const,
  textAlign: 'center' as const,
});

const getButtonSubtextStyles = (colors: any, disabled: boolean) => ({
  color: disabled ? colors.textTertiary : colors.textInverse,
  fontSize: 12,
  opacity: disabled ? 0.6 : 0.9,
  marginTop: 2,
  textAlign: 'center' as const,
});

const getGeneratingTextStyles = (colors: any) => ({
  color: colors.textInverse,
  fontSize: 16,
  fontWeight: '600' as const,
  marginBottom: 8,
});

const getProgressBarStyles = (colors: any) => ({
  width: 200,
  height: 4,
  backgroundColor: colors.textInverse + '4D', // 30% opacity
  borderRadius: 2,
  overflow: 'hidden' as const,
});

const getProgressFillStyles = (colors: any) => ({
  height: '100%',
  width: '100%',
  backgroundColor: colors.textInverse,
  borderRadius: 2,
});

const styles = StyleSheet.create({
  container: {
    alignItems: 'center',
    marginVertical: 16,
  },
  generatingContent: {
    alignItems: 'center',
    justifyContent: 'center',
  },
  spinner: {
    marginBottom: 8,
  },
  srOnly: {
    position: 'absolute',
    width: 1,
    height: 1,
    padding: 0,
    margin: -1,
    overflow: 'hidden',
    clip: 'rect(0, 0, 0, 0)',
    whiteSpace: 'nowrap',
    border: 0,
  },
});

export default FillMyWeekButton;
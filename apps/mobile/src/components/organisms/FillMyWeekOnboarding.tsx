import React, { useState } from 'react';
import {
  View,
  Text,
  StyleSheet,
  TouchableOpacity,
  Animated,
  Dimensions,
  Modal,
} from 'react-native';

interface FillMyWeekOnboardingProps {
  visible: boolean;
  onComplete: () => void;
  onSkip: () => void;
}

interface OnboardingStep {
  title: string;
  description: string;
  emoji: string;
  highlight: string;
}

const onboardingSteps: OnboardingStep[] = [
  {
    title: 'Welcome to Fill My Week!',
    description: 'Generate a complete weekly meal plan in seconds with our intelligent rotation algorithm.',
    emoji: '✨',
    highlight: 'One-tap meal planning',
  },
  {
    title: 'Smart Recipe Rotation',
    description: 'Our algorithm ensures variety by avoiding recently used recipes and balancing complexity throughout the week.',
    emoji: '🔄',
    highlight: 'No repeated meals',
  },
  {
    title: 'Personalized for You',
    description: 'The system learns your preferences, dietary restrictions, and cooking skill level for better recommendations.',
    emoji: '🎯',
    highlight: 'Tailored to your needs',
  },
  {
    title: 'Ready to Start?',
    description: 'Tap "Fill My Week" anytime to generate a new meal plan. You can always regenerate if you want different options!',
    emoji: '🚀',
    highlight: 'Start cooking!',
  },
];

const FillMyWeekOnboarding: React.FC<FillMyWeekOnboardingProps> = ({
  visible,
  onComplete,
  onSkip,
}) => {
  const [currentStep, setCurrentStep] = useState(0);
  const [fadeAnim] = useState(new Animated.Value(1));
  const [slideAnim] = useState(new Animated.Value(0));

  const screenWidth = Dimensions.get('window').width;

  const animateToNextStep = (nextStep: number) => {
    Animated.sequence([
      Animated.parallel([
        Animated.timing(fadeAnim, {
          toValue: 0,
          duration: 200,
          useNativeDriver: true,
        }),
        Animated.timing(slideAnim, {
          toValue: -50,
          duration: 200,
          useNativeDriver: true,
        }),
      ]),
      Animated.timing(slideAnim, {
        toValue: 50,
        duration: 0,
        useNativeDriver: true,
      }),
      Animated.parallel([
        Animated.timing(fadeAnim, {
          toValue: 1,
          duration: 300,
          useNativeDriver: true,
        }),
        Animated.timing(slideAnim, {
          toValue: 0,
          duration: 300,
          useNativeDriver: true,
        }),
      ]),
    ]).start(() => {
      setCurrentStep(nextStep);
    });
  };

  const handleNext = () => {
    if (currentStep < onboardingSteps.length - 1) {
      animateToNextStep(currentStep + 1);
    } else {
      onComplete();
    }
  };

  const handlePrevious = () => {
    if (currentStep > 0) {
      animateToNextStep(currentStep - 1);
    }
  };

  const currentStepData = onboardingSteps[currentStep];
  const isFirstStep = currentStep === 0;
  const isLastStep = currentStep === onboardingSteps.length - 1;

  return (
    <Modal
      visible={visible}
      animationType="slide"
      presentationStyle="pageSheet"
    >
      <View style={styles.container}>
        {/* Skip Button */}
        <TouchableOpacity style={styles.skipButton} onPress={onSkip}>
          <Text style={styles.skipButtonText}>Skip</Text>
        </TouchableOpacity>

        {/* Progress Indicator */}
        <View style={styles.progressContainer}>
          {onboardingSteps.map((_, index) => (
            <View
              key={index}
              style={[
                styles.progressDot,
                index === currentStep && styles.progressDotActive,
                index < currentStep && styles.progressDotCompleted,
              ]}
            />
          ))}
        </View>

        {/* Content */}
        <Animated.View
          style={[
            styles.contentContainer,
            {
              opacity: fadeAnim,
              transform: [{ translateX: slideAnim }],
            },
          ]}
        >
          <View style={styles.emojiContainer}>
            <Text style={styles.emoji}>{currentStepData.emoji}</Text>
          </View>

          <Text style={styles.title}>{currentStepData.title}</Text>
          
          <Text style={styles.description}>{currentStepData.description}</Text>

          <View style={styles.highlightContainer}>
            <Text style={styles.highlight}>{currentStepData.highlight}</Text>
          </View>

          {/* Feature Preview */}
          {currentStep === 0 && (
            <View style={styles.featurePreview}>
              <View style={styles.mockButton}>
                <Text style={styles.mockButtonText}>✨ Fill My Week</Text>
                <Text style={styles.mockButtonSubtext}>Instant meal plan generation</Text>
              </View>
              <View style={styles.mockArrow}>
                <Text style={styles.arrowText}>👆 Tap here anytime!</Text>
              </View>
            </View>
          )}

          {currentStep === 1 && (
            <View style={styles.rotationDemo}>
              <Text style={styles.demoTitle}>This week vs Last week:</Text>
              <View style={styles.comparisonContainer}>
                <View style={styles.weekColumn}>
                  <Text style={styles.weekTitle}>Last Week</Text>
                  <Text style={styles.recipeItem}>🍝 Pasta Carbonara</Text>
                  <Text style={styles.recipeItem}>🍗 Grilled Chicken</Text>
                  <Text style={styles.recipeItem}>🥗 Caesar Salad</Text>
                </View>
                <Text style={styles.arrowEmoji}>➡️</Text>
                <View style={styles.weekColumn}>
                  <Text style={styles.weekTitle}>This Week</Text>
                  <Text style={styles.recipeItem}>🍜 Ramen Bowl</Text>
                  <Text style={styles.recipeItem}>🐟 Salmon Teriyaki</Text>
                  <Text style={styles.recipeItem}>🌮 Fish Tacos</Text>
                </View>
              </View>
            </View>
          )}

          {currentStep === 2 && (
            <View style={styles.personalizationDemo}>
              <Text style={styles.demoTitle}>Learns your preferences:</Text>
              <View style={styles.preferencesList}>
                <View style={styles.preferenceItem}>
                  <Text style={styles.preferenceEmoji}>🌱</Text>
                  <Text style={styles.preferenceText}>Vegetarian meals</Text>
                </View>
                <View style={styles.preferenceItem}>
                  <Text style={styles.preferenceEmoji}>⏱️</Text>
                  <Text style={styles.preferenceText}>30-minute recipes</Text>
                </View>
                <View style={styles.preferenceItem}>
                  <Text style={styles.preferenceEmoji}>🍝</Text>
                  <Text style={styles.preferenceText}>Italian cuisine</Text>
                </View>
                <View style={styles.preferenceItem}>
                  <Text style={styles.preferenceEmoji}>📚</Text>
                  <Text style={styles.preferenceText}>Beginner-friendly</Text>
                </View>
              </View>
            </View>
          )}
        </Animated.View>

        {/* Navigation Buttons */}
        <View style={styles.navigationContainer}>
          <TouchableOpacity
            style={[styles.navButton, isFirstStep && styles.navButtonDisabled]}
            onPress={handlePrevious}
            disabled={isFirstStep}
          >
            <Text style={[styles.navButtonText, isFirstStep && styles.navButtonTextDisabled]}>
              Previous
            </Text>
          </TouchableOpacity>

          <TouchableOpacity
            style={[styles.navButton, styles.navButtonPrimary]}
            onPress={handleNext}
          >
            <Text style={[styles.navButtonText, styles.navButtonTextPrimary]}>
              {isLastStep ? 'Get Started' : 'Next'}
            </Text>
          </TouchableOpacity>
        </View>
      </View>
    </Modal>
  );
};

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#FFFFFF',
    paddingTop: 60,
  },
  skipButton: {
    position: 'absolute',
    top: 60,
    right: 20,
    zIndex: 10,
    padding: 10,
  },
  skipButtonText: {
    fontSize: 16,
    color: '#666666',
    fontWeight: '500',
  },
  progressContainer: {
    flexDirection: 'row',
    justifyContent: 'center',
    alignItems: 'center',
    marginTop: 20,
    marginBottom: 40,
    gap: 8,
  },
  progressDot: {
    width: 8,
    height: 8,
    borderRadius: 4,
    backgroundColor: '#E0E0E0',
  },
  progressDotActive: {
    backgroundColor: '#4CAF50',
    width: 24,
  },
  progressDotCompleted: {
    backgroundColor: '#81C784',
  },
  contentContainer: {
    flex: 1,
    paddingHorizontal: 32,
    alignItems: 'center',
    justifyContent: 'center',
  },
  emojiContainer: {
    width: 80,
    height: 80,
    borderRadius: 40,
    backgroundColor: '#F0F8F0',
    justifyContent: 'center',
    alignItems: 'center',
    marginBottom: 24,
  },
  emoji: {
    fontSize: 36,
  },
  title: {
    fontSize: 24,
    fontWeight: '700',
    color: '#333333',
    textAlign: 'center',
    marginBottom: 16,
  },
  description: {
    fontSize: 16,
    color: '#666666',
    textAlign: 'center',
    lineHeight: 24,
    marginBottom: 24,
  },
  highlightContainer: {
    backgroundColor: '#E8F5E8',
    paddingHorizontal: 16,
    paddingVertical: 8,
    borderRadius: 16,
    marginBottom: 32,
  },
  highlight: {
    fontSize: 14,
    fontWeight: '600',
    color: '#2E7D2E',
    textAlign: 'center',
  },
  featurePreview: {
    alignItems: 'center',
  },
  mockButton: {
    backgroundColor: '#4CAF50',
    paddingHorizontal: 32,
    paddingVertical: 16,
    borderRadius: 12,
    alignItems: 'center',
    marginBottom: 12,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.25,
    shadowRadius: 3.84,
    elevation: 5,
  },
  mockButtonText: {
    color: '#FFFFFF',
    fontSize: 18,
    fontWeight: '700',
  },
  mockButtonSubtext: {
    color: '#FFFFFF',
    fontSize: 12,
    opacity: 0.9,
    marginTop: 2,
  },
  mockArrow: {
    marginTop: 8,
  },
  arrowText: {
    fontSize: 14,
    color: '#666666',
  },
  rotationDemo: {
    alignItems: 'center',
    width: '100%',
  },
  demoTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#333333',
    marginBottom: 16,
  },
  comparisonContainer: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    width: '100%',
  },
  weekColumn: {
    flex: 1,
    alignItems: 'center',
  },
  weekTitle: {
    fontSize: 14,
    fontWeight: '600',
    color: '#666666',
    marginBottom: 12,
  },
  recipeItem: {
    fontSize: 14,
    color: '#333333',
    marginBottom: 6,
  },
  arrowEmoji: {
    fontSize: 24,
    marginHorizontal: 16,
  },
  personalizationDemo: {
    alignItems: 'center',
    width: '100%',
  },
  preferencesList: {
    width: '100%',
  },
  preferenceItem: {
    flexDirection: 'row',
    alignItems: 'center',
    paddingVertical: 8,
    paddingHorizontal: 16,
    backgroundColor: '#F8F9FA',
    borderRadius: 8,
    marginBottom: 8,
  },
  preferenceEmoji: {
    fontSize: 20,
    marginRight: 12,
  },
  preferenceText: {
    fontSize: 16,
    color: '#333333',
  },
  navigationContainer: {
    flexDirection: 'row',
    paddingHorizontal: 32,
    paddingBottom: 32,
    gap: 16,
  },
  navButton: {
    flex: 1,
    paddingVertical: 14,
    borderRadius: 8,
    justifyContent: 'center',
    alignItems: 'center',
    borderWidth: 1,
    borderColor: '#E0E0E0',
    backgroundColor: '#FFFFFF',
  },
  navButtonPrimary: {
    backgroundColor: '#4CAF50',
    borderColor: '#4CAF50',
  },
  navButtonDisabled: {
    backgroundColor: '#F8F9FA',
    borderColor: '#E0E0E0',
  },
  navButtonText: {
    fontSize: 16,
    fontWeight: '600',
    color: '#333333',
  },
  navButtonTextPrimary: {
    color: '#FFFFFF',
  },
  navButtonTextDisabled: {
    color: '#CCCCCC',
  },
});

export default FillMyWeekOnboarding;
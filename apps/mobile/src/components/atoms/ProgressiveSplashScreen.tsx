/**
 * Progressive Splash Screen Component
 * 
 * Multi-phase splash screen with loading indicators that show
 * initialization progress through different app startup phases.
 * 
 * Features:
 * - Multi-phase loading animation
 * - Real-time initialization step tracking
 * - Smooth transitions between phases
 * - Timeout and error handling
 * - Accessibility support for loading states
 */

import React, { useState, useEffect, useCallback } from 'react';
import {
  View,
  Text,
  StyleSheet,
  Animated,
  Dimensions,
  ActivityIndicator,
  StatusBar
} from 'react-native';
import { splashScreenService, InitializationStep, InitializationPhase } from '../../services/splash_screen_service';

export interface ProgressiveSplashScreenProps {
  onInitializationComplete: () => void;
  onError?: (error: Error) => void;
  timeout?: number; // in milliseconds
  showDebugInfo?: boolean;
}

export const ProgressiveSplashScreen: React.FC<ProgressiveSplashScreenProps> = ({
  onInitializationComplete,
  onError,
  timeout = 10000, // 10 second default timeout
  showDebugInfo = false
}) => {
  const [currentPhase, setCurrentPhase] = useState<InitializationPhase>('loading');
  const [currentStep, setCurrentStep] = useState<InitializationStep | null>(null);
  const [progress, setProgress] = useState(0);
  const [error, setError] = useState<Error | null>(null);
  const [timeoutReached, setTimeoutReached] = useState(false);

  // Animation values
  const [fadeAnim] = useState(new Animated.Value(0));
  const [scaleAnim] = useState(new Animated.Value(0.8));
  const [progressAnim] = useState(new Animated.Value(0));

  const { width: screenWidth, height: screenHeight } = Dimensions.get('window');

  // Initialize splash screen and start loading
  useEffect(() => {
    const initializeSplashScreen = async () => {
      try {
        // Start entrance animation
        Animated.parallel([
          Animated.timing(fadeAnim, {
            toValue: 1,
            duration: 500,
            useNativeDriver: true
          }),
          Animated.spring(scaleAnim, {
            toValue: 1,
            tension: 50,
            friction: 5,
            useNativeDriver: true
          })
        ]).start();

        // Set up progress tracking
        splashScreenService.onProgressUpdate((step, progress) => {
          setCurrentStep(step);
          setProgress(progress);
          
          // Animate progress bar
          Animated.timing(progressAnim, {
            toValue: progress / 100,
            duration: 300,
            useNativeDriver: false
          }).start();
        });

        splashScreenService.onPhaseChange((phase) => {
          setCurrentPhase(phase);
        });

        splashScreenService.onError((error) => {
          setError(error);
          onError?.(error);
        });

        splashScreenService.onComplete(() => {
          // Exit animation before completing
          Animated.parallel([
            Animated.timing(fadeAnim, {
              toValue: 0,
              duration: 300,
              useNativeDriver: true
            }),
            Animated.timing(scaleAnim, {
              toValue: 1.1,
              duration: 300,
              useNativeDriver: true
            })
          ]).start(() => {
            onInitializationComplete();
          });
        });

        // Start initialization
        await splashScreenService.initializeApp();

      } catch (err) {
        const error = err instanceof Error ? err : new Error('Unknown initialization error');
        setError(error);
        onError?.(error);
      }
    };

    initializeSplashScreen();

    // Set up timeout
    const timeoutId = setTimeout(() => {
      if (currentPhase !== 'complete') {
        setTimeoutReached(true);
        const timeoutError = new Error(`App initialization timed out after ${timeout}ms`);
        setError(timeoutError);
        onError?.(timeoutError);
      }
    }, timeout);

    return () => {
      clearTimeout(timeoutId);
      splashScreenService.cleanup();
    };
  }, [fadeAnim, scaleAnim, progressAnim, onInitializationComplete, onError, timeout, currentPhase]);

  const getPhaseTitle = useCallback((phase: InitializationPhase): string => {
    switch (phase) {
      case 'loading':
        return 'Starting imkitchen...';
      case 'auth':
        return 'Setting up authentication...';
      case 'data':
        return 'Loading your recipes...';
      case 'cache':
        return 'Preparing for offline use...';
      case 'complete':
        return 'Welcome to imkitchen!';
      case 'error':
        return 'Something went wrong';
      default:
        return 'Loading...';
    }
  }, []);

  const getStepDescription = useCallback((step: InitializationStep | null): string => {
    if (!step) return '';

    switch (step) {
      case 'initializing_services':
        return 'Starting core services...';
      case 'checking_auth':
        return 'Checking authentication...';
      case 'loading_user_preferences':
        return 'Loading your preferences...';
      case 'warming_recipe_cache':
        return 'Preparing recipe cache...';
      case 'preloading_critical_screens':
        return 'Loading essential screens...';
      case 'syncing_offline_data':
        return 'Syncing offline data...';
      case 'finalizing':
        return 'Finishing up...';
      default:
        return '';
    }
  }, []);

  const getProgressColor = useCallback((phase: InitializationPhase): string => {
    switch (phase) {
      case 'loading':
        return '#007AFF';
      case 'auth':
        return '#34C759';
      case 'data':
        return '#FF9500';
      case 'cache':
        return '#AF52DE';
      case 'complete':
        return '#34C759';
      case 'error':
        return '#FF3B30';
      default:
        return '#007AFF';
    }
  }, []);

  if (error || timeoutReached) {
    return (
      <View style={styles.container}>
        <StatusBar barStyle="light-content" backgroundColor="#FF3B30" />
        <Animated.View style={[
          styles.content,
          {
            opacity: fadeAnim,
            transform: [{ scale: scaleAnim }]
          }
        ]}>
          <View style={styles.errorContainer}>
            <Text style={styles.errorIcon}>⚠️</Text>
            <Text style={styles.errorTitle}>
              {timeoutReached ? 'Loading Timeout' : 'Initialization Error'}
            </Text>
            <Text style={styles.errorMessage}>
              {error?.message || 'The app is taking longer than expected to load. Please try again.'}
            </Text>
            <Text style={styles.retryHint}>
              Pull down to refresh or restart the app
            </Text>
          </View>
        </Animated.View>
      </View>
    );
  }

  return (
    <View style={styles.container}>
      <StatusBar barStyle="light-content" backgroundColor="#007AFF" />
      
      <Animated.View style={[
        styles.content,
        {
          opacity: fadeAnim,
          transform: [{ scale: scaleAnim }]
        }
      ]}>
        {/* App Logo/Branding */}
        <View style={styles.logoContainer}>
          <View style={styles.logo}>
            <Text style={styles.logoText}>🍳</Text>
          </View>
          <Text style={styles.appName}>imkitchen</Text>
          <Text style={styles.tagline}>Smart Meal Planning</Text>
        </View>

        {/* Loading Content */}
        <View style={styles.loadingContainer}>
          <Text style={styles.phaseTitle}>{getPhaseTitle(currentPhase)}</Text>
          
          {currentStep && (
            <Text style={styles.stepDescription} numberOfLines={1}>
              {getStepDescription(currentStep)}
            </Text>
          )}

          {/* Progress Bar */}
          <View style={styles.progressContainer}>
            <View style={styles.progressTrack}>
              <Animated.View style={[
                styles.progressFill,
                {
                  width: progressAnim.interpolate({
                    inputRange: [0, 1],
                    outputRange: ['0%', '100%']
                  }),
                  backgroundColor: getProgressColor(currentPhase)
                }
              ]} />
            </View>
            <Text style={styles.progressText}>{Math.round(progress)}%</Text>
          </View>

          {/* Phase Indicator */}
          <View style={styles.phaseIndicator}>
            <ActivityIndicator 
              size="large" 
              color={getProgressColor(currentPhase)}
            />
          </View>

          {/* Debug Information */}
          {showDebugInfo && (
            <View style={styles.debugContainer}>
              <Text style={styles.debugText}>Phase: {currentPhase}</Text>
              <Text style={styles.debugText}>Step: {currentStep || 'none'}</Text>
              <Text style={styles.debugText}>Progress: {progress.toFixed(1)}%</Text>
            </View>
          )}
        </View>

        {/* Accessibility Loading Announcement */}
        <View style={styles.accessibilityContainer}>
          <Text 
            style={styles.accessibilityText}
            accessibilityLiveRegion="polite"
            accessibilityRole="text"
          >
            {getPhaseTitle(currentPhase)} {getStepDescription(currentStep)}
          </Text>
        </View>
      </Animated.View>
    </View>
  );
};

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#007AFF',
    justifyContent: 'center',
    alignItems: 'center'
  },
  content: {
    alignItems: 'center',
    justifyContent: 'center',
    paddingHorizontal: 40,
    width: '100%'
  },
  logoContainer: {
    alignItems: 'center',
    marginBottom: 60
  },
  logo: {
    width: 80,
    height: 80,
    borderRadius: 20,
    backgroundColor: 'rgba(255, 255, 255, 0.2)',
    justifyContent: 'center',
    alignItems: 'center',
    marginBottom: 16
  },
  logoText: {
    fontSize: 40,
    textAlign: 'center'
  },
  appName: {
    fontSize: 32,
    fontWeight: 'bold',
    color: '#FFFFFF',
    marginBottom: 8
  },
  tagline: {
    fontSize: 16,
    color: 'rgba(255, 255, 255, 0.8)',
    textAlign: 'center'
  },
  loadingContainer: {
    alignItems: 'center',
    width: '100%'
  },
  phaseTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#FFFFFF',
    textAlign: 'center',
    marginBottom: 12
  },
  stepDescription: {
    fontSize: 14,
    color: 'rgba(255, 255, 255, 0.8)',
    textAlign: 'center',
    marginBottom: 24,
    minHeight: 20
  },
  progressContainer: {
    width: '100%',
    alignItems: 'center',
    marginBottom: 32
  },
  progressTrack: {
    width: '100%',
    height: 4,
    backgroundColor: 'rgba(255, 255, 255, 0.3)',
    borderRadius: 2,
    marginBottom: 12
  },
  progressFill: {
    height: '100%',
    borderRadius: 2,
    backgroundColor: '#FFFFFF'
  },
  progressText: {
    fontSize: 14,
    color: 'rgba(255, 255, 255, 0.9)',
    fontWeight: '500'
  },
  phaseIndicator: {
    marginBottom: 20
  },
  debugContainer: {
    backgroundColor: 'rgba(0, 0, 0, 0.3)',
    padding: 12,
    borderRadius: 8,
    marginTop: 20
  },
  debugText: {
    fontSize: 12,
    color: '#FFFFFF',
    fontFamily: 'Courier New'
  },
  errorContainer: {
    alignItems: 'center',
    paddingHorizontal: 20
  },
  errorIcon: {
    fontSize: 48,
    marginBottom: 16
  },
  errorTitle: {
    fontSize: 20,
    fontWeight: 'bold',
    color: '#FFFFFF',
    textAlign: 'center',
    marginBottom: 12
  },
  errorMessage: {
    fontSize: 16,
    color: 'rgba(255, 255, 255, 0.9)',
    textAlign: 'center',
    lineHeight: 22,
    marginBottom: 20
  },
  retryHint: {
    fontSize: 14,
    color: 'rgba(255, 255, 255, 0.7)',
    textAlign: 'center'
  },
  accessibilityContainer: {
    position: 'absolute',
    left: -10000,
    width: 1,
    height: 1,
    overflow: 'hidden'
  },
  accessibilityText: {
    fontSize: 1,
    color: 'transparent'
  }
});

export default ProgressiveSplashScreen;
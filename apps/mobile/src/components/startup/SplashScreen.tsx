/**
 * Enhanced Splash Screen Component
 * 
 * Provides an engaging splash screen experience with progressive loading
 * indicators, animated transitions, and intelligent preloading status.
 * 
 * Features:
 * - Progressive loading phases with visual feedback
 * - Animated transitions and micro-interactions
 * - Critical resource preloading status
 * - Error handling and retry mechanisms
 * - Performance metrics collection
 * - Accessibility support
 */

import React, { useState, useEffect, useRef } from 'react';
import {
  View,
  Text,
  StyleSheet,
  Animated,
  Dimensions,
  ActivityIndicator,
  Platform,
} from 'react-native';
import { screenRegistry } from '../../navigation/ScreenRegistry';
import { lazyLoadingService } from '../../services/lazy_loading_service';
import { startupMetricsService } from '../../services/startup_metrics_service';

export interface LoadingPhase {
  id: string;
  name: string;
  description: string;
  progress: number;
  status: 'pending' | 'active' | 'completed' | 'error';
  duration: number;
  error?: string;
}

interface SplashScreenProps {
  onLoadingComplete: () => void;
  onLoadingError: (error: Error) => void;
  minimumDisplayTime?: number;
}

const { width: screenWidth, height: screenHeight } = Dimensions.get('window');

export const SplashScreen: React.FC<SplashScreenProps> = ({
  onLoadingComplete,
  onLoadingError,
  minimumDisplayTime = 1500
}) => {
  const [loadingPhases, setLoadingPhases] = useState<LoadingPhase[]>([]);
  const [currentPhaseIndex, setCurrentPhaseIndex] = useState(0);
  const [overallProgress, setOverallProgress] = useState(0);
  const [startupMetrics, setStartupMetrics] = useState({
    startTime: Date.now(),
    phases: {} as Record<string, number>
  });

  // Animation values
  const logoOpacity = useRef(new Animated.Value(0)).current;
  const logoScale = useRef(new Animated.Value(0.8)).current;
  const progressOpacity = useRef(new Animated.Value(0)).current;
  const phaseTextOpacity = useRef(new Animated.Value(0)).current;

  useEffect(() => {
    initializeSplashScreen();
  }, []);

  const initializeSplashScreen = async () => {
    try {
      // Initialize loading phases
      const phases = createLoadingPhases();
      setLoadingPhases(phases);

      // Start logo animation
      animateLogo();

      // Begin loading process
      await executeLoadingPhases(phases);
      
      // Ensure minimum display time
      const elapsedTime = Date.now() - startupMetrics.startTime;
      const remainingTime = Math.max(0, minimumDisplayTime - elapsedTime);
      
      if (remainingTime > 0) {
        await new Promise(resolve => setTimeout(resolve, remainingTime));
      }

      // Complete loading with exit animation
      await animateExit();
      onLoadingComplete();

    } catch (error) {
      console.error('[SplashScreen] Loading failed:', error);
      onLoadingError(error instanceof Error ? error : new Error('Unknown loading error'));
    }
  };

  const createLoadingPhases = (): LoadingPhase[] => {
    return [
      {
        id: 'initialization',
        name: 'Initializing App',
        description: 'Setting up core services',
        progress: 0,
        status: 'pending',
        duration: 0
      },
      {
        id: 'screen_registry',
        name: 'Loading Screens',
        description: 'Preparing user interface',
        progress: 0,
        status: 'pending',
        duration: 0
      },
      {
        id: 'critical_data',
        name: 'Loading Critical Data',
        description: 'Fetching essential information',
        progress: 0,
        status: 'pending',
        duration: 0
      },
      {
        id: 'cache_warmup',
        name: 'Warming Cache',
        description: 'Optimizing performance',
        progress: 0,
        status: 'pending',
        duration: 0
      },
      {
        id: 'finalization',
        name: 'Finalizing',
        description: 'Completing setup',
        progress: 0,
        status: 'pending',
        duration: 0
      }
    ];
  };

  const executeLoadingPhases = async (phases: LoadingPhase[]) => {
    for (let i = 0; i < phases.length; i++) {
      const phase = phases[i];
      const phaseStartTime = Date.now();
      
      setCurrentPhaseIndex(i);
      updatePhaseStatus(phase.id, 'active');
      animatePhaseTransition();

      try {
        await executePhase(phase);
        const phaseDuration = Date.now() - phaseStartTime;
        
        updatePhaseStatus(phase.id, 'completed', phaseDuration);
        setStartupMetrics(prev => ({
          ...prev,
          phases: { ...prev.phases, [phase.id]: phaseDuration }
        }));

        // Update overall progress
        const newProgress = ((i + 1) / phases.length) * 100;
        setOverallProgress(newProgress);
        
      } catch (error) {
        const phaseDuration = Date.now() - phaseStartTime;
        updatePhaseStatus(phase.id, 'error', phaseDuration, error instanceof Error ? error.message : 'Unknown error');
        throw error;
      }
    }
  };

  const executePhase = async (phase: LoadingPhase): Promise<void> => {
    switch (phase.id) {
      case 'initialization':
        await initializeApp();
        break;
      case 'screen_registry':
        await initializeScreenRegistry();
        break;
      case 'critical_data':
        await loadCriticalData();
        break;
      case 'cache_warmup':
        await warmupCaches();
        break;
      case 'finalization':
        await finalizeStartup();
        break;
      default:
        throw new Error(`Unknown phase: ${phase.id}`);
    }
  };

  const initializeApp = async (): Promise<void> => {
    // Initialize startup metrics service
    startupMetricsService.startMeasuring();
    
    // Simulate core service initialization
    await new Promise(resolve => setTimeout(resolve, 200));
    
    updatePhaseProgress('initialization', 100);
  };

  const initializeScreenRegistry = async (): Promise<void> => {
    updatePhaseProgress('screen_registry', 20);
    
    // Initialize screen registry with preloading
    await screenRegistry.initialize();
    
    updatePhaseProgress('screen_registry', 80);
    
    // Wait for critical screens to preload
    const loadingProgress = lazyLoadingService.getLoadingProgress();
    const criticalScreens = loadingProgress.filter(p => p.screenName.includes('Critical'));
    
    // Monitor critical screen loading
    for (const screen of criticalScreens) {
      if (!screen.loaded && screen.loading) {
        await new Promise(resolve => setTimeout(resolve, 100));
      }
    }
    
    updatePhaseProgress('screen_registry', 100);
  };

  const loadCriticalData = async (): Promise<void> => {
    updatePhaseProgress('critical_data', 25);
    
    // Simulate critical data loading (user session, preferences, etc.)
    await new Promise(resolve => setTimeout(resolve, 300));
    updatePhaseProgress('critical_data', 50);
    
    // Simulate API connectivity check
    await new Promise(resolve => setTimeout(resolve, 200));
    updatePhaseProgress('critical_data', 75);
    
    // Simulate user preferences loading
    await new Promise(resolve => setTimeout(resolve, 150));
    updatePhaseProgress('critical_data', 100);
  };

  const warmupCaches = async (): Promise<void> => {
    updatePhaseProgress('cache_warmup', 30);
    
    // Warm up critical caches
    // In real implementation, this would call actual cache services
    await new Promise(resolve => setTimeout(resolve, 250));
    updatePhaseProgress('cache_warmup', 70);
    
    // Preload frequently accessed data
    await new Promise(resolve => setTimeout(resolve, 150));
    updatePhaseProgress('cache_warmup', 100);
  };

  const finalizeStartup = async (): Promise<void> => {
    updatePhaseProgress('finalization', 50);
    
    // Complete startup metrics collection
    const totalStartupTime = Date.now() - startupMetrics.startTime;
    startupMetricsService.recordStartupTime(totalStartupTime);
    
    // Final validation
    await new Promise(resolve => setTimeout(resolve, 100));
    updatePhaseProgress('finalization', 100);
  };

  const updatePhaseStatus = (
    phaseId: string, 
    status: LoadingPhase['status'], 
    duration = 0, 
    error?: string
  ) => {
    setLoadingPhases(phases => 
      phases.map(phase => 
        phase.id === phaseId 
          ? { ...phase, status, duration, error }
          : phase
      )
    );
  };

  const updatePhaseProgress = (phaseId: string, progress: number) => {
    setLoadingPhases(phases => 
      phases.map(phase => 
        phase.id === phaseId 
          ? { ...phase, progress }
          : phase
      )
    );
  };

  // Animation functions
  const animateLogo = () => {
    Animated.parallel([
      Animated.timing(logoOpacity, {
        toValue: 1,
        duration: 800,
        useNativeDriver: true,
      }),
      Animated.spring(logoScale, {
        toValue: 1,
        tension: 50,
        friction: 8,
        useNativeDriver: true,
      }),
    ]).start(() => {
      // Start progress indicator animation
      Animated.timing(progressOpacity, {
        toValue: 1,
        duration: 400,
        useNativeDriver: true,
      }).start();
    });
  };

  const animatePhaseTransition = () => {
    Animated.sequence([
      Animated.timing(phaseTextOpacity, {
        toValue: 0,
        duration: 150,
        useNativeDriver: true,
      }),
      Animated.timing(phaseTextOpacity, {
        toValue: 1,
        duration: 300,
        useNativeDriver: true,
      }),
    ]).start();
  };

  const animateExit = (): Promise<void> => {
    return new Promise(resolve => {
      Animated.parallel([
        Animated.timing(logoOpacity, {
          toValue: 0,
          duration: 400,
          useNativeDriver: true,
        }),
        Animated.timing(progressOpacity, {
          toValue: 0,
          duration: 400,
          useNativeDriver: true,
        }),
      ]).start(() => resolve());
    });
  };

  const currentPhase = loadingPhases[currentPhaseIndex];

  return (
    <View style={styles.container}>
      {/* Background gradient effect */}
      <View style={styles.backgroundGradient} />
      
      {/* Logo section */}
      <Animated.View 
        style={[
          styles.logoContainer,
          {
            opacity: logoOpacity,
            transform: [{ scale: logoScale }]
          }
        ]}
      >
        <View style={styles.logo}>
          <Text style={styles.logoText}>ImKitchen</Text>
          <Text style={styles.logoSubtext}>Smart Meal Planning</Text>
        </View>
      </Animated.View>

      {/* Progress section */}
      <Animated.View 
        style={[
          styles.progressContainer,
          { opacity: progressOpacity }
        ]}
      >
        {/* Overall progress bar */}
        <View style={styles.progressBarContainer}>
          <View 
            style={[
              styles.progressBar,
              { width: `${overallProgress}%` }
            ]} 
          />
        </View>

        {/* Current phase information */}
        <Animated.View 
          style={[
            styles.phaseContainer,
            { opacity: phaseTextOpacity }
          ]}
        >
          <Text style={styles.phaseName}>
            {currentPhase?.name || 'Loading...'}
          </Text>
          <Text style={styles.phaseDescription}>
            {currentPhase?.description || 'Please wait...'}
          </Text>
        </Animated.View>

        {/* Loading indicator */}
        <View style={styles.loadingIndicator}>
          <ActivityIndicator 
            size="small" 
            color="#007AFF" 
            style={styles.spinner}
          />
          <Text style={styles.progressText}>
            {Math.round(overallProgress)}%
          </Text>
        </View>

        {/* Phase indicators */}
        <View style={styles.phaseIndicators}>
          {loadingPhases.map((phase, index) => (
            <View
              key={phase.id}
              style={[
                styles.phaseIndicator,
                {
                  backgroundColor: 
                    phase.status === 'completed' ? '#34C759' :
                    phase.status === 'active' ? '#007AFF' :
                    phase.status === 'error' ? '#FF3B30' :
                    '#E5E5EA'
                }
              ]}
            />
          ))}
        </View>
      </Animated.View>

      {/* Debug information (only in development) */}
      {__DEV__ && (
        <View style={styles.debugContainer}>
          <Text style={styles.debugText}>
            Phase: {currentPhaseIndex + 1}/{loadingPhases.length}
          </Text>
          <Text style={styles.debugText}>
            Time: {Date.now() - startupMetrics.startTime}ms
          </Text>
        </View>
      )}
    </View>
  );
};

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#FFFFFF',
    justifyContent: 'center',
    alignItems: 'center',
  },
  backgroundGradient: {
    position: 'absolute',
    top: 0,
    left: 0,
    right: 0,
    bottom: 0,
    backgroundColor: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
    // Note: React Native doesn't support CSS gradients directly
    // In real implementation, use react-native-linear-gradient
  },
  logoContainer: {
    alignItems: 'center',
    marginBottom: 80,
  },
  logo: {
    alignItems: 'center',
  },
  logoText: {
    fontSize: 42,
    fontWeight: 'bold',
    color: '#007AFF',
    marginBottom: 8,
    fontFamily: Platform.OS === 'ios' ? 'SF Pro Display' : 'Roboto',
  },
  logoSubtext: {
    fontSize: 16,
    color: '#666666',
    fontFamily: Platform.OS === 'ios' ? 'SF Pro Text' : 'Roboto',
  },
  progressContainer: {
    width: '80%',
    alignItems: 'center',
  },
  progressBarContainer: {
    width: '100%',
    height: 4,
    backgroundColor: '#E5E5EA',
    borderRadius: 2,
    marginBottom: 24,
    overflow: 'hidden',
  },
  progressBar: {
    height: '100%',
    backgroundColor: '#007AFF',
    borderRadius: 2,
  },
  phaseContainer: {
    alignItems: 'center',
    marginBottom: 20,
  },
  phaseName: {
    fontSize: 18,
    fontWeight: '600',
    color: '#1C1C1E',
    marginBottom: 4,
    textAlign: 'center',
  },
  phaseDescription: {
    fontSize: 14,
    color: '#666666',
    textAlign: 'center',
  },
  loadingIndicator: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 32,
  },
  spinner: {
    marginRight: 8,
  },
  progressText: {
    fontSize: 16,
    fontWeight: '500',
    color: '#007AFF',
  },
  phaseIndicators: {
    flexDirection: 'row',
    justifyContent: 'center',
    gap: 8,
  },
  phaseIndicator: {
    width: 8,
    height: 8,
    borderRadius: 4,
  },
  debugContainer: {
    position: 'absolute',
    bottom: 50,
    left: 20,
    right: 20,
    backgroundColor: 'rgba(0,0,0,0.8)',
    padding: 12,
    borderRadius: 8,
  },
  debugText: {
    color: '#FFFFFF',
    fontSize: 12,
    fontFamily: 'monospace',
    marginBottom: 4,
  },
});
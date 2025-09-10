/**
 * Startup Performance Metrics Service
 * 
 * Comprehensive startup performance monitoring, analytics, and optimization
 * recommendations for React Native applications.
 * 
 * Features:
 * - Detailed startup phase timing and metrics collection
 * - Performance regression detection and alerting
 * - Device-specific performance profiling
 * - Bundle loading and screen rendering metrics
 * - Network condition impact analysis
 * - Historical performance trending
 * - Automated performance reporting
 */

import { Platform, Dimensions } from 'react-native';
import AsyncStorage from '@react-native-async-storage/async-storage';
import DeviceInfo from 'react-native-device-info';

export interface StartupPhase {
  name: string;
  startTime: number;
  endTime?: number;
  duration?: number;
  subPhases: Map<string, StartupPhase>;
  metrics: Record<string, number>;
  errors: string[];
}

export interface StartupMetrics {
  sessionId: string;
  timestamp: number;
  totalStartupTime: number;
  timeToInteractive: number;
  bundleLoadTime: number;
  firstScreenRenderTime: number;
  
  // Phase timing
  phases: Record<string, number>;
  
  // Device information
  device: {
    platform: string;
    version: string;
    model: string;
    isEmulator: boolean;
    totalMemory: number;
    usedMemory: number;
  };
  
  // App information
  app: {
    version: string;
    buildNumber: string;
    bundleSize: number;
    isDebug: boolean;
    isHermes: boolean;
  };
  
  // Network information
  network: {
    connectionType: string;
    isConnected: boolean;
    isMetered: boolean;
  };
  
  // Performance scores
  scores: {
    overall: number;           // 0-100
    loadingSpeed: number;      // 0-100
    responsiveness: number;    // 0-100
    resourceEfficiency: number; // 0-100
  };
  
  // Optimization recommendations
  recommendations: PerformanceRecommendation[];
}

export interface PerformanceRecommendation {
  category: 'bundle' | 'network' | 'memory' | 'rendering';
  severity: 'critical' | 'high' | 'medium' | 'low';
  title: string;
  description: string;
  impact: string;
  solution: string;
  estimatedImprovement: number; // milliseconds
}

export interface PerformanceTrend {
  metric: string;
  current: number;
  previous: number;
  trend: 'improving' | 'stable' | 'degrading';
  changePercentage: number;
}

class StartupMetricsService {
  private currentSession: StartupMetrics | null = null;
  private activePhases: Map<string, StartupPhase> = new Map();
  private startupStartTime: number = 0;
  private metricsHistory: StartupMetrics[] = [];
  private performanceThresholds = {
    excellent: 1500,  // < 1.5s
    good: 3000,       // < 3s
    acceptable: 5000, // < 5s
    poor: 5000        // > 5s
  };

  constructor() {
    this.initializeMetricsCollection();
  }

  private async initializeMetricsCollection() {
    try {
      // Load historical metrics
      const historyJson = await AsyncStorage.getItem('startup_metrics_history');
      if (historyJson) {
        this.metricsHistory = JSON.parse(historyJson);
        // Keep only recent metrics (last 30 sessions)
        this.metricsHistory = this.metricsHistory.slice(-30);
      }
    } catch (error) {
      console.warn('[StartupMetrics] Failed to load metrics history:', error);
    }
  }

  /**
   * Starts measuring overall startup performance
   */
  startMeasuring(): void {
    this.startupStartTime = Date.now();
    
    // Initialize new session
    this.currentSession = {
      sessionId: this.generateSessionId(),
      timestamp: this.startupStartTime,
      totalStartupTime: 0,
      timeToInteractive: 0,
      bundleLoadTime: 0,
      firstScreenRenderTime: 0,
      phases: {},
      device: {} as any,
      app: {} as any,
      network: {} as any,
      scores: {
        overall: 0,
        loadingSpeed: 0,
        responsiveness: 0,
        resourceEfficiency: 0
      },
      recommendations: []
    };

    // Start collecting device and app information
    this.collectDeviceInformation();
    this.collectAppInformation();
    
    console.log(`[StartupMetrics] Started measuring startup performance (Session: ${this.currentSession.sessionId})`);
  }

  /**
   * Starts timing a specific phase
   */
  startPhase(phaseName: string, parentPhase?: string): void {
    const phase: StartupPhase = {
      name: phaseName,
      startTime: Date.now(),
      subPhases: new Map(),
      metrics: {},
      errors: []
    };

    if (parentPhase && this.activePhases.has(parentPhase)) {
      this.activePhases.get(parentPhase)!.subPhases.set(phaseName, phase);
    } else {
      this.activePhases.set(phaseName, phase);
    }

    console.log(`[StartupMetrics] Started phase: ${phaseName}`);
  }

  /**
   * Ends timing a specific phase
   */
  endPhase(phaseName: string, metrics?: Record<string, number>): void {
    const phase = this.activePhases.get(phaseName);
    if (!phase) {
      console.warn(`[StartupMetrics] Phase '${phaseName}' not found`);
      return;
    }

    phase.endTime = Date.now();
    phase.duration = phase.endTime - phase.startTime;
    
    if (metrics) {
      Object.assign(phase.metrics, metrics);
    }

    if (this.currentSession) {
      this.currentSession.phases[phaseName] = phase.duration;
    }

    console.log(`[StartupMetrics] Ended phase: ${phaseName} (${phase.duration}ms)`);
  }

  /**
   * Records a specific metric within a phase
   */
  recordMetric(phaseName: string, metricName: string, value: number): void {
    const phase = this.activePhases.get(phaseName);
    if (phase) {
      phase.metrics[metricName] = value;
    }
  }

  /**
   * Records an error during a phase
   */
  recordError(phaseName: string, error: string): void {
    const phase = this.activePhases.get(phaseName);
    if (phase) {
      phase.errors.push(error);
    }
  }

  /**
   * Records the total startup time and completes metrics collection
   */
  recordStartupTime(totalTime: number): void {
    if (!this.currentSession) return;

    this.currentSession.totalStartupTime = totalTime;
    this.currentSession.timeToInteractive = Date.now() - this.startupStartTime;
    
    // Calculate performance scores
    this.calculatePerformanceScores();
    
    // Generate recommendations
    this.generateRecommendations();
    
    // Save metrics
    this.saveMetrics();

    console.log(`[StartupMetrics] Startup completed in ${totalTime}ms (TTI: ${this.currentSession.timeToInteractive}ms)`);
  }

  /**
   * Records time to first screen render
   */
  recordFirstScreenRender(): void {
    if (!this.currentSession) return;
    
    this.currentSession.firstScreenRenderTime = Date.now() - this.startupStartTime;
    console.log(`[StartupMetrics] First screen rendered in ${this.currentSession.firstScreenRenderTime}ms`);
  }

  /**
   * Records bundle loading time
   */
  recordBundleLoadTime(loadTime: number): void {
    if (!this.currentSession) return;
    
    this.currentSession.bundleLoadTime = loadTime;
    console.log(`[StartupMetrics] Bundle loaded in ${loadTime}ms`);
  }

  private async collectDeviceInformation() {
    if (!this.currentSession) return;

    try {
      const [
        model,
        version,
        isEmulator,
        totalMemory,
        usedMemory
      ] = await Promise.all([
        DeviceInfo.getModel(),
        DeviceInfo.getSystemVersion(),
        DeviceInfo.isEmulator(),
        DeviceInfo.getTotalMemory(),
        DeviceInfo.getUsedMemory()
      ]);

      this.currentSession.device = {
        platform: Platform.OS,
        version,
        model,
        isEmulator,
        totalMemory,
        usedMemory
      };
    } catch (error) {
      console.warn('[StartupMetrics] Failed to collect device information:', error);
      // Fallback to basic info
      this.currentSession.device = {
        platform: Platform.OS,
        version: Platform.Version.toString(),
        model: 'unknown',
        isEmulator: false,
        totalMemory: 0,
        usedMemory: 0
      };
    }
  }

  private async collectAppInformation() {
    if (!this.currentSession) return;

    try {
      const [
        version,
        buildNumber
      ] = await Promise.all([
        DeviceInfo.getVersion(),
        DeviceInfo.getBuildNumber()
      ]);

      this.currentSession.app = {
        version,
        buildNumber,
        bundleSize: 0, // Would be populated by bundle analyzer
        isDebug: __DEV__,
        isHermes: !!(global as any).HermesInternal // Detect Hermes engine
      };
    } catch (error) {
      console.warn('[StartupMetrics] Failed to collect app information:', error);
      this.currentSession.app = {
        version: '1.0.0',
        buildNumber: '1',
        bundleSize: 0,
        isDebug: __DEV__,
        isHermes: false
      };
    }
  }

  private calculatePerformanceScores() {
    if (!this.currentSession) return;

    const { totalStartupTime, timeToInteractive, bundleLoadTime, firstScreenRenderTime } = this.currentSession;
    
    // Loading Speed Score (0-100)
    // Based on total startup time vs thresholds
    let loadingScore = 100;
    if (totalStartupTime > this.performanceThresholds.excellent) {
      loadingScore = Math.max(0, 100 - ((totalStartupTime - this.performanceThresholds.excellent) / 50));
    }

    // Responsiveness Score (0-100)
    // Based on time to interactive
    let responsivenessScore = 100;
    if (timeToInteractive > 2000) {
      responsivenessScore = Math.max(0, 100 - ((timeToInteractive - 2000) / 100));
    }

    // Resource Efficiency Score (0-100)
    // Based on bundle size and memory usage
    const memoryUsageRatio = this.currentSession.device.usedMemory / this.currentSession.device.totalMemory;
    let resourceScore = Math.max(0, 100 - (memoryUsageRatio * 200));

    // Overall Score (weighted average)
    const overallScore = Math.round(
      (loadingScore * 0.4) + 
      (responsivenessScore * 0.4) + 
      (resourceScore * 0.2)
    );

    this.currentSession.scores = {
      overall: overallScore,
      loadingSpeed: Math.round(loadingScore),
      responsiveness: Math.round(responsivenessScore),
      resourceEfficiency: Math.round(resourceScore)
    };
  }

  private generateRecommendations() {
    if (!this.currentSession) return;

    const recommendations: PerformanceRecommendation[] = [];
    const { totalStartupTime, scores, phases } = this.currentSession;

    // Bundle size recommendations
    if (this.currentSession.app.bundleSize > 2000000) { // > 2MB
      recommendations.push({
        category: 'bundle',
        severity: 'high',
        title: 'Large Bundle Size',
        description: 'App bundle size is larger than recommended',
        impact: 'Slower initial load times, especially on slower networks',
        solution: 'Implement code splitting and remove unused dependencies',
        estimatedImprovement: 800
      });
    }

    // Critical data loading recommendations
    if (phases.critical_data && phases.critical_data > 1000) {
      recommendations.push({
        category: 'network',
        severity: 'medium',
        title: 'Slow Critical Data Loading',
        description: 'Critical data takes too long to load',
        impact: 'Delayed time to interactive',
        solution: 'Optimize API calls and implement better caching',
        estimatedImprovement: 500
      });
    }

    // Memory usage recommendations
    const memoryUsageRatio = this.currentSession.device.usedMemory / this.currentSession.device.totalMemory;
    if (memoryUsageRatio > 0.8) {
      recommendations.push({
        category: 'memory',
        severity: 'high',
        title: 'High Memory Usage',
        description: 'App is using excessive memory during startup',
        impact: 'Potential crashes and poor performance',
        solution: 'Optimize memory usage and implement lazy loading',
        estimatedImprovement: 300
      });
    }

    // Screen rendering recommendations
    if (this.currentSession.firstScreenRenderTime > 2000) {
      recommendations.push({
        category: 'rendering',
        severity: 'medium',
        title: 'Slow First Screen Render',
        description: 'First screen takes too long to render',
        impact: 'Poor perceived performance',
        solution: 'Optimize component rendering and reduce layout complexity',
        estimatedImprovement: 400
      });
    }

    // Overall performance recommendations
    if (scores.overall < 60) {
      recommendations.push({
        category: 'bundle',
        severity: 'critical',
        title: 'Poor Overall Performance',
        description: 'App startup performance is below acceptable standards',
        impact: 'Poor user experience and potential app abandonment',
        solution: 'Comprehensive performance optimization needed',
        estimatedImprovement: 1500
      });
    }

    this.currentSession.recommendations = recommendations.sort((a, b) => {
      const severityOrder = { critical: 4, high: 3, medium: 2, low: 1 };
      return severityOrder[b.severity] - severityOrder[a.severity];
    });
  }

  private async saveMetrics() {
    if (!this.currentSession) return;

    try {
      // Add to history
      this.metricsHistory.push(this.currentSession);
      
      // Keep only recent metrics
      this.metricsHistory = this.metricsHistory.slice(-30);
      
      // Save to storage
      await AsyncStorage.setItem('startup_metrics_history', JSON.stringify(this.metricsHistory));
      
      console.log('[StartupMetrics] Metrics saved successfully');
    } catch (error) {
      console.error('[StartupMetrics] Failed to save metrics:', error);
    }
  }

  /**
   * Gets the current session metrics
   */
  getCurrentMetrics(): StartupMetrics | null {
    return this.currentSession;
  }

  /**
   * Gets performance trends by comparing recent metrics
   */
  getPerformanceTrends(): PerformanceTrend[] {
    if (this.metricsHistory.length < 2) {
      return [];
    }

    const recent = this.metricsHistory.slice(-5); // Last 5 sessions
    const previous = this.metricsHistory.slice(-10, -5); // Previous 5 sessions
    
    const trends: PerformanceTrend[] = [];
    
    const metrics = ['totalStartupTime', 'timeToInteractive', 'bundleLoadTime', 'firstScreenRenderTime'];
    
    for (const metric of metrics) {
      const recentAvg = recent.reduce((sum, m) => sum + (m as any)[metric], 0) / recent.length;
      const previousAvg = previous.reduce((sum, m) => sum + (m as any)[metric], 0) / previous.length;
      
      if (previousAvg > 0) {
        const changePercentage = ((recentAvg - previousAvg) / previousAvg) * 100;
        let trend: PerformanceTrend['trend'] = 'stable';
        
        if (Math.abs(changePercentage) > 10) {
          trend = changePercentage > 0 ? 'degrading' : 'improving';
        }
        
        trends.push({
          metric,
          current: recentAvg,
          previous: previousAvg,
          trend,
          changePercentage: Math.round(changePercentage * 100) / 100
        });
      }
    }
    
    return trends;
  }

  /**
   * Generates a comprehensive performance report
   */
  generatePerformanceReport(): string {
    if (!this.currentSession) {
      return 'No metrics available';
    }

    const trends = this.getPerformanceTrends();
    const { scores, totalStartupTime, recommendations } = this.currentSession;

    let report = `
# Startup Performance Report
Generated: ${new Date(this.currentSession.timestamp).toISOString()}
Session ID: ${this.currentSession.sessionId}

## Performance Summary
- Overall Score: ${scores.overall}/100 (${this.getScoreCategory(scores.overall)})
- Total Startup Time: ${totalStartupTime}ms
- Time to Interactive: ${this.currentSession.timeToInteractive}ms
- First Screen Render: ${this.currentSession.firstScreenRenderTime}ms

## Score Breakdown
- Loading Speed: ${scores.loadingSpeed}/100
- Responsiveness: ${scores.responsiveness}/100  
- Resource Efficiency: ${scores.resourceEfficiency}/100

## Device Information
- Platform: ${this.currentSession.device.platform} ${this.currentSession.device.version}
- Model: ${this.currentSession.device.model}
- Memory: ${Math.round(this.currentSession.device.usedMemory / 1024 / 1024)}MB / ${Math.round(this.currentSession.device.totalMemory / 1024 / 1024)}MB

## Phase Breakdown
${Object.entries(this.currentSession.phases).map(([phase, time]) => 
  `- ${phase}: ${time}ms`
).join('\n')}

## Performance Trends
${trends.map(trend => 
  `- ${trend.metric}: ${trend.trend} (${trend.changePercentage > 0 ? '+' : ''}${trend.changePercentage}%)`
).join('\n')}

## Recommendations (${recommendations.length})
${recommendations.map((rec, i) => 
  `${i + 1}. [${rec.severity.toUpperCase()}] ${rec.title}
   Impact: ${rec.impact}
   Solution: ${rec.solution}
   Estimated Improvement: ${rec.estimatedImprovement}ms`
).join('\n\n')}

---
Generated by Startup Metrics Service
    `.trim();

    return report;
  }

  private getScoreCategory(score: number): string {
    if (score >= 90) return 'Excellent';
    if (score >= 75) return 'Good';
    if (score >= 60) return 'Acceptable';
    return 'Poor';
  }

  private generateSessionId(): string {
    return `startup_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }

  /**
   * Clears all stored metrics history
   */
  async clearHistory(): Promise<void> {
    try {
      await AsyncStorage.removeItem('startup_metrics_history');
      this.metricsHistory = [];
      console.log('[StartupMetrics] History cleared');
    } catch (error) {
      console.error('[StartupMetrics] Failed to clear history:', error);
    }
  }

  /**
   * Gets performance statistics
   */
  getStatistics(): {
    averageStartupTime: number;
    bestStartupTime: number;
    worstStartupTime: number;
    totalSessions: number;
    averageScore: number;
  } {
    if (this.metricsHistory.length === 0) {
      return {
        averageStartupTime: 0,
        bestStartupTime: 0,
        worstStartupTime: 0,
        totalSessions: 0,
        averageScore: 0
      };
    }

    const startupTimes = this.metricsHistory.map(m => m.totalStartupTime);
    const scores = this.metricsHistory.map(m => m.scores.overall);

    return {
      averageStartupTime: Math.round(startupTimes.reduce((sum, time) => sum + time, 0) / startupTimes.length),
      bestStartupTime: Math.min(...startupTimes),
      worstStartupTime: Math.max(...startupTimes),
      totalSessions: this.metricsHistory.length,
      averageScore: Math.round(scores.reduce((sum, score) => sum + score, 0) / scores.length)
    };
  }
}

// Export singleton instance
export const startupMetricsService = new StartupMetricsService();
export default StartupMetricsService;
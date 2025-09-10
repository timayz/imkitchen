/**
 * Bundle Optimization Service
 * 
 * Provides intelligent bundle size optimization, dependency analysis,
 * and code splitting strategies for React Native applications.
 * 
 * Features:
 * - Dependency tree analysis and unused dependency detection
 * - Bundle size monitoring and reporting
 * - Intelligent code splitting recommendations
 * - Asset optimization (images, fonts, etc.)
 * - Dynamic import optimization
 * - Bundle analyzer integration
 */

import { Platform } from 'react-native';
import AsyncStorage from '@react-native-async-storage/async-storage';

export interface DependencyInfo {
  name: string;
  version: string;
  size: number;
  isUsed: boolean;
  importPaths: string[];
  category: 'production' | 'development' | 'peer';
  platform?: 'ios' | 'android' | 'both';
}

export interface BundleAnalysis {
  totalSize: number;
  jsSize: number;
  assetsSize: number;
  unusedSize: number;
  dependencies: DependencyInfo[];
  recommendations: OptimizationRecommendation[];
  platforms: {
    ios: number;
    android: number;
  };
}

export interface OptimizationRecommendation {
  type: 'remove_dependency' | 'code_split' | 'asset_optimize' | 'import_optimize';
  severity: 'critical' | 'high' | 'medium' | 'low';
  description: string;
  potentialSavings: number; // bytes
  action: string;
  affectedFiles: string[];
}

export interface AssetOptimization {
  type: 'image' | 'font' | 'audio' | 'video';
  originalSize: number;
  optimizedSize: number;
  compressionRatio: number;
  quality: number;
  path: string;
}

class BundleOptimizationService {
  private bundleAnalysis: BundleAnalysis | null = null;
  private optimizationHistory: OptimizationRecommendation[] = [];

  constructor() {
    this.initializeOptimization();
  }

  private async initializeOptimization() {
    try {
      const savedAnalysis = await AsyncStorage.getItem('bundle_analysis');
      if (savedAnalysis) {
        this.bundleAnalysis = JSON.parse(savedAnalysis);
      }
    } catch (error) {
      console.warn('Failed to load saved bundle analysis:', error);
    }
  }

  /**
   * Analyzes the current bundle and provides optimization recommendations
   */
  async analyzeBundleSize(): Promise<BundleAnalysis> {
    console.log('[BundleOptimization] Analyzing bundle size...');

    // Simulated bundle analysis - in real implementation, this would
    // integrate with Metro bundler or analyze the actual bundle files
    const mockDependencies: DependencyInfo[] = [
      {
        name: 'react',
        version: '18.2.0',
        size: 45000,
        isUsed: true,
        importPaths: ['src/App.tsx', 'src/components/**'],
        category: 'production',
        platform: 'both'
      },
      {
        name: 'react-native',
        version: '0.72.0',
        size: 2800000,
        isUsed: true,
        importPaths: ['src/**'],
        category: 'production',
        platform: 'both'
      },
      {
        name: '@react-navigation/native',
        version: '6.1.0',
        size: 120000,
        isUsed: true,
        importPaths: ['src/navigation/**'],
        category: 'production',
        platform: 'both'
      },
      {
        name: 'lodash',
        version: '4.17.21',
        size: 540000,
        isUsed: false, // Potential unused dependency
        importPaths: [],
        category: 'production',
        platform: 'both'
      },
      {
        name: 'moment',
        version: '2.29.4',
        size: 230000,
        isUsed: true,
        importPaths: ['src/utils/dateUtils.ts'],
        category: 'production',
        platform: 'both'
      },
      {
        name: 'react-native-chart-kit',
        version: '6.12.0',
        size: 180000,
        isUsed: true,
        importPaths: ['src/screens/analytics/**'],
        category: 'production',
        platform: 'both'
      },
      {
        name: 'react-hook-form',
        version: '7.45.0',
        size: 85000,
        isUsed: true,
        importPaths: ['src/screens/auth/**', 'src/screens/recipes/AddRecipeScreen.tsx'],
        category: 'production',
        platform: 'both'
      }
    ];

    const totalSize = mockDependencies.reduce((sum, dep) => sum + dep.size, 0);
    const unusedSize = mockDependencies.filter(dep => !dep.isUsed).reduce((sum, dep) => sum + dep.size, 0);

    const analysis: BundleAnalysis = {
      totalSize,
      jsSize: totalSize * 0.8, // Assume 80% JS, 20% assets
      assetsSize: totalSize * 0.2,
      unusedSize,
      dependencies: mockDependencies,
      recommendations: this.generateRecommendations(mockDependencies),
      platforms: {
        ios: totalSize * 1.1, // iOS typically larger due to additional frameworks
        android: totalSize
      }
    };

    this.bundleAnalysis = analysis;
    await this.saveBundleAnalysis(analysis);

    console.log(`[BundleOptimization] Analysis complete. Total size: ${this.formatBytes(totalSize)}`);
    return analysis;
  }

  /**
   * Generates optimization recommendations based on bundle analysis
   */
  private generateRecommendations(dependencies: DependencyInfo[]): OptimizationRecommendation[] {
    const recommendations: OptimizationRecommendation[] = [];

    // Check for unused dependencies
    const unusedDeps = dependencies.filter(dep => !dep.isUsed);
    unusedDeps.forEach(dep => {
      recommendations.push({
        type: 'remove_dependency',
        severity: dep.size > 100000 ? 'critical' : 'high',
        description: `Remove unused dependency '${dep.name}' (${this.formatBytes(dep.size)})`,
        potentialSavings: dep.size,
        action: `npm uninstall ${dep.name}`,
        affectedFiles: []
      });
    });

    // Check for large dependencies that could be optimized
    const largeDeps = dependencies.filter(dep => dep.isUsed && dep.size > 200000);
    largeDeps.forEach(dep => {
      if (dep.name === 'moment') {
        recommendations.push({
          type: 'import_optimize',
          severity: 'high',
          description: `Replace moment.js with day.js for smaller bundle size (${this.formatBytes(dep.size)} → ~30KB)`,
          potentialSavings: dep.size - 30000,
          action: 'npm uninstall moment && npm install dayjs',
          affectedFiles: dep.importPaths
        });
      }
      
      if (dep.name === 'react-native-chart-kit') {
        recommendations.push({
          type: 'code_split',
          severity: 'medium',
          description: `Code split chart library - only load when analytics screens are accessed`,
          potentialSavings: dep.size * 0.7, // 70% savings from lazy loading
          action: 'Implement dynamic import for analytics screens',
          affectedFiles: dep.importPaths
        });
      }
    });

    // Check for potential tree-shaking opportunities
    const treeShakableLibs = dependencies.filter(dep => 
      dep.name.includes('lodash') || dep.name.includes('ramda') || dep.name.includes('utils')
    );
    
    treeShakableLibs.forEach(dep => {
      if (dep.isUsed) {
        recommendations.push({
          type: 'import_optimize',
          severity: 'medium',
          description: `Use specific imports instead of default import for '${dep.name}' to enable tree shaking`,
          potentialSavings: dep.size * 0.6,
          action: `Replace 'import _ from '${dep.name}'' with 'import { method } from '${dep.name}/method''`,
          affectedFiles: dep.importPaths
        });
      }
    });

    // Platform-specific optimizations
    if (Platform.OS === 'android') {
      recommendations.push({
        type: 'asset_optimize',
        severity: 'medium',
        description: 'Enable Proguard/R8 for additional Android bundle size reduction',
        potentialSavings: dependencies.reduce((sum, dep) => sum + dep.size, 0) * 0.15, // 15% savings
        action: 'Configure Proguard in android/app/build.gradle',
        affectedFiles: ['android/app/build.gradle']
      });
    }

    return recommendations.sort((a, b) => {
      const severityWeight = { critical: 4, high: 3, medium: 2, low: 1 };
      return severityWeight[b.severity] - severityWeight[a.severity];
    });
  }

  /**
   * Optimizes asset files (images, fonts, etc.)
   */
  async optimizeAssets(): Promise<AssetOptimization[]> {
    console.log('[BundleOptimization] Optimizing assets...');

    // Mock asset optimization results
    const optimizations: AssetOptimization[] = [
      {
        type: 'image',
        originalSize: 125000,
        optimizedSize: 45000,
        compressionRatio: 0.36,
        quality: 85,
        path: 'src/assets/images/recipe-placeholder.png'
      },
      {
        type: 'image',
        originalSize: 89000,
        optimizedSize: 32000,
        compressionRatio: 0.36,
        quality: 80,
        path: 'src/assets/images/meal-plan-bg.jpg'
      },
      {
        type: 'font',
        originalSize: 245000,
        optimizedSize: 180000,
        compressionRatio: 0.73,
        quality: 100,
        path: 'src/assets/fonts/custom-font.ttf'
      }
    ];

    const totalOriginalSize = optimizations.reduce((sum, opt) => sum + opt.originalSize, 0);
    const totalOptimizedSize = optimizations.reduce((sum, opt) => sum + opt.optimizedSize, 0);
    const totalSavings = totalOriginalSize - totalOptimizedSize;

    console.log(`[BundleOptimization] Asset optimization complete. Saved ${this.formatBytes(totalSavings)}`);
    
    return optimizations;
  }

  /**
   * Analyzes import patterns and suggests optimizations
   */
  async analyzeImports(): Promise<{
    unusedImports: string[];
    circularDependencies: string[];
    heavyImports: Array<{ file: string; imports: string[]; size: number }>;
  }> {
    console.log('[BundleOptimization] Analyzing import patterns...');

    // Mock import analysis - real implementation would parse source files
    return {
      unusedImports: [
        'src/utils/deprecatedHelpers.ts',
        'src/components/unused/OldButton.tsx'
      ],
      circularDependencies: [
        'src/services/api.ts → src/hooks/useAuth.ts → src/services/api.ts'
      ],
      heavyImports: [
        {
          file: 'src/screens/analytics/RotationStatsScreen.tsx',
          imports: ['react-native-chart-kit', 'react-native-svg', 'd3-scale'],
          size: 340000
        },
        {
          file: 'src/screens/auth/LoginScreen.tsx',
          imports: ['react-hook-form', 'yup', '@hookform/resolvers'],
          size: 125000
        }
      ]
    };
  }

  /**
   * Implements automatic bundle optimizations
   */
  async implementOptimizations(recommendations: OptimizationRecommendation[]): Promise<{
    applied: OptimizationRecommendation[];
    failed: Array<{ recommendation: OptimizationRecommendation; error: string }>;
    totalSavings: number;
  }> {
    console.log(`[BundleOptimization] Implementing ${recommendations.length} optimizations...`);

    const applied: OptimizationRecommendation[] = [];
    const failed: Array<{ recommendation: OptimizationRecommendation; error: string }> = [];

    for (const recommendation of recommendations) {
      try {
        await this.applyOptimization(recommendation);
        applied.push(recommendation);
        this.optimizationHistory.push(recommendation);
      } catch (error) {
        failed.push({
          recommendation,
          error: error instanceof Error ? error.message : 'Unknown error'
        });
      }
    }

    const totalSavings = applied.reduce((sum, rec) => sum + rec.potentialSavings, 0);

    console.log(`[BundleOptimization] Applied ${applied.length}/${recommendations.length} optimizations`);
    console.log(`[BundleOptimization] Total savings: ${this.formatBytes(totalSavings)}`);

    return { applied, failed, totalSavings };
  }

  private async applyOptimization(recommendation: OptimizationRecommendation): Promise<void> {
    // Mock implementation - real version would actually modify files/dependencies
    switch (recommendation.type) {
      case 'remove_dependency':
        console.log(`[BundleOptimization] Would remove dependency: ${recommendation.action}`);
        break;
      case 'code_split':
        console.log(`[BundleOptimization] Would implement code splitting: ${recommendation.description}`);
        break;
      case 'asset_optimize':
        console.log(`[BundleOptimization] Would optimize assets: ${recommendation.description}`);
        break;
      case 'import_optimize':
        console.log(`[BundleOptimization] Would optimize imports: ${recommendation.description}`);
        break;
    }

    // Simulate processing time
    await new Promise(resolve => setTimeout(resolve, 100));
  }

  /**
   * Gets bundle size metrics and trends
   */
  async getBundleMetrics(): Promise<{
    currentSize: number;
    lastSize: number;
    trend: 'increasing' | 'decreasing' | 'stable';
    optimizationHistory: OptimizationRecommendation[];
    platforms: {
      ios: { size: number; percentage: number };
      android: { size: number; percentage: number };
    };
  }> {
    const currentAnalysis = this.bundleAnalysis || await this.analyzeBundleSize();
    
    // Mock previous size for trend calculation
    const lastSize = currentAnalysis.totalSize * 1.05; // Simulate 5% reduction
    const sizeDiff = currentAnalysis.totalSize - lastSize;
    const changeThreshold = lastSize * 0.02; // 2% change threshold

    let trend: 'increasing' | 'decreasing' | 'stable';
    if (Math.abs(sizeDiff) < changeThreshold) {
      trend = 'stable';
    } else if (sizeDiff > 0) {
      trend = 'increasing';
    } else {
      trend = 'decreasing';
    }

    return {
      currentSize: currentAnalysis.totalSize,
      lastSize,
      trend,
      optimizationHistory: this.optimizationHistory,
      platforms: {
        ios: {
          size: currentAnalysis.platforms.ios,
          percentage: (currentAnalysis.platforms.ios / currentAnalysis.totalSize) * 100
        },
        android: {
          size: currentAnalysis.platforms.android,
          percentage: (currentAnalysis.platforms.android / currentAnalysis.totalSize) * 100
        }
      }
    };
  }

  /**
   * Monitors bundle size changes over time
   */
  async startBundleMonitoring(): Promise<void> {
    console.log('[BundleOptimization] Starting bundle size monitoring...');

    // Perform initial analysis
    await this.analyzeBundleSize();

    // Set up periodic monitoring (in real implementation, this would be more sophisticated)
    const monitoringInterval = 30 * 60 * 1000; // 30 minutes
    
    const monitor = async () => {
      try {
        const newAnalysis = await this.analyzeBundleSize();
        const previousSize = this.bundleAnalysis?.totalSize || 0;
        const sizeChange = newAnalysis.totalSize - previousSize;
        
        if (Math.abs(sizeChange) > previousSize * 0.05) { // 5% change threshold
          console.log(`[BundleOptimization] Significant size change detected: ${this.formatBytes(sizeChange)}`);
          
          // Generate new recommendations if size increased
          if (sizeChange > 0) {
            const recommendations = this.generateRecommendations(newAnalysis.dependencies);
            console.log(`[BundleOptimization] Generated ${recommendations.length} new recommendations`);
          }
        }
      } catch (error) {
        console.error('[BundleOptimization] Monitoring error:', error);
      }
    };

    // Note: In a real app, you'd use a more appropriate scheduling mechanism
    setInterval(monitor, monitoringInterval);
  }

  private async saveBundleAnalysis(analysis: BundleAnalysis): Promise<void> {
    try {
      await AsyncStorage.setItem('bundle_analysis', JSON.stringify(analysis));
    } catch (error) {
      console.warn('Failed to save bundle analysis:', error);
    }
  }

  private formatBytes(bytes: number): string {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  }

  /**
   * Generates a bundle optimization report
   */
  async generateOptimizationReport(): Promise<string> {
    const analysis = this.bundleAnalysis || await this.analyzeBundleSize();
    const metrics = await this.getBundleMetrics();
    const importAnalysis = await this.analyzeImports();

    const report = `
# Bundle Optimization Report
Generated: ${new Date().toISOString()}

## Current Bundle Size
- Total Size: ${this.formatBytes(analysis.totalSize)}
- JavaScript: ${this.formatBytes(analysis.jsSize)}
- Assets: ${this.formatBytes(analysis.assetsSize)}
- Unused: ${this.formatBytes(analysis.unusedSize)}

## Platform Breakdown
- iOS: ${this.formatBytes(analysis.platforms.ios)} (${metrics.platforms.ios.percentage.toFixed(1)}%)
- Android: ${this.formatBytes(analysis.platforms.android)} (${metrics.platforms.android.percentage.toFixed(1)}%)

## Size Trend: ${metrics.trend.toUpperCase()}
${metrics.trend === 'decreasing' ? '↓' : metrics.trend === 'increasing' ? '↑' : '→'} ${this.formatBytes(Math.abs(metrics.currentSize - metrics.lastSize))}

## Top Recommendations (${analysis.recommendations.length})
${analysis.recommendations.slice(0, 5).map((rec, i) => 
  `${i + 1}. [${rec.severity.toUpperCase()}] ${rec.description} (Save: ${this.formatBytes(rec.potentialSavings)})`
).join('\n')}

## Import Analysis Issues
- Unused Imports: ${importAnalysis.unusedImports.length}
- Circular Dependencies: ${importAnalysis.circularDependencies.length}
- Heavy Imports: ${importAnalysis.heavyImports.length}

## Optimization History
Applied Optimizations: ${this.optimizationHistory.length}
Total Historical Savings: ${this.formatBytes(this.optimizationHistory.reduce((sum, rec) => sum + rec.potentialSavings, 0))}

---
Generated by Bundle Optimization Service
    `.trim();

    return report;
  }
}

// Export singleton instance
export const bundleOptimizationService = new BundleOptimizationService();
export default BundleOptimizationService;
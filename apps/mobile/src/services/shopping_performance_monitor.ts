interface PerformanceMetric {
  operation: string;
  startTime: number;
  endTime?: number;
  duration?: number;
  metadata?: Record<string, any>;
}

interface PerformanceSummary {
  totalOperations: number;
  averageDuration: number;
  minDuration: number;
  maxDuration: number;
  p95Duration: number;
  slowOperations: PerformanceMetric[];
}

class ShoppingPerformanceMonitor {
  private metrics: Map<string, PerformanceMetric[]> = new Map();
  private activeOperations: Map<string, PerformanceMetric> = new Map();

  // Start monitoring an operation
  startOperation(operationId: string, operationName: string, metadata?: Record<string, any>): void {
    const metric: PerformanceMetric = {
      operation: operationName,
      startTime: performance.now(),
      metadata,
    };

    this.activeOperations.set(operationId, metric);
  }

  // End monitoring an operation
  endOperation(operationId: string): number | null {
    const metric = this.activeOperations.get(operationId);
    if (!metric) {
      console.warn(`No active operation found for ID: ${operationId}`);
      return null;
    }

    metric.endTime = performance.now();
    metric.duration = metric.endTime - metric.startTime;

    // Store the completed metric
    const operationMetrics = this.metrics.get(metric.operation) || [];
    operationMetrics.push(metric);
    this.metrics.set(metric.operation, operationMetrics);

    // Clean up active operations
    this.activeOperations.delete(operationId);

    // Log slow operations (>3 seconds for shopping list generation)
    if (metric.operation === 'shopping-list-generation' && metric.duration > 3000) {
      console.warn(`Slow shopping list generation detected: ${metric.duration.toFixed(2)}ms`, metric.metadata);
    }

    return metric.duration;
  }

  // Get performance summary for an operation type
  getSummary(operationType: string): PerformanceSummary | null {
    const metrics = this.metrics.get(operationType);
    if (!metrics || metrics.length === 0) {
      return null;
    }

    const durations = metrics
      .filter(m => m.duration !== undefined)
      .map(m => m.duration!)
      .sort((a, b) => a - b);

    if (durations.length === 0) {
      return null;
    }

    const totalOperations = durations.length;
    const averageDuration = durations.reduce((sum, d) => sum + d, 0) / totalOperations;
    const minDuration = durations[0];
    const maxDuration = durations[durations.length - 1];
    const p95Index = Math.floor(durations.length * 0.95);
    const p95Duration = durations[p95Index];

    // Identify slow operations (above p95)
    const slowOperations = metrics.filter(m => m.duration && m.duration >= p95Duration);

    return {
      totalOperations,
      averageDuration,
      minDuration,
      maxDuration,
      p95Duration,
      slowOperations,
    };
  }

  // Get all performance data
  getAllSummaries(): Record<string, PerformanceSummary> {
    const summaries: Record<string, PerformanceSummary> = {};
    
    for (const operationType of this.metrics.keys()) {
      const summary = this.getSummary(operationType);
      if (summary) {
        summaries[operationType] = summary;
      }
    }

    return summaries;
  }

  // Monitor shopping list generation with automatic timing
  async monitorShoppingListGeneration<T>(
    mealPlanId: string,
    mergeExisting: boolean,
    operation: () => Promise<T>
  ): Promise<T> {
    const operationId = `shopping-gen-${mealPlanId}-${Date.now()}`;
    
    this.startOperation(operationId, 'shopping-list-generation', {
      mealPlanId,
      mergeExisting,
      timestamp: new Date().toISOString(),
    });

    try {
      const result = await operation();
      const duration = this.endOperation(operationId);
      
      console.log(`Shopping list generation completed in ${duration?.toFixed(2)}ms for meal plan ${mealPlanId}`);
      
      return result;
    } catch (error) {
      this.endOperation(operationId);
      console.error(`Shopping list generation failed for meal plan ${mealPlanId}:`, error);
      throw error;
    }
  }

  // Monitor export operations
  async monitorExportOperation<T>(
    listId: string,
    format: string,
    operation: () => Promise<T>
  ): Promise<T> {
    const operationId = `export-${listId}-${format}-${Date.now()}`;
    
    this.startOperation(operationId, 'shopping-list-export', {
      listId,
      format,
      timestamp: new Date().toISOString(),
    });

    try {
      const result = await operation();
      const duration = this.endOperation(operationId);
      
      console.log(`Shopping list export (${format}) completed in ${duration?.toFixed(2)}ms`);
      
      return result;
    } catch (error) {
      this.endOperation(operationId);
      console.error(`Shopping list export failed:`, error);
      throw error;
    }
  }

  // Clear old metrics to prevent memory leaks
  clearOldMetrics(maxAge = 24 * 60 * 60 * 1000): void { // Default: 24 hours
    const cutoffTime = Date.now() - maxAge;
    
    for (const [operationType, metrics] of this.metrics.entries()) {
      const filteredMetrics = metrics.filter(m => m.startTime > cutoffTime);
      this.metrics.set(operationType, filteredMetrics);
    }
  }

  // Export performance data for analysis
  exportMetrics(): Array<{
    operation: string;
    startTime: number;
    endTime?: number;
    duration?: number;
    metadata?: Record<string, any>;
  }> {
    const allMetrics: PerformanceMetric[] = [];
    
    for (const metrics of this.metrics.values()) {
      allMetrics.push(...metrics);
    }
    
    return allMetrics.sort((a, b) => a.startTime - b.startTime);
  }

  // Check if performance is within acceptable thresholds
  checkPerformanceHealth(): {
    isHealthy: boolean;
    issues: string[];
    recommendations: string[];
  } {
    const issues: string[] = [];
    const recommendations: string[] = [];

    // Check shopping list generation performance
    const generationSummary = this.getSummary('shopping-list-generation');
    if (generationSummary) {
      if (generationSummary.averageDuration > 2000) {
        issues.push(`Average shopping list generation time is ${generationSummary.averageDuration.toFixed(0)}ms (target: <2000ms)`);
        recommendations.push('Consider optimizing ingredient aggregation logic');
      }

      if (generationSummary.p95Duration > 3000) {
        issues.push(`95th percentile generation time is ${generationSummary.p95Duration.toFixed(0)}ms (target: <3000ms)`);
        recommendations.push('Implement caching for frequently accessed recipes');
      }

      if (generationSummary.slowOperations.length > generationSummary.totalOperations * 0.1) {
        issues.push(`${((generationSummary.slowOperations.length / generationSummary.totalOperations) * 100).toFixed(1)}% of operations are slow`);
        recommendations.push('Review meal plans with many recipes or complex ingredients');
      }
    }

    // Check export performance
    const exportSummary = this.getSummary('shopping-list-export');
    if (exportSummary && exportSummary.averageDuration > 5000) {
      issues.push(`Average export time is ${exportSummary.averageDuration.toFixed(0)}ms (target: <5000ms)`);
      recommendations.push('Optimize export formatting and data serialization');
    }

    return {
      isHealthy: issues.length === 0,
      issues,
      recommendations,
    };
  }

  // Real-time performance dashboard data
  getDashboardData(): {
    recentGenerations: Array<{ timestamp: Date; duration: number; mealPlanId: string }>;
    averagePerformance: { last24h: number; last7d: number };
    cacheHitRate: number;
    slowOperationsCount: number;
  } {
    const now = Date.now();
    const last24h = now - (24 * 60 * 60 * 1000);
    const last7d = now - (7 * 24 * 60 * 60 * 1000);

    const generationMetrics = this.metrics.get('shopping-list-generation') || [];
    
    // Recent generations (last 10)
    const recentGenerations = generationMetrics
      .filter(m => m.duration !== undefined)
      .slice(-10)
      .map(m => ({
        timestamp: new Date(m.startTime),
        duration: m.duration!,
        mealPlanId: m.metadata?.mealPlanId || 'unknown',
      }));

    // Average performance
    const last24hMetrics = generationMetrics.filter(m => m.startTime > last24h && m.duration);
    const last7dMetrics = generationMetrics.filter(m => m.startTime > last7d && m.duration);
    
    const avg24h = last24hMetrics.length > 0 
      ? last24hMetrics.reduce((sum, m) => sum + m.duration!, 0) / last24hMetrics.length 
      : 0;
    
    const avg7d = last7dMetrics.length > 0 
      ? last7dMetrics.reduce((sum, m) => sum + m.duration!, 0) / last7dMetrics.length 
      : 0;

    // Slow operations count (>3 seconds)
    const slowOperationsCount = generationMetrics.filter(m => m.duration && m.duration > 3000).length;

    return {
      recentGenerations,
      averagePerformance: {
        last24h: avg24h,
        last7d: avg7d,
      },
      cacheHitRate: 0.85, // This would come from the cache service in real implementation
      slowOperationsCount,
    };
  }
}

// Create singleton instance
export const shoppingPerformanceMonitor = new ShoppingPerformanceMonitor();

// Auto-cleanup old metrics every hour
setInterval(() => {
  shoppingPerformanceMonitor.clearOldMetrics();
}, 60 * 60 * 1000);
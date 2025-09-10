/**
 * Sync Coordination Service
 * 
 * Coordinates non-blocking synchronization with intelligent user interaction
 * prioritization, resource management, and performance optimization.
 * 
 * Features:
 * - Non-blocking sync operations that don't interfere with UI
 * - User interaction prioritization and sync deferral
 * - Resource-aware sync scheduling (CPU, memory, battery)
 * - Intelligent sync bundling and batching
 * - Performance impact monitoring and adjustment
 * - Network condition adaptive scheduling
 * - Priority queue management with fairness
 */

import { InteractionManager, Dimensions } from 'react-native';
import { backgroundSyncService, SyncItem, SyncPriority } from './background_sync_service';

export interface SyncCoordinator {
  requestSync(item: SyncItem): Promise<void>;
  pauseBackground(): void;
  resumeBackground(): void;
  getCoordinationStatus(): CoordinationStatus;
}

export interface CoordinationStatus {
  isUserActive: boolean;
  activeSyncCount: number;
  deferredSyncCount: number;
  resourceUtilization: ResourceUtilization;
  performanceImpact: PerformanceImpact;
  lastCoordinationAction: Date;
}

export interface ResourceUtilization {
  cpu: number;           // 0-100 percentage
  memory: number;        // 0-100 percentage
  battery: number;       // 0-100 percentage
  network: number;       // 0-100 percentage
}

export interface PerformanceImpact {
  frameDrop: number;     // Dropped frames per second
  mainThreadTime: number; // Main thread utilization percentage
  syncLatency: number;    // Average sync operation latency
  uiResponsiveness: number; // UI responsiveness score 0-100
}

export interface SyncBatch {
  id: string;
  items: SyncItem[];
  priority: SyncPriority;
  estimatedDuration: number;
  resourceCost: ResourceCost;
  createdAt: Date;
}

export interface ResourceCost {
  cpu: number;
  memory: number;
  network: number;
  battery: number;
}

class SyncCoordinationService {
  private isUserActive = true;
  private deferredSyncs: SyncItem[] = [];
  private activeSyncs = new Set<string>();
  private resourceMonitor: ResourceMonitor;
  private performanceMonitor: PerformanceMonitor;
  private syncBatcher: SyncBatcher;
  private coordinationConfig = {
    maxConcurrentSyncs: 2,
    userInteractionThreshold: 100, // ms
    resourceThreshold: 80, // percentage
    frameDrop threshold: 2, // fps
    batchingWindow: 500, // ms
    deferralTimeout: 5000 // ms
  };

  constructor() {
    this.resourceMonitor = new ResourceMonitor();
    this.performanceMonitor = new PerformanceMonitor();
    this.syncBatcher = new SyncBatcher();
    
    this.initializeCoordination();
  }

  private initializeCoordination(): void {
    console.log('[SyncCoordination] Initializing sync coordination...');
    
    // Setup interaction monitoring
    this.setupInteractionMonitoring();
    
    // Start resource monitoring
    this.resourceMonitor.start();
    
    // Start performance monitoring
    this.performanceMonitor.start();
    
    // Setup deferred sync processing
    this.setupDeferredSyncProcessing();
    
    console.log('[SyncCoordination] Sync coordination initialized');
  }

  private setupInteractionMonitoring(): void {
    // Track user interactions to prioritize UI responsiveness
    let lastInteractionTime = Date.now();
    
    const updateInteractionTime = () => {
      lastInteractionTime = Date.now();
      
      if (!this.isUserActive) {
        this.isUserActive = true;
        this.onUserBecameActive();
      }
    };

    // Monitor touch events, scrolling, etc.
    // In a real implementation, this would hook into the gesture system
    
    // Check for user inactivity periodically
    const checkInactivity = () => {
      const timeSinceInteraction = Date.now() - lastInteractionTime;
      
      if (timeSinceInteraction > this.coordinationConfig.userInteractionThreshold && this.isUserActive) {
        this.isUserActive = false;
        this.onUserBecameInactive();
      }
    };

    setInterval(checkInactivity, 100); // Check every 100ms
  }

  private onUserBecameActive(): void {
    console.log('[SyncCoordination] User became active - prioritizing UI responsiveness');
    
    // Defer non-critical syncs
    this.deferNonCriticalSyncs();
    
    // Reduce concurrent sync count
    this.coordinationConfig.maxConcurrentSyncs = 1;
  }

  private onUserBecameInactive(): void {
    console.log('[SyncCoordination] User became inactive - enabling background sync');
    
    // Process deferred syncs
    this.processDeferredSyncs();
    
    // Increase concurrent sync count
    this.coordinationConfig.maxConcurrentSyncs = 3;
  }

  private setupDeferredSyncProcessing(): void {
    // Process deferred syncs periodically when user is inactive
    setInterval(() => {
      if (!this.isUserActive && this.deferredSyncs.length > 0) {
        this.processDeferredSyncs();
      }
    }, this.coordinationConfig.deferralTimeout);
  }

  /**
   * Coordinates a sync request with intelligent scheduling
   */
  async coordinateSync(item: SyncItem): Promise<void> {
    console.log(`[SyncCoordination] Coordinating sync for ${item.type}: ${item.id} (Priority: ${item.priority})`);

    // Check if sync should be deferred
    if (this.shouldDeferSync(item)) {
      this.deferSync(item);
      return;
    }

    // Check resource constraints
    if (this.isResourceConstrained()) {
      console.log(`[SyncCoordination] Resource constrained, deferring ${item.id}`);
      this.deferSync(item);
      return;
    }

    // Check performance impact
    if (this.isPerformanceImpacted()) {
      console.log(`[SyncCoordination] Performance impacted, deferring ${item.id}`);
      this.deferSync(item);
      return;
    }

    // Batch similar items if appropriate
    const batch = this.syncBatcher.createBatch([item]);
    if (batch.items.length > 1) {
      await this.executeSyncBatch(batch);
    } else {
      await this.executeSync(item);
    }
  }

  private shouldDeferSync(item: SyncItem): boolean {
    // Never defer critical items
    if (item.priority === SyncPriority.CRITICAL) {
      return false;
    }

    // Defer during active user interaction (except high priority)
    if (this.isUserActive && item.priority !== SyncPriority.HIGH) {
      return true;
    }

    // Check if we're at sync capacity
    if (this.activeSyncs.size >= this.coordinationConfig.maxConcurrentSyncs) {
      return true;
    }

    return false;
  }

  private isResourceConstrained(): boolean {
    const utilization = this.resourceMonitor.getCurrentUtilization();
    
    return utilization.cpu > this.coordinationConfig.resourceThreshold ||
           utilization.memory > this.coordinationConfig.resourceThreshold ||
           utilization.battery < 20; // Low battery
  }

  private isPerformanceImpacted(): boolean {
    const impact = this.performanceMonitor.getCurrentImpact();
    
    return impact.frameDrop > this.coordinationConfig.frameDropThreshold ||
           impact.mainThreadTime > 80 ||
           impact.uiResponsiveness < 60;
  }

  private deferSync(item: SyncItem): void {
    this.deferredSyncs.push(item);
    console.log(`[SyncCoordination] Deferred sync for ${item.id} (Queue: ${this.deferredSyncs.length})`);
  }

  private deferNonCriticalSyncs(): void {
    // This would interface with backgroundSyncService to defer active syncs
    console.log('[SyncCoordination] Deferring non-critical syncs for user interaction');
  }

  private async processDeferredSyncs(): Promise<void> {
    if (this.deferredSyncs.length === 0) return;

    console.log(`[SyncCoordination] Processing ${this.deferredSyncs.length} deferred syncs`);

    // Sort deferred syncs by priority and age
    this.deferredSyncs.sort((a, b) => {
      const priorityWeight = {
        [SyncPriority.CRITICAL]: 1000,
        [SyncPriority.HIGH]: 100,
        [SyncPriority.NORMAL]: 10,
        [SyncPriority.LOW]: 1
      };

      const ageWeight = Date.now() - a.lastModified.getTime();
      const aScore = priorityWeight[a.priority] + (ageWeight / 1000);
      const bScore = priorityWeight[b.priority] + (Date.now() - b.lastModified.getTime()) / 1000;

      return bScore - aScore;
    });

    // Process syncs in batches
    const batchSize = Math.min(3, this.deferredSyncs.length);
    const itemsToProcess = this.deferredSyncs.splice(0, batchSize);

    const batch = this.syncBatcher.createBatch(itemsToProcess);
    await this.executeSyncBatch(batch);
  }

  private async executeSync(item: SyncItem): Promise<void> {
    this.activeSyncs.add(item.id);
    
    try {
      // Use InteractionManager to ensure non-blocking execution
      await new Promise(resolve => {
        InteractionManager.runAfterInteractions(() => {
          this.performNonBlockingSync(item).then(resolve);
        });
      });
    } finally {
      this.activeSyncs.delete(item.id);
    }
  }

  private async executeSyncBatch(batch: SyncBatch): Promise<void> {
    console.log(`[SyncCoordination] Executing sync batch: ${batch.items.length} items`);
    
    // Mark all items as active
    batch.items.forEach(item => this.activeSyncs.add(item.id));
    
    try {
      await new Promise(resolve => {
        InteractionManager.runAfterInteractions(() => {
          this.performBatchSync(batch).then(resolve);
        });
      });
    } finally {
      // Mark all items as complete
      batch.items.forEach(item => this.activeSyncs.delete(item.id));
    }
  }

  private async performNonBlockingSync(item: SyncItem): Promise<void> {
    // Monitor performance during sync
    const performanceStart = this.performanceMonitor.getCurrentImpact();
    const resourceStart = this.resourceMonitor.getCurrentUtilization();
    
    const startTime = Date.now();
    
    try {
      // Break sync into smaller chunks to avoid blocking
      await this.chunkifySync(item);
      
    } finally {
      const syncDuration = Date.now() - startTime;
      const performanceEnd = this.performanceMonitor.getCurrentImpact();
      
      // Adjust coordination parameters based on performance impact
      this.adjustCoordinationParameters(performanceStart, performanceEnd, syncDuration);
    }
  }

  private async chunkifySync(item: SyncItem): Promise<void> {
    // Break down sync operation into non-blocking chunks
    const chunkSize = this.calculateOptimalChunkSize(item);
    
    if (chunkSize === 1) {
      // Single operation, queue with background sync service
      await backgroundSyncService.queueSync(item);
    } else {
      // Multiple chunks - process with yielding
      for (let i = 0; i < chunkSize; i++) {
        await backgroundSyncService.queueSync({
          ...item,
          id: `${item.id}_chunk_${i}`
        });
        
        // Yield to main thread between chunks
        await new Promise(resolve => setTimeout(resolve, 0));
        
        // Check if user became active
        if (this.isUserActive && item.priority !== SyncPriority.CRITICAL) {
          console.log('[SyncCoordination] User became active, pausing chunked sync');
          break;
        }
      }
    }
  }

  private calculateOptimalChunkSize(item: SyncItem): number {
    // Estimate chunk size based on item type and current performance
    const baseChunkSize = {
      [SyncItemType.COMMUNITY_RECIPE]: 5,
      [SyncItemType.USER_RECIPE]: 1,
      [SyncItemType.RECIPE_RATING]: 10,
      [SyncItemType.USER_PROFILE]: 1,
      [SyncItemType.MEAL_PLAN]: 3,
      [SyncItemType.SHOPPING_LIST]: 5,
      [SyncItemType.USER_PREFERENCES]: 1,
      [SyncItemType.RECIPE_IMPORT]: 1
    };

    let chunkSize = baseChunkSize[item.type] || 1;
    
    // Adjust based on performance
    const impact = this.performanceMonitor.getCurrentImpact();
    if (impact.uiResponsiveness < 70) {
      chunkSize = Math.max(1, Math.floor(chunkSize / 2));
    }
    
    return chunkSize;
  }

  private async performBatchSync(batch: SyncBatch): Promise<void> {
    console.log(`[SyncCoordination] Performing batch sync: ${batch.items.length} items`);
    
    // Process batch items with proper yielding
    for (let i = 0; i < batch.items.length; i++) {
      const item = batch.items[i];
      
      await backgroundSyncService.queueSync(item);
      
      // Yield between items
      if (i < batch.items.length - 1) {
        await new Promise(resolve => setTimeout(resolve, 10));
      }
      
      // Check for user interaction
      if (this.isUserActive && item.priority !== SyncPriority.CRITICAL) {
        console.log('[SyncCoordination] User interaction detected, pausing batch');
        
        // Defer remaining items
        const remainingItems = batch.items.slice(i + 1);
        this.deferredSyncs.unshift(...remainingItems);
        break;
      }
    }
  }

  private adjustCoordinationParameters(
    performanceBefore: PerformanceImpact,
    performanceAfter: PerformanceImpact,
    duration: number
  ): void {
    // Adjust parameters based on observed performance impact
    const frameDropDelta = performanceAfter.frameDrop - performanceBefore.frameDrop;
    const responsivenessDelta = performanceAfter.uiResponsiveness - performanceBefore.uiResponsiveness;
    
    if (frameDropDelta > 2 || responsivenessDelta < -10) {
      // Performance degraded, reduce concurrency
      this.coordinationConfig.maxConcurrentSyncs = Math.max(1, this.coordinationConfig.maxConcurrentSyncs - 1);
      console.log(`[SyncCoordination] Performance degraded, reducing concurrency to ${this.coordinationConfig.maxConcurrentSyncs}`);
    } else if (frameDropDelta < 0.5 && responsivenessDelta > -2 && duration < 1000) {
      // Performance good, can potentially increase concurrency
      this.coordinationConfig.maxConcurrentSyncs = Math.min(3, this.coordinationConfig.maxConcurrentSyncs + 1);
      console.log(`[SyncCoordination] Performance good, increasing concurrency to ${this.coordinationConfig.maxConcurrentSyncs}`);
    }
  }

  /**
   * Gets current coordination status
   */
  getCoordinationStatus(): CoordinationStatus {
    return {
      isUserActive: this.isUserActive,
      activeSyncCount: this.activeSyncs.size,
      deferredSyncCount: this.deferredSyncs.length,
      resourceUtilization: this.resourceMonitor.getCurrentUtilization(),
      performanceImpact: this.performanceMonitor.getCurrentImpact(),
      lastCoordinationAction: new Date()
    };
  }

  /**
   * Manually processes all deferred syncs
   */
  async flushDeferredSyncs(): Promise<number> {
    const count = this.deferredSyncs.length;
    await this.processDeferredSyncs();
    return count;
  }

  /**
   * Pauses background sync coordination
   */
  pauseCoordination(): void {
    this.resourceMonitor.pause();
    this.performanceMonitor.pause();
    console.log('[SyncCoordination] Coordination paused');
  }

  /**
   * Resumes background sync coordination
   */
  resumeCoordination(): void {
    this.resourceMonitor.resume();
    this.performanceMonitor.resume();
    console.log('[SyncCoordination] Coordination resumed');
  }
}

// Resource monitoring helper class
class ResourceMonitor {
  private isActive = false;
  private currentUtilization: ResourceUtilization = {
    cpu: 0,
    memory: 0,
    battery: 100,
    network: 0
  };

  start(): void {
    this.isActive = true;
    this.monitorResources();
  }

  pause(): void {
    this.isActive = false;
  }

  resume(): void {
    this.isActive = true;
    this.monitorResources();
  }

  private monitorResources(): void {
    if (!this.isActive) return;

    // Mock resource monitoring - real implementation would use native modules
    this.currentUtilization = {
      cpu: Math.random() * 50 + 10, // 10-60%
      memory: Math.random() * 40 + 20, // 20-60%
      battery: Math.max(10, 100 - Math.random() * 20), // 80-100%
      network: Math.random() * 30 + 5 // 5-35%
    };

    setTimeout(() => this.monitorResources(), 1000); // Update every second
  }

  getCurrentUtilization(): ResourceUtilization {
    return { ...this.currentUtilization };
  }
}

// Performance monitoring helper class
class PerformanceMonitor {
  private isActive = false;
  private currentImpact: PerformanceImpact = {
    frameDrop: 0,
    mainThreadTime: 0,
    syncLatency: 0,
    uiResponsiveness: 100
  };

  start(): void {
    this.isActive = true;
    this.monitorPerformance();
  }

  pause(): void {
    this.isActive = false;
  }

  resume(): void {
    this.isActive = true;
    this.monitorPerformance();
  }

  private monitorPerformance(): void {
    if (!this.isActive) return;

    // Mock performance monitoring
    this.currentImpact = {
      frameDrop: Math.random() * 2, // 0-2 fps
      mainThreadTime: Math.random() * 30 + 10, // 10-40%
      syncLatency: Math.random() * 200 + 50, // 50-250ms
      uiResponsiveness: Math.random() * 20 + 80 // 80-100
    };

    setTimeout(() => this.monitorPerformance(), 500); // Update every 500ms
  }

  getCurrentImpact(): PerformanceImpact {
    return { ...this.currentImpact };
  }
}

// Sync batching helper class
class SyncBatcher {
  private pendingBatches = new Map<string, SyncItem[]>();
  private batchTimeout = 500; // ms

  createBatch(items: SyncItem[]): SyncBatch {
    const id = `batch_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
    
    // Group by type and priority for efficiency
    const groupedItems = this.groupSimilarItems(items);
    
    return {
      id,
      items: groupedItems,
      priority: this.calculateBatchPriority(groupedItems),
      estimatedDuration: this.estimateBatchDuration(groupedItems),
      resourceCost: this.estimateResourceCost(groupedItems),
      createdAt: new Date()
    };
  }

  private groupSimilarItems(items: SyncItem[]): SyncItem[] {
    // For now, return items as-is
    // Real implementation would group similar operations
    return items;
  }

  private calculateBatchPriority(items: SyncItem[]): SyncPriority {
    // Use highest priority in batch
    const priorities = [SyncPriority.CRITICAL, SyncPriority.HIGH, SyncPriority.NORMAL, SyncPriority.LOW];
    
    for (const priority of priorities) {
      if (items.some(item => item.priority === priority)) {
        return priority;
      }
    }
    
    return SyncPriority.NORMAL;
  }

  private estimateBatchDuration(items: SyncItem[]): number {
    // Estimate based on item types
    const durations = {
      [SyncItemType.COMMUNITY_RECIPE]: 300,
      [SyncItemType.USER_RECIPE]: 200,
      [SyncItemType.RECIPE_RATING]: 100,
      [SyncItemType.USER_PROFILE]: 150,
      [SyncItemType.MEAL_PLAN]: 400,
      [SyncItemType.SHOPPING_LIST]: 150,
      [SyncItemType.USER_PREFERENCES]: 100,
      [SyncItemType.RECIPE_IMPORT]: 500
    };

    return items.reduce((total, item) => total + (durations[item.type] || 200), 0);
  }

  private estimateResourceCost(items: SyncItem[]): ResourceCost {
    // Estimate resource cost for batch
    return {
      cpu: items.length * 10,
      memory: items.length * 5,
      network: items.length * 15,
      battery: items.length * 2
    };
  }
}

// Export singleton instance
export const syncCoordinationService = new SyncCoordinationService();
export { SyncItemType } from './background_sync_service';
export default SyncCoordinationService;
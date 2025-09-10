import type { ShoppingList } from '../types/shopping';
import type { MealPlanChange } from './ChangeHistoryTracker';
import { shoppingIntegrationService } from './shopping_integration_service';

interface RealtimeUpdate {
  type: 'shopping_list_update' | 'meal_plan_change' | 'connection_status';
  data: any;
  timestamp: number;
  source: 'websocket' | 'polling';
}

interface RealtimeSubscription {
  id: string;
  mealPlanId: string;
  callback: (update: RealtimeUpdate) => void;
  active: boolean;
}

interface ConnectionConfig {
  websocketUrl?: string;
  pollingInterval?: number;
  reconnectDelay?: number;
  maxReconnectAttempts?: number;
  enablePollingFallback?: boolean;
}

export class ShoppingListRealtimeService {
  private websocket: WebSocket | null = null;
  private pollingInterval: number | null = null;
  private subscriptions = new Map<string, RealtimeSubscription>();
  private connectionConfig: Required<ConnectionConfig>;
  private reconnectAttempts = 0;
  private connectionStatus: 'disconnected' | 'connecting' | 'connected' | 'polling' = 'disconnected';
  private lastPollingTimestamp = new Map<string, number>();

  constructor(config: ConnectionConfig = {}) {
    this.connectionConfig = {
      websocketUrl: config.websocketUrl || this.getWebSocketUrl(),
      pollingInterval: config.pollingInterval || 5000, // 5 seconds
      reconnectDelay: config.reconnectDelay || 2000,
      maxReconnectAttempts: config.maxReconnectAttempts || 5,
      enablePollingFallback: config.enablePollingFallback !== false,
    };

    // Auto-start connection when subscriptions are added
    this.attemptConnection();
  }

  // Subscribe to real-time updates for a meal plan
  subscribe(
    mealPlanId: string,
    callback: (update: RealtimeUpdate) => void
  ): string {
    const subscriptionId = `sub-${mealPlanId}-${Date.now()}`;
    
    const subscription: RealtimeSubscription = {
      id: subscriptionId,
      mealPlanId,
      callback,
      active: true,
    };

    this.subscriptions.set(subscriptionId, subscription);
    console.log(`Subscribed to real-time updates for meal plan: ${mealPlanId}`);

    // Start connection if not already active
    if (this.connectionStatus === 'disconnected') {
      this.attemptConnection();
    } else if (this.websocket?.readyState === WebSocket.OPEN) {
      // Send subscription message if WebSocket is connected
      this.sendSubscription(mealPlanId, 'subscribe');
    }

    return subscriptionId;
  }

  // Unsubscribe from updates
  unsubscribe(subscriptionId: string): void {
    const subscription = this.subscriptions.get(subscriptionId);
    if (subscription) {
      subscription.active = false;
      this.subscriptions.delete(subscriptionId);
      
      console.log(`Unsubscribed from meal plan: ${subscription.mealPlanId}`);

      // If this was the last subscription for this meal plan, unsubscribe from server
      const remainingSubsForMealPlan = Array.from(this.subscriptions.values())
        .filter(sub => sub.mealPlanId === subscription.mealPlanId && sub.active);
      
      if (remainingSubsForMealPlan.length === 0 && this.websocket?.readyState === WebSocket.OPEN) {
        this.sendSubscription(subscription.mealPlanId, 'unsubscribe');
      }
    }

    // If no active subscriptions remain, disconnect
    if (this.subscriptions.size === 0) {
      this.disconnect();
    }
  }

  // Get current connection status
  getConnectionStatus(): 'disconnected' | 'connecting' | 'connected' | 'polling' {
    return this.connectionStatus;
  }

  // Manually trigger connection attempt
  connect(): void {
    if (this.connectionStatus === 'disconnected') {
      this.attemptConnection();
    }
  }

  // Disconnect from real-time updates
  disconnect(): void {
    this.connectionStatus = 'disconnected';
    
    if (this.websocket) {
      this.websocket.close();
      this.websocket = null;
    }

    if (this.pollingInterval) {
      clearInterval(this.pollingInterval);
      this.pollingInterval = null;
    }

    this.reconnectAttempts = 0;
    console.log('Disconnected from real-time shopping list updates');
  }

  // Private methods

  private attemptConnection(): void {
    if (this.connectionStatus === 'connecting' || this.connectionStatus === 'connected') {
      return;
    }

    this.connectionStatus = 'connecting';
    
    try {
      this.connectWebSocket();
    } catch (error) {
      console.error('WebSocket connection failed:', error);
      this.fallbackToPolling();
    }
  }

  private connectWebSocket(): void {
    const wsUrl = this.connectionConfig.websocketUrl;
    console.log(`Attempting WebSocket connection to: ${wsUrl}`);
    
    this.websocket = new WebSocket(wsUrl);
    
    this.websocket.onopen = () => {
      console.log('WebSocket connected');
      this.connectionStatus = 'connected';
      this.reconnectAttempts = 0;
      
      // Subscribe to all active meal plans
      this.resubscribeAll();
      
      // Notify subscribers of connection
      this.notifyConnectionStatus('connected', 'websocket');
    };

    this.websocket.onmessage = (event) => {
      try {
        const message = JSON.parse(event.data);
        this.handleWebSocketMessage(message);
      } catch (error) {
        console.error('Failed to parse WebSocket message:', error);
      }
    };

    this.websocket.onerror = (error) => {
      console.error('WebSocket error:', error);
    };

    this.websocket.onclose = () => {
      console.log('WebSocket connection closed');
      this.websocket = null;
      
      if (this.connectionStatus === 'connected') {
        this.handleConnectionLoss();
      }
    };
  }

  private handleWebSocketMessage(message: any): void {
    const update: RealtimeUpdate = {
      type: message.type,
      data: message.data,
      timestamp: message.timestamp || Date.now(),
      source: 'websocket',
    };

    // Route message to appropriate subscribers
    if (message.mealPlanId) {
      this.notifySubscribers(message.mealPlanId, update);
    } else {
      // Broadcast to all subscribers if no specific meal plan
      this.broadcastToAll(update);
    }
  }

  private handleConnectionLoss(): void {
    this.connectionStatus = 'disconnected';
    
    if (this.reconnectAttempts < this.connectionConfig.maxReconnectAttempts) {
      this.reconnectAttempts++;
      const delay = this.connectionConfig.reconnectDelay * Math.pow(2, this.reconnectAttempts - 1);
      
      console.log(`Attempting to reconnect in ${delay}ms (attempt ${this.reconnectAttempts})`);
      
      setTimeout(() => {
        this.attemptConnection();
      }, delay);
    } else if (this.connectionConfig.enablePollingFallback) {
      console.log('Max reconnection attempts reached, falling back to polling');
      this.fallbackToPolling();
    } else {
      console.log('Connection lost and polling disabled');
      this.notifyConnectionStatus('disconnected', 'websocket');
    }
  }

  private fallbackToPolling(): void {
    if (this.connectionStatus === 'polling' || this.pollingInterval) {
      return;
    }

    console.log('Starting polling fallback');
    this.connectionStatus = 'polling';
    
    this.pollingInterval = window.setInterval(() => {
      this.performPollingCheck();
    }, this.connectionConfig.pollingInterval);

    this.notifyConnectionStatus('polling', 'polling');
  }

  private async performPollingCheck(): Promise<void> {
    const activeMealPlanIds = new Set(
      Array.from(this.subscriptions.values())
        .filter(sub => sub.active)
        .map(sub => sub.mealPlanId)
    );

    for (const mealPlanId of activeMealPlanIds) {
      try {
        await this.checkForUpdates(mealPlanId);
      } catch (error) {
        console.error(`Polling check failed for meal plan ${mealPlanId}:`, error);
      }
    }
  }

  private async checkForUpdates(mealPlanId: string): Promise<void> {
    const lastCheck = this.lastPollingTimestamp.get(mealPlanId) || 0;
    const now = Date.now();

    try {
      // This would call an API endpoint to check for updates since lastCheck
      // For now, we'll simulate by checking if there are shopping list changes
      const updates = await this.fetchUpdatesForMealPlan(mealPlanId, lastCheck);
      
      if (updates.length > 0) {
        updates.forEach(update => {
          const realtimeUpdate: RealtimeUpdate = {
            type: 'shopping_list_update',
            data: update,
            timestamp: now,
            source: 'polling',
          };
          
          this.notifySubscribers(mealPlanId, realtimeUpdate);
        });
      }

      this.lastPollingTimestamp.set(mealPlanId, now);
    } catch (error) {
      console.error(`Failed to check updates for meal plan ${mealPlanId}:`, error);
    }
  }

  private async fetchUpdatesForMealPlan(mealPlanId: string, since: number): Promise<any[]> {
    // Mock implementation - in reality would call API endpoint like:
    // GET /meal-plans/${mealPlanId}/updates?since=${since}
    // This endpoint would return any changes to the meal plan or associated shopping lists
    
    // For now, return empty array (no updates)
    return [];
  }

  private sendSubscription(mealPlanId: string, action: 'subscribe' | 'unsubscribe'): void {
    if (this.websocket?.readyState === WebSocket.OPEN) {
      const message = {
        type: action,
        mealPlanId,
        timestamp: Date.now(),
      };
      
      this.websocket.send(JSON.stringify(message));
    }
  }

  private resubscribeAll(): void {
    const uniqueMealPlanIds = new Set(
      Array.from(this.subscriptions.values())
        .filter(sub => sub.active)
        .map(sub => sub.mealPlanId)
    );

    uniqueMealPlanIds.forEach(mealPlanId => {
      this.sendSubscription(mealPlanId, 'subscribe');
    });
  }

  private notifySubscribers(mealPlanId: string, update: RealtimeUpdate): void {
    const relevantSubscriptions = Array.from(this.subscriptions.values())
      .filter(sub => sub.mealPlanId === mealPlanId && sub.active);

    relevantSubscriptions.forEach(subscription => {
      try {
        subscription.callback(update);
      } catch (error) {
        console.error('Error in subscription callback:', error);
      }
    });
  }

  private broadcastToAll(update: RealtimeUpdate): void {
    Array.from(this.subscriptions.values())
      .filter(sub => sub.active)
      .forEach(subscription => {
        try {
          subscription.callback(update);
        } catch (error) {
          console.error('Error in broadcast callback:', error);
        }
      });
  }

  private notifyConnectionStatus(status: string, source: 'websocket' | 'polling'): void {
    const update: RealtimeUpdate = {
      type: 'connection_status',
      data: { status, source },
      timestamp: Date.now(),
      source,
    };

    this.broadcastToAll(update);
  }

  private getWebSocketUrl(): string {
    // Construct WebSocket URL based on current environment
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const host = window.location.host;
    const path = '/ws/shopping-updates';
    
    return `${protocol}//${host}${path}`;
  }
}

// Create singleton instance
export const shoppingListRealtimeService = new ShoppingListRealtimeService();
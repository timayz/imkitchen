import { ShoppingListRealtimeService } from '../../src/services/ShoppingListRealtimeService';

// Mock WebSocket
class MockWebSocket {
  static CONNECTING = 0;
  static OPEN = 1;
  static CLOSING = 2;
  static CLOSED = 3;

  readyState = MockWebSocket.CONNECTING;
  url: string;
  onopen: ((event: Event) => void) | null = null;
  onclose: ((event: CloseEvent) => void) | null = null;
  onmessage: ((event: MessageEvent) => void) | null = null;
  onerror: ((event: Event) => void) | null = null;
  
  private messageQueue: any[] = [];

  constructor(url: string) {
    this.url = url;
    
    // Simulate async connection
    setTimeout(() => {
      this.readyState = MockWebSocket.OPEN;
      if (this.onopen) {
        this.onopen(new Event('open'));
      }
    }, 10);
  }

  send(data: string) {
    if (this.readyState === MockWebSocket.OPEN) {
      this.messageQueue.push(JSON.parse(data));
    } else {
      throw new Error('WebSocket is not open');
    }
  }

  close() {
    this.readyState = MockWebSocket.CLOSED;
    if (this.onclose) {
      this.onclose(new CloseEvent('close'));
    }
  }

  // Test helper methods
  simulateMessage(message: any) {
    if (this.onmessage) {
      this.onmessage(new MessageEvent('message', {
        data: JSON.stringify(message),
      }));
    }
  }

  simulateError() {
    if (this.onerror) {
      this.onerror(new Event('error'));
    }
  }

  simulateClose() {
    this.readyState = MockWebSocket.CLOSED;
    if (this.onclose) {
      this.onclose(new CloseEvent('close'));
    }
  }

  getSentMessages() {
    return this.messageQueue;
  }
}

// Mock window and global objects
(global as any).WebSocket = MockWebSocket;
(global as any).window = {
  location: {
    protocol: 'https:',
    host: 'localhost:3000',
  },
  setInterval: global.setInterval,
  clearInterval: global.clearInterval,
};

describe('ShoppingListRealtimeService', () => {
  let service: ShoppingListRealtimeService;
  let mockWebSocket: MockWebSocket;
  const mealPlanId = 'test-meal-plan-123';

  beforeEach(() => {
    jest.clearAllMocks();
    jest.useFakeTimers();
    
    service = new ShoppingListRealtimeService({
      pollingInterval: 1000,
      reconnectDelay: 100,
      maxReconnectAttempts: 3,
    });

    // Get reference to the created WebSocket
    mockWebSocket = (WebSocket as any).mock?.instances?.[0] || new MockWebSocket('test');
  });

  afterEach(() => {
    service.disconnect();
    jest.useRealTimers();
  });

  describe('Subscription Management', () => {
    test('should create subscription and start connection', () => {
      const callback = jest.fn();
      const subscriptionId = service.subscribe(mealPlanId, callback);

      expect(subscriptionId).toBeDefined();
      expect(subscriptionId).toMatch(/^sub-test-meal-plan-123-\d+$/);
      expect(service.getConnectionStatus()).toBe('connecting');
    });

    test('should handle multiple subscriptions for same meal plan', () => {
      const callback1 = jest.fn();
      const callback2 = jest.fn();
      
      const sub1 = service.subscribe(mealPlanId, callback1);
      const sub2 = service.subscribe(mealPlanId, callback2);

      expect(sub1).not.toBe(sub2);
      expect(service.getConnectionStatus()).toBe('connecting');
    });

    test('should unsubscribe correctly', () => {
      const callback = jest.fn();
      const subscriptionId = service.subscribe(mealPlanId, callback);

      service.unsubscribe(subscriptionId);

      // Should disconnect when no subscriptions remain
      expect(service.getConnectionStatus()).toBe('disconnected');
    });

    test('should not disconnect when other subscriptions remain', () => {
      const callback1 = jest.fn();
      const callback2 = jest.fn();
      
      const sub1 = service.subscribe(mealPlanId, callback1);
      const sub2 = service.subscribe('other-meal-plan', callback2);

      service.unsubscribe(sub1);

      // Should still be connected due to remaining subscription
      expect(service.getConnectionStatus()).toBe('connecting');
    });
  });

  describe('WebSocket Connection', () => {
    test('should establish WebSocket connection successfully', async () => {
      const callback = jest.fn();
      service.subscribe(mealPlanId, callback);

      // Wait for connection
      jest.advanceTimersByTime(50);

      expect(service.getConnectionStatus()).toBe('connected');
      expect(callback).toHaveBeenCalledWith(
        expect.objectContaining({
          type: 'connection_status',
          data: { status: 'connected', source: 'websocket' },
          source: 'websocket',
        })
      );
    });

    test('should send subscription message after connection', async () => {
      const callback = jest.fn();
      service.subscribe(mealPlanId, callback);

      // Wait for connection
      jest.advanceTimersByTime(50);

      const sentMessages = mockWebSocket.getSentMessages();
      expect(sentMessages).toContainEqual({
        type: 'subscribe',
        mealPlanId,
        timestamp: expect.any(Number),
      });
    });

    test('should handle WebSocket messages correctly', () => {
      const callback = jest.fn();
      service.subscribe(mealPlanId, callback);

      jest.advanceTimersByTime(50); // Wait for connection

      const testMessage = {
        type: 'shopping_list_update',
        mealPlanId,
        data: { listId: 'list-123', changes: ['item added'] },
        timestamp: Date.now(),
      };

      mockWebSocket.simulateMessage(testMessage);

      expect(callback).toHaveBeenCalledWith(
        expect.objectContaining({
          type: 'shopping_list_update',
          data: testMessage.data,
          source: 'websocket',
        })
      );
    });

    test('should broadcast messages without meal plan ID to all subscribers', () => {
      const callback1 = jest.fn();
      const callback2 = jest.fn();
      
      service.subscribe('meal-plan-1', callback1);
      service.subscribe('meal-plan-2', callback2);

      jest.advanceTimersByTime(50); // Wait for connection

      const broadcastMessage = {
        type: 'connection_status',
        data: { status: 'maintenance' },
        timestamp: Date.now(),
      };

      mockWebSocket.simulateMessage(broadcastMessage);

      expect(callback1).toHaveBeenCalledWith(
        expect.objectContaining({
          type: 'connection_status',
          data: broadcastMessage.data,
          source: 'websocket',
        })
      );
      expect(callback2).toHaveBeenCalledWith(
        expect.objectContaining({
          type: 'connection_status',
          data: broadcastMessage.data,
          source: 'websocket',
        })
      );
    });
  });

  describe('Connection Recovery', () => {
    test('should attempt reconnection after connection loss', () => {
      const callback = jest.fn();
      service.subscribe(mealPlanId, callback);

      jest.advanceTimersByTime(50); // Initial connection
      expect(service.getConnectionStatus()).toBe('connected');

      // Simulate connection loss
      mockWebSocket.simulateClose();
      expect(service.getConnectionStatus()).toBe('disconnected');

      // Should attempt reconnection
      jest.advanceTimersByTime(200); // Wait for reconnect delay
      expect(service.getConnectionStatus()).toBe('connecting');
    });

    test('should use exponential backoff for reconnection attempts', () => {
      const callback = jest.fn();
      service.subscribe(mealPlanId, callback);

      jest.advanceTimersByTime(50); // Initial connection

      // First reconnection attempt
      mockWebSocket.simulateClose();
      jest.advanceTimersByTime(100); // First delay: 100ms
      
      // Second reconnection attempt  
      mockWebSocket.simulateClose();
      jest.advanceTimersByTime(200); // Second delay: 200ms (2^1 * base)
      
      // Third reconnection attempt
      mockWebSocket.simulateClose();
      jest.advanceTimersByTime(400); // Third delay: 400ms (2^2 * base)

      // Should have attempted multiple reconnections
      expect(service.getConnectionStatus()).toBe('connecting');
    });

    test('should fall back to polling after max reconnection attempts', () => {
      const callback = jest.fn();
      service.subscribe(mealPlanId, callback);

      jest.advanceTimersByTime(50); // Initial connection

      // Exhaust reconnection attempts
      for (let i = 0; i < 4; i++) {
        mockWebSocket.simulateClose();
        jest.advanceTimersByTime(1000); // Wait for reconnection attempt
      }

      expect(service.getConnectionStatus()).toBe('polling');
    });

    test('should not fall back to polling if disabled', () => {
      const noPollingService = new ShoppingListRealtimeService({
        enablePollingFallback: false,
        maxReconnectAttempts: 1,
        reconnectDelay: 100,
      });

      const callback = jest.fn();
      noPollingService.subscribe(mealPlanId, callback);

      jest.advanceTimersByTime(50); // Initial connection

      // Exhaust reconnection attempts
      for (let i = 0; i < 2; i++) {
        mockWebSocket.simulateClose();
        jest.advanceTimersByTime(200);
      }

      expect(noPollingService.getConnectionStatus()).toBe('disconnected');
      
      noPollingService.disconnect();
    });
  });

  describe('Polling Fallback', () => {
    test('should start polling when WebSocket connection fails', () => {
      const service = new ShoppingListRealtimeService({
        pollingInterval: 1000,
        enablePollingFallback: true,
      });

      // Force polling mode
      (service as any).fallbackToPolling();
      
      expect(service.getConnectionStatus()).toBe('polling');

      service.disconnect();
    });

    test('should perform polling checks at specified interval', () => {
      const callback = jest.fn();
      const service = new ShoppingListRealtimeService({
        pollingInterval: 1000,
      });

      service.subscribe(mealPlanId, callback);
      (service as any).fallbackToPolling();

      // Mock the polling check method
      const pollingCheckSpy = jest.spyOn(service as any, 'performPollingCheck');
      
      jest.advanceTimersByTime(1000);
      expect(pollingCheckSpy).toHaveBeenCalledTimes(1);
      
      jest.advanceTimersByTime(1000);
      expect(pollingCheckSpy).toHaveBeenCalledTimes(2);

      service.disconnect();
    });

    test('should handle polling errors gracefully', async () => {
      const callback = jest.fn();
      const service = new ShoppingListRealtimeService({
        pollingInterval: 1000,
      });

      service.subscribe(mealPlanId, callback);
      (service as any).fallbackToPolling();

      // Mock the checkForUpdates method to throw an error
      jest.spyOn(service as any, 'checkForUpdates').mockRejectedValue(
        new Error('Network error')
      );

      // Should not throw and should continue polling
      jest.advanceTimersByTime(1000);
      expect(service.getConnectionStatus()).toBe('polling');

      service.disconnect();
    });
  });

  describe('Error Handling', () => {
    test('should handle WebSocket connection errors', () => {
      const callback = jest.fn();
      service.subscribe(mealPlanId, callback);

      mockWebSocket.simulateError();

      // Should still be in connecting state, waiting for potential recovery
      expect(service.getConnectionStatus()).toBe('connecting');
    });

    test('should handle malformed WebSocket messages', () => {
      const callback = jest.fn();
      service.subscribe(mealPlanId, callback);

      jest.advanceTimersByTime(50); // Wait for connection

      // Simulate malformed message
      if (mockWebSocket.onmessage) {
        mockWebSocket.onmessage(new MessageEvent('message', {
          data: 'invalid json',
        }));
      }

      // Should not crash and should remain connected
      expect(service.getConnectionStatus()).toBe('connected');
    });

    test('should handle callback errors gracefully', () => {
      const errorCallback = jest.fn(() => {
        throw new Error('Callback error');
      });
      const goodCallback = jest.fn();

      service.subscribe(mealPlanId, errorCallback);
      service.subscribe(mealPlanId, goodCallback);

      jest.advanceTimersByTime(50); // Wait for connection

      const testMessage = {
        type: 'shopping_list_update',
        mealPlanId,
        data: { test: 'data' },
      };

      // Should not crash despite error callback
      mockWebSocket.simulateMessage(testMessage);

      expect(errorCallback).toHaveBeenCalled();
      expect(goodCallback).toHaveBeenCalled();
    });
  });

  describe('Connection States', () => {
    test('should report correct connection status', () => {
      expect(service.getConnectionStatus()).toBe('disconnected');

      const callback = jest.fn();
      service.subscribe(mealPlanId, callback);
      expect(service.getConnectionStatus()).toBe('connecting');

      jest.advanceTimersByTime(50);
      expect(service.getConnectionStatus()).toBe('connected');

      service.disconnect();
      expect(service.getConnectionStatus()).toBe('disconnected');
    });

    test('should handle manual connection requests', () => {
      expect(service.getConnectionStatus()).toBe('disconnected');

      service.connect();
      expect(service.getConnectionStatus()).toBe('connecting');

      service.disconnect();
      expect(service.getConnectionStatus()).toBe('disconnected');
    });
  });

  describe('Resource Cleanup', () => {
    test('should clean up resources on disconnect', () => {
      const callback = jest.fn();
      service.subscribe(mealPlanId, callback);

      jest.advanceTimersByTime(50); // Wait for connection

      service.disconnect();

      expect(service.getConnectionStatus()).toBe('disconnected');
      expect(mockWebSocket.readyState).toBe(MockWebSocket.CLOSED);
    });

    test('should stop polling on disconnect', () => {
      const service = new ShoppingListRealtimeService({
        pollingInterval: 1000,
      });

      const callback = jest.fn();
      service.subscribe(mealPlanId, callback);
      (service as any).fallbackToPolling();

      expect(service.getConnectionStatus()).toBe('polling');

      service.disconnect();
      expect(service.getConnectionStatus()).toBe('disconnected');

      // Polling should have stopped
      const pollingCheckSpy = jest.spyOn(service as any, 'performPollingCheck');
      jest.advanceTimersByTime(2000);
      expect(pollingCheckSpy).not.toHaveBeenCalled();

      service.disconnect();
    });

    test('should handle multiple disconnect calls safely', () => {
      const callback = jest.fn();
      service.subscribe(mealPlanId, callback);

      service.disconnect();
      service.disconnect(); // Should not throw
      service.disconnect(); // Should not throw

      expect(service.getConnectionStatus()).toBe('disconnected');
    });
  });

  describe('Message Routing', () => {
    test('should route messages to correct subscribers based on meal plan ID', () => {
      const callback1 = jest.fn();
      const callback2 = jest.fn();
      const callback3 = jest.fn();

      service.subscribe('meal-plan-1', callback1);
      service.subscribe('meal-plan-2', callback2);
      service.subscribe('meal-plan-1', callback3); // Same meal plan as first

      jest.advanceTimersByTime(50); // Wait for connection

      const message = {
        type: 'shopping_list_update',
        mealPlanId: 'meal-plan-1',
        data: { test: 'data' },
      };

      mockWebSocket.simulateMessage(message);

      // Only subscribers to meal-plan-1 should receive the message
      expect(callback1).toHaveBeenCalled();
      expect(callback3).toHaveBeenCalled();
      expect(callback2).not.toHaveBeenCalled();
    });

    test('should handle subscription messages to server', () => {
      const callback = jest.fn();
      service.subscribe(mealPlanId, callback);

      jest.advanceTimersByTime(50); // Wait for connection

      const sentMessages = mockWebSocket.getSentMessages();
      expect(sentMessages).toContainEqual(
        expect.objectContaining({
          type: 'subscribe',
          mealPlanId,
        })
      );

      service.unsubscribe(service.subscribe(mealPlanId, callback));

      // Should send unsubscribe when no more subscriptions for meal plan
      const laterMessages = mockWebSocket.getSentMessages();
      expect(laterMessages).toContainEqual(
        expect.objectContaining({
          type: 'unsubscribe',
          mealPlanId,
        })
      );
    });
  });
});
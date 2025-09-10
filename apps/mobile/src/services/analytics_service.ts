import type { 
  RotationAnalytics, 
  RotationResetOptions, 
  AnalyticsExportOptions,
  RotationDebugLog 
} from '../types/analytics';

const API_BASE_URL = process.env.REACT_APP_API_URL || 'http://localhost:8080/api/v1';

interface ApiResponse<T> {
  data: T;
  message?: string;
  metadata?: {
    retrievedAt?: string;
    updatedAt?: string;
    resetAt?: string;
  };
}

class AnalyticsService {
  private async getAuthToken(): Promise<string> {
    const AsyncStorage = require('@react-native-async-storage/async-storage').default;
    const token = await AsyncStorage.getItem('authToken') || '';
    return token;
  }

  private async apiRequest<T>(
    endpoint: string,
    options: RequestInit = {}
  ): Promise<ApiResponse<T>> {
    const token = await this.getAuthToken();
    
    const response = await fetch(`${API_BASE_URL}${endpoint}`, {
      ...options,
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${token}`,
        ...options.headers,
      },
    });

    if (!response.ok) {
      throw new Error(`API request failed: ${response.status} ${response.statusText}`);
    }

    return response.json();
  }

  async getRotationAnalytics(weeks: number = 12, includeDetailed: boolean = true): Promise<RotationAnalytics> {
    try {
      const response = await this.apiRequest<RotationAnalytics>(
        `/users/rotation/stats?weeks=${weeks}&includeDetailed=${includeDetailed}`
      );
      return response.data;
    } catch (error) {
      console.error('Failed to fetch rotation analytics:', error);
      throw error;
    }
  }

  async resetRotationCycle(options: RotationResetOptions): Promise<void> {
    try {
      await this.apiRequest<void>('/users/rotation/reset', {
        method: 'POST',
        body: JSON.stringify({
          confirmReset: options.confirmReset,
          preservePatterns: options.preservePatterns ?? true,
          preserveFavorites: options.preserveFavorites ?? true,
        }),
      });
    } catch (error) {
      console.error('Failed to reset rotation cycle:', error);
      throw error;
    }
  }

  async exportRotationData(options: AnalyticsExportOptions): Promise<Blob> {
    try {
      const params = new URLSearchParams();
      params.append('format', options.format);
      
      if (options.dateRange) {
        params.append('dateRange', `${options.dateRange.startDate},${options.dateRange.endDate}`);
      }
      
      if (options.includeDebugLogs) {
        params.append('includeDebugLogs', 'true');
      }

      const token = await this.getAuthToken();
      const response = await fetch(`${API_BASE_URL}/users/rotation/export?${params.toString()}`, {
        headers: {
          'Authorization': `Bearer ${token}`,
        },
      });

      if (!response.ok) {
        throw new Error(`Export request failed: ${response.status} ${response.statusText}`);
      }
      
      return response.blob();
    } catch (error) {
      console.error('Failed to export rotation data:', error);
      throw error;
    }
  }

  async getDebugLogs(limit: number = 100): Promise<RotationDebugLog[]> {
    try {
      const response = await this.apiRequest<RotationDebugLog[]>(
        `/users/rotation/debug-logs?limit=${limit}`
      );
      return response.data;
    } catch (error) {
      console.error('Failed to fetch debug logs:', error);
      throw error;
    }
  }

  // Calculate variety score locally for real-time feedback
  calculateLocalVarietyScore(recipes: Array<{id: string, complexity: number}>): number {
    if (recipes.length === 0) return 0;
    
    const complexityDistribution: Record<number, number> = {};
    
    recipes.forEach(recipe => {
      complexityDistribution[recipe.complexity] = 
        (complexityDistribution[recipe.complexity] || 0) + 1;
    });
    
    // Shannon entropy calculation for variety
    const total = recipes.length;
    let entropy = 0;
    
    Object.values(complexityDistribution).forEach(count => {
      const probability = count / total;
      if (probability > 0) {
        entropy -= probability * Math.log2(probability);
      }
    });
    
    // Normalize to 0-100 scale (max entropy for 5 complexity levels is log2(5) ≈ 2.32)
    return Math.round((entropy / 2.32) * 100);
  }
}

export const analyticsService = new AnalyticsService();
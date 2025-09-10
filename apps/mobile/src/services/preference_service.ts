import { CoreUserPreferences, WeeklyPattern } from '../store/preference_store';

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

class PreferenceService {
  private async getAuthToken(): Promise<string> {
    // Get token from auth store or AsyncStorage
    // This should match your existing auth pattern
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
      const errorData = await response.json().catch(() => ({}));
      throw new Error(errorData.message || `HTTP error! status: ${response.status}`);
    }

    return response.json();
  }

  // Get user preferences
  async getUserPreferences(): Promise<ApiResponse<CoreUserPreferences>> {
    return this.apiRequest<CoreUserPreferences>('/users/preferences');
  }

  // Update user preferences
  async updateUserPreferences(
    preferences: CoreUserPreferences
  ): Promise<ApiResponse<CoreUserPreferences>> {
    return this.apiRequest<CoreUserPreferences>('/users/preferences', {
      method: 'PUT',
      body: JSON.stringify(preferences),
    });
  }

  // Reset user preferences to defaults
  async resetUserPreferences(): Promise<ApiResponse<CoreUserPreferences>> {
    return this.apiRequest<CoreUserPreferences>('/users/preferences/reset', {
      method: 'POST',
    });
  }

  // Get weekly cooking patterns
  async getWeeklyPatterns(): Promise<WeeklyPattern[]> {
    const response = await this.apiRequest<WeeklyPattern[]>('/users/preferences/patterns');
    return response.data;
  }

  // Update weekly cooking patterns
  async updateWeeklyPatterns(patterns: WeeklyPattern[]): Promise<WeeklyPattern[]> {
    const response = await this.apiRequest<WeeklyPattern[]>('/users/preferences/patterns', {
      method: 'PUT',
      body: JSON.stringify({ weeklyPatterns: patterns }),
    });
    return response.data;
  }

  // Validate preferences locally before sending to API
  validatePreferences(preferences: CoreUserPreferences): { isValid: boolean; errors: string[] } {
    const errors: string[] = [];

    // Validate maxCookTime (15-180 minutes)
    if (preferences.maxCookTime < 15 || preferences.maxCookTime > 180) {
      errors.push('Max cook time must be between 15 and 180 minutes');
    }

    // Validate preferredComplexity
    const validComplexities = ['simple', 'moderate', 'complex'];
    if (!validComplexities.includes(preferences.preferredComplexity)) {
      errors.push('Preferred complexity must be simple, moderate, or complex');
    }

    return {
      isValid: errors.length === 0,
      errors,
    };
  }
}

export default new PreferenceService();
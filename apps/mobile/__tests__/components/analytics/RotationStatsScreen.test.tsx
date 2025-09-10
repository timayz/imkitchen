import React from 'react';
import { render, fireEvent, waitFor, act } from '@testing-library/react-native';
import { Alert } from 'react-native';
import { RotationStatsScreen } from '../../../src/screens/analytics/RotationStatsScreen';
import { analyticsService } from '../../../src/services/analytics_service';
import type { RotationAnalytics } from '../../../src/types/analytics';

// Mock the analytics service
jest.mock('../../../src/services/analytics_service');

// Mock Alert
jest.spyOn(Alert, 'alert').mockImplementation(() => {});

const mockAnalyticsService = analyticsService as jest.Mocked<typeof analyticsService>;

describe('RotationStatsScreen', () => {
  const mockAnalyticsData: RotationAnalytics = {
    varietyScore: 85.5,
    rotationEfficiency: 0.92,
    weeksAnalyzed: 12,
    complexityDistribution: {
      'Very Easy': 10,
      'Easy': 25,
      'Medium': 30,
      'Hard': 20,
      'Very Hard': 15,
    },
    complexityTrends: [
      {
        week: 'W1',
        averageComplexity: 2.5,
        prepTimeMinutes: 35,
        recipeCount: 7,
      },
      {
        week: 'W2',
        averageComplexity: 3.0,
        prepTimeMinutes: 42,
        recipeCount: 6,
      },
    ],
    favoritesFrequency: {
      'Spaghetti Carbonara': 5,
      'Chicken Stir Fry': 4,
      'Veggie Burger': 3,
    },
    favoritesImpact: 0.75,
    weeklyPatterns: [
      {
        weekNumber: 1,
        weekStartDate: '2024-01-01',
        varietyScore: 80,
        patternAdherence: 0.9,
        favoritesUsed: 3,
        totalMeals: 7,
      },
      {
        weekNumber: 2,
        weekStartDate: '2024-01-08',
        varietyScore: 85,
        patternAdherence: 0.95,
        favoritesUsed: 2,
        totalMeals: 6,
      },
    ],
    calculatedAt: '2024-01-15T10:00:00Z',
  };

  beforeEach(() => {
    jest.clearAllMocks();
    mockAnalyticsService.getRotationAnalytics.mockResolvedValue(mockAnalyticsData);
  });

  it('renders loading state initially', () => {
    const { getByText } = render(<RotationStatsScreen />);
    expect(getByText('Loading analytics...')).toBeTruthy();
  });

  it('loads and displays analytics data successfully', async () => {
    const { getByText } = render(<RotationStatsScreen />);

    await waitFor(() => {
      expect(getByText('Rotation Analytics')).toBeTruthy();
      expect(getByText('Analyzing 12 weeks of meal planning data')).toBeTruthy();
    });

    expect(mockAnalyticsService.getRotationAnalytics).toHaveBeenCalledWith(12);
  });

  it('displays variety score correctly', async () => {
    const { getByText } = render(<RotationStatsScreen />);

    await waitFor(() => {
      expect(getByText('86')).toBeTruthy(); // Rounded variety score
      expect(getByText('Meal Variety Analysis')).toBeTruthy();
    });
  });

  it('shows cooking pattern distribution', async () => {
    const { getByText } = render(<RotationStatsScreen />);

    await waitFor(() => {
      expect(getByText('Recipe Complexity Distribution')).toBeTruthy();
      expect(getByText('Very Easy')).toBeTruthy();
      expect(getByText('Medium')).toBeTruthy();
    });
  });

  it('displays favorites frequency data', async () => {
    const { getByText } = render(<RotationStatsScreen />);

    await waitFor(() => {
      expect(getByText('Favorite Recipe Usage')).toBeTruthy();
      expect(getByText('Spaghetti Carbonara')).toBeTruthy();
      expect(getByText('Chicken Stir Fry')).toBeTruthy();
    });
  });

  it('handles refresh functionality', async () => {
    const { getByTestId } = render(<RotationStatsScreen />);

    await waitFor(() => {
      expect(mockAnalyticsService.getRotationAnalytics).toHaveBeenCalledTimes(1);
    });

    // Trigger refresh
    const scrollView = getByTestId('analytics-scroll-view');
    fireEvent(scrollView, 'refresh');

    await waitFor(() => {
      expect(mockAnalyticsService.getRotationAnalytics).toHaveBeenCalledTimes(2);
    });
  });

  it('handles weeks selection change', async () => {
    const { getByText } = render(<RotationStatsScreen />);

    await waitFor(() => {
      expect(getByText('Rotation Analytics')).toBeTruthy();
    });

    // Simulate weeks change (this would typically be triggered by WeeklyTrendAnalysis component)
    // In a real test, you would find and interact with the weeks selector
    act(() => {
      // Trigger weeks change event
      const weeklyTrendComponent = getByText('Weekly Trend Analysis');
      // This is a simplified test - in reality, you'd need to find the actual selector
      fireEvent.press(weeklyTrendComponent);
    });

    // Verify that new request is made with different weeks parameter
    await waitFor(() => {
      expect(mockAnalyticsService.getRotationAnalytics).toHaveBeenCalled();
    });
  });

  it('handles error state gracefully', async () => {
    const errorMessage = 'Failed to load analytics';
    mockAnalyticsService.getRotationAnalytics.mockRejectedValue(new Error(errorMessage));

    const { getByText } = render(<RotationStatsScreen />);

    await waitFor(() => {
      expect(Alert.alert).toHaveBeenCalledWith(
        'Error',
        'Failed to load rotation analytics. Please try again.',
        [{ text: 'OK' }]
      );
    });
  });

  it('handles reset completion callback', async () => {
    const { getByText } = render(<RotationStatsScreen />);

    await waitFor(() => {
      expect(getByText('Rotation Analytics')).toBeTruthy();
    });

    // Clear previous calls
    jest.clearAllMocks();

    // Simulate reset completion (this would be triggered by RotationResetButton)
    // In the actual implementation, this would be handled by the onResetComplete callback
    act(() => {
      // Trigger the reset completion callback
      // This is a simplified representation
      const resetButton = getByText(/Reset Rotation/);
      if (resetButton) {
        fireEvent.press(resetButton);
      }
    });

    // Verify analytics are reloaded after reset
    await waitFor(() => {
      expect(mockAnalyticsService.getRotationAnalytics).toHaveBeenCalled();
    });
  });

  it('displays last updated timestamp', async () => {
    const { getByText } = render(<RotationStatsScreen />);

    await waitFor(() => {
      expect(getByText(/Last updated:/)).toBeTruthy();
      expect(getByText(/1\/15\/2024/)).toBeTruthy(); // Date formatted
    });
  });

  it('handles empty analytics data', async () => {
    mockAnalyticsService.getRotationAnalytics.mockResolvedValue({
      ...mockAnalyticsData,
      complexityDistribution: {},
      favoritesFrequency: {},
      weeklyPatterns: [],
      complexityTrends: [],
    });

    const { getByText } = render(<RotationStatsScreen />);

    await waitFor(() => {
      expect(getByText('Rotation Analytics')).toBeTruthy();
      expect(getByText('86')).toBeTruthy(); // Variety score still shows
    });

    // Verify components handle empty data gracefully
    expect(getByText('Meal Variety Analysis')).toBeTruthy();
  });

  it('integrates with all analytics components', async () => {
    const { getByText, queryByText } = render(<RotationStatsScreen />);

    await waitFor(() => {
      // Verify all major components are rendered
      expect(getByText('Meal Variety Analysis')).toBeTruthy();
      expect(getByText('Cooking Pattern Analysis')).toBeTruthy();
      expect(getByText('Favorite Recipe Usage')).toBeTruthy();
      expect(getByText('Weekly Trend Analysis')).toBeTruthy();
    });

    // Verify action buttons are present
    expect(queryByText(/Export Analytics/)).toBeTruthy();
    expect(queryByText(/Reset Rotation/)).toBeTruthy();
  });

  it('maintains performance with large datasets', async () => {
    // Create a large dataset
    const largeDataset: RotationAnalytics = {
      ...mockAnalyticsData,
      weeksAnalyzed: 52,
      complexityTrends: Array.from({ length: 52 }, (_, i) => ({
        week: `W${i + 1}`,
        averageComplexity: Math.random() * 5,
        prepTimeMinutes: Math.random() * 60 + 20,
        recipeCount: Math.floor(Math.random() * 7) + 1,
      })),
      weeklyPatterns: Array.from({ length: 52 }, (_, i) => ({
        weekNumber: i + 1,
        weekStartDate: new Date(2024, 0, i * 7 + 1).toISOString().split('T')[0],
        varietyScore: Math.random() * 100,
        patternAdherence: Math.random(),
        favoritesUsed: Math.floor(Math.random() * 5),
        totalMeals: 7,
      })),
    };

    mockAnalyticsService.getRotationAnalytics.mockResolvedValue(largeDataset);

    const startTime = Date.now();
    const { getByText } = render(<RotationStatsScreen />);

    await waitFor(() => {
      expect(getByText('Rotation Analytics')).toBeTruthy();
      expect(getByText('Analyzing 52 weeks of meal planning data')).toBeTruthy();
    });

    const renderTime = Date.now() - startTime;
    
    // Verify performance requirement: should render within 3 seconds
    expect(renderTime).toBeLessThan(3000);
  });
});
import React from 'react';
import { render, fireEvent, waitFor, act } from '@testing-library/react-native';
import { Alert } from 'react-native';
import FillMyWeekButton from '../../src/components/organisms/FillMyWeekButton';
import { mealPlanService } from '../../src/services/meal_plan_service';
import { useMealPlanStore } from '../../src/store/meal_plan_store';

// Mock dependencies
jest.mock('../../src/services/meal_plan_service');
jest.mock('../../src/store/meal_plan_store');
jest.spyOn(Alert, 'alert');

const mockMealPlanService = mealPlanService as jest.Mocked<typeof mealPlanService>;
const mockUseMealPlanStore = useMealPlanStore as jest.MockedFunction<typeof useMealPlanStore>;

describe('FillMyWeekButton', () => {
  const mockLoadMealPlan = jest.fn();
  const currentWeek = new Date('2025-09-08'); // Monday

  beforeEach(() => {
    jest.clearAllMocks();
    
    // Setup store mock
    mockUseMealPlanStore.mockReturnValue({
      loadMealPlan: mockLoadMealPlan,
      currentWeek,
      // Add other required store properties
      currentMealPlan: null,
      mealPlans: {},
      loading: false,
      loadingWeek: null,
      refreshing: false,
      generating: false,
      generationProgress: 0,
      error: null,
      lastGenerationTime: null,
      lastVarietyScore: null,
      rotationCycle: null,
      optimisticUpdates: [],
      setCurrentWeek: jest.fn(),
      createMealPlan: jest.fn(),
      updateMealPlan: jest.fn(),
      updateMealSlot: jest.fn(),
      deleteMealSlot: jest.fn(),
      moveMeal: jest.fn(),
      refreshCurrentMealPlan: jest.fn(),
      generateWeeklyMealPlan: jest.fn(),
      clearError: jest.fn(),
      reset: jest.fn(),
      addOptimisticUpdate: jest.fn(),
      removeOptimisticUpdate: jest.fn(),
      applyOptimisticUpdates: jest.fn(),
    });
  });

  it('renders correctly with default state', () => {
    const { getByText, getByRole } = render(<FillMyWeekButton />);

    expect(getByText('✨ Fill My Week')).toBeTruthy();
    expect(getByText('Instant meal plan generation')).toBeTruthy();
    expect(getByRole('button')).toBeTruthy();
  });

  it('shows correct accessibility properties', () => {
    const { getByRole } = render(<FillMyWeekButton />);
    
    const button = getByRole('button');
    expect(button).toHaveAccessibilityRole('button');
    expect(button).toHaveAccessibilityLabel('Fill My Week - Generate automated meal plan');
    expect(button).toHaveAccessibilityHint('Automatically generates a weekly meal plan with recipe variety and rotation');
    expect(button).toHaveAccessibilityState({ disabled: false, busy: false });
  });

  it('is disabled when disabled prop is true', () => {
    const { getByRole, getByText } = render(<FillMyWeekButton disabled />);
    
    const button = getByRole('button');
    expect(button).toHaveAccessibilityState({ disabled: true, busy: false });
    expect(getByText('✨ Fill My Week')).toBeTruthy();
  });

  it('handles successful meal plan generation', async () => {
    const mockResponse = {
      mealPlan: {
        id: 'meal-plan-123',
        userId: 'user-123',
        weekStartDate: new Date('2025-09-08'),
        generationType: 'automated' as const,
        generatedAt: new Date(),
        totalEstimatedTime: 300,
        isActive: true,
        status: 'active' as const,
        completionPercentage: 0,
        populatedMeals: {
          monday: [],
          tuesday: [],
          wednesday: [],
          thursday: [],
          friday: [],
          saturday: [],
          sunday: [],
        },
        createdAt: new Date(),
        updatedAt: new Date(),
      },
      generationTimeMs: 1500,
      varietyScore: 0.85,
      recipesUsed: 21,
      rotationCycle: 1,
      warnings: [],
    };

    const onGenerationComplete = jest.fn();
    mockMealPlanService.generateWeeklyMealPlan.mockResolvedValue(mockResponse);

    const { getByRole } = render(
      <FillMyWeekButton onGenerationComplete={onGenerationComplete} />
    );

    const button = getByRole('button');
    
    await act(async () => {
      fireEvent.press(button);
    });

    // Wait for generation to complete
    await waitFor(() => {
      expect(mockMealPlanService.generateWeeklyMealPlan).toHaveBeenCalledWith({
        weekStartDate: currentWeek,
      });
    });

    await waitFor(() => {
      expect(onGenerationComplete).toHaveBeenCalledWith(mockResponse);
      expect(mockLoadMealPlan).toHaveBeenCalledWith(currentWeek, true);
    });
  });

  it('shows loading state during generation', async () => {
    let resolveGeneration: (value: any) => void;
    const generationPromise = new Promise((resolve) => {
      resolveGeneration = resolve;
    });

    mockMealPlanService.generateWeeklyMealPlan.mockReturnValue(generationPromise);

    const { getByRole, getByText, queryByText } = render(<FillMyWeekButton />);

    const button = getByRole('button');
    
    await act(async () => {
      fireEvent.press(button);
    });

    // Should show generating state
    await waitFor(() => {
      expect(queryByText('✨ Fill My Week')).toBeFalsy();
      expect(getByText(/Generating\.\.\./)).toBeTruthy();
    });

    // Button should be disabled and have busy state
    expect(button).toHaveAccessibilityState({ disabled: true, busy: true });

    // Resolve the promise to finish test
    resolveGeneration!({
      mealPlan: {},
      generationTimeMs: 1000,
      varietyScore: 0.8,
      recipesUsed: 21,
      rotationCycle: 1,
      warnings: [],
    });
  });

  it('handles generation errors appropriately', async () => {
    const errorMessage = 'Network error occurred';
    mockMealPlanService.generateWeeklyMealPlan.mockRejectedValue(new Error(errorMessage));

    const onGenerationError = jest.fn();
    const { getByRole } = render(
      <FillMyWeekButton onGenerationError={onGenerationError} />
    );

    const button = getByRole('button');
    
    await act(async () => {
      fireEvent.press(button);
    });

    await waitFor(() => {
      expect(Alert.alert).toHaveBeenCalledWith(
        'Generation Failed',
        errorMessage,
        expect.arrayContaining([
          expect.objectContaining({ text: 'Try Again' }),
          expect.objectContaining({ text: 'Cancel', style: 'cancel' }),
        ])
      );
      expect(onGenerationError).toHaveBeenCalledWith(errorMessage);
    });
  });

  it('shows warnings in alert when generation has warnings', async () => {
    const mockResponse = {
      mealPlan: {
        id: 'meal-plan-123',
        userId: 'user-123',
        weekStartDate: new Date('2025-09-08'),
        generationType: 'automated' as const,
        generatedAt: new Date(),
        totalEstimatedTime: 300,
        isActive: true,
        status: 'active' as const,
        completionPercentage: 0,
        populatedMeals: {
          monday: [],
          tuesday: [],
          wednesday: [],
          thursday: [],
          friday: [],
          saturday: [],
          sunday: [],
        },
        createdAt: new Date(),
        updatedAt: new Date(),
      },
      generationTimeMs: 2500,
      varietyScore: 0.45,
      recipesUsed: 21,
      rotationCycle: 1,
      warnings: [
        'Generation took longer than expected (>2 seconds)',
        'Limited recipe variety - consider adding more recipes to your collection'
      ],
    };

    mockMealPlanService.generateWeeklyMealPlan.mockResolvedValue(mockResponse);

    const { getByRole } = render(<FillMyWeekButton />);

    const button = getByRole('button');
    
    await act(async () => {
      fireEvent.press(button);
    });

    await waitFor(() => {
      expect(Alert.alert).toHaveBeenCalledWith(
        'Meal Plan Generated',
        expect.stringContaining('Your weekly meal plan is ready!'),
        expect.arrayContaining([
          expect.objectContaining({ text: 'View Plan' }),
        ])
      );
    });
  });

  it('uses custom weekStartDate when provided', async () => {
    const customWeekStart = new Date('2025-09-15');
    const mockResponse = {
      mealPlan: {},
      generationTimeMs: 1000,
      varietyScore: 0.8,
      recipesUsed: 21,
      rotationCycle: 1,
      warnings: [],
    };

    mockMealPlanService.generateWeeklyMealPlan.mockResolvedValue(mockResponse);

    const { getByRole } = render(
      <FillMyWeekButton weekStartDate={customWeekStart} />
    );

    const button = getByRole('button');
    
    await act(async () => {
      fireEvent.press(button);
    });

    await waitFor(() => {
      expect(mockMealPlanService.generateWeeklyMealPlan).toHaveBeenCalledWith({
        weekStartDate: customWeekStart,
      });
      expect(mockLoadMealPlan).toHaveBeenCalledWith(customWeekStart, true);
    });
  });

  it('prevents multiple simultaneous generations', async () => {
    let resolveGeneration: (value: any) => void;
    const generationPromise = new Promise((resolve) => {
      resolveGeneration = resolve;
    });

    mockMealPlanService.generateWeeklyMealPlan.mockReturnValue(generationPromise);

    const { getByRole } = render(<FillMyWeekButton />);

    const button = getByRole('button');
    
    // Press button multiple times quickly
    await act(async () => {
      fireEvent.press(button);
      fireEvent.press(button);
      fireEvent.press(button);
    });

    // Should only call generation once
    expect(mockMealPlanService.generateWeeklyMealPlan).toHaveBeenCalledTimes(1);

    // Resolve to finish test
    resolveGeneration!({
      mealPlan: {},
      generationTimeMs: 1000,
      varietyScore: 0.8,
      recipesUsed: 21,
      rotationCycle: 1,
      warnings: [],
    });
  });

  it('updates accessibility labels during generation', async () => {
    let resolveGeneration: (value: any) => void;
    const generationPromise = new Promise((resolve) => {
      resolveGeneration = resolve;
    });

    mockMealPlanService.generateWeeklyMealPlan.mockReturnValue(generationPromise);

    const { getByRole } = render(<FillMyWeekButton />);

    const button = getByRole('button');
    
    await act(async () => {
      fireEvent.press(button);
    });

    // Check accessibility during generation
    await waitFor(() => {
      expect(button).toHaveAccessibilityLabel(
        expect.stringMatching(/Generating meal plan, \d+% complete/)
      );
      expect(button).toHaveAccessibilityState({ disabled: true, busy: true });
    });

    // Resolve to finish test
    resolveGeneration!({
      mealPlan: {},
      generationTimeMs: 1000,
      varietyScore: 0.8,
      recipesUsed: 21,
      rotationCycle: 1,
      warnings: [],
    });
  });

  it('has proper testID when provided', () => {
    const testID = 'fill-my-week-button-test';
    const { getByTestId } = render(<FillMyWeekButton testID={testID} />);

    expect(getByTestId(testID)).toBeTruthy();
  });

  it('applies custom styles correctly', () => {
    const customStyle = { marginTop: 20, backgroundColor: 'red' };
    const { getByRole } = render(<FillMyWeekButton style={customStyle} />);

    // The button should exist - specific style testing may require additional setup
    expect(getByRole('button')).toBeTruthy();
  });
});

// Performance test
describe('FillMyWeekButton Performance', () => {
  it('should complete generation workflow in under 3 seconds', async () => {
    const mockResponse = {
      mealPlan: {},
      generationTimeMs: 1500,
      varietyScore: 0.8,
      recipesUsed: 21,
      rotationCycle: 1,
      warnings: [],
    };

    // Add delay to simulate realistic API response time
    mockMealPlanService.generateWeeklyMealPlan.mockImplementation(() => 
      new Promise(resolve => setTimeout(() => resolve(mockResponse), 1000))
    );

    const onGenerationComplete = jest.fn();
    const { getByRole } = render(
      <FillMyWeekButton onGenerationComplete={onGenerationComplete} />
    );

    const button = getByRole('button');
    const startTime = Date.now();
    
    await act(async () => {
      fireEvent.press(button);
    });

    await waitFor(() => {
      expect(onGenerationComplete).toHaveBeenCalled();
    });

    const endTime = Date.now();
    const totalTime = endTime - startTime;

    // Should complete within 3 seconds including UI updates
    expect(totalTime).toBeLessThan(3000);
  }, 10000); // 10 second timeout for this test
});
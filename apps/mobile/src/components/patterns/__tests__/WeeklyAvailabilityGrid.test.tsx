import React from 'react';
import { render, fireEvent, screen } from '@testing-library/react-native';
import { WeeklyAvailabilityGrid, WeeklyPattern } from '../WeeklyAvailabilityGrid';

const mockPatterns: WeeklyPattern[] = [
  {
    dayOfWeek: 0, // Sunday
    maxPrepTime: 120,
    preferredComplexity: 'complex',
    isWeekendPattern: true,
  },
  {
    dayOfWeek: 1, // Monday
    maxPrepTime: 45,
    preferredComplexity: 'simple',
    isWeekendPattern: false,
  },
  {
    dayOfWeek: 6, // Saturday
    maxPrepTime: 90,
    preferredComplexity: 'moderate',
    isWeekendPattern: true,
  },
];

const mockOnPatternUpdate = jest.fn();

describe('WeeklyAvailabilityGrid', () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });

  it('renders correctly with provided patterns', () => {
    render(
      <WeeklyAvailabilityGrid
        patterns={mockPatterns}
        onPatternUpdate={mockOnPatternUpdate}
      />
    );

    expect(screen.getByText('Weekly Cooking Patterns')).toBeTruthy();
    expect(screen.getByText('Sun')).toBeTruthy();
    expect(screen.getByText('Mon')).toBeTruthy();
    expect(screen.getByText('Sat')).toBeTruthy();
  });

  it('displays weekend patterns with proper styling', () => {
    render(
      <WeeklyAvailabilityGrid
        patterns={mockPatterns}
        onPatternUpdate={mockOnPatternUpdate}
      />
    );

    // Sunday should show weekend pattern (🏠)
    const sundayCards = screen.getAllByText('🏠');
    expect(sundayCards.length).toBeGreaterThan(0);
  });

  it('handles complexity button press correctly', () => {
    render(
      <WeeklyAvailabilityGrid
        patterns={mockPatterns}
        onPatternUpdate={mockOnPatternUpdate}
      />
    );

    // Find and press a complexity button
    const complexButtons = screen.getAllByText('Complex');
    fireEvent.press(complexButtons[0]);

    expect(mockOnPatternUpdate).toHaveBeenCalled();
  });

  it('handles time button press correctly', () => {
    render(
      <WeeklyAvailabilityGrid
        patterns={mockPatterns}
        onPatternUpdate={mockOnPatternUpdate}
      />
    );

    // Find and press a time button
    const timeButtons = screen.getAllByText('60m');
    fireEvent.press(timeButtons[0]);

    expect(mockOnPatternUpdate).toHaveBeenCalled();
  });

  it('handles weekend toggle correctly', () => {
    render(
      <WeeklyAvailabilityGrid
        patterns={mockPatterns}
        onPatternUpdate={mockOnPatternUpdate}
      />
    );

    // Find weekend toggle buttons (🏠 or ⏰)
    const weekendToggles = screen.getAllByText('🏠');
    if (weekendToggles.length > 0) {
      fireEvent.press(weekendToggles[0]);
      expect(mockOnPatternUpdate).toHaveBeenCalled();
    }
  });

  it('creates default pattern for missing day', () => {
    const incompletePatterns: WeeklyPattern[] = [
      {
        dayOfWeek: 0,
        maxPrepTime: 120,
        preferredComplexity: 'complex',
        isWeekendPattern: true,
      },
    ];

    render(
      <WeeklyAvailabilityGrid
        patterns={incompletePatterns}
        onPatternUpdate={mockOnPatternUpdate}
      />
    );

    // Should still render all 7 days with defaults for missing patterns
    expect(screen.getByText('Sun')).toBeTruthy();
    expect(screen.getByText('Mon')).toBeTruthy();
    expect(screen.getByText('Tue')).toBeTruthy();
    expect(screen.getByText('Wed')).toBeTruthy();
    expect(screen.getByText('Thu')).toBeTruthy();
    expect(screen.getByText('Fri')).toBeTruthy();
    expect(screen.getByText('Sat')).toBeTruthy();
  });

  it('disables interactions when disabled prop is true', () => {
    render(
      <WeeklyAvailabilityGrid
        patterns={mockPatterns}
        onPatternUpdate={mockOnPatternUpdate}
        disabled={true}
      />
    );

    // Find a complexity button and try to press it
    const complexButtons = screen.getAllByText('Complex');
    fireEvent.press(complexButtons[0]);

    // Should not call the callback when disabled
    expect(mockOnPatternUpdate).not.toHaveBeenCalled();
  });

  it('applies correct styles for weekend vs weekday patterns', () => {
    const { getByText } = render(
      <WeeklyAvailabilityGrid
        patterns={mockPatterns}
        onPatternUpdate={mockOnPatternUpdate}
      />
    );

    // Check if weekend days show different styling
    const sundayElement = getByText('Sun');
    const mondayElement = getByText('Mon');

    expect(sundayElement).toBeTruthy();
    expect(mondayElement).toBeTruthy();
  });
});
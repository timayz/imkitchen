import React from 'react';
import { render, fireEvent, screen } from '@testing-library/react-native';
import { ComplexityPreferenceSelector } from '../../src/components/atoms/ComplexityPreferenceSelector';

describe('ComplexityPreferenceSelector', () => {
  const mockOnValueChange = jest.fn();

  beforeEach(() => {
    jest.clearAllMocks();
  });

  it('renders all complexity options', () => {
    render(
      <ComplexityPreferenceSelector
        value="moderate"
        onValueChange={mockOnValueChange}
      />
    );

    expect(screen.getByText('Preferred Complexity')).toBeTruthy();
    expect(screen.getByText('Choose your cooking comfort level')).toBeTruthy();
    
    expect(screen.getByText('Simple')).toBeTruthy();
    expect(screen.getByText('Quick & easy recipes')).toBeTruthy();
    
    expect(screen.getByText('Moderate')).toBeTruthy();
    expect(screen.getByText('Balanced complexity')).toBeTruthy();
    
    expect(screen.getByText('Complex')).toBeTruthy();
    expect(screen.getByText('Advanced techniques')).toBeTruthy();
  });

  it('shows selected option correctly', () => {
    render(
      <ComplexityPreferenceSelector
        value="simple"
        onValueChange={mockOnValueChange}
      />
    );

    const simpleOption = screen.getByLabelText('Simple: Quick & easy recipes');
    expect(simpleOption.props.accessibilityState.selected).toBe(true);
  });

  it('calls onValueChange when option is pressed', () => {
    render(
      <ComplexityPreferenceSelector
        value="moderate"
        onValueChange={mockOnValueChange}
      />
    );

    const complexOption = screen.getByLabelText('Complex: Advanced techniques');
    fireEvent.press(complexOption);

    expect(mockOnValueChange).toHaveBeenCalledWith('complex');
  });

  it('does not call onValueChange when disabled', () => {
    render(
      <ComplexityPreferenceSelector
        value="moderate"
        onValueChange={mockOnValueChange}
        disabled={true}
      />
    );

    const simpleOption = screen.getByLabelText('Simple: Quick & easy recipes');
    fireEvent.press(simpleOption);

    expect(mockOnValueChange).not.toHaveBeenCalled();
  });

  it('applies disabled styling when disabled', () => {
    render(
      <ComplexityPreferenceSelector
        value="moderate"
        onValueChange={mockOnValueChange}
        disabled={true}
      />
    );

    const options = screen.getAllByRole('radio');
    options.forEach(option => {
      expect(option.props.disabled).toBe(true);
    });
  });

  it('shows correct icons for each complexity level', () => {
    render(
      <ComplexityPreferenceSelector
        value="moderate"
        onValueChange={mockOnValueChange}
      />
    );

    expect(screen.getByText('⚡')).toBeTruthy(); // Simple icon
    expect(screen.getByText('🍳')).toBeTruthy(); // Moderate icon  
    expect(screen.getByText('👨‍🍳')).toBeTruthy(); // Complex icon
  });

  it('has proper accessibility attributes', () => {
    render(
      <ComplexityPreferenceSelector
        value="moderate"
        onValueChange={mockOnValueChange}
      />
    );

    const options = screen.getAllByRole('radio');
    expect(options).toHaveLength(3);

    const moderateOption = screen.getByLabelText('Moderate: Balanced complexity');
    expect(moderateOption.props.accessibilityState.selected).toBe(true);
    expect(moderateOption.props.accessibilityRole).toBe('radio');
  });

  it('handles all complexity values correctly', () => {
    const { rerender } = render(
      <ComplexityPreferenceSelector
        value="simple"
        onValueChange={mockOnValueChange}
      />
    );

    expect(screen.getByLabelText('Simple: Quick & easy recipes').props.accessibilityState.selected).toBe(true);

    rerender(
      <ComplexityPreferenceSelector
        value="moderate"
        onValueChange={mockOnValueChange}
      />
    );

    expect(screen.getByLabelText('Moderate: Balanced complexity').props.accessibilityState.selected).toBe(true);

    rerender(
      <ComplexityPreferenceSelector
        value="complex"
        onValueChange={mockOnValueChange}
      />
    );

    expect(screen.getByLabelText('Complex: Advanced techniques').props.accessibilityState.selected).toBe(true);
  });
});
import React from 'react';
import { render, fireEvent, screen } from '@testing-library/react-native';
import { TimeConstraintSlider } from '../../src/components/atoms/TimeConstraintSlider';

describe('TimeConstraintSlider', () => {
  const mockOnValueChange = jest.fn();
  const mockOnSlidingComplete = jest.fn();

  beforeEach(() => {
    jest.clearAllMocks();
  });

  it('renders with default props', () => {
    render(
      <TimeConstraintSlider
        value={60}
        onValueChange={mockOnValueChange}
      />
    );

    expect(screen.getByText('Maximum Cook Time')).toBeTruthy();
    expect(screen.getByText('1h')).toBeTruthy();
    expect(screen.getByText('Standard cooking')).toBeTruthy();
  });

  it('displays correct time format for different values', () => {
    const { rerender } = render(
      <TimeConstraintSlider
        value={30}
        onValueChange={mockOnValueChange}
      />
    );

    expect(screen.getByText('30m')).toBeTruthy();

    rerender(
      <TimeConstraintSlider
        value={90}
        onValueChange={mockOnValueChange}
      />
    );

    expect(screen.getByText('1h 30m')).toBeTruthy();

    rerender(
      <TimeConstraintSlider
        value={120}
        onValueChange={mockOnValueChange}
      />
    );

    expect(screen.getByText('2h')).toBeTruthy();
  });

  it('shows correct descriptions for different time ranges', () => {
    const { rerender } = render(
      <TimeConstraintSlider
        value={15}
        onValueChange={mockOnValueChange}
      />
    );

    expect(screen.getByText('Quick meals')).toBeTruthy();

    rerender(
      <TimeConstraintSlider
        value={45}
        onValueChange={mockOnValueChange}
      />
    );

    expect(screen.getByText('Standard cooking')).toBeTruthy();

    rerender(
      <TimeConstraintSlider
        value={90}
        onValueChange={mockOnValueChange}
      />
    );

    expect(screen.getByText('Longer recipes')).toBeTruthy();

    rerender(
      <TimeConstraintSlider
        value={150}
        onValueChange={mockOnValueChange}
      />
    );

    expect(screen.getByText('Complex dishes')).toBeTruthy();
  });

  it('calls onValueChange when slider value changes', () => {
    render(
      <TimeConstraintSlider
        value={60}
        onValueChange={mockOnValueChange}
      />
    );

    const slider = screen.getByTestId('slider'); // Assuming Slider has testID
    fireEvent(slider, 'valueChange', 75);

    // Value should be rounded to nearest 5
    expect(mockOnValueChange).toHaveBeenCalledWith(75);
  });

  it('calls onSlidingComplete when provided', () => {
    render(
      <TimeConstraintSlider
        value={60}
        onValueChange={mockOnValueChange}
        onSlidingComplete={mockOnSlidingComplete}
      />
    );

    const slider = screen.getByTestId('slider');
    fireEvent(slider, 'slidingComplete', 83);

    // Value should be rounded to nearest 5
    expect(mockOnSlidingComplete).toHaveBeenCalledWith(85);
  });

  it('respects custom min/max values', () => {
    render(
      <TimeConstraintSlider
        value={45}
        onValueChange={mockOnValueChange}
        minimumValue={30}
        maximumValue={120}
      />
    );

    expect(screen.getByText('30m')).toBeTruthy(); // Min label
    expect(screen.getByText('2h')).toBeTruthy(); // Max label
  });

  it('handles disabled state', () => {
    render(
      <TimeConstraintSlider
        value={60}
        onValueChange={mockOnValueChange}
        disabled={true}
      />
    );

    const slider = screen.getByTestId('slider');
    expect(slider.props.disabled).toBe(true);
  });

  it('rounds values to nearest 5 minutes', () => {
    render(
      <TimeConstraintSlider
        value={60}
        onValueChange={mockOnValueChange}
      />
    );

    const slider = screen.getByTestId('slider');
    
    // Test rounding down
    fireEvent(slider, 'valueChange', 42);
    expect(mockOnValueChange).toHaveBeenCalledWith(40);

    // Test rounding up
    fireEvent(slider, 'valueChange', 48);
    expect(mockOnValueChange).toHaveBeenCalledWith(50);
  });
});
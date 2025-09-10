import React from 'react';
import { render } from '@testing-library/react-native';
import { ShoppingProgressBar } from '../ShoppingProgressBar';

describe('ShoppingProgressBar', () => {
  describe('Progress calculation', () => {
    it('calculates completion percentage correctly', () => {
      const { getByText } = render(
        <ShoppingProgressBar
          totalItems={10}
          completedItems={3}
          showPercentage={true}
        />
      );

      expect(getByText('30%')).toBeTruthy();
      expect(getByText('3 of 10 completed')).toBeTruthy();
    });

    it('handles zero total items', () => {
      const { getByText } = render(
        <ShoppingProgressBar
          totalItems={0}
          completedItems={0}
          showPercentage={true}
        />
      );

      expect(getByText('No items')).toBeTruthy();
    });

    it('shows completion celebration when all items done', () => {
      const { getByText } = render(
        <ShoppingProgressBar
          totalItems={5}
          completedItems={5}
          showPercentage={true}
        />
      );

      expect(getByText('🎉 All done!')).toBeTruthy();
      expect(getByText('100%')).toBeTruthy();
    });
  });

  describe('Progress text variations', () => {
    it('shows correct text for partial completion', () => {
      const { getByText } = render(
        <ShoppingProgressBar
          totalItems={20}
          completedItems={7}
        />
      );

      expect(getByText('7 of 20 completed')).toBeTruthy();
    });

    it('shows correct text for no completion', () => {
      const { getByText } = render(
        <ShoppingProgressBar
          totalItems={15}
          completedItems={0}
        />
      );

      expect(getByText('0 of 15 completed')).toBeTruthy();
    });
  });

  describe('Percentage display', () => {
    it('shows percentage when showPercentage is true', () => {
      const { getByText } = render(
        <ShoppingProgressBar
          totalItems={8}
          completedItems={2}
          showPercentage={true}
        />
      );

      expect(getByText('25%')).toBeTruthy();
    });

    it('hides percentage when showPercentage is false', () => {
      const { queryByText } = render(
        <ShoppingProgressBar
          totalItems={8}
          completedItems={2}
          showPercentage={false}
        />
      );

      expect(queryByText('25%')).toBeFalsy();
    });

    it('hides percentage by default', () => {
      const { queryByText } = render(
        <ShoppingProgressBar
          totalItems={8}
          completedItems={2}
        />
      );

      expect(queryByText('25%')).toBeFalsy();
    });
  });

  describe('Progress breakdown', () => {
    it('shows completed and remaining breakdown', () => {
      const { getByText } = render(
        <ShoppingProgressBar
          totalItems={12}
          completedItems={4}
        />
      );

      expect(getByText('4 completed')).toBeTruthy();
      expect(getByText('8 remaining')).toBeTruthy();
    });

    it('hides remaining count when all items completed', () => {
      const { getByText, queryByText } = render(
        <ShoppingProgressBar
          totalItems={6}
          completedItems={6}
        />
      );

      expect(getByText('6 completed')).toBeTruthy();
      expect(queryByText('0 remaining')).toBeFalsy();
    });

    it('shows only remaining when no items completed', () => {
      const { getByText } = render(
        <ShoppingProgressBar
          totalItems={10}
          completedItems={0}
        />
      );

      expect(getByText('0 completed')).toBeTruthy();
      expect(getByText('10 remaining')).toBeTruthy();
    });
  });

  describe('Edge cases', () => {
    it('handles negative values gracefully', () => {
      const { getByText } = render(
        <ShoppingProgressBar
          totalItems={5}
          completedItems={-1}
          showPercentage={true}
        />
      );

      // Should still render without crashing
      expect(getByText(/completed/)).toBeTruthy();
    });

    it('handles completedItems greater than totalItems', () => {
      const { getByText } = render(
        <ShoppingProgressBar
          totalItems={5}
          completedItems={7}
          showPercentage={true}
        />
      );

      // Should handle gracefully (likely showing 100% or more)
      expect(getByText(/completed/)).toBeTruthy();
    });
  });

  describe('Progress bar rendering', () => {
    it('renders progress bar when there are items', () => {
      const { getByTestId } = render(
        <ShoppingProgressBar
          totalItems={10}
          completedItems={3}
        />
      );

      // Should render progress bar elements
      const progressBar = getByTestId('progress-bar') || getByTestId('progress-bar-container');
      expect(progressBar).toBeTruthy();
    });

    it('does not render progress bar when no items', () => {
      const { queryByTestId } = render(
        <ShoppingProgressBar
          totalItems={0}
          completedItems={0}
        />
      );

      // Should not render progress bar when no items
      const progressBar = queryByTestId('progress-bar') || queryByTestId('progress-bar-container');
      expect(progressBar).toBeFalsy();
    });
  });
});
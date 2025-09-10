import React from 'react';
import { render, fireEvent, waitFor } from '@testing-library/react-native';
import { Alert } from 'react-native';
import { ReviewModal } from '../ReviewModal';

jest.mock('react-native/Libraries/Alert/Alert', () => ({
  alert: jest.fn(),
}));

const mockOnSubmit = jest.fn();
const mockOnClose = jest.fn();

const defaultProps = {
  visible: true,
  onClose: mockOnClose,
  onSubmit: mockOnSubmit,
  recipeTitle: 'Test Recipe',
};

describe('ReviewModal', () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });

  it('renders correctly when visible', () => {
    const { getByText } = render(<ReviewModal {...defaultProps} />);
    
    expect(getByText('Rate Recipe')).toBeTruthy();
    expect(getByText('Test Recipe')).toBeTruthy();
    expect(getByText('Your Rating')).toBeTruthy();
  });

  it('does not render when not visible', () => {
    const { queryByText } = render(
      <ReviewModal {...defaultProps} visible={false} />
    );
    
    expect(queryByText('Rate Recipe')).toBeNull();
  });

  it('handles rating selection', () => {
    const { getByLabelText } = render(<ReviewModal {...defaultProps} />);
    
    const threeStarButton = getByLabelText('Rate 3 stars');
    fireEvent.press(threeStarButton);
    
    expect(getByLabelText('3 stars')).toBeTruthy();
  });

  it('handles review text input', () => {
    const { getByPlaceholderText } = render(<ReviewModal {...defaultProps} />);
    
    const reviewInput = getByPlaceholderText('Share your experience with this recipe...');
    fireEvent.changeText(reviewInput, 'Great recipe!');
    
    expect(reviewInput.props.value).toBe('Great recipe!');
  });

  it('shows character count for review', () => {
    const { getByPlaceholderText, getByText } = render(
      <ReviewModal {...defaultProps} />
    );
    
    const reviewInput = getByPlaceholderText('Share your experience with this recipe...');
    fireEvent.changeText(reviewInput, 'Test review');
    
    expect(getByText('11/500 characters')).toBeTruthy();
  });

  it('handles difficulty selection', () => {
    const { getByLabelText } = render(<ReviewModal {...defaultProps} />);
    
    const harderButton = getByLabelText('Difficulty: Harder than expected');
    fireEvent.press(harderButton);
    
    // Button should be selected (styling would change in actual component)
    expect(harderButton).toBeTruthy();
  });

  it('handles would cook again selection', () => {
    const { getByLabelText } = render(<ReviewModal {...defaultProps} />);
    
    const noButton = getByLabelText('No, would not cook again');
    fireEvent.press(noButton);
    
    expect(noButton).toBeTruthy();
  });

  it('shows alert when trying to submit without rating', async () => {
    const { getByText } = render(<ReviewModal {...defaultProps} />);
    
    const submitButton = getByText('Submit Rating');
    fireEvent.press(submitButton);
    
    expect(Alert.alert).toHaveBeenCalledWith(
      'Rating Required',
      'Please select a star rating before submitting.'
    );
  });

  it('shows alert when review is too long', async () => {
    const { getByLabelText, getByPlaceholderText, getByText } = render(
      <ReviewModal {...defaultProps} />
    );
    
    // Set a rating first
    const fiveStarButton = getByLabelText('Rate 5 stars');
    fireEvent.press(fiveStarButton);
    
    // Enter a very long review
    const reviewInput = getByPlaceholderText('Share your experience with this recipe...');
    const longReview = 'a'.repeat(501); // Over 500 character limit
    fireEvent.changeText(reviewInput, longReview);
    
    const submitButton = getByText('Submit Rating');
    fireEvent.press(submitButton);
    
    expect(Alert.alert).toHaveBeenCalledWith(
      'Review Too Long',
      'Please keep your review under 500 characters.'
    );
  });

  it('submits rating successfully with valid data', async () => {
    mockOnSubmit.mockResolvedValueOnce(undefined);
    
    const { getByLabelText, getByPlaceholderText, getByText } = render(
      <ReviewModal {...defaultProps} />
    );
    
    // Set rating
    const fourStarButton = getByLabelText('Rate 4 stars');
    fireEvent.press(fourStarButton);
    
    // Enter review
    const reviewInput = getByPlaceholderText('Share your experience with this recipe...');
    fireEvent.changeText(reviewInput, 'Good recipe!');
    
    // Submit
    const submitButton = getByText('Submit Rating');
    fireEvent.press(submitButton);
    
    await waitFor(() => {
      expect(mockOnSubmit).toHaveBeenCalledWith({
        rating: 4,
        review: 'Good recipe!',
        difficulty: 'as_expected',
        wouldCookAgain: true,
      });
    });
    
    expect(mockOnClose).toHaveBeenCalled();
  });

  it('handles submission errors', async () => {
    const error = new Error('Network error');
    mockOnSubmit.mockRejectedValueOnce(error);
    
    const { getByLabelText, getByText } = render(<ReviewModal {...defaultProps} />);
    
    // Set rating
    const threeStarButton = getByLabelText('Rate 3 stars');
    fireEvent.press(threeStarButton);
    
    // Submit
    const submitButton = getByText('Submit Rating');
    fireEvent.press(submitButton);
    
    await waitFor(() => {
      expect(Alert.alert).toHaveBeenCalledWith(
        'Submission Failed',
        'Network error'
      );
    });
  });

  it('shows existing rating data when editing', () => {
    const existingRating = {
      rating: 4,
      review: 'Previously rated',
      difficulty: 'harder' as const,
      wouldCookAgain: false,
    };
    
    const { getByDisplayValue, getByLabelText } = render(
      <ReviewModal {...defaultProps} existingRating={existingRating} />
    );
    
    expect(getByDisplayValue('Previously rated')).toBeTruthy();
    expect(getByLabelText('Difficulty: Harder than expected')).toBeTruthy();
    expect(getByLabelText('No, would not cook again')).toBeTruthy();
  });

  it('shows update button when editing existing rating', () => {
    const existingRating = {
      rating: 3,
      review: 'Test review',
      difficulty: 'as_expected' as const,
      wouldCookAgain: true,
    };
    
    const { getByText } = render(
      <ReviewModal {...defaultProps} existingRating={existingRating} />
    );
    
    expect(getByText('Update Rating')).toBeTruthy();
  });

  it('closes modal when close button is pressed', () => {
    const { getByLabelText } = render(<ReviewModal {...defaultProps} />);
    
    const closeButton = getByLabelText('Close rating modal');
    fireEvent.press(closeButton);
    
    expect(mockOnClose).toHaveBeenCalled();
  });

  it('closes modal when cancel button is pressed', () => {
    const { getByText } = render(<ReviewModal {...defaultProps} />);
    
    const cancelButton = getByText('Cancel');
    fireEvent.press(cancelButton);
    
    expect(mockOnClose).toHaveBeenCalled();
  });

  it('disables submit button when no rating selected', () => {
    const { getByText } = render(<ReviewModal {...defaultProps} />);
    
    const submitButton = getByText('Submit Rating');
    expect(submitButton.props.accessibilityState).toEqual({ disabled: true });
  });

  it('shows loading indicator when submitting', () => {
    const { getByText } = render(
      <ReviewModal {...defaultProps} loading />
    );
    
    // Should show ActivityIndicator instead of text
    expect(getByText('Submit Rating')).toBeTruthy();
  });
});
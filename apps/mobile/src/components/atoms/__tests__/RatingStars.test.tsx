import React from 'react';
import { render, fireEvent } from '@testing-library/react-native';
import { RatingStars } from '../RatingStars';

describe('RatingStars', () => {
  it('renders correct number of stars', () => {
    const { getAllByText } = render(
      <RatingStars rating={3} maxRating={5} />
    );
    
    const stars = getAllByText('★');
    expect(stars).toHaveLength(5);
  });

  it('displays correct rating value', () => {
    const { getAllByText } = render(
      <RatingStars rating={3.5} />
    );
    
    const stars = getAllByText('★');
    // First 4 stars should be active (rounded up from 3.5)
    expect(stars[0].props.style).toMatchObject({
      color: '#FFD700'
    });
    expect(stars[3].props.style).toMatchObject({
      color: '#FFD700'
    });
    expect(stars[4].props.style).toMatchObject({
      color: '#DDD'
    });
  });

  it('handles interactive mode correctly', () => {
    const mockOnRatingChange = jest.fn();
    const { getAllByText } = render(
      <RatingStars
        rating={0}
        interactive
        onRatingChange={mockOnRatingChange}
      />
    );
    
    const stars = getAllByText('★');
    fireEvent.press(stars[2]); // Third star (rating 3)
    
    expect(mockOnRatingChange).toHaveBeenCalledWith(3);
  });

  it('does not trigger rating change in non-interactive mode', () => {
    const mockOnRatingChange = jest.fn();
    const { getAllByText } = render(
      <RatingStars
        rating={3}
        interactive={false}
        onRatingChange={mockOnRatingChange}
      />
    );
    
    const stars = getAllByText('★');
    fireEvent.press(stars[0]);
    
    expect(mockOnRatingChange).not.toHaveBeenCalled();
  });

  it('applies correct size styles', () => {
    const { getAllByText, rerender } = render(
      <RatingStars rating={3} size="small" />
    );
    
    let stars = getAllByText('★');
    expect(stars[0].props.style).toMatchObject({
      fontSize: 14
    });
    
    rerender(<RatingStars rating={3} size="large" />);
    stars = getAllByText('★');
    expect(stars[0].props.style).toMatchObject({
      fontSize: 24
    });
  });

  it('provides correct accessibility labels', () => {
    const { getByLabelText } = render(
      <RatingStars rating={3} interactive />
    );
    
    expect(getByLabelText('Rate 1 star')).toBeTruthy();
    expect(getByLabelText('Rate 5 stars')).toBeTruthy();
  });

  it('provides correct non-interactive accessibility label', () => {
    const { getByLabelText } = render(
      <RatingStars rating={3} interactive={false} />
    );
    
    expect(getByLabelText('Rating: 3 out of 5 stars')).toBeTruthy();
  });
});
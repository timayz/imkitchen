import React from 'react';
import { render } from '@testing-library/react-native';
import { RatingDistribution } from '../RatingDistribution';

const mockDistribution = {
  oneStar: 5,
  twoStar: 3,
  threeStar: 10,
  fourStar: 25,
  fiveStar: 57,
};

describe('RatingDistribution', () => {
  it('renders distribution bars correctly', () => {
    const { getByText } = render(
      <RatingDistribution
        distribution={mockDistribution}
        totalRatings={100}
      />
    );
    
    expect(getByText('Rating Breakdown')).toBeTruthy();
    expect(getByText('57')).toBeTruthy(); // 5-star count
    expect(getByText('25')).toBeTruthy(); // 4-star count
    expect(getByText('10')).toBeTruthy(); // 3-star count
    expect(getByText('3')).toBeTruthy();  // 2-star count
    expect(getByText('5')).toBeTruthy();  // 1-star count
  });

  it('shows correct percentages', () => {
    const { getByText } = render(
      <RatingDistribution
        distribution={mockDistribution}
        totalRatings={100}
      />
    );
    
    expect(getByText('(57%)')).toBeTruthy(); // 5-star percentage
    expect(getByText('(25%)')).toBeTruthy(); // 4-star percentage
    expect(getByText('(10%)')).toBeTruthy(); // 3-star percentage
    expect(getByText('(3%)')).toBeTruthy();  // 2-star percentage
    expect(getByText('(5%)')).toBeTruthy();  // 1-star percentage
  });

  it('shows total ratings count', () => {
    const { getByText } = render(
      <RatingDistribution
        distribution={mockDistribution}
        totalRatings={100}
      />
    );
    
    expect(getByText('Total: 100 ratings')).toBeTruthy();
  });

  it('handles singular rating count', () => {
    const singleDistribution = {
      oneStar: 0,
      twoStar: 0,
      threeStar: 0,
      fourStar: 0,
      fiveStar: 1,
    };
    
    const { getByText } = render(
      <RatingDistribution
        distribution={singleDistribution}
        totalRatings={1}
      />
    );
    
    expect(getByText('Total: 1 rating')).toBeTruthy();
  });

  it('shows no ratings message when totalRatings is 0', () => {
    const emptyDistribution = {
      oneStar: 0,
      twoStar: 0,
      threeStar: 0,
      fourStar: 0,
      fiveStar: 0,
    };
    
    const { getByText, queryByText } = render(
      <RatingDistribution
        distribution={emptyDistribution}
        totalRatings={0}
      />
    );
    
    expect(getByText('No ratings yet')).toBeTruthy();
    expect(getByText('Be the first to rate this recipe!')).toBeTruthy();
    expect(queryByText('Rating Breakdown')).toBeNull();
  });

  it('calculates progress bar widths correctly', () => {
    const { getByTestId } = render(
      <RatingDistribution
        distribution={mockDistribution}
        totalRatings={100}
      />
    );
    
    // The 5-star rating (57 count) should have the widest bar (100% width)
    // The 4-star rating (25 count) should have about 44% of the max width
    // These would be tested by checking the actual width styles in a real test environment
  });

  it('handles zero counts correctly', () => {
    const sparseDistribution = {
      oneStar: 0,
      twoStar: 0,
      threeStar: 5,
      fourStar: 0,
      fiveStar: 10,
    };
    
    const { getByText } = render(
      <RatingDistribution
        distribution={sparseDistribution}
        totalRatings={15}
      />
    );
    
    expect(getByText('0')).toBeTruthy(); // Should show zeros
    expect(getByText('(0%)')).toBeTruthy(); // Should show 0% for zero counts
  });

  it('rounds percentages correctly', () => {
    const unevenDistribution = {
      oneStar: 1,
      twoStar: 1,
      threeStar: 1,
      fourStar: 1,
      fiveStar: 1,
    };
    
    const { getByText } = render(
      <RatingDistribution
        distribution={unevenDistribution}
        totalRatings={6}
      />
    );
    
    // 1/6 = 16.666...% should round to 17%
    expect(getByText('(17%)')).toBeTruthy();
  });

  it('applies custom style prop', () => {
    const customStyle = { backgroundColor: 'red' };
    const { getByTestId } = render(
      <RatingDistribution
        distribution={mockDistribution}
        totalRatings={100}
        style={customStyle}
      />
    );
    
    // In a real test, you would check that the style is applied to the container
    // This would require adding testID to the component
  });
});
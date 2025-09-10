import React from 'react';
import { render, fireEvent } from '@testing-library/react-native';
import { RecipeAttributionCard } from '../RecipeAttributionCard';
import type { RecipeAttribution, ContributorProfile } from '@imkitchen/shared-types';

describe('RecipeAttributionCard', () => {
  const mockOnContributorPress = jest.fn();
  const mockOnViewMetrics = jest.fn();

  const mockAttribution: RecipeAttribution = {
    id: 'attr-123',
    recipeId: 'recipe-123',
    originalContributorId: 'user-456',
    originalContributor: 'Chef Sarah',
    importDate: new Date('2024-09-01'),
    preserveAttribution: true,
    customizations: ['title', 'servings'],
    communityMetrics: {
      totalImports: 1250,
      averageRating: 4.7,
      totalRatings: 89,
      trendingScore: 8.5,
      popularityRank: 3,
    },
    recipeChain: [
      {
        contributorId: 'user-789',
        contributorName: 'Original Chef',
        adaptedAt: new Date('2024-08-15'),
        recipeId: 'recipe-original',
      },
      {
        contributorId: 'user-456',
        contributorName: 'Chef Sarah',
        adaptedAt: new Date('2024-08-20'),
        recipeId: 'recipe-adapted',
      },
    ],
    engagementStats: {
      weeklyViews: 450,
      savesToMealPlans: 67,
      socialShares: 23,
    },
  };

  const mockContributor: ContributorProfile = {
    id: 'user-456',
    username: 'chef_sarah',
    displayName: 'Chef Sarah',
    avatarUrl: 'https://example.com/avatar.jpg',
    totalRecipes: 42,
    averageRating: 4.6,
    totalImports: 2890,
    joinedAt: new Date('2023-06-15'),
    badges: [
      {
        id: 'badge-1',
        name: 'Popular Creator',
        description: 'Has recipes with 1000+ imports',
        emoji: '🔥',
        earnedAt: new Date('2024-07-01'),
      },
      {
        id: 'badge-2',
        name: 'Community Favorite',
        description: 'Voted community favorite',
        emoji: '❤️',
        earnedAt: new Date('2024-08-01'),
      },
    ],
    achievements: [],
    bio: 'Passionate home cook sharing family recipes',
    location: 'San Francisco, CA',
  };

  beforeEach(() => {
    jest.clearAllMocks();
  });

  it('renders correctly with basic attribution', () => {
    const { getByText } = render(
      <RecipeAttributionCard
        attribution={mockAttribution}
        onContributorPress={mockOnContributorPress}
      />
    );

    expect(getByText('Recipe Attribution')).toBeTruthy();
    expect(getByText('Chef Sarah')).toBeTruthy();
    expect(getByText('Sep 1, 2024')).toBeTruthy();
    expect(getByText('1,250')).toBeTruthy(); // Total imports
    expect(getByText('⭐ 4.7')).toBeTruthy(); // Average rating
  });

  it('displays contributor information when provided', () => {
    const { getByText } = render(
      <RecipeAttributionCard
        attribution={mockAttribution}
        contributor={mockContributor}
        onContributorPress={mockOnContributorPress}
      />
    );

    expect(getByText('42 recipes')).toBeTruthy();
    expect(getByText('⭐ 4.6')).toBeTruthy(); // Contributor rating
    expect(getByText('🔥')).toBeTruthy(); // First badge
    expect(getByText('❤️')).toBeTruthy(); // Second badge
    expect(getByText('+0')).toBeTruthy(); // Badge overflow (2 shown, 0 more)
  });

  it('handles contributor profile press', () => {
    const { getByLabelText } = render(
      <RecipeAttributionCard
        attribution={mockAttribution}
        contributor={mockContributor}
        onContributorPress={mockOnContributorPress}
      />
    );

    const contributorButton = getByLabelText("View Chef Sarah's profile");
    fireEvent.press(contributorButton);

    expect(mockOnContributorPress).toHaveBeenCalledWith('user-456');
  });

  it('shows view metrics button when callback provided', () => {
    const { getByText } = render(
      <RecipeAttributionCard
        attribution={mockAttribution}
        onViewMetrics={mockOnViewMetrics}
      />
    );

    const viewMetricsButton = getByText('View Details');
    fireEvent.press(viewMetricsButton);

    expect(mockOnViewMetrics).toHaveBeenCalled();
  });

  it('displays import customizations when present', () => {
    const { getByText } = render(
      <RecipeAttributionCard attribution={mockAttribution} />
    );

    expect(getByText('with 2 customizations')).toBeTruthy();
  });

  it('shows community metrics section', () => {
    const { getByText } = render(
      <RecipeAttributionCard attribution={mockAttribution} />
    );

    expect(getByText('Community Metrics')).toBeTruthy();
    expect(getByText('1,250')).toBeTruthy(); // Imports
    expect(getByText('89')).toBeTruthy(); // Reviews
    expect(getByText('📈 8.5')).toBeTruthy(); // Trending score
    expect(getByText('#3 most popular this week')).toBeTruthy(); // Popularity rank
  });

  it('displays recipe chain when showFullDetails is true', () => {
    const { getByText } = render(
      <RecipeAttributionCard
        attribution={mockAttribution}
        showFullDetails={true}
      />
    );

    expect(getByText('Recipe Chain')).toBeTruthy();
    expect(getByText('Original Chef')).toBeTruthy();
    expect(getByText('Chef Sarah')).toBeTruthy();
    expect(getByText('This recipe has been adapted 1 time')).toBeTruthy();
  });

  it('displays engagement stats when showFullDetails is true', () => {
    const { getByText } = render(
      <RecipeAttributionCard
        attribution={mockAttribution}
        showFullDetails={true}
      />
    );

    expect(getByText('Community Engagement')).toBeTruthy();
    expect(getByText('450')).toBeTruthy(); // Weekly views
    expect(getByText('67')).toBeTruthy(); // Meal plan saves
    expect(getByText('23')).toBeTruthy(); // Social shares
  });

  it('shows attribution preservation notice when enabled', () => {
    const { getByText } = render(
      <RecipeAttributionCard attribution={mockAttribution} />
    );

    expect(getByText('ℹ️ Attribution preserved as requested by original contributor')).toBeTruthy();
  });

  it('hides attribution notice when preservation is disabled', () => {
    const attributionWithoutPreservation = {
      ...mockAttribution,
      preserveAttribution: false,
    };

    const { queryByText } = render(
      <RecipeAttributionCard attribution={attributionWithoutPreservation} />
    );

    expect(queryByText('ℹ️ Attribution preserved as requested by original contributor')).toBeNull();
  });

  it('handles missing contributor avatar gracefully', () => {
    const contributorWithoutAvatar = {
      ...mockContributor,
      avatarUrl: undefined,
    };

    const { getByText } = render(
      <RecipeAttributionCard
        attribution={mockAttribution}
        contributor={contributorWithoutAvatar}
      />
    );

    // Should show first letter of name as fallback
    expect(getByText('C')).toBeTruthy();
  });

  it('displays correct recipe chain description for multiple adaptations', () => {
    const attributionWithLongerChain = {
      ...mockAttribution,
      recipeChain: [
        ...mockAttribution.recipeChain,
        {
          contributorId: 'user-999',
          contributorName: 'Another Chef',
          adaptedAt: new Date('2024-08-25'),
          recipeId: 'recipe-adapted-2',
        },
      ],
    };

    const { getByText } = render(
      <RecipeAttributionCard
        attribution={attributionWithLongerChain}
        showFullDetails={true}
      />
    );

    expect(getByText('This recipe has been adapted 2 times')).toBeTruthy();
  });

  it('handles recipe without engagement stats', () => {
    const attributionWithoutEngagement = {
      ...mockAttribution,
      engagementStats: undefined,
    };

    const { queryByText } = render(
      <RecipeAttributionCard
        attribution={attributionWithoutEngagement}
        showFullDetails={true}
      />
    );

    expect(queryByText('Community Engagement')).toBeNull();
  });

  it('displays badge overflow correctly', () => {
    const contributorWithManyBadges = {
      ...mockContributor,
      badges: [
        ...mockContributor.badges,
        {
          id: 'badge-3',
          name: 'Expert',
          description: 'Expert level',
          emoji: '👨‍🍳',
          earnedAt: new Date('2024-09-01'),
        },
      ],
    };

    const { getByText } = render(
      <RecipeAttributionCard
        attribution={mockAttribution}
        contributor={contributorWithManyBadges}
      />
    );

    expect(getByText('+1')).toBeTruthy(); // Should show +1 for the third badge
  });

  it('handles recipe without customizations', () => {
    const attributionWithoutCustomizations = {
      ...mockAttribution,
      customizations: [],
    };

    const { queryByText } = render(
      <RecipeAttributionCard attribution={attributionWithoutCustomizations} />
    );

    expect(queryByText(/with \d+ customizations?/)).toBeNull();
  });

  it('applies custom styles when provided', () => {
    const customStyle = { marginTop: 20 };
    const { getByTestId } = render(
      <RecipeAttributionCard
        attribution={mockAttribution}
        style={customStyle}
        testID="attribution-card"
      />
    );

    const card = getByTestId('attribution-card');
    expect(card.props.style).toEqual(expect.arrayContaining([
      expect.objectContaining(customStyle)
    ]));
  });
});
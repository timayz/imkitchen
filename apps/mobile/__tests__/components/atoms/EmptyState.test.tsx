import React from 'react';
import { render, fireEvent, waitFor } from '@testing-library/react-native';
import { View, Text, Animated } from 'react-native';
import EmptyState, { 
  EmptyMealPlanState, 
  EmptyRecipeCollectionState, 
  EmptyShoppingListState 
} from '../../../src/components/atoms/EmptyState';
import { ThemeProvider } from '../../../src/theme/ThemeProvider';

// Mock the theme and animations
jest.mock('../../../src/theme/ThemeProvider', () => ({
  useTheme: () => ({
    colors: {
      primary: '#007AFF',
      background: '#FFFFFF',
      backgroundSecondary: '#F8F8F8',
      border: '#E5E5E5',
      text: '#000000',
      textSecondary: '#6C6C70',
      textInverse: '#FFFFFF',
    },
  }),
  ThemeProvider: ({ children }: { children: React.ReactNode }) => children,
}));

jest.mock('../../../src/theme/animations', () => ({
  createFadeAnimation: jest.fn((animValue, toValue, duration) => 
    Animated.timing(animValue, {
      toValue,
      duration,
      useNativeDriver: false,
    })
  ),
  ANIMATION_DURATION: {
    NORMAL: 300,
  },
}));

const renderWithTheme = (component: React.ReactElement) => {
  return render(
    <ThemeProvider>
      {component}
    </ThemeProvider>
  );
};

describe('EmptyState', () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });

  describe('Basic Rendering', () => {
    it('renders with required props', () => {
      const { getByText, getByTestId } = renderWithTheme(
        <EmptyState
          title="Test Title"
          message="Test message"
          testID="empty-state"
        />
      );

      expect(getByTestId('empty-state')).toBeTruthy();
      expect(getByText('Test Title')).toBeTruthy();
      expect(getByText('Test message')).toBeTruthy();
      expect(getByText('📝')).toBeTruthy(); // Default icon
    });

    it('renders with custom string icon', () => {
      const { getByText } = renderWithTheme(
        <EmptyState
          icon="🎉"
          title="Custom Icon"
          message="Test message"
        />
      );

      expect(getByText('🎉')).toBeTruthy();
    });

    it('renders with custom React node icon', () => {
      const CustomIcon = () => <Text testID="custom-icon">Custom</Text>;
      
      const { getByTestId } = renderWithTheme(
        <EmptyState
          icon={<CustomIcon />}
          title="Custom React Icon"
          message="Test message"
        />
      );

      expect(getByTestId('custom-icon')).toBeTruthy();
    });
  });

  describe('Variants', () => {
    it('renders default variant correctly', () => {
      const { getByTestId } = renderWithTheme(
        <EmptyState
          title="Default Variant"
          message="Test message"
          variant="default"
          testID="default-variant"
        />
      );

      expect(getByTestId('default-variant')).toBeTruthy();
    });

    it('renders compact variant correctly', () => {
      const { getByTestId } = renderWithTheme(
        <EmptyState
          title="Compact Variant"
          message="Test message"
          variant="compact"
          testID="compact-variant"
        />
      );

      expect(getByTestId('compact-variant')).toBeTruthy();
    });

    it('renders centered variant correctly', () => {
      const { getByTestId } = renderWithTheme(
        <EmptyState
          title="Centered Variant"
          message="Test message"
          variant="centered"
          testID="centered-variant"
        />
      );

      expect(getByTestId('centered-variant')).toBeTruthy();
    });
  });

  describe('Actions', () => {
    it('renders primary action button', () => {
      const mockPress = jest.fn();
      
      const { getByText } = renderWithTheme(
        <EmptyState
          title="With Action"
          message="Test message"
          primaryAction={{
            text: "Primary Action",
            onPress: mockPress,
          }}
        />
      );

      const button = getByText('Primary Action');
      expect(button).toBeTruthy();
      
      fireEvent.press(button);
      expect(mockPress).toHaveBeenCalledTimes(1);
    });

    it('renders secondary action button', () => {
      const mockPress = jest.fn();
      
      const { getByText } = renderWithTheme(
        <EmptyState
          title="With Action"
          message="Test message"
          secondaryAction={{
            text: "Secondary Action",
            onPress: mockPress,
          }}
        />
      );

      const button = getByText('Secondary Action');
      expect(button).toBeTruthy();
      
      fireEvent.press(button);
      expect(mockPress).toHaveBeenCalledTimes(1);
    });

    it('renders both primary and secondary actions', () => {
      const mockPrimaryPress = jest.fn();
      const mockSecondaryPress = jest.fn();
      
      const { getByText } = renderWithTheme(
        <EmptyState
          title="With Both Actions"
          message="Test message"
          primaryAction={{
            text: "Primary",
            onPress: mockPrimaryPress,
          }}
          secondaryAction={{
            text: "Secondary",
            onPress: mockSecondaryPress,
          }}
        />
      );

      expect(getByText('Primary')).toBeTruthy();
      expect(getByText('Secondary')).toBeTruthy();
    });

    it('renders action buttons with icons', () => {
      const mockPress = jest.fn();
      
      const { getByText } = renderWithTheme(
        <EmptyState
          title="With Icon Action"
          message="Test message"
          primaryAction={{
            text: "Action",
            onPress: mockPress,
            icon: "🚀",
          }}
        />
      );

      expect(getByText('🚀')).toBeTruthy();
      expect(getByText('Action')).toBeTruthy();
    });

    it('does not render actions container when no actions provided', () => {
      const { queryByText } = renderWithTheme(
        <EmptyState
          title="No Actions"
          message="Test message"
        />
      );

      // Since there are no buttons, there should be no button-related accessibility roles
      expect(queryByText('Primary')).toBeNull();
      expect(queryByText('Secondary')).toBeNull();
    });
  });

  describe('Animation', () => {
    it('triggers fade animation when animated is true', async () => {
      const { createFadeAnimation } = require('../../../src/theme/animations');
      
      renderWithTheme(
        <EmptyState
          title="Animated"
          message="Test message"
          animated={true}
        />
      );

      await waitFor(() => {
        expect(createFadeAnimation).toHaveBeenCalledWith(
          expect.any(Animated.Value),
          1,
          300 // ANIMATION_DURATION.NORMAL
        );
      });
    });

    it('skips animation when animated is false', () => {
      const { createFadeAnimation } = require('../../../src/theme/animations');
      
      renderWithTheme(
        <EmptyState
          title="Not Animated"
          message="Test message"
          animated={false}
        />
      );

      expect(createFadeAnimation).not.toHaveBeenCalled();
    });
  });

  describe('Accessibility', () => {
    it('has correct accessibility attributes for action buttons', () => {
      const mockPress = jest.fn();
      
      const { getByRole } = renderWithTheme(
        <EmptyState
          title="Accessibility Test"
          message="Test message"
          primaryAction={{
            text: "Accessible Button",
            onPress: mockPress,
          }}
        />
      );

      const button = getByRole('button');
      expect(button).toBeTruthy();
      expect(button.props.accessibilityLabel).toBe('Accessible Button');
    });
  });

  describe('Custom Styling', () => {
    it('applies custom container style', () => {
      const customStyle = { backgroundColor: '#FF0000' };
      
      const { getByTestId } = renderWithTheme(
        <EmptyState
          title="Custom Style"
          message="Test message"
          style={customStyle}
          testID="styled-empty-state"
        />
      );

      expect(getByTestId('styled-empty-state')).toBeTruthy();
    });

    it('applies custom title style', () => {
      const customTitleStyle = { fontSize: 24 };
      
      const { getByText } = renderWithTheme(
        <EmptyState
          title="Custom Title"
          message="Test message"
          titleStyle={customTitleStyle}
        />
      );

      expect(getByText('Custom Title')).toBeTruthy();
    });

    it('applies custom message style', () => {
      const customMessageStyle = { fontSize: 18 };
      
      const { getByText } = renderWithTheme(
        <EmptyState
          title="Title"
          message="Custom Message"
          messageStyle={customMessageStyle}
        />
      );

      expect(getByText('Custom Message')).toBeTruthy();
    });
  });
});

describe('EmptyMealPlanState', () => {
  it('renders meal plan empty state with correct content', () => {
    const { getByText, getByTestId } = renderWithTheme(
      <EmptyMealPlanState />
    );

    expect(getByTestId('empty-meal-plan-state')).toBeTruthy();
    expect(getByText('🍽️')).toBeTruthy();
    expect(getByText('No Meal Plan Yet')).toBeTruthy();
    expect(getByText(/Create your first meal plan/)).toBeTruthy();
  });

  it('renders with action callbacks', () => {
    const mockCreateMealPlan = jest.fn();
    const mockImportRecipes = jest.fn();
    
    const { getByText } = renderWithTheme(
      <EmptyMealPlanState
        onCreateMealPlan={mockCreateMealPlan}
        onImportRecipes={mockImportRecipes}
      />
    );

    expect(getByText('Fill My Week')).toBeTruthy();
    expect(getByText('Add Recipes First')).toBeTruthy();
    
    fireEvent.press(getByText('Fill My Week'));
    expect(mockCreateMealPlan).toHaveBeenCalledTimes(1);
    
    fireEvent.press(getByText('Add Recipes First'));
    expect(mockImportRecipes).toHaveBeenCalledTimes(1);
  });

  it('does not render actions when callbacks not provided', () => {
    const { queryByText } = renderWithTheme(
      <EmptyMealPlanState />
    );

    expect(queryByText('Fill My Week')).toBeNull();
    expect(queryByText('Add Recipes First')).toBeNull();
  });
});

describe('EmptyRecipeCollectionState', () => {
  it('renders recipe collection empty state with correct content', () => {
    const { getByText, getByTestId } = renderWithTheme(
      <EmptyRecipeCollectionState />
    );

    expect(getByTestId('empty-recipe-collection-state')).toBeTruthy();
    expect(getByText('📚')).toBeTruthy();
    expect(getByText('No Recipes Yet')).toBeTruthy();
    expect(getByText(/Start building your recipe collection/)).toBeTruthy();
  });

  it('renders with action callbacks', () => {
    const mockBrowseCommunity = jest.fn();
    const mockAddRecipe = jest.fn();
    
    const { getByText } = renderWithTheme(
      <EmptyRecipeCollectionState
        onBrowseCommunity={mockBrowseCommunity}
        onAddRecipe={mockAddRecipe}
      />
    );

    expect(getByText('Browse Community')).toBeTruthy();
    expect(getByText('Add Recipe')).toBeTruthy();
    
    fireEvent.press(getByText('Browse Community'));
    expect(mockBrowseCommunity).toHaveBeenCalledTimes(1);
    
    fireEvent.press(getByText('Add Recipe'));
    expect(mockAddRecipe).toHaveBeenCalledTimes(1);
  });
});

describe('EmptyShoppingListState', () => {
  it('renders shopping list empty state with correct content and compact variant', () => {
    const { getByText, getByTestId } = renderWithTheme(
      <EmptyShoppingListState />
    );

    expect(getByTestId('empty-shopping-list-state')).toBeTruthy();
    expect(getByText('🛒')).toBeTruthy();
    expect(getByText('Shopping List is Empty')).toBeTruthy();
    expect(getByText(/Your shopping list will appear here/)).toBeTruthy();
  });

  it('renders with action callbacks', () => {
    const mockGenerateFromMealPlan = jest.fn();
    const mockAddItems = jest.fn();
    
    const { getByText } = renderWithTheme(
      <EmptyShoppingListState
        onGenerateFromMealPlan={mockGenerateFromMealPlan}
        onAddItems={mockAddItems}
      />
    );

    expect(getByText('Generate from Meal Plan')).toBeTruthy();
    expect(getByText('Add Items')).toBeTruthy();
    
    fireEvent.press(getByText('Generate from Meal Plan'));
    expect(mockGenerateFromMealPlan).toHaveBeenCalledTimes(1);
    
    fireEvent.press(getByText('Add Items'));
    expect(mockAddItems).toHaveBeenCalledTimes(1);
  });
});
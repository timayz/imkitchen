import React from 'react';
import { render, fireEvent, waitFor } from '@testing-library/react-native';
import { Alert } from 'react-native';
import { TagManagementInterface } from '../TagManagementInterface';
import { TagManagementService } from '../../../services/tag_management_service';

// Mock the service
jest.mock('../../../services/tag_management_service');
const MockedTagManagementService = TagManagementService as jest.MockedClass<typeof TagManagementService>;

// Mock Alert
jest.spyOn(Alert, 'alert').mockImplementation(() => {});

describe('TagManagementInterface', () => {
  const mockOnTagsUpdate = jest.fn();
  const mockService = {
    getPopularTags: jest.fn(),
    getRecipeTags: jest.fn(),
    getTagSuggestions: jest.fn(),
    validateTags: jest.fn(),
    updateRecipeTags: jest.fn(),
    voteOnTag: jest.fn(),
  };

  beforeEach(() => {
    jest.clearAllMocks();
    MockedTagManagementService.mockImplementation(() => mockService as any);
    
    // Default mock implementations
    mockService.getPopularTags.mockResolvedValue([
      { tag: 'vegetarian', usageCount: 150, category: 'dietary', trendingUp: true },
      { tag: 'quick', usageCount: 200, category: 'time', trendingUp: false },
    ]);
    
    mockService.getRecipeTags.mockResolvedValue({
      recipeId: 'recipe-1',
      userTags: [],
      communityTags: [
        { tag: 'healthy', voteCount: 5, userVoted: false, confidence: 0.8 },
      ],
      tagStats: {},
    });
  });

  const defaultProps = {
    recipeId: 'recipe-1',
    initialTags: ['comfort-food', 'easy'],
    onTagsUpdate: mockOnTagsUpdate,
  };

  it('renders correctly with initial tags', async () => {
    const { getByText } = render(<TagManagementInterface {...defaultProps} />);

    await waitFor(() => {
      expect(getByText('Your Tags (2/10)')).toBeTruthy();
      expect(getByText('#comfort-food')).toBeTruthy();
      expect(getByText('#easy')).toBeTruthy();
    });
  });

  it('displays popular tags section', async () => {
    const { getByText } = render(<TagManagementInterface {...defaultProps} />);

    await waitFor(() => {
      expect(getByText('Popular Tags')).toBeTruthy();
      expect(getByText('#vegetarian')).toBeTruthy();
      expect(getByText('#quick')).toBeTruthy();
    });
  });

  it('shows community tags when enabled', async () => {
    const { getByText } = render(
      <TagManagementInterface {...defaultProps} showCommunityTags={true} />
    );

    await waitFor(() => {
      expect(getByText('Community Tags')).toBeTruthy();
      expect(getByText('#healthy')).toBeTruthy();
    });
  });

  it('allows adding new tags via input', async () => {
    mockService.validateTags.mockResolvedValue({
      validTags: ['new-tag'],
      invalidTags: [],
    });
    mockService.updateRecipeTags.mockResolvedValue({
      recipeId: 'recipe-1',
      updatedTags: ['comfort-food', 'easy', 'new-tag'],
      message: 'Success',
    });

    const { getByPlaceholderText, getByText } = render(
      <TagManagementInterface {...defaultProps} />
    );

    const input = getByPlaceholderText('Type to search or add new tag...');
    const addButton = getByText('Add');

    fireEvent.changeText(input, 'new-tag');
    fireEvent.press(addButton);

    await waitFor(() => {
      expect(mockService.validateTags).toHaveBeenCalledWith(['new-tag']);
      expect(mockService.updateRecipeTags).toHaveBeenCalledWith('recipe-1', ['new-tag'], 'add');
      expect(mockOnTagsUpdate).toHaveBeenCalledWith(['comfort-food', 'easy', 'new-tag']);
    });
  });

  it('prevents adding duplicate tags', async () => {
    const { getByPlaceholderText, getByText } = render(
      <TagManagementInterface {...defaultProps} />
    );

    const input = getByPlaceholderText('Type to search or add new tag...');
    const addButton = getByText('Add');

    fireEvent.changeText(input, 'comfort-food'); // Already exists
    fireEvent.press(addButton);

    // Should not call the service
    expect(mockService.validateTags).not.toHaveBeenCalled();
    expect(mockService.updateRecipeTags).not.toHaveBeenCalled();
  });

  it('enforces maximum tag limit', async () => {
    const propsWithMaxTags = {
      ...defaultProps,
      initialTags: ['tag1', 'tag2'],
      maxTags: 2,
    };

    const { getByPlaceholderText, getByText } = render(
      <TagManagementInterface {...propsWithMaxTags} />
    );

    const input = getByPlaceholderText('Type to search or add new tag...');
    const addButton = getByText('Add');

    fireEvent.changeText(input, 'new-tag');
    fireEvent.press(addButton);

    await waitFor(() => {
      expect(Alert.alert).toHaveBeenCalledWith(
        'Tag Limit Reached',
        'You can only add up to 2 tags per recipe.'
      );
    });
  });

  it('allows removing tags', async () => {
    mockService.updateRecipeTags.mockResolvedValue({
      recipeId: 'recipe-1',
      updatedTags: ['easy'],
      message: 'Success',
    });

    const { getAllByText } = render(<TagManagementInterface {...defaultProps} />);

    // Find the remove button (×) for the first tag
    const removeButtons = getAllByText('×');
    fireEvent.press(removeButtons[0]);

    await waitFor(() => {
      expect(mockService.updateRecipeTags).toHaveBeenCalledWith(
        'recipe-1',
        ['comfort-food'],
        'remove'
      );
      expect(mockOnTagsUpdate).toHaveBeenCalledWith(['easy']);
    });
  });

  it('shows tag suggestions when typing', async () => {
    mockService.getTagSuggestions.mockResolvedValue([
      { tag: 'healthy', confidence: 0.9, usageCount: 100, category: 'style' },
      { tag: 'hearty', confidence: 0.7, usageCount: 50, category: 'style' },
    ]);

    const { getByPlaceholderText, getByText } = render(
      <TagManagementInterface {...defaultProps} />
    );

    const input = getByPlaceholderText('Type to search or add new tag...');
    fireEvent.changeText(input, 'hea');

    await waitFor(() => {
      expect(mockService.getTagSuggestions).toHaveBeenCalledWith(
        'hea',
        'recipe-1',
        ['comfort-food', 'easy'],
        10
      );
      expect(getByText('Suggestions')).toBeTruthy();
      expect(getByText('#healthy')).toBeTruthy();
      expect(getByText('#hearty')).toBeTruthy();
    });
  });

  it('allows voting on community tags', async () => {
    mockService.voteOnTag.mockResolvedValue({
      tag: 'healthy',
      voteCount: 6,
      userVoted: true,
      message: 'Success',
    });

    const { getByLabelText } = render(
      <TagManagementInterface {...defaultProps} showCommunityTags={true} />
    );

    await waitFor(() => {
      const upvoteButton = getByLabelText('Upvote for healthy');
      fireEvent.press(upvoteButton);
    });

    await waitFor(() => {
      expect(mockService.voteOnTag).toHaveBeenCalledWith('recipe-1', 'healthy', 'upvote');
    });
  });

  it('handles validation errors gracefully', async () => {
    mockService.validateTags.mockResolvedValue({
      validTags: [],
      invalidTags: [{ tag: 'invalid-tag', reason: 'Contains banned words' }],
    });

    const { getByPlaceholderText, getByText } = render(
      <TagManagementInterface {...defaultProps} />
    );

    const input = getByPlaceholderText('Type to search or add new tag...');
    const addButton = getByText('Add');

    fireEvent.changeText(input, 'invalid-tag');
    fireEvent.press(addButton);

    await waitFor(() => {
      expect(Alert.alert).toHaveBeenCalledWith('Invalid Tag', 'Contains banned words');
    });
  });

  it('is read-only when specified', () => {
    const { queryByPlaceholderText, queryByText } = render(
      <TagManagementInterface {...defaultProps} isReadOnly={true} />
    );

    expect(queryByPlaceholderText('Type to search or add new tag...')).toBeNull();
    expect(queryByText('Add Tags')).toBeNull();
    expect(queryByText('×')).toBeNull(); // No remove buttons
  });

  it('adds tags from popular tags section', async () => {
    mockService.validateTags.mockResolvedValue({
      validTags: ['vegetarian'],
      invalidTags: [],
    });
    mockService.updateRecipeTags.mockResolvedValue({
      recipeId: 'recipe-1',
      updatedTags: ['comfort-food', 'easy', 'vegetarian'],
      message: 'Success',
    });

    const { getByLabelText } = render(<TagManagementInterface {...defaultProps} />);

    await waitFor(() => {
      const popularTag = getByLabelText('Add popular tag vegetarian');
      fireEvent.press(popularTag);
    });

    await waitFor(() => {
      expect(mockService.updateRecipeTags).toHaveBeenCalledWith('recipe-1', ['vegetarian'], 'add');
    });
  });

  it('handles network errors when adding tags', async () => {
    mockService.validateTags.mockResolvedValue({
      validTags: ['new-tag'],
      invalidTags: [],
    });
    mockService.updateRecipeTags.mockRejectedValue(new Error('Network error'));

    const { getByPlaceholderText, getByText } = render(
      <TagManagementInterface {...defaultProps} />
    );

    const input = getByPlaceholderText('Type to search or add new tag...');
    const addButton = getByText('Add');

    fireEvent.changeText(input, 'new-tag');
    fireEvent.press(addButton);

    await waitFor(() => {
      expect(Alert.alert).toHaveBeenCalledWith('Error', 'Failed to add tag. Please try again.');
    });
  });

  it('shows trending indicator on popular tags', async () => {
    const { getByText } = render(<TagManagementInterface {...defaultProps} />);

    await waitFor(() => {
      // The vegetarian tag should have a trending indicator (📈)
      expect(getByText('📈')).toBeTruthy();
    });
  });

  it('calls onTagsUpdate when tags change', async () => {
    mockService.validateTags.mockResolvedValue({
      validTags: ['new-tag'],
      invalidTags: [],
    });
    mockService.updateRecipeTags.mockResolvedValue({
      recipeId: 'recipe-1',
      updatedTags: ['comfort-food', 'easy', 'new-tag'],
      message: 'Success',
    });

    const { getByPlaceholderText, getByText } = render(
      <TagManagementInterface {...defaultProps} />
    );

    const input = getByPlaceholderText('Type to search or add new tag...');
    const addButton = getByText('Add');

    fireEvent.changeText(input, 'new-tag');
    fireEvent.press(addButton);

    await waitFor(() => {
      expect(mockOnTagsUpdate).toHaveBeenCalledWith(['comfort-food', 'easy', 'new-tag']);
    });
  });
});
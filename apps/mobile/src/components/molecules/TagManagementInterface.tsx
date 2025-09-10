import React, { useState, useEffect, useCallback } from 'react';
import {
  View,
  Text,
  TextInput,
  TouchableOpacity,
  FlatList,
  StyleSheet,
  Alert,
  ActivityIndicator,
  Keyboard,
  ScrollView,
} from 'react-native';
import { useDebouncedCallback } from 'use-debounce';
import { TagManagementService } from '../../services/tag_management_service';
import { CategoryChip } from '../atoms/CategoryChip';
import type { TagSuggestion, PopularTag, RecipeTag } from '@imkitchen/shared-types';

interface TagManagementInterfaceProps {
  recipeId: string;
  initialTags: string[];
  onTagsUpdate: (tags: string[]) => void;
  isReadOnly?: boolean;
  showCommunityTags?: boolean;
  maxTags?: number;
  style?: any;
}

export const TagManagementInterface: React.FC<TagManagementInterfaceProps> = ({
  recipeId,
  initialTags,
  onTagsUpdate,
  isReadOnly = false,
  showCommunityTags = true,
  maxTags = 10,
  style,
}) => {
  const [tags, setTags] = useState<string[]>(initialTags);
  const [inputValue, setInputValue] = useState('');
  const [suggestions, setSuggestions] = useState<TagSuggestion[]>([]);
  const [popularTags, setPopularTags] = useState<PopularTag[]>([]);
  const [communityTags, setCommunityTags] = useState<RecipeTag[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [showSuggestions, setShowSuggestions] = useState(false);

  const tagService = new TagManagementService();

  // Load initial data
  useEffect(() => {
    loadPopularTags();
    if (showCommunityTags) {
      loadCommunityTags();
    }
  }, [recipeId, showCommunityTags]);

  // Update parent when tags change
  useEffect(() => {
    if (JSON.stringify(tags) !== JSON.stringify(initialTags)) {
      onTagsUpdate(tags);
    }
  }, [tags, onTagsUpdate, initialTags]);

  const loadPopularTags = async () => {
    try {
      const popular = await tagService.getPopularTags(20, undefined, 'week');
      setPopularTags(popular);
    } catch (error) {
      console.error('Failed to load popular tags:', error);
    }
  };

  const loadCommunityTags = async () => {
    try {
      const { communityTags: community } = await tagService.getRecipeTags(recipeId);
      setCommunityTags(community);
    } catch (error) {
      console.error('Failed to load community tags:', error);
    }
  };

  const debouncedGetSuggestions = useDebouncedCallback(
    async (query: string) => {
      if (query.length < 2) {
        setSuggestions([]);
        setShowSuggestions(false);
        return;
      }

      setIsLoading(true);
      try {
        const suggestions = await tagService.getTagSuggestions(
          query,
          recipeId,
          tags,
          10
        );
        setSuggestions(suggestions);
        setShowSuggestions(true);
      } catch (error) {
        console.error('Failed to get tag suggestions:', error);
        setSuggestions([]);
      } finally {
        setIsLoading(false);
      }
    },
    300
  );

  const handleInputChange = (text: string) => {
    setInputValue(text);
    debouncedGetSuggestions(text);
  };

  const addTag = async (tag: string) => {
    const cleanedTag = tag.trim().toLowerCase();
    
    if (cleanedTag === '' || tags.includes(cleanedTag)) {
      return;
    }

    if (tags.length >= maxTags) {
      Alert.alert('Tag Limit Reached', `You can only add up to ${maxTags} tags per recipe.`);
      return;
    }

    // Validate tag
    try {
      const { validTags, invalidTags } = await tagService.validateTags([cleanedTag]);
      
      if (invalidTags.length > 0) {
        Alert.alert('Invalid Tag', invalidTags[0].reason);
        return;
      }

      const newTags = [...tags, cleanedTag];
      setTags(newTags);
      setInputValue('');
      setShowSuggestions(false);
      Keyboard.dismiss();

      // Update recipe tags on server
      try {
        await tagService.updateRecipeTags(recipeId, [cleanedTag], 'add');
      } catch (error) {
        console.error('Failed to update recipe tags on server:', error);
        // Rollback local change
        setTags(tags);
        Alert.alert('Error', 'Failed to add tag. Please try again.');
      }
    } catch (error) {
      console.error('Tag validation failed:', error);
      Alert.alert('Error', 'Failed to validate tag. Please try again.');
    }
  };

  const removeTag = async (tagToRemove: string) => {
    const newTags = tags.filter(tag => tag !== tagToRemove);
    setTags(newTags);

    // Update recipe tags on server
    try {
      await tagService.updateRecipeTags(recipeId, [tagToRemove], 'remove');
    } catch (error) {
      console.error('Failed to remove recipe tag on server:', error);
      // Rollback local change
      setTags(tags);
      Alert.alert('Error', 'Failed to remove tag. Please try again.');
    }
  };

  const handleSubmit = () => {
    if (inputValue.trim()) {
      addTag(inputValue.trim());
    }
  };

  const voteOnCommunityTag = async (tag: string, action: 'upvote' | 'downvote') => {
    try {
      const { voteCount, userVoted } = await tagService.voteOnTag(recipeId, tag, action);
      
      // Update local community tags
      setCommunityTags(prev => 
        prev.map(t => 
          t.tag === tag 
            ? { ...t, voteCount, userVoted }
            : t
        )
      );
    } catch (error) {
      console.error('Failed to vote on community tag:', error);
      Alert.alert('Error', 'Failed to vote on tag. Please try again.');
    }
  };

  const renderTag = ({ item: tag }: { item: string }) => (
    <View style={styles.tagContainer}>
      <Text style={styles.tagText}>#{tag}</Text>
      {!isReadOnly && (
        <TouchableOpacity
          onPress={() => removeTag(tag)}
          style={styles.removeButton}
          accessibilityLabel={`Remove ${tag} tag`}
          accessibilityRole="button"
        >
          <Text style={styles.removeButtonText}>×</Text>
        </TouchableOpacity>
      )}
    </View>
  );

  const renderSuggestion = ({ item }: { item: TagSuggestion }) => (
    <TouchableOpacity
      style={styles.suggestionItem}
      onPress={() => addTag(item.tag)}
      accessibilityLabel={`Add ${item.tag} tag`}
      accessibilityRole="button"
    >
      <Text style={styles.suggestionText}>#{item.tag}</Text>
      <View style={styles.suggestionMeta}>
        <Text style={styles.suggestionCategory}>{item.category}</Text>
        <Text style={styles.suggestionUsage}>{item.usageCount} uses</Text>
      </View>
    </TouchableOpacity>
  );

  const renderPopularTag = ({ item }: { item: PopularTag }) => (
    <TouchableOpacity
      style={[
        styles.popularTagChip,
        item.trendingUp && styles.trendingTag,
        tags.includes(item.tag) && styles.selectedTag,
      ]}
      onPress={() => !tags.includes(item.tag) && addTag(item.tag)}
      disabled={tags.includes(item.tag) || isReadOnly}
      accessibilityLabel={`Add popular tag ${item.tag}`}
      accessibilityRole="button"
    >
      <Text style={[
        styles.popularTagText,
        tags.includes(item.tag) && styles.selectedTagText,
      ]}>
        #{item.tag}
      </Text>
      {item.trendingUp && (
        <Text style={styles.trendingIndicator}>📈</Text>
      )}
    </TouchableOpacity>
  );

  const renderCommunityTag = ({ item }: { item: RecipeTag }) => (
    <View style={styles.communityTagContainer}>
      <Text style={styles.communityTagText}>#{item.tag}</Text>
      <View style={styles.communityTagActions}>
        <TouchableOpacity
          style={[
            styles.voteButton,
            item.userVoted && styles.votedButton,
          ]}
          onPress={() => voteOnCommunityTag(item.tag, item.userVoted ? 'remove' : 'upvote')}
          accessibilityLabel={`${item.userVoted ? 'Remove vote' : 'Upvote'} for ${item.tag}`}
          accessibilityRole="button"
        >
          <Text style={styles.voteButtonText}>👍</Text>
        </TouchableOpacity>
        <Text style={styles.voteCount}>{item.voteCount}</Text>
        <TouchableOpacity
          style={styles.voteButton}
          onPress={() => voteOnCommunityTag(item.tag, 'downvote')}
          accessibilityLabel={`Downvote ${item.tag}`}
          accessibilityRole="button"
        >
          <Text style={styles.voteButtonText}>👎</Text>
        </TouchableOpacity>
      </View>
    </View>
  );

  return (
    <ScrollView style={[styles.container, style]}>
      {/* Current Tags */}
      <View style={styles.section}>
        <Text style={styles.sectionTitle}>Your Tags ({tags.length}/{maxTags})</Text>
        {tags.length > 0 ? (
          <FlatList
            data={tags}
            renderItem={renderTag}
            keyExtractor={(item) => item}
            horizontal
            showsHorizontalScrollIndicator={false}
            style={styles.tagsList}
            contentContainerStyle={styles.tagsListContent}
          />
        ) : (
          <Text style={styles.emptyText}>No tags added yet</Text>
        )}
      </View>

      {/* Tag Input */}
      {!isReadOnly && (
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>Add Tags</Text>
          <View style={styles.inputContainer}>
            <TextInput
              style={styles.input}
              value={inputValue}
              onChangeText={handleInputChange}
              placeholder="Type to search or add new tag..."
              maxLength={30}
              returnKeyType="done"
              onSubmitEditing={handleSubmit}
              autoCapitalize="none"
              autoCorrect={false}
              accessibilityLabel="Tag input field"
            />
            {isLoading && (
              <ActivityIndicator style={styles.loadingIndicator} size="small" />
            )}
            {inputValue.length > 0 && (
              <TouchableOpacity
                style={styles.addButton}
                onPress={handleSubmit}
                accessibilityLabel="Add tag"
                accessibilityRole="button"
              >
                <Text style={styles.addButtonText}>Add</Text>
              </TouchableOpacity>
            )}
          </View>

          {/* Suggestions */}
          {showSuggestions && suggestions.length > 0 && (
            <View style={styles.suggestionsContainer}>
              <Text style={styles.suggestionsTitle}>Suggestions</Text>
              <FlatList
                data={suggestions}
                renderItem={renderSuggestion}
                keyExtractor={(item) => item.tag}
                style={styles.suggestionsList}
                nestedScrollEnabled
              />
            </View>
          )}
        </View>
      )}

      {/* Popular Tags */}
      <View style={styles.section}>
        <Text style={styles.sectionTitle}>Popular Tags</Text>
        <FlatList
          data={popularTags.filter(tag => !tags.includes(tag.tag))}
          renderItem={renderPopularTag}
          keyExtractor={(item) => item.tag}
          horizontal
          showsHorizontalScrollIndicator={false}
          style={styles.popularTagsList}
          contentContainerStyle={styles.popularTagsContent}
        />
      </View>

      {/* Community Tags */}
      {showCommunityTags && communityTags.length > 0 && (
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>Community Tags</Text>
          <FlatList
            data={communityTags}
            renderItem={renderCommunityTag}
            keyExtractor={(item) => item.tag}
            style={styles.communityTagsList}
            nestedScrollEnabled
          />
        </View>
      )}
    </ScrollView>
  );
};

const styles = StyleSheet.create({
  container: {
    flex: 1,
    padding: 16,
  },
  section: {
    marginBottom: 24,
  },
  sectionTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#333',
    marginBottom: 12,
  },
  tagsList: {
    maxHeight: 60,
  },
  tagsListContent: {
    alignItems: 'center',
  },
  tagContainer: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: '#007AFF',
    borderRadius: 16,
    paddingHorizontal: 12,
    paddingVertical: 6,
    marginRight: 8,
    marginBottom: 8,
  },
  tagText: {
    color: '#fff',
    fontSize: 14,
    fontWeight: '500',
  },
  removeButton: {
    marginLeft: 8,
    width: 20,
    height: 20,
    borderRadius: 10,
    backgroundColor: 'rgba(255, 255, 255, 0.3)',
    alignItems: 'center',
    justifyContent: 'center',
  },
  removeButtonText: {
    color: '#fff',
    fontSize: 16,
    fontWeight: 'bold',
  },
  emptyText: {
    color: '#666',
    fontStyle: 'italic',
    textAlign: 'center',
    paddingVertical: 20,
  },
  inputContainer: {
    flexDirection: 'row',
    alignItems: 'center',
    borderWidth: 1,
    borderColor: '#ddd',
    borderRadius: 8,
    paddingHorizontal: 12,
    backgroundColor: '#fff',
  },
  input: {
    flex: 1,
    paddingVertical: 12,
    fontSize: 16,
    color: '#333',
  },
  loadingIndicator: {
    marginLeft: 8,
  },
  addButton: {
    backgroundColor: '#007AFF',
    paddingHorizontal: 16,
    paddingVertical: 8,
    borderRadius: 6,
    marginLeft: 8,
  },
  addButtonText: {
    color: '#fff',
    fontSize: 14,
    fontWeight: '600',
  },
  suggestionsContainer: {
    marginTop: 8,
    backgroundColor: '#f9f9f9',
    borderRadius: 8,
    padding: 12,
    maxHeight: 200,
  },
  suggestionsTitle: {
    fontSize: 14,
    fontWeight: '600',
    color: '#666',
    marginBottom: 8,
  },
  suggestionsList: {
    maxHeight: 150,
  },
  suggestionItem: {
    paddingVertical: 8,
    borderBottomWidth: 1,
    borderBottomColor: '#eee',
  },
  suggestionText: {
    fontSize: 16,
    color: '#007AFF',
    fontWeight: '500',
  },
  suggestionMeta: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    marginTop: 2,
  },
  suggestionCategory: {
    fontSize: 12,
    color: '#666',
    textTransform: 'capitalize',
  },
  suggestionUsage: {
    fontSize: 12,
    color: '#999',
  },
  popularTagsList: {
    maxHeight: 60,
  },
  popularTagsContent: {
    alignItems: 'center',
  },
  popularTagChip: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: '#f0f0f0',
    borderRadius: 16,
    paddingHorizontal: 12,
    paddingVertical: 6,
    marginRight: 8,
    marginBottom: 8,
  },
  trendingTag: {
    backgroundColor: '#E8F5E8',
    borderWidth: 1,
    borderColor: '#4CAF50',
  },
  selectedTag: {
    backgroundColor: '#ccc',
    opacity: 0.6,
  },
  popularTagText: {
    color: '#333',
    fontSize: 14,
    fontWeight: '500',
  },
  selectedTagText: {
    color: '#666',
  },
  trendingIndicator: {
    marginLeft: 4,
    fontSize: 12,
  },
  communityTagsList: {
    maxHeight: 200,
  },
  communityTagContainer: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    paddingVertical: 12,
    paddingHorizontal: 8,
    borderBottomWidth: 1,
    borderBottomColor: '#eee',
  },
  communityTagText: {
    fontSize: 16,
    color: '#007AFF',
    fontWeight: '500',
    flex: 1,
  },
  communityTagActions: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  voteButton: {
    padding: 8,
    borderRadius: 16,
    backgroundColor: '#f0f0f0',
  },
  votedButton: {
    backgroundColor: '#007AFF',
  },
  voteButtonText: {
    fontSize: 16,
  },
  voteCount: {
    marginHorizontal: 8,
    fontSize: 16,
    fontWeight: '600',
    color: '#333',
    minWidth: 30,
    textAlign: 'center',
  },
});
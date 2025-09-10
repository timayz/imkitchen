import React, { useState, useCallback, useEffect } from 'react';
import {
  View,
  Text,
  TextInput,
  TouchableOpacity,
  Modal,
  StyleSheet,
  ScrollView,
  KeyboardAvoidingView,
  Platform,
  Alert,
  ActivityIndicator,
} from 'react-native';
import { RatingStars } from '../atoms/RatingStars';
import type { RecipeRatingSubmission } from '@imkitchen/shared-types';

interface ReviewModalProps {
  visible: boolean;
  onClose: () => void;
  onSubmit: (rating: RecipeRatingSubmission) => Promise<void>;
  recipeTitle: string;
  existingRating?: {
    rating: number;
    review?: string;
    difficulty?: 'easier' | 'as_expected' | 'harder';
    wouldCookAgain?: boolean;
  };
  loading?: boolean;
}

export const ReviewModal: React.FC<ReviewModalProps> = ({
  visible,
  onClose,
  onSubmit,
  recipeTitle,
  existingRating,
  loading = false,
}) => {
  const [rating, setRating] = useState(existingRating?.rating || 0);
  const [review, setReview] = useState(existingRating?.review || '');
  const [difficulty, setDifficulty] = useState<'easier' | 'as_expected' | 'harder'>(
    existingRating?.difficulty || 'as_expected'
  );
  const [wouldCookAgain, setWouldCookAgain] = useState(
    existingRating?.wouldCookAgain ?? true
  );
  const [isSubmitting, setIsSubmitting] = useState(false);

  // Reset form when modal opens/closes
  useEffect(() => {
    if (visible) {
      setRating(existingRating?.rating || 0);
      setReview(existingRating?.review || '');
      setDifficulty(existingRating?.difficulty || 'as_expected');
      setWouldCookAgain(existingRating?.wouldCookAgain ?? true);
    }
  }, [visible, existingRating]);

  const handleSubmit = useCallback(async () => {
    if (rating === 0) {
      Alert.alert('Rating Required', 'Please select a star rating before submitting.');
      return;
    }

    if (review.length > 500) {
      Alert.alert('Review Too Long', 'Please keep your review under 500 characters.');
      return;
    }

    setIsSubmitting(true);

    try {
      const submission: RecipeRatingSubmission = {
        rating,
        review: review.trim() || undefined,
        difficulty,
        wouldCookAgain,
      };

      await onSubmit(submission);
      onClose();
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Failed to submit rating';
      Alert.alert('Submission Failed', message);
    } finally {
      setIsSubmitting(false);
    }
  }, [rating, review, difficulty, wouldCookAgain, onSubmit, onClose]);

  const getDifficultyLabel = (value: string) => {
    switch (value) {
      case 'easier':
        return 'Easier than expected';
      case 'harder':
        return 'Harder than expected';
      default:
        return 'As expected';
    }
  };

  const getDifficultyColor = (value: string, isSelected: boolean) => {
    if (!isSelected) return '#666';
    
    switch (value) {
      case 'easier':
        return '#4CAF50';
      case 'harder':
        return '#F44336';
      default:
        return '#FF9800';
    }
  };

  return (
    <Modal
      visible={visible}
      transparent
      animationType="slide"
      onRequestClose={onClose}
    >
      <KeyboardAvoidingView
        style={styles.overlay}
        behavior={Platform.OS === 'ios' ? 'padding' : 'height'}
      >
        <View style={styles.container}>
          <ScrollView
            style={styles.content}
            keyboardShouldPersistTaps="handled"
            showsVerticalScrollIndicator={false}
          >
            {/* Header */}
            <View style={styles.header}>
              <Text style={styles.title}>Rate Recipe</Text>
              <TouchableOpacity
                onPress={onClose}
                style={styles.closeButton}
                accessibilityLabel="Close rating modal"
              >
                <Text style={styles.closeButtonText}>✕</Text>
              </TouchableOpacity>
            </View>

            {/* Recipe Title */}
            <Text style={styles.recipeTitle} numberOfLines={2}>
              {recipeTitle}
            </Text>

            {/* Star Rating */}
            <View style={styles.section}>
              <Text style={styles.sectionTitle}>Your Rating</Text>
              <View style={styles.ratingContainer}>
                <RatingStars
                  rating={rating}
                  interactive
                  size="large"
                  onRatingChange={setRating}
                />
                <Text style={styles.ratingText}>
                  {rating > 0 ? `${rating} star${rating > 1 ? 's' : ''}` : 'Tap to rate'}
                </Text>
              </View>
            </View>

            {/* Written Review */}
            <View style={styles.section}>
              <Text style={styles.sectionTitle}>
                Written Review <Text style={styles.optional}>(Optional)</Text>
              </Text>
              <TextInput
                style={styles.textInput}
                multiline
                numberOfLines={4}
                placeholder="Share your experience with this recipe..."
                value={review}
                onChangeText={setReview}
                maxLength={500}
                textAlignVertical="top"
                accessibilityLabel="Write your review"
              />
              <Text style={styles.characterCount}>
                {review.length}/500 characters
              </Text>
            </View>

            {/* Difficulty Assessment */}
            <View style={styles.section}>
              <Text style={styles.sectionTitle}>How was the difficulty?</Text>
              <View style={styles.difficultyContainer}>
                {['easier', 'as_expected', 'harder'].map((option) => (
                  <TouchableOpacity
                    key={option}
                    style={[
                      styles.difficultyOption,
                      difficulty === option && styles.difficultyOptionActive,
                      {
                        borderColor: getDifficultyColor(option, difficulty === option),
                      },
                    ]}
                    onPress={() => setDifficulty(option as typeof difficulty)}
                    accessibilityLabel={`Difficulty: ${getDifficultyLabel(option)}`}
                    accessibilityState={{ selected: difficulty === option }}
                  >
                    <Text
                      style={[
                        styles.difficultyText,
                        {
                          color: getDifficultyColor(option, difficulty === option),
                          fontWeight: difficulty === option ? '600' : '400',
                        },
                      ]}
                    >
                      {getDifficultyLabel(option)}
                    </Text>
                  </TouchableOpacity>
                ))}
              </View>
            </View>

            {/* Would Cook Again */}
            <View style={styles.section}>
              <Text style={styles.sectionTitle}>Would you cook this again?</Text>
              <View style={styles.difficultyContainer}>
                <TouchableOpacity
                  style={[
                    styles.difficultyOption,
                    wouldCookAgain && styles.difficultyOptionActive,
                    {
                      borderColor: wouldCookAgain ? '#4CAF50' : '#666',
                    },
                  ]}
                  onPress={() => setWouldCookAgain(true)}
                  accessibilityLabel="Yes, would cook again"
                  accessibilityState={{ selected: wouldCookAgain }}
                >
                  <Text
                    style={[
                      styles.difficultyText,
                      {
                        color: wouldCookAgain ? '#4CAF50' : '#666',
                        fontWeight: wouldCookAgain ? '600' : '400',
                      },
                    ]}
                  >
                    Yes
                  </Text>
                </TouchableOpacity>
                
                <TouchableOpacity
                  style={[
                    styles.difficultyOption,
                    !wouldCookAgain && styles.difficultyOptionActive,
                    {
                      borderColor: !wouldCookAgain ? '#F44336' : '#666',
                    },
                  ]}
                  onPress={() => setWouldCookAgain(false)}
                  accessibilityLabel="No, would not cook again"
                  accessibilityState={{ selected: !wouldCookAgain }}
                >
                  <Text
                    style={[
                      styles.difficultyText,
                      {
                        color: !wouldCookAgain ? '#F44336' : '#666',
                        fontWeight: !wouldCookAgain ? '600' : '400',
                      },
                    ]}
                  >
                    No
                  </Text>
                </TouchableOpacity>
              </View>
            </View>
          </ScrollView>

          {/* Action Buttons */}
          <View style={styles.actionButtons}>
            <TouchableOpacity
              style={[styles.button, styles.cancelButton]}
              onPress={onClose}
              disabled={isSubmitting}
              accessibilityLabel="Cancel rating"
            >
              <Text style={[styles.buttonText, styles.cancelButtonText]}>
                Cancel
              </Text>
            </TouchableOpacity>
            
            <TouchableOpacity
              style={[
                styles.button,
                styles.submitButton,
                (isSubmitting || rating === 0) && styles.buttonDisabled,
              ]}
              onPress={handleSubmit}
              disabled={isSubmitting || rating === 0}
              accessibilityLabel={existingRating ? 'Update rating' : 'Submit rating'}
            >
              {isSubmitting ? (
                <ActivityIndicator size="small" color="#fff" />
              ) : (
                <Text style={[styles.buttonText, styles.submitButtonText]}>
                  {existingRating ? 'Update' : 'Submit'} Rating
                </Text>
              )}
            </TouchableOpacity>
          </View>
        </View>
      </KeyboardAvoidingView>
    </Modal>
  );
};

const styles = StyleSheet.create({
  overlay: {
    flex: 1,
    backgroundColor: 'rgba(0, 0, 0, 0.5)',
    justifyContent: 'flex-end',
  },
  container: {
    backgroundColor: '#fff',
    borderTopLeftRadius: 20,
    borderTopRightRadius: 20,
    maxHeight: '90%',
  },
  content: {
    paddingHorizontal: 20,
    paddingBottom: 20,
  },
  header: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    paddingVertical: 20,
    borderBottomWidth: 1,
    borderBottomColor: '#eee',
    marginBottom: 20,
  },
  title: {
    fontSize: 20,
    fontWeight: '700',
    color: '#333',
  },
  closeButton: {
    padding: 4,
  },
  closeButtonText: {
    fontSize: 18,
    color: '#666',
    fontWeight: '600',
  },
  recipeTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#333',
    marginBottom: 24,
    lineHeight: 22,
  },
  section: {
    marginBottom: 24,
  },
  sectionTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#333',
    marginBottom: 12,
  },
  optional: {
    fontWeight: '400',
    color: '#666',
  },
  ratingContainer: {
    alignItems: 'center',
  },
  ratingText: {
    fontSize: 14,
    color: '#666',
    marginTop: 8,
  },
  textInput: {
    borderWidth: 1,
    borderColor: '#ddd',
    borderRadius: 8,
    padding: 12,
    fontSize: 16,
    minHeight: 100,
    backgroundColor: '#f9f9f9',
  },
  characterCount: {
    fontSize: 12,
    color: '#666',
    textAlign: 'right',
    marginTop: 4,
  },
  difficultyContainer: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: 8,
  },
  difficultyOption: {
    borderWidth: 1.5,
    borderRadius: 20,
    paddingHorizontal: 16,
    paddingVertical: 8,
    backgroundColor: '#f9f9f9',
  },
  difficultyOptionActive: {
    backgroundColor: 'rgba(0, 122, 255, 0.1)',
  },
  difficultyText: {
    fontSize: 14,
  },
  actionButtons: {
    flexDirection: 'row',
    paddingHorizontal: 20,
    paddingVertical: 16,
    paddingBottom: 32,
    borderTopWidth: 1,
    borderTopColor: '#eee',
    gap: 12,
  },
  button: {
    flex: 1,
    paddingVertical: 14,
    borderRadius: 8,
    alignItems: 'center',
    justifyContent: 'center',
    minHeight: 48,
  },
  cancelButton: {
    backgroundColor: '#f5f5f5',
    borderWidth: 1,
    borderColor: '#ddd',
  },
  submitButton: {
    backgroundColor: '#007AFF',
  },
  buttonDisabled: {
    opacity: 0.6,
  },
  buttonText: {
    fontSize: 16,
    fontWeight: '600',
  },
  cancelButtonText: {
    color: '#666',
  },
  submitButtonText: {
    color: '#fff',
  },
});
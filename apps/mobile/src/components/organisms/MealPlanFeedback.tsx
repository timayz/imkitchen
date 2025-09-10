import React, { useState } from 'react';
import {
  View,
  Text,
  StyleSheet,
  TouchableOpacity,
  Modal,
  TextInput,
  Alert,
  ScrollView,
} from 'react-native';

interface MealPlanFeedbackProps {
  visible: boolean;
  mealPlanId: string;
  onClose: () => void;
  onSubmit: (feedback: MealPlanFeedbackData) => void;
}

export interface MealPlanFeedbackData {
  mealPlanId: string;
  overallRating: number;
  varietyRating: number;
  difficultyRating: number;
  timelinessRating: number;
  improvementSuggestions: string[];
  freeformFeedback: string;
  wouldRecommend: boolean;
  generationTime: number;
  isHelpful: boolean;
}

interface RatingProps {
  title: string;
  subtitle: string;
  rating: number;
  onRatingChange: (rating: number) => void;
  emoji?: string[];
}

const RatingComponent: React.FC<RatingProps> = ({
  title,
  subtitle,
  rating,
  onRatingChange,
  emoji = ['😞', '😐', '🙂', '😊', '🤩'],
}) => {
  return (
    <View style={styles.ratingContainer}>
      <View style={styles.ratingHeader}>
        <Text style={styles.ratingTitle}>{title}</Text>
        <Text style={styles.ratingSubtitle}>{subtitle}</Text>
      </View>
      <View style={styles.ratingStars}>
        {[1, 2, 3, 4, 5].map((star) => (
          <TouchableOpacity
            key={star}
            onPress={() => onRatingChange(star)}
            style={styles.starButton}
          >
            <Text style={styles.ratingEmoji}>
              {star <= rating ? emoji[star - 1] : '⚪'}
            </Text>
          </TouchableOpacity>
        ))}
      </View>
    </View>
  );
};

const MealPlanFeedback: React.FC<MealPlanFeedbackProps> = ({
  visible,
  mealPlanId,
  onClose,
  onSubmit,
}) => {
  const [overallRating, setOverallRating] = useState(0);
  const [varietyRating, setVarietyRating] = useState(0);
  const [difficultyRating, setDifficultyRating] = useState(0);
  const [timelinessRating, setTimelinessRating] = useState(0);
  const [improvementSuggestions, setImprovementSuggestions] = useState<string[]>([]);
  const [freeformFeedback, setFreeformFeedback] = useState('');
  const [wouldRecommend, setWouldRecommend] = useState<boolean | null>(null);
  const [isHelpful, setIsHelpful] = useState<boolean | null>(null);

  const suggestionOptions = [
    'More variety in cuisines',
    'Better difficulty balancing',
    'Faster generation time',
    'More dietary restriction options',
    'Better ingredient substitutions',
    'Improved prep time estimates',
    'More breakfast options',
    'Better weekend meal suggestions',
    'Seasonal recipe recommendations',
    'Family-friendly options',
  ];

  const handleSuggestionToggle = (suggestion: string) => {
    if (improvementSuggestions.includes(suggestion)) {
      setImprovementSuggestions(prev => prev.filter(s => s !== suggestion));
    } else {
      setImprovementSuggestions(prev => [...prev, suggestion]);
    }
  };

  const handleSubmit = () => {
    // Validate required fields
    if (overallRating === 0) {
      Alert.alert('Missing Rating', 'Please provide an overall rating before submitting.');
      return;
    }

    if (wouldRecommend === null || isHelpful === null) {
      Alert.alert('Missing Information', 'Please answer all questions before submitting.');
      return;
    }

    const feedbackData: MealPlanFeedbackData = {
      mealPlanId,
      overallRating,
      varietyRating,
      difficultyRating,
      timelinessRating,
      improvementSuggestions,
      freeformFeedback: freeformFeedback.trim(),
      wouldRecommend: wouldRecommend!,
      generationTime: Date.now(),
      isHelpful: isHelpful!,
    };

    onSubmit(feedbackData);
    
    // Reset form
    setOverallRating(0);
    setVarietyRating(0);
    setDifficultyRating(0);
    setTimelinessRating(0);
    setImprovementSuggestions([]);
    setFreeformFeedback('');
    setWouldRecommend(null);
    setIsHelpful(null);
  };

  const isFormValid = overallRating > 0 && wouldRecommend !== null && isHelpful !== null;

  return (
    <Modal
      visible={visible}
      animationType="slide"
      presentationStyle="pageSheet"
    >
      <View style={styles.container}>
        {/* Header */}
        <View style={styles.header}>
          <TouchableOpacity onPress={onClose} style={styles.closeButton}>
            <Text style={styles.closeButtonText}>✕</Text>
          </TouchableOpacity>
          <Text style={styles.headerTitle}>Share Your Feedback</Text>
          <View style={styles.headerSpacer} />
        </View>

        <ScrollView style={styles.content} showsVerticalScrollIndicator={false}>
          {/* Introduction */}
          <View style={styles.introContainer}>
            <Text style={styles.introTitle}>How was your meal plan?</Text>
            <Text style={styles.introSubtitle}>
              Your feedback helps us improve the Fill My Week experience for everyone.
            </Text>
          </View>

          {/* Overall Rating */}
          <RatingComponent
            title="Overall Experience"
            subtitle="How satisfied are you with this meal plan?"
            rating={overallRating}
            onRatingChange={setOverallRating}
          />

          {/* Specific Ratings */}
          <RatingComponent
            title="Recipe Variety"
            subtitle="Did you get a good mix of different recipes?"
            rating={varietyRating}
            onRatingChange={setVarietyRating}
            emoji={['🔄', '📚', '🎨', '🌈', '✨']}
          />

          <RatingComponent
            title="Difficulty Balance"
            subtitle="Were the recipes appropriately matched to your skill level?"
            rating={difficultyRating}
            onRatingChange={setDifficultyRating}
            emoji={['😰', '😟', '😐', '😊', '🎯']}
          />

          <RatingComponent
            title="Generation Speed"
            subtitle="How was the meal plan generation time?"
            rating={timelinessRating}
            onRatingChange={setTimelinessRating}
            emoji={['🐌', '⏳', '⏱️', '⚡', '🚀']}
          />

          {/* Binary Questions */}
          <View style={styles.questionContainer}>
            <Text style={styles.questionTitle}>Would you recommend Fill My Week to friends?</Text>
            <View style={styles.binaryOptions}>
              <TouchableOpacity
                style={[
                  styles.binaryButton,
                  wouldRecommend === true && styles.binaryButtonSelected,
                ]}
                onPress={() => setWouldRecommend(true)}
              >
                <Text style={[
                  styles.binaryButtonText,
                  wouldRecommend === true && styles.binaryButtonTextSelected,
                ]}>
                  👍 Yes
                </Text>
              </TouchableOpacity>
              <TouchableOpacity
                style={[
                  styles.binaryButton,
                  wouldRecommend === false && styles.binaryButtonSelected,
                ]}
                onPress={() => setWouldRecommend(false)}
              >
                <Text style={[
                  styles.binaryButtonText,
                  wouldRecommend === false && styles.binaryButtonTextSelected,
                ]}>
                  👎 No
                </Text>
              </TouchableOpacity>
            </View>
          </View>

          <View style={styles.questionContainer}>
            <Text style={styles.questionTitle}>Did this meal plan save you time?</Text>
            <View style={styles.binaryOptions}>
              <TouchableOpacity
                style={[
                  styles.binaryButton,
                  isHelpful === true && styles.binaryButtonSelected,
                ]}
                onPress={() => setIsHelpful(true)}
              >
                <Text style={[
                  styles.binaryButtonText,
                  isHelpful === true && styles.binaryButtonTextSelected,
                ]}>
                  ⏰ Yes, saved time
                </Text>
              </TouchableOpacity>
              <TouchableOpacity
                style={[
                  styles.binaryButton,
                  isHelpful === false && styles.binaryButtonSelected,
                ]}
                onPress={() => setIsHelpful(false)}
              >
                <Text style={[
                  styles.binaryButtonText,
                  isHelpful === false && styles.binaryButtonTextSelected,
                ]}>
                  🤔 Not really
                </Text>
              </TouchableOpacity>
            </View>
          </View>

          {/* Improvement Suggestions */}
          <View style={styles.suggestionsContainer}>
            <Text style={styles.suggestionsTitle}>What could we improve?</Text>
            <Text style={styles.suggestionsSubtitle}>Select all that apply:</Text>
            <View style={styles.suggestionsList}>
              {suggestionOptions.map((suggestion) => (
                <TouchableOpacity
                  key={suggestion}
                  style={[
                    styles.suggestionChip,
                    improvementSuggestions.includes(suggestion) && styles.suggestionChipSelected,
                  ]}
                  onPress={() => handleSuggestionToggle(suggestion)}
                >
                  <Text style={[
                    styles.suggestionChipText,
                    improvementSuggestions.includes(suggestion) && styles.suggestionChipTextSelected,
                  ]}>
                    {suggestion}
                  </Text>
                </TouchableOpacity>
              ))}
            </View>
          </View>

          {/* Freeform Feedback */}
          <View style={styles.freeformContainer}>
            <Text style={styles.freeformTitle}>Additional Comments</Text>
            <Text style={styles.freeformSubtitle}>
              Tell us more about your experience or suggest specific improvements:
            </Text>
            <TextInput
              style={styles.freeformInput}
              multiline
              numberOfLines={4}
              placeholder="Share your thoughts, suggestions, or specific feedback..."
              value={freeformFeedback}
              onChangeText={setFreeformFeedback}
              textAlignVertical="top"
            />
          </View>

          {/* Submit Button */}
          <TouchableOpacity
            style={[styles.submitButton, !isFormValid && styles.submitButtonDisabled]}
            onPress={handleSubmit}
            disabled={!isFormValid}
          >
            <Text style={[styles.submitButtonText, !isFormValid && styles.submitButtonTextDisabled]}>
              Submit Feedback
            </Text>
          </TouchableOpacity>

          {/* Privacy Note */}
          <Text style={styles.privacyNote}>
            Your feedback is anonymous and helps us improve the Fill My Week feature.
          </Text>

          <View style={{ height: 20 }} />
        </ScrollView>
      </View>
    </Modal>
  );
};

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#FFFFFF',
  },
  header: {
    flexDirection: 'row',
    alignItems: 'center',
    paddingHorizontal: 16,
    paddingTop: 60,
    paddingBottom: 16,
    borderBottomWidth: 1,
    borderBottomColor: '#E0E0E0',
    backgroundColor: '#FAFAFA',
  },
  closeButton: {
    width: 32,
    height: 32,
    borderRadius: 16,
    backgroundColor: '#F0F0F0',
    justifyContent: 'center',
    alignItems: 'center',
  },
  closeButtonText: {
    fontSize: 16,
    fontWeight: '600',
    color: '#666666',
  },
  headerTitle: {
    flex: 1,
    fontSize: 18,
    fontWeight: '700',
    textAlign: 'center',
    color: '#333333',
  },
  headerSpacer: {
    width: 32,
  },
  content: {
    flex: 1,
    paddingHorizontal: 16,
  },
  introContainer: {
    paddingVertical: 24,
    alignItems: 'center',
  },
  introTitle: {
    fontSize: 24,
    fontWeight: '700',
    color: '#333333',
    textAlign: 'center',
    marginBottom: 8,
  },
  introSubtitle: {
    fontSize: 16,
    color: '#666666',
    textAlign: 'center',
    lineHeight: 24,
  },
  ratingContainer: {
    marginBottom: 24,
    paddingBottom: 24,
    borderBottomWidth: 1,
    borderBottomColor: '#F0F0F0',
  },
  ratingHeader: {
    marginBottom: 12,
  },
  ratingTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#333333',
    marginBottom: 4,
  },
  ratingSubtitle: {
    fontSize: 14,
    color: '#666666',
  },
  ratingStars: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    maxWidth: 200,
  },
  starButton: {
    padding: 8,
  },
  ratingEmoji: {
    fontSize: 28,
  },
  questionContainer: {
    marginBottom: 24,
    paddingBottom: 24,
    borderBottomWidth: 1,
    borderBottomColor: '#F0F0F0',
  },
  questionTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#333333',
    marginBottom: 12,
  },
  binaryOptions: {
    flexDirection: 'row',
    gap: 12,
  },
  binaryButton: {
    flex: 1,
    paddingVertical: 12,
    paddingHorizontal: 16,
    borderRadius: 8,
    borderWidth: 1,
    borderColor: '#E0E0E0',
    backgroundColor: '#FFFFFF',
    alignItems: 'center',
  },
  binaryButtonSelected: {
    backgroundColor: '#E8F5E8',
    borderColor: '#4CAF50',
  },
  binaryButtonText: {
    fontSize: 16,
    fontWeight: '600',
    color: '#666666',
  },
  binaryButtonTextSelected: {
    color: '#4CAF50',
  },
  suggestionsContainer: {
    marginBottom: 24,
    paddingBottom: 24,
    borderBottomWidth: 1,
    borderBottomColor: '#F0F0F0',
  },
  suggestionsTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#333333',
    marginBottom: 4,
  },
  suggestionsSubtitle: {
    fontSize: 14,
    color: '#666666',
    marginBottom: 12,
  },
  suggestionsList: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: 8,
  },
  suggestionChip: {
    paddingVertical: 8,
    paddingHorizontal: 12,
    borderRadius: 16,
    borderWidth: 1,
    borderColor: '#E0E0E0',
    backgroundColor: '#FFFFFF',
  },
  suggestionChipSelected: {
    backgroundColor: '#E8F5E8',
    borderColor: '#4CAF50',
  },
  suggestionChipText: {
    fontSize: 14,
    color: '#666666',
  },
  suggestionChipTextSelected: {
    color: '#4CAF50',
    fontWeight: '500',
  },
  freeformContainer: {
    marginBottom: 32,
  },
  freeformTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#333333',
    marginBottom: 4,
  },
  freeformSubtitle: {
    fontSize: 14,
    color: '#666666',
    marginBottom: 12,
  },
  freeformInput: {
    borderWidth: 1,
    borderColor: '#E0E0E0',
    borderRadius: 8,
    padding: 12,
    fontSize: 16,
    minHeight: 100,
    backgroundColor: '#FAFAFA',
  },
  submitButton: {
    backgroundColor: '#4CAF50',
    paddingVertical: 16,
    borderRadius: 8,
    alignItems: 'center',
    marginBottom: 16,
  },
  submitButtonDisabled: {
    backgroundColor: '#CCCCCC',
  },
  submitButtonText: {
    fontSize: 18,
    fontWeight: '700',
    color: '#FFFFFF',
  },
  submitButtonTextDisabled: {
    color: '#888888',
  },
  privacyNote: {
    fontSize: 12,
    color: '#999999',
    textAlign: 'center',
    fontStyle: 'italic',
  },
});

export default MealPlanFeedback;
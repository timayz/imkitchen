package services

import (
	"regexp"
	"strings"
	"time"
)

// ContentModerationService handles review content filtering and moderation
type ContentModerationService struct {
	profanityWords   []string
	spamPatterns     []*regexp.Regexp
	maxContentLength int
}

// ModerationResult represents the result of content moderation
type ModerationResult struct {
	IsApproved       bool   `json:"isApproved"`
	ModerationStatus string `json:"moderationStatus"`
	FlaggedReason    string `json:"flaggedReason,omitempty"`
	SanitizedContent string `json:"sanitizedContent"`
}

// NewContentModerationService creates a new content moderation service
func NewContentModerationService() *ContentModerationService {
	return &ContentModerationService{
		profanityWords: []string{
			// Basic profanity filter - in production, use comprehensive service
			"damn", "hell", "crap", "stupid", "idiot", "hate", "suck", 
			"terrible", "awful", "disgusting", "shit", "fuck", "ass",
		},
		spamPatterns: []*regexp.Regexp{
			regexp.MustCompile(`(.)\1{4,}`),                    // Repeated characters
			regexp.MustCompile(`http[s]?://`),                  // URLs
			regexp.MustCompile(`\b(buy|sell|cheap|free|money|cash|prize|winner|click)\b`), // Spam keywords
			regexp.MustCompile(`(.{1,10})\1{3,}`),             // Repeated patterns
		},
		maxContentLength: 500,
	}
}

// ModerateReviewText moderates review text content
func (c *ContentModerationService) ModerateReviewText(content string) *ModerationResult {
	if content == "" {
		return &ModerationResult{
			IsApproved:       true,
			ModerationStatus: "approved",
			SanitizedContent: "",
		}
	}

	// Check length
	if len(content) > c.maxContentLength {
		return &ModerationResult{
			IsApproved:       false,
			ModerationStatus: "rejected",
			FlaggedReason:    "Content exceeds maximum length",
			SanitizedContent: c.sanitizeContent(content),
		}
	}

	// Sanitize content first
	sanitizedContent := c.sanitizeContent(content)
	
	// Check for profanity
	if c.containsProfanity(sanitizedContent) {
		return &ModerationResult{
			IsApproved:       false,
			ModerationStatus: "pending",
			FlaggedReason:    "Contains inappropriate language",
			SanitizedContent: sanitizedContent,
		}
	}

	// Check for spam patterns
	if c.isLikelySpam(sanitizedContent) {
		return &ModerationResult{
			IsApproved:       false,
			ModerationStatus: "pending",
			FlaggedReason:    "Content appears to be spam",
			SanitizedContent: sanitizedContent,
		}
	}

	// Check for suspicious patterns
	if c.hasSuspiciousPatterns(sanitizedContent) {
		return &ModerationResult{
			IsApproved:       false,
			ModerationStatus: "pending",
			FlaggedReason:    "Content flagged for manual review",
			SanitizedContent: sanitizedContent,
		}
	}

	return &ModerationResult{
		IsApproved:       true,
		ModerationStatus: "approved",
		SanitizedContent: sanitizedContent,
	}
}

// BulkModerateContent moderates multiple pieces of content
func (c *ContentModerationService) BulkModerateContent(contents []string) []*ModerationResult {
	results := make([]*ModerationResult, len(contents))
	for i, content := range contents {
		results[i] = c.ModerateReviewText(content)
	}
	return results
}

// ApproveContent manually approves flagged content
func (c *ContentModerationService) ApproveContent(content string) *ModerationResult {
	return &ModerationResult{
		IsApproved:       true,
		ModerationStatus: "approved",
		SanitizedContent: c.sanitizeContent(content),
	}
}

// RejectContent manually rejects flagged content
func (c *ContentModerationService) RejectContent(content string, reason string) *ModerationResult {
	return &ModerationResult{
		IsApproved:       false,
		ModerationStatus: "rejected",
		FlaggedReason:    reason,
		SanitizedContent: c.sanitizeContent(content),
	}
}

// sanitizeContent cleans and sanitizes input content
func (c *ContentModerationService) sanitizeContent(content string) string {
	// Trim whitespace
	content = strings.TrimSpace(content)
	
	// Replace multiple spaces with single space
	spaceRegex := regexp.MustCompile(`\s+`)
	content = spaceRegex.ReplaceAllString(content, " ")
	
	// Remove HTML tags
	htmlRegex := regexp.MustCompile(`<[^>]*>`)
	content = htmlRegex.ReplaceAllString(content, "")
	
	// Remove potentially dangerous characters but keep punctuation
	dangerousRegex := regexp.MustCompile(`[^\w\s.,!?'"-]`)
	content = dangerousRegex.ReplaceAllString(content, "")
	
	// Truncate if too long
	if len(content) > c.maxContentLength {
		content = content[:c.maxContentLength]
	}
	
	return content
}

// containsProfanity checks if content contains profane words
func (c *ContentModerationService) containsProfanity(content string) bool {
	lowercaseContent := strings.ToLower(content)
	for _, word := range c.profanityWords {
		if strings.Contains(lowercaseContent, word) {
			return true
		}
	}
	return false
}

// isLikelySpam checks if content matches spam patterns
func (c *ContentModerationService) isLikelySpam(content string) bool {
	for _, pattern := range c.spamPatterns {
		if pattern.MatchString(content) {
			return true
		}
	}
	return false
}

// hasSuspiciousPatterns checks for other suspicious content patterns
func (c *ContentModerationService) hasSuspiciousPatterns(content string) bool {
	// Check for excessive caps
	if c.hasExcessiveCaps(content) {
		return true
	}
	
	// Check for repetitive content
	if c.isRepetitive(content) {
		return true
	}
	
	// Check for suspicious keywords
	suspiciousKeywords := []string{
		"contact me", "email me", "call me", "phone", "website",
		"promotion", "discount", "offer", "deal", "sale",
	}
	
	lowercaseContent := strings.ToLower(content)
	for _, keyword := range suspiciousKeywords {
		if strings.Contains(lowercaseContent, keyword) {
			return true
		}
	}
	
	return false
}

// hasExcessiveCaps checks if content has too many capital letters
func (c *ContentModerationService) hasExcessiveCaps(content string) bool {
	if len(content) < 10 {
		return false
	}
	
	capsCount := 0
	for _, char := range content {
		if char >= 'A' && char <= 'Z' {
			capsCount++
		}
	}
	
	// More than 30% caps is suspicious
	capsRatio := float64(capsCount) / float64(len(content))
	return capsRatio > 0.3
}

// isRepetitive checks if content is overly repetitive
func (c *ContentModerationService) isRepetitive(content string) bool {
	words := strings.Fields(content)
	if len(words) < 5 {
		return false
	}
	
	wordCount := make(map[string]int)
	for _, word := range words {
		wordCount[strings.ToLower(word)]++
	}
	
	// Check if any word appears too frequently
	for _, count := range wordCount {
		if count > len(words)/3 {
			return true
		}
	}
	
	return false
}

// GetModerationStats returns statistics about moderated content
func (c *ContentModerationService) GetModerationStats() map[string]interface{} {
	return map[string]interface{}{
		"service_version":     "1.0.0",
		"max_content_length":  c.maxContentLength,
		"profanity_words":     len(c.profanityWords),
		"spam_patterns":       len(c.spamPatterns),
		"last_updated":        time.Now().Format(time.RFC3339),
	}
}

// UpdateProfanityList updates the profanity word list (for admin use)
func (c *ContentModerationService) UpdateProfanityList(words []string) {
	c.profanityWords = words
}

// AddProfanityWords adds words to the profanity list
func (c *ContentModerationService) AddProfanityWords(words []string) {
	c.profanityWords = append(c.profanityWords, words...)
}

// RemoveProfanityWords removes words from the profanity list
func (c *ContentModerationService) RemoveProfanityWords(words []string) {
	for _, word := range words {
		for i, existingWord := range c.profanityWords {
			if existingWord == word {
				c.profanityWords = append(c.profanityWords[:i], c.profanityWords[i+1:]...)
				break
			}
		}
	}
}
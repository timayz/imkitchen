package repositories

import (
	"context"
	"database/sql"
	"fmt"
	"strings"
	"time"

	"github.com/lib/pq"
)

// TagSuggestion represents a tag suggestion
type TagSuggestion struct {
	Tag           string     `json:"tag"`
	Confidence    float64    `json:"confidence"`
	UsageCount    int        `json:"usage_count"`
	Category      string     `json:"category"`
	IsPopular     bool       `json:"is_popular"`
	LastUsed      *time.Time `json:"last_used,omitempty"`
	Description   string     `json:"description,omitempty"`
	RelatedTags   []string   `json:"related_tags,omitempty"`
}

// PopularTag represents a popular tag
type PopularTag struct {
	Tag         string    `json:"tag"`
	UsageCount  int       `json:"usage_count"`
	Category    string    `json:"category"`
	Confidence  float64   `json:"confidence"`
	LastUsed    time.Time `json:"last_used"`
	Description string    `json:"description,omitempty"`
}

// CommunityTag represents a community-contributed tag
type CommunityTag struct {
	Tag           string    `json:"tag"`
	VoteCount     int       `json:"vote_count"`
	UserVoted     bool      `json:"user_voted"`
	Confidence    float64   `json:"confidence"`
	AddedBy       string    `json:"added_by"`
	AddedAt       time.Time `json:"added_at"`
	Category      string    `json:"category"`
	IsVerified    bool      `json:"is_verified"`
	Description   string    `json:"description,omitempty"`
}

// TagStat represents statistics for a tag
type TagStat struct {
	Tag            string    `json:"tag"`
	UsageCount     int       `json:"usage_count"`
	Category       string    `json:"category"`
	Confidence     float64   `json:"confidence"`
	TotalUsage     int       `json:"total_usage"`
	RecentUsage    int       `json:"recent_usage"`
	TrendDirection string    `json:"trend_direction"`
	TrendScore     float64   `json:"trend_score"`
	FirstUsed      time.Time `json:"first_used"`
	LastUsed       time.Time `json:"last_used"`
	Trending       bool      `json:"trending"`
}

type TagManagementRepository struct {
	db *sql.DB
}

func NewTagManagementRepository(db *sql.DB) *TagManagementRepository {
	return &TagManagementRepository{db: db}
}

func (r *TagManagementRepository) GetTagSuggestions(ctx context.Context, query string, exclude []string, limit int) ([]TagSuggestion, error) {
	baseQuery := `
		SELECT 
			tag,
			COUNT(*) as usage_count,
			AVG(confidence) as avg_confidence,
			COALESCE(category, 'general') as category
		FROM recipe_tags rt
		LEFT JOIN tag_categories tc ON rt.tag = tc.tag
		WHERE LOWER(tag) LIKE LOWER($1)
	`
	
	args := []interface{}{"%" + query + "%"}
	argIndex := 2

	// Add exclude filter
	if len(exclude) > 0 {
		placeholders := make([]string, len(exclude))
		for i, excludeTag := range exclude {
			placeholders[i] = fmt.Sprintf("$%d", argIndex)
			args = append(args, strings.ToLower(excludeTag))
			argIndex++
		}
		baseQuery += fmt.Sprintf(" AND LOWER(tag) NOT IN (%s)", strings.Join(placeholders, ","))
	}

	baseQuery += `
		GROUP BY tag, tc.category
		ORDER BY usage_count DESC, avg_confidence DESC
		LIMIT $` + fmt.Sprintf("%d", argIndex)
	
	args = append(args, limit)

	rows, err := r.db.QueryContext(ctx, baseQuery, args...)
	if err != nil {
		return nil, fmt.Errorf("failed to query tag suggestions: %w", err)
	}
	defer rows.Close()

	var suggestions []TagSuggestion
	for rows.Next() {
		var suggestion TagSuggestion
		err := rows.Scan(
			&suggestion.Tag,
			&suggestion.UsageCount,
			&suggestion.Confidence,
			&suggestion.Category,
		)
		if err != nil {
			return nil, fmt.Errorf("failed to scan tag suggestion: %w", err)
		}
		suggestions = append(suggestions, suggestion)
	}

	return suggestions, nil
}

func (r *TagManagementRepository) GetPopularTags(ctx context.Context, limit int, categoryFilter string, since *time.Time) ([]PopularTag, error) {
	baseQuery := `
		SELECT 
			rt.tag,
			COUNT(*) as usage_count,
			COALESCE(tc.category, 'general') as category,
			COALESCE(tc.description, '') as description
		FROM recipe_tags rt
		LEFT JOIN tag_categories tc ON rt.tag = tc.tag
		WHERE 1=1
	`
	
	args := []interface{}{}
	argIndex := 1

	// Add category filter
	if categoryFilter != "" {
		baseQuery += fmt.Sprintf(" AND COALESCE(tc.category, 'general') = $%d", argIndex)
		args = append(args, categoryFilter)
		argIndex++
	}

	// Add time filter
	if since != nil {
		baseQuery += fmt.Sprintf(" AND rt.created_at >= $%d", argIndex)
		args = append(args, *since)
		argIndex++
	}

	baseQuery += `
		GROUP BY rt.tag, tc.category, tc.description
		ORDER BY usage_count DESC
		LIMIT $` + fmt.Sprintf("%d", argIndex)
	
	args = append(args, limit)

	rows, err := r.db.QueryContext(ctx, baseQuery, args...)
	if err != nil {
		return nil, fmt.Errorf("failed to query popular tags: %w", err)
	}
	defer rows.Close()

	var tags []PopularTag
	for rows.Next() {
		var tag PopularTag
		err := rows.Scan(
			&tag.Tag,
			&tag.UsageCount,
			&tag.Category,
			&tag.Description,
		)
		if err != nil {
			return nil, fmt.Errorf("failed to scan popular tag: %w", err)
		}
		tags = append(tags, tag)
	}

	return tags, nil
}

func (r *TagManagementRepository) IsTagTrending(ctx context.Context, tag string, since time.Time) (bool, error) {
	query := `
		SELECT 
			COUNT(CASE WHEN created_at >= $2 THEN 1 END) as recent_count,
			COUNT(*) as total_count
		FROM recipe_tags 
		WHERE tag = $1
	`

	var recentCount, totalCount int
	err := r.db.QueryRowContext(ctx, query, tag, since).Scan(&recentCount, &totalCount)
	if err != nil {
		return false, fmt.Errorf("failed to check trending status: %w", err)
	}

	// Consider trending if recent usage is > 50% of total usage and total > 10
	return totalCount > 10 && float64(recentCount)/float64(totalCount) > 0.5, nil
}

func (r *TagManagementRepository) AddTagsToRecipe(ctx context.Context, recipeID string, tags []string) ([]string, error) {
	tx, err := r.db.BeginTx(ctx, nil)
	if err != nil {
		return nil, fmt.Errorf("failed to begin transaction: %w", err)
	}
	defer tx.Rollback()

	// Get current tags
	currentTags, err := r.getRecipeTagsInTx(tx, recipeID)
	if err != nil {
		return nil, fmt.Errorf("failed to get current tags: %w", err)
	}

	// Create a set of current tags for quick lookup
	currentTagSet := make(map[string]bool)
	for _, tag := range currentTags {
		currentTagSet[tag] = true
	}

	// Add new tags
	var newTags []string
	for _, tag := range tags {
		if !currentTagSet[tag] {
			newTags = append(newTags, tag)
			currentTags = append(currentTags, tag)
		}
	}

	// Insert new tags
	if len(newTags) > 0 {
		err = r.insertRecipeTagsInTx(tx, recipeID, newTags)
		if err != nil {
			return nil, fmt.Errorf("failed to insert new tags: %w", err)
		}
	}

	// Update recipe user_tags array
	err = r.updateRecipeUserTagsInTx(tx, recipeID, currentTags)
	if err != nil {
		return nil, fmt.Errorf("failed to update recipe user_tags: %w", err)
	}

	if err = tx.Commit(); err != nil {
		return nil, fmt.Errorf("failed to commit transaction: %w", err)
	}

	return currentTags, nil
}

func (r *TagManagementRepository) RemoveTagsFromRecipe(ctx context.Context, recipeID string, tags []string) ([]string, error) {
	tx, err := r.db.BeginTx(ctx, nil)
	if err != nil {
		return nil, fmt.Errorf("failed to begin transaction: %w", err)
	}
	defer tx.Rollback()

	// Get current tags
	currentTags, err := r.getRecipeTagsInTx(tx, recipeID)
	if err != nil {
		return nil, fmt.Errorf("failed to get current tags: %w", err)
	}

	// Create a set of tags to remove
	removeTagSet := make(map[string]bool)
	for _, tag := range tags {
		removeTagSet[tag] = true
	}

	// Filter out tags to remove
	var updatedTags []string
	for _, tag := range currentTags {
		if !removeTagSet[tag] {
			updatedTags = append(updatedTags, tag)
		}
	}

	// Delete tags from recipe_tags table
	if len(tags) > 0 {
		err = r.deleteRecipeTagsInTx(tx, recipeID, tags)
		if err != nil {
			return nil, fmt.Errorf("failed to delete tags: %w", err)
		}
	}

	// Update recipe user_tags array
	err = r.updateRecipeUserTagsInTx(tx, recipeID, updatedTags)
	if err != nil {
		return nil, fmt.Errorf("failed to update recipe user_tags: %w", err)
	}

	if err = tx.Commit(); err != nil {
		return nil, fmt.Errorf("failed to commit transaction: %w", err)
	}

	return updatedTags, nil
}

func (r *TagManagementRepository) ReplaceRecipeTags(ctx context.Context, recipeID string, tags []string) ([]string, error) {
	tx, err := r.db.BeginTx(ctx, nil)
	if err != nil {
		return nil, fmt.Errorf("failed to begin transaction: %w", err)
	}
	defer tx.Rollback()

	// Delete all existing tags
	_, err = tx.ExecContext(ctx, "DELETE FROM recipe_tags WHERE recipe_id = $1", recipeID)
	if err != nil {
		return nil, fmt.Errorf("failed to delete existing tags: %w", err)
	}

	// Insert new tags
	if len(tags) > 0 {
		err = r.insertRecipeTagsInTx(tx, recipeID, tags)
		if err != nil {
			return nil, fmt.Errorf("failed to insert new tags: %w", err)
		}
	}

	// Update recipe user_tags array
	err = r.updateRecipeUserTagsInTx(tx, recipeID, tags)
	if err != nil {
		return nil, fmt.Errorf("failed to update recipe user_tags: %w", err)
	}

	if err = tx.Commit(); err != nil {
		return nil, fmt.Errorf("failed to commit transaction: %w", err)
	}

	return tags, nil
}

func (r *TagManagementRepository) GetRecipeUserTags(ctx context.Context, recipeID string) ([]string, error) {
	query := "SELECT COALESCE(user_tags, '{}') FROM recipes WHERE id = $1"
	
	var tags pq.StringArray
	err := r.db.QueryRowContext(ctx, query, recipeID).Scan(&tags)
	if err != nil {
		return nil, fmt.Errorf("failed to get recipe user tags: %w", err)
	}

	return []string(tags), nil
}

func (r *TagManagementRepository) GetRecipeCommunityTags(ctx context.Context, recipeID, userID string) ([]CommunityTag, error) {
	query := `
		SELECT 
			ct.tag,
			COALESCE(SUM(CASE WHEN ctv.vote_type = 'upvote' THEN 1 WHEN ctv.vote_type = 'downvote' THEN -1 ELSE 0 END), 0) as vote_count,
			EXISTS(SELECT 1 FROM community_tag_votes WHERE tag = ct.tag AND recipe_id = ct.recipe_id AND user_id = $2) as user_voted,
			ct.confidence
		FROM community_tags ct
		LEFT JOIN community_tag_votes ctv ON ct.tag = ctv.tag AND ct.recipe_id = ctv.recipe_id
		WHERE ct.recipe_id = $1
		GROUP BY ct.tag, ct.confidence
		ORDER BY vote_count DESC, ct.confidence DESC
	`

	rows, err := r.db.QueryContext(ctx, query, recipeID, userID)
	if err != nil {
		return nil, fmt.Errorf("failed to query community tags: %w", err)
	}
	defer rows.Close()

	var tags []CommunityTag
	for rows.Next() {
		var tag CommunityTag
		err := rows.Scan(
			&tag.Tag,
			&tag.VoteCount,
			&tag.UserVoted,
			&tag.Confidence,
		)
		if err != nil {
			return nil, fmt.Errorf("failed to scan community tag: %w", err)
		}
		tags = append(tags, tag)
	}

	return tags, nil
}

func (r *TagManagementRepository) GetTagStats(ctx context.Context, tags []string) (map[string]TagStat, error) {
	if len(tags) == 0 {
		return make(map[string]TagStat), nil
	}

	placeholders := make([]string, len(tags))
	args := make([]interface{}, len(tags))
	for i, tag := range tags {
		placeholders[i] = fmt.Sprintf("$%d", i+1)
		args[i] = tag
	}

	query := fmt.Sprintf(`
		SELECT 
			rt.tag,
			COUNT(*) as usage_count,
			COALESCE(tc.category, 'general') as category,
			AVG(rt.confidence) as avg_confidence
		FROM recipe_tags rt
		LEFT JOIN tag_categories tc ON rt.tag = tc.tag
		WHERE rt.tag IN (%s)
		GROUP BY rt.tag, tc.category
	`, strings.Join(placeholders, ","))

	rows, err := r.db.QueryContext(ctx, query, args...)
	if err != nil {
		return nil, fmt.Errorf("failed to query tag stats: %w", err)
	}
	defer rows.Close()

	stats := make(map[string]TagStat)
	for rows.Next() {
		var tag string
		var stat TagStat
		err := rows.Scan(
			&tag,
			&stat.UsageCount,
			&stat.Category,
			&stat.Confidence,
		)
		if err != nil {
			return nil, fmt.Errorf("failed to scan tag stat: %w", err)
		}

		// Check if trending
		trending, _ := r.IsTagTrending(ctx, tag, time.Now().AddDate(0, 0, -7))
		stat.Trending = trending

		stats[tag] = stat
	}

	return stats, nil
}

func (r *TagManagementRepository) UpvoteTag(ctx context.Context, recipeID, userID, tag string) (int, bool, error) {
	return r.voteOnTag(ctx, recipeID, userID, tag, "upvote")
}

func (r *TagManagementRepository) DownvoteTag(ctx context.Context, recipeID, userID, tag string) (int, bool, error) {
	return r.voteOnTag(ctx, recipeID, userID, tag, "downvote")
}

func (r *TagManagementRepository) RemoveVote(ctx context.Context, recipeID, userID, tag string) (int, bool, error) {
	tx, err := r.db.BeginTx(ctx, nil)
	if err != nil {
		return 0, false, fmt.Errorf("failed to begin transaction: %w", err)
	}
	defer tx.Rollback()

	// Remove the vote
	_, err = tx.ExecContext(ctx, 
		"DELETE FROM community_tag_votes WHERE recipe_id = $1 AND user_id = $2 AND tag = $3",
		recipeID, userID, tag,
	)
	if err != nil {
		return 0, false, fmt.Errorf("failed to remove vote: %w", err)
	}

	// Get updated vote count
	voteCount, err := r.getTagVoteCountInTx(tx, recipeID, tag)
	if err != nil {
		return 0, false, fmt.Errorf("failed to get vote count: %w", err)
	}

	if err = tx.Commit(); err != nil {
		return 0, false, fmt.Errorf("failed to commit transaction: %w", err)
	}

	return voteCount, false, nil
}

func (r *TagManagementRepository) UpdateTagUsageStats(ctx context.Context, tags []string) error {
	if len(tags) == 0 {
		return nil
	}

	// Update global tag statistics
	for _, tag := range tags {
		_, err := r.db.ExecContext(ctx, `
			INSERT INTO tag_usage_stats (tag, usage_count, last_used) 
			VALUES ($1, 1, NOW())
			ON CONFLICT (tag) 
			DO UPDATE SET 
				usage_count = tag_usage_stats.usage_count + 1,
				last_used = NOW()
		`, tag)
		if err != nil {
			return fmt.Errorf("failed to update tag usage stats for %s: %w", tag, err)
		}
	}

	return nil
}

func (r *TagManagementRepository) UpdateTagConfidence(ctx context.Context, recipeID, tag string) error {
	// Calculate confidence based on vote ratio
	query := `
		UPDATE community_tags 
		SET confidence = GREATEST(0.0, LEAST(1.0, 
			0.5 + (
				COALESCE((
					SELECT SUM(CASE WHEN vote_type = 'upvote' THEN 1 WHEN vote_type = 'downvote' THEN -1 ELSE 0 END)
					FROM community_tag_votes 
					WHERE recipe_id = $1 AND tag = $2
				), 0) * 0.1
			)
		))
		WHERE recipe_id = $1 AND tag = $2
	`

	_, err := r.db.ExecContext(ctx, query, recipeID, tag)
	if err != nil {
		return fmt.Errorf("failed to update tag confidence: %w", err)
	}

	return nil
}

// Helper methods

func (r *TagManagementRepository) voteOnTag(ctx context.Context, recipeID, userID, tag, voteType string) (int, bool, error) {
	tx, err := r.db.BeginTx(ctx, nil)
	if err != nil {
		return 0, false, fmt.Errorf("failed to begin transaction: %w", err)
	}
	defer tx.Rollback()

	// Ensure the community tag exists
	err = r.ensureCommunityTagExistsInTx(tx, recipeID, tag)
	if err != nil {
		return 0, false, fmt.Errorf("failed to ensure tag exists: %w", err)
	}

	// Insert or update the vote
	_, err = tx.ExecContext(ctx, `
		INSERT INTO community_tag_votes (recipe_id, user_id, tag, vote_type, created_at)
		VALUES ($1, $2, $3, $4, NOW())
		ON CONFLICT (recipe_id, user_id, tag)
		DO UPDATE SET vote_type = $4, updated_at = NOW()
	`, recipeID, userID, tag, voteType)
	if err != nil {
		return 0, false, fmt.Errorf("failed to insert/update vote: %w", err)
	}

	// Get updated vote count
	voteCount, err := r.getTagVoteCountInTx(tx, recipeID, tag)
	if err != nil {
		return 0, false, fmt.Errorf("failed to get vote count: %w", err)
	}

	if err = tx.Commit(); err != nil {
		return 0, false, fmt.Errorf("failed to commit transaction: %w", err)
	}

	return voteCount, true, nil
}

func (r *TagManagementRepository) getRecipeTagsInTx(tx *sql.Tx, recipeID string) ([]string, error) {
	query := "SELECT COALESCE(user_tags, '{}') FROM recipes WHERE id = $1"
	
	var tags pq.StringArray
	err := tx.QueryRow(query, recipeID).Scan(&tags)
	if err != nil {
		return nil, fmt.Errorf("failed to get recipe tags: %w", err)
	}

	return []string(tags), nil
}

func (r *TagManagementRepository) insertRecipeTagsInTx(tx *sql.Tx, recipeID string, tags []string) error {
	for _, tag := range tags {
		_, err := tx.Exec(`
			INSERT INTO recipe_tags (recipe_id, tag, confidence, created_at)
			VALUES ($1, $2, 1.0, NOW())
		`, recipeID, tag)
		if err != nil {
			return fmt.Errorf("failed to insert tag %s: %w", tag, err)
		}
	}
	return nil
}

func (r *TagManagementRepository) deleteRecipeTagsInTx(tx *sql.Tx, recipeID string, tags []string) error {
	placeholders := make([]string, len(tags))
	args := []interface{}{recipeID}
	for i, tag := range tags {
		placeholders[i] = fmt.Sprintf("$%d", i+2)
		args = append(args, tag)
	}

	query := fmt.Sprintf("DELETE FROM recipe_tags WHERE recipe_id = $1 AND tag IN (%s)", strings.Join(placeholders, ","))
	
	_, err := tx.Exec(query, args...)
	if err != nil {
		return fmt.Errorf("failed to delete tags: %w", err)
	}

	return nil
}

func (r *TagManagementRepository) updateRecipeUserTagsInTx(tx *sql.Tx, recipeID string, tags []string) error {
	_, err := tx.Exec("UPDATE recipes SET user_tags = $1 WHERE id = $2", pq.Array(tags), recipeID)
	if err != nil {
		return fmt.Errorf("failed to update recipe user_tags: %w", err)
	}
	return nil
}

func (r *TagManagementRepository) ensureCommunityTagExistsInTx(tx *sql.Tx, recipeID, tag string) error {
	_, err := tx.Exec(`
		INSERT INTO community_tags (recipe_id, tag, confidence, created_at)
		VALUES ($1, $2, 0.5, NOW())
		ON CONFLICT (recipe_id, tag) DO NOTHING
	`, recipeID, tag)
	if err != nil {
		return fmt.Errorf("failed to ensure community tag exists: %w", err)
	}
	return nil
}

func (r *TagManagementRepository) getTagVoteCountInTx(tx *sql.Tx, recipeID, tag string) (int, error) {
	var voteCount int
	err := tx.QueryRow(`
		SELECT COALESCE(SUM(CASE WHEN vote_type = 'upvote' THEN 1 WHEN vote_type = 'downvote' THEN -1 ELSE 0 END), 0)
		FROM community_tag_votes 
		WHERE recipe_id = $1 AND tag = $2
	`, recipeID, tag).Scan(&voteCount)
	if err != nil {
		return 0, fmt.Errorf("failed to get vote count: %w", err)
	}
	return voteCount, nil
}
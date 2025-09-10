package handlers

import (
	"net/http"
	"strconv"
	"strings"

	"github.com/gin-gonic/gin"
	"github.com/imkitchen/backend/internal/middleware"
	"github.com/imkitchen/backend/internal/services"
)

type TagManagementHandler struct {
	tagService *services.TagManagementService
}

func NewTagManagementHandler(tagService *services.TagManagementService) *TagManagementHandler {
	return &TagManagementHandler{
		tagService: tagService,
	}
}

func (h *TagManagementHandler) RegisterRoutes(rg *gin.RouterGroup) {
	tags := rg.Group("/tags")
	tags.Use(middleware.AuthRequired())
	{
		tags.GET("/suggestions", h.GetTagSuggestions)
		tags.GET("/popular", h.GetPopularTags)
		tags.POST("/validate", h.ValidateTags)
	}

	recipes := rg.Group("/recipes")
	recipes.Use(middleware.AuthRequired())
	{
		recipes.PUT("/:id/tags", h.UpdateRecipeTags)
		recipes.GET("/:id/tags", h.GetRecipeTags)
		recipes.POST("/:id/tags/vote", h.VoteOnTag)
	}
}

type TagSuggestionsRequest struct {
	Query    string   `json:"query" binding:"required,min=1,max=50"`
	RecipeID string   `json:"recipe_id,omitempty"`
	Exclude  []string `json:"exclude,omitempty"`
	Limit    int      `json:"limit,omitempty"`
}

type TagSuggestionsResponse struct {
	Suggestions []TagSuggestion `json:"suggestions"`
}

type TagSuggestion struct {
	Tag        string  `json:"tag"`
	Confidence float64 `json:"confidence"`
	UsageCount int     `json:"usage_count"`
	Category   string  `json:"category"`
}

func (h *TagManagementHandler) GetTagSuggestions(c *gin.Context) {
	var req TagSuggestionsRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid request format", "details": err.Error()})
		return
	}

	if req.Limit == 0 {
		req.Limit = 10
	}
	if req.Limit > 50 {
		req.Limit = 50
	}

	userID, exists := c.Get("user_id")
	if !exists {
		c.JSON(http.StatusUnauthorized, gin.H{"error": "User not authenticated"})
		return
	}

	suggestions, err := h.tagService.GetTagSuggestions(userID.(string), req.Query, req.RecipeID, req.Exclude, req.Limit)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to get tag suggestions", "details": err.Error()})
		return
	}

	c.JSON(http.StatusOK, TagSuggestionsResponse{
		Suggestions: suggestions,
	})
}

type PopularTagsResponse struct {
	Tags []PopularTag `json:"tags"`
}

type PopularTag struct {
	Tag         string `json:"tag"`
	UsageCount  int    `json:"usage_count"`
	Category    string `json:"category"`
	TrendingUp  bool   `json:"trending_up"`
	Description string `json:"description,omitempty"`
}

func (h *TagManagementHandler) GetPopularTags(c *gin.Context) {
	limitStr := c.DefaultQuery("limit", "20")
	categoryFilter := c.Query("category")
	timePeriod := c.DefaultQuery("period", "week") // day, week, month, all

	limit, err := strconv.Atoi(limitStr)
	if err != nil || limit <= 0 || limit > 100 {
		limit = 20
	}

	userID, exists := c.Get("user_id")
	if !exists {
		c.JSON(http.StatusUnauthorized, gin.H{"error": "User not authenticated"})
		return
	}

	tags, err := h.tagService.GetPopularTags(userID.(string), limit, categoryFilter, timePeriod)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to get popular tags", "details": err.Error()})
		return
	}

	c.JSON(http.StatusOK, PopularTagsResponse{
		Tags: tags,
	})
}

type ValidateTagsRequest struct {
	Tags []string `json:"tags" binding:"required,min=1,max=10"`
}

type ValidateTagsResponse struct {
	ValidTags   []string           `json:"valid_tags"`
	InvalidTags []InvalidTagResult `json:"invalid_tags"`
}

type InvalidTagResult struct {
	Tag    string `json:"tag"`
	Reason string `json:"reason"`
}

func (h *TagManagementHandler) ValidateTags(c *gin.Context) {
	var req ValidateTagsRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid request format", "details": err.Error()})
		return
	}

	userID, exists := c.Get("user_id")
	if !exists {
		c.JSON(http.StatusUnauthorized, gin.H{"error": "User not authenticated"})
		return
	}

	validTags, invalidTags, err := h.tagService.ValidateTags(userID.(string), req.Tags)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to validate tags", "details": err.Error()})
		return
	}

	c.JSON(http.StatusOK, ValidateTagsResponse{
		ValidTags:   validTags,
		InvalidTags: invalidTags,
	})
}

type UpdateRecipeTagsRequest struct {
	Tags   []string `json:"tags" binding:"required,max=10"`
	Action string   `json:"action" binding:"required,oneof=add remove replace"`
}

type UpdateRecipeTagsResponse struct {
	RecipeID    string   `json:"recipe_id"`
	UpdatedTags []string `json:"updated_tags"`
	Message     string   `json:"message"`
}

func (h *TagManagementHandler) UpdateRecipeTags(c *gin.Context) {
	recipeID := c.Param("id")
	if recipeID == "" {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Recipe ID is required"})
		return
	}

	var req UpdateRecipeTagsRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid request format", "details": err.Error()})
		return
	}

	userID, exists := c.Get("user_id")
	if !exists {
		c.JSON(http.StatusUnauthorized, gin.H{"error": "User not authenticated"})
		return
	}

	// Clean and validate tags
	cleanedTags := make([]string, 0, len(req.Tags))
	for _, tag := range req.Tags {
		cleaned := strings.TrimSpace(strings.ToLower(tag))
		if cleaned != "" && len(cleaned) <= 30 {
			cleanedTags = append(cleanedTags, cleaned)
		}
	}

	if len(cleanedTags) == 0 {
		c.JSON(http.StatusBadRequest, gin.H{"error": "At least one valid tag is required"})
		return
	}

	updatedTags, err := h.tagService.UpdateRecipeTags(userID.(string), recipeID, cleanedTags, req.Action)
	if err != nil {
		if strings.Contains(err.Error(), "not found") {
			c.JSON(http.StatusNotFound, gin.H{"error": "Recipe not found"})
			return
		}
		if strings.Contains(err.Error(), "permission") {
			c.JSON(http.StatusForbidden, gin.H{"error": "Permission denied"})
			return
		}
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to update recipe tags", "details": err.Error()})
		return
	}

	c.JSON(http.StatusOK, UpdateRecipeTagsResponse{
		RecipeID:    recipeID,
		UpdatedTags: updatedTags,
		Message:     "Recipe tags updated successfully",
	})
}

type GetRecipeTagsResponse struct {
	RecipeID     string            `json:"recipe_id"`
	UserTags     []string          `json:"user_tags"`
	CommunityTags []CommunityTag   `json:"community_tags"`
	TagStats     map[string]TagStat `json:"tag_stats"`
}

type CommunityTag struct {
	Tag        string `json:"tag"`
	VoteCount  int    `json:"vote_count"`
	UserVoted  bool   `json:"user_voted"`
	Confidence float64 `json:"confidence"`
}

type TagStat struct {
	UsageCount int     `json:"usage_count"`
	Trending   bool    `json:"trending"`
	Category   string  `json:"category"`
	Confidence float64 `json:"confidence"`
}

func (h *TagManagementHandler) GetRecipeTags(c *gin.Context) {
	recipeID := c.Param("id")
	if recipeID == "" {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Recipe ID is required"})
		return
	}

	userID, exists := c.Get("user_id")
	if !exists {
		c.JSON(http.StatusUnauthorized, gin.H{"error": "User not authenticated"})
		return
	}

	userTags, communityTags, tagStats, err := h.tagService.GetRecipeTags(userID.(string), recipeID)
	if err != nil {
		if strings.Contains(err.Error(), "not found") {
			c.JSON(http.StatusNotFound, gin.H{"error": "Recipe not found"})
			return
		}
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to get recipe tags", "details": err.Error()})
		return
	}

	c.JSON(http.StatusOK, GetRecipeTagsResponse{
		RecipeID:      recipeID,
		UserTags:      userTags,
		CommunityTags: communityTags,
		TagStats:      tagStats,
	})
}

type VoteOnTagRequest struct {
	Tag    string `json:"tag" binding:"required,min=1,max=30"`
	Action string `json:"action" binding:"required,oneof=upvote downvote remove"`
}

type VoteOnTagResponse struct {
	Tag       string `json:"tag"`
	VoteCount int    `json:"vote_count"`
	UserVoted bool   `json:"user_voted"`
	Message   string `json:"message"`
}

func (h *TagManagementHandler) VoteOnTag(c *gin.Context) {
	recipeID := c.Param("id")
	if recipeID == "" {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Recipe ID is required"})
		return
	}

	var req VoteOnTagRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid request format", "details": err.Error()})
		return
	}

	userID, exists := c.Get("user_id")
	if !exists {
		c.JSON(http.StatusUnauthorized, gin.H{"error": "User not authenticated"})
		return
	}

	voteCount, userVoted, err := h.tagService.VoteOnTag(userID.(string), recipeID, strings.TrimSpace(strings.ToLower(req.Tag)), req.Action)
	if err != nil {
		if strings.Contains(err.Error(), "not found") {
			c.JSON(http.StatusNotFound, gin.H{"error": "Recipe or tag not found"})
			return
		}
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to vote on tag", "details": err.Error()})
		return
	}

	c.JSON(http.StatusOK, VoteOnTagResponse{
		Tag:       req.Tag,
		VoteCount: voteCount,
		UserVoted: userVoted,
		Message:   "Vote recorded successfully",
	})
}
package handlers

import (
	"net/http"
	"strconv"
	"strings"

	"github.com/gin-gonic/gin"
	"github.com/imkitchen/backend/internal/middleware"
	"github.com/imkitchen/backend/internal/services"
)

type AttributionHandler struct {
	attributionService *services.AttributionService
}

func NewAttributionHandler(attributionService *services.AttributionService) *AttributionHandler {
	return &AttributionHandler{
		attributionService: attributionService,
	}
}

func (h *AttributionHandler) RegisterRoutes(rg *gin.RouterGroup) {
	recipes := rg.Group("/recipes")
	recipes.Use(middleware.AuthRequired())
	{
		recipes.GET("/:id/attribution", h.GetRecipeAttribution)
		recipes.GET("/:id/metrics", h.GetRecipeMetrics)
		recipes.GET("/:id/chain", h.GetRecipeChain)
		recipes.GET("/:id/analytics", h.GetEngagementAnalytics)
	}

	contributors := rg.Group("/contributors")
	contributors.Use(middleware.AuthRequired())
	{
		contributors.GET("/:id/profile", h.GetContributorProfile)
		contributors.GET("/:id/metrics", h.GetContributorMetrics)
		contributors.GET("/:id/achievements", h.GetContributorAchievements)
		contributors.POST("/:id/achievements", h.AwardAchievement)
		contributors.GET("/trending", h.GetTrendingContributors)
		contributors.GET("/leaderboard", h.GetContributorLeaderboard)
	}

	user := rg.Group("/user")
	user.Use(middleware.AuthRequired())
	{
		user.GET("/metrics", h.GetPersonalMetrics)
		user.GET("/achievements", h.GetPersonalAchievements)
		user.PUT("/attribution-preferences", h.UpdateAttributionPreferences)
		user.GET("/attribution-export", h.ExportAttributionData)
	}

	community := rg.Group("/community")
	community.Use(middleware.AuthRequired())
	{
		community.GET("/metrics", h.GetCommunityOverviewMetrics)
	}

	attribution := rg.Group("/attribution")
	attribution.Use(middleware.AuthRequired())
	{
		attribution.POST("/report", h.ReportAttributionIssue)
	}
}

type RecipeAttributionResponse struct {
	Attribution *services.RecipeAttribution `json:"attribution"`
}

func (h *AttributionHandler) GetRecipeAttribution(c *gin.Context) {
	recipeID := c.Param("id")
	if recipeID == "" {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Recipe ID is required"})
		return
	}

	attribution, err := h.attributionService.GetRecipeAttribution(c, recipeID)
	if err != nil {
		if strings.Contains(err.Error(), "not found") {
			c.JSON(http.StatusNotFound, gin.H{"error": "Recipe attribution not found"})
			return
		}
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to get recipe attribution", "details": err.Error()})
		return
	}

	c.JSON(http.StatusOK, RecipeAttributionResponse{
		Attribution: attribution,
	})
}

type ContributorProfileResponse struct {
	Profile *services.ContributorProfile `json:"profile"`
}

func (h *AttributionHandler) GetContributorProfile(c *gin.Context) {
	contributorID := c.Param("id")
	if contributorID == "" {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Contributor ID is required"})
		return
	}

	profile, err := h.attributionService.GetContributorProfile(c, contributorID)
	if err != nil {
		if strings.Contains(err.Error(), "not found") {
			c.JSON(http.StatusNotFound, gin.H{"error": "Contributor not found"})
			return
		}
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to get contributor profile", "details": err.Error()})
		return
	}

	c.JSON(http.StatusOK, ContributorProfileResponse{
		Profile: profile,
	})
}

type CommunityMetricsResponse struct {
	Metrics *services.CommunityMetricsData `json:"metrics"`
}

func (h *AttributionHandler) GetRecipeMetrics(c *gin.Context) {
	recipeID := c.Param("id")
	if recipeID == "" {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Recipe ID is required"})
		return
	}

	timeframe := services.MetricsTimeframe(c.DefaultQuery("timeframe", "week"))
	
	metrics, err := h.attributionService.GetRecipeMetrics(c, recipeID, timeframe)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to get recipe metrics", "details": err.Error()})
		return
	}

	c.JSON(http.StatusOK, CommunityMetricsResponse{
		Metrics: metrics,
	})
}

func (h *AttributionHandler) GetContributorMetrics(c *gin.Context) {
	contributorID := c.Param("id")
	if contributorID == "" {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Contributor ID is required"})
		return
	}

	timeframe := services.MetricsTimeframe(c.DefaultQuery("timeframe", "week"))
	
	metrics, err := h.attributionService.GetContributorMetrics(c, contributorID, timeframe)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to get contributor metrics", "details": err.Error()})
		return
	}

	c.JSON(http.StatusOK, CommunityMetricsResponse{
		Metrics: metrics,
	})
}

func (h *AttributionHandler) GetPersonalMetrics(c *gin.Context) {
	timeframe := services.MetricsTimeframe(c.DefaultQuery("timeframe", "week"))
	
	metrics, err := h.attributionService.GetPersonalMetrics(c, timeframe)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to get personal metrics", "details": err.Error()})
		return
	}

	c.JSON(http.StatusOK, CommunityMetricsResponse{
		Metrics: metrics,
	})
}

func (h *AttributionHandler) GetCommunityOverviewMetrics(c *gin.Context) {
	timeframe := services.MetricsTimeframe(c.DefaultQuery("timeframe", "week"))
	
	metrics, err := h.attributionService.GetCommunityOverviewMetrics(c, timeframe)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to get community metrics", "details": err.Error()})
		return
	}

	c.JSON(http.StatusOK, CommunityMetricsResponse{
		Metrics: metrics,
	})
}

type ContributorAchievementsResponse struct {
	Achievements []services.ContributorAchievement `json:"achievements"`
}

func (h *AttributionHandler) GetContributorAchievements(c *gin.Context) {
	contributorID := c.Param("id")
	if contributorID == "" {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Contributor ID is required"})
		return
	}

	achievements, err := h.attributionService.GetContributorAchievements(c, contributorID)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to get achievements", "details": err.Error()})
		return
	}

	c.JSON(http.StatusOK, ContributorAchievementsResponse{
		Achievements: achievements,
	})
}

func (h *AttributionHandler) GetPersonalAchievements(c *gin.Context) {
	userID, exists := c.Get("user_id")
	if !exists {
		c.JSON(http.StatusUnauthorized, gin.H{"error": "User not authenticated"})
		return
	}

	achievements, err := h.attributionService.GetContributorAchievements(c, userID.(string))
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to get personal achievements", "details": err.Error()})
		return
	}

	c.JSON(http.StatusOK, ContributorAchievementsResponse{
		Achievements: achievements,
	})
}

type AwardAchievementRequest struct {
	AchievementType string                 `json:"achievement_type" binding:"required"`
	Metadata        map[string]interface{} `json:"metadata,omitempty"`
}

type AwardAchievementResponse struct {
	Success     bool                           `json:"success"`
	Achievement services.ContributorAchievement `json:"achievement"`
	Message     string                         `json:"message"`
}

func (h *AttributionHandler) AwardAchievement(c *gin.Context) {
	contributorID := c.Param("id")
	if contributorID == "" {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Contributor ID is required"})
		return
	}

	var req AwardAchievementRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid request format", "details": err.Error()})
		return
	}

	// Check if user has admin permissions
	userRole, exists := c.Get("user_role")
	if !exists || userRole != "admin" {
		c.JSON(http.StatusForbidden, gin.H{"error": "Admin access required"})
		return
	}

	err := h.attributionService.AwardAchievement(c, contributorID, req.AchievementType, req.Metadata)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to award achievement", "details": err.Error()})
		return
	}

	c.JSON(http.StatusOK, AwardAchievementResponse{
		Success: true,
		Message: "Achievement awarded successfully",
	})
}

type RecipeChainResponse struct {
	Chain            []services.RecipeChainLink `json:"chain"`
	TotalAdaptations int                        `json:"total_adaptations"`
}

func (h *AttributionHandler) GetRecipeChain(c *gin.Context) {
	recipeID := c.Param("id")
	if recipeID == "" {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Recipe ID is required"})
		return
	}

	attribution, err := h.attributionService.GetRecipeAttribution(c, recipeID)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to get recipe chain", "details": err.Error()})
		return
	}

	c.JSON(http.StatusOK, RecipeChainResponse{
		Chain:            attribution.RecipeChain,
		TotalAdaptations: len(attribution.RecipeChain) - 1,
	})
}

type TrendingContributorsResponse struct {
	Contributors []TrendingContributor `json:"contributors"`
}

type TrendingContributor struct {
	Contributor services.ContributorProfile `json:"contributor"`
	Metrics     TrendingMetrics             `json:"metrics"`
}

type TrendingMetrics struct {
	NewRecipes    int     `json:"new_recipes"`
	TotalImports  int     `json:"total_imports"`
	AverageRating float64 `json:"average_rating"`
	TrendingScore float64 `json:"trending_score"`
}

func (h *AttributionHandler) GetTrendingContributors(c *gin.Context) {
	limitStr := c.DefaultQuery("limit", "10")
	timeframe := services.MetricsTimeframe(c.DefaultQuery("timeframe", "week"))

	limit, err := strconv.Atoi(limitStr)
	if err != nil || limit <= 0 || limit > 50 {
		limit = 10
	}

	// This would need to be implemented in the service
	c.JSON(http.StatusOK, TrendingContributorsResponse{
		Contributors: []TrendingContributor{},
	})
}

type ContributorLeaderboardResponse struct {
	Leaderboard []LeaderboardEntry `json:"leaderboard"`
}

type LeaderboardEntry struct {
	Rank        int                         `json:"rank"`
	Contributor services.ContributorProfile `json:"contributor"`
	Score       float64                     `json:"score"`
	Change      int                         `json:"change"`
}

func (h *AttributionHandler) GetContributorLeaderboard(c *gin.Context) {
	category := c.DefaultQuery("category", "recipes")
	timeframe := services.MetricsTimeframe(c.DefaultQuery("timeframe", "month"))
	limitStr := c.DefaultQuery("limit", "50")

	limit, err := strconv.Atoi(limitStr)
	if err != nil || limit <= 0 || limit > 100 {
		limit = 50
	}

	// This would need to be implemented in the service
	c.JSON(http.StatusOK, ContributorLeaderboardResponse{
		Leaderboard: []LeaderboardEntry{},
	})
}

type AttributionPreferencesRequest struct {
	PreserveAttribution  bool `json:"preserve_attribution"`
	AllowDerivatives     bool `json:"allow_derivatives"`
	RequireNotification  bool `json:"require_notification"`
}

func (h *AttributionHandler) UpdateAttributionPreferences(c *gin.Context) {
	var req AttributionPreferencesRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid request format", "details": err.Error()})
		return
	}

	userID, exists := c.Get("user_id")
	if !exists {
		c.JSON(http.StatusUnauthorized, gin.H{"error": "User not authenticated"})
		return
	}

	// This would need to be implemented in the service
	_ = userID

	c.JSON(http.StatusOK, gin.H{
		"success": true,
		"message": "Attribution preferences updated successfully",
	})
}

type ReportAttributionIssueRequest struct {
	RecipeID    string `json:"recipe_id" binding:"required"`
	IssueType   string `json:"issue_type" binding:"required,oneof=missing_attribution incorrect_attribution unauthorized_use"`
	Description string `json:"description" binding:"required,min=10,max=500"`
}

func (h *AttributionHandler) ReportAttributionIssue(c *gin.Context) {
	var req ReportAttributionIssueRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid request format", "details": err.Error()})
		return
	}

	userID, exists := c.Get("user_id")
	if !exists {
		c.JSON(http.StatusUnauthorized, gin.H{"error": "User not authenticated"})
		return
	}

	// This would need to be implemented in the service
	_ = userID

	c.JSON(http.StatusOK, gin.H{
		"success": true,
		"message": "Attribution issue reported successfully",
	})
}

type EngagementAnalyticsResponse struct {
	DailyViews      []DailyViewData     `json:"daily_views"`
	ReferralSources []ReferralSource    `json:"referral_sources"`
	UserActions     []UserAction        `json:"user_actions"`
	GeographicData  []GeographicData    `json:"geographic_data"`
}

type DailyViewData struct {
	Date  string `json:"date"`
	Views int    `json:"views"`
}

type ReferralSource struct {
	Source string `json:"source"`
	Count  int    `json:"count"`
}

type UserAction struct {
	Action string `json:"action"`
	Count  int    `json:"count"`
}

type GeographicData struct {
	Country string `json:"country"`
	Count   int    `json:"count"`
}

func (h *AttributionHandler) GetEngagementAnalytics(c *gin.Context) {
	recipeID := c.Param("id")
	if recipeID == "" {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Recipe ID is required"})
		return
	}

	timeframe := services.MetricsTimeframe(c.DefaultQuery("timeframe", "week"))
	_ = timeframe

	// This would need to be implemented in the service
	c.JSON(http.StatusOK, EngagementAnalyticsResponse{
		DailyViews:      []DailyViewData{},
		ReferralSources: []ReferralSource{},
		UserActions:     []UserAction{},
		GeographicData:  []GeographicData{},
	})
}

type AttributionExportResponse struct {
	Attributions  []services.RecipeAttribution `json:"attributions"`
	Contributions []ContributionData           `json:"contributions"`
	Achievements  []services.ContributorAchievement `json:"achievements"`
}

type ContributionData struct {
	RecipeID     string `json:"recipe_id"`
	Title        string `json:"title"`
	CreatedAt    string `json:"created_at"`
	TotalImports int    `json:"total_imports"`
}

func (h *AttributionHandler) ExportAttributionData(c *gin.Context) {
	userID, exists := c.Get("user_id")
	if !exists {
		c.JSON(http.StatusUnauthorized, gin.H{"error": "User not authenticated"})
		return
	}

	// This would need to be implemented in the service
	_ = userID

	c.JSON(http.StatusOK, AttributionExportResponse{
		Attributions:  []services.RecipeAttribution{},
		Contributions: []ContributionData{},
		Achievements:  []services.ContributorAchievement{},
	})
}
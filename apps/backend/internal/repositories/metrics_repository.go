package repositories

import (
	"context"
	"time"
	
	"github.com/google/uuid"
)

// MetricsRepository handles application metrics storage
type MetricsRepository interface {
	RecordMetric(ctx context.Context, metric *Metric) error
	GetMetricsByType(ctx context.Context, metricType string, timeRange time.Duration) ([]*Metric, error)
	GetAggregatedMetrics(ctx context.Context, metricType string, timeRange time.Duration, groupBy string) ([]*AggregatedMetric, error)
	DeleteOldMetrics(ctx context.Context, olderThan time.Time) error
	GetRecipeOverview(ctx context.Context, recipeID string, timeframe string) (*RecipeOverview, error)
	GetRecipePopularity(ctx context.Context, recipeID string, timeframe string) (*RecipePopularity, error)
	GetRecipeEngagement(ctx context.Context, recipeID string, timeframe string) (*RecipeEngagement, error)
	GetRecipeGeographic(ctx context.Context, recipeID string, timeframe string) (*RecipeGeographic, error)
	GetContributorOverview(ctx context.Context, contributorID string, timeframe string) (*ContributorOverview, error)
}

// Metric represents a single metric data point
type Metric struct {
	ID         uuid.UUID              `json:"id" db:"id"`
	Type       string                 `json:"type" db:"type"`
	Value      float64                `json:"value" db:"value"`
	Tags       map[string]interface{} `json:"tags" db:"tags"`
	Timestamp  time.Time              `json:"timestamp" db:"timestamp"`
	CreatedAt  time.Time              `json:"created_at" db:"created_at"`
}

// AggregatedMetric represents aggregated metric data
type AggregatedMetric struct {
	Type      string    `json:"type"`
	Count     int64     `json:"count"`
	Sum       float64   `json:"sum"`
	Average   float64   `json:"average"`
	Min       float64   `json:"min"`
	Max       float64   `json:"max"`
	Period    time.Time `json:"period"`
}

// RecipeOverview represents recipe overview metrics
type RecipeOverview struct {
	TotalViews   int `json:"total_views"`
	TotalImports int `json:"total_imports"`
	TotalRatings int `json:"total_ratings"`
	AvgRating    float64 `json:"avg_rating"`
}

// RecipePopularity represents recipe popularity metrics
type RecipePopularity struct {
	PopularityScore float64 `json:"popularity_score"`
	TrendingRank    int     `json:"trending_rank"`
	CategoryRank    int     `json:"category_rank"`
}

// RecipeEngagement represents recipe engagement metrics
type RecipeEngagement struct {
	WeeklyViews      int `json:"weekly_views"`
	SavesToMealPlans int `json:"saves_to_meal_plans"`
	SocialShares     int `json:"social_shares"`
	Comments         int `json:"comments"`
}

// RecipeGeographic represents recipe geographic metrics
type RecipeGeographic struct {
	TopCountries []string `json:"top_countries"`
	TopCities    []string `json:"top_cities"`
}

// ContributorOverview represents contributor overview metrics
type ContributorOverview struct {
	TotalRecipes      int     `json:"total_recipes"`
	TotalViews        int     `json:"total_views"`
	TotalImports      int     `json:"total_imports"`
	AverageRating     float64 `json:"average_rating"`
	PopularityRank    int     `json:"popularity_rank"`
}
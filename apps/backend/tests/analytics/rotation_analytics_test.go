package analytics

import (
	"context"
	"encoding/json"
	"fmt"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
	"time"

	"github.com/gin-gonic/gin"
	"github.com/google/uuid"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"github.com/imkitchen/backend/internal/handlers"
	"github.com/imkitchen/backend/internal/services"
)

func TestRotationAnalyticsEndToEnd(t *testing.T) {
	// Setup test environment
	gin.SetMode(gin.TestMode)
	
	// Mock services
	mockRotationService := &MockRotationService{}
	mockAnalyticsCache := &MockAnalyticsCacheService{}
	
	// Setup routes
	router := gin.New()
	router.Use(func(c *gin.Context) {
		c.Set("UserID", "test-user-id")
		c.Next()
	})
	
	router.GET("/api/v1/users/rotation/stats", handlers.GetRotationAnalytics(mockRotationService, mockAnalyticsCache))
	router.POST("/api/v1/users/rotation/reset", handlers.ResetRotationCycle(mockRotationService))
	router.GET("/api/v1/users/rotation/export", handlers.ExportRotationData(mockRotationService))
	router.GET("/api/v1/users/rotation/debug-logs", handlers.GetDebugLogs(mockRotationService))

	t.Run("GetRotationAnalytics_Success", func(t *testing.T) {
		// Setup mock data
		expectedAnalytics := &services.RotationAnalytics{
			VarietyScore:       85.5,
			RotationEfficiency: 0.92,
			WeeksAnalyzed:      12,
			ComplexityDistribution: map[string]float64{
				"Easy":   0.4,
				"Medium": 0.4,
				"Hard":   0.2,
			},
			FavoritesImpact: 0.75,
			CalculatedAt:    time.Now().Format(time.RFC3339),
		}
		
		mockRotationService.SetAnalytics(expectedAnalytics)
		
		// Make request
		req, _ := http.NewRequest("GET", "/api/v1/users/rotation/stats?weeks=12&includeDetailed=true", nil)
		w := httptest.NewRecorder()
		router.ServeHTTP(w, req)
		
		// Verify response
		assert.Equal(t, http.StatusOK, w.Code)
		
		var response map[string]interface{}
		err := json.Unmarshal(w.Body.Bytes(), &response)
		require.NoError(t, err)
		
		data := response["data"].(map[string]interface{})
		assert.Equal(t, 85.5, data["varietyScore"])
		assert.Equal(t, 0.92, data["rotationEfficiency"])
		assert.Equal(t, float64(12), data["weeksAnalyzed"])
	})

	t.Run("GetRotationAnalytics_CacheHit", func(t *testing.T) {
		// Setup cached data
		cachedAnalytics := &services.RotationAnalytics{
			VarietyScore:       90.0,
			RotationEfficiency: 0.95,
			WeeksAnalyzed:      8,
			CalculatedAt:       time.Now().Format(time.RFC3339),
		}
		
		userID := uuid.MustParse("test-user-id")
		mockAnalyticsCache.SetCachedAnalytics(userID, 8, true, cachedAnalytics)
		
		// Make request
		req, _ := http.NewRequest("GET", "/api/v1/users/rotation/stats?weeks=8", nil)
		w := httptest.NewRecorder()
		router.ServeHTTP(w, req)
		
		// Verify response
		assert.Equal(t, http.StatusOK, w.Code)
		
		var response map[string]interface{}
		err := json.Unmarshal(w.Body.Bytes(), &response)
		require.NoError(t, err)
		
		data := response["data"].(map[string]interface{})
		assert.Equal(t, 90.0, data["varietyScore"])
		
		// Verify cache was hit
		assert.True(t, mockAnalyticsCache.WasCacheHit())
	})

	t.Run("ResetRotationCycle_Success", func(t *testing.T) {
		resetRequest := map[string]interface{}{
			"confirmReset":      true,
			"preservePatterns":  true,
			"preserveFavorites": true,
		}
		
		requestBody, _ := json.Marshal(resetRequest)
		
		req, _ := http.NewRequest("POST", "/api/v1/users/rotation/reset", strings.NewReader(string(requestBody)))
		req.Header.Set("Content-Type", "application/json")
		w := httptest.NewRecorder()
		router.ServeHTTP(w, req)
		
		// Verify response
		assert.Equal(t, http.StatusOK, w.Code)
		
		var response map[string]interface{}
		err := json.Unmarshal(w.Body.Bytes(), &response)
		require.NoError(t, err)
		
		assert.Equal(t, "success", response["status"])
		assert.True(t, mockRotationService.WasResetCalled())
	})

	t.Run("ExportRotationData_JSON", func(t *testing.T) {
		req, _ := http.NewRequest("GET", "/api/v1/users/rotation/export?format=json&dateRange=2024-01-01,2024-12-31", nil)
		w := httptest.NewRecorder()
		router.ServeHTTP(w, req)
		
		// Verify response
		assert.Equal(t, http.StatusOK, w.Code)
		assert.Equal(t, "application/json", w.Header().Get("Content-Type"))
		
		// Verify data structure
		var exportData map[string]interface{}
		err := json.Unmarshal(w.Body.Bytes(), &exportData)
		require.NoError(t, err)
		
		assert.Contains(t, exportData, "analytics")
		assert.Contains(t, exportData, "exportedAt")
	})

	t.Run("GetDebugLogs_Success", func(t *testing.T) {
		// Setup mock debug logs
		mockLogs := []services.RotationDebugLog{
			{
				ID:               "log1",
				Timestamp:        time.Now().Format(time.RFC3339),
				DecisionType:     "recipe_selection",
				RecipeName:       stringPtr("Spaghetti Carbonara"),
				AlgorithmVersion: "v2.1.0",
			},
			{
				ID:                 "log2",
				Timestamp:          time.Now().Format(time.RFC3339),
				DecisionType:       "constraint_violation",
				ConstraintViolated: stringPtr("Time constraint exceeded"),
				AlgorithmVersion:   "v2.1.0",
			},
		}
		
		mockRotationService.SetDebugLogs(mockLogs)
		
		req, _ := http.NewRequest("GET", "/api/v1/users/rotation/debug-logs?limit=100", nil)
		w := httptest.NewRecorder()
		router.ServeHTTP(w, req)
		
		// Verify response
		assert.Equal(t, http.StatusOK, w.Code)
		
		var response map[string]interface{}
		err := json.Unmarshal(w.Body.Bytes(), &response)
		require.NoError(t, err)
		
		data := response["data"].([]interface{})
		assert.Len(t, data, 2)
		
		log1 := data[0].(map[string]interface{})
		assert.Equal(t, "recipe_selection", log1["decisionType"])
		assert.Equal(t, "Spaghetti Carbonara", log1["recipeName"])
	})

	t.Run("AnalyticsPerformance_LoadTest", func(t *testing.T) {
		// Performance test - multiple concurrent requests
		const numRequests = 50
		results := make(chan time.Duration, numRequests)
		
		for i := 0; i < numRequests; i++ {
			go func() {
				start := time.Now()
				req, _ := http.NewRequest("GET", "/api/v1/users/rotation/stats?weeks=12", nil)
				w := httptest.NewRecorder()
				router.ServeHTTP(w, req)
				
				duration := time.Since(start)
				results <- duration
			}()
		}
		
		// Collect results
		var totalDuration time.Duration
		var maxDuration time.Duration
		
		for i := 0; i < numRequests; i++ {
			duration := <-results
			totalDuration += duration
			if duration > maxDuration {
				maxDuration = duration
			}
		}
		
		avgDuration := totalDuration / numRequests
		
		t.Logf("Performance Results:")
		t.Logf("  Average response time: %v", avgDuration)
		t.Logf("  Max response time: %v", maxDuration)
		t.Logf("  Total requests: %d", numRequests)
		
		// Assert performance requirements
		assert.Less(t, avgDuration, 1*time.Second, "Average response time should be under 1 second")
		assert.Less(t, maxDuration, 3*time.Second, "Max response time should be under 3 seconds")
	})
}

// Mock services for testing
type MockRotationService struct {
	analytics    *services.RotationAnalytics
	debugLogs    []services.RotationDebugLog
	resetCalled  bool
	exportCalled bool
}

func (m *MockRotationService) SetAnalytics(analytics *services.RotationAnalytics) {
	m.analytics = analytics
}

func (m *MockRotationService) SetDebugLogs(logs []services.RotationDebugLog) {
	m.debugLogs = logs
}

func (m *MockRotationService) WasResetCalled() bool {
	return m.resetCalled
}

func (m *MockRotationService) GetRotationAnalytics(userID uuid.UUID, weeks int, includeDetailed bool) (*services.RotationAnalytics, error) {
	if m.analytics == nil {
		return &services.RotationAnalytics{
			VarietyScore:       75.0,
			RotationEfficiency: 0.85,
			WeeksAnalyzed:      weeks,
			CalculatedAt:       time.Now().Format(time.RFC3339),
		}, nil
	}
	return m.analytics, nil
}

func (m *MockRotationService) ResetRotationCycle(userID uuid.UUID) error {
	m.resetCalled = true
	return nil
}

func (m *MockRotationService) ExportRotationData(userID uuid.UUID, format string, dateRange string) (map[string]interface{}, error) {
	m.exportCalled = true
	return map[string]interface{}{
		"analytics":  m.analytics,
		"format":     format,
		"dateRange":  dateRange,
		"exportedAt": time.Now().Format(time.RFC3339),
	}, nil
}

func (m *MockRotationService) GetDebugLogs(userID uuid.UUID, limit int) ([]services.RotationDebugLog, error) {
	if m.debugLogs == nil {
		return []services.RotationDebugLog{}, nil
	}
	return m.debugLogs, nil
}

type MockAnalyticsCacheService struct {
	cachedAnalytics map[string]*services.RotationAnalytics
	cacheHit        bool
}

func (m *MockAnalyticsCacheService) SetCachedAnalytics(userID uuid.UUID, weeks int, includeDetailed bool, analytics *services.RotationAnalytics) {
	if m.cachedAnalytics == nil {
		m.cachedAnalytics = make(map[string]*services.RotationAnalytics)
	}
	key := fmt.Sprintf("%s_%d_%t", userID.String(), weeks, includeDetailed)
	m.cachedAnalytics[key] = analytics
}

func (m *MockAnalyticsCacheService) WasCacheHit() bool {
	return m.cacheHit
}

func (m *MockAnalyticsCacheService) GetCachedRotationAnalytics(ctx context.Context, userID uuid.UUID, weeks int, includeDetailed bool) (*services.RotationAnalytics, error) {
	if m.cachedAnalytics == nil {
		return nil, nil
	}
	
	key := fmt.Sprintf("%s_%d_%t", userID.String(), weeks, includeDetailed)
	if analytics, exists := m.cachedAnalytics[key]; exists {
		m.cacheHit = true
		return analytics, nil
	}
	
	return nil, nil
}

func (m *MockAnalyticsCacheService) CacheRotationAnalytics(ctx context.Context, userID uuid.UUID, weeks int, includeDetailed bool, analytics *services.RotationAnalytics) error {
	m.SetCachedAnalytics(userID, weeks, includeDetailed, analytics)
	return nil
}

func (m *MockAnalyticsCacheService) InvalidateUserAnalyticsCache(ctx context.Context, userID uuid.UUID) error {
	m.cachedAnalytics = make(map[string]*services.RotationAnalytics)
	return nil
}

// Helper function
func stringPtr(s string) *string {
	return &s
}
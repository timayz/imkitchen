package tests

import (
	"bytes"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/gin-gonic/gin"
	"github.com/google/uuid"
	"github.com/imkitchen/backend/internal/handlers"
	"github.com/imkitchen/backend/internal/models"
	"github.com/imkitchen/backend/internal/services"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"
	"github.com/stretchr/testify/require"
)

func TestPreferenceHandler_GetUserPreferences(t *testing.T) {
	gin.SetMode(gin.TestMode)
	
	mockRepo := new(MockPreferenceRepository)
	service := services.NewPreferenceService(mockRepo)
	handler := handlers.NewPreferenceHandler(service)
	
	userID := uuid.New()
	
	t.Run("returns preferences successfully", func(t *testing.T) {
		expectedPrefs := &models.CoreUserPreferences{
			MaxCookTime:         60,
			PreferredComplexity: "moderate",
		}
		
		mockRepo.On("GetUserPreferences", userID).Return(expectedPrefs, nil).Once()
		
		router := gin.New()
		router.GET("/preferences", func(c *gin.Context) {
			c.Set("user_id", userID)
			handler.GetUserPreferences(c)
		})
		
		req := httptest.NewRequest(http.MethodGet, "/preferences", nil)
		w := httptest.NewRecorder()
		
		router.ServeHTTP(w, req)
		
		assert.Equal(t, http.StatusOK, w.Code)
		
		var response map[string]interface{}
		err := json.Unmarshal(w.Body.Bytes(), &response)
		require.NoError(t, err)
		
		data := response["data"].(map[string]interface{})
		assert.Equal(t, float64(60), data["maxCookTime"])
		assert.Equal(t, "moderate", data["preferredComplexity"])
		
		mockRepo.AssertExpectations(t)
	})
	
	t.Run("returns unauthorized when user_id missing", func(t *testing.T) {
		router := gin.New()
		router.GET("/preferences", handler.GetUserPreferences)
		
		req := httptest.NewRequest(http.MethodGet, "/preferences", nil)
		w := httptest.NewRecorder()
		
		router.ServeHTTP(w, req)
		
		assert.Equal(t, http.StatusUnauthorized, w.Code)
	})
}

func TestPreferenceHandler_UpdateUserPreferences(t *testing.T) {
	gin.SetMode(gin.TestMode)
	
	mockRepo := new(MockPreferenceRepository)
	service := services.NewPreferenceService(mockRepo)
	handler := handlers.NewPreferenceHandler(service)
	
	userID := uuid.New()
	
	t.Run("updates preferences successfully", func(t *testing.T) {
		updatePrefs := &models.CoreUserPreferences{
			MaxCookTime:         90,
			PreferredComplexity: "complex",
		}
		
		mockRepo.On("UpdateUserPreferences", userID, updatePrefs).Return(nil).Once()
		mockRepo.On("GetUserPreferences", userID).Return(updatePrefs, nil).Once()
		
		router := gin.New()
		router.PUT("/preferences", func(c *gin.Context) {
			c.Set("user_id", userID)
			handler.UpdateUserPreferences(c)
		})
		
		reqBody, _ := json.Marshal(updatePrefs)
		req := httptest.NewRequest(http.MethodPUT, "/preferences", bytes.NewBuffer(reqBody))
		req.Header.Set("Content-Type", "application/json")
		w := httptest.NewRecorder()
		
		router.ServeHTTP(w, req)
		
		assert.Equal(t, http.StatusOK, w.Code)
		
		var response map[string]interface{}
		err := json.Unmarshal(w.Body.Bytes(), &response)
		require.NoError(t, err)
		
		data := response["data"].(map[string]interface{})
		assert.Equal(t, float64(90), data["maxCookTime"])
		assert.Equal(t, "complex", data["preferredComplexity"])
		
		mockRepo.AssertExpectations(t)
	})
}

func TestPreferenceHandler_ResetUserPreferences(t *testing.T) {
	gin.SetMode(gin.TestMode)
	
	mockRepo := new(MockPreferenceRepository)
	service := services.NewPreferenceService(mockRepo)
	handler := handlers.NewPreferenceHandler(service)
	
	userID := uuid.New()
	
	t.Run("resets preferences successfully", func(t *testing.T) {
		defaultPrefs := &models.CoreUserPreferences{
			MaxCookTime:         60,
			PreferredComplexity: "moderate",
		}
		
		mockRepo.On("UpdateUserPreferences", userID, defaultPrefs).Return(nil).Once()
		mockRepo.On("GetUserPreferences", userID).Return(defaultPrefs, nil).Once()
		
		router := gin.New()
		router.POST("/preferences/reset", func(c *gin.Context) {
			c.Set("user_id", userID)
			handler.ResetUserPreferences(c)
		})
		
		req := httptest.NewRequest(http.MethodPost, "/preferences/reset", nil)
		w := httptest.NewRecorder()
		
		router.ServeHTTP(w, req)
		
		assert.Equal(t, http.StatusOK, w.Code)
		
		var response map[string]interface{}
		err := json.Unmarshal(w.Body.Bytes(), &response)
		require.NoError(t, err)
		
		data := response["data"].(map[string]interface{})
		assert.Equal(t, float64(60), data["maxCookTime"])
		assert.Equal(t, "moderate", data["preferredComplexity"])
		
		mockRepo.AssertExpectations(t)
	})
}
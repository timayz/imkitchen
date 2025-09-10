package services

import (
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"net/url"
	"regexp"
	"strconv"
	"strings"
	"time"

	"github.com/imkitchen/backend/internal/models"
)

type RecipeImportService interface {
	ImportFromURL(input *models.ImportRecipeInput) (*models.ImportRecipeResult, error)
	ParseRecipeFromHTML(htmlContent, sourceURL string) (*models.CreateRecipeInput, error)
}

type recipeImportService struct {
	httpClient *http.Client
}

func NewRecipeImportService() RecipeImportService {
	return &recipeImportService{
		httpClient: &http.Client{
			Timeout: 30 * time.Second,
		},
	}
}

func (s *recipeImportService) ImportFromURL(input *models.ImportRecipeInput) (*models.ImportRecipeResult, error) {
	// Validate URL
	parsedURL, err := url.Parse(input.URL)
	if err != nil {
		return &models.ImportRecipeResult{
			Success: false,
			Error:   stringPtr("Invalid URL format"),
		}, nil
	}

	if parsedURL.Scheme != "http" && parsedURL.Scheme != "https" {
		return &models.ImportRecipeResult{
			Success: false,
			Error:   stringPtr("URL must use HTTP or HTTPS protocol"),
		}, nil
	}

	// Fetch the webpage
	htmlContent, err := s.fetchURL(input.URL)
	if err != nil {
		return &models.ImportRecipeResult{
			Success: false,
			Error:   stringPtr(fmt.Sprintf("Failed to fetch URL: %v", err)),
		}, nil
	}

	// Parse recipe from HTML
	recipe, err := s.ParseRecipeFromHTML(htmlContent, input.URL)
	if err != nil {
		return &models.ImportRecipeResult{
			Success: false,
			Error:   stringPtr(fmt.Sprintf("Failed to parse recipe: %v", err)),
		}, nil
	}

	// Apply override fields if provided
	if input.OverrideFields != nil {
		s.applyOverrides(recipe, input.OverrideFields)
	}

	// Set source URL
	recipe.SourceURL = &input.URL

	warnings := []string{}
	if recipe.Title == "" {
		warnings = append(warnings, "Could not extract recipe title")
	}
	if len(recipe.Ingredients) == 0 {
		warnings = append(warnings, "Could not extract ingredients")
	}
	if len(recipe.Instructions) == 0 {
		warnings = append(warnings, "Could not extract instructions")
	}

	// Convert to Recipe model (this would typically be done by the service layer)
	// For now, return the CreateRecipeInput
	return &models.ImportRecipeResult{
		Success:  true,
		Error:    nil,
		Warnings: warnings,
	}, nil
}

func (s *recipeImportService) fetchURL(urlStr string) (string, error) {
	req, err := http.NewRequest("GET", urlStr, nil)
	if err != nil {
		return "", err
	}

	// Set a proper User-Agent to avoid blocking
	req.Header.Set("User-Agent", "imkitchen-recipe-importer/1.0")
	req.Header.Set("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")

	resp, err := s.httpClient.Do(req)
	if err != nil {
		return "", err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return "", fmt.Errorf("HTTP error: %d %s", resp.StatusCode, resp.Status)
	}

	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return "", err
	}

	return string(body), nil
}

func (s *recipeImportService) ParseRecipeFromHTML(htmlContent, sourceURL string) (*models.CreateRecipeInput, error) {
	// Try to parse JSON-LD structured data first
	if recipe, err := s.parseJSONLD(htmlContent); err == nil && recipe != nil {
		return recipe, nil
	}

	// Fall back to microdata parsing
	if recipe, err := s.parseMicrodata(htmlContent); err == nil && recipe != nil {
		return recipe, nil
	}

	// Final fallback: heuristic parsing
	return s.parseHeuristic(htmlContent, sourceURL)
}

func (s *recipeImportService) parseJSONLD(htmlContent string) (*models.CreateRecipeInput, error) {
	// Look for JSON-LD script tags
	jsonLDRegex := regexp.MustCompile(`<script[^>]*type=["']application/ld\+json["'][^>]*>(.*?)</script>`)
	matches := jsonLDRegex.FindAllStringSubmatch(htmlContent, -1)

	for _, match := range matches {
		if len(match) < 2 {
			continue
		}

		var data map[string]interface{}
		if err := json.Unmarshal([]byte(match[1]), &data); err != nil {
			continue
		}

		// Check if this is a Recipe schema
		if s.isRecipeSchema(data) {
			return s.parseRecipeFromJSONLD(data)
		}

		// Handle arrays of structured data
		if arr, ok := data["@graph"].([]interface{}); ok {
			for _, item := range arr {
				if itemMap, ok := item.(map[string]interface{}); ok && s.isRecipeSchema(itemMap) {
					return s.parseRecipeFromJSONLD(itemMap)
				}
			}
		}
	}

	return nil, fmt.Errorf("no JSON-LD recipe data found")
}

func (s *recipeImportService) isRecipeSchema(data map[string]interface{}) bool {
	typeVal, ok := data["@type"]
	if !ok {
		return false
	}

	switch v := typeVal.(type) {
	case string:
		return v == "Recipe"
	case []interface{}:
		for _, t := range v {
			if str, ok := t.(string); ok && str == "Recipe" {
				return true
			}
		}
	}
	return false
}

func (s *recipeImportService) parseRecipeFromJSONLD(data map[string]interface{}) (*models.CreateRecipeInput, error) {
	recipe := &models.CreateRecipeInput{
		Servings:     4, // Default
		Complexity:   "moderate", // Default
		MealType:     []string{"dinner"}, // Default
		Ingredients:  []models.RecipeIngredient{},
		Instructions: []models.RecipeInstruction{},
	}

	// Extract title
	if name, ok := data["name"].(string); ok {
		recipe.Title = name
	}

	// Extract description
	if desc, ok := data["description"].(string); ok {
		recipe.Description = &desc
	}

	// Extract image
	if image := s.extractImageFromJSONLD(data); image != "" {
		recipe.ImageURL = &image
	}

	// Extract timing
	if prepTime := s.extractDurationFromJSONLD(data, "prepTime"); prepTime > 0 {
		recipe.PrepTime = prepTime
	}
	if cookTime := s.extractDurationFromJSONLD(data, "cookTime"); cookTime > 0 {
		recipe.CookTime = cookTime
	}

	// Extract servings
	if yield := s.extractYieldFromJSONLD(data); yield > 0 {
		recipe.Servings = yield
	}

	// Extract ingredients
	if ingredients := s.extractIngredientsFromJSONLD(data); len(ingredients) > 0 {
		recipe.Ingredients = ingredients
	}

	// Extract instructions
	if instructions := s.extractInstructionsFromJSONLD(data); len(instructions) > 0 {
		recipe.Instructions = instructions
	}

	// Extract cuisine
	if cuisine, ok := data["recipeCuisine"].(string); ok {
		recipe.CuisineType = &cuisine
	}

	return recipe, nil
}

func (s *recipeImportService) parseMicrodata(htmlContent string) (*models.CreateRecipeInput, error) {
	// Basic microdata parsing - this could be much more sophisticated
	return nil, fmt.Errorf("microdata parsing not implemented")
}

func (s *recipeImportService) parseHeuristic(htmlContent, sourceURL string) (*models.CreateRecipeInput, error) {
	// Very basic heuristic parsing
	recipe := &models.CreateRecipeInput{
		Servings:     4,
		Complexity:   "moderate",
		MealType:     []string{"dinner"},
		Ingredients:  []models.RecipeIngredient{},
		Instructions: []models.RecipeInstruction{},
		PrepTime:     30, // Default
		CookTime:     30, // Default
	}

	// Try to extract title from <title> or <h1>
	titleRegex := regexp.MustCompile(`<title[^>]*>(.*?)</title>`)
	if match := titleRegex.FindStringSubmatch(htmlContent); len(match) > 1 {
		title := strings.TrimSpace(match[1])
		// Clean up common title suffixes
		title = regexp.MustCompile(`\s*-\s*.*$`).ReplaceAllString(title, "")
		if title != "" {
			recipe.Title = title
		}
	}

	if recipe.Title == "" {
		h1Regex := regexp.MustCompile(`<h1[^>]*>(.*?)</h1>`)
		if match := h1Regex.FindStringSubmatch(htmlContent); len(match) > 1 {
			recipe.Title = strings.TrimSpace(match[1])
		}
	}

	return recipe, nil
}

// Helper methods for JSON-LD parsing

func (s *recipeImportService) extractImageFromJSONLD(data map[string]interface{}) string {
	if image, ok := data["image"]; ok {
		switch v := image.(type) {
		case string:
			return v
		case map[string]interface{}:
			if url, ok := v["url"].(string); ok {
				return url
			}
		case []interface{}:
			if len(v) > 0 {
				if str, ok := v[0].(string); ok {
					return str
				}
				if obj, ok := v[0].(map[string]interface{}); ok {
					if url, ok := obj["url"].(string); ok {
						return url
					}
				}
			}
		}
	}
	return ""
}

func (s *recipeImportService) extractDurationFromJSONLD(data map[string]interface{}, key string) int {
	if duration, ok := data[key].(string); ok {
		// Parse ISO 8601 duration format (PT15M, PT1H30M, etc.)
		return s.parseISODuration(duration)
	}
	return 0
}

func (s *recipeImportService) parseISODuration(duration string) int {
	// Simple ISO 8601 duration parser
	duration = strings.ToUpper(duration)
	if !strings.HasPrefix(duration, "PT") {
		return 0
	}

	duration = duration[2:] // Remove "PT"
	minutes := 0

	// Extract hours
	hourRegex := regexp.MustCompile(`(\d+)H`)
	if match := hourRegex.FindStringSubmatch(duration); len(match) > 1 {
		if hours, err := strconv.Atoi(match[1]); err == nil {
			minutes += hours * 60
		}
	}

	// Extract minutes
	minuteRegex := regexp.MustCompile(`(\d+)M`)
	if match := minuteRegex.FindStringSubmatch(duration); len(match) > 1 {
		if mins, err := strconv.Atoi(match[1]); err == nil {
			minutes += mins
		}
	}

	return minutes
}

func (s *recipeImportService) extractYieldFromJSONLD(data map[string]interface{}) int {
	if yield, ok := data["recipeYield"]; ok {
		switch v := yield.(type) {
		case string:
			if num, err := strconv.Atoi(v); err == nil {
				return num
			}
		case float64:
			return int(v)
		case []interface{}:
			if len(v) > 0 {
				if str, ok := v[0].(string); ok {
					if num, err := strconv.Atoi(str); err == nil {
						return num
					}
				}
			}
		}
	}
	return 0
}

func (s *recipeImportService) extractIngredientsFromJSONLD(data map[string]interface{}) []models.RecipeIngredient {
	ingredients := []models.RecipeIngredient{}
	
	if ingredientList, ok := data["recipeIngredient"].([]interface{}); ok {
		for _, ingredient := range ingredientList {
			if ingredientStr, ok := ingredient.(string); ok {
				parsed := s.parseIngredientString(ingredientStr)
				ingredients = append(ingredients, parsed)
			}
		}
	}

	return ingredients
}

func (s *recipeImportService) parseIngredientString(ingredientStr string) models.RecipeIngredient {
	// Basic ingredient parsing - this could be much more sophisticated
	ingredient := models.RecipeIngredient{
		Name:     ingredientStr,
		Amount:   1.0,
		Unit:     "item",
		Category: "other",
	}

	// Try to extract amount and unit
	amountRegex := regexp.MustCompile(`^(\d+(?:\.\d+)?(?:\s+\d+/\d+)?)\s*([a-zA-Z]+)?\s+(.+)`)
	if match := amountRegex.FindStringSubmatch(strings.TrimSpace(ingredientStr)); len(match) > 3 {
		if amount, err := strconv.ParseFloat(match[1], 64); err == nil {
			ingredient.Amount = amount
		}
		if match[2] != "" {
			ingredient.Unit = match[2]
		}
		ingredient.Name = strings.TrimSpace(match[3])
	}

	return ingredient
}

func (s *recipeImportService) extractInstructionsFromJSONLD(data map[string]interface{}) []models.RecipeInstruction {
	instructions := []models.RecipeInstruction{}
	
	if instructionList, ok := data["recipeInstructions"].([]interface{}); ok {
		for i, instruction := range instructionList {
			var text string
			
			switch v := instruction.(type) {
			case string:
				text = v
			case map[string]interface{}:
				if t, ok := v["text"].(string); ok {
					text = t
				}
			}
			
			if text != "" {
				instructions = append(instructions, models.RecipeInstruction{
					StepNumber:  i + 1,
					Instruction: text,
				})
			}
		}
	}

	return instructions
}

func (s *recipeImportService) applyOverrides(recipe *models.CreateRecipeInput, overrides *models.CreateRecipeInput) {
	if overrides.Title != "" {
		recipe.Title = overrides.Title
	}
	if overrides.Description != nil {
		recipe.Description = overrides.Description
	}
	if overrides.PrepTime > 0 {
		recipe.PrepTime = overrides.PrepTime
	}
	if overrides.CookTime > 0 {
		recipe.CookTime = overrides.CookTime
	}
	if len(overrides.MealType) > 0 {
		recipe.MealType = overrides.MealType
	}
	if overrides.Complexity != "" {
		recipe.Complexity = overrides.Complexity
	}
	if overrides.CuisineType != nil {
		recipe.CuisineType = overrides.CuisineType
	}
	if overrides.Servings > 0 {
		recipe.Servings = overrides.Servings
	}
	if len(overrides.Ingredients) > 0 {
		recipe.Ingredients = overrides.Ingredients
	}
	if len(overrides.Instructions) > 0 {
		recipe.Instructions = overrides.Instructions
	}
	if len(overrides.DietaryLabels) > 0 {
		recipe.DietaryLabels = overrides.DietaryLabels
	}
	if overrides.ImageURL != nil {
		recipe.ImageURL = overrides.ImageURL
	}
}
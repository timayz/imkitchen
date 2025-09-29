use super::{ApiClient, ApiConfig, ApiCredentials, ApiError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NutritionData {
    pub ingredient_name: String,
    pub calories_per_100g: f64,
    pub protein_g: f64,
    pub carbs_g: f64,
    pub fat_g: f64,
    pub fiber_g: f64,
    pub sugar_g: f64,
    pub sodium_mg: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngredientLookupRequest {
    pub ingredient_name: String,
    pub quantity: Option<f64>,
    pub unit: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NutritionFactsResponse {
    pub nutrition_data: Vec<NutritionData>,
    pub total_calories: f64,
    pub total_protein: f64,
    pub total_carbs: f64,
    pub total_fat: f64,
}

/// Future implementation for nutrition data API integration
pub struct NutritionApiClient {
    client: ApiClient,
}

impl NutritionApiClient {
    pub fn new(config: ApiConfig, credentials: ApiCredentials) -> Result<Self, ApiError> {
        let client = ApiClient::new(config, credentials)?;
        Ok(Self { client })
    }

    /// Check if ingredient lookup is supported
    pub fn supports_ingredient_lookup(&self) -> bool {
        true
    }

    /// Check if nutrition facts calculation is supported
    pub fn supports_nutrition_facts(&self) -> bool {
        true
    }

    /// Future implementation: Look up nutrition data for a single ingredient
    pub async fn lookup_ingredient(
        &self,
        request: &IngredientLookupRequest,
    ) -> Result<NutritionData, ApiError> {
        let endpoint = "/v1/ingredients/lookup";
        let body = serde_json::to_string(request).map_err(|e| {
            ApiError::InvalidResponse(format!("Failed to serialize request: {}", e))
        })?;

        let response = self.client.post(endpoint, Some(body)).await?;

        let nutrition_data: NutritionData = serde_json::from_str(&response.body)
            .map_err(|e| ApiError::InvalidResponse(format!("Failed to parse response: {}", e)))?;

        Ok(nutrition_data)
    }

    /// Future implementation: Get nutrition facts for multiple ingredients
    pub async fn get_nutrition_facts(
        &self,
        ingredients: &[IngredientLookupRequest],
    ) -> Result<NutritionFactsResponse, ApiError> {
        let endpoint = "/v1/nutrition/facts";
        let body = serde_json::to_string(ingredients).map_err(|e| {
            ApiError::InvalidResponse(format!("Failed to serialize request: {}", e))
        })?;

        let response = self.client.post(endpoint, Some(body)).await?;

        let nutrition_facts: NutritionFactsResponse = serde_json::from_str(&response.body)
            .map_err(|e| ApiError::InvalidResponse(format!("Failed to parse response: {}", e)))?;

        Ok(nutrition_facts)
    }

    /// Future implementation: Search for ingredients by name
    pub async fn search_ingredients(
        &self,
        query: &str,
        limit: Option<u32>,
    ) -> Result<Vec<String>, ApiError> {
        let endpoint = format!(
            "/v1/ingredients/search?q={}&limit={}",
            urlencoding::encode(query),
            limit.unwrap_or(10)
        );

        let response = self.client.get(&endpoint).await?;

        let ingredients: Vec<String> = serde_json::from_str(&response.body)
            .map_err(|e| ApiError::InvalidResponse(format!("Failed to parse response: {}", e)))?;

        Ok(ingredients)
    }

    /// Health check for nutrition API
    pub async fn health_check(&self) -> Result<bool, ApiError> {
        self.client.health_check().await
    }
}

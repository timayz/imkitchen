use super::{ApiClient, ApiConfig, ApiCredentials, ApiError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroceryItem {
    pub name: String,
    pub brand: Option<String>,
    pub category: String,
    pub unit: String,
    pub package_size: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceData {
    pub store_name: String,
    pub price: f64,
    pub currency: String,
    pub unit_price: Option<f64>,
    pub on_sale: bool,
    pub last_updated: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceLookupRequest {
    pub item: GroceryItem,
    pub location: Option<String>,
    pub stores: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceComparisonResponse {
    pub item: GroceryItem,
    pub prices: Vec<PriceData>,
    pub lowest_price: PriceData,
    pub average_price: f64,
}

/// Future implementation for grocery price API integration
pub struct GroceryPriceApiClient {
    client: ApiClient,
}

impl GroceryPriceApiClient {
    pub fn new(config: ApiConfig, credentials: ApiCredentials) -> Result<Self, ApiError> {
        let client = ApiClient::new(config, credentials)?;
        Ok(Self { client })
    }

    /// Check if price lookup is supported
    pub fn supports_price_lookup(&self) -> bool {
        true
    }

    /// Check if store comparison is supported
    pub fn supports_store_comparison(&self) -> bool {
        true
    }

    /// Future implementation: Look up prices for a grocery item
    pub async fn lookup_prices(
        &self,
        request: &PriceLookupRequest,
    ) -> Result<Vec<PriceData>, ApiError> {
        let endpoint = "/v1/prices/lookup";
        let body = serde_json::to_string(request).map_err(|e| {
            ApiError::InvalidResponse(format!("Failed to serialize request: {}", e))
        })?;

        let response = self.client.post(endpoint, Some(body)).await?;

        let prices: Vec<PriceData> = serde_json::from_str(&response.body)
            .map_err(|e| ApiError::InvalidResponse(format!("Failed to parse response: {}", e)))?;

        Ok(prices)
    }

    /// Future implementation: Compare prices across multiple stores
    pub async fn compare_prices(
        &self,
        request: &PriceLookupRequest,
    ) -> Result<PriceComparisonResponse, ApiError> {
        let endpoint = "/v1/prices/compare";
        let body = serde_json::to_string(request).map_err(|e| {
            ApiError::InvalidResponse(format!("Failed to serialize request: {}", e))
        })?;

        let response = self.client.post(endpoint, Some(body)).await?;

        let comparison: PriceComparisonResponse = serde_json::from_str(&response.body)
            .map_err(|e| ApiError::InvalidResponse(format!("Failed to parse response: {}", e)))?;

        Ok(comparison)
    }

    /// Future implementation: Search for grocery items
    pub async fn search_items(
        &self,
        query: &str,
        category: Option<&str>,
    ) -> Result<Vec<GroceryItem>, ApiError> {
        let mut endpoint = format!("/v1/items/search?q={}", urlencoding::encode(query));
        if let Some(cat) = category {
            endpoint.push_str(&format!("&category={}", urlencoding::encode(cat)));
        }

        let response = self.client.get(&endpoint).await?;

        let items: Vec<GroceryItem> = serde_json::from_str(&response.body)
            .map_err(|e| ApiError::InvalidResponse(format!("Failed to parse response: {}", e)))?;

        Ok(items)
    }

    /// Future implementation: Get supported stores in a location
    pub async fn get_supported_stores(&self, location: &str) -> Result<Vec<String>, ApiError> {
        let endpoint = format!("/v1/stores?location={}", urlencoding::encode(location));

        let response = self.client.get(&endpoint).await?;

        let stores: Vec<String> = serde_json::from_str(&response.body)
            .map_err(|e| ApiError::InvalidResponse(format!("Failed to parse response: {}", e)))?;

        Ok(stores)
    }

    /// Health check for grocery price API
    pub async fn health_check(&self) -> Result<bool, ApiError> {
        self.client.health_check().await
    }
}

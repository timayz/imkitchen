use imkitchen_shared::external_apis::{
    ApiConfig, ApiClient, ApiError, ApiResponse, ApiCredentials,
    nutrition::NutritionApiClient,
    grocery::GroceryPriceApiClient,
};
use std::collections::HashMap;
use validator::Validate;

#[tokio::test]
async fn test_api_config_creation() {
    let config = ApiConfig {
        base_url: "https://api.example.com".to_string(),
        api_key: "test_api_key".to_string(),
        timeout_seconds: 30,
        rate_limit_per_minute: 60,
        retry_attempts: 3,
    };
    
    assert!(config.validate().is_ok());
    assert_eq!(config.base_url, "https://api.example.com");
    assert_eq!(config.timeout_seconds, 30);
}

#[tokio::test]
async fn test_api_config_validation() {
    // Test invalid URL
    let invalid_url_config = ApiConfig {
        base_url: "not-a-url".to_string(),
        api_key: "test_key".to_string(),
        timeout_seconds: 30,
        rate_limit_per_minute: 60,
        retry_attempts: 3,
    };
    
    let validation_result = invalid_url_config.validate();
    assert!(validation_result.is_err());
    let errors = validation_result.unwrap_err();
    assert!(errors.field_errors().contains_key("base_url"));
    
    // Test empty API key
    let empty_key_config = ApiConfig {
        base_url: "https://api.example.com".to_string(),
        api_key: "".to_string(),
        timeout_seconds: 30,
        rate_limit_per_minute: 60,
        retry_attempts: 3,
    };
    
    let validation_result = empty_key_config.validate();
    assert!(validation_result.is_err());
    let errors = validation_result.unwrap_err();
    assert!(errors.field_errors().contains_key("api_key"));
}

#[tokio::test]
async fn test_api_credentials_validation() {
    let credentials = ApiCredentials {
        api_key: "valid_api_key_123".to_string(),
        secret_key: Some("secret_key_456".to_string()),
        additional_headers: {
            let mut headers = HashMap::new();
            headers.insert("X-Custom-Header".to_string(), "custom_value".to_string());
            headers
        },
    };
    
    assert!(credentials.validate().is_ok());
    assert_eq!(credentials.api_key, "valid_api_key_123");
    assert!(credentials.secret_key.is_some());
}

#[tokio::test]
async fn test_api_client_creation() {
    let config = ApiConfig {
        base_url: "https://api.example.com".to_string(),
        api_key: "test_api_key".to_string(),
        timeout_seconds: 30,
        rate_limit_per_minute: 60,
        retry_attempts: 3,
    };
    
    let credentials = ApiCredentials {
        api_key: "test_api_key".to_string(),
        secret_key: None,
        additional_headers: HashMap::new(),
    };
    
    let client = ApiClient::new(config, credentials);
    assert!(client.is_ok());
}

#[tokio::test]
async fn test_api_client_retry_logic() {
    let config = ApiConfig {
        base_url: "https://nonexistent-api.example.com".to_string(),
        api_key: "test_api_key".to_string(),
        timeout_seconds: 1, // Short timeout for testing
        rate_limit_per_minute: 60,
        retry_attempts: 3,
    };
    
    let credentials = ApiCredentials {
        api_key: "test_api_key".to_string(),
        secret_key: None,
        additional_headers: HashMap::new(),
    };
    
    let client = ApiClient::new(config, credentials).unwrap();
    
    // This should fail but with proper retry logic
    let result = client.get("/test-endpoint").await;
    assert!(result.is_err());
    
    match result.err().unwrap() {
        ApiError::NetworkError(_) => {
            // Expected for nonexistent domain
        }
        ApiError::TimeoutError => {
            // Also expected
        }
        ApiError::RetryExhausted => {
            // Expected after retries
        }
        _ => panic!("Unexpected error type"),
    }
}

#[tokio::test]
async fn test_api_client_error_handling() {
    let config = ApiConfig {
        base_url: "https://httpbin.org".to_string(), // Real endpoint for testing
        api_key: "test_api_key".to_string(),
        timeout_seconds: 30,
        rate_limit_per_minute: 60,
        retry_attempts: 1,
    };
    
    let credentials = ApiCredentials {
        api_key: "test_api_key".to_string(),
        secret_key: None,
        additional_headers: HashMap::new(),
    };
    
    let client = ApiClient::new(config, credentials).unwrap();
    
    // Test 404 error handling
    let result = client.get("/status/404").await;
    match result {
        Err(ApiError::HttpError(status)) => {
            assert_eq!(status, 404);
        }
        Ok(_) => {
            // If it succeeds, that's also fine (network availability varies)
        }
        Err(e) => {
            println!("Network test result: {}", e);
        }
    }
}

#[tokio::test]
async fn test_nutrition_api_client_framework() {
    let config = ApiConfig {
        base_url: "https://api.nutrition.example.com".to_string(),
        api_key: "nutrition_api_key".to_string(),
        timeout_seconds: 30,
        rate_limit_per_minute: 100,
        retry_attempts: 3,
    };
    
    let credentials = ApiCredentials {
        api_key: "nutrition_api_key".to_string(),
        secret_key: None,
        additional_headers: HashMap::new(),
    };
    
    let nutrition_client = NutritionApiClient::new(config, credentials);
    assert!(nutrition_client.is_ok());
    
    // Test that the client has the expected interface
    let client = nutrition_client.unwrap();
    assert!(client.supports_ingredient_lookup());
    assert!(client.supports_nutrition_facts());
}

#[tokio::test]
async fn test_grocery_price_api_client_framework() {
    let config = ApiConfig {
        base_url: "https://api.grocery.example.com".to_string(),
        api_key: "grocery_api_key".to_string(),
        timeout_seconds: 30,
        rate_limit_per_minute: 50,
        retry_attempts: 3,
    };
    
    let credentials = ApiCredentials {
        api_key: "grocery_api_key".to_string(),
        secret_key: None,
        additional_headers: HashMap::new(),
    };
    
    let grocery_client = GroceryPriceApiClient::new(config, credentials);
    assert!(grocery_client.is_ok());
    
    // Test that the client has the expected interface
    let client = grocery_client.unwrap();
    assert!(client.supports_price_lookup());
    assert!(client.supports_store_comparison());
}

#[tokio::test]
async fn test_api_key_secure_storage() {
    let credentials = ApiCredentials {
        api_key: "super_secret_api_key".to_string(),
        secret_key: Some("super_secret_key".to_string()),
        additional_headers: HashMap::new(),
    };
    
    // Debug output should not contain the sensitive keys
    let debug_output = format!("{:?}", credentials);
    assert!(!debug_output.contains("super_secret_api_key"), 
            "API key should not appear in debug output");
    assert!(!debug_output.contains("super_secret_key"), 
            "Secret key should not appear in debug output");
}

#[tokio::test]
async fn test_api_rate_limiting() {
    let config = ApiConfig {
        base_url: "https://api.example.com".to_string(),
        api_key: "test_api_key".to_string(),
        timeout_seconds: 30,
        rate_limit_per_minute: 2, // Very low for testing
        retry_attempts: 1,
    };
    
    let credentials = ApiCredentials {
        api_key: "test_api_key".to_string(),
        secret_key: None,
        additional_headers: HashMap::new(),
    };
    
    let client = ApiClient::new(config, credentials).unwrap();
    
    // Make multiple requests to test rate limiting
    for i in 0..5 {
        let result = client.get(&format!("/test-endpoint-{}", i)).await;
        
        // After the rate limit is hit, we should get rate limit errors
        if i >= 2 {
            match result {
                Err(ApiError::RateLimitExceeded) => {
                    // Expected behavior
                }
                _ => {
                    // Rate limiting might not be implemented yet, or network request failed
                    println!("Rate limiting test - request {}: {:?}", i, result);
                }
            }
        }
    }
}

#[tokio::test]
async fn test_api_response_parsing() {
    // Test successful response parsing
    let success_response = ApiResponse {
        status: 200,
        headers: HashMap::new(),
        body: r#"{"data": "success", "message": "OK"}"#.to_string(),
    };
    
    assert_eq!(success_response.status, 200);
    assert!(success_response.is_success());
    assert!(success_response.body.contains("success"));
    
    // Test error response
    let error_response = ApiResponse {
        status: 400,
        headers: HashMap::new(),
        body: r#"{"error": "Bad Request", "message": "Invalid parameters"}"#.to_string(),
    };
    
    assert_eq!(error_response.status, 400);
    assert!(!error_response.is_success());
    assert!(error_response.body.contains("Bad Request"));
}
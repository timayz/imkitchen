# External APIs

## Spoonacular Recipe API

- **Purpose:** Recipe database, nutrition information, ingredient recognition
- **Documentation:** https://spoonacular.com/food-api/docs
- **Base URL(s):** https://api.spoonacular.com/
- **Authentication:** API key in query parameter or header
- **Rate Limits:** 150 requests/day (free), 500+ (paid plans)

**Key Endpoints Used:**
- `GET /recipes/complexSearch` - Recipe search with ingredient filters
- `GET /recipes/{id}/information` - Detailed recipe information
- `GET /recipes/findByIngredients` - Recipes based on available ingredients
- `GET /food/ingredients/search` - Ingredient search and autocomplete

**Integration Notes:** Primary recipe source with fallback to user-generated content; caching essential due to rate limits; nutrition data integration for meal planning

**Error Recovery Strategy:**
- **Rate Limit Exceeded:** Switch to cached recipes and user-generated content; display rate limit notice with retry timing
- **Service Unavailable:** Fallback to OpenFoodFacts data and local recipe database; maintain core search functionality
- **API Key Invalid:** Graceful degradation to manual recipe entry with clear user messaging about reduced functionality
- **Network Timeout:** Use cached recipe data with offline indicators; retry with exponential backoff

## OpenFoodFacts API

- **Purpose:** Product information, barcode scanning, nutritional data
- **Documentation:** https://openfoodfacts.github.io/openfoodfacts-server/api/
- **Base URL(s):** https://world.openfoodfacts.org/api/v0/
- **Authentication:** None required (open data)
- **Rate Limits:** None specified (reasonable use expected)

**Key Endpoints Used:**
- `GET /product/{barcode}.json` - Product information by barcode
- `GET /cgi/search.pl` - Product search functionality
- `GET /api/v0/product/{barcode}` - Detailed product data

**Integration Notes:** Used for barcode scanning functionality in inventory management; supplement to manual ingredient entry; multilingual product data available

**Error Recovery Strategy:**
- **Service Unavailable:** Fallback to manual ingredient entry with barcode recognition disabled temporarily
- **Product Not Found:** Provide manual entry form pre-populated with barcode for future database contribution
- **Network Issues:** Cache successful barcode scans locally; sync when connectivity restored
- **Invalid Barcode:** Clear error messaging with option to manually enter product information

## Web Speech API

- **Purpose:** Voice recognition and speech synthesis for cooking mode
- **Documentation:** https://developer.mozilla.org/en-US/docs/Web/API/Web_Speech_API
- **Base URL(s):** Browser-native API (no external calls)
- **Authentication:** User permission required
- **Rate Limits:** Browser-dependent

**Key Endpoints Used:**
- `SpeechRecognition` - Voice command recognition
- `SpeechSynthesis` - Text-to-speech for cooking instructions
- `speechSynthesis.speak()` - Read cooking steps aloud

**Integration Notes:** Primary voice interface with cloud service fallback; offline capability essential for cooking mode; multiple language support required

**Error Recovery Strategy:**
- **Microphone Permission Denied:** Fallback to touch/click interface with clear instructions for enabling voice
- **Speech Recognition Failure:** Provide manual text input alternative; retry voice recognition with user feedback
- **Browser Compatibility:** Progressive enhancement with feature detection; manual controls always available
- **Noisy Environment:** Implement noise filtering; provide visual feedback for recognized commands; manual override options

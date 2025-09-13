# External APIs

## Web Push API

- **Purpose:** Deliver real-time cooking notifications and timer alerts to user devices
- **Documentation:** https://developer.mozilla.org/en-US/docs/Web/API/Push_API
- **Base URL(s):** FCM: https://fcm.googleapis.com/fcm/send
- **Authentication:** Voluntary Application Server Identification (VAPID) keys
- **Rate Limits:** 1000 notifications per app per user per hour

**Key Endpoints Used:**
- `POST /fcm/send` - Send push notification to specific user subscription

**Integration Notes:** Requires service worker registration on frontend, VAPID key generation for authentication, graceful fallback to WebSocket notifications if push not available

## Recipe Website APIs

- **Purpose:** Import recipe data from popular cooking websites with structured data
- **Documentation:** Schema.org Recipe microdata specification
- **Base URL(s):** Various recipe sites (AllRecipes, Food Network, etc.)
- **Authentication:** None required for public recipes
- **Rate Limits:** Respect robots.txt and implement polite crawling delays

**Key Endpoints Used:**
- Various recipe URLs with JSON-LD or microdata markup

**Integration Notes:** Implement HTML parsing fallback for sites without structured data, respect copyright and fair use policies, cache parsed results to minimize requests

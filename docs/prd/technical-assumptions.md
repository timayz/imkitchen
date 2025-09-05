# Technical Assumptions

## Repository Structure: Monorepo
Based on the brief's indication for shared component libraries and the need to support mobile app, web interface, and API services efficiently.

## Service Architecture
**Microservices within Monorepo** - The brief indicates microservices approach with dedicated services for scheduling engine, notification system, and community features, while maintaining monorepo benefits for shared libraries and coordinated deployment.

## Testing Requirements
**Unit + Integration Testing** - Given the complexity of the scheduling algorithms and the critical nature of meal planning automation, comprehensive testing is essential for user trust. Integration testing will be crucial for validating the "Fill My Week" workflow end-to-end.

## Additional Technical Assumptions and Requests

**Frontend Framework:** 
- Lynx-js for cross-platform mobile development (primary user interface)
- TwinSpark integrated with Rust backend for admin web UI only
- Mobile-first responsive design targeting iOS 14+ and Android 8+

**Backend Technology:**
- Rust-based API services with integrated admin web UI using TwinSpark
- Sub-2-second meal plan generation requirement drives high-performance backend choice

**Database & Caching:**
- PostgreSQL for relational recipe and user data storage
- Redis caching layer for meal plan generation optimization

**Infrastructure & Hosting:**
- Cloud-native deployment supporting horizontal scaling for community features

**Security & Compliance:**
- User data privacy compliance and GDPR readiness for international expansion
- Secure payment processing integration for premium subscriptions

**Push Notification Strategy:** Native platform notifications (iOS UserNotifications framework, Android NotificationManager) for maximum reliability and integration with device notification settings.

**Recipe Data Import:** Dual approach supporting web scraping for popular recipe sites and manual entry for personal recipes. Web scraping will target major recipe platforms (AllRecipes, Food Network, etc.) with fallback to manual entry for unsupported sources.

**Offline Capability:** Sync-when-connected model - users can access downloaded recipes and current meal plans offline, with data synchronization occurring automatically when internet connection is available.
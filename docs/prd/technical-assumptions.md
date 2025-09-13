# Technical Assumptions

## Repository Structure: Monorepo

Single repository containing frontend, backend, shared types, and deployment configurations. This supports rapid development by a single developer while maintaining type safety and shared utilities across the stack.

## Service Architecture

**Monolith** - Single backend service using Rust with axum framework (0.8+) for rapid development and deployment simplicity. The service will handle authentication, recipe management, meal planning logic, and API endpoints. Database layer uses PostgreSQL (17+) for structured data with Redis (8.2+) for caching and session management.

## Testing Requirements

**Unit + Integration Testing** - Comprehensive unit tests for business logic, API integration tests for critical user flows, and manual testing protocols for UI/UX validation. Focus on testing timing calculations, recipe parsing accuracy, and meal planning algorithms.

## Additional Technical Assumptions and Requests

- **Frontend Technology:** Askama 0.14+ templating with twinspark-js for UI reactivity to minimize JavaScript complexity
- **Database Design:** PostgreSQL with consideration for vector database integration for recipe similarity matching
- **Deployment:** Docker-based containerization for consistent deployment across environments
- **Offline Support:** Service Worker implementation for offline recipe access and basic cooking guidance
- **Performance:** Redis caching layer for frequently accessed recipes and user preferences
- **Security:** JWT-based authentication with secure password hashing and session management
- **Notifications:** Web Push API integration for timing alerts and cooking reminders

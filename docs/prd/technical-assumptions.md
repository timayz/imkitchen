# Technical Assumptions

## Repository Structure: Modular Monorepo with Bounded Context Crates

Single Rust workspace containing multiple crates organized by bounded contexts and shared utilities. Each service/bounded context has its own dedicated crate providing strong boundaries, independent evolution, and clear dependency management. This approach provides type safety across the full stack while maintaining proper domain separation through workspace crates.

## Service Architecture: Domain-Driven CRUD with Bounded Contexts

**Domain-Driven Design (DDD):** Modular monolithic architecture with each bounded context as a separate Rust crate, deployed as a single binary. Core bounded context crates include:
- **imkitchen-user-crate:** Authentication, user profiles, and account management (separate crate)
- **imkitchen-recipe-crate:** Recipe management, ratings, collections, and community features (separate crate)
- **imkitchen-meal-planning-crate:** Intelligent scheduling, optimization algorithms, and weekly planning (separate crate)
- **imkitchen-shopping-crate:** Shopping lists, ingredient management, and preparation workflows (separate crate)
- **imkitchen-notification-crate:** Push notifications, reminders, and communication systems (separate crate)
- **imkitchen-shared-crate:** Common domain types, events, and utilities shared across contexts
- **imkitchen-web-crate:** Axum web server, Askama templates, and HTTP handlers

**Simple CRUD Operations:** Direct database operations with domain model validation:
- **Commands:** State-changing operations implemented as direct SQLx database operations with domain validation
- **Services:** Domain services encapsulate business logic and coordinate database operations
- **Queries:** Read operations use optimized SQLx queries with prepared statements for performance
- **Database:** SQLite with connection pooling for reliable, embedded data persistence
- **Domain Validation:** Business rules enforced through Rust domain models and validator derive macros
- **Transactions:** Database transactions ensure data consistency for complex multi-table operations
- **Migrations:** SQLx migration system for reliable schema evolution and deployment

Technology stack organized by crate:

**CLI Binary (imkitchen):**
- **CLI Framework:** clap 4.5+ for command-line argument parsing and subcommands
- **Database Migrations:** SQLx 0.8+ CLI integration for migrate commands
- **Server Management:** Integration with imkitchen-web library for web server startup

**Shared Dependencies (All Crates):**
- **Database:** SQLx 0.8+ for type-safe database operations with connection pooling and migrations
- **Serialization:** serde 1.0+ for JSON serialization with backward compatibility and API integration
- **Input Validation:** validator 0.20+ for comprehensive input validation with derive macros and custom validators
- **Monitoring:** Tracing 0.1+ for structured logging, performance tracking, and domain observability
- **Configuration:** config 0.15+ for secure secrets and environment variable management

**Web Library Crate (imkitchen-web):**
- **Web Framework:** Axum 0.8+ for high-performance async HTTP handling (library implementation)
- **Templates:** Askama 0.14+ for type-safe server-side HTML rendering with compile-time template validation
- **UI Framework:** Tailwind CSS 3.0+ for utility-first styling directly in Askama templates
- **UI Reactivity:** twinspark-js for declarative HTML-driven interactivity with zero custom JavaScript
- **Server Library:** Exports server configuration, routing, and startup functions for CLI binary
- **JavaScript-Free Design:** All interactions handled via TwinSpark HTML attributes and server-side rendering
- **CSS Build Process:** Tailwind CLI for CSS compilation with Askama template scanning

**Domain Crates (Each Bounded Context):**
- **Database Operations:** SQLx 0.8+ with SQLite3 backend for direct CRUD operations and query optimization per context
- **Domain Services:** Context-specific business logic services with validated input/output operations
- **Domain Modeling:** Strong typing with Rust enums, structs, and domain-specific value objects per context

**Infrastructure Crates:**
- **Email Service:** lettre 0.11+ for SMTP integration (imkitchen-notification-crate)
- **Internationalization:** rust-i18n for multi-language support (imkitchen-shared-crate)

**Architecture Patterns:** 
- **DDD + Direct CRUD:** Domain-driven design with direct database operations and domain validation for robust state management
- **Askama + TwinSpark Rendering:** Type-safe server-side HTML templates with declarative interactivity eliminates API and JavaScript complexity
- **Request Flow:** HTML forms → Axum handlers → domain validation → database operations → Askama templates → TwinSpark fragments
- **Validation Flow:** HTML forms → validator validation → domain logic → database persistence → Askama templates → TwinSpark error fragments
- **Query Flow:** Read requests → SQLx queries → domain models → Askama templates → server-rendered HTML → TwinSpark declarative interactions
- **JavaScript-Minimal Design:** All interactions through TwinSpark HTML attributes (ts-req, ts-trigger, ts-target) without custom JavaScript
- **Template Safety:** Compile-time HTML template validation with Askama prevents runtime template errors
- **Transaction-Based:** Database transactions ensure consistency for complex operations with proper rollback handling

## Testing Requirements: Test-Driven Development (TDD)

**MANDATORY TDD Approach:** All development must follow strict Test-Driven Development methodology:
1. **RED:** Write failing tests first that define expected behavior
2. **GREEN:** Write minimal code to make tests pass
3. **REFACTOR:** Improve code while maintaining all tests passing

Comprehensive testing pyramid including:
- **Unit Tests:** Individual function and module testing with >90% code coverage (increased from 80%)
- **Integration Tests:** Database operations, HTTP endpoints, and cross-module communication
- **End-to-End Tests:** Critical user journeys including meal planning, recipe management, and community features
- **Performance Tests:** Load testing for optimization algorithms and concurrent user scenarios
- **Security Tests:** Authentication flows, input validation, and data protection verification

**TDD Enforcement:**
- No production code written without corresponding failing test first
- All commits must include tests that validate the implemented functionality
- CI/CD pipeline blocks merges if test coverage falls below 90%
- Code reviews must verify TDD methodology was followed

## Additional Technical Assumptions and Requests

- **TwinSpark Architecture:** Server-side HTML rendering with progressive enhancement eliminates API complexity
- **Progressive Web App (PWA):** Installable app experience with offline functionality and push notification support
- **Mobile-First Performance:** Server-rendered HTML with selective JavaScript enhancement for optimal mobile performance
- **Container Deployment:** Docker containerization with Kubernetes orchestration support for scalable cloud deployment
- **OWASP Security Compliance:** Authentication system following OWASP Authentication Cheat Sheet guidelines
- **Database Migrations:** Automated schema migration system for safe production deployments
- **CI/CD Pipeline:** Automated testing, building, and deployment pipeline with quality gates
- **Monitoring Integration:** Application performance monitoring and error tracking for production observability
- **Form-Based Interactions:** All user interactions use HTML forms with server-side processing and fragment updates
- **No JSON APIs:** Eliminates client-server serialization complexity through direct HTML template rendering
- **Mandatory TDD:** All development follows strict Test-Driven Development with Red-Green-Refactor cycle enforcement
- **Test Coverage:** Minimum 90% code coverage with comprehensive unit, integration, and end-to-end test suites
- **Test-First Culture:** No production code commits allowed without corresponding failing tests written first
- **Continuous Testing:** cargo-watch integration for real-time test execution during development
- **Domain Testing:** Domain model tests verify business rules, validation logic, and service behavior
- **Database Testing:** Direct SQLx tests for CRUD operations, query performance, and transaction handling
- **Service Testing:** Domain service tests covering business logic and database integration patterns
- **Handler Testing:** Axum handler tests for HTTP endpoints, validation, and response generation
- **Integration Testing:** End-to-end tests covering request processing, database persistence, and template rendering
- **Bounded Context Integration:** Clear service interfaces between contexts with shared type definitions
- **Ubiquitous Language:** Shared domain vocabulary between business stakeholders and development team
- **Database Schema Versioning:** Backward-compatible schema evolution with SQLx migration system
- **Data Migration Support:** Database migration scripts for schema changes and data transformations
- **Data Persistence:** SQLx handles database operations, connection pooling, and transaction management
- **Request Processing:** Direct handler processing with middleware for validation, logging, and error handling
- **Input Validation:** Comprehensive validation using validator crate with domain-specific validation rules
- **Validation Integration:** Validation integrated into value objects, domain services, and HTTP handlers
- **Askama Template Testing:** All HTML templates tested with compile-time validation and runtime integration tests
- **TwinSpark Pattern:** Server-rendered Askama templates with declarative HTML attributes for JavaScript-free interactivity
- **Tailwind CSS Integration:** Utility-first styling directly in Askama templates with responsive design classes
- **Minimal JavaScript:** Zero custom JavaScript code, all interactions via TwinSpark HTML attributes and server responses
- **Declarative Interactions:** Form submissions, content updates, and dynamic behavior through HTML data attributes
- **CSS Build Pipeline:** Tailwind CSS compilation with template scanning for optimal bundle size and purging
- **Template Organization:** Modular template structure with shared layouts, components, and fragments for maintainability
- **Crate Organization:** Each bounded context implemented as separate Rust crate with clear boundaries and dependencies
- **Database Integration:** All domain operations and queries implemented using SQLx with type-safe database access per crate
- **Database Backend:** SQLx configured with SQLite backend for embedded, zero-dependency data persistence per context
- **Handler Middleware:** Axum middleware for validation, logging, authentication, and error handling per crate
- **Service Reliability:** Reliable operation processing with proper error handling and transaction management within each crate
- **Data Consistency:** Strong consistency managed through database transactions with proper error recovery per context
- **Inter-Crate Communication:** Bounded contexts communicate through well-defined service interfaces and shared types
- **Dependency Management:** Clear crate dependencies with shared-crate for common types and anti-corruption layers
- **CLI Architecture:** Root binary orchestrates all operations through library crates, no direct main.rs in domain crates
- **Command Structure:** Hierarchical CLI commands with proper error handling and user feedback
- **Migration Management:** Database schema versioning and migration commands integrated into CLI
- **Server Lifecycle:** Web server startup, shutdown, and configuration management through CLI commands
- **JavaScript-Free Interface:** All user interactions handled via TwinSpark HTML attributes without custom JavaScript development
- **TwinSpark Patterns:** Form submissions (`ts-req`), live search (`ts-trigger="keyup"`), content updates (`ts-target`), confirmations (`ts-confirm`)
- **Declarative Interactions:** Dynamic content loading, form validation, and UI updates through HTML data attributes only
- **Validation Architecture:** Synchronous validation (format, length, required) in Axum handlers; asynchronous validation (database checks, external APIs) via background services
- **Performance Optimization:** Direct validation provides immediate feedback; background services handle time-consuming operations without blocking UI
- **Process Separation:** Simple CRUD operations use direct database access; complex business processes use dedicated service handlers
- **Tailwind CSS Integration:** Utility-first styling with responsive design, dark mode support, and kitchen-optimized color palette
- **Template-First Styling:** All CSS classes embedded directly in Askama templates for component co-location and maintainability
- **Design System:** Consistent Tailwind utilities for spacing, typography, colors, and responsive breakpoints across all components
- **Kitchen-Optimized Design:** Large touch targets (`min-h-12 min-w-12`), high contrast colors (`text-gray-900 bg-white`), readable typography (`text-lg md:text-xl`)
- **Responsive Grid System:** Mobile-first design with `grid`, `flex`, and responsive modifiers for all screen sizes from phone to desktop
- **Custom Color Palette:** Extended Tailwind config with food-inspired colors (tomato-red, olive-green, butter-yellow) for kitchen context

# Technical Assumptions

## Repository Structure: Modular Monorepo with Bounded Context Crates

Single Rust workspace containing multiple crates organized by bounded contexts and shared utilities. Each service/bounded context has its own dedicated crate providing strong boundaries, independent evolution, and clear dependency management. This approach provides type safety across the full stack while maintaining proper domain separation through workspace crates.

## Service Architecture: DDD + CQRS + Event Sourcing

**Domain-Driven Design (DDD):** Modular monolithic architecture with each bounded context as a separate Rust crate, deployed as a single binary. Core bounded context crates include:
- **imkitchen-user-crate:** Authentication, user profiles, and account management (separate crate)
- **imkitchen-recipe-crate:** Recipe management, ratings, collections, and community features (separate crate)
- **imkitchen-meal-planning-crate:** Intelligent scheduling, optimization algorithms, and weekly planning (separate crate)
- **imkitchen-shopping-crate:** Shopping lists, ingredient management, and preparation workflows (separate crate)
- **imkitchen-notification-crate:** Push notifications, reminders, and communication systems (separate crate)
- **imkitchen-shared-crate:** Common domain types, events, and utilities shared across contexts
- **imkitchen-web-crate:** Axum web server, Askama templates, and HTTP handlers

**CQRS (Command Query Responsibility Segregation):** Separate command and query models with Evento-driven data flows:
- **Commands:** State-changing operations implemented as Evento commands that generate domain events
- **Command Handlers:** Evento command handlers validate business rules and emit events to event store
- **Queries:** Read operations use Evento-maintained projection views for fast data retrieval
- **Event Store:** Evento-managed single source of truth containing all domain events in chronological order
- **Projections:** Materialized views built from events using Evento projection builders, optimized for specific query patterns
- **Event Bus:** Evento event bus ensures reliable event delivery between bounded contexts

**Event Sourcing (ES):** All state changes captured as immutable domain events using Evento:
- **Evento Event Store:** SQLite-based event storage with automatic serialization, versioning, and replay capabilities
- **Domain Events:** RecipeCreated, MealPlanned, UserRegistered, ShoppingListGenerated implemented as Evento events
- **Event Handlers:** Evento event handlers process events to update projections and trigger side effects with guaranteed delivery
- **Event Streams:** Organized by aggregate ID with Evento stream management and concurrency control
- **Snapshots:** Periodic aggregate snapshots managed by Evento for performance optimization and fast aggregate reconstruction

Technology stack organized by crate:

**CLI Binary (imkitchen):**
- **CLI Framework:** clap 4.5+ for command-line argument parsing and subcommands
- **Database Migrations:** SQLx 0.8+ CLI integration for migrate commands
- **Server Management:** Integration with imkitchen-web library for web server startup

**Shared Dependencies (All Crates):**
- **Event System:** Evento 1.1+ for domain event publishing, handling, projection building, and event store management
- **Serialization:** serde 1.0+ for event serialization with backward compatibility and versioning
- **Input Validation:** validator 0.20+ for comprehensive input validation with derive macros and custom validators
- **Monitoring:** Tracing 0.1+ for structured logging, event tracking, and domain observability
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
- **Event Store:** Evento 1.1+ with SQLite3 backend for event sourcing and projection storage per context
- **CQRS Framework:** Evento 1.1+ based implementation with context-specific command/query handlers
- **Domain Modeling:** Strong typing with Rust enums, structs, and domain-specific value objects per context

**Infrastructure Crates:**
- **Email Service:** lettre 0.11+ for SMTP integration (imkitchen-notification-crate)
- **Internationalization:** rust-i18n for multi-language support (imkitchen-shared-crate)

**Architecture Patterns:** 
- **DDD + CQRS + ES:** Domain-driven design with command-query separation and event sourcing for robust state management
- **Askama + TwinSpark Rendering:** Type-safe server-side HTML templates with declarative interactivity eliminates API and JavaScript complexity
- **Command Flow (Async/Long Processes):** HTML forms → Evento command handlers → domain aggregates → events → projections → Askama templates → TwinSpark fragments
- **Validation Flow (Sync):** HTML forms → direct validator validation → Askama templates → TwinSpark error fragments (no Evento for simple validation)
- **Query Flow:** Read requests → projection queries → Askama templates → server-rendered HTML → TwinSpark declarative interactions
- **JavaScript-Minimal Design:** All interactions through TwinSpark HTML attributes (ts-req, ts-trigger, ts-target) without custom JavaScript
- **Template Safety:** Compile-time HTML template validation with Askama prevents runtime template errors
- **Event-Driven:** All state changes flow through domain events, enabling audit trails, replay, and temporal queries

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
- **DDD Testing:** Domain model tests verify business rules, aggregate behavior, and Evento event generation
- **CQRS Testing:** Separate test suites for Evento command handlers, query handlers, and projection builders
- **Event Sourcing Testing:** Evento event store tests, event replay tests, and projection consistency validation
- **Evento Testing:** Command bus tests, event handler tests, projection builder tests, and event stream tests
- **Integration Testing:** End-to-end tests covering Evento command processing, event propagation, and projection updates
- **Bounded Context Integration:** Anti-corruption layers between contexts with domain event communication
- **Ubiquitous Language:** Shared domain vocabulary between business stakeholders and development team
- **Evento Schema Versioning:** Backward-compatible event evolution with Evento's upcasting support for schema migrations
- **Evento Projection Rebuilding:** Rebuild all projections from events using Evento's replay capabilities for schema changes
- **Event Persistence:** Evento handles event persistence, serialization, and deserialization with SQLite backend
- **Command Processing:** Evento command bus with middleware for validation, logging, and error handling
- **Input Validation:** Comprehensive validation using validator crate with domain-specific validation rules
- **Validation Integration:** Validation integrated into value objects, CQRS commands, and domain services
- **Askama Template Testing:** All HTML templates tested with compile-time validation and runtime integration tests
- **TwinSpark Pattern:** Server-rendered Askama templates with declarative HTML attributes for JavaScript-free interactivity
- **Tailwind CSS Integration:** Utility-first styling directly in Askama templates with responsive design classes
- **Minimal JavaScript:** Zero custom JavaScript code, all interactions via TwinSpark HTML attributes and server responses
- **Declarative Interactions:** Form submissions, content updates, and dynamic behavior through HTML data attributes
- **CSS Build Pipeline:** Tailwind CSS compilation with template scanning for optimal bundle size and purging
- **Template Organization:** Modular template structure with shared layouts, components, and fragments for maintainability
- **Crate Organization:** Each bounded context implemented as separate Rust crate with clear boundaries and dependencies
- **Evento Integration:** All domain events, commands, queries, and projections implemented using Evento traits per crate
- **Event Store Backend:** Evento configured with SQLite backend for embedded, zero-dependency event persistence per context
- **Command Bus Middleware:** Evento command bus with validation, logging, authentication, and error handling middleware per crate
- **Event Handlers:** Reliable event processing with Evento's guaranteed delivery and retry mechanisms within each crate
- **Projection Consistency:** Eventual consistency managed by Evento with projection rebuilding and error recovery per context
- **Inter-Crate Communication:** Bounded contexts communicate exclusively through Evento domain events
- **Dependency Management:** Clear crate dependencies with shared-crate for common types and anti-corruption layers
- **CLI Architecture:** Root binary orchestrates all operations through library crates, no direct main.rs in domain crates
- **Command Structure:** Hierarchical CLI commands with proper error handling and user feedback
- **Migration Management:** Database schema versioning and migration commands integrated into CLI
- **Server Lifecycle:** Web server startup, shutdown, and configuration management through CLI commands
- **JavaScript-Free Interface:** All user interactions handled via TwinSpark HTML attributes without custom JavaScript development
- **TwinSpark Patterns:** Form submissions (`ts-req`), live search (`ts-trigger="keyup"`), content updates (`ts-target`), confirmations (`ts-confirm`)
- **Declarative Interactions:** Dynamic content loading, form validation, and UI updates through HTML data attributes only
- **Validation Architecture:** Synchronous validation (format, length, required) in Axum handlers; asynchronous validation (database checks, external APIs) via Evento
- **Performance Optimization:** Direct validation provides immediate feedback; Evento commands handle time-consuming operations without blocking UI
- **Process Separation:** Simple CRUD operations bypass Evento; complex business processes and external API calls use Evento command handlers
- **Tailwind CSS Integration:** Utility-first styling with responsive design, dark mode support, and kitchen-optimized color palette
- **Template-First Styling:** All CSS classes embedded directly in Askama templates for component co-location and maintainability
- **Design System:** Consistent Tailwind utilities for spacing, typography, colors, and responsive breakpoints across all components
- **Kitchen-Optimized Design:** Large touch targets (`min-h-12 min-w-12`), high contrast colors (`text-gray-900 bg-white`), readable typography (`text-lg md:text-xl`)
- **Responsive Grid System:** Mobile-first design with `grid`, `flex`, and responsive modifiers for all screen sizes from phone to desktop
- **Custom Color Palette:** Extended Tailwind config with food-inspired colors (tomato-red, olive-green, butter-yellow) for kitchen context

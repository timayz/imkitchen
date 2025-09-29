# Tech Stack

## Technology Stack Table

| Category | Technology | Version | Purpose | Rationale |
|----------|------------|---------|---------|-----------|
| Backend Language | Rust | 1.90+ | Core application development | Memory safety, performance, type safety for complex meal planning algorithms |
| Backend Framework | Axum | 0.8+ | HTTP server and routing | High-performance async, excellent ecosystem integration, middleware support |
| Frontend Language | Rust | 1.90+ | Template rendering logic | Type-safe server-side rendering, shared types between backend/frontend |
| Frontend Framework | Askama | 0.14+ | HTML template engine | Compile-time template validation, type-safe data binding, performance |
| UI Component Library | Tailwind CSS | 4.1+ | Utility-first styling | Kitchen-optimized responsive design, rapid development, consistent design system |
| Database ORM | SQLx | 0.8+ | Type-safe database operations and migrations | Direct database operations, connection pooling, embedded SQLite support |
| Database | SQLite | 3.40+ | Data persistence and storage | Embedded database, excellent Rust support, per-context isolation |
| Cache | In-Memory | Built-in | Query result caching | Fast query responses, reduced database load |
| File Storage | Local Filesystem | Built-in | Recipe images, assets | Simple deployment, no external dependencies |
| Authentication | Custom Rust | Built-in | User auth with OWASP compliance | Full control, security compliance, integrated session management |
| Frontend Testing | Rust Test | Built-in | Template integration tests | Type-safe testing, compile-time validation |
| Backend Testing | Rust Test | Built-in | Domain logic and API tests | Comprehensive test coverage, TDD support |
| E2E Testing | Playwright | Latest | User journey testing | Cross-browser testing, kitchen environment simulation |
| Build Tool | Cargo | Built-in | Rust compilation and dependencies | Native Rust tooling, workspace support |
| Bundler | Tailwind CLI | 4.1+ | CSS compilation and optimization | Utility-first CSS, tree-shaking, responsive design |
| IaC Tool | Docker | Latest | Container deployment | Consistent environments, orchestration support |
| CI/CD | GitHub Actions | Latest | Automated testing and deployment | Git integration, Rust ecosystem support |
| Monitoring | Tracing | 0.1+ | Structured logging and observability | Rust-native observability, distributed tracing |
| Logging | Tracing | 0.1+ | Application logging | Structured JSON logs, correlation IDs |
| CSS Framework | Tailwind CSS | 4.1+ | Kitchen-optimized responsive design | Large touch targets, high contrast, mobile-first |

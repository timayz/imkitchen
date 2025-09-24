# Tech Stack

## Technology Stack Table

| Category | Technology | Version | Purpose | Rationale |
|----------|------------|---------|---------|-----------|
| Backend Language | Rust | 1.70+ | Core application logic and web server | Memory safety, performance, and excellent async ecosystem for meal planning algorithms |
| Backend Framework | Axum | 0.8+ | HTTP server and routing | Fast async performance, type-safe routing, excellent ecosystem integration |
| Template Engine | Askama | 0.14+ | Server-side HTML rendering | Compile-time template safety, Jinja2-like syntax, zero-runtime overhead |
| Frontend Enhancement | twinspark-js | Latest | Progressive client-side interactivity | Lightweight HTMX-like enhancement without heavy JavaScript frameworks |
| Event System | Evento | 1.1+ | Internal event-driven architecture | Rust-native event system for decoupled service communication |
| Database | SQLite | 3.40+ | Data persistence and caching | Embedded database eliminating external dependencies, excellent for single-binary deployment |
| Database Driver | SQLx | 0.8+ | Type-safe database access | Async database operations without compile-time query checking for development flexibility |
| Monitoring | Tracing | 0.1+ | Structured logging and observability | Rust ecosystem standard for distributed tracing and performance monitoring |
| Internationalization | rust-i18n | Latest | Multi-language support | Compile-time i18n for global recipe sharing and community features |
| Configuration | config | 0.15+ | Environment variable and secrets management | Secure configuration management with multiple source support |
| CSS Framework | Tailwind CSS | 3.4+ | Utility-first styling | Mobile-first design system with excellent customization for kitchen environments |
| Authentication | Custom Rust + Sessions | - | OWASP-compliant user authentication | Custom implementation following OWASP guidelines for full control and security |
| Testing Framework | tokio-test + rstest | Latest | Backend unit and integration testing | Async-compatible testing with parameterized test support |
| E2E Testing | Playwright | Latest | End-to-end browser testing | Cross-browser testing for PWA functionality and mobile experience |
| Build Tool | Cargo | Latest | Rust build system and dependency management | Native Rust toolchain for optimal build performance |
| Containerization | Docker | Latest | Application containerization | Consistent deployment across environments |
| CI/CD | GitHub Actions | Latest | Automated testing and deployment | Integrated with GitHub, excellent Rust toolchain support |
| Image Storage | Local Filesystem + Backups | - | Recipe image storage | Simple approach for MVP, with automated backup strategy |

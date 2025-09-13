# Tech Stack

## Technology Stack Table

| Category | Technology | Version | Purpose | Rationale |
|----------|------------|---------|---------|-----------|
| Frontend Language | Rust | 1.70+ | Template rendering, business logic | Type safety, performance, single language across stack |
| Frontend Framework | Askama | 0.14+ | Server-side templating | Compile-time template checking, minimal runtime overhead |
| UI Reactivity | twinspark-js | Latest | Client-side DOM reactivity | Lightweight alternative to heavy JS frameworks |
| State Management | HTML + Alpine.js | 3.x | Client-side state | Minimal JS footprint with reactive capabilities |
| Backend Language | Rust | 1.70+ | API server, business logic | Memory safety, performance, excellent ecosystem |
| Backend Framework | axum | 0.8+ | HTTP server and routing | Modern async framework with Tower ecosystem |
| API Style | REST | HTTP/1.1 | Client-server communication | Simple, well-understood, cacheable |
| Database | PostgreSQL | 17+ | Primary data storage | ACID compliance, JSON support, vector extensions |
| Cache | Redis | 8.2+ | Session storage, caching | High performance, pub/sub for real-time features |
| File Storage | Local + S3 Compatible | - | Recipe images, assets | Cost-effective with cloud migration path |
| Authentication | JWT + Sessions | - | User authentication | Stateless tokens with server-side session validation |
| Frontend Testing | wasm-pack-test | Latest | Rust WebAssembly testing | Test business logic in same language |
| Backend Testing | tokio-test | Latest | Async Rust testing | First-class async testing support |
| E2E Testing | Playwright | Latest | Browser automation | Reliable cross-browser testing |
| Build Tool | Cargo | Latest | Rust compilation | Built-in Rust toolchain |
| Bundler | Vite | 5.x | Asset bundling | Fast development builds, tree shaking |
| IaC Tool | Docker Compose | 2.x | Local development | Simple orchestration with production parity |
| CI/CD | GitHub Actions | - | Automated testing/deployment | Integrated with repository, free for open source |
| Monitoring | Prometheus | Latest | Metrics collection | Industry standard with rich ecosystem |
| Logging | tracing + OTEL | Latest | Structured logging | Rust-native observability with OpenTelemetry |
| CSS Framework | Tailwind CSS | 3.x | Utility-first styling | Rapid development, small bundle size |

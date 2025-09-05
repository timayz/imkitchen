# Tech Stack

This is the DEFINITIVE technology selection for the entire project. All development must use these exact versions.

## Technology Stack Table

| Category | Technology | Version | Purpose | Rationale |
|----------|------------|---------|---------|-----------|
| Frontend Language | TypeScript | 5.3+ | Type-safe mobile/web development | Strong typing for complex meal planning logic, excellent tooling support |
| Frontend Framework | Lynx-js | Latest | Cross-platform mobile app | PRD requirement, native performance with cross-platform efficiency |
| UI Component Library | Tamagui | 1.x | Mobile-optimized components | Performance-focused, kitchen-friendly touch targets (44px min) |
| State Management | Zustand | 4.x | Lightweight state management | Simple API, excellent TypeScript support, minimal overhead for mobile |
| Backend Language | Rust | 1.75+ | High-performance API services | PRD requirement, sub-2-second meal planning engine performance |
| Backend Framework | Axum | 0.7+ | Async web framework | Tokio-based, excellent performance, built-in middleware support |
| Admin UI Framework | TwinSpark | Latest | Integrated admin interface | PRD requirement, seamless Rust backend integration |
| API Style | REST | - | HTTP JSON APIs | Simple, cacheable, excellent mobile client support |
| Database | PostgreSQL | 15+ | Relational data storage | Complex recipe relationships, ACID transactions, JSON support |
| Cache | Redis | 7+ | High-performance caching | Sub-2-second meal plan generation optimization |
| File Storage | MinIO | Latest | S3-compatible object storage | Cloud-agnostic, self-hostable, S3 API compatibility |
| Authentication | JWT + bcrypt | - | Secure user authentication | Stateless tokens for mobile, secure password hashing |
| Frontend Testing | Vitest | 1.x | Fast unit/integration testing | Native TypeScript support, excellent performance |
| Backend Testing | cargo test | - | Rust native testing | Built-in test framework, async test support |
| E2E Testing | Playwright | 1.x | Cross-platform end-to-end | Mobile testing support, reliable automation |
| Build Tool | Vite | 5.x | Fast frontend builds | Excellent TypeScript support, fast HMR for development |
| Bundler | Rollup | 4.x | Production bundling | Vite's production bundler, optimal mobile bundle sizes |
| IaC Tool | Terraform | 1.6+ | Infrastructure as Code | Cloud-agnostic, multi-provider support, mature ecosystem |
| CI/CD | GitHub Actions | - | Automated deployment | Provider-agnostic workflows, cost-effective, good mobile tooling |
| Monitoring | Prometheus + Grafana | Latest | Application monitoring | Cloud-agnostic metrics collection and visualization |
| Logging | OpenTelemetry + tracing | 0.1.x | Distributed tracing/logging | Vendor-neutral observability, structured logging |
| CSS Framework | Tailwind CSS | 3.x | Utility-first styling | Mobile-first responsive design, consistent spacing system |
| Container Runtime | Docker | Latest | Application containerization | Platform-agnostic deployment, consistent environments |
| Container Orchestration | Kubernetes | 1.28+ | Container management | Cloud-agnostic, horizontal scaling, mature ecosystem |
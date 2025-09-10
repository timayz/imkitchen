# Tech Stack

## Technology Stack Table

| Category | Technology | Version | Purpose | Rationale |
|----------|------------|---------|---------|-----------|
| Frontend Language | TypeScript | 5.x | Type-safe mobile/web development | Enables shared types across mobile/web, reduces runtime errors in automation workflows |
| Frontend Framework | Lynx.js | Latest | Cross-platform mobile + web | PRD requirement, single codebase for iOS/Android/Web with native performance |
| UI Component Library | Custom Design System | - | imkitchen-specific components | Supports unique "Fill My Week" automation UI and warm kitchen aesthetic |
| State Management | Zustand | 4.x | Lightweight React state | Simple, performant state management for mobile contexts, supports automation workflows |
| Backend Language | Go | 1.21+ | High-performance API services | Optimal for 2-second meal plan generation requirement and rotation algorithms |
| Backend Framework | Gin | 1.9+ | Fast HTTP web framework | Proven performance, excellent middleware ecosystem, cloud-agnostic |
| API Style | REST | - | HTTP-based APIs | Simple, cacheable, excellent mobile network handling, broad tooling support |
| Database | PostgreSQL | 15+ | Primary relational database | ACID compliance for recipe/user data, excellent Go integration, multi-cloud support |
| Cache | Redis | 7.x | High-performance caching | Essential for 2-second generation target, session storage, rate limiting |
| File Storage | MinIO (S3-compatible) | Latest | Recipe images and assets | Self-hosted S3-compatible storage, multi-cloud deployment, cost control |
| Authentication | Supabase Auth | Latest | User auth and management | Open-source, self-hostable, social login support, excellent mobile SDKs |
| Frontend Testing | Vitest + Testing Library | Latest | Component and unit testing | Fast, modern testing for Lynx.js components and automation logic |
| Backend Testing | Go Testing + Testify | Latest | API and service testing | Built-in Go testing with assertions, perfect for algorithm validation |
| E2E Testing | Playwright | 1.x | Cross-platform automation | Mobile and web testing, critical for "Fill My Week" workflow validation |
| Build Tool | Nx | 17+ | Monorepo orchestration | Shared code between mobile/API, efficient builds, TypeScript integration |
| Bundler | Vite | 5.x | Fast frontend builds | Modern bundling for Lynx.js, excellent development experience |
| IaC Tool | Terraform | 1.6+ | Infrastructure as code | Cloud-agnostic, mature ecosystem, supports multiple providers |
| CI/CD | GitHub Actions | - | Automated testing and deployment | Free tier, multi-cloud deployment support, excellent community actions |
| Monitoring | Grafana + Prometheus | Latest | Application monitoring | Open-source, cloud-agnostic, excellent Go metrics, customizable dashboards |
| Logging | Loki + Promtail | Latest | Centralized logging | Cloud-agnostic log aggregation, integrates with Grafana, cost-effective |
| CSS Framework | Tailwind CSS | 3.x | Utility-first styling | Rapid UI development, excellent mobile responsiveness, design system support |

## Platform Strategy

**Platform:** Multi-Cloud Strategy (Cloud-Agnostic)
- **Primary:** Digital Ocean or Hetzner Cloud for cost-effective, developer-friendly hosting
- **Containerization:** Docker + Kubernetes for maximum portability
- **CDN:** Cloudflare for provider-agnostic performance and security
- **Database:** Managed PostgreSQL (provider-agnostic) with automated backups

**Key Benefits:** No vendor lock-in, cost control, deployment flexibility, open-source ecosystem

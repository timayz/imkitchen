# Tech Stack

## Technology Stack Table

| Category             | Technology                   | Version  | Purpose                                    | Rationale                                                                                      |
| -------------------- | ---------------------------- | -------- | ------------------------------------------ | ---------------------------------------------------------------------------------------------- |
| Frontend Language    | TypeScript                   | 5.0+     | Type-safe frontend development             | Kitchen safety requires predictable interfaces; prevents runtime errors during cooking         |
| Frontend Framework   | Next.js                      | 14+      | Full-stack React framework with App Router | Unified frontend/backend development; excellent SSG/SSR for SEO; built-in internationalization |
| UI Component Library | Custom + Radix UI            | Latest   | Accessible component primitives            | Kitchen accessibility requirements; voice/keyboard navigation; high customization needs        |
| State Management     | React Context + useReducer   | Built-in | Client-side state management               | Sufficient for app complexity; reduces bundle size; integrates with Server Components          |
| Backend Language     | TypeScript                   | 5.0+     | Type-safe backend development              | Shared types between frontend/backend; prevents API contract mismatches                        |
| Backend Framework    | Next.js API Routes           | 14+      | Serverless-style API endpoints             | Unified codebase; automatic deployment optimization; edge computing support                    |
| API Style            | REST + OpenAPI               | 3.0      | RESTful APIs with documentation            | Standard, cacheable, voice-command compatible; clear contract definitions                      |
| Database             | PostgreSQL                   | 15+      | Primary relational database                | ACID compliance for inventory tracking; complex recipe relationships; international scaling    |
| Cache                | Redis                        | 7+       | Session and application caching            | Fast recipe lookup; voice command response times; shopping list synchronization                |
| File Storage         | S3-Compatible Storage        | Latest   | Recipe images and user uploads             | Global CDN distribution; vendor independence; cost optimization                                |
| Authentication       | NextAuth.js                  | 4+       | Authentication and session management      | Next.js integration; multiple providers; secure session handling                               |
| Frontend Testing     | Jest + React Testing Library | Latest   | Component and integration testing          | Kitchen workflow testing; accessibility compliance verification                                |
| Backend Testing      | Jest + Supertest             | Latest   | API endpoint and service testing           | Critical for food safety features; recipe data integrity                                       |
| E2E Testing          | Playwright                   | Latest   | End-to-end user journey testing            | Voice interaction testing; mobile cooking mode validation                                      |
| Build Tool           | Next.js Build                | Built-in | Application bundling and optimization      | Integrated toolchain; automatic optimization; edge deployment                                  |
| Bundler              | Webpack (via Next.js)        | Latest   | Module bundling and code splitting         | Automatic bundle optimization; dynamic imports for cooking mode                                |
| IaC Tool             | Docker + Docker Compose      | Latest   | Infrastructure as code                     | Platform independence; local development parity; cloud agnostic                                |
| CI/CD                | GitHub Actions               | Latest   | Continuous integration and deployment      | Free for open source; excellent Next.js integration; multi-environment support                 |
| Monitoring           | Sentry + Vercel Analytics    | Latest   | Error tracking and performance monitoring  | Real-time cooking session error detection; performance optimization                            |
| Logging              | Winston + Structured Logging | Latest   | Application logging                        | Kitchen session debugging; voice command analysis; security monitoring                         |
| CSS Framework        | Tailwind CSS                 | 3+       | Utility-first styling framework            | Kitchen-first responsive design; consistent spacing; dark mode support                         |

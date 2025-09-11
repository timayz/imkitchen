# 3. Technology Stack

## 3.1 T3 Stack Foundation

**Core Technologies:**
- **Next.js 15:** React framework with App Router, Server Components, and streaming SSR
- **TypeScript 5.0+:** Strict type safety across entire application stack
- **tRPC:** End-to-end typesafe APIs with automatic client generation
- **Prisma:** Type-safe database client with migration management
- **Tailwind CSS 4.1:** Utility-first CSS framework with design system support
- **NextAuth.js:** Authentication library with multiple provider support

## 3.2 Frontend Stack

**User Interface:**
```json
{
  "framework": "Next.js 15",
  "styling": "Tailwind CSS 4.1",
  "components": "Headless UI 2.1",
  "stateManagement": "Zustand + React Query",
  "forms": "React Hook Form + Zod",
  "testing": "Jest + Testing Library + Playwright"
}
```

**Progressive Web App:**
- Service Worker for offline functionality and background sync
- Web App Manifest for native app-like installation
- Push notification support for timing alerts
- IndexedDB for local recipe and meal plan storage

## 3.3 Backend Stack

**API Layer:**
```typescript
// tRPC Router Example
export const mealPlanRouter = router({
  create: protectedProcedure
    .input(createMealPlanSchema)
    .output(mealPlanSchema)
    .mutation(async ({ input, ctx }) => {
      return await ctx.prisma.mealPlan.create({
        data: {
          ...input,
          userId: ctx.session.user.id
        }
      });
    })
});
```

**Database & ORM:**
- PostgreSQL 17.6 with JSONB support for flexible recipe data
- Prisma ORM with type-safe queries and migrations
- Connection pooling with PgBouncer for scalability
- Redis for session storage and caching

## 3.4 Infrastructure Stack

**Deployment Platform:**
```yaml
# Kubernetes Deployment
apiVersion: apps/v1
kind: Deployment
metadata:
  name: imkitchen-api
spec:
  replicas: 3
  selector:
    matchLabels:
      app: imkitchen-api
  template:
    metadata:
      labels:
        app: imkitchen-api
    spec:
      containers:
      - name: api
        image: imkitchen/api:latest
        ports:
        - containerPort: 3000
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: db-secret
              key: url
```

**Monitoring & Observability:**
- DataDog for application performance monitoring
- Sentry for error tracking and performance insights
- Prometheus + Grafana for infrastructure metrics
- Custom business metrics dashboard

## 3.5 Development Tools

**Code Quality:**
```json
{
  "linting": "ESLint + @typescript-eslint",
  "formatting": "Prettier",
  "preCommitHooks": "Husky + lint-staged",
  "testing": "Jest + Vitest + Playwright",
  "bundling": "Next.js built-in + Turbopack"
}
```

**Monorepo Management:**
- Turborepo for build optimization and task scheduling
- pnpm for efficient package management
- Shared TypeScript configurations and ESLint rules
- Workspace-based dependency management

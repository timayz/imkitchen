# Unified Project Structure

```plaintext
imkitchen/
├── .github/                    # CI/CD workflows
│   └── workflows/
│       ├── ci.yaml
│       ├── deploy-staging.yaml
│       └── deploy-production.yaml
├── .next/                      # Next.js build output (ignored)
├── public/                     # Static assets
│   ├── icons/
│   ├── images/
│   ├── locales/               # Translation files
│   │   ├── en/
│   │   ├── es/
│   │   ├── fr/
│   │   └── de/
│   ├── manifest.json          # PWA manifest
│   └── sw.js                  # Service worker
├── src/
│   ├── app/                   # Next.js 14 App Router
│   │   ├── [locale]/          # Internationalized routes
│   │   │   ├── (auth)/        # Auth route group
│   │   │   │   ├── login/
│   │   │   │   └── register/
│   │   │   ├── dashboard/
│   │   │   │   └── page.tsx
│   │   │   ├── inventory/
│   │   │   │   ├── page.tsx
│   │   │   │   ├── add/
│   │   │   │   └── [itemId]/
│   │   │   ├── recipes/
│   │   │   │   ├── page.tsx
│   │   │   │   ├── search/
│   │   │   │   ├── [recipeId]/
│   │   │   │   └── favorites/
│   │   │   ├── meal-planning/
│   │   │   │   ├── page.tsx
│   │   │   │   └── calendar/
│   │   │   ├── shopping/
│   │   │   │   ├── page.tsx
│   │   │   │   └── [listId]/
│   │   │   ├── cooking/
│   │   │   │   └── [recipeId]/
│   │   │   └── settings/
│   │   ├── api/               # API routes
│   │   │   ├── auth/
│   │   │   │   ├── [...nextauth]/
│   │   │   │   └── register/
│   │   │   ├── inventory/
│   │   │   │   ├── route.ts
│   │   │   │   ├── [itemId]/
│   │   │   │   └── expiring/
│   │   │   ├── recipes/
│   │   │   │   ├── route.ts
│   │   │   │   ├── search/
│   │   │   │   ├── [recipeId]/
│   │   │   │   └── suggestions/
│   │   │   ├── meal-plans/
│   │   │   │   ├── route.ts
│   │   │   │   └── [planId]/
│   │   │   ├── shopping/
│   │   │   │   ├── route.ts
│   │   │   │   └── lists/
│   │   │   ├── voice/
│   │   │   │   ├── process/
│   │   │   │   └── cooking/
│   │   │   └── webhooks/
│   │   ├── globals.css        # Global styles
│   │   ├── layout.tsx         # Root layout
│   │   ├── loading.tsx        # Loading UI
│   │   ├── error.tsx          # Error UI
│   │   └── not-found.tsx      # 404 page
│   ├── components/            # React components
│   │   ├── ui/                # Base UI components
│   │   │   ├── button.tsx
│   │   │   ├── input.tsx
│   │   │   ├── modal.tsx
│   │   │   └── index.ts
│   │   ├── forms/             # Form components
│   │   │   ├── inventory-form.tsx
│   │   │   ├── recipe-form.tsx
│   │   │   └── login-form.tsx
│   │   ├── inventory/         # Inventory components
│   │   │   ├── inventory-list.tsx
│   │   │   ├── inventory-item.tsx
│   │   │   └── expiration-alert.tsx
│   │   ├── recipes/           # Recipe components
│   │   │   ├── recipe-card.tsx
│   │   │   ├── recipe-detail.tsx
│   │   │   └── recipe-search.tsx
│   │   ├── meal-planning/     # Meal planning components
│   │   │   ├── calendar-view.tsx
│   │   │   ├── meal-slot.tsx
│   │   │   └── drag-drop-provider.tsx
│   │   ├── shopping/          # Shopping components
│   │   │   ├── shopping-list.tsx
│   │   │   ├── shopping-item.tsx
│   │   │   └── store-categories.tsx
│   │   ├── cooking/           # Cooking mode components
│   │   │   ├── cooking-interface.tsx
│   │   │   ├── timer-manager.tsx
│   │   │   └── voice-controls.tsx
│   │   ├── voice/             # Voice interaction components
│   │   │   ├── voice-button.tsx
│   │   │   ├── voice-status.tsx
│   │   │   └── speech-recognition.tsx
│   │   └── layout/            # Layout components
│   │       ├── navigation.tsx
│   │       ├── sidebar.tsx
│   │       ├── header.tsx
│   │       └── footer.tsx
│   ├── hooks/                 # Custom React hooks
│   │   ├── use-auth.ts
│   │   ├── use-inventory.ts
│   │   ├── use-recipes.ts
│   │   ├── use-voice.ts
│   │   ├── use-timers.ts
│   │   └── use-local-storage.ts
│   ├── lib/                   # Utility functions and configurations
│   │   ├── auth.ts            # NextAuth configuration
│   │   ├── db.ts              # Database connection
│   │   ├── redis.ts           # Redis connection
│   │   ├── storage.ts         # File storage abstraction
│   │   ├── email.ts           # Email service abstraction
│   │   ├── voice.ts           # Voice processing utilities
│   │   ├── utils.ts           # General utilities
│   │   ├── constants.ts       # Application constants
│   │   ├── validators.ts      # Zod schemas
│   │   ├── api-client.ts      # API client
│   │   └── services/          # Business logic services
│   │       ├── inventory-service.ts
│   │       ├── recipe-service.ts
│   │       ├── meal-plan-service.ts
│   │       ├── shopping-service.ts
│   │       ├── voice-service.ts
│   │       └── notification-service.ts
│   ├── stores/                # State management
│   │   ├── auth-store.ts
│   │   ├── inventory-store.ts
│   │   ├── recipe-store.ts
│   │   ├── meal-plan-store.ts
│   │   ├── shopping-store.ts
│   │   ├── cooking-store.ts
│   │   ├── voice-store.ts
│   │   └── ui-store.ts
│   ├── types/                 # TypeScript type definitions
│   │   ├── auth.ts
│   │   ├── inventory.ts
│   │   ├── recipe.ts
│   │   ├── meal-plan.ts
│   │   ├── shopping.ts
│   │   ├── voice.ts
│   │   ├── api.ts
│   │   └── index.ts
│   └── middleware.ts          # Next.js middleware
├── prisma/                    # Database schema and migrations
│   ├── schema.prisma
│   ├── migrations/
│   └── seed.ts
├── tests/                     # Test files
│   ├── __mocks__/
│   ├── components/
│   ├── pages/
│   ├── api/
│   ├── e2e/
│   └── setup.ts
├── docker/                    # Docker configuration
│   ├── Dockerfile
│   ├── docker-compose.yml
│   ├── docker-compose.prod.yml
│   └── nginx.conf
├── docs/                      # Documentation
│   ├── prd.md
│   ├── front-end-spec.md
│   ├── architecture.md
│   ├── api-docs.md
│   └── deployment.md
├── scripts/                   # Build and deployment scripts
│   ├── build.sh
│   ├── deploy.sh
│   ├── backup.sh
│   └── seed-data.ts
├── .env.example               # Environment template
├── .env.local                 # Local development environment
├── .gitignore
├── .eslintrc.json
├── .prettierrc
├── tailwind.config.js
├── next.config.js
├── tsconfig.json
├── package.json
├── pnpm-lock.yaml
├── jest.config.js
├── playwright.config.ts
└── README.md
```

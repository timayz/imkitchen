# 12. Unified Project Structure

## Monorepo Organization (Nx Workspace)

```
imkitchen/
├── README.md
├── package.json                    # Root package.json with workspace config
├── nx.json                         # Nx workspace configuration  
├── tsconfig.base.json             # Base TypeScript config
├── .gitignore
├── .env.example
├── docker-compose.yml             # Local development environment
├── Dockerfile.frontend            # Frontend production build
├── Dockerfile.backend             # Backend production build
├── kubernetes/                    # K8s deployment manifests
│   ├── namespace.yaml
│   ├── configmap.yaml
│   ├── secrets.yaml
│   ├── backend-deployment.yaml
│   ├── frontend-deployment.yaml
│   ├── postgres-deployment.yaml
│   ├── redis-deployment.yaml
│   └── ingress.yaml
├── apps/
│   ├── mobile/                    # Lynx.js Mobile App
│   │   ├── src/
│   │   │   ├── screens/          # Screen components
│   │   │   │   ├── auth/         # Authentication screens
│   │   │   │   ├── meal-plans/   # Meal planning screens
│   │   │   │   ├── recipes/      # Recipe browsing screens
│   │   │   │   ├── shopping/     # Shopping list screens
│   │   │   │   └── profile/      # User profile screens
│   │   │   ├── components/       # Reusable UI components
│   │   │   │   ├── atoms/        # Basic building blocks
│   │   │   │   ├── molecules/    # Simple combinations
│   │   │   │   ├── organisms/    # Complex sections
│   │   │   │   └── templates/    # Page layouts
│   │   │   ├── services/         # API clients and services
│   │   │   ├── store/            # Redux store configuration
│   │   │   └── types/            # TypeScript type definitions
│   │   └── __tests__/            # Mobile app tests
│   └── backend/                   # Go Backend Service
│       ├── internal/
│       │   ├── handlers/         # HTTP request handlers
│       │   ├── services/         # Application services
│       │   ├── repositories/     # Data access layer
│       │   ├── middleware/       # HTTP middleware
│       │   └── clients/          # External API clients
│       ├── migrations/           # Database migrations
│       └── tests/                # Backend tests
├── libs/                         # Shared libraries
│   ├── shared-types/            # Shared TypeScript types
│   ├── ui-components/           # Shared UI components
│   ├── api-client/              # Shared API client
│   └── validation/              # Shared validation rules
└── tools/                       # Development tools
    ├── database/
    ├── scripts/
    └── docker/
```

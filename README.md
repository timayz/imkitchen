# ImKitchen 🍳

Smart recipe management and meal planning application built with the T3 stack.

## Features

- **Recipe Management**: Create, edit, and organize your personal recipe collection
- **Meal Planning**: Plan your weekly meals with intelligent timing
- **Smart Shopping Lists**: Auto-generated shopping lists from your meal plans
- **Timing Intelligence**: Get cooking notifications at the perfect time
- **Progressive Web App**: Install and use offline on any device

## Tech Stack

- **Frontend**: Next.js 15, TypeScript, Tailwind CSS, PWA
- **Backend**: tRPC, Express, TypeScript
- **Database**: PostgreSQL, Prisma ORM
- **Infrastructure**: Docker, Turborepo, pnpm
- **Development**: ESLint, Prettier, Jest, Playwright

## Prerequisites

Before you begin, ensure you have the following installed:

- **Node.js** 18+ ([Download](https://nodejs.org/))
- **pnpm** 8.6.0+ (`npm install -g pnpm`)
- **Docker** ([Download](https://www.docker.com/get-started))
- **Git** ([Download](https://git-scm.com/))

### Platform-Specific Notes

#### Windows
- Use WSL2 for the best development experience
- Docker Desktop with WSL2 backend recommended

#### macOS
- Docker Desktop recommended
- Homebrew can be used to install dependencies: `brew install node pnpm docker`

#### Linux
- Use your distribution's package manager for Node.js and Docker
- Ensure Docker service is running: `sudo systemctl start docker`

## Getting Started

### 1. Clone the Repository

\`\`\`bash
git clone https://github.com/your-org/imkitchen.git
cd imkitchen
\`\`\`

### 2. Install Dependencies

\`\`\`bash
pnpm install
\`\`\`

### 3. Environment Configuration

Copy the environment example files and configure them:

\`\`\`bash
# Root environment
cp .env.example .env.local

# API environment
cp packages/api/.env.example packages/api/.env.local

# Web environment  
cp packages/web/.env.local.example packages/web/.env.local
\`\`\`

### 4. Start Database Services

\`\`\`bash
# Start PostgreSQL and Redis with Docker
pnpm docker:up

# Wait for services to be ready (check with docker ps)
\`\`\`

### 5. Setup Database

\`\`\`bash
# Run database migrations
pnpm db:migrate

# Seed database with sample data
pnpm db:seed
\`\`\`

### 6. Start Development Servers

\`\`\`bash
# Start all services in parallel
pnpm dev
\`\`\`

This will start:
- 🌐 **Frontend**: http://localhost:3000
- 🔌 **API**: http://localhost:3001
- 📊 **Database**: localhost:5432
- 🔄 **Redis**: localhost:6379

## Development Workflow

### Available Scripts

\`\`\`bash
# Development
pnpm dev              # Start all services in development mode
pnpm build            # Build all packages
pnpm lint             # Lint all packages
pnpm test             # Run all tests

# Database
pnpm db:migrate       # Run database migrations
pnpm db:push          # Push schema changes to database
pnpm db:seed          # Seed database with sample data
pnpm db:studio        # Open Prisma Studio

# Docker
pnpm docker:up        # Start database services
pnpm docker:build     # Build Docker images
\`\`\`

### Project Structure

\`\`\`
imkitchen/
├── packages/
│   ├── web/                 # Next.js frontend application
│   │   ├── src/app/        # Next.js App Router
│   │   ├── src/components/ # React components
│   │   └── public/         # Static assets
│   ├── api/                # tRPC API server
│   │   ├── src/routers/    # API route definitions
│   │   ├── src/lib/        # Utilities and helpers
│   │   └── prisma/         # Database schema
│   ├── shared/             # Shared utilities and types
│   └── worker/             # Background job processing
├── tools/                  # Shared configurations
├── docs/                   # Documentation
├── docker-compose.yml      # Database services
├── turbo.json             # Turborepo configuration
└── package.json           # Root package configuration
\`\`\`

### Code Quality

The project uses several tools to maintain code quality:

- **TypeScript**: Strict type checking across all packages
- **ESLint**: Code linting with TypeScript rules
- **Prettier**: Consistent code formatting
- **Husky**: Pre-commit hooks for quality checks

### Testing

\`\`\`bash
# Run all tests
pnpm test

# Run tests for specific package
cd packages/web && pnpm test
cd packages/api && pnpm test

# Run E2E tests
pnpm test:e2e
\`\`\`

## Troubleshooting

### Common Issues

**Port already in use**
\`\`\`bash
# Kill processes on ports 3000, 3001, 5432, 6379
lsof -ti:3000,3001,5432,6379 | xargs kill -9
\`\`\`

**Docker issues**
\`\`\`bash
# Reset Docker containers
docker-compose down -v
docker-compose up -d

# Check service health
docker-compose ps
\`\`\`

**Database connection issues**
\`\`\`bash
# Verify database is running
docker-compose logs postgres

# Reset database
pnpm db:push --force-reset
pnpm db:seed
\`\`\`

**pnpm installation issues**
\`\`\`bash
# Clear pnpm cache
pnpm store prune

# Reinstall dependencies
rm -rf node_modules packages/*/node_modules
pnpm install
\`\`\`

### Getting Help

- 📖 **Documentation**: Check the `/docs` folder for detailed guides
- 🐛 **Issues**: Report bugs on GitHub Issues
- 💬 **Discussions**: Join GitHub Discussions for questions

## Contributing

1. Fork the repository
2. Create a feature branch: \`git checkout -b feature/amazing-feature\`
3. Commit your changes: \`git commit -m 'Add amazing feature'\`
4. Push to the branch: \`git push origin feature/amazing-feature\`
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

**Happy Cooking! 🍳✨**
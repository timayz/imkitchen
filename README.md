# imkitchen 🍳

A modern kitchen management application built with Next.js 14, TypeScript, and Tailwind CSS. Manage your recipes, track inventory, plan meals, and optimize your cooking experience with voice-enabled interactions.

## Project Overview

imkitchen is a comprehensive kitchen management solution designed for modern cooking needs. It combines recipe management, inventory tracking, meal planning, and smart shopping lists with accessibility-first design and voice command support.

### Key Features

- 📱 Responsive design optimized for kitchen use
- 🎨 Kitchen-themed design system with dark mode support
- 🗣️ Voice command integration for hands-free operation
- 🌍 Multi-language support (English, Spanish, French, German)
- 🔒 Secure user authentication and data protection
- 📊 Smart meal planning and nutritional tracking

## Technology Stack

- **Frontend**: Next.js 14+ (App Router), TypeScript 5.0+, Tailwind CSS 3+
- **Testing**: Jest + React Testing Library, Playwright for E2E
- **Database**: PostgreSQL 15+ with Prisma ORM
- **Cache**: Redis 7+
- **Authentication**: NextAuth.js
- **Development**: ESLint, Prettier, Husky pre-commit hooks

## Development Setup

### Prerequisites

- Node.js 18+
- npm (or pnpm/yarn)
- Docker and Docker Compose (for database services)

### Quick Start

1. **Clone and install dependencies**

   ```bash
   git clone <repository-url>
   cd imkitchen
   npm install
   ```

2. **Set up environment variables**

   ```bash
   cp .env.example .env.local
   # Edit .env.local with your API keys and configuration
   ```

3. **Start infrastructure services**

   ```bash
   docker-compose up -d postgres redis
   ```

4. **Run database migrations** (when available in Story 1.2)

   ```bash
   npm run db:migrate
   ```

5. **Start development server**

   ```bash
   npm run dev
   ```

6. **Open application**
   Navigate to [http://localhost:3000](http://localhost:3000)

### Available Scripts

| Script               | Description                          |
| -------------------- | ------------------------------------ |
| `npm run dev`        | Start development server             |
| `npm run build`      | Build production application         |
| `npm run start`      | Start production server              |
| `npm run test`       | Run unit tests with Jest             |
| `npm run test:e2e`   | Run end-to-end tests with Playwright |
| `npm run lint`       | Run ESLint code linting              |
| `npm run type-check` | Run TypeScript type checking         |
| `npm run format`     | Format code with Prettier            |

### Docker Development

Run the entire stack with Docker Compose:

```bash
docker-compose up --build
```

This starts:

- Next.js application on port 3000
- PostgreSQL database on port 5432
- Redis cache on port 6379

## External Service Setup

### Required API Keys

1. **Spoonacular API** (Recipe data)
   - Register at [spoonacular.com/food-api](https://spoonacular.com/food-api)
   - Add `SPOONACULAR_API_KEY` to your `.env.local`

2. **OpenFoodFacts API** (Product information)
   - Free API, no registration required
   - Documentation: [world.openfoodfacts.org](https://world.openfoodfacts.org/data)

3. **OpenAI API** (Voice services)
   - Register at [platform.openai.com](https://platform.openai.com)
   - Add `OPENAI_API_KEY` to your `.env.local`

### API Setup Troubleshooting

- **Spoonacular Rate Limits**: Free tier allows 150 requests/day
- **OpenAI Voice API**: Requires billing setup for production use
- **Development Mode**: Most external APIs can be mocked during development

## Project Structure

```
imkitchen/
├── src/
│   ├── app/           # Next.js 14 App Router pages
│   ├── components/    # Reusable React components
│   ├── lib/          # Utility functions and configurations
│   └── types/        # Shared TypeScript definitions
├── tests/
│   ├── components/   # Component unit tests
│   ├── api/         # API integration tests
│   └── e2e/         # End-to-end tests
├── docs/            # Project documentation
├── docker/          # Docker configuration files
└── public/          # Static assets
```

## Contributing Guidelines

### Code Style

- Use TypeScript for all new code
- Follow ESLint and Prettier configurations
- Write tests for new features
- Use conventional commit messages

### Development Workflow

1. Create feature branch from `main`
2. Implement changes with tests
3. Run quality checks: `npm run lint && npm run type-check && npm test`
4. Submit pull request with clear description

### Git Hooks

Pre-commit hooks automatically run:

- ESLint with auto-fix
- Prettier formatting
- TypeScript type checking

## Testing

### Unit Tests

```bash
npm run test              # Run all tests
npm run test:watch        # Watch mode
npm run test:coverage     # Coverage report
```

### End-to-End Tests

```bash
npm run test:e2e          # Run E2E tests
npm run test:e2e:ui       # Run with UI mode
```

### Testing Philosophy

- **Unit Tests**: Test individual components and functions
- **Integration Tests**: Test API endpoints and service interactions
- **E2E Tests**: Test complete user workflows, especially voice interactions

## Troubleshooting

### Common Issues

1. **Port 3000 already in use**

   ```bash
   lsof -ti:3000 | xargs kill -9
   ```

2. **Database connection issues**

   ```bash
   docker-compose down && docker-compose up -d postgres
   ```

3. **Node modules issues**

   ```bash
   rm -rf node_modules package-lock.json && npm install
   ```

4. **TypeScript errors**
   ```bash
   npm run type-check
   ```

### Getting Help

- Check the [Issues](https://github.com/your-org/imkitchen/issues) page
- Review project documentation in `docs/`
- Run `npm run lint` and `npm run type-check` for code issues

## License

[Your License] - See LICENSE file for details

---

Built with ❤️ for modern kitchens

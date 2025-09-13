# ImKitchen 🍳

A modern, intelligent recipe management and meal planning platform built with Rust and designed for home cooks who want to organize, plan, and cook with confidence.

## ✨ Features

- **Recipe Management**: Import, organize, and manage your favorite recipes
- **Intelligent Meal Planning**: AI-assisted weekly meal planning
- **Cook Mode**: Interactive cooking with timing coordination
- **Recipe Import**: Automatic parsing from major recipe websites
- **Responsive Design**: Optimized for mobile and desktop

## 🚀 Quick Start

### Prerequisites

- [Rust](https://rustup.rs/) 1.70 or later
- [Node.js](https://nodejs.org/) 20 or later (for CSS/JS tooling)
- [PostgreSQL](https://www.postgresql.org/) 17 or later
- [Redis](https://redis.io/) 8.2 or later
- [Docker](https://www.docker.com/) (optional, for containerized development)

### Local Development Setup

1. **Clone the repository**
   ```bash
   git clone <repository-url>
   cd imkitchen
   ```

2. **Install dependencies**
   ```bash
   # Install Rust dependencies
   cargo build
   
   # Install frontend tooling dependencies
   npm install
   ```

3. **Environment configuration**
   ```bash
   # Copy environment template
   cp .env.example .env
   
   # Edit .env with your database credentials and settings
   # Required variables:
   # - DATABASE_URL
   # - JWT_SECRET (minimum 32 characters)
   ```

4. **Database setup**
   ```bash
   # Option 1: Using Docker (recommended)
   docker-compose up -d postgres redis
   
   # Option 2: Local installation
   # Start PostgreSQL and Redis services locally
   # Create database: createdb imkitchen_dev
   ```

5. **Install sqlx-cli and run migrations**
   ```bash
   cargo install sqlx-cli --no-default-features --features postgres
   sqlx migrate run
   ```

6. **Start the development server**
   ```bash
   # Start the Rust application
   cargo run
   
   # In a separate terminal, build CSS during development
   npm run css:watch
   ```

The application will be available at `http://localhost:3000`

### Docker Development

For a completely containerized development experience:

```bash
# Start all services with hot-reload
docker-compose -f docker-compose.dev.yml up

# Or build and run the production container
docker-compose -f docker-compose.prod.yml up --build
```

## 🏗️ Architecture

ImKitchen is built with a modern, scalable architecture:

- **Backend**: Rust with axum web framework
- **Frontend**: Server-side rendered templates with Askama
- **Database**: PostgreSQL with full-text search
- **Cache**: Redis for session management
- **Reactivity**: twinspark-js and Alpine.js for client-side interactivity

### Project Structure

```
imkitchen/
├── src/                     # Rust application source
│   ├── routes/             # HTTP route handlers
│   ├── services/           # Business logic services
│   ├── repositories/       # Data access layer
│   ├── models/             # Domain models
│   ├── middleware/         # HTTP middleware
│   └── config/             # Configuration management
├── templates/              # Askama HTML templates
├── static/                 # Static assets (CSS, JS, images)
├── migrations/             # Database migrations
├── tests/                  # Test suites
└── infrastructure/         # Docker and deployment configs
```

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run with coverage
cargo test --coverage

# Run end-to-end tests (requires running application)
npm run test:e2e

# Run linting
cargo clippy
cargo fmt --check
```

## 🚀 Deployment

### Production Build

```bash
# Build optimized binary
cargo build --release

# Build production assets
npm run build

# Or build Docker image
docker build -t imkitchen .
```

### Environment Variables

See `.env.example` for all available configuration options. Required variables for production:

- `DATABASE_URL`: PostgreSQL connection string
- `REDIS_URL`: Redis connection string  
- `JWT_SECRET`: Secret key for JWT tokens (min 32 chars)
- `ENVIRONMENT`: Set to "production"

### Health Checks

The application provides a health check endpoint at `/health` that returns:
- Overall application status
- Database connection status
- Redis connection status
- System resource usage
- Application version and uptime

## 📝 API Documentation

### Health Check
```
GET /health
```

Returns application health status and system information.

### Recipe Management
```
GET /recipes          # List all recipes
GET /recipes/{id}     # Get specific recipe
POST /recipes         # Create new recipe
PUT /recipes/{id}     # Update recipe
DELETE /recipes/{id}  # Delete recipe
```

## 🔧 Development

### Code Style

This project follows Rust community standards:
- Use `cargo fmt` for formatting
- Use `cargo clippy` for linting
- Follow the naming conventions in `docs/architecture/coding-standards.md`

### Database Migrations

```bash
# Create new migration
sqlx migrate add migration_name

# Run migrations
sqlx migrate run

# Revert last migration
sqlx migrate revert
```

### Adding New Features

1. Create feature branch from `main`
2. Implement feature following existing patterns
3. Add tests for new functionality
4. Update documentation if needed
5. Submit pull request

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests and documentation
5. Submit a pull request

Please read our [contributing guidelines](CONTRIBUTING.md) for more details.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🆘 Support

- **Documentation**: See the `docs/` directory for detailed documentation
- **Issues**: Report bugs and request features via GitHub Issues
- **Discussions**: Join discussions in GitHub Discussions

## 🗺️ Roadmap

- [ ] Mobile app with offline support
- [ ] Advanced meal planning with dietary restrictions
- [ ] Recipe sharing and social features
- [ ] Integration with grocery delivery services
- [ ] Voice-activated cooking assistance

---

**Built with ❤️ by the ImKitchen team**
# IMKitchen 🍳

> **Intelligent Kitchen Management System**  
> A comprehensive meal planning and kitchen management platform built with Rust, Askama, and SQLite

## 📋 Project Overview

IMKitchen is a full-stack kitchen management application that helps users plan meals, manage recipes, organize shopping lists, and optimize their cooking workflow. Built with modern Rust technologies, it provides a fast, secure, and reliable platform for managing your culinary life.

### ✨ Key Features

- **🔐 User Authentication & Profiles** - Secure user management with dietary preferences and family settings
- **📝 Recipe Management** - Comprehensive recipe collection with search, categorization, and ratings
- **🗓️ Intelligent Meal Planning** - AI-powered meal planning with nutritional optimization
- **🛒 Smart Shopping Lists** - Automated shopping list generation from meal plans
- **📱 Progressive Web App** - Mobile-optimized interface with offline capabilities
- **📧 Email Notifications** - SMTP-powered notifications for meal planning and reminders

## 🏗️ Architecture Summary

### Technology Stack

| Component | Technology | Version | Purpose |
|-----------|------------|---------|---------|
| **Backend** | Rust + Axum | 1.90+ / 0.8+ | High-performance async web server |
| **Frontend** | Askama + Tailwind CSS | 0.14+ / 4.1+ | Server-side rendered templates |
| **Database** | SQLite + SQLx | 3.40+ / 0.8+ | Embedded database with type safety |
| **Testing** | Rust Test + Playwright | Built-in / Latest | Comprehensive test coverage |
| **Deployment** | Docker | Latest | Containerized deployment |

### Project Structure

```
imkitchen/
├── crates/                    # Bounded context crates
│   ├── imkitchen-shared/      # Common types and utilities
│   ├── imkitchen-user/        # User management
│   ├── imkitchen-recipe/      # Recipe management
│   ├── imkitchen-meal-planning/  # Meal planning engine
│   ├── imkitchen-shopping/    # Shopping list management
│   ├── imkitchen-notification/ # Email and notifications
│   └── imkitchen-web/         # Web server and templates
├── docs/                      # Documentation
│   ├── development/           # Developer guides
│   ├── architecture/          # Technical documentation
│   ├── deployment/            # Deployment guides
│   └── api/                   # API documentation
├── src/                       # CLI binary
└── README.md                  # This file
```

## 🚀 Quick Start

### Prerequisites

- **Rust 1.90+** - [Install Rust](https://rustup.rs/)
- **Node.js** (for Tailwind CSS) - [Install Node.js](https://nodejs.org/)
- **Git** - [Install Git](https://git-scm.com/)

### Installation

1. **Clone the repository**
   ```bash
   git clone https://github.com/your-org/imkitchen.git
   cd imkitchen
   ```

2. **Set up environment**
   ```bash
   cp .env.example .env
   # Edit .env with your configuration
   ```

3. **Install dependencies**
   ```bash
   # Install Rust tools
   cargo install cargo-watch sqlx-cli
   
   # Install Tailwind CSS
   npm install -g @tailwindcss/cli@next
   ```

4. **Initialize database**
   ```bash
   # Create database and run migrations
   cargo sqlx migrate run --database-url sqlite:imkitchen.db
   ```

5. **Build and run**
   ```bash
   # Build the project
   cargo build --workspace
   
   # Start the development server
   cargo run -- web start --port 3000
   ```

6. **Open your browser**
   ```
   http://localhost:3000
   ```

### Development Commands

```bash
# Start with auto-reload
cargo watch -x "run -- web start --port 3000"

# Run tests
cargo test --workspace

# Check code quality
cargo fmt --all
cargo clippy --workspace

# Build CSS
tailwindcss -i ./crates/imkitchen-web/static/css/input.css -o ./crates/imkitchen-web/static/css/tailwind.css --watch
```

## 📚 Documentation

### For Developers

- **[Development Setup](docs/development/setup.md)** - Detailed environment setup for all platforms
- **[Coding Standards](docs/development/coding-standards.md)** - Project coding conventions and best practices
- **[Testing Guide](docs/development/testing.md)** - TDD procedures and testing strategies
- **[Architecture Overview](docs/architecture/README.md)** - Technical architecture and design decisions

### For Users

- **[API Documentation](docs/api/README.md)** - Complete API reference and examples
- **[Deployment Guide](docs/deployment/README.md)** - Production deployment procedures
- **[Troubleshooting](docs/troubleshooting.md)** - Common issues and solutions

### For Contributors

- **[Contributing Guidelines](docs/development/contributing.md)** - How to contribute to the project
- **[Code Review Process](docs/development/code-review.md)** - Code review standards and procedures
- **[Onboarding Checklist](docs/development/onboarding.md)** - New developer onboarding guide

## 🔧 Configuration

### Required Environment Variables

```bash
# Application
DATABASE_URL=sqlite:imkitchen.db
RUST_LOG=info
SERVER_HOST=0.0.0.0
SERVER_PORT=3000

# Security
SESSION_SECRET=your-secret-key-here-32-chars-min
PASSWORD_SALT_ROUNDS=12

# SMTP Configuration (Optional)
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USERNAME=your-email@example.com
SMTP_PASSWORD=your-app-password
SMTP_FROM_EMAIL=noreply@imkitchen.com
SMTP_FROM_NAME=IMKitchen

# Feature Flags
ENABLE_REGISTRATION=true
ENABLE_EMAIL_VERIFICATION=true
```

See [Environment Configuration Guide](docs/development/environment.md) for detailed setup instructions.

## 🧪 Testing

The project maintains high test coverage with a comprehensive testing strategy:

```bash
# Run all tests
cargo test --workspace

# Run tests for specific crate
cargo test -p imkitchen-user

# Run with coverage
cargo test --workspace -- --test-threads=1

# Run E2E tests (requires server running)
npx playwright test
```

Test organization:
- **Unit Tests** - Domain logic and individual components
- **Integration Tests** - Cross-crate functionality and handlers
- **E2E Tests** - Complete user workflows with Playwright

## 🚀 Deployment

### Docker Deployment

```bash
# Build container
docker build -t imkitchen .

# Run container
docker run -p 3000:3000 \
  -e DATABASE_URL=sqlite:/data/imkitchen.db \
  -v imkitchen-data:/data \
  imkitchen
```

### Production Deployment

See the [Deployment Guide](docs/deployment/README.md) for detailed production deployment instructions including:
- Container orchestration
- Environment configuration
- SSL/TLS setup
- Monitoring and logging

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guidelines](docs/development/contributing.md) for details on:

- Code style and conventions
- Testing requirements
- Pull request process
- Issue reporting

### Development Workflow

1. Fork the repository
2. Create a feature branch
3. Write tests first (TDD)
4. Implement functionality
5. Ensure all tests pass
6. Submit a pull request

## 📞 Support

### Troubleshooting

Common issues and solutions are documented in our [Troubleshooting Guide](docs/troubleshooting.md).

### Getting Help

- **Issues** - [GitHub Issues](https://github.com/your-org/imkitchen/issues)
- **Documentation** - [Full documentation](docs/)
- **Contributing** - [Contributing Guidelines](docs/development/contributing.md)

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Built with [Rust](https://www.rust-lang.org/) for performance and safety
- UI powered by [Tailwind CSS](https://tailwindcss.com/) for responsive design
- Templates rendered with [Askama](https://djc.github.io/askama/) for type safety
- Data persistence with [SQLite](https://www.sqlite.org/) and [SQLx](https://github.com/launchbadge/sqlx)

---

**Getting Started?** → Check out our [Development Setup Guide](docs/development/setup.md)  
**Need Help?** → See our [Troubleshooting Guide](docs/troubleshooting.md)  
**Want to Contribute?** → Read our [Contributing Guidelines](docs/development/contributing.md)
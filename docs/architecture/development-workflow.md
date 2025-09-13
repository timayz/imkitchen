# Development Workflow

## Local Development Setup

### Prerequisites

```bash
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Node.js for frontend tooling
# Using nvm (recommended)
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
nvm install 20
nvm use 20

# Install PostgreSQL and Redis
# On macOS
brew install postgresql@17 redis

# On Ubuntu
sudo apt install postgresql-17 redis-server
```

### Initial Setup

```bash
# Clone repository
git clone <repository-url>
cd imkitchen

# Install Rust dependencies
cargo build

# Install frontend tooling dependencies (CSS/JS processing only)
npm install

# Set up environment variables
cp .env.example .env
# Edit .env with your database credentials and settings

# Set up database
createdb imkitchen_dev
sqlx database create
sqlx migrate run

# Generate initial data (optional)
cargo run --bin seed
```

### Development Commands

```bash
# Start all services (unified application)
docker-compose up -d postgres redis
cargo run

# Build CSS during development (in separate terminal)
npm run css:watch

# Run tests
cargo test                    # All Rust tests (unit + integration)
npm run test:e2e             # End-to-end browser tests

# Build production assets
npm run build                 # Compile CSS and optimize JS
cargo build --release        # Build optimized binary
```

## Environment Configuration

### Required Environment Variables

```bash
# Application (.env)
DATABASE_URL=postgresql://username:password@localhost/imkitchen_dev
REDIS_URL=redis://localhost:6379
JWT_SECRET=<your-jwt-secret-key>
VAPID_PRIVATE_KEY=<your-vapid-private-key>
VAPID_PUBLIC_KEY=<your-vapid-public-key>
SERVER_HOST=0.0.0.0
SERVER_PORT=3000
RUST_LOG=info
ENVIRONMENT=development
```

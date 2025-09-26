# Development Workflow

## Local Development Setup

### Prerequisites
```bash
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup update stable

# Install additional tools
cargo install cargo-watch sqlx-cli
npm install -g @tailwindcss/cli@next
```

### Initial Setup
```bash
# Clone repository and setup
git clone <repository-url>
cd imkitchen
cp .env.example .env

# Build all crates and run migrations
cargo build --workspace
cargo sqlx migrate run --database-url sqlite:imkitchen.db

# Download TwinSpark and compile CSS
curl -o ./crates/imkitchen-web/static/js/twinspark.js https://unpkg.com/twinspark@1/dist/twinspark.js
tailwindcss -i ./crates/imkitchen-web/static/css/input.css -o ./crates/imkitchen-web/static/css/tailwind.css --watch
```

### Development Commands
```bash
# Start all services
cargo run -- web start --port 3000

# Start with auto-reload
cargo watch -x "run -- web start --port 3000"

# Run tests
cargo test --workspace

# Run specific crate tests
cargo test -p imkitchen-meal-planning

# Check formatting and linting
cargo fmt --all
cargo clippy --workspace
```

## Environment Configuration

### Required Environment Variables
```bash
# Application (.env)
DATABASE_URL=sqlite:imkitchen.db
RUST_LOG=info
SERVER_HOST=0.0.0.0
SERVER_PORT=3000

# SMTP Configuration
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USERNAME=your-email@example.com
SMTP_PASSWORD=your-app-password
SMTP_FROM_EMAIL=noreply@imkitchen.com
SMTP_FROM_NAME=IMKitchen

# Security
SESSION_SECRET=your-secret-key-here-32-chars-min
PASSWORD_SALT_ROUNDS=12

# Feature Flags
ENABLE_REGISTRATION=true
ENABLE_EMAIL_VERIFICATION=true
ENABLE_COMMUNITY_FEATURES=true
```

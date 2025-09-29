# Development Environment Setup

This guide provides step-by-step instructions for setting up the IMKitchen development environment on different operating systems.

## Table of Contents

- [Prerequisites](#prerequisites)
- [macOS Setup](#macos-setup)
- [Linux Setup](#linux-setup)
- [Windows Setup](#windows-setup)
- [Docker Setup (Alternative)](#docker-setup-alternative)
- [Environment Variables](#environment-variables)
- [SMTP Configuration](#smtp-configuration)
- [Verification](#verification)
- [Troubleshooting](#troubleshooting)

## Prerequisites

All platforms require:
- **Git** - Version control
- **Internet connection** - For downloading dependencies

## macOS Setup

### 1. Install Rust

```bash
# Install Rust using rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Follow the prompts, then reload your shell
source ~/.zshrc  # or ~/.bash_profile

# Verify installation
rustc --version
cargo --version
```

### 2. Install Node.js (for Tailwind CSS)

```bash
# Using Homebrew (recommended)
brew install node

# Or download from https://nodejs.org/
# Verify installation
node --version
npm --version
```

### 3. Install Development Tools

```bash
# Install Rust development tools
cargo install cargo-watch sqlx-cli

# Install Tailwind CSS CLI
npm install -g @tailwindcss/cli@next

# Install optional tools
brew install git # if not already installed
```

### 4. Clone and Setup Project

```bash
# Clone the repository
git clone https://github.com/your-org/imkitchen.git
cd imkitchen

# Copy environment template
cp .env.example .env

# Edit environment variables (see Environment Variables section)
nano .env  # or use your preferred editor
```

### 5. Build and Run

```bash
# Build all crates
cargo build --workspace

# Initialize database
cargo sqlx migrate run --database-url sqlite:imkitchen.db

# Download TwinSpark JavaScript library
curl -o ./crates/imkitchen-web/static/js/twinspark.js https://unpkg.com/twinspark@1/dist/twinspark.js

# Build CSS (in separate terminal)
tailwindcss -i ./crates/imkitchen-web/static/css/input.css -o ./crates/imkitchen-web/static/css/tailwind.css --watch

# Start development server
cargo run -- web start --port 3000
```

## Linux Setup

### Ubuntu/Debian

```bash
# Update package manager
sudo apt update

# Install dependencies
sudo apt install -y curl build-essential pkg-config libssl-dev

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.bashrc

# Install Node.js
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt-get install -y nodejs

# Install development tools
cargo install cargo-watch sqlx-cli
npm install -g @tailwindcss/cli@next

# Clone and setup project
git clone https://github.com/your-org/imkitchen.git
cd imkitchen
cp .env.example .env

# Build and run (same as macOS steps 5)
```

### CentOS/RHEL/Fedora

```bash
# Install dependencies
sudo dnf install -y curl gcc openssl-devel pkg-config

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.bashrc

# Install Node.js
sudo dnf install -y nodejs npm

# Continue with same steps as Ubuntu
```

### Arch Linux

```bash
# Install dependencies
sudo pacman -S curl base-devel openssl pkg-config

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.bashrc

# Install Node.js
sudo pacman -S nodejs npm

# Continue with development tools installation
```

## Windows Setup

### Option 1: Windows Subsystem for Linux (WSL) - Recommended

```powershell
# Install WSL2 with Ubuntu (run as Administrator)
wsl --install

# Restart computer, then open Ubuntu terminal
# Follow the Linux Ubuntu setup instructions above
```

### Option 2: Native Windows

```powershell
# Install Rust using rustup-init.exe
# Download from: https://rustup.rs/
# Run the installer and follow prompts

# Install Node.js
# Download from: https://nodejs.org/
# Install the LTS version

# Install Git
# Download from: https://git-scm.com/download/win

# Open PowerShell or Command Prompt
# Verify installations
rustc --version
cargo --version
node --version
git --version

# Install development tools
cargo install cargo-watch sqlx-cli
npm install -g @tailwindcss/cli@next

# Clone and setup project
git clone https://github.com/your-org/imkitchen.git
cd imkitchen
copy .env.example .env

# Edit .env file with your preferred editor
notepad .env

# Build and run
cargo build --workspace
cargo sqlx migrate run --database-url sqlite:imkitchen.db

# Download TwinSpark (PowerShell)
Invoke-WebRequest -Uri "https://unpkg.com/twinspark@1/dist/twinspark.js" -OutFile "./crates/imkitchen-web/static/js/twinspark.js"

# Build CSS (in separate terminal)
tailwindcss -i ./crates/imkitchen-web/static/css/input.css -o ./crates/imkitchen-web/static/css/tailwind.css --watch

# Start development server
cargo run -- web start --port 3000
```

## Docker Setup (Alternative)

If you prefer containerized development:

```bash
# Clone repository
git clone https://github.com/your-org/imkitchen.git
cd imkitchen

# Build development container
docker build -f Dockerfile.dev -t imkitchen-dev .

# Run with volume mounting
docker run -it --rm \
  -p 3000:3000 \
  -v $(pwd):/workspace \
  -w /workspace \
  imkitchen-dev

# Inside container, run setup commands
cargo build --workspace
cargo sqlx migrate run --database-url sqlite:imkitchen.db
cargo run -- web start --port 3000 --host 0.0.0.0
```

## Environment Variables

Create a `.env` file in the project root with the following variables:

```bash
# Required - Application Configuration
DATABASE_URL=sqlite:imkitchen.db
RUST_LOG=info
SERVER_HOST=0.0.0.0
SERVER_PORT=3000

# Required - Security Configuration
SESSION_SECRET=your-very-secure-32-character-secret-key-here
PASSWORD_SALT_ROUNDS=12

# Optional - SMTP Configuration (for email features)
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USERNAME=your-email@example.com
SMTP_PASSWORD=your-app-password
SMTP_FROM_EMAIL=noreply@imkitchen.com
SMTP_FROM_NAME=IMKitchen
SMTP_SECURITY=starttls
SMTP_TIMEOUT=30

# Optional - Feature Flags
ENABLE_REGISTRATION=true
ENABLE_EMAIL_VERIFICATION=true
ENABLE_COMMUNITY_FEATURES=true

# Development - Additional Configuration
RUST_BACKTRACE=1
CARGO_WATCH_IGNORE="*.db"
```

### Environment Variable Descriptions

| Variable | Required | Description | Example |
|----------|----------|-------------|---------|
| `DATABASE_URL` | ✅ | SQLite database file path | `sqlite:imkitchen.db` |
| `RUST_LOG` | ✅ | Logging level | `info`, `debug`, `warn`, `error` |
| `SERVER_HOST` | ✅ | Server bind address | `0.0.0.0` (all interfaces) |
| `SERVER_PORT` | ✅ | Server port number | `3000` |
| `SESSION_SECRET` | ✅ | Secret key for session encryption | 32+ character random string |
| `PASSWORD_SALT_ROUNDS` | ✅ | Bcrypt salt rounds for passwords | `12` (recommended) |
| `SMTP_HOST` | ❌ | SMTP server hostname | `smtp.gmail.com` |
| `SMTP_PORT` | ❌ | SMTP server port | `587` (STARTTLS), `465` (SSL) |
| `SMTP_USERNAME` | ❌ | SMTP authentication username | Email address |
| `SMTP_PASSWORD` | ❌ | SMTP authentication password | App-specific password |
| `SMTP_FROM_EMAIL` | ❌ | Sender email address | `noreply@imkitchen.com` |
| `SMTP_FROM_NAME` | ❌ | Sender display name | `IMKitchen` |
| `SMTP_SECURITY` | ❌ | SMTP encryption method | `starttls`, `ssl`, `none` |
| `SMTP_TIMEOUT` | ❌ | SMTP connection timeout (seconds) | `30` |

## SMTP Configuration

### Gmail Setup

1. **Enable 2-Factor Authentication** on your Google account
2. **Create App Password**:
   - Go to Google Account settings
   - Security → 2-Step Verification → App passwords
   - Generate password for "Mail"
3. **Configure .env**:
   ```bash
   SMTP_HOST=smtp.gmail.com
   SMTP_PORT=587
   SMTP_USERNAME=your-email@gmail.com
   SMTP_PASSWORD=your-app-password
   SMTP_SECURITY=starttls
   ```

### Other Email Providers

#### SendGrid
```bash
SMTP_HOST=smtp.sendgrid.net
SMTP_PORT=587
SMTP_USERNAME=apikey
SMTP_PASSWORD=your-sendgrid-api-key
SMTP_SECURITY=starttls
```

#### Mailgun
```bash
SMTP_HOST=smtp.mailgun.org
SMTP_PORT=587
SMTP_USERNAME=your-mailgun-username
SMTP_PASSWORD=your-mailgun-password
SMTP_SECURITY=starttls
```

#### Development (Local Testing)
```bash
# Disable SMTP for local development
# Comment out or remove SMTP_* variables
# Application will log emails to console instead
```

## Verification

After setup, verify your installation:

### 1. Check Rust Installation
```bash
rustc --version
# Should show: rustc 1.90+ 

cargo --version
# Should show: cargo 1.90+
```

### 2. Check Node.js Installation
```bash
node --version
# Should show: v18+ or v20+

npm --version
# Should show: 8+
```

### 3. Check Project Build
```bash
cd imkitchen
cargo check --workspace
# Should complete without errors
```

### 4. Check Database Setup
```bash
cargo sqlx migrate info --database-url sqlite:imkitchen.db
# Should show migration status
```

### 5. Check Web Server
```bash
cargo run -- web start --port 3000
# Should start server without errors
# Visit http://localhost:3000
```

### 6. Run Tests
```bash
cargo test --workspace
# Should pass all tests
```

## Troubleshooting

### Common Issues

#### Rust Installation Issues
```bash
# Permission denied during installation
# Solution: Ensure you have write access to home directory
chmod 755 ~/.cargo

# rustc not found after installation
# Solution: Reload shell configuration
source ~/.bashrc  # or ~/.zshrc on macOS
```

#### Build Failures
```bash
# OpenSSL linking errors on Linux
sudo apt install pkg-config libssl-dev

# Missing C compiler
sudo apt install build-essential  # Ubuntu/Debian
sudo dnf install gcc               # CentOS/RHEL
```

#### Database Issues
```bash
# Database locked error
# Solution: Ensure no other instances are running
pkill imkitchen

# Migration failed
# Solution: Delete database and re-run migrations
rm imkitchen.db
cargo sqlx migrate run --database-url sqlite:imkitchen.db
```

#### SMTP Issues
```bash
# Authentication failed
# Solution: Check credentials and use app-specific passwords

# Connection timeout
# Solution: Check firewall and network connectivity
telnet smtp.gmail.com 587

# Permission denied on port
# Solution: Use port 3000+ or run with elevated privileges
```

#### CSS Build Issues
```bash
# Tailwind command not found
npm install -g @tailwindcss/cli@next

# CSS not updating
# Solution: Check input.css exists and restart watch process
```

For more troubleshooting help, see our [Troubleshooting Guide](../troubleshooting.md).

## Next Steps

Once your environment is set up:

1. **Read the [Architecture Overview](../architecture/README.md)**
2. **Review [Coding Standards](coding-standards.md)**
3. **Check the [Testing Guide](testing.md)**
4. **Explore the [API Documentation](../api/README.md)**
5. **Complete the [Onboarding Checklist](onboarding.md)**

## Development Workflow

### Daily Development

```bash
# Start development session
cd imkitchen

# Terminal 1: Start CSS watch
tailwindcss -i ./crates/imkitchen-web/static/css/input.css -o ./crates/imkitchen-web/static/css/tailwind.css --watch

# Terminal 2: Start server with auto-reload
cargo watch -x "run -- web start --port 3000"

# Terminal 3: Run tests in watch mode
cargo watch -x "test --workspace"
```

### Code Quality Checks

```bash
# Format code
cargo fmt --all

# Check for issues
cargo clippy --workspace

# Run all tests
cargo test --workspace

# Check security vulnerabilities
cargo audit
```
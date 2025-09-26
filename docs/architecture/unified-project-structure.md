# Unified Project Structure

```
imkitchen/
├── .github/                    # CI/CD workflows
│   └── workflows/
│       ├── ci.yml             # Rust testing and linting
│       └── deploy.yml         # Container build and deploy
├── crates/                     # Bounded context crates
│   ├── imkitchen-shared/       # Common types and utilities
│   │   ├── src/
│   │   │   ├── events/         # Domain event definitions
│   │   │   ├── types/          # Shared value objects
│   │   │   └── utils/          # Common utilities
│   │   └── Cargo.toml
│   ├── imkitchen-user/         # User bounded context
│   │   ├── src/
│   │   │   ├── domain/         # User domain logic
│   │   │   ├── commands/       # CQRS commands
│   │   │   ├── queries/        # CQRS queries
│   │   │   ├── projections/    # Evento projections
│   │   │   └── events/         # User domain events
│   │   └── Cargo.toml
│   ├── imkitchen-recipe/       # Recipe bounded context
│   ├── imkitchen-meal-planning/ # Meal planning bounded context
│   ├── imkitchen-shopping/     # Shopping bounded context
│   ├── imkitchen-notification/ # Notification bounded context
│   └── imkitchen-web/          # Web server library
│       ├── src/
│       │   ├── handlers/       # Axum request handlers
│       │   ├── middleware/     # HTTP middleware
│       │   ├── templates/      # Askama HTML templates
│       │   └── lib.rs          # Web server library
│       ├── templates/          # Askama template files
│       ├── static/             # Static assets
│       │   ├── css/            # Tailwind CSS output
│       │   ├── js/             # TwinSpark JavaScript library
│       │   └── images/         # Recipe and UI images
│       └── Cargo.toml
├── src/                        # CLI binary
│   └── main.rs                 # CLI entry point with clap
├── scripts/                    # Build and deployment scripts
│   ├── build-docker.sh         # Container build script
│   └── setup-dev.sh            # Development setup
├── docs/                       # Documentation
│   ├── prd.md
│   ├── front-end-spec.md
│   └── architecture.md
├── Dockerfile                  # Container definition
├── .env.example                # Environment template
├── Cargo.toml                  # Workspace configuration
└── README.md
```

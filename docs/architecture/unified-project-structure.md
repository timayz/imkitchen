# Unified Project Structure

```plaintext
imkitchen/
├── .github/                        # CI/CD workflows
│   └── workflows/
│       ├── ci.yml                  # Test and lint
│       └── deploy.yml              # Build and deploy
├── src/                            # Unified Rust application
│   ├── main.rs                     # Application entry point and server setup
│   ├── routes/                     # Route handlers
│   │   ├── mod.rs
│   │   ├── pages/                  # Full page handlers
│   │   │   ├── mod.rs
│   │   │   ├── dashboard.rs        # Dashboard page
│   │   │   ├── recipes.rs          # Recipe management pages
│   │   │   ├── planner.rs          # Meal planning interface
│   │   │   ├── cooking.rs          # Cook mode pages
│   │   │   └── auth.rs             # Authentication pages
│   │   └── fragments/              # HTML fragment handlers
│   │       ├── mod.rs
│   │       ├── recipe_card.rs      # Recipe card fragments
│   │       ├── timer_widget.rs     # Timer widget fragments
│   │       ├── meal_list.rs        # Meal list fragments
│   │       └── notifications.rs    # Notification fragments
│   ├── services/                   # Business logic services
│   │   ├── mod.rs
│   │   ├── recipe_parser.rs        # Recipe import/parsing
│   │   ├── meal_planner.rs         # AI meal planning
│   │   ├── timing_engine.rs        # Cooking coordination
│   │   └── notification.rs         # Push notifications
│   ├── repositories/               # Data access layer
│   │   ├── mod.rs
│   │   ├── user_repo.rs
│   │   ├── recipe_repo.rs
│   │   └── session_repo.rs
│   ├── models/                     # Domain models
│   │   ├── mod.rs
│   │   ├── user.rs
│   │   ├── recipe.rs
│   │   └── cooking.rs
│   ├── middleware/                 # HTTP middleware
│   │   ├── mod.rs
│   │   ├── auth.rs
│   │   ├── cors.rs
│   │   └── logging.rs
│   └── config/                     # Configuration management
│       ├── mod.rs
│       ├── database.rs
│       └── settings.rs
├── templates/                      # Askama templates (embedded in binary)
│   ├── layouts/                    # Base layouts
│   │   ├── main.html              # Main application layout
│   │   └── auth.html              # Authentication layout
│   ├── pages/                      # Full page templates
│   │   ├── dashboard.html         # Main dashboard
│   │   ├── recipes/               # Recipe management pages
│   │   │   ├── index.html         # Recipe library
│   │   │   ├── detail.html        # Recipe detail view
│   │   │   └── new.html           # Add/import recipe
│   │   ├── planner/               # Meal planning interface
│   │   │   └── index.html
│   │   ├── cook/                  # Cooking mode templates
│   │   │   ├── session.html       # Active cooking session
│   │   │   └── dashboard.html     # Cook mode dashboard
│   │   └── auth/                  # Authentication pages
│   │       ├── login.html
│   │       └── register.html
│   ├── components/                 # Reusable template fragments
│   │   ├── recipe_card.html
│   │   ├── timer_widget.html
│   │   ├── meal_planner_grid.html
│   │   └── navigation.html
│   └── partials/                   # Small reusable fragments
│       ├── form_input.html
│       └── notification.html
├── static/                         # Static assets served by axum
│   ├── css/                        # Compiled Tailwind CSS
│   │   └── main.css
│   ├── js/                         # Client-side JavaScript
│   │   ├── twinspark.js           # Reactivity library
│   │   ├── alpine.min.js          # State management
│   │   └── app.js                 # Application-specific JS
│   ├── images/                     # UI images and icons
│   │   ├── icons/
│   │   └── logo.svg
│   └── sw.js                       # Service worker for offline support
├── migrations/                     # Database migrations
│   ├── 001_initial_schema.sql
│   ├── 002_add_cooking_sessions.sql
│   └── 003_add_push_subscriptions.sql
├── tests/                          # All tests
│   ├── integration/                # API integration tests
│   │   ├── auth_tests.rs
│   │   ├── recipe_tests.rs
│   │   └── cooking_tests.rs
│   ├── unit/                       # Unit tests
│   │   ├── services/
│   │   ├── repositories/
│   │   └── utils/
│   └── e2e/                        # End-to-end tests
│       ├── auth_flow.spec.js
│       ├── recipe_management.spec.js
│       └── cooking_workflow.spec.js
├── infrastructure/                 # Infrastructure as Code
│   ├── docker-compose.yml         # Local development stack
│   ├── docker-compose.prod.yml    # Production configuration
│   ├── Dockerfile                 # Application container
│   ├── nginx/                     # Reverse proxy configuration
│   └── scripts/                   # Deployment and utility scripts
├── docs/                          # Documentation
│   ├── prd.md                     # Product Requirements Document
│   ├── front-end-spec.md          # UI/UX Specification
│   ├── architecture.md            # This document
│   └── api/                       # API documentation
├── .env.example                   # Environment variable template
├── Cargo.toml                     # Rust workspace configuration
├── package.json                   # Node.js for CSS/JS tooling only
├── tailwind.config.js            # Tailwind CSS configuration
└── README.md                      # Project documentation
```

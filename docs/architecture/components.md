# Components

## IMKitchen Web Server Component

**Responsibility:** HTTP request handling, Askama template rendering, TwinSpark fragment generation, and coordination between bounded context crates

**Key Interfaces:**
- HTTP endpoints returning HTML fragments for TwinSpark ts-target replacement
- Askama template rendering with TwinSpark attribute integration  
- Static asset serving for Tailwind CSS, local TwinSpark JavaScript, and images

**Dependencies:** All domain crates (user, recipe, meal-planning, shopping, notification), Askama template engine, Axum HTTP framework

**Technology Stack:** Axum 0.8+, Askama 0.14+, TwinSpark integration, Tailwind CSS compilation

## User Management Component

**Responsibility:** User authentication, profile management, preference storage, and dietary restriction handling with OWASP security compliance

**Key Interfaces:**
- User registration and login with email verification
- Profile CRUD operations with validation
- Preference management for meal planning algorithms

**Dependencies:** Evento event store, validator for input validation, lettre for email services

**Technology Stack:** Rust with validator 0.20+, Evento 1.1+, SQLite event store, SMTP integration

## Recipe Management Component

**Responsibility:** Recipe CRUD operations, community rating system, collection management, and recipe discovery with full-text search

**Key Interfaces:**
- Recipe creation and editing with ingredient parsing
- Rating and review system with community moderation
- Collection management with public/private settings

**Dependencies:** Evento event store, search indexing, image storage management

**Technology Stack:** Rust with Evento 1.1+, SQLite full-text search, file system storage

## Meal Planning Engine Component

**Responsibility:** Intelligent weekly meal plan generation, recipe rotation algorithms, difficulty balancing, and real-time plan adaptation

**Key Interfaces:**
- "Fill My Week" automation with constraint satisfaction
- Recipe rotation logic preventing repetition
- Real-time meal rescheduling with dependency updates

**Dependencies:** Recipe component for recipe data, User component for preferences, advanced scheduling algorithms

**Technology Stack:** Rust with complex algorithm implementation, Evento for state management, SQLite for optimization data

## Shopping List Manager Component

**Responsibility:** Automatic shopping list generation, ingredient consolidation, quantity optimization, and family sharing coordination

**Key Interfaces:**
- Shopping list generation from meal plans
- Ingredient quantity optimization and unit conversion
- Family sharing with real-time synchronization

**Dependencies:** Meal planning component for meal data, notification component for sharing alerts

**Technology Stack:** Rust with decimal calculations, Evento for state synchronization, SQLite for shopping data

## Notification Service Component

**Responsibility:** Email delivery, preparation reminders, shopping notifications, and community interaction alerts

**Key Interfaces:**
- SMTP email sending with template rendering
- Scheduled preparation reminders with timing optimization
- Real-time notification delivery with retry logic

**Dependencies:** External SMTP service, all domain components for notification triggers

**Technology Stack:** Lettre 0.11+ for SMTP, Rust async scheduling, Evento for reliable delivery

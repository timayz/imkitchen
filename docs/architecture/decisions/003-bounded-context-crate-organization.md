# ADR-003: Bounded Context Crate Organization

## Status
Accepted

## Context
IMKitchen is a complex domain with multiple interconnected concepts: users, recipes, kitchen management, inventory, meal planning, and notifications. As the application grows, we need a code organization strategy that:

- **Maintains Clear Boundaries**: Prevents tight coupling between different domain areas
- **Enables Independent Development**: Teams can work on different areas without conflicts
- **Supports Domain Evolution**: Each domain area can evolve at its own pace
- **Enforces Business Rules**: Domain logic stays within appropriate boundaries
- **Facilitates Testing**: Each domain can be tested in isolation
- **Enables Microservice Migration**: Clear boundaries support future service extraction

Traditional monolithic organization patterns often lead to circular dependencies, unclear ownership, and business logic scattered across layers.

## Decision
We will organize IMKitchen using **Bounded Context Crate Organization** based on Domain-Driven Design principles:

1. **Separate Crates per Bounded Context**: Each domain area gets its own Rust crate
2. **Shared Types Crate**: Common types and traits shared across contexts
3. **Web Crate**: HTTP handlers and presentation layer
4. **Clear Dependency Rules**: Domain crates cannot depend on web crate
5. **Event-Driven Communication**: Contexts communicate through domain events

## Alternatives Considered

### Traditional Layered Architecture
```
src/
├── controllers/
├── services/
├── models/
├── repositories/
└── utils/
```

**Pros:**
- Familiar to most developers
- Clear technical separation
- Simple to understand initially

**Cons:**
- Business logic scattered across layers
- Circular dependencies common
- Difficult to maintain as project grows
- No clear domain boundaries
- Hard to test business logic in isolation

### Feature-Based Organization
```
src/
├── recipes/
├── users/
├── inventory/
├── meal_planning/
└── shared/
```

**Pros:**
- Features grouped together
- Easier to locate related code
- Some domain separation

**Cons:**
- Still allows circular dependencies
- No enforced boundaries
- Shared code becomes dumping ground
- Difficult to extract services later

### Microservices from Day One
**Pros:**
- Complete service isolation
- Independent deployment
- Technology diversity possible

**Cons:**
- Premature complexity for startup
- Network overhead and latency
- Distributed system complexity
- Harder to refactor domain boundaries

### Single Crate with Modules
**Pros:**
- Simple build and deployment
- Easy refactoring within crate
- No dependency management complexity

**Cons:**
- No enforced boundaries
- Risk of tight coupling
- Single point of compilation
- Difficult to extract services later

## Consequences

### Positive
- **Enforced Boundaries**: Cargo prevents unauthorized dependencies between contexts
- **Independent Development**: Teams can work on separate crates without merge conflicts
- **Clear Ownership**: Each bounded context has clear responsibility and ownership
- **Testability**: Domain logic can be tested without web framework dependencies
- **Performance**: Only compile changed crates, faster build times
- **Future Migration**: Clear boundaries enable microservice extraction
- **Domain Modeling**: Forces proper domain modeling and business rule placement
- **Reusability**: Domain crates can be reused in different applications (CLI, mobile backend)

### Negative
- **Initial Complexity**: More complex project structure than simple modules
- **Dependency Management**: Need to manage inter-crate dependencies carefully
- **Learning Curve**: Team needs to understand DDD concepts and boundaries
- **Potential Over-Engineering**: Risk of creating too many small crates initially

### Risks
- **Incorrect Boundaries**: Risk of defining wrong bounded context boundaries
  - *Mitigation*: Start with broader contexts, split as understanding improves
- **Circular Dependencies**: Risk of creating dependency cycles between crates
  - *Mitigation*: Use shared types crate and event-driven communication
- **Performance Overhead**: Multiple crate compilation might be slower initially
  - *Mitigation*: Use Cargo workspaces and incremental compilation

## Implementation Notes

### Crate Structure
```
imkitchen/
├── Cargo.toml (workspace)
├── crates/
│   ├── imkitchen-shared/          # Shared types and traits
│   ├── imkitchen-user/            # User management domain
│   ├── imkitchen-recipe/          # Recipe and cooking domain
│   ├── imkitchen-inventory/       # Inventory management domain  
│   ├── imkitchen-meal-planning/   # Meal planning domain
│   ├── imkitchen-notification/    # Notification domain
│   └── imkitchen-web/             # HTTP handlers and presentation
├── docs/
└── README.md
```

### Workspace Configuration
```toml
# Root Cargo.toml
[workspace]
members = [
    "crates/imkitchen-shared",
    "crates/imkitchen-user", 
    "crates/imkitchen-recipe",
    "crates/imkitchen-inventory",
    "crates/imkitchen-meal-planning",
    "crates/imkitchen-notification",
    "crates/imkitchen-web",
]

[workspace.dependencies]
# Shared dependencies across all crates
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
sqlx = { version = "0.8", features = ["sqlite", "runtime-tokio-rustls"] }
```

### Bounded Context Definitions

#### User Context (`imkitchen-user`)
**Responsibility**: User registration, authentication, profile management, preferences
```rust
// Core domain concepts
pub struct User {
    pub id: UserId,
    pub email: Email,
    pub profile: UserProfile,
    pub preferences: UserPreferences,
    pub created_at: DateTime<Utc>,
}

// Domain events
pub enum UserEvent {
    UserRegistered { id: UserId, email: Email, profile: UserProfile },
    UserProfileUpdated { id: UserId, profile: UserProfile },
    UserPreferencesChanged { id: UserId, preferences: UserPreferences },
}

// Domain services
pub trait UserRepository: Send + Sync {
    async fn save(&self, user: &User) -> Result<(), UserError>;
    async fn find_by_email(&self, email: &Email) -> Result<Option<User>, UserError>;
}
```

#### Recipe Context (`imkitchen-recipe`)
**Responsibility**: Recipe creation, cooking instructions, nutritional information, ratings
```rust
// Core domain concepts
pub struct Recipe {
    pub id: RecipeId,
    pub name: String,
    pub ingredients: Vec<Ingredient>,
    pub instructions: Vec<CookingStep>,
    pub nutrition: NutritionalInfo,
    pub created_by: UserId, // Reference to User context
}

// Domain events
pub enum RecipeEvent {
    RecipeCreated { id: RecipeId, recipe: Recipe },
    RecipeRated { recipe_id: RecipeId, user_id: UserId, rating: Rating },
    RecipeShared { recipe_id: RecipeId, shared_by: UserId, shared_with: Vec<UserId> },
}
```

#### Inventory Context (`imkitchen-inventory`)
**Responsibility**: Ingredient tracking, expiration management, shopping lists
```rust
// Core domain concepts
pub struct InventoryItem {
    pub id: InventoryItemId,
    pub ingredient: IngredientReference,
    pub quantity: Quantity,
    pub expiration_date: Option<DateTime<Utc>>,
    pub location: StorageLocation,
}

// Domain events  
pub enum InventoryEvent {
    ItemAdded { item: InventoryItem },
    ItemConsumed { item_id: InventoryItemId, quantity: Quantity },
    ItemExpired { item_id: InventoryItemId },
}
```

### Dependency Rules
```rust
// ✅ Allowed dependencies
imkitchen-web → imkitchen-user
imkitchen-web → imkitchen-recipe
imkitchen-user → imkitchen-shared
imkitchen-recipe → imkitchen-shared

// ❌ Forbidden dependencies
imkitchen-user → imkitchen-web
imkitchen-recipe → imkitchen-user (direct)
imkitchen-inventory → imkitchen-recipe (direct)
```

### Inter-Context Communication
```rust
// Shared event bus for cross-context communication
#[async_trait]
pub trait EventBus: Send + Sync {
    async fn publish<E: DomainEvent>(&self, event: E) -> Result<(), EventBusError>;
    async fn subscribe<E: DomainEvent>(&self, handler: Box<dyn EventHandler<E>>) -> Result<(), EventBusError>;
}

// Example: Recipe context reacting to user events
pub struct RecipeUserEventHandler {
    recipe_repository: Arc<dyn RecipeRepository>,
}

#[async_trait]
impl EventHandler<UserEvent> for RecipeUserEventHandler {
    async fn handle(&self, event: &UserEvent) -> Result<(), EventHandlerError> {
        match event {
            UserEvent::UserDeleted { id } => {
                // Handle user deletion by updating recipe ownership
                self.recipe_repository.reassign_orphaned_recipes(id).await?;
            }
            _ => {} // Ignore other user events
        }
        Ok(())
    }
}
```

### Shared Types Crate
```rust
// imkitchen-shared/src/lib.rs
pub mod events;
pub mod identifiers;
pub mod errors;
pub mod time;

// Common identifier types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RecipeId(pub String);

// Common error handling
pub trait DomainError: std::error::Error + Send + Sync + 'static {}

// Common event trait
pub trait DomainEvent: Serialize + DeserializeOwned + Clone + Debug + Send + Sync {
    fn event_type(&self) -> &'static str;
    fn aggregate_id(&self) -> String;
}
```

### Web Crate Organization
```rust
// imkitchen-web/src/lib.rs
pub mod handlers;
pub mod middleware;
pub mod templates;
pub mod static_files;

// Handler organization by context
// handlers/
├── user/
│   ├── registration.rs
│   ├── authentication.rs
│   └── profile.rs
├── recipe/
│   ├── creation.rs
│   ├── browsing.rs
│   └── cooking.rs
└── inventory/
    ├── management.rs
    └── shopping_list.rs
```

### Testing Strategy
```rust
// Context-specific testing
// crates/imkitchen-user/tests/
├── integration/
│   ├── user_registration_test.rs
│   └── user_authentication_test.rs
├── unit/
│   ├── user_domain_test.rs
│   └── user_validation_test.rs
└── fixtures/
    └── user_fixtures.rs

// Cross-context integration testing
// tests/integration/
├── user_recipe_integration_test.rs
├── recipe_inventory_integration_test.rs
└── full_workflow_test.rs
```

### Migration Path
```rust
// Future microservice extraction
// Each crate can become a separate service:

// User Service
imkitchen-user → user-service (HTTP API)

// Recipe Service  
imkitchen-recipe → recipe-service (HTTP API)

// API Gateway
imkitchen-web → api-gateway (aggregates services)
```

## References
- [Domain-Driven Design by Eric Evans](https://www.domainlanguage.com/ddd/)
- [Implementing Domain-Driven Design by Vaughn Vernon](https://kalele.io/books/)
- [Rust Cargo Workspaces](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html)
- [Bounded Context Pattern](https://martinfowler.com/bliki/BoundedContext.html)
- [Strategic Domain-Driven Design](https://thedomaindrivendesign.io/strategic-domain-driven-design/)
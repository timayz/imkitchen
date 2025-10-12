# Technical Specification: Recipe Management System

Date: 2025-10-11
Author: Jonathan
Epic ID: 2
Status: Draft

---

## Overview

Epic 2 delivers the Recipe Management System, enabling users to create, organize, share, and discover recipes within the imkitchen platform. This epic implements the core recipe domain with full CRUD operations, collection management, privacy controls, community sharing, and rating/review functionality. The system leverages event sourcing via evento to maintain full audit trails of recipe changes and supports both private recipe libraries and public community discovery with SEO-optimized pages.

**Key Capabilities:**
- Recipe creation with ingredients, instructions, timing, and advance preparation requirements
- Recipe editing and deletion with ownership verification
- Organization into user-defined collections with automatic tagging
- Privacy controls (private vs. community-shared recipes)
- Favorite marking for meal planning algorithm integration
- Community recipe discovery with filtering (rating, cuisine, dietary, prep time)
- Recipe rating and review system (1-5 stars with text reviews)
- MinIO integration for recipe image storage and serving
- SEO-friendly community pages with Open Graph/Schema.org markup

**Business Value:**
This epic enables users to build their personal recipe libraries and access community-curated content, directly supporting the primary user journey of recipe variety expansion. The freemium 10-recipe limit enforcement drives premium conversions, while community sharing creates network effects for organic growth.

## Objectives and Scope

### Objectives

1. **Recipe Library Management**: Enable users to create, edit, delete, and organize recipes with full metadata (ingredients, instructions, timing, dietary attributes)

2. **Privacy and Sharing Controls**: Support private recipe libraries with optional community sharing, ensuring user data ownership while enabling content network effects

3. **Community Discovery**: Provide SEO-optimized public recipe discovery with filtering, search, and quality signals (ratings/reviews) to drive user acquisition

4. **Meal Planning Integration**: Maintain favorite recipe tracking and complexity metadata to feed the intelligent meal planning algorithm

5. **Freemium Conversion**: Enforce 10-recipe limit for free tier users, driving premium upgrades for unlimited recipe access

6. **Image Management**: Integrate MinIO for scalable recipe image storage with upload, retrieval, and deletion operations

7. **Quality Signals**: Implement rating and review system to surface high-quality community recipes and build user trust

### In Scope

**Recipe Management (Stories 1-4):**
- Create recipe with full details (title, ingredients, instructions, prep/cook time, advance prep hours, serving size)
- Edit existing recipes (ownership verification required)
- Delete recipes (soft delete with event sourcing)
- Recipe image upload/management via MinIO
- Recipe metadata extraction (complexity, cuisine type, dietary attributes)

**Organization and Favorites (Stories 5-6):**
- User-defined recipe collections (create, rename, delete collections)
- Assign recipes to collections (many-to-many relationship)
- Mark/unmark recipes as favorites for meal planning
- Automatic tagging based on recipe attributes (complexity, cuisine, dietary)

**Privacy and Sharing (Stories 7-8):**
- Privacy toggle (private vs. shared)
- Shared recipes visible in public community discovery
- Recipe attribution to original creator
- Prevent editing of other users' recipes

**Community Discovery (Stories 9-12):**
- Public recipe feed with pagination
- Filter by rating, cuisine, prep time, dietary preferences
- Search by recipe title and ingredients
- SEO-optimized recipe detail pages (Open Graph, Schema.org Recipe markup)
- Guest access (browse without auth, must register to add/rate)

**Ratings and Reviews (Stories 13-15):**
- Rate recipes 1-5 stars (one rating per user per recipe)
- Write text reviews (optional)
- Aggregate ratings display (average score, review count)
- Edit/delete own reviews
- Review moderation flags (future enhancement)

### Out of Scope

**Deferred to Future Epics:**
- Recipe import from URLs or files (MVP: manual entry only)
- Ingredient substitution suggestions
- Nutritional analysis and macro tracking
- Recipe scaling calculator (serving size adjustment)
- Video cooking instructions
- Recipe versioning and fork/clone functionality
- Advanced search with natural language queries
- Recipe recommendation engine based on user preferences
- Collaborative recipe editing
- Recipe print formatting and PDF export
- Social media sharing integration (Twitter, Pinterest, Instagram)
- Recipe contests and community challenges

## System Architecture Alignment

### Architecture Context

The Recipe Management System aligns with the event-sourced monolith architecture using evento for CQRS and Axum for server-side rendering. All recipe state changes are captured as immutable events, with read models materialized via evento subscriptions for query optimization.

**Architectural Components:**

1. **Domain Crate**: `crates/recipe/`
   - `RecipeAggregate`: Event-sourced aggregate managing recipe lifecycle
   - Commands: `CreateRecipe`, `UpdateRecipe`, `DeleteRecipe`, `FavoriteRecipe`, `ShareRecipe`
   - Events: `RecipeCreated`, `RecipeUpdated`, `RecipeDeleted`, `RecipeFavorited`, `RecipeShared`
   - Read Models: Recipe queries and projections from event stream

2. **Ratings Domain Crate**: `crates/ratings/`
   - `RatingAggregate`: Event-sourced aggregate for recipe ratings
   - Commands: `RateRecipe`, `UpdateRating`, `DeleteRating`
   - Events: `RecipeRated`, `RatingUpdated`, `RatingDeleted`
   - Read Models: Aggregate rating projections per recipe

3. **HTTP Routes** (in root binary `src/routes/recipes.rs`, `src/routes/discover.rs`):
   - Recipe CRUD endpoints (authenticated)
   - Community discovery endpoints (public with SEO)
   - Rating/review endpoints (authenticated actions)

4. **MinIO Integration** (in `recipe` crate):
   - Image upload to `imkitchen-recipes` bucket
   - Pre-signed URL generation for public access
   - Image deletion on recipe removal

5. **Read Model Tables** (SQLite):
   - `recipes`: Denormalized recipe data for queries
   - `recipe_collections`: User-defined collections
   - `recipe_collection_assignments`: Many-to-many recipe-to-collection
   - `ratings`: Recipe ratings and reviews
   - `recipe_tags`: Automatic and manual tags

### Integration Points

**Upstream Dependencies:**
- `user` crate: User ID validation, freemium tier enforcement
- Authentication middleware: JWT validation for protected routes
- MinIO service: S3-compatible storage for recipe images

**Downstream Dependencies:**
- `meal_planning` crate: Consumes favorite recipes and complexity metadata
- `shopping` crate: Consumes recipe ingredients for shopping list generation
- Event bus (evento): Publishes `RecipeCreated`, `RecipeFavorited` events for cross-domain subscriptions

**External Services:**
- MinIO (rust-s3 crate): Recipe image storage
- Web crawler access: Public `/discover` routes for SEO indexing

### Event Flow

**Recipe Creation Flow:**
```
User POST /recipes (multipart/form-data with image)
  ↓
Axum Handler: Validate form, upload image to MinIO
  ↓
Domain Command: CreateRecipe { title, ingredients, image_url, ... }
  ↓
RecipeAggregate: Apply business rules (recipe limit for free tier)
  ↓
Event: RecipeCreated { recipe_id, user_id, title, ... }
  ↓
evento: Write event to event stream
  ↓
Read Model Subscription: Insert into `recipes` table
  ↓
Response: 302 Redirect to /recipes/:id
```

**Recipe Rating Flow:**
```
User POST /discover/:id/rate (authenticated)
  ↓
Axum Handler: Validate rating (1-5 stars)
  ↓
Domain Command: RateRecipe { recipe_id, user_id, stars, review_text }
  ↓
RatingAggregate: Check if user already rated (update vs. create)
  ↓
Event: RecipeRated { recipe_id, user_id, stars, review_text }
  ↓
evento: Write event to event stream
  ↓
Read Model Subscription: Insert/update `ratings` table, recalculate aggregate rating
  ↓
Response: 200 OK with updated recipe detail HTML fragment (TwinSpark)
```

## Detailed Design

### Services and Modules

#### 1. Recipe Domain Crate (`crates/recipe/`)

**File Structure:**
```
crates/recipe/
├── Cargo.toml
├── src/
│   ├── lib.rs                  # Public API exports
│   ├── aggregate.rs            # RecipeAggregate (evento)
│   ├── commands.rs             # Command structs and handlers
│   ├── events.rs               # Event definitions (bincode serialization)
│   ├── read_model.rs           # Query functions for recipes table
│   ├── collections.rs          # Collection management logic
│   ├── image_storage.rs        # MinIO integration (rust-s3)
│   ├── tagging.rs              # Automatic tag extraction
│   ├── validation.rs           # Recipe validation rules
│   └── error.rs                # Domain-specific errors
└── tests/
    ├── aggregate_tests.rs      # Recipe aggregate tests
    ├── read_model_tests.rs     # Query tests
    └── fixtures.rs             # Test data fixtures
```

**RecipeAggregate Implementation:**

```rust
use evento::{aggregator, AggregatorName, EventDetails};
use serde::{Deserialize, Serialize};
use bincode::{Encode, Decode};

#[derive(Default, Serialize, Deserialize, Encode, Decode, Clone, Debug)]
pub struct RecipeAggregate {
    pub id: String,
    pub user_id: String,
    pub title: String,
    pub ingredients: Vec<Ingredient>,
    pub instructions: Vec<String>,
    pub prep_time_min: u32,
    pub cook_time_min: u32,
    pub advance_prep_hours: Option<u32>,
    pub serving_size: u32,
    pub image_url: Option<String>,
    pub complexity: Complexity,
    pub cuisine: Option<String>,
    pub dietary_tags: Vec<String>,
    pub is_favorite: bool,
    pub is_shared: bool,
    pub deleted_at: Option<String>,
}

#[derive(Serialize, Deserialize, Encode, Decode, Clone, Debug)]
pub struct Ingredient {
    pub name: String,
    pub quantity: f32,
    pub unit: String,
}

#[derive(Serialize, Deserialize, Encode, Decode, Clone, Debug)]
pub enum Complexity {
    Simple,
    Moderate,
    Complex,
}

// Event definitions
#[derive(AggregatorName, Encode, Decode)]
pub struct RecipeCreated {
    pub user_id: String,
    pub title: String,
    pub ingredients: Vec<Ingredient>,
    pub instructions: Vec<String>,
    pub prep_time_min: u32,
    pub cook_time_min: u32,
    pub advance_prep_hours: Option<u32>,
    pub serving_size: u32,
    pub image_url: Option<String>,
}

#[derive(AggregatorName, Encode, Decode)]
pub struct RecipeUpdated {
    pub title: Option<String>,
    pub ingredients: Option<Vec<Ingredient>>,
    pub instructions: Option<Vec<String>>,
    // ... (partial update fields)
}

#[derive(AggregatorName, Encode, Decode)]
pub struct RecipeFavorited {
    pub favorited: bool,
}

#[derive(AggregatorName, Encode, Decode)]
pub struct RecipeShared {
    pub shared: bool,
}

#[derive(AggregatorName, Encode, Decode)]
pub struct RecipeDeleted {
    pub deleted_at: String,
}

// Aggregate event handlers
#[aggregator]
impl RecipeAggregate {
    pub async fn recipe_created(&mut self, event: EventDetails<RecipeCreated>) -> anyhow::Result<()> {
        self.id = event.aggregator_id.clone();
        self.user_id = event.data.user_id.clone();
        self.title = event.data.title.clone();
        self.ingredients = event.data.ingredients.clone();
        self.instructions = event.data.instructions.clone();
        self.prep_time_min = event.data.prep_time_min;
        self.cook_time_min = event.data.cook_time_min;
        self.advance_prep_hours = event.data.advance_prep_hours;
        self.serving_size = event.data.serving_size;
        self.image_url = event.data.image_url.clone();
        self.complexity = calculate_complexity(self.prep_time_min, self.cook_time_min, &self.instructions);
        Ok(())
    }

    pub async fn recipe_updated(&mut self, event: EventDetails<RecipeUpdated>) -> anyhow::Result<()> {
        if let Some(title) = &event.data.title {
            self.title = title.clone();
        }
        if let Some(ingredients) = &event.data.ingredients {
            self.ingredients = ingredients.clone();
        }
        // ... (apply partial updates)
        Ok(())
    }

    pub async fn recipe_favorited(&mut self, event: EventDetails<RecipeFavorited>) -> anyhow::Result<()> {
        self.is_favorite = event.data.favorited;
        Ok(())
    }

    pub async fn recipe_shared(&mut self, event: EventDetails<RecipeShared>) -> anyhow::Result<()> {
        self.is_shared = event.data.shared;
        Ok(())
    }

    pub async fn recipe_deleted(&mut self, event: EventDetails<RecipeDeleted>) -> anyhow::Result<()> {
        self.deleted_at = Some(event.data.deleted_at.clone());
        Ok(())
    }
}

fn calculate_complexity(prep_min: u32, cook_min: u32, instructions: &[String]) -> Complexity {
    let total_time = prep_min + cook_min;
    let step_count = instructions.len();

    if total_time <= 30 && step_count <= 5 {
        Complexity::Simple
    } else if total_time <= 60 && step_count <= 10 {
        Complexity::Moderate
    } else {
        Complexity::Complex
    }
}
```

**Command Handlers:**

```rust
// commands.rs
use validator::Validate;
use evento::{Executor, create};

#[derive(Deserialize, Validate)]
pub struct CreateRecipeCommand {
    #[validate(length(min = 3, max = 200))]
    pub title: String,

    #[validate(length(min = 1))]
    pub ingredients: Vec<Ingredient>,

    #[validate(length(min = 1))]
    pub instructions: Vec<String>,

    #[validate(range(min = 1, max = 999))]
    pub prep_time_min: u32,

    #[validate(range(min = 1, max = 999))]
    pub cook_time_min: u32,

    pub advance_prep_hours: Option<u32>,

    #[validate(range(min = 1, max = 100))]
    pub serving_size: u32,

    pub image_url: Option<String>,
}

pub async fn create_recipe<E: Executor>(
    user_id: String,
    user_tier: UserTier,
    cmd: CreateRecipeCommand,
    executor: &E,
) -> Result<String, RecipeError> {
    // Validate command
    cmd.validate()?;

    // Check freemium limit
    let recipe_count = read_model::count_user_recipes(&user_id, executor).await?;
    if user_tier == UserTier::Free && recipe_count >= 10 {
        return Err(RecipeError::RecipeLimitReached);
    }

    // Create recipe aggregate with RecipeCreated event
    let recipe_id = create::<RecipeAggregate>()
        .data(&RecipeCreated {
            user_id: user_id.clone(),
            title: cmd.title,
            ingredients: cmd.ingredients,
            instructions: cmd.instructions,
            prep_time_min: cmd.prep_time_min,
            cook_time_min: cmd.cook_time_min,
            advance_prep_hours: cmd.advance_prep_hours,
            serving_size: cmd.serving_size,
            image_url: cmd.image_url,
        })?
        .metadata(&user_id)?
        .commit(executor)
        .await?;

    Ok(recipe_id)
}

pub async fn update_recipe<E: Executor>(
    recipe_id: String,
    user_id: String,
    cmd: UpdateRecipeCommand,
    executor: &E,
) -> Result<(), RecipeError> {
    // Load aggregate from event stream
    let recipe = evento::get::<RecipeAggregate>(&recipe_id, executor).await?;

    // Verify ownership
    if recipe.user_id != user_id {
        return Err(RecipeError::Unauthorized);
    }

    // Check if deleted
    if recipe.deleted_at.is_some() {
        return Err(RecipeError::RecipeDeleted);
    }

    // Validate command
    cmd.validate()?;

    // Apply RecipeUpdated event
    evento::update::<RecipeAggregate>(&recipe_id)
        .data(&RecipeUpdated {
            title: cmd.title,
            ingredients: cmd.ingredients,
            instructions: cmd.instructions,
            prep_time_min: cmd.prep_time_min,
            cook_time_min: cmd.cook_time_min,
            advance_prep_hours: cmd.advance_prep_hours,
            serving_size: cmd.serving_size,
        })?
        .metadata(&user_id)?
        .commit(executor)
        .await?;

    Ok(())
}

pub async fn delete_recipe<E: Executor>(
    recipe_id: String,
    user_id: String,
    executor: &E,
) -> Result<(), RecipeError> {
    // Load aggregate
    let recipe = evento::get::<RecipeAggregate>(&recipe_id, executor).await?;

    // Verify ownership
    if recipe.user_id != user_id {
        return Err(RecipeError::Unauthorized);
    }

    // Soft delete with event
    evento::update::<RecipeAggregate>(&recipe_id)
        .data(&RecipeDeleted {
            deleted_at: chrono::Utc::now().to_rfc3339(),
        })?
        .metadata(&user_id)?
        .commit(executor)
        .await?;

    // Delete image from MinIO if exists
    if let Some(image_url) = recipe.image_url {
        image_storage::delete_image(&image_url).await?;
    }

    Ok(())
}

pub async fn favorite_recipe<E: Executor>(
    recipe_id: String,
    user_id: String,
    favorited: bool,
    executor: &E,
) -> Result<(), RecipeError> {
    let recipe = evento::get::<RecipeAggregate>(&recipe_id, executor).await?;

    // Verify ownership
    if recipe.user_id != user_id {
        return Err(RecipeError::Unauthorized);
    }

    evento::update::<RecipeAggregate>(&recipe_id)
        .data(&RecipeFavorited { favorited })?
        .metadata(&user_id)?
        .commit(executor)
        .await?;

    Ok(())
}

pub async fn share_recipe<E: Executor>(
    recipe_id: String,
    user_id: String,
    shared: bool,
    executor: &E,
) -> Result<(), RecipeError> {
    let recipe = evento::get::<RecipeAggregate>(&recipe_id, executor).await?;

    if recipe.user_id != user_id {
        return Err(RecipeError::Unauthorized);
    }

    evento::update::<RecipeAggregate>(&recipe_id)
        .data(&RecipeShared { shared })?
        .metadata(&user_id)?
        .commit(executor)
        .await?;

    Ok(())
}
```

#### 2. Ratings Domain Crate (`crates/ratings/`)

**RatingAggregate Implementation:**

```rust
#[derive(Default, Serialize, Deserialize, Encode, Decode, Clone, Debug)]
pub struct RatingAggregate {
    pub id: String,
    pub recipe_id: String,
    pub user_id: String,
    pub stars: u8,
    pub review_text: Option<String>,
    pub created_at: String,
    pub updated_at: Option<String>,
}

#[derive(AggregatorName, Encode, Decode)]
pub struct RecipeRated {
    pub recipe_id: String,
    pub user_id: String,
    pub stars: u8,
    pub review_text: Option<String>,
}

#[derive(AggregatorName, Encode, Decode)]
pub struct RatingUpdated {
    pub stars: u8,
    pub review_text: Option<String>,
}

#[aggregator]
impl RatingAggregate {
    pub async fn recipe_rated(&mut self, event: EventDetails<RecipeRated>) -> anyhow::Result<()> {
        self.id = event.aggregator_id.clone();
        self.recipe_id = event.data.recipe_id.clone();
        self.user_id = event.data.user_id.clone();
        self.stars = event.data.stars;
        self.review_text = event.data.review_text.clone();
        self.created_at = chrono::Utc::now().to_rfc3339();
        Ok(())
    }

    pub async fn rating_updated(&mut self, event: EventDetails<RatingUpdated>) -> anyhow::Result<()> {
        self.stars = event.data.stars;
        self.review_text = event.data.review_text.clone();
        self.updated_at = Some(chrono::Utc::now().to_rfc3339());
        Ok(())
    }
}

pub async fn rate_recipe<E: Executor>(
    recipe_id: String,
    user_id: String,
    stars: u8,
    review_text: Option<String>,
    executor: &E,
) -> Result<String, RatingError> {
    // Validate stars (1-5)
    if stars < 1 || stars > 5 {
        return Err(RatingError::InvalidStars);
    }

    // Check if user already rated this recipe
    if let Some(existing_rating_id) = read_model::find_rating(&recipe_id, &user_id, executor).await? {
        // Update existing rating
        evento::update::<RatingAggregate>(&existing_rating_id)
            .data(&RatingUpdated { stars, review_text })?
            .metadata(&user_id)?
            .commit(executor)
            .await?;

        Ok(existing_rating_id)
    } else {
        // Create new rating
        let rating_id = create::<RatingAggregate>()
            .data(&RecipeRated {
                recipe_id: recipe_id.clone(),
                user_id: user_id.clone(),
                stars,
                review_text,
            })?
            .metadata(&user_id)?
            .commit(executor)
            .await?;

        Ok(rating_id)
    }
}
```

#### 3. MinIO Image Storage (`crates/recipe/src/image_storage.rs`)

```rust
use s3::bucket::Bucket;
use s3::creds::Credentials;
use s3::region::Region;
use uuid::Uuid;

const BUCKET_NAME: &str = "imkitchen-recipes";

pub async fn upload_image(image_data: Vec<u8>, content_type: &str) -> Result<String, ImageError> {
    let credentials = Credentials::new(
        Some(&std::env::var("MINIO_ACCESS_KEY")?),
        Some(&std::env::var("MINIO_SECRET_KEY")?),
        None,
        None,
        None,
    )?;

    let region = Region::Custom {
        region: "us-east-1".to_string(),
        endpoint: std::env::var("MINIO_ENDPOINT")?,
    };

    let bucket = Bucket::new(BUCKET_NAME, region, credentials)?;

    // Generate unique filename
    let filename = format!("{}.jpg", Uuid::new_v4());

    // Upload to MinIO
    bucket.put_object(&filename, &image_data).await?;

    // Return public URL
    let url = format!("{}/{}/{}",
        std::env::var("MINIO_ENDPOINT")?,
        BUCKET_NAME,
        filename
    );

    Ok(url)
}

pub async fn delete_image(image_url: &str) -> Result<(), ImageError> {
    // Extract filename from URL
    let filename = image_url.split('/').last()
        .ok_or(ImageError::InvalidUrl)?;

    let credentials = Credentials::new(
        Some(&std::env::var("MINIO_ACCESS_KEY")?),
        Some(&std::env::var("MINIO_SECRET_KEY")?),
        None,
        None,
        None,
    )?;

    let region = Region::Custom {
        region: "us-east-1".to_string(),
        endpoint: std::env::var("MINIO_ENDPOINT")?,
    };

    let bucket = Bucket::new(BUCKET_NAME, region, credentials)?;
    bucket.delete_object(filename).await?;

    Ok(())
}
```

#### 4. HTTP Route Handlers (`src/routes/recipes.rs`)

```rust
use axum::{
    Router,
    routing::{get, post, put, delete},
    extract::{Path, Multipart, State, Query},
    response::{Redirect, IntoResponse, Html},
    http::StatusCode,
};
use askama::Template;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/recipes", get(list_recipes).post(create_recipe_handler))
        .route("/recipes/new", get(new_recipe_form))
        .route("/recipes/:id", get(recipe_detail))
        .route("/recipes/:id/edit", get(edit_recipe_form))
        .route("/recipes/:id", put(update_recipe_handler))
        .route("/recipes/:id", delete(delete_recipe_handler))
        .route("/recipes/:id/favorite", post(favorite_recipe_handler))
        .route("/recipes/:id/share", post(share_recipe_handler))
}

// Templates
#[derive(Template)]
#[template(path = "pages/recipe-list.html")]
struct RecipeListTemplate {
    recipes: Vec<RecipeView>,
    collections: Vec<CollectionView>,
    current_filter: Option<String>,
}

#[derive(Template)]
#[template(path = "pages/recipe-detail.html")]
struct RecipeDetailTemplate {
    recipe: RecipeView,
    rating: Option<AggregateRating>,
    reviews: Vec<ReviewView>,
    is_owner: bool,
}

#[derive(Template)]
#[template(path = "pages/recipe-form.html")]
struct RecipeFormTemplate {
    recipe: Option<RecipeView>,
    errors: Vec<String>,
}

// Route handlers
async fn list_recipes(
    auth: Auth,
    Query(params): Query<ListParams>,
    State(state): State<AppState>,
) -> Result<Html<String>, AppError> {
    let recipes = recipe::read_model::list_user_recipes(
        &auth.user_id,
        params.collection_id.as_deref(),
        params.favorite_only.unwrap_or(false),
        &state.executor,
    ).await?;

    let collections = recipe::read_model::list_user_collections(&auth.user_id, &state.executor).await?;

    let template = RecipeListTemplate {
        recipes,
        collections,
        current_filter: params.collection_id,
    };

    Ok(Html(template.render()?))
}

async fn create_recipe_handler(
    auth: Auth,
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    let mut form_data = CreateRecipeForm::default();
    let mut image_data: Option<Vec<u8>> = None;

    // Parse multipart form
    while let Some(field) = multipart.next_field().await? {
        match field.name() {
            Some("title") => form_data.title = field.text().await?,
            Some("ingredients") => form_data.ingredients = field.text().await?,
            Some("instructions") => form_data.instructions = field.text().await?,
            Some("prep_time_min") => form_data.prep_time_min = field.text().await?.parse()?,
            Some("cook_time_min") => form_data.cook_time_min = field.text().await?.parse()?,
            Some("advance_prep_hours") => {
                if let Ok(hours) = field.text().await?.parse() {
                    form_data.advance_prep_hours = Some(hours);
                }
            },
            Some("serving_size") => form_data.serving_size = field.text().await?.parse()?,
            Some("image") => {
                if let Some(filename) = field.file_name() {
                    if !filename.is_empty() {
                        image_data = Some(field.bytes().await?.to_vec());
                    }
                }
            },
            _ => {}
        }
    }

    // Upload image to MinIO if present
    let image_url = if let Some(data) = image_data {
        Some(recipe::image_storage::upload_image(data, "image/jpeg").await?)
    } else {
        None
    };

    // Parse ingredients JSON
    let ingredients: Vec<Ingredient> = serde_json::from_str(&form_data.ingredients)?;
    let instructions: Vec<String> = serde_json::from_str(&form_data.instructions)?;

    // Create command
    let cmd = CreateRecipeCommand {
        title: form_data.title,
        ingredients,
        instructions,
        prep_time_min: form_data.prep_time_min,
        cook_time_min: form_data.cook_time_min,
        advance_prep_hours: form_data.advance_prep_hours,
        serving_size: form_data.serving_size,
        image_url,
    };

    // Invoke domain command
    let recipe_id = recipe::create_recipe(
        auth.user_id,
        auth.user_tier,
        cmd,
        &state.executor,
    ).await?;

    // Redirect to recipe detail
    Ok(Redirect::to(&format!("/recipes/{}", recipe_id)))
}

async fn favorite_recipe_handler(
    auth: Auth,
    Path(recipe_id): Path<String>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    // Toggle favorite status
    let recipe = evento::get::<RecipeAggregate>(&recipe_id, &state.executor).await?;
    let new_favorite_status = !recipe.is_favorite;

    recipe::favorite_recipe(recipe_id, auth.user_id, new_favorite_status, &state.executor).await?;

    // Return 200 OK (TwinSpark will update UI)
    Ok(StatusCode::OK)
}
```

#### 5. Community Discovery Routes (`src/routes/discover.rs`)

```rust
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/discover", get(community_feed))
        .route("/discover/:id", get(community_recipe_detail))
        .route("/discover/:id/add", post(add_to_library))
        .route("/discover/:id/rate", post(rate_community_recipe))
}

#[derive(Template)]
#[template(path = "pages/community-feed.html")]
struct CommunityFeedTemplate {
    recipes: Vec<CommunityRecipeView>,
    filters: DiscoveryFilters,
    pagination: Pagination,
}

async fn community_feed(
    Query(params): Query<DiscoveryParams>,
    State(state): State<AppState>,
) -> Result<Html<String>, AppError> {
    let recipes = recipe::read_model::list_shared_recipes(
        params.cuisine.as_deref(),
        params.min_rating,
        params.max_prep_time,
        params.dietary.as_deref(),
        params.page.unwrap_or(1),
        &state.executor,
    ).await?;

    let template = CommunityFeedTemplate {
        recipes,
        filters: DiscoveryFilters::from_params(&params),
        pagination: Pagination::new(params.page.unwrap_or(1), recipes.len()),
    };

    Ok(Html(template.render()?))
}

async fn community_recipe_detail(
    Path(recipe_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Html<String>, AppError> {
    let recipe = recipe::read_model::get_recipe(&recipe_id, &state.executor).await?;

    // Verify recipe is shared (public access)
    if !recipe.is_shared {
        return Err(AppError::NotFound);
    }

    let rating = ratings::read_model::get_aggregate_rating(&recipe_id, &state.executor).await?;
    let reviews = ratings::read_model::list_reviews(&recipe_id, &state.executor).await?;

    let template = RecipeDetailTemplate {
        recipe,
        rating: Some(rating),
        reviews,
        is_owner: false,
    };

    Ok(Html(template.render()?))
}

async fn add_to_library(
    auth: Auth,
    Path(recipe_id): Path<String>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    // Load source recipe
    let source_recipe = evento::get::<RecipeAggregate>(&recipe_id, &state.executor).await?;

    // Verify it's shared
    if !source_recipe.is_shared {
        return Err(AppError::Unauthorized);
    }

    // Create copy in user's library (new recipe with copied data)
    let cmd = CreateRecipeCommand {
        title: format!("{} (from community)", source_recipe.title),
        ingredients: source_recipe.ingredients,
        instructions: source_recipe.instructions,
        prep_time_min: source_recipe.prep_time_min,
        cook_time_min: source_recipe.cook_time_min,
        advance_prep_hours: source_recipe.advance_prep_hours,
        serving_size: source_recipe.serving_size,
        image_url: source_recipe.image_url,
    };

    let new_recipe_id = recipe::create_recipe(
        auth.user_id,
        auth.user_tier,
        cmd,
        &state.executor,
    ).await?;

    Ok(Redirect::to(&format!("/recipes/{}", new_recipe_id)))
}

async fn rate_community_recipe(
    auth: Auth,
    Path(recipe_id): Path<String>,
    Form(form): Form<RateRecipeForm>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    ratings::rate_recipe(
        recipe_id.clone(),
        auth.user_id,
        form.stars,
        form.review_text,
        &state.executor,
    ).await?;

    // Return updated rating HTML fragment (TwinSpark)
    let rating = ratings::read_model::get_aggregate_rating(&recipe_id, &state.executor).await?;
    let template = AggregateRatingPartial { rating };

    Ok(Html(template.render()?))
}
```

### Data Models and Contracts

#### Database Schema

**Read Model Tables (SQLite):**

```sql
-- Recipe read model table
CREATE TABLE recipes (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    title TEXT NOT NULL,
    ingredients TEXT NOT NULL,              -- JSON array of {name, quantity, unit}
    instructions TEXT NOT NULL,             -- JSON array of step strings
    prep_time_min INTEGER NOT NULL,
    cook_time_min INTEGER NOT NULL,
    advance_prep_hours INTEGER,             -- NULL if no advance prep
    serving_size INTEGER NOT NULL,
    image_url TEXT,
    complexity TEXT NOT NULL,               -- simple|moderate|complex
    cuisine TEXT,
    dietary_tags TEXT,                      -- JSON array of dietary attributes
    is_favorite BOOLEAN DEFAULT FALSE,
    is_shared BOOLEAN DEFAULT FALSE,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    deleted_at TEXT,                        -- Soft delete timestamp
    FOREIGN KEY (user_id) REFERENCES users(id)
);

CREATE INDEX idx_recipes_user_id ON recipes(user_id);
CREATE INDEX idx_recipes_favorite ON recipes(user_id, is_favorite) WHERE deleted_at IS NULL;
CREATE INDEX idx_recipes_shared ON recipes(is_shared) WHERE is_shared = TRUE AND deleted_at IS NULL;
CREATE INDEX idx_recipes_complexity ON recipes(complexity) WHERE deleted_at IS NULL;
CREATE INDEX idx_recipes_cuisine ON recipes(cuisine) WHERE deleted_at IS NULL;

-- Recipe collections
CREATE TABLE recipe_collections (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    name TEXT NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id),
    UNIQUE(user_id, name)
);

CREATE INDEX idx_collections_user_id ON recipe_collections(user_id);

-- Many-to-many recipe-collection assignments
CREATE TABLE recipe_collection_assignments (
    id TEXT PRIMARY KEY,
    recipe_id TEXT NOT NULL,
    collection_id TEXT NOT NULL,
    assigned_at TEXT NOT NULL,
    FOREIGN KEY (recipe_id) REFERENCES recipes(id) ON DELETE CASCADE,
    FOREIGN KEY (collection_id) REFERENCES recipe_collections(id) ON DELETE CASCADE,
    UNIQUE(recipe_id, collection_id)
);

CREATE INDEX idx_assignments_recipe ON recipe_collection_assignments(recipe_id);
CREATE INDEX idx_assignments_collection ON recipe_collection_assignments(collection_id);

-- Recipe tags (automatic and manual)
CREATE TABLE recipe_tags (
    id TEXT PRIMARY KEY,
    recipe_id TEXT NOT NULL,
    tag TEXT NOT NULL,
    tag_type TEXT NOT NULL,                -- automatic|manual
    created_at TEXT NOT NULL,
    FOREIGN KEY (recipe_id) REFERENCES recipes(id) ON DELETE CASCADE,
    UNIQUE(recipe_id, tag)
);

CREATE INDEX idx_tags_recipe ON recipe_tags(recipe_id);
CREATE INDEX idx_tags_tag ON recipe_tags(tag);

-- Recipe ratings and reviews
CREATE TABLE ratings (
    id TEXT PRIMARY KEY,
    recipe_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    stars INTEGER NOT NULL CHECK(stars >= 1 AND stars <= 5),
    review_text TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT,
    FOREIGN KEY (recipe_id) REFERENCES recipes(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id),
    UNIQUE(recipe_id, user_id)
);

CREATE INDEX idx_ratings_recipe ON ratings(recipe_id);
CREATE INDEX idx_ratings_user ON ratings(user_id);
CREATE INDEX idx_ratings_stars ON ratings(recipe_id, stars);

-- Aggregate rating view (materialized for performance)
CREATE TABLE recipe_aggregate_ratings (
    recipe_id TEXT PRIMARY KEY,
    average_rating REAL NOT NULL,
    total_ratings INTEGER NOT NULL,
    rating_distribution TEXT NOT NULL,     -- JSON: {1: count, 2: count, ...}
    updated_at TEXT NOT NULL,
    FOREIGN KEY (recipe_id) REFERENCES recipes(id) ON DELETE CASCADE
);
```

#### Event-to-ReadModel Projections

**Recipe Projections (evento subscriptions):**

```rust
// Read model projection subscription handlers
#[evento::handler(RecipeAggregate)]
async fn project_recipe_created<E: evento::Executor>(
    context: &evento::Context<'_, E>,
    event: EventDetails<RecipeCreated>,
) -> anyhow::Result<()> {
    let complexity = calculate_complexity(
        event.data.prep_time_min,
        event.data.cook_time_min,
        &event.data.instructions,
    );

    sqlx::query(
        r#"
        INSERT INTO recipes (
            id, user_id, title, ingredients, instructions,
            prep_time_min, cook_time_min, advance_prep_hours, serving_size,
            image_url, complexity, created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#
    )
    .bind(event.aggregator_id)
    .bind(event.data.user_id)
    .bind(event.data.title)
    .bind(serde_json::to_string(&event.data.ingredients)?)
    .bind(serde_json::to_string(&event.data.instructions)?)
    .bind(event.data.prep_time_min)
    .bind(event.data.cook_time_min)
    .bind(event.data.advance_prep_hours)
    .bind(event.data.serving_size)
    .bind(event.data.image_url)
    .bind(complexity.to_string())
    .bind(chrono::Utc::now().to_rfc3339())
    .bind(chrono::Utc::now().to_rfc3339())
    .execute(context.executor.pool())
    .await?;

    // Extract and insert automatic tags
    extract_and_insert_tags(&event.aggregator_id, &event.data, context.executor).await?;

    Ok(())
}

#[evento::handler(RecipeAggregate)]
async fn project_recipe_updated<E: evento::Executor>(
    context: &evento::Context<'_, E>,
    event: EventDetails<RecipeUpdated>,
) -> anyhow::Result<()> {
    // Build dynamic SQL for partial update
    let mut updates = Vec::new();
    let mut params = Vec::new();

    if let Some(ref title) = event.data.title {
        updates.push("title = ?");
        params.push(title.clone());
    }
    // ... (handle other optional fields)

    updates.push("updated_at = ?");
    params.push(chrono::Utc::now().to_rfc3339());

    let sql = format!("UPDATE recipes SET {} WHERE id = ?", updates.join(", "));
    params.push(event.aggregator_id.clone());

    sqlx::query(&sql)
        .bind_all(params)
        .execute(context.executor.pool())
        .await?;

    Ok(())
}

#[evento::handler(RecipeAggregate)]
async fn project_recipe_favorited<E: evento::Executor>(
    context: &evento::Context<'_, E>,
    event: EventDetails<RecipeFavorited>,
) -> anyhow::Result<()> {
    sqlx::query(
        "UPDATE recipes SET is_favorite = ?, updated_at = ? WHERE id = ?"
    )
    .bind(event.data.favorited)
    .bind(chrono::Utc::now().to_rfc3339())
    .bind(event.aggregator_id)
    .execute(context.executor.pool())
    .await?;

    Ok(())
}

#[evento::handler(RecipeAggregate)]
async fn project_recipe_shared<E: evento::Executor>(
    context: &evento::Context<'_, E>,
    event: EventDetails<RecipeShared>,
) -> anyhow::Result<()> {
    sqlx::query(
        "UPDATE recipes SET is_shared = ?, updated_at = ? WHERE id = ?"
    )
    .bind(event.data.shared)
    .bind(chrono::Utc::now().to_rfc3339())
    .bind(event.aggregator_id)
    .execute(context.executor.pool())
    .await?;

    Ok(())
}

#[evento::handler(RecipeAggregate)]
async fn project_recipe_deleted<E: evento::Executor>(
    context: &evento::Context<'_, E>,
    event: EventDetails<RecipeDeleted>,
) -> anyhow::Result<()> {
    sqlx::query(
        "UPDATE recipes SET deleted_at = ? WHERE id = ?"
    )
    .bind(event.data.deleted_at)
    .bind(event.aggregator_id)
    .execute(context.executor.pool())
    .await?;

    Ok(())
}
```

**Rating Projections:**

```rust
#[evento::handler(RatingAggregate)]
async fn project_recipe_rated<E: evento::Executor>(
    context: &evento::Context<'_, E>,
    event: EventDetails<RecipeRated>,
) -> anyhow::Result<()> {
    // Insert/update rating
    sqlx::query(
        r#"
        INSERT INTO ratings (id, recipe_id, user_id, stars, review_text, created_at)
        VALUES (?, ?, ?, ?, ?, ?)
        ON CONFLICT(recipe_id, user_id) DO UPDATE SET
            stars = excluded.stars,
            review_text = excluded.review_text,
            updated_at = excluded.created_at
        "#
    )
    .bind(event.aggregator_id)
    .bind(event.data.recipe_id)
    .bind(event.data.user_id)
    .bind(event.data.stars)
    .bind(event.data.review_text)
    .bind(chrono::Utc::now().to_rfc3339())
    .execute(context.executor.pool())
    .await?;

    // Recalculate aggregate rating
    recalculate_aggregate_rating(&event.data.recipe_id, context.executor).await?;

    Ok(())
}

async fn recalculate_aggregate_rating<E: Executor>(
    recipe_id: &str,
    executor: &E,
) -> Result<(), anyhow::Error> {
    let ratings: Vec<(u8,)> = sqlx::query_as(
        "SELECT stars FROM ratings WHERE recipe_id = ?"
    )
    .bind(recipe_id)
    .fetch_all(executor.pool())
    .await?;

    let total_ratings = ratings.len();
    let sum: u32 = ratings.iter().map(|(stars,)| *stars as u32).sum();
    let average = if total_ratings > 0 {
        sum as f64 / total_ratings as f64
    } else {
        0.0
    };

    // Calculate distribution
    let mut distribution = serde_json::json!({
        "1": 0, "2": 0, "3": 0, "4": 0, "5": 0
    });
    for (stars,) in ratings {
        distribution[stars.to_string()] = distribution[stars.to_string()].as_i64().unwrap() + 1;
    }

    // Upsert aggregate rating
    sqlx::query(
        r#"
        INSERT INTO recipe_aggregate_ratings (recipe_id, average_rating, total_ratings, rating_distribution, updated_at)
        VALUES (?, ?, ?, ?, ?)
        ON CONFLICT(recipe_id) DO UPDATE SET
            average_rating = excluded.average_rating,
            total_ratings = excluded.total_ratings,
            rating_distribution = excluded.rating_distribution,
            updated_at = excluded.updated_at
        "#
    )
    .bind(recipe_id)
    .bind(average)
    .bind(total_ratings as i64)
    .bind(serde_json::to_string(&distribution)?)
    .bind(chrono::Utc::now().to_rfc3339())
    .execute(executor.pool())
    .await?;

    Ok(())
}
```

### APIs and Interfaces

#### HTTP Endpoints

**Recipe CRUD Endpoints (Authenticated):**

| Method | Path | Description | Request Body | Response |
|--------|------|-------------|--------------|----------|
| GET | `/recipes` | List user's recipes with optional filters | Query: `?collection_id=`, `?favorite_only=true` | HTML: Recipe list page |
| GET | `/recipes/new` | New recipe creation form | - | HTML: Recipe form |
| POST | `/recipes` | Create new recipe | `multipart/form-data`: title, ingredients (JSON), instructions (JSON), prep_time_min, cook_time_min, advance_prep_hours, serving_size, image (file) | 302 Redirect to `/recipes/:id` |
| GET | `/recipes/:id` | Recipe detail page | - | HTML: Recipe detail with rating/reviews |
| GET | `/recipes/:id/edit` | Edit recipe form | - | HTML: Recipe form (pre-filled) |
| PUT | `/recipes/:id` | Update recipe | `multipart/form-data`: (same as POST) | 302 Redirect to `/recipes/:id` |
| DELETE | `/recipes/:id` | Delete recipe (soft delete) | - | 302 Redirect to `/recipes` |
| POST | `/recipes/:id/favorite` | Toggle favorite status | Form: `favorited=true/false` | 200 OK |
| POST | `/recipes/:id/share` | Toggle community sharing | Form: `shared=true/false` | 200 OK |

**Community Discovery Endpoints (Public Read, Authenticated Write):**

| Method | Path | Description | Auth Required | Response |
|--------|------|-------------|---------------|----------|
| GET | `/discover` | Community recipe feed with filters | No (public) | HTML: Community feed with SEO meta tags |
| GET | `/discover/:id` | Community recipe detail | No (public) | HTML: Recipe detail with Open Graph/Schema.org |
| POST | `/discover/:id/add` | Add community recipe to library | Yes | 302 Redirect to new recipe |
| POST | `/discover/:id/rate` | Rate/review recipe | Yes | 200 OK with updated rating HTML fragment |

**Collection Management Endpoints (Authenticated):**

| Method | Path | Description | Request Body | Response |
|--------|------|-------------|--------------|----------|
| GET | `/collections` | List user's collections | - | HTML: Collection list |
| POST | `/collections` | Create new collection | Form: `name` | 302 Redirect to `/recipes?collection_id=:id` |
| PUT | `/collections/:id` | Rename collection | Form: `name` | 200 OK |
| DELETE | `/collections/:id` | Delete collection | - | 302 Redirect to `/recipes` |
| POST | `/collections/:id/assign` | Assign recipe to collection | Form: `recipe_id` | 200 OK |
| DELETE | `/collections/:id/unassign/:recipe_id` | Remove recipe from collection | - | 200 OK |

#### Form Contracts

**CreateRecipeForm (multipart/form-data):**

```rust
#[derive(Deserialize, Validate)]
pub struct CreateRecipeForm {
    #[validate(length(min = 3, max = 200))]
    pub title: String,

    // JSON string: [{"name": "chicken", "quantity": 2, "unit": "lbs"}, ...]
    #[validate(length(min = 1))]
    pub ingredients: String,

    // JSON string: ["Step 1...", "Step 2...", ...]
    #[validate(length(min = 1))]
    pub instructions: String,

    #[validate(range(min = 1, max = 999))]
    pub prep_time_min: u32,

    #[validate(range(min = 1, max = 999))]
    pub cook_time_min: u32,

    pub advance_prep_hours: Option<u32>,

    #[validate(range(min = 1, max = 100))]
    pub serving_size: u32,

    // Multipart file upload
    pub image: Option<UploadedFile>,
}
```

**RateRecipeForm:**

```rust
#[derive(Deserialize, Validate)]
pub struct RateRecipeForm {
    #[validate(range(min = 1, max = 5))]
    pub stars: u8,

    #[validate(length(max = 1000))]
    pub review_text: Option<String>,
}
```

#### Query Interfaces

**Read Model Query Functions (`crates/recipe/src/read_model.rs`):**

```rust
pub async fn list_user_recipes<E: Executor>(
    user_id: &str,
    collection_id: Option<&str>,
    favorite_only: bool,
    executor: &E,
) -> Result<Vec<RecipeView>, RecipeError> {
    let mut query = String::from(
        "SELECT r.* FROM recipes r WHERE r.user_id = ? AND r.deleted_at IS NULL"
    );
    let mut params: Vec<String> = vec![user_id.to_string()];

    if favorite_only {
        query.push_str(" AND r.is_favorite = TRUE");
    }

    if let Some(coll_id) = collection_id {
        query.push_str(" AND r.id IN (SELECT recipe_id FROM recipe_collection_assignments WHERE collection_id = ?)");
        params.push(coll_id.to_string());
    }

    query.push_str(" ORDER BY r.updated_at DESC");

    let recipes = sqlx::query_as(&query)
        .bind_all(params)
        .fetch_all(executor.pool())
        .await?;

    Ok(recipes)
}

pub async fn list_shared_recipes<E: Executor>(
    cuisine: Option<&str>,
    min_rating: Option<f64>,
    max_prep_time: Option<u32>,
    dietary: Option<&str>,
    page: u32,
    executor: &E,
) -> Result<Vec<CommunityRecipeView>, RecipeError> {
    let mut query = String::from(
        r#"
        SELECT r.*, ar.average_rating, ar.total_ratings, u.email as creator_email
        FROM recipes r
        LEFT JOIN recipe_aggregate_ratings ar ON r.id = ar.recipe_id
        JOIN users u ON r.user_id = u.id
        WHERE r.is_shared = TRUE AND r.deleted_at IS NULL
        "#
    );
    let mut params = Vec::new();

    if let Some(c) = cuisine {
        query.push_str(" AND r.cuisine = ?");
        params.push(c.to_string());
    }

    if let Some(rating) = min_rating {
        query.push_str(" AND ar.average_rating >= ?");
        params.push(rating.to_string());
    }

    if let Some(max_prep) = max_prep_time {
        query.push_str(" AND r.prep_time_min <= ?");
        params.push(max_prep.to_string());
    }

    if let Some(diet) = dietary {
        query.push_str(" AND r.dietary_tags LIKE ?");
        params.push(format!("%{}%", diet));
    }

    query.push_str(" ORDER BY ar.average_rating DESC, ar.total_ratings DESC LIMIT 20 OFFSET ?");
    params.push(((page - 1) * 20).to_string());

    let recipes = sqlx::query_as(&query)
        .bind_all(params)
        .fetch_all(executor.pool())
        .await?;

    Ok(recipes)
}

pub async fn count_user_recipes<E: Executor>(
    user_id: &str,
    executor: &E,
) -> Result<u32, RecipeError> {
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM recipes WHERE user_id = ? AND deleted_at IS NULL"
    )
    .bind(user_id)
    .fetch_one(executor.pool())
    .await?;

    Ok(count.0 as u32)
}

pub async fn get_recipe<E: Executor>(
    recipe_id: &str,
    executor: &E,
) -> Result<RecipeView, RecipeError> {
    let recipe = sqlx::query_as(
        "SELECT * FROM recipes WHERE id = ? AND deleted_at IS NULL"
    )
    .bind(recipe_id)
    .fetch_optional(executor.pool())
    .await?
    .ok_or(RecipeError::NotFound)?;

    Ok(recipe)
}
```

### Workflows and Sequencing

#### Recipe Creation Workflow

```
1. User navigates to GET /recipes/new
   ↓
2. Server renders recipe form (Askama template)
   ↓
3. User fills form: title, ingredients, instructions, timing, uploads image
   ↓
4. User submits POST /recipes (multipart/form-data)
   ↓
5. Route Handler:
   - Parse multipart form data
   - Validate form inputs (validator crate)
   - Upload image to MinIO (if present)
   - Parse ingredients/instructions JSON
   ↓
6. Invoke Domain Command:
   - Check freemium limit (user_tier, recipe count)
   - If free tier and count >= 10: return RecipeLimitReached error
   ↓
7. Create Recipe Aggregate:
   - evento::create::<RecipeAggregate>()
   - Apply RecipeCreated event
   - Commit to event stream
   ↓
8. evento Subscription Handler:
   - Listen for RecipeCreated event
   - Insert into `recipes` table (read model)
   - Extract and insert automatic tags
   ↓
9. Response:
   - 302 Redirect to /recipes/:id
   ↓
10. User sees recipe detail page with success message
```

**Error Handling:**
- Validation error → Re-render form with inline error messages (422 status)
- Recipe limit error → Display flash message "Upgrade to Premium for unlimited recipes" (422 status)
- MinIO upload error → Display error message, allow retry (500 status)
- Unexpected error → Generic error page (500 status)

#### Community Recipe Discovery Workflow

```
1. Guest/User navigates to GET /discover (public, no auth required)
   ↓
2. Server queries `recipes` table:
   - Filter: is_shared = TRUE, deleted_at IS NULL
   - Join: recipe_aggregate_ratings for average_rating
   - Apply filters from query params (cuisine, rating, prep_time, dietary)
   - Paginate: 20 recipes per page
   ↓
3. Server renders community feed:
   - Askama template with recipe cards
   - SEO meta tags (title, description, Open Graph)
   - Filter controls (dropdowns for cuisine, rating, etc.)
   ↓
4. User applies filters:
   - Client updates query params (e.g., ?cuisine=italian&min_rating=4)
   - Full page reload with filtered results (or TwinSpark AJAX)
   ↓
5. User clicks recipe card → GET /discover/:id
   ↓
6. Server renders recipe detail:
   - Query `recipes` table for recipe
   - Query `recipe_aggregate_ratings` for aggregate score
   - Query `ratings` table for reviews
   - Render with Schema.org Recipe JSON-LD for SEO
   ↓
7. Authenticated user clicks "Add to My Recipes":
   - POST /discover/:id/add (requires JWT)
   - Domain command: CreateRecipe (copy of source recipe)
   - Check freemium limit
   - If success: Redirect to /recipes/:new_id
   ↓
8. Authenticated user rates recipe:
   - POST /discover/:id/rate with stars and review_text
   - Domain command: RateRecipe (create or update)
   - evento subscription: Update `ratings` table, recalculate aggregate
   - TwinSpark response: Updated rating HTML fragment
```

**SEO Optimization:**
- `/discover` and `/discover/:id` routes are public (no auth wall)
- Server-rendered HTML with full recipe content for crawlers
- Open Graph meta tags for social sharing previews
- Schema.org Recipe markup for rich snippets in Google search
- Sitemap includes all shared recipes (future enhancement)

#### Recipe Rating and Review Workflow

```
1. User views recipe detail page (GET /recipes/:id or /discover/:id)
   ↓
2. If authenticated, form displays "Rate this recipe" section
   ↓
3. User selects stars (1-5) and optionally writes review text
   ↓
4. User submits POST /discover/:id/rate (or /recipes/:id/rate)
   ↓
5. Route Handler:
   - Validate: stars in range 1-5
   - Validate: review_text max 1000 chars
   ↓
6. Domain Command: RateRecipe
   - Check if user already rated this recipe (query `ratings` table)
   - If exists: Update existing rating (RatingUpdated event)
   - If new: Create new rating (RecipeRated event)
   ↓
7. evento Subscription:
   - Insert/update `ratings` table
   - Recalculate aggregate rating for recipe
   - Update `recipe_aggregate_ratings` table
   ↓
8. Response (TwinSpark):
   - Return HTML fragment with updated aggregate rating
   - Client swaps fragment into DOM (no full page reload)
   ↓
9. User sees updated rating and their review appears in list
```

**Review Editing/Deletion:**
- User can edit own review: PUT /ratings/:id
- User can delete own review: DELETE /ratings/:id
- Both trigger RatingUpdated/RatingDeleted events
- Aggregate rating recalculated on each change

## Non-Functional Requirements

### Performance

**Response Time Targets:**
- Recipe list page: < 500ms (95th percentile)
- Recipe detail page: < 500ms (includes rating aggregation)
- Community discovery page: < 800ms (includes joins, filters, pagination)
- Recipe creation: < 2s (includes MinIO image upload)
- Image upload to MinIO: < 3s for 5MB JPEG

**Optimization Strategies:**
1. **Database Indexing**:
   - Index on `user_id`, `is_favorite`, `is_shared`, `complexity`, `cuisine`
   - Composite index on `(recipe_id, user_id)` for ratings uniqueness
   - Index on `recipe_id` in ratings table for aggregate calculation

2. **Read Model Materialization**:
   - `recipe_aggregate_ratings` table pre-calculates average ratings
   - Avoid real-time aggregation on every recipe detail page load
   - Update aggregate asynchronously via evento subscription

3. **Query Optimization**:
   - Use SQLite prepared statements (cached by SQLx)
   - Limit joins (denormalize where necessary)
   - Paginate community discovery (20 recipes/page)

4. **MinIO Performance**:
   - Use pre-signed URLs for direct browser-to-MinIO uploads (future)
   - Serve images via CDN (future enhancement)
   - Compress images server-side before upload (JPEG quality 85%)

**Load Testing:**
- Target: 100 concurrent users creating/viewing recipes
- Tool: k6 or Locust (future)
- Baseline: Single SQLite instance on SSD storage

### Security

**Authentication and Authorization:**
1. **Recipe Ownership Verification**:
   - All edit/delete operations verify `recipe.user_id == auth.user_id`
   - Unauthorized access returns 401 Unauthorized
   - Aggregate loads from event stream include ownership check

2. **Freemium Limit Enforcement**:
   - Recipe count checked in domain command (not UI)
   - Free tier: max 10 recipes (enforced in `CreateRecipeCommand`)
   - Bypass not possible via direct API access (domain validates)

3. **Input Validation**:
   - Server-side validation with `validator` crate (all form inputs)
   - Ingredients/instructions JSON parsed and validated
   - Image uploads: validate MIME type (image/jpeg, image/png only)
   - Max image size: 5MB (enforced in multipart parser)

4. **SQL Injection Prevention**:
   - All queries use SQLx parameterized statements
   - No string concatenation in SQL
   - User input never directly interpolated

5. **XSS Prevention**:
   - Askama templates auto-escape HTML by default
   - Recipe titles, instructions, reviews sanitized on render
   - No `|safe` filter used on user-generated content

6. **Image Upload Security**:
   - Validate file extension and MIME type
   - Generate UUID-based filenames (prevent path traversal)
   - Store in isolated MinIO bucket (no execution permissions)
   - Scan uploads for malware (future enhancement)

**Privacy Controls:**
1. **Private Recipes**:
   - Default `is_shared = false` on creation
   - Private recipes excluded from `/discover` queries
   - Direct URL access to private recipe returns 404 if not owner

2. **Community Sharing**:
   - Explicit opt-in via "Share to Community" toggle
   - User can unshare anytime (RecipeShared event with `shared=false`)
   - Attribution preserved (creator email displayed on community page)

3. **GDPR Compliance**:
   - User deletion anonymizes recipes (replace user_id with anonymized value)
   - Events retained with anonymized metadata (event sourcing constraint)
   - Data export includes all user recipes as JSON (future enhancement)

### Reliability/Availability

**Data Durability:**
1. **Event Sourcing Guarantees**:
   - All recipe changes recorded as immutable events
   - SQLite WAL mode ensures atomic commits
   - evento guarantees event ordering per aggregate

2. **Image Storage Reliability**:
   - MinIO configured with erasure coding (future)
   - Backup bucket to separate storage (future)
   - Image deletion on recipe delete (cleanup job)

3. **Soft Delete Pattern**:
   - Recipes marked `deleted_at` (not hard deleted)
   - Events preserved for audit trail
   - Recovery possible by clearing `deleted_at`

**Error Handling:**
1. **Graceful Degradation**:
   - If MinIO unavailable: Recipe creation succeeds without image
   - If rating aggregation fails: Display "Rating unavailable"
   - If community feed query fails: Display cached results (future)

2. **Retry Logic**:
   - MinIO upload retries 3 times with exponential backoff
   - evento event commit retries on transient failures
   - Image deletion queued for retry if MinIO down

3. **Transaction Boundaries**:
   - Recipe creation: Single transaction (event write)
   - Rating: Atomic insert/update with aggregate recalculation
   - Image upload separate from event commit (eventual consistency)

**Availability Target:**
- 99.5% uptime (4 hours/month downtime allowance per NFRs)
- Health check endpoint: GET /health (returns 200 if recipe domain ready)
- Readiness check: Verify SQLite connection, MinIO bucket exists

### Observability

**Distributed Tracing:**
1. **OpenTelemetry Instrumentation**:
   - Trace all HTTP requests to recipe endpoints
   - Span for each domain command (create, update, delete, rate)
   - Span for MinIO operations (upload, delete)
   - Span for evento event commits

2. **Trace Attributes**:
   - `user.id`: User performing action
   - `recipe.id`: Recipe being operated on
   - `command.type`: CreateRecipe, UpdateRecipe, etc.
   - `minio.operation`: upload, delete
   - `error.type`: RecipeLimitReached, Unauthorized, etc.

**Metrics:**
1. **Business Metrics**:
   - Total recipes created (counter)
   - Recipes shared to community (counter)
   - Community recipe views (counter)
   - Ratings submitted (counter)
   - Freemium limit hits (counter)

2. **Technical Metrics**:
   - Recipe creation latency (histogram)
   - Image upload duration (histogram)
   - Database query duration (histogram)
   - evento event commit rate (gauge)

**Structured Logging:**
```rust
#[tracing::instrument(skip(executor), fields(user_id = %user_id, recipe_id))]
pub async fn create_recipe<E: Executor>(
    user_id: String,
    user_tier: UserTier,
    cmd: CreateRecipeCommand,
    executor: &E,
) -> Result<String, RecipeError> {
    tracing::info!("Creating recipe for user {}", user_id);

    let recipe_count = read_model::count_user_recipes(&user_id, executor).await?;
    if user_tier == UserTier::Free && recipe_count >= 10 {
        tracing::warn!("Freemium limit reached for user {}", user_id);
        return Err(RecipeError::RecipeLimitReached);
    }

    let recipe_id = create::<RecipeAggregate>()
        // ... event creation
        .commit(executor)
        .await?;

    tracing::info!(recipe_id = %recipe_id, "Recipe created successfully");
    Ok(recipe_id)
}
```

**Alerting:**
- High recipe creation error rate (> 5% in 5 min window)
- MinIO upload failures (> 10 in 1 min)
- Freemium limit hit rate spike (potential abuse)
- Database connection pool exhaustion

## Dependencies and Integrations

### Upstream Dependencies

1. **User Domain Crate (`crates/user/`)**:
   - **Purpose**: User ID validation, freemium tier enforcement
   - **Integration**: Query user tier before recipe creation
   - **Contract**: `get_user_tier(user_id: &str) -> Result<UserTier, UserError>`
   - **Failure Handling**: Return 401 Unauthorized if user not found

2. **Authentication Middleware**:
   - **Purpose**: JWT validation for protected routes
   - **Integration**: Axum middleware extracts `Auth` from request extensions
   - **Contract**: `Auth { user_id: String, user_tier: UserTier }`
   - **Failure Handling**: 401 Unauthorized redirects to /login

3. **MinIO Service**:
   - **Purpose**: S3-compatible object storage for recipe images
   - **Integration**: rust-s3 crate with MINIO_ENDPOINT, credentials from env
   - **Operations**: upload_image(), delete_image()
   - **Failure Handling**: Retry 3x with exponential backoff, fallback to no image

### Downstream Consumers

1. **Meal Planning Domain (`crates/meal_planning/`)**:
   - **Consumes**: RecipeFavorited events, recipe complexity metadata
   - **Purpose**: Include favorite recipes in meal plan generation
   - **Integration**: evento subscription listens for RecipeFavorited events
   - **Query**: `list_user_favorites(user_id: &str) -> Vec<RecipeView>`

2. **Shopping List Domain (`crates/shopping/`)**:
   - **Consumes**: Recipe ingredients from read model
   - **Purpose**: Aggregate ingredients for weekly shopping lists
   - **Integration**: Query recipe read model for meal plan recipes
   - **Query**: `get_recipe_ingredients(recipe_id: &str) -> Vec<Ingredient>`

3. **SEO/Search Engines**:
   - **Consumes**: Public `/discover` routes, Schema.org markup
   - **Purpose**: Index community recipes for organic traffic
   - **Integration**: robots.txt allows /discover/*, sitemap.xml (future)

### External Service Integrations

**MinIO (rust-s3 crate):**

```rust
// Configuration (from env vars)
MINIO_ENDPOINT=http://minio:9000
MINIO_ACCESS_KEY=minioadmin
MINIO_SECRET_KEY=minioadmin
MINIO_BUCKET=imkitchen-recipes

// Initialization (on startup)
async fn init_minio() -> Result<(), Error> {
    let bucket = Bucket::new(
        MINIO_BUCKET,
        Region::Custom {
            region: "us-east-1".to_string(),
            endpoint: env::var("MINIO_ENDPOINT")?,
        },
        Credentials::new(
            Some(&env::var("MINIO_ACCESS_KEY")?),
            Some(&env::var("MINIO_SECRET_KEY")?),
            None, None, None,
        )?,
    )?;

    // Create bucket if not exists
    if !bucket.exists().await? {
        bucket.create().await?;
    }

    Ok(())
}

// Usage in recipe creation
let image_url = if let Some(image_data) = image {
    Some(image_storage::upload_image(image_data, "image/jpeg").await?)
} else {
    None
};
```

**Error Handling:**
- MinIO unavailable: Recipe creation succeeds with `image_url = None`
- Upload timeout: Retry 3x, then fail gracefully with error message
- Delete failure: Queue for retry (background job)

### Event Bus (evento)

**Published Events:**
1. `RecipeCreated` → Consumed by: meal_planning (if favorited), analytics (future)
2. `RecipeFavorited` → Consumed by: meal_planning (update favorite list)
3. `RecipeShared` → Consumed by: analytics (track community growth)
4. `RecipeRated` → Consumed by: analytics, recommendation engine (future)

**Subscription Registration (in main.rs):**

```rust
// Register recipe read model projections
evento::subscribe("recipe-projections")
    .aggregator::<RecipeAggregate>()
    .handler(project_recipe_created)
    .handler(project_recipe_updated)
    .handler(project_recipe_favorited)
    .handler(project_recipe_shared)
    .handler(project_recipe_deleted)
    .run(&executor)
    .await?;

// Register rating projections
evento::subscribe("rating-projections")
    .aggregator::<RatingAggregate>()
    .handler(project_recipe_rated)
    .handler(project_rating_updated)
    .run(&executor)
    .await?;

// Cross-domain subscription: meal planning consumes RecipeFavorited
evento::subscribe("meal-planning-favorites")
    .aggregator::<RecipeAggregate>()
    .handler(meal_planning::on_recipe_favorited)
    .run(&executor)
    .await?;
```

## Acceptance Criteria (Authoritative)

**Epic 2 is complete when ALL of the following criteria are met:**

### Recipe CRUD (Stories 1-4)

**AC-2.1: Recipe Creation**
- [ ] User can create recipe with title (3-200 chars), ingredients (min 1), instructions (min 1 step)
- [ ] User can specify prep time (1-999 min), cook time (1-999 min), serving size (1-100)
- [ ] User can optionally specify advance prep hours (e.g., 24 for marinade)
- [ ] User can upload recipe image (JPEG/PNG, max 5MB) to MinIO
- [ ] Free tier users blocked at 10 recipes with "Upgrade to Premium" message
- [ ] Premium users can create unlimited recipes
- [ ] Recipe complexity auto-calculated (simple/moderate/complex) based on time and steps
- [ ] RecipeCreated event written to evento stream
- [ ] Recipe inserted into `recipes` read model table via subscription
- [ ] User redirected to recipe detail page on success

**AC-2.2: Recipe Editing**
- [ ] User can edit own recipes (title, ingredients, instructions, timing, image)
- [ ] Editing other users' recipes returns 401 Unauthorized
- [ ] Deleted recipes cannot be edited (returns 404 or error message)
- [ ] RecipeUpdated event written to evento stream
- [ ] Read model updated via subscription
- [ ] User sees success message on recipe detail page

**AC-2.3: Recipe Deletion**
- [ ] User can delete own recipes (soft delete with `deleted_at` timestamp)
- [ ] Deleted recipes excluded from all user queries
- [ ] RecipeDeleted event written to evento stream
- [ ] Recipe image deleted from MinIO (if exists)
- [ ] Deleted recipes not visible in meal planning or community discovery
- [ ] User redirected to recipe list with success message

**AC-2.4: Recipe Images**
- [ ] Image upload validates MIME type (image/jpeg, image/png only)
- [ ] Image upload validates size (max 5MB)
- [ ] Image stored in MinIO `imkitchen-recipes` bucket with UUID filename
- [ ] Image URL stored in `recipes.image_url` column
- [ ] Image displayed on recipe detail page
- [ ] Placeholder image shown if no image uploaded
- [ ] Image deleted from MinIO when recipe deleted

### Organization and Favorites (Stories 5-6)

**AC-2.5: Recipe Collections**
- [ ] User can create named collections (e.g., "Weeknight Dinners", "Desserts")
- [ ] User can rename and delete collections
- [ ] User can assign recipes to multiple collections (many-to-many)
- [ ] User can remove recipes from collections
- [ ] Recipe list filtered by collection via `?collection_id=` query param
- [ ] Collections stored in `recipe_collections` table
- [ ] Assignments stored in `recipe_collection_assignments` table

**AC-2.6: Favorite Recipes**
- [ ] User can mark recipe as favorite (toggle button)
- [ ] Favorite status persisted in `recipes.is_favorite` column
- [ ] RecipeFavorited event written to evento stream
- [ ] Favorite recipes consumed by meal planning algorithm
- [ ] Recipe list filtered by favorites via `?favorite_only=true` query param
- [ ] Favorite count displayed on recipe list

### Privacy and Sharing (Stories 7-8)

**AC-2.7: Privacy Controls**
- [ ] New recipes default to private (`is_shared = false`)
- [ ] User can toggle "Share to Community" (RecipeShared event)
- [ ] Private recipes excluded from `/discover` routes
- [ ] Direct URL to private recipe returns 404 for non-owners
- [ ] Shared recipes visible in community discovery feed
- [ ] User can unshare recipe anytime (reverts to private)

**AC-2.8: Recipe Attribution**
- [ ] Shared recipes display creator's email/username
- [ ] Creator cannot be modified by other users
- [ ] Attribution preserved when recipe added to user's library (copy, not reference)

### Community Discovery (Stories 9-12)

**AC-2.9: Community Recipe Feed**
- [ ] Public `/discover` route accessible without authentication
- [ ] Feed displays shared recipes (`is_shared = true`, `deleted_at IS NULL`)
- [ ] Pagination: 20 recipes per page
- [ ] Sorting: Default by average rating DESC, then total ratings DESC
- [ ] Filter by cuisine (dropdown: Italian, Mexican, Asian, etc.)
- [ ] Filter by minimum rating (1-5 stars)
- [ ] Filter by max prep time (15 min, 30 min, 60 min, 60+ min)
- [ ] Filter by dietary preference (vegetarian, vegan, gluten-free)
- [ ] Guest users can browse (no auth wall)
- [ ] SEO meta tags included (title, description, Open Graph)

**AC-2.10: Community Recipe Detail**
- [ ] Public `/discover/:id` route accessible without authentication
- [ ] Recipe detail displays full recipe (title, ingredients, instructions, timing)
- [ ] Aggregate rating displayed (average stars, total ratings)
- [ ] Reviews listed (user, stars, review text, date)
- [ ] Schema.org Recipe JSON-LD markup for SEO (Google rich snippets)
- [ ] Open Graph tags for social sharing (Facebook, Twitter preview cards)
- [ ] "Add to My Recipes" button visible for authenticated users
- [ ] "Rate Recipe" form visible for authenticated users

**AC-2.11: Add to Library**
- [ ] Authenticated user can click "Add to My Recipes" on community recipe
- [ ] Creates copy in user's library (new RecipeAggregate, not reference)
- [ ] Copy includes all recipe data (ingredients, instructions, image URL)
- [ ] Freemium limit enforced (free tier: max 10 recipes)
- [ ] User redirected to new recipe detail page
- [ ] Original community recipe unchanged

**AC-2.12: Search (Future Enhancement - Document for MVP Exclusion)**
- [ ] NOT IN MVP: Full-text search by recipe title/ingredients
- [ ] NOT IN MVP: Natural language search ("quick chicken dinner")
- [ ] NOT IN MVP: Search by tags or automatic categorization

### Ratings and Reviews (Stories 13-15)

**AC-2.13: Rate Recipe**
- [ ] Authenticated user can rate recipe (1-5 stars, required)
- [ ] User can write optional review text (max 1000 chars)
- [ ] One rating per user per recipe (enforced by unique constraint)
- [ ] Updating existing rating replaces previous (RatingUpdated event)
- [ ] RecipeRated event written to evento stream
- [ ] Rating inserted/updated in `ratings` table via subscription
- [ ] Aggregate rating recalculated on each rating change

**AC-2.14: Aggregate Rating Display**
- [ ] Recipe detail shows average rating (0.0-5.0, 1 decimal)
- [ ] Recipe detail shows total rating count (e.g., "4.8 stars from 47 reviews")
- [ ] Rating distribution displayed (bar chart: 5★: 30, 4★: 12, 3★: 5, etc.)
- [ ] Aggregate rating stored in `recipe_aggregate_ratings` table
- [ ] Aggregate updated asynchronously via evento subscription (eventual consistency)

**AC-2.15: Review Management**
- [ ] User can edit own review (stars and text)
- [ ] User can delete own review (RatingDeleted event)
- [ ] Reviews sorted by date DESC (most recent first)
- [ ] Review shows user email/username, date, stars, text
- [ ] No review moderation in MVP (flagging system future)

### Non-Functional (Stories NFR-1 to NFR-3)

**AC-2.NFR-1: Performance**
- [ ] Recipe list page loads in < 500ms (95th percentile)
- [ ] Recipe detail page loads in < 500ms (includes rating aggregation)
- [ ] Community feed loads in < 800ms (includes filters, joins)
- [ ] Image upload completes in < 3s for 5MB JPEG
- [ ] Database queries use indexes (user_id, is_shared, recipe_id in ratings)

**AC-2.NFR-2: Security**
- [ ] All recipe mutations verify ownership (recipe.user_id == auth.user_id)
- [ ] Freemium limit enforced in domain command (not UI only)
- [ ] All form inputs validated server-side (validator crate)
- [ ] SQL injection prevented (SQLx parameterized queries)
- [ ] XSS prevented (Askama auto-escaping)
- [ ] Image uploads validated (MIME type, size, extension)
- [ ] Private recipes return 404 for non-owners (not 403, prevents enumeration)

**AC-2.NFR-3: Observability**
- [ ] OpenTelemetry spans for all recipe commands (create, update, delete, rate)
- [ ] Trace attributes: user.id, recipe.id, command.type
- [ ] Structured logs for recipe creation, freemium limit hits, errors
- [ ] Metrics: recipe_created_total, recipe_shared_total, rating_submitted_total
- [ ] Health check endpoint returns 200 if recipe domain ready

## Traceability Mapping

### PRD Requirements to Technical Implementation

**FR-1: Recipe Creation and Storage → AC-2.1**
- PRD Requirement: "Users can create recipes with title, ingredients, step-by-step instructions, preparation time, cooking time, advance preparation requirements, and serving size"
- Implementation: `CreateRecipeCommand` with validation, `RecipeAggregate` with `RecipeCreated` event, MinIO image storage
- Files: `crates/recipe/src/commands.rs`, `src/routes/recipes.rs`

**FR-2: Recipe Organization → AC-2.5**
- PRD Requirement: "System organizes recipes into user-defined collections and automatically tags recipes"
- Implementation: `recipe_collections` and `recipe_collection_assignments` tables, automatic tagging on creation
- Files: `crates/recipe/src/collections.rs`, `crates/recipe/src/tagging.rs`

**FR-3: Recipe Sharing and Privacy Controls → AC-2.7, AC-2.8**
- PRD Requirement: "Users can mark recipes as private or shared. Shared recipes appear in community discovery with attribution"
- Implementation: `is_shared` boolean, `RecipeShared` event, `/discover` routes filter by `is_shared=true`, creator attribution
- Files: `crates/recipe/src/commands.rs` (share_recipe), `src/routes/discover.rs`

**FR-11: Recipe Rating and Reviews → AC-2.13, AC-2.14, AC-2.15**
- PRD Requirement: "Users can rate recipes (1-5 stars) and write text reviews. Ratings aggregate to show community quality scores"
- Implementation: `RatingAggregate`, `RecipeRated` event, `recipe_aggregate_ratings` table with average/total
- Files: `crates/ratings/src/aggregate.rs`, `crates/ratings/src/read_model.rs`

**FR-12: Community Recipe Discovery → AC-2.9, AC-2.10, AC-2.11**
- PRD Requirement: "Users browse shared recipes from other users, filtered by rating, cuisine, preparation time, and dietary preferences"
- Implementation: `/discover` public routes with filters, SEO optimization (Open Graph, Schema.org)
- Files: `src/routes/discover.rs`, `templates/pages/community-feed.html`

**FR-14: Favorite Recipe Management → AC-2.6**
- PRD Requirement: "Users mark recipes as favorites, which feeds the meal planning algorithm"
- Implementation: `RecipeFavorited` event, `is_favorite` boolean, evento subscription for meal_planning integration
- Files: `crates/recipe/src/commands.rs` (favorite_recipe), cross-domain event subscription

**FR-15: Freemium Access Controls → AC-2.1 (Recipe Limit)**
- PRD Requirement: "Free tier users limited to 10 recipes maximum. Premium users access unlimited recipes"
- Implementation: Recipe count check in `create_recipe` command, return `RecipeLimitReached` error if free tier and count >= 10
- Files: `crates/recipe/src/commands.rs` (create_recipe with tier check)

**NFR-1: Performance → AC-2.NFR-1**
- PRD Requirement: "Page load times <3 seconds on 3G, HTML response <500ms for 95th percentile"
- Implementation: Database indexes, read model materialization, SQLite optimization (WAL mode), MinIO CDN (future)
- Files: `migrations/002_create_recipes_table.sql` (indexes), `crates/recipe/src/read_model.rs` (optimized queries)

**NFR-4: Security → AC-2.NFR-2**
- PRD Requirement: "All user data encrypted at rest and in transit, JWT cookie-based auth, OWASP Top 10 compliance"
- Implementation: Ownership verification, input validation, SQL injection prevention, XSS auto-escaping, image upload validation
- Files: All route handlers (ownership checks), `validator` usage, SQLx parameterized queries, Askama templates

**NFR-8: Observability → AC-2.NFR-3**
- PRD Requirement: "OpenTelemetry instrumentation for distributed tracing, structured logging, real-time metrics"
- Implementation: Tracing spans for all commands, structured logs, metrics counters/histograms
- Files: `crates/recipe/src/commands.rs` (tracing::instrument), route handlers (tracing::info)

### User Stories (from epics.md) to Acceptance Criteria

| Story ID | Title | Acceptance Criteria |
|----------|-------|---------------------|
| 2.1 | Create Recipe with Details | AC-2.1 (recipe creation with all fields, image upload, freemium limit) |
| 2.2 | Edit Existing Recipe | AC-2.2 (edit with ownership check, RecipeUpdated event) |
| 2.3 | Delete Recipe | AC-2.3 (soft delete, image cleanup, RecipeDeleted event) |
| 2.4 | Upload Recipe Image | AC-2.4 (MinIO integration, validation, placeholder) |
| 2.5 | Organize into Collections | AC-2.5 (create/rename/delete collections, assign recipes) |
| 2.6 | Mark Recipes as Favorites | AC-2.6 (favorite toggle, RecipeFavorited event, meal planning integration) |
| 2.7 | Privacy Toggle | AC-2.7 (private/shared, RecipeShared event, discovery filtering) |
| 2.8 | Recipe Attribution | AC-2.8 (creator display, attribution preservation) |
| 2.9 | Community Feed | AC-2.9 (public route, filters, pagination, SEO) |
| 2.10 | Community Recipe Detail | AC-2.10 (public detail page, Schema.org, Open Graph) |
| 2.11 | Add to Library | AC-2.11 (copy recipe, freemium limit, redirect) |
| 2.12 | Search Recipes | AC-2.12 (OUT OF SCOPE for MVP - documented) |
| 2.13 | Rate Recipe | AC-2.13 (1-5 stars, review text, RecipeRated event) |
| 2.14 | View Aggregate Ratings | AC-2.14 (average, total, distribution display) |
| 2.15 | Manage Reviews | AC-2.15 (edit/delete own reviews, RatingUpdated/Deleted events) |

### Architecture Components to Implementation

| Architecture Component | Implementation | Files |
|------------------------|----------------|-------|
| RecipeAggregate (evento) | Event-sourced aggregate with commands/events | `crates/recipe/src/aggregate.rs` |
| RatingAggregate (evento) | Event-sourced rating management | `crates/ratings/src/aggregate.rs` |
| Recipe Read Model | SQLite `recipes` table with projections | `migrations/002_create_recipes_table.sql`, `crates/recipe/src/read_model.rs` |
| Rating Read Model | SQLite `ratings` and `recipe_aggregate_ratings` tables | `migrations/005_create_ratings_table.sql`, `crates/ratings/src/read_model.rs` |
| MinIO Integration | Image upload/delete via rust-s3 | `crates/recipe/src/image_storage.rs` |
| HTTP Routes (CRUD) | Axum routes for recipe management | `src/routes/recipes.rs` |
| HTTP Routes (Community) | Axum routes for discovery/rating | `src/routes/discover.rs` |
| Askama Templates | Server-rendered HTML pages/partials | `templates/pages/recipe-*.html`, `templates/partials/` |
| evento Subscriptions | Event-to-ReadModel projection handlers | `crates/recipe/src/read_model.rs` (projection functions) |

## Risks, Assumptions, Open Questions

### Risks

**R-2.1: Image Storage Scalability**
- **Risk**: MinIO single-instance may not scale to 10K users with high image upload volume
- **Likelihood**: Medium
- **Impact**: High (slow uploads, timeouts)
- **Mitigation**: Monitor MinIO metrics, plan for distributed MinIO cluster or CDN integration
- **Owner**: DevOps

**R-2.2: Community Content Moderation**
- **Risk**: No moderation system in MVP - inappropriate recipes/reviews may be shared
- **Likelihood**: High (inevitable with user-generated content)
- **Impact**: Medium (reputation risk, user trust)
- **Mitigation**: Implement flagging system in post-MVP iteration, manual moderation initially
- **Owner**: Product Manager

**R-2.3: Freemium Limit Bypass**
- **Risk**: Users may attempt to bypass 10-recipe limit via direct API manipulation
- **Likelihood**: Low (domain enforces limit)
- **Impact**: Low (premium revenue leak)
- **Mitigation**: Enforce limit in domain command (not UI), log bypass attempts
- **Owner**: Security Engineer

**R-2.4: Rating Manipulation**
- **Risk**: Users create multiple accounts to artificially inflate recipe ratings
- **Likelihood**: Medium
- **Impact**: Medium (trust in community quality signals)
- **Mitigation**: Rate limiting on rating submission (future), email verification required (future)
- **Owner**: Product Manager

**R-2.5: Image Copyright Violation**
- **Risk**: Users upload copyrighted recipe images without permission
- **Likelihood**: High
- **Impact**: High (DMCA takedown requests, legal liability)
- **Mitigation**: DMCA policy documented, takedown process established, image scanning (future)
- **Owner**: Legal/Compliance

### Assumptions

**A-2.1: Manual Recipe Entry Acceptable**
- **Assumption**: Users willing to manually enter recipes (no import from URLs/files in MVP)
- **Validation**: User research indicates 70%+ users have <15 recipes to start
- **Risk if False**: High friction, low adoption → Add recipe import in post-MVP

**A-2.2: SQLite Handles Recipe Queries at Scale**
- **Assumption**: SQLite with indexes sufficient for 10K users, community feed queries
- **Validation**: Load testing shows <500ms query times with 100K recipes
- **Risk if False**: Migration to PostgreSQL required earlier than planned

**A-2.3: Community Sharing Drives Growth**
- **Assumption**: 30%+ users will share recipes to community, driving organic discovery
- **Validation**: Similar platforms (AllRecipes, Yummly) show 20-40% sharing rate
- **Risk if False**: Low community content → Invest in recipe seeding, partnerships

**A-2.4: Aggregate Rating Eventual Consistency Acceptable**
- **Assumption**: Users tolerate slight delay (< 1 second) in aggregate rating updates
- **Validation**: evento subscriptions process events near-real-time
- **Risk if False**: Implement synchronous aggregate update (performance trade-off)

**A-2.5: Image Quality Not Critical**
- **Assumption**: Compressed JPEGs (85% quality) acceptable for recipe images
- **Validation**: User feedback on image quality (to be validated in beta)
- **Risk if False**: Support lossless formats, higher quality uploads (storage cost increase)

### Open Questions

**OQ-2.1: Recipe Image Requirements**
- **Question**: Should we enforce minimum image dimensions (e.g., 800x600px) or aspect ratio?
- **Impact**: Better visual consistency vs. user friction
- **Decision Needed By**: Development start (Story 2.4)
- **Stakeholder**: UX Designer, Product Manager

**OQ-2.2: Rating Aggregation Strategy**
- **Question**: Should we use weighted average (recent ratings count more) or simple average?
- **Impact**: Algorithm complexity vs. signal quality for trending recipes
- **Decision Needed By**: Story 2.14 implementation
- **Stakeholder**: Product Manager, Data Analyst

**OQ-2.3: Community Recipe Licensing**
- **Question**: What license applies to shared recipes (CC BY, CC BY-SA, proprietary)?
- **Impact**: Legal clarity, user trust, commercial use restrictions
- **Decision Needed By**: Before MVP launch (terms of service)
- **Stakeholder**: Legal/Compliance

**OQ-2.4: Deleted Recipe Visibility in Meal Plans**
- **Question**: If recipe is deleted while in active meal plan, what happens? Show placeholder? Replace automatically?
- **Impact**: User experience, meal plan integrity
- **Decision Needed By**: Integration with Epic 3 (Meal Planning)
- **Stakeholder**: Product Manager, Engineering Lead

**OQ-2.5: Search Implementation Priority**
- **Question**: Should we prioritize basic keyword search in MVP or defer to post-MVP?
- **Impact**: User discoverability vs. development timeline
- **Decision Needed By**: Sprint planning for Epic 2
- **Stakeholder**: Product Manager, Engineering Lead
- **Current Status**: Deferred to post-MVP (AC-2.12 documents exclusion)

## Test Strategy Summary

### Unit Tests (TDD Approach)

**Domain Aggregate Tests (`crates/recipe/tests/aggregate_tests.rs`):**
- Test `RecipeAggregate` event application (RecipeCreated, RecipeUpdated, etc.)
- Test recipe complexity calculation logic (simple/moderate/complex)
- Test freemium limit enforcement (CreateRecipeCommand with free tier)
- Test ownership validation (update/delete commands)
- Test soft delete logic (deleted_at timestamp)

**Rating Aggregate Tests (`crates/ratings/tests/aggregate_tests.rs`):**
- Test `RatingAggregate` event application (RecipeRated, RatingUpdated)
- Test rating uniqueness constraint (one rating per user per recipe)
- Test aggregate rating recalculation logic (average, distribution)

**Read Model Tests (`crates/recipe/tests/read_model_tests.rs`):**
- Test `list_user_recipes` with filters (collection_id, favorite_only)
- Test `list_shared_recipes` with filters (cuisine, rating, prep_time, dietary)
- Test `count_user_recipes` for freemium limit checks
- Test aggregate rating query accuracy

**Coverage Target**: 80% code coverage (per NFRs), enforced in CI

### Integration Tests (Root Level)

**Recipe CRUD Integration (`tests/recipe_tests.rs`):**
```rust
#[tokio::test]
async fn test_create_recipe_full_flow() {
    let app = test_app().await;
    let client = authenticated_client(&app, "test@example.com").await;

    // Create recipe with image
    let image_data = load_test_image();
    let resp = client.post("/recipes")
        .multipart(/* form data with image */)
        .send().await.unwrap();

    assert_eq!(resp.status(), StatusCode::SEE_OTHER);

    // Verify recipe in database
    let recipe = get_recipe_from_db(&app.db, recipe_id).await;
    assert_eq!(recipe.title, "Test Recipe");
    assert!(recipe.image_url.is_some());

    // Verify image in MinIO
    let image_exists = minio_client.object_exists(&recipe.image_url.unwrap()).await.unwrap();
    assert!(image_exists);
}

#[tokio::test]
async fn test_freemium_limit_enforcement() {
    let app = test_app().await;
    let client = authenticated_client_free_tier(&app, "free@example.com").await;

    // Create 10 recipes (limit)
    for i in 0..10 {
        let resp = create_recipe(&client, &format!("Recipe {}", i)).await;
        assert_eq!(resp.status(), StatusCode::SEE_OTHER);
    }

    // 11th recipe should fail
    let resp = create_recipe(&client, "Recipe 11").await;
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert!(resp.text().await.unwrap().contains("Upgrade to Premium"));
}
```

**Community Discovery Integration (`tests/community_tests.rs`):**
- Test `/discover` public access (no auth required)
- Test `/discover` filters (cuisine, rating, dietary)
- Test "Add to Library" creates copy in user's recipes
- Test rating submission updates aggregate rating
- Test SEO meta tags present in HTML

**Rating Integration (`tests/rating_tests.rs`):**
- Test rate recipe creates `RecipeRated` event and updates read model
- Test updating existing rating (unique constraint)
- Test aggregate rating recalculation on new rating
- Test review list ordered by date DESC

### E2E Tests (Playwright)

**Recipe Management E2E (`e2e/tests/recipe-management.spec.ts`):**
```typescript
test('complete recipe lifecycle', async ({ page }) => {
  // Login
  await login(page, 'test@example.com', 'password123');

  // Navigate to create recipe
  await page.goto('/recipes/new');

  // Fill form
  await page.fill('input[name="title"]', 'Chicken Tikka Masala');
  await page.fill('textarea[name="ingredients"]', JSON.stringify([
    { name: 'chicken', quantity: 2, unit: 'lbs' },
    { name: 'yogurt', quantity: 1, unit: 'cup' }
  ]));
  await page.fill('textarea[name="instructions"]', JSON.stringify([
    'Marinate chicken in yogurt',
    'Grill chicken until cooked'
  ]));
  await page.fill('input[name="prep_time_min"]', '20');
  await page.fill('input[name="cook_time_min"]', '30');
  await page.setInputFiles('input[name="image"]', 'fixtures/recipe-image.jpg');

  // Submit
  await page.click('button[type="submit"]');

  // Verify redirect to detail page
  await page.waitForURL(/\/recipes\/.+/);
  expect(await page.textContent('h1')).toBe('Chicken Tikka Masala');

  // Verify image displayed
  const imgSrc = await page.getAttribute('img.recipe-image', 'src');
  expect(imgSrc).toContain('minio');

  // Mark as favorite
  await page.click('button#favorite-toggle');
  expect(await page.isVisible('.favorite-indicator')).toBe(true);

  // Delete recipe
  await page.click('button#delete-recipe');
  await page.click('button#confirm-delete');

  // Verify redirect to recipe list
  await page.waitForURL('/recipes');
  expect(await page.textContent('.recipe-count')).toBe('0 recipes');
});
```

**Community Discovery E2E (`e2e/tests/community.spec.ts`):**
- Test guest user can browse `/discover` without login
- Test filter recipes by cuisine, rating, prep time
- Test pagination (20 recipes per page)
- Test add to library (requires login, redirects to auth if guest)
- Test rate recipe updates aggregate rating in real-time (TwinSpark)

**Freemium Flow E2E (`e2e/tests/freemium.spec.ts`):**
- Test free tier user creates 10 recipes successfully
- Test 11th recipe shows upgrade prompt
- Test upgrade to premium unlocks unlimited recipes
- Test downgrade to free tier re-enforces 10-recipe limit (cannot create new, can keep existing)

### Performance Tests (Future)

**Load Testing (k6 or Locust):**
- Recipe list query: 100 concurrent users, <500ms p95
- Community feed query: 100 concurrent users, <800ms p95
- Image upload: 50 concurrent uploads, <3s completion
- Rating submission: 100 concurrent ratings, <200ms event commit

**Database Benchmarks:**
- Recipe queries with 100K recipes in read model
- Aggregate rating calculation with 10K ratings per recipe
- Index performance validation (EXPLAIN QUERY PLAN analysis)

---

**End of Technical Specification: Epic 2 - Recipe Management System**

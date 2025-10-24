# Technical Specification: Shopping & Preparation Orchestration

Date: 2025-10-11
Author: Jonathan
Epic ID: 4
Status: Draft

---

## Overview

Epic 4 delivers automated shopping list generation and intelligent preparation reminder functionality that eliminates the complexity of advance preparation timing. This epic bridges the gap between meal planning and successful recipe execution by automatically aggregating ingredients across weekly meal plans and sending timely notifications for advance preparation tasks.

The shopping orchestration system generates category-grouped shopping lists with ingredient quantities aggregated across multiple recipes, enabling efficient grocery shopping for entire weeks. The preparation orchestration system schedules and delivers push notifications to users at optimal times before meals that require advance preparation (marination, rising, defrosting, etc.).

This epic implements two new domain crates (`shopping` and `notifications`) with event-sourced aggregates, read model projections, and background workers for notification scheduling. The system integrates Web Push API for browser-native notifications without vendor lock-in, ensuring users receive actionable reminders across devices.

**Key Capabilities Delivered:**
- Weekly shopping list generation with ingredient aggregation and category grouping
- Multi-week shopping list access (current and future weeks)
- Real-time shopping list updates when meal plan changes
- Push notification subscription management (Web Push API)
- Intelligent reminder scheduling for advance preparation tasks
- Morning prep reminders with specific timing guidance
- Day-of cooking reminders
- Prep task completion tracking

**Epic Dependencies:**
- Epic 2 (Recipe Management): Requires recipe ingredients and advance prep metadata
- Epic 3 (Meal Planning): Requires meal plan generation and meal assignment data
- Epic 1 (User Management): Requires user authentication and profile data

---

## Objectives and Scope

### Objectives

1. **Automate Shopping List Generation**: Generate comprehensive weekly shopping lists by aggregating ingredients across all recipes in active meal plan, eliminating manual list creation and reducing planning time by 60%.

2. **Optimize Grocery Shopping Experience**: Group ingredients by category (produce, dairy, meat, pantry) to mirror typical grocery store layouts, reducing shopping time and minimizing forgotten items.

3. **Enable Multi-Week Planning**: Allow users to access shopping lists for current and future weeks, supporting bulk purchasing decisions and advance shopping strategies.

4. **Ensure List Synchronization**: Automatically update shopping lists when meal plans change (individual meal replacements or full regenerations), maintaining data consistency.

5. **Deliver Timely Preparation Reminders**: Send push notifications 4-24 hours before meals requiring advance preparation, ensuring users have sufficient time for marination, rising, defrosting tasks.

6. **Maximize Notification Reach**: Implement Web Push API (browser standard) for cross-platform notifications without requiring native apps or vendor-specific services.

7. **Track Preparation Progress**: Enable users to mark prep tasks complete, providing visual confirmation and reducing cognitive overhead during cooking.

8. **Maintain Full Auditability**: Leverage event sourcing to track all shopping list generations, notification deliveries, and task completions for analytics and troubleshooting.

### Scope

**In Scope:**
- Shopping list generation aggregate and command handlers
- Ingredient aggregation algorithm (sum quantities, deduplicate)
- Category assignment logic (produce, dairy, meat, pantry, spices, baking)
- Shopping list read models and queries
- Shopping list item completion tracking
- Notification scheduling aggregate and event handlers
- Web Push API integration (VAPID-based push notifications)
- Push subscription management (store browser endpoints)
- Background worker for notification delivery
- Morning prep reminder logic (scheduled 8-12 hours before meal)
- Day-of cooking reminders (scheduled 1 hour before meal time)
- Notification read models (pending, sent, dismissed)
- HTTP endpoints for shopping lists and notification preferences
- Askama templates for shopping list UI
- TwinSpark-enhanced shopping list interactions (mark items collected)
- Database migrations for `shopping_lists`, `shopping_list_items`, `notifications`, `push_subscriptions` tables

**Out of Scope (Deferred):**
- Grocery store API integrations (online ordering)
- Barcode scanning for pantry tracking
- Ingredient substitution suggestions
- Price estimation and budget tracking
- Shared shopping lists (family members)
- Custom notification timing preferences (MVP uses fixed schedule)
- SMS/email notifications (Web Push only in MVP)
- Recipe scaling adjustments in shopping list
- Ingredient expiration tracking
- Smart home integration (voice assistants)

### Success Criteria

1. **Shopping List Generation Performance**: Shopping lists generated within 2 seconds for meal plans with up to 21 meals (7 days × 3 meals).

2. **Ingredient Aggregation Accuracy**: 100% accuracy in quantity aggregation for same ingredient across recipes (e.g., "chicken breast 2lbs" + "chicken breast 1lb" = "chicken breast 3lbs").

3. **Category Grouping Coverage**: 95% of ingredients automatically assigned to correct categories (produce, dairy, meat, pantry, spices, baking, other).

4. **List Synchronization**: Shopping lists update within 3 seconds after meal plan changes (meal replacement or regeneration).

5. **Notification Delivery Rate**: 95% of scheduled prep reminders delivered successfully (excluding user permission denials).

6. **Notification Timing Accuracy**: Prep reminders delivered within ±5 minutes of scheduled time (e.g., 8 hours before meal).

7. **Push Subscription Success Rate**: 90% successful subscription creation when user enables notifications (excluding browser compatibility issues).

8. **Task Completion Tracking**: Users can mark shopping items and prep tasks complete with <500ms response time.

9. **Multi-Week Access**: Users can view shopping lists for current week + 3 future weeks without performance degradation.

10. **Audit Trail Completeness**: All shopping list generations, notification schedules, and task completions recorded as events in evento stream.

---

## System Architecture Alignment

### Domain Crate Structure

Epic 4 introduces two new domain crates within the event-sourced monolithic architecture:

**Shopping Crate** (`crates/shopping/`):
- **Aggregate**: `ShoppingListAggregate` - Manages shopping list lifecycle, ingredient aggregation, item completion state
- **Commands**: `GenerateShoppingList`, `MarkItemCollected`, `UpdateShoppingList`
- **Events**: `ShoppingListGenerated`, `ItemCollected`, `ShoppingListUpdated`
- **Read Models**: `shopping_lists` table (metadata), `shopping_list_items` table (aggregated ingredients with categories)
- **Business Logic**:
  - Ingredient aggregation algorithm (sum quantities, unit conversion, deduplication)
  - Category assignment logic (keyword matching and ML-based classification - phase 2)
  - Ingredient parsing and normalization

**Notifications Crate** (`crates/notifications/`):
- **Aggregate**: `NotificationAggregate` - Manages notification scheduling, delivery tracking, user preferences
- **Commands**: `ScheduleReminder`, `SendReminder`, `DismissNotification`, `SubscribeToPush`
- **Events**: `ReminderScheduled`, `ReminderSent`, `ReminderDismissed`, `PushSubscriptionCreated`
- **Read Models**: `notifications` table (scheduled/sent reminders), `push_subscriptions` table (browser push endpoints)
- **Business Logic**:
  - Reminder scheduling algorithm (calculate trigger times based on advance prep hours)
  - Push notification payload generation
  - Background worker (tokio tasks) for scheduled notification delivery
  - VAPID-based Web Push API integration

### Integration with Existing Crates

**Meal Planning Crate Integration**:
- evento subscription: `MealPlanGenerated` event triggers `GenerateShoppingList` command
- evento subscription: `MealReplaced` event triggers `UpdateShoppingList` command
- evento subscription: `MealPlanGenerated` event triggers `ScheduleReminder` commands for all meals with advance prep

**Recipe Crate Integration**:
- Shopping crate queries recipe read model for ingredient details (name, quantity, unit)
- Notifications crate queries recipe read model for advance prep hours and recipe titles

**User Crate Integration**:
- Shopping crate filters by user_id for multi-tenancy
- Notifications crate queries push_subscriptions by user_id for notification delivery

### Data Flow Architecture

**Shopping List Generation Flow**:
```
1. User generates meal plan (Epic 3)
   ↓
2. MealPlanGenerated event published
   ↓
3. Shopping crate evento subscription handler triggered
   ↓
4. GenerateShoppingList command invoked with meal_plan_id
   ↓
5. Shopping aggregate loads meal assignments from meal_planning read model
   ↓
6. For each meal assignment, query recipe read model for ingredients
   ↓
7. Aggregate ingredients (sum quantities, deduplicate, normalize units)
   ↓
8. Assign categories to each ingredient
   ↓
9. ShoppingListGenerated event written to evento stream
   ↓
10. Read model projection inserts rows into shopping_lists and shopping_list_items tables
   ↓
11. User views shopping list at GET /shopping (queries read model)
```

**Notification Scheduling Flow**:
```
1. User generates meal plan (Epic 3)
   ↓
2. MealPlanGenerated event published
   ↓
3. Notifications crate evento subscription handler triggered
   ↓
4. For each meal with advance_prep_hours > 0:
   ↓
5. Calculate reminder trigger time (meal date/time - advance_prep_hours)
   ↓
6. ScheduleReminder command invoked
   ↓
7. ReminderScheduled event written to evento stream
   ↓
8. Read model projection inserts row into notifications table
   ↓
9. Background worker polls notifications table for due reminders
   ↓
10. When reminder due, SendReminder command invoked
   ↓
11. Query push_subscriptions for user_id
   ↓
12. Send push notification via Web Push API (web-push crate)
   ↓
13. ReminderSent event written to evento stream
   ↓
14. Read model projection updates notification status to "sent"
```

### CQRS Implementation

**Commands (Writes)**:
- `GenerateShoppingList`: Creates new shopping list aggregate from meal plan
- `MarkItemCollected`: Updates individual item collected status
- `UpdateShoppingList`: Regenerates shopping list when meal plan changes
- `ScheduleReminder`: Creates scheduled notification for prep task
- `SendReminder`: Delivers push notification to user's devices
- `DismissNotification`: Marks notification as dismissed by user
- `SubscribeToPush`: Stores browser push subscription endpoint

**Queries (Reads)**:
- `GetShoppingListByWeek(user_id, week_start_date)`: Returns shopping list with categorized items
- `GetMultiWeekShoppingLists(user_id, start_week, num_weeks)`: Returns shopping lists for multiple weeks
- `GetPendingNotifications(user_id)`: Returns scheduled notifications not yet sent
- `GetNotificationHistory(user_id, limit)`: Returns sent/dismissed notifications
- `GetPushSubscriptions(user_id)`: Returns active push subscriptions for user

**Read Model Tables**:

```sql
-- Shopping Lists
CREATE TABLE shopping_lists (
  id TEXT PRIMARY KEY,              -- UUID
  user_id TEXT NOT NULL,
  meal_plan_id TEXT NOT NULL,
  week_start_date TEXT NOT NULL,    -- ISO 8601 date, always Monday (week convention)
  generated_at TEXT NOT NULL,       -- ISO 8601 timestamp
  item_count INTEGER NOT NULL,      -- Denormalized count for quick display
  FOREIGN KEY (user_id) REFERENCES users(id),
  FOREIGN KEY (meal_plan_id) REFERENCES meal_plans(id)
);

CREATE INDEX idx_shopping_lists_user_week ON shopping_lists(user_id, week_start_date);

-- Shopping List Items
CREATE TABLE shopping_list_items (
  id TEXT PRIMARY KEY,
  shopping_list_id TEXT NOT NULL,
  ingredient_name TEXT NOT NULL,
  quantity REAL NOT NULL,           -- Aggregated quantity
  unit TEXT NOT NULL,               -- Normalized unit (cups, lbs, grams, etc.)
  category TEXT,                    -- produce|dairy|meat|pantry|spices|baking|other
  is_collected BOOLEAN DEFAULT FALSE,
  recipe_ids TEXT NOT NULL,         -- JSON array of recipe IDs (traceability)
  FOREIGN KEY (shopping_list_id) REFERENCES shopping_lists(id)
);

CREATE INDEX idx_shopping_items_list ON shopping_list_items(shopping_list_id);
CREATE INDEX idx_shopping_items_category ON shopping_list_items(shopping_list_id, category);

-- Notifications
CREATE TABLE notifications (
  id TEXT PRIMARY KEY,
  user_id TEXT NOT NULL,
  notification_type TEXT NOT NULL,  -- prep_reminder|cooking_reminder
  recipe_id TEXT NOT NULL,
  meal_date TEXT NOT NULL,          -- ISO 8601 date
  meal_type TEXT NOT NULL,          -- breakfast|lunch|dinner
  scheduled_time TEXT NOT NULL,     -- ISO 8601 timestamp (when to send)
  sent_at TEXT,                     -- ISO 8601 timestamp (when sent, NULL if pending)
  status TEXT NOT NULL,             -- pending|sent|failed|dismissed
  title TEXT NOT NULL,              -- Notification title
  body TEXT NOT NULL,               -- Notification body text
  action_url TEXT,                  -- Deep link to recipe detail
  FOREIGN KEY (user_id) REFERENCES users(id),
  FOREIGN KEY (recipe_id) REFERENCES recipes(id)
);

CREATE INDEX idx_notifications_user_pending ON notifications(user_id, status, scheduled_time);
CREATE INDEX idx_notifications_due ON notifications(status, scheduled_time)
  WHERE status = 'pending' AND scheduled_time <= datetime('now');

-- Push Subscriptions
CREATE TABLE push_subscriptions (
  id TEXT PRIMARY KEY,
  user_id TEXT NOT NULL,
  endpoint TEXT NOT NULL,           -- Browser push endpoint URL
  p256dh_key TEXT NOT NULL,         -- Encryption key
  auth_key TEXT NOT NULL,           -- Authentication key
  created_at TEXT NOT NULL,
  last_used_at TEXT,                -- Track active subscriptions
  FOREIGN KEY (user_id) REFERENCES users(id),
  UNIQUE(user_id, endpoint)         -- One subscription per browser/device
);

CREATE INDEX idx_push_subscriptions_user ON push_subscriptions(user_id);
```

---

## Detailed Design

### Services and Modules

#### Shopping Crate Architecture

**Module: `crates/shopping/src/lib.rs`**
```rust
pub mod aggregate;      // ShoppingListAggregate
pub mod commands;       // Command types and handlers
pub mod events;         // Event types
pub mod read_model;     // Read model queries
pub mod aggregation;    // Ingredient aggregation logic
pub mod categorization; // Category assignment logic
pub mod error;          // Domain errors

pub use aggregate::ShoppingListAggregate;
pub use commands::{GenerateShoppingList, MarkItemCollected, UpdateShoppingList};
pub use events::{ShoppingListGenerated, ItemCollected, ShoppingListUpdated};
pub use read_model::{ShoppingListQuery, ShoppingListItem};
```

**Module: `aggregation.rs` - Ingredient Aggregation Logic**

Core algorithm for combining ingredients across recipes:

```rust
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Ingredient {
    pub name: String,
    pub quantity: f64,
    pub unit: String,
    pub recipe_id: String,
}

#[derive(Debug, Clone)]
pub struct AggregatedIngredient {
    pub name: String,
    pub quantity: f64,
    pub unit: String,
    pub recipe_ids: Vec<String>,
    pub category: String,
}

/// Aggregates ingredients from multiple recipes
///
/// Algorithm:
/// 1. Normalize ingredient names (lowercase, trim, singular form)
/// 2. Convert units to standard forms (cups, lbs, grams, etc.)
/// 3. Group by normalized name
/// 4. Sum quantities for same ingredient with same unit
/// 5. Assign category based on ingredient name
pub fn aggregate_ingredients(ingredients: Vec<Ingredient>) -> Vec<AggregatedIngredient> {
    let mut grouped: HashMap<String, AggregatedIngredient> = HashMap::new();

    for ingredient in ingredients {
        // Normalize ingredient name
        let normalized_name = normalize_ingredient_name(&ingredient.name);

        // Normalize unit
        let normalized_unit = normalize_unit(&ingredient.unit);

        // Create unique key: "normalized_name::normalized_unit"
        let key = format!("{}::{}", normalized_name, normalized_unit);

        grouped.entry(key)
            .and_modify(|agg| {
                agg.quantity += ingredient.quantity;
                agg.recipe_ids.push(ingredient.recipe_id.clone());
            })
            .or_insert_with(|| AggregatedIngredient {
                name: ingredient.name.clone(),
                quantity: ingredient.quantity,
                unit: normalized_unit,
                recipe_ids: vec![ingredient.recipe_id.clone()],
                category: assign_category(&normalized_name),
            });
    }

    grouped.into_values().collect()
}

/// Normalizes ingredient name (lowercase, trim, singular)
fn normalize_ingredient_name(name: &str) -> String {
    let trimmed = name.trim().to_lowercase();

    // Remove quantity descriptors (e.g., "2 chicken breasts" -> "chicken breast")
    let without_numbers = trimmed
        .split_whitespace()
        .filter(|word| !word.chars().all(|c| c.is_numeric() || c == '.'))
        .collect::<Vec<_>>()
        .join(" ");

    // Singularize (basic implementation - remove trailing 's')
    let singular = if without_numbers.ends_with("ies") {
        without_numbers.trim_end_matches("ies").to_string() + "y"
    } else if without_numbers.ends_with("es") {
        without_numbers.trim_end_matches("es").to_string()
    } else if without_numbers.ends_with('s') && !without_numbers.ends_with("ss") {
        without_numbers.trim_end_matches('s').to_string()
    } else {
        without_numbers
    };

    singular
}

/// Normalizes unit to standard forms
fn normalize_unit(unit: &str) -> String {
    let lower = unit.trim().to_lowercase();

    match lower.as_str() {
        // Volume
        "cup" | "cups" | "c" => "cups".to_string(),
        "tablespoon" | "tablespoons" | "tbsp" | "tbs" => "tbsp".to_string(),
        "teaspoon" | "teaspoons" | "tsp" => "tsp".to_string(),
        "fluid ounce" | "fluid ounces" | "fl oz" => "fl oz".to_string(),
        "pint" | "pints" | "pt" => "pint".to_string(),
        "quart" | "quarts" | "qt" => "quart".to_string(),
        "gallon" | "gallons" | "gal" => "gallon".to_string(),
        "liter" | "liters" | "l" => "liter".to_string(),
        "milliliter" | "milliliters" | "ml" => "ml".to_string(),

        // Weight
        "pound" | "pounds" | "lb" | "lbs" => "lbs".to_string(),
        "ounce" | "ounces" | "oz" => "oz".to_string(),
        "gram" | "grams" | "g" => "grams".to_string(),
        "kilogram" | "kilograms" | "kg" => "kg".to_string(),

        // Count
        "piece" | "pieces" | "pc" => "pieces".to_string(),
        "whole" | "each" => "whole".to_string(),
        "clove" | "cloves" => "cloves".to_string(),

        // Unknown - return as-is
        _ => lower,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aggregate_same_ingredient() {
        let ingredients = vec![
            Ingredient {
                name: "chicken breast".to_string(),
                quantity: 2.0,
                unit: "lbs".to_string(),
                recipe_id: "recipe-1".to_string(),
            },
            Ingredient {
                name: "Chicken Breast".to_string(),
                quantity: 1.0,
                unit: "lb".to_string(),
                recipe_id: "recipe-2".to_string(),
            },
        ];

        let aggregated = aggregate_ingredients(ingredients);

        assert_eq!(aggregated.len(), 1);
        assert_eq!(aggregated[0].quantity, 3.0);
        assert_eq!(aggregated[0].unit, "lbs");
        assert_eq!(aggregated[0].recipe_ids.len(), 2);
    }

    #[test]
    fn test_different_units_not_aggregated() {
        let ingredients = vec![
            Ingredient {
                name: "milk".to_string(),
                quantity: 2.0,
                unit: "cups".to_string(),
                recipe_id: "recipe-1".to_string(),
            },
            Ingredient {
                name: "milk".to_string(),
                quantity: 1.0,
                unit: "liter".to_string(),
                recipe_id: "recipe-2".to_string(),
            },
        ];

        let aggregated = aggregate_ingredients(ingredients);

        // Different units should remain separate (unit conversion future enhancement)
        assert_eq!(aggregated.len(), 2);
    }
}
```

**Module: `categorization.rs` - Category Assignment**

```rust
/// Assigns category to ingredient based on name
///
/// Categories: produce, dairy, meat, pantry, spices, baking, other
pub fn assign_category(ingredient_name: &str) -> String {
    let lower = ingredient_name.to_lowercase();

    // Produce
    if PRODUCE_KEYWORDS.iter().any(|&kw| lower.contains(kw)) {
        return "produce".to_string();
    }

    // Dairy
    if DAIRY_KEYWORDS.iter().any(|&kw| lower.contains(kw)) {
        return "dairy".to_string();
    }

    // Meat/Protein
    if MEAT_KEYWORDS.iter().any(|&kw| lower.contains(kw)) {
        return "meat".to_string();
    }

    // Spices
    if SPICE_KEYWORDS.iter().any(|&kw| lower.contains(kw)) {
        return "spices".to_string();
    }

    // Baking
    if BAKING_KEYWORDS.iter().any(|&kw| lower.contains(kw)) {
        return "baking".to_string();
    }

    // Pantry (default for common staples)
    if PANTRY_KEYWORDS.iter().any(|&kw| lower.contains(kw)) {
        return "pantry".to_string();
    }

    // Other (unknown)
    "other".to_string()
}

const PRODUCE_KEYWORDS: &[&str] = &[
    "lettuce", "spinach", "kale", "arugula", "tomato", "cucumber", "pepper",
    "onion", "garlic", "carrot", "celery", "potato", "broccoli", "cauliflower",
    "zucchini", "eggplant", "mushroom", "apple", "banana", "orange", "lemon",
    "lime", "strawberry", "blueberry", "avocado", "cilantro", "parsley", "basil",
];

const DAIRY_KEYWORDS: &[&str] = &[
    "milk", "cream", "butter", "cheese", "yogurt", "sour cream", "cottage cheese",
    "ricotta", "mozzarella", "parmesan", "cheddar", "feta", "cream cheese",
];

const MEAT_KEYWORDS: &[&str] = &[
    "chicken", "beef", "pork", "lamb", "turkey", "duck", "fish", "salmon",
    "tuna", "shrimp", "bacon", "sausage", "ham", "steak", "ground beef",
    "ground turkey", "chicken breast", "thigh", "drumstick",
];

const SPICE_KEYWORDS: &[&str] = &[
    "salt", "pepper", "paprika", "cumin", "coriander", "turmeric", "cinnamon",
    "nutmeg", "ginger", "oregano", "thyme", "rosemary", "bay leaf", "chili",
    "cayenne", "curry", "garam masala", "cardamom", "clove", "saffron",
];

const BAKING_KEYWORDS: &[&str] = &[
    "flour", "sugar", "brown sugar", "baking powder", "baking soda", "yeast",
    "cornstarch", "vanilla extract", "cocoa powder", "chocolate chips", "honey",
];

const PANTRY_KEYWORDS: &[&str] = &[
    "rice", "pasta", "noodles", "bread", "oil", "olive oil", "vinegar",
    "soy sauce", "chicken stock", "beef stock", "broth", "tomato paste",
    "tomato sauce", "beans", "lentils", "chickpeas", "canned tomatoes",
];
```

#### Notifications Crate Architecture

**Module: `crates/notifications/src/lib.rs`**
```rust
pub mod aggregate;      // NotificationAggregate
pub mod commands;       // Command types and handlers
pub mod events;         // Event types
pub mod read_model;     // Read model queries
pub mod scheduler;      // Background worker for notification delivery
pub mod push;           // Web Push API integration
pub mod error;          // Domain errors

pub use aggregate::NotificationAggregate;
pub use commands::{ScheduleReminder, SendReminder, DismissNotification, SubscribeToPush};
pub use events::{ReminderScheduled, ReminderSent, ReminderDismissed, PushSubscriptionCreated};
pub use scheduler::NotificationScheduler;
```

**Module: `scheduler.rs` - Background Worker**

```rust
use chrono::{DateTime, Utc, Duration};
use tokio::time::{sleep, interval};
use sqlx::SqlitePool;

/// Background worker that polls for due notifications and sends them
pub struct NotificationScheduler {
    pool: SqlitePool,
    poll_interval: std::time::Duration,
}

impl NotificationScheduler {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            pool,
            poll_interval: std::time::Duration::from_secs(60), // Poll every 60 seconds
        }
    }

    /// Starts the background worker (tokio task)
    pub async fn start(self) {
        let mut ticker = interval(self.poll_interval);

        loop {
            ticker.tick().await;

            match self.process_due_notifications().await {
                Ok(count) => {
                    if count > 0 {
                        tracing::info!("Sent {} notifications", count);
                    }
                }
                Err(e) => {
                    tracing::error!("Error processing notifications: {}", e);
                }
            }
        }
    }

    /// Queries for notifications with scheduled_time <= now and status='pending'
    /// Sends each notification and updates status to 'sent'
    async fn process_due_notifications(&self) -> anyhow::Result<usize> {
        let now = Utc::now().to_rfc3339();

        // Query due notifications
        let notifications = sqlx::query(
            r#"
            SELECT id, user_id, notification_type, recipe_id, meal_date, meal_type,
                   title, body, action_url
            FROM notifications
            WHERE status = 'pending'
              AND scheduled_time <= ?
            ORDER BY scheduled_time ASC
            LIMIT 100
            "#
        )
        .bind(now)
        .fetch_all(&self.pool)
        .await?;

        let count = notifications.len();

        for notification in notifications {
            // Send notification via Web Push API
            match self.send_push_notification(&notification).await {
                Ok(_) => {
                    // Update status to 'sent'
                    sqlx::query(
                        r#"
                        UPDATE notifications
                        SET status = 'sent', sent_at = ?
                        WHERE id = ?
                        "#
                    )
                    .bind(now)
                    .bind(notification.id)
                    .execute(&self.pool)
                    .await?;

                    tracing::info!("Sent notification {} to user {}", notification.id, notification.user_id);
                }
                Err(e) => {
                    // Update status to 'failed'
                    sqlx::query(
                        r#"
                        UPDATE notifications
                        SET status = 'failed'
                        WHERE id = ?
                        "#
                    )
                    .bind(notification.id)
                    .execute(&self.pool)
                    .await?;

                    tracing::error!("Failed to send notification {}: {}", notification.id, e);
                }
            }
        }

        Ok(count)
    }

    async fn send_push_notification(&self, notification: &Notification) -> anyhow::Result<()> {
        // Query push subscriptions for user
        let subscriptions = sqlx::query(
            r#"
            SELECT endpoint, p256dh_key, auth_key
            FROM push_subscriptions
            WHERE user_id = ?
            "#
        )
        .bind(notification.user_id)
        .fetch_all(&self.pool)
        .await?;

        if subscriptions.is_empty() {
            tracing::warn!("No push subscriptions found for user {}", notification.user_id);
            return Err(anyhow::anyhow!("No push subscriptions"));
        }

        // Send to all subscriptions (multiple browsers/devices)
        for sub in subscriptions {
            match crate::push::send_web_push(
                &sub.endpoint,
                &sub.p256dh_key,
                &sub.auth_key,
                &notification.title,
                &notification.body,
                notification.action_url.as_deref(),
            ).await {
                Ok(_) => {
                    tracing::info!("Push sent to endpoint {}", sub.endpoint);
                }
                Err(e) => {
                    tracing::error!("Push failed for endpoint {}: {}", sub.endpoint, e);
                    // Continue to other subscriptions (don't fail entire batch)
                }
            }
        }

        Ok(())
    }
}

struct Notification {
    id: String,
    user_id: String,
    notification_type: String,
    recipe_id: String,
    meal_date: String,
    meal_type: String,
    title: String,
    body: String,
    action_url: Option<String>,
}
```

**Module: `push.rs` - Web Push API Integration**

```rust
use web_push::{
    WebPushClient, WebPushMessageBuilder, SubscriptionInfo, VapidSignatureBuilder,
    PartialVapidSignatureBuilder, ContentEncoding,
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};

/// Sends Web Push notification using VAPID authentication
pub async fn send_web_push(
    endpoint: &str,
    p256dh_key: &str,
    auth_key: &str,
    title: &str,
    body: &str,
    action_url: Option<&str>,
) -> anyhow::Result<()> {
    // Load VAPID keys from environment
    let vapid_private_key = std::env::var("VAPID_PRIVATE_KEY")?;
    let vapid_public_key = std::env::var("VAPID_PUBLIC_KEY")?;

    // Build subscription info
    let subscription_info = SubscriptionInfo::new(
        endpoint,
        p256dh_key,
        auth_key,
    );

    // Build notification payload
    let payload = serde_json::json!({
        "title": title,
        "body": body,
        "icon": "/static/icons/icon-192.png",
        "badge": "/static/icons/badge-72.png",
        "actions": [
            {
                "action": "view",
                "title": "View Recipe"
            },
            {
                "action": "dismiss",
                "title": "Dismiss"
            }
        ],
        "data": {
            "url": action_url.unwrap_or("/dashboard")
        }
    });

    let payload_str = serde_json::to_string(&payload)?;

    // Build VAPID signature
    let mut builder = VapidSignatureBuilder::from_pem_no_sub(vapid_private_key.as_bytes())?;
    builder.add_claim("sub", "mailto:noreply@imkitchen.app");
    let signature = builder.build()?;

    // Build Web Push message
    let mut message_builder = WebPushMessageBuilder::new(&subscription_info);
    message_builder.set_payload(ContentEncoding::Aes128Gcm, payload_str.as_bytes());
    message_builder.set_vapid_signature(signature);

    let message = message_builder.build()?;

    // Send via HTTP client
    let client = WebPushClient::new()?;
    client.send(message).await?;

    Ok(())
}

/// Generates VAPID key pair (run once during setup)
pub fn generate_vapid_keys() -> anyhow::Result<(String, String)> {
    use web_push::VapidSignatureBuilder;

    // Generate new key pair
    let key_pair = web_push::VapidSignatureBuilder::generate_keypair()?;

    // Encode keys as base64
    let private_key = URL_SAFE_NO_PAD.encode(&key_pair.private_key);
    let public_key = URL_SAFE_NO_PAD.encode(&key_pair.public_key);

    Ok((private_key, public_key))
}
```

**Module: `aggregate.rs` - Notification Aggregate**

```rust
use evento::{AggregatorName, EventDetails};
use chrono::{DateTime, Utc, Duration};
use serde::{Serialize, Deserialize};

#[derive(Default, Serialize, Deserialize, bincode::Encode, bincode::Decode, Clone, Debug)]
pub struct NotificationAggregate {
    pub id: String,
    pub user_id: String,
    pub notification_type: String,
    pub recipe_id: String,
    pub meal_date: String,
    pub meal_type: String,
    pub scheduled_time: String,
    pub status: String, // pending|sent|failed|dismissed
}

// Events
#[derive(AggregatorName, bincode::Encode, bincode::Decode, Serialize, Deserialize)]
pub struct ReminderScheduled {
    pub user_id: String,
    pub notification_type: String,
    pub recipe_id: String,
    pub meal_date: String,
    pub meal_type: String,
    pub scheduled_time: String,
    pub title: String,
    pub body: String,
    pub action_url: String,
}

#[derive(AggregatorName, bincode::Encode, bincode::Decode, Serialize, Deserialize)]
pub struct ReminderSent {
    pub sent_at: String,
}

#[derive(AggregatorName, bincode::Encode, bincode::Decode, Serialize, Deserialize)]
pub struct ReminderDismissed {
    pub dismissed_at: String,
}

// Aggregate event handlers
#[evento::aggregator]
impl NotificationAggregate {
    async fn reminder_scheduled(&mut self, event: EventDetails<ReminderScheduled>) -> anyhow::Result<()> {
        self.user_id = event.data.user_id;
        self.notification_type = event.data.notification_type;
        self.recipe_id = event.data.recipe_id;
        self.meal_date = event.data.meal_date;
        self.meal_type = event.data.meal_type;
        self.scheduled_time = event.data.scheduled_time;
        self.status = "pending".to_string();
        Ok(())
    }

    async fn reminder_sent(&mut self, event: EventDetails<ReminderSent>) -> anyhow::Result<()> {
        self.status = "sent".to_string();
        Ok(())
    }

    async fn reminder_dismissed(&mut self, event: EventDetails<ReminderDismissed>) -> anyhow::Result<()> {
        self.status = "dismissed".to_string();
        Ok(())
    }
}

/// Calculates reminder scheduled time based on meal time and advance prep hours
pub fn calculate_reminder_time(
    meal_datetime: DateTime<Utc>,
    advance_prep_hours: i64,
) -> DateTime<Utc> {
    // Schedule reminder for morning (9am) on the day we need to start prep
    let prep_start = meal_datetime - Duration::hours(advance_prep_hours);

    // Round to 9am on that day
    prep_start
        .date_naive()
        .and_hms_opt(9, 0, 0)
        .unwrap()
        .and_utc()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_calculate_reminder_time() {
        // Meal on Thursday 6pm, 24 hour marinade
        let meal_time = Utc.with_ymd_and_hms(2025, 10, 16, 18, 0, 0).unwrap();
        let advance_hours = 24;

        let reminder = calculate_reminder_time(meal_time, advance_hours);

        // Should be Wednesday 9am
        assert_eq!(reminder.date_naive().day(), 15);
        assert_eq!(reminder.hour(), 9);
        assert_eq!(reminder.minute(), 0);
    }
}
```

### Data Models and Contracts

#### Shopping List Domain Models

**Command: GenerateShoppingList**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateShoppingList {
    pub user_id: String,
    pub meal_plan_id: String,
    pub week_start_date: String, // ISO 8601 date
}
```

**Event: ShoppingListGenerated**
```rust
#[derive(AggregatorName, bincode::Encode, bincode::Decode, Serialize, Deserialize)]
pub struct ShoppingListGenerated {
    pub user_id: String,
    pub meal_plan_id: String,
    pub week_start_date: String,
    pub items: Vec<ShoppingItem>,
    pub item_count: usize,
}

#[derive(Debug, Clone, bincode::Encode, bincode::Decode, Serialize, Deserialize)]
pub struct ShoppingItem {
    pub ingredient_name: String,
    pub quantity: f64,
    pub unit: String,
    pub category: String,
    pub recipe_ids: Vec<String>, // Traceability
}
```

**Command: MarkItemCollected**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkItemCollected {
    pub shopping_list_id: String,
    pub item_id: String,
    pub is_collected: bool,
}
```

**Event: ItemCollected**
```rust
#[derive(AggregatorName, bincode::Encode, bincode::Decode, Serialize, Deserialize)]
pub struct ItemCollected {
    pub item_id: String,
    pub is_collected: bool,
    pub collected_at: String, // ISO 8601 timestamp
}
```

#### Notification Domain Models

**Command: ScheduleReminder**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleReminder {
    pub user_id: String,
    pub notification_type: NotificationType,
    pub recipe_id: String,
    pub recipe_title: String,
    pub meal_date: String, // ISO 8601 date
    pub meal_type: MealType,
    pub scheduled_time: String, // ISO 8601 timestamp
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationType {
    PrepReminder,
    CookingReminder,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MealType {
    Breakfast,
    Lunch,
    Dinner,
}
```

**Command: SubscribeToPush**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscribeToPush {
    pub user_id: String,
    pub endpoint: String,
    pub p256dh_key: String,
    pub auth_key: String,
}
```

**Event: PushSubscriptionCreated**
```rust
#[derive(AggregatorName, bincode::Encode, bincode::Decode, Serialize, Deserialize)]
pub struct PushSubscriptionCreated {
    pub user_id: String,
    pub endpoint: String,
    pub p256dh_key: String,
    pub auth_key: String,
    pub created_at: String,
}
```

#### Read Model Query Results

**Shopping List Query Result**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShoppingListView {
    pub id: String,
    pub user_id: String,
    pub meal_plan_id: String,
    pub week_start_date: String,
    pub generated_at: String,
    pub categories: Vec<CategoryGroup>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryGroup {
    pub category: String,
    pub items: Vec<ShoppingListItemView>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShoppingListItemView {
    pub id: String,
    pub ingredient_name: String,
    pub quantity: f64,
    pub unit: String,
    pub is_collected: bool,
    pub recipe_ids: Vec<String>,
}
```

**Notification Query Result**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationView {
    pub id: String,
    pub notification_type: String,
    pub recipe_id: String,
    pub recipe_title: String,
    pub meal_date: String,
    pub meal_type: String,
    pub scheduled_time: String,
    pub sent_at: Option<String>,
    pub status: String,
    pub title: String,
    pub body: String,
    pub action_url: String,
}
```

### APIs and Interfaces

#### HTTP Endpoints

**Shopping List Endpoints**

```
GET  /shopping
     Description: Display current week's shopping list
     Auth: Required (JWT cookie)
     Response: HTML page with shopping list grouped by category
     Template: templates/pages/shopping-list.html
     Query: ShoppingListQuery::get_by_week(user_id, current_week_start)

GET  /shopping/week/:week_start_date
     Description: Display specific week's shopping list
     Auth: Required
     Params: week_start_date (ISO 8601 date, must be Monday)
     Response: HTML page with shopping list for specified week
     Template: templates/pages/shopping-list.html
     Query: ShoppingListQuery::get_by_week(user_id, week_start_date)

POST /shopping/:shopping_list_id/item/:item_id/collect
     Description: Mark shopping list item as collected/uncollected
     Auth: Required
     Params: shopping_list_id, item_id
     Form: is_collected (boolean)
     Response: 200 OK with HTML fragment (TwinSpark)
     Template: templates/partials/shopping-item.html
     Command: MarkItemCollected { shopping_list_id, item_id, is_collected }

GET  /shopping/multi-week
     Description: Display shopping lists for multiple weeks
     Auth: Required
     Query Params: start_week (ISO date), num_weeks (integer, default 4)
     Response: HTML page with accordion of weekly shopping lists
     Template: templates/pages/shopping-multi-week.html
     Query: ShoppingListQuery::get_multi_week(user_id, start_week, num_weeks)
```

**Notification Endpoints**

```
GET  /notifications
     Description: Display notification settings and history
     Auth: Required
     Response: HTML page with push subscription status and notification history
     Template: templates/pages/notifications.html
     Query: NotificationQuery::get_user_notifications(user_id)

POST /notifications/subscribe
     Description: Subscribe to push notifications
     Auth: Required
     Form: endpoint, p256dh_key, auth_key (from browser push API)
     Response: 200 OK with success message
     Command: SubscribeToPush { user_id, endpoint, p256dh_key, auth_key }

POST /notifications/:notification_id/dismiss
     Description: Dismiss notification
     Auth: Required
     Params: notification_id
     Response: 200 OK with HTML fragment (TwinSpark)
     Command: DismissNotification { notification_id, user_id }

GET  /api/vapid-public-key
     Description: Returns VAPID public key for Web Push subscription
     Auth: Not required (public key is public)
     Response: JSON { "publicKey": "..." }
     Note: Used by client-side JavaScript to subscribe to push
```

#### Client-Side JavaScript (Web Push Subscription)

**File: `static/js/notifications.js`**

```javascript
// Request notification permission and subscribe to Web Push
async function subscribeToNotifications() {
    // Check browser support
    if (!('serviceWorker' in navigator) || !('PushManager' in window)) {
        alert('Push notifications not supported in this browser');
        return;
    }

    // Request notification permission
    const permission = await Notification.requestPermission();
    if (permission !== 'granted') {
        alert('Notification permission denied');
        return;
    }

    // Get service worker registration
    const registration = await navigator.serviceWorker.ready;

    // Fetch VAPID public key from server
    const response = await fetch('/api/vapid-public-key');
    const { publicKey } = await response.json();

    // Subscribe to push notifications
    const subscription = await registration.pushManager.subscribe({
        userVisibleOnly: true,
        applicationServerKey: urlBase64ToUint8Array(publicKey)
    });

    // Send subscription to server
    await fetch('/notifications/subscribe', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify({
            endpoint: subscription.endpoint,
            p256dh_key: btoa(String.fromCharCode(...new Uint8Array(subscription.getKey('p256dh')))),
            auth_key: btoa(String.fromCharCode(...new Uint8Array(subscription.getKey('auth')))),
        }),
    });

    alert('Successfully subscribed to notifications!');
}

// Helper function to convert VAPID key
function urlBase64ToUint8Array(base64String) {
    const padding = '='.repeat((4 - base64String.length % 4) % 4);
    const base64 = (base64String + padding)
        .replace(/\-/g, '+')
        .replace(/_/g, '/');

    const rawData = window.atob(base64);
    const outputArray = new Uint8Array(rawData.length);

    for (let i = 0; i < rawData.length; ++i) {
        outputArray[i] = rawData.charCodeAt(i);
    }
    return outputArray;
}

// Attach to button click
document.getElementById('subscribe-notifications').addEventListener('click', subscribeToNotifications);
```

**Service Worker: `static/js/sw.js` (Notification Handler)**

```javascript
// Handle notification click (open recipe URL)
self.addEventListener('notificationclick', (event) => {
    event.notification.close();

    if (event.action === 'view') {
        const url = event.notification.data.url || '/dashboard';
        event.waitUntil(
            clients.openWindow(url)
        );
    } else if (event.action === 'dismiss') {
        // Send dismiss event to server
        fetch('/notifications/' + event.notification.data.notification_id + '/dismiss', {
            method: 'POST',
        });
    }
});

// Handle push message received
self.addEventListener('push', (event) => {
    const data = event.data.json();

    const options = {
        body: data.body,
        icon: data.icon,
        badge: data.badge,
        actions: data.actions,
        data: data.data,
    };

    event.waitUntil(
        self.registration.showNotification(data.title, options)
    );
});
```

### Workflows and Sequencing

#### Workflow 1: Shopping List Generation from Meal Plan

**Trigger**: User generates meal plan (Epic 3)

**Sequence Diagram**:
```
User             Axum Handler          MealPlanning Crate    Evento         Shopping Crate       Read Model
 |                     |                        |                |                 |                |
 | POST /plan/generate |                        |                |                 |                |
 |-------------------->|                        |                |                 |                |
 |                     |                        |                |                 |                |
 |                     | GenerateMealPlan cmd   |                |                 |                |
 |                     |----------------------->|                |                 |                |
 |                     |                        |                |                 |                |
 |                     |                        | MealPlanGenerated event          |                |
 |                     |                        |--------------->|                 |                |
 |                     |                        |                |                 |                |
 |                     |                        |                | evento subscription triggers    |
 |                     |                        |                |---------------->|                |
 |                     |                        |                |                 |                |
 |                     |                        |                |  Load meal assignments from      |
 |                     |                        |                |  meal_planning read model        |
 |                     |                        |                |                 |                |
 |                     |                        |                |  For each meal, load recipe      |
 |                     |                        |                |  ingredients from recipe read    |
 |                     |                        |                |                 |                |
 |                     |                        |                |  Aggregate ingredients           |
 |                     |                        |                |  (sum quantities, dedupe)        |
 |                     |                        |                |                 |                |
 |                     |                        |                |  Assign categories               |
 |                     |                        |                |                 |                |
 |                     |                        |                | ShoppingListGenerated event      |
 |                     |                        |                |<----------------|                |
 |                     |                        |                |                 |                |
 |                     |                        |                | evento subscription triggers     |
 |                     |                        |                |--------------------------------->|
 |                     |                        |                |                 |                |
 |                     |                        |                |  INSERT shopping_lists           |
 |                     |                        |                |  INSERT shopping_list_items       |
 |                     |                        |                |                 |                |
 |<---- 302 Redirect to /plan --------------------------------------------------|
 |                     |                        |                |                 |                |
```

**Step-by-Step Flow**:

1. **User Action**: User clicks "Generate Meal Plan" button, submits POST to `/plan/generate`
2. **Route Handler**: Axum handler validates auth, invokes `meal_planning::generate_meal_plan(cmd)`
3. **Meal Planning Command**: `MealPlanAggregate` processes command, emits `MealPlanGenerated` event to evento stream
4. **Event Subscription**: Shopping crate has evento subscription listening for `MealPlanGenerated` events
5. **Subscription Handler Triggered**: Handler function `on_meal_plan_generated()` receives event details
6. **Load Meal Assignments**: Query `meal_assignments` table for all meals in meal plan
7. **Load Recipe Ingredients**: For each meal assignment, query `recipes` table to get ingredient list (JSON field)
8. **Aggregate Ingredients**:
   - Parse ingredients from all recipes
   - Normalize ingredient names (lowercase, trim, singular)
   - Normalize units (cups, lbs, grams, etc.)
   - Group by normalized name and unit
   - Sum quantities for same ingredient
9. **Assign Categories**: Use keyword matching to assign category to each ingredient (produce, dairy, meat, etc.)
10. **Emit Event**: Create `ShoppingListGenerated` event with aggregated items, commit to evento stream
11. **Read Model Projection**: Another subscription handler listens for `ShoppingListGenerated` events
12. **Persist Read Model**: Insert row into `shopping_lists` table, insert rows into `shopping_list_items` table
13. **User Redirect**: User redirected to meal calendar (`/plan`), can now navigate to shopping list

**Error Handling**:
- If meal plan has no meals: Skip shopping list generation (no-op)
- If recipe ingredient data malformed: Log error, use empty ingredient list for that recipe
- If aggregation fails: Log error, generate partial shopping list with available data
- If database insert fails: Retry with exponential backoff (evento guarantees event durability)

#### Workflow 2: Notification Scheduling and Delivery

**Trigger**: User generates meal plan with recipes containing advance prep requirements

**Sequence Diagram**:
```
MealPlanning Crate    Evento         Notifications Crate      Background Worker    Web Push API
       |                 |                    |                       |                    |
       | MealPlanGenerated event              |                       |                    |
       |---------------->|                    |                       |                    |
       |                 |                    |                       |                    |
       |                 | evento subscription triggers               |                    |
       |                 |------------------->|                       |                    |
       |                 |                    |                       |                    |
       |                 |  For each meal with advance_prep_hours:   |                    |
       |                 |                    |                       |                    |
       |                 |  Calculate reminder_time                  |                    |
       |                 |  (meal_datetime - advance_prep_hours)     |                    |
       |                 |                    |                       |                    |
       |                 |  ScheduleReminder command                 |                    |
       |                 |                    |                       |                    |
       |                 |  ReminderScheduled event                  |                    |
       |                 |<-------------------|                       |                    |
       |                 |                    |                       |                    |
       |                 | evento subscription updates read model    |                    |
       |                 |                    |                       |                    |
       |                 |  INSERT notifications (status='pending')  |                    |
       |                 |                    |                       |                    |
       |        [Time passes - reminder becomes due]                 |                    |
       |                 |                    |                       |                    |
       |                 |                    |   Poll for due notifications             |
       |                 |                    |   (scheduled_time <= now)                 |
       |                 |                    |<----------------------|                    |
       |                 |                    |                       |                    |
       |                 |                    |   Query push_subscriptions for user      |
       |                 |                    |                       |                    |
       |                 |                    |   Send Web Push notification             |
       |                 |                    |                       |------------------->|
       |                 |                    |                       |                    |
       |                 |                    |   ReminderSent command                    |
       |                 |  ReminderSent event|                       |                    |
       |                 |<-------------------|                       |                    |
       |                 |                    |                       |                    |
       |                 | evento subscription updates read model    |                    |
       |                 |                    |                       |                    |
       |                 |  UPDATE notifications (status='sent')     |                    |
       |                 |                    |                       |                    |
```

**Step-by-Step Flow**:

1. **Meal Plan Generated**: User generates meal plan containing recipes with `advance_prep_hours > 0`
2. **Event Subscription**: Notifications crate subscribes to `MealPlanGenerated` events
3. **Filter Meals Requiring Prep**: Handler filters meal assignments where recipe has advance prep requirements
4. **Calculate Reminder Times**: For each meal:
   - `meal_datetime` = meal date + typical meal time (breakfast 8am, lunch 12pm, dinner 6pm)
   - `prep_start_datetime` = meal_datetime - advance_prep_hours
   - `reminder_time` = 9am on day of prep_start (morning reminder)
5. **Schedule Reminders**: Invoke `ScheduleReminder` command for each reminder
6. **Emit Events**: `ReminderScheduled` events written to evento stream
7. **Persist Read Model**: Subscription handler inserts rows into `notifications` table with `status='pending'`
8. **Background Worker Polling**: Tokio task runs every 60 seconds, queries notifications with `scheduled_time <= now AND status='pending'`
9. **Load Push Subscriptions**: For each due notification, query `push_subscriptions` table for user's browser endpoints
10. **Send Push Notifications**: Use `web-push` crate to send VAPID-signed push notification to each endpoint
11. **Update Status**: Invoke `SendReminder` command, emit `ReminderSent` event
12. **Read Model Update**: Subscription handler updates notification `status='sent'` and sets `sent_at` timestamp
13. **User Receives Notification**: Browser displays notification with title, body, and action buttons
14. **User Interaction**: User clicks "View Recipe" to open recipe detail page, or "Dismiss" to mark complete

**Error Handling**:
- If no push subscriptions found: Log warning, mark notification as 'failed' (user hasn't enabled notifications)
- If Web Push API returns 410 Gone: Delete subscription (browser unsubscribed), mark notification as 'failed'
- If Web Push API times out: Retry up to 3 times with exponential backoff, then mark as 'failed'
- If background worker crashes: Restart via Kubernetes liveness probe, poll again on next interval

#### Workflow 3: Shopping List Item Collection (TwinSpark AJAX)

**Trigger**: User taps checkbox next to shopping list item

**Sequence Diagram**:
```
User (Browser)    TwinSpark    Axum Handler       Shopping Crate      Read Model
      |                |             |                   |                 |
      | Tap checkbox   |             |                   |                 |
      |--------------->|             |                   |                 |
      |                |             |                   |                 |
      | POST /shopping/:list_id/item/:item_id/collect   |                 |
      |                |------------>|                   |                 |
      |                |             |                   |                 |
      |                |             | MarkItemCollected command           |
      |                |             |------------------>|                 |
      |                |             |                   |                 |
      |                |             |  ItemCollected event                |
      |                |             |<------------------|                 |
      |                |             |                   |                 |
      |                |             | evento subscription updates read model
      |                |             |                   |---------------->|
      |                |             |                   |                 |
      |                |             |  UPDATE shopping_list_items         |
      |                |             |     SET is_collected = true         |
      |                |             |                   |                 |
      |                | HTML fragment (updated item)    |                 |
      |<---------------------------- |                   |                 |
      |                |             |                   |                 |
      | TwinSpark swaps DOM element |                   |                 |
      |                |             |                   |                 |
```

**Step-by-Step Flow**:

1. **User Interaction**: User taps checkbox next to "chicken breast 2lbs" in shopping list UI
2. **TwinSpark Intercept**: TwinSpark intercepts form submit, sends AJAX POST to `/shopping/list-123/item/item-456/collect`
3. **Route Handler**: Axum handler validates auth, extracts `shopping_list_id`, `item_id`, and `is_collected` from form
4. **Authorization Check**: Verify shopping list belongs to authenticated user (query `shopping_lists.user_id`)
5. **Invoke Command**: Call `shopping::mark_item_collected(cmd)`
6. **Aggregate Logic**: Shopping aggregate processes command, emits `ItemCollected` event with timestamp
7. **Persist Event**: Event written to evento stream
8. **Read Model Update**: Subscription handler updates `shopping_list_items.is_collected = true` and sets timestamp
9. **Render Partial Template**: Handler renders `templates/partials/shopping-item.html` with updated item state
10. **Return HTML Fragment**: Response contains updated `<li>` element with checked checkbox and strike-through styling
11. **TwinSpark Swap**: TwinSpark replaces existing `<li>` in DOM with new HTML fragment
12. **Visual Feedback**: User sees item strike-through, haptic feedback (mobile), smooth transition

**Askama Template** (`templates/partials/shopping-item.html`):
```html
<li id="shopping-item-{{ item.id }}" class="shopping-item {% if item.is_collected %}collected{% endif %}">
  <form ts-req="/shopping/{{ shopping_list_id }}/item/{{ item.id }}/collect"
        ts-req-method="POST"
        ts-target="#shopping-item-{{ item.id }}"
        ts-swap="outerHTML">
    <input type="checkbox"
           name="is_collected"
           value="true"
           {% if item.is_collected %}checked{% endif %}
           onchange="this.form.requestSubmit()">
    <span class="ingredient-name">{{ item.ingredient_name }}</span>
    <span class="quantity">{{ item.quantity }} {{ item.unit }}</span>
  </form>
</li>
```

**CSS Styling**:
```css
.shopping-item.collected .ingredient-name {
  text-decoration: line-through;
  color: var(--text-muted);
}

.shopping-item {
  transition: all 0.2s ease;
}
```

#### Workflow 4: Multi-Week Shopping List Access

**Trigger**: User navigates to "Multi-Week Shopping" view

**Sequence Diagram**:
```
User             Axum Handler          Shopping Read Model
 |                     |                        |
 | GET /shopping/multi-week?start_week=2025-10-14&num_weeks=4
 |-------------------->|                        |
 |                     |                        |
 |                     | Query shopping lists   |
 |                     | for 4 consecutive weeks|
 |                     |----------------------->|
 |                     |                        |
 |                     | SELECT * FROM shopping_lists
 |                     | WHERE user_id = ? AND week_start_date IN (...)
 |                     |                        |
 |                     | SELECT * FROM shopping_list_items
 |                     | WHERE shopping_list_id IN (...)
 |                     |                        |
 |                     |<-----------------------|
 |                     |                        |
 |                     | Render accordion template with 4 weeks
 |                     |                        |
 |<--------------------|                        |
 |                     |                        |
```

**Query Logic**:
```rust
pub async fn get_multi_week_shopping_lists(
    pool: &SqlitePool,
    user_id: &str,
    start_week: NaiveDate,
    num_weeks: usize,
) -> Result<Vec<ShoppingListView>, Error> {
    // Generate list of week start dates
    let mut week_dates = Vec::new();
    for i in 0..num_weeks {
        let week_date = start_week + Duration::weeks(i as i64);
        week_dates.push(week_date.to_string());
    }

    // Query shopping lists for all weeks
    let lists = sqlx::query_as::<_, ShoppingListRow>(
        r#"
        SELECT id, user_id, meal_plan_id, week_start_date, generated_at, item_count
        FROM shopping_lists
        WHERE user_id = ? AND week_start_date IN (?, ?, ?, ?)
        ORDER BY week_start_date ASC
        "#
    )
    .bind(user_id)
    .bind(week_dates.get(0))
    .bind(week_dates.get(1))
    .bind(week_dates.get(2))
    .bind(week_dates.get(3))
    .fetch_all(pool)
    .await?;

    // For each list, query items
    let mut result = Vec::new();
    for list in lists {
        let items = query_shopping_list_items(pool, &list.id).await?;
        result.push(ShoppingListView {
            id: list.id,
            user_id: list.user_id,
            week_start_date: list.week_start_date,
            generated_at: list.generated_at,
            categories: group_by_category(items),
        });
    }

    Ok(result)
}
```

**Askama Template** (`templates/pages/shopping-multi-week.html`):
```html
{% extends "base.html" %}

{% block content %}
<div class="container">
  <h1>Multi-Week Shopping Lists</h1>

  <div class="accordion">
    {% for list in lists %}
    <div class="accordion-item">
      <h2 class="accordion-header">
        <button class="accordion-button" type="button"
                data-bs-toggle="collapse"
                data-bs-target="#week-{{ list.id }}">
          Week of {{ list.week_start_date | format_date }}
          <span class="badge">{{ list.categories | total_items }} items</span>
        </button>
      </h2>
      <div id="week-{{ list.id }}" class="accordion-collapse collapse">
        <div class="accordion-body">
          {% include "partials/shopping-category-list.html" %}
        </div>
      </div>
    </div>
    {% endfor %}
  </div>
</div>
{% endblock %}
```

---

## Non-Functional Requirements

### Performance

**Shopping List Generation**:
- **Target**: Complete within 2 seconds for meal plans with up to 21 meals (7 days × 3 meals/day)
- **Optimization Strategies**:
  - Single database query to load all meal assignments for meal plan
  - Batch query to load all recipe details (use IN clause with recipe IDs)
  - In-memory ingredient aggregation (no database writes during aggregation)
  - Single bulk insert for shopping list items (batch INSERT statement)
- **Measurement**: OpenTelemetry span instrumentation tracks `generate_shopping_list` duration
- **Acceptance**: 95th percentile latency < 2 seconds under normal load (10 concurrent users)

**Shopping List Item Collection**:
- **Target**: < 500ms response time for marking item as collected
- **Optimization Strategies**:
  - Direct read model update (bypass event sourcing for read model-only changes)
  - Indexed query on `shopping_list_items.id` (primary key lookup)
  - Minimal HTML fragment response (single `<li>` element, ~200 bytes)
- **Measurement**: HTTP request duration logged in tracing
- **Acceptance**: 99th percentile latency < 500ms

**Notification Delivery**:
- **Target**: Notifications delivered within ±5 minutes of scheduled time
- **Optimization Strategies**:
  - Background worker polls every 60 seconds (trade-off: lower CPU vs. slight timing variance)
  - Database index on `notifications(status, scheduled_time)` for fast query
  - Batch processing: Send up to 100 notifications per poll (limit to prevent timeout)
  - Async Web Push API calls (tokio tasks for concurrent sends)
- **Measurement**: Track delta between `scheduled_time` and `sent_at` in metrics
- **Acceptance**: 95% of notifications delivered within ±5 minutes, 99% within ±10 minutes

**Multi-Week Query Performance**:
- **Target**: < 3 seconds to load 4 weeks of shopping lists with up to 200 total items
- **Optimization Strategies**:
  - Single query with IN clause for week dates
  - Join query to load items (avoid N+1 queries)
  - Denormalized `item_count` in `shopping_lists` table for quick display
- **Measurement**: Query execution time logged
- **Acceptance**: 95th percentile latency < 3 seconds

### Security

**Authorization**:
- **Shopping Lists**: All endpoints verify shopping list belongs to authenticated user (query `shopping_lists.user_id = auth_user_id`)
- **Notifications**: Users can only view/dismiss their own notifications (query `notifications.user_id = auth_user_id`)
- **Push Subscriptions**: Subscriptions scoped to authenticated user (no cross-user notification delivery)

**Input Validation**:
- **Week Dates**: Validate ISO 8601 format, must be Monday (reject if not start of week per Monday-first week convention)
- **Item IDs**: Validate UUID format, check existence in database
- **Push Subscription Data**: Validate endpoint URL format, key lengths (p256dh 65 bytes, auth 16 bytes)

**Data Privacy**:
- **Shopping List Visibility**: Private per user, no sharing in MVP (shopping lists contain ingredient data from private recipes)
- **Notification Content**: Push notification body contains recipe titles (no sensitive user data)
- **Push Endpoint URLs**: Stored securely, never exposed in API responses

**Web Push Security**:
- **VAPID Authentication**: Use application-specific VAPID keys (not reusing keys across apps)
- **Endpoint Validation**: Verify push endpoint URL is HTTPS (reject HTTP)
- **Subscription Expiry**: Clean up push subscriptions that return 410 Gone (browser unsubscribed)

### Reliability/Availability

**Event Sourcing Guarantees**:
- **Durability**: All shopping list generations and notification schedules persisted as events before acknowledgment
- **Idempotency**: Shopping list generation command idempotent (regenerating for same meal plan produces consistent result)
- **Replay**: If read model corrupted, can replay `ShoppingListGenerated` events to rebuild state

**Notification Delivery Resilience**:
- **Background Worker Restart**: Kubernetes liveness probe restarts worker if crashed
- **Retry Logic**: Web Push API calls retry up to 3 times with exponential backoff (1s, 2s, 4s)
- **Failed Notification Handling**: Notifications marked 'failed' after 3 retries, user can manually retry from UI
- **At-Least-Once Delivery**: Worker polls continuously, ensuring due notifications eventually sent (may duplicate if worker crashes mid-send)

**Database Consistency**:
- **Transaction Boundaries**: Shopping list item updates wrapped in transactions
- **Foreign Key Constraints**: Prevent orphaned shopping items (cascade delete on shopping list deletion)
- **Read Model Lag**: evento subscriptions process events asynchronously (eventual consistency acceptable, typically < 100ms lag)

### Observability

**Tracing**:
- **Shopping List Generation**: `#[tracing::instrument]` on `generate_shopping_list()` function
- **Notification Scheduling**: Span tracking from `MealPlanGenerated` event to `ReminderScheduled` event emission
- **Web Push Delivery**: Trace push notification send attempts, status codes, error messages
- **Background Worker**: Log each poll cycle (due notifications found, sent, failed)

**Metrics**:
- **Shopping List Metrics**:
  - `shopping_lists_generated_total` (counter)
  - `shopping_list_generation_duration_seconds` (histogram)
  - `shopping_list_items_per_list` (histogram)
- **Notification Metrics**:
  - `notifications_scheduled_total` (counter by type: prep_reminder, cooking_reminder)
  - `notifications_sent_total` (counter)
  - `notifications_failed_total` (counter by failure reason: no_subscription, push_api_error, timeout)
  - `notification_delivery_latency_seconds` (histogram: scheduled_time to sent_at delta)
- **Push Subscription Metrics**:
  - `push_subscriptions_active` (gauge)
  - `push_subscriptions_created_total` (counter)
  - `push_subscriptions_expired_total` (counter)

**Logging**:
- **Structured Logs**: All events logged with correlation ID (user_id, meal_plan_id, shopping_list_id, notification_id)
- **Error Logs**: Web Push API failures logged with endpoint URL (redacted), HTTP status, error message
- **Audit Trail**: All shopping list generations, item collections, and notification deliveries logged for analytics

**Alerting**:
- **Critical Alert**: Notification delivery success rate < 80% over 1 hour (indicates systemic push API issue)
- **Warning Alert**: Background worker hasn't processed notifications in 10 minutes (worker may be stuck)
- **Info Alert**: Shopping list generation latency > 5 seconds (performance degradation)

---

## Dependencies and Integrations

### External Dependencies

**Web Push API (Browser Standard)**:
- **Purpose**: Deliver push notifications to user's browsers/devices without vendor-specific services
- **Integration**: `web-push` Rust crate (version 0.10+)
- **Authentication**: VAPID (Voluntary Application Server Identification) with self-generated key pair
- **Key Generation**: Run once during setup: `scripts/generate-vapid.sh` outputs VAPID private/public keys
- **Environment Variables**:
  - `VAPID_PRIVATE_KEY`: Base64-encoded private key (32 bytes)
  - `VAPID_PUBLIC_KEY`: Base64-encoded public key (65 bytes, exposed to client JavaScript)
- **Browser Support**: Chrome 50+, Firefox 44+, Safari 16+ (iOS Safari 16.4+), Edge 79+
- **No Vendor Lock-in**: Standard Web Push protocol, works across all supporting browsers
- **Subscription Lifecycle**: Browsers generate push subscription endpoints, stored in `push_subscriptions` table
- **Error Handling**: 410 Gone response indicates subscription expired, delete from database

### Internal Dependencies

**Meal Planning Crate** (Epic 3):
- **Dependency**: Shopping crate subscribes to `MealPlanGenerated` and `MealReplaced` events
- **Data Required**:
  - Meal plan ID
  - Meal assignments (date, meal_type, recipe_id)
  - Week start date
- **Read Model Query**: Shopping crate queries `meal_plans` and `meal_assignments` tables directly
- **Contract**: `MealPlanGenerated` event must include meal_plan_id and all assignments

**Recipe Crate** (Epic 2):
- **Dependency**: Shopping crate reads recipe ingredients; Notifications crate reads advance prep hours
- **Data Required**:
  - Recipe ingredients (JSON array: `[{ name, quantity, unit }]`)
  - Recipe advance_prep_hours (integer or NULL)
  - Recipe title (for notification body text)
- **Read Model Query**: Shopping crate queries `recipes` table by recipe_id
- **Contract**: Recipe ingredients stored as valid JSON in `recipes.ingredients` field

**User Crate** (Epic 1):
- **Dependency**: Shopping and Notifications crates filter by user_id for multi-tenancy
- **Data Required**: User ID (from JWT claims), timezone (for notification scheduling - future enhancement)
- **Authorization**: All queries scoped to authenticated user
- **Contract**: User ID exists in `users` table (foreign key constraint)

### Database Migrations

**Migration 004: Shopping Lists Tables**
```sql
-- File: migrations/004_create_shopping_lists_table.sql

CREATE TABLE shopping_lists (
  id TEXT PRIMARY KEY,
  user_id TEXT NOT NULL,
  meal_plan_id TEXT NOT NULL,
  week_start_date TEXT NOT NULL,
  generated_at TEXT NOT NULL,
  item_count INTEGER NOT NULL DEFAULT 0,
  FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
  FOREIGN KEY (meal_plan_id) REFERENCES meal_plans(id) ON DELETE CASCADE
);

CREATE INDEX idx_shopping_lists_user_week ON shopping_lists(user_id, week_start_date);
CREATE INDEX idx_shopping_lists_meal_plan ON shopping_lists(meal_plan_id);

CREATE TABLE shopping_list_items (
  id TEXT PRIMARY KEY,
  shopping_list_id TEXT NOT NULL,
  ingredient_name TEXT NOT NULL,
  quantity REAL NOT NULL,
  unit TEXT NOT NULL,
  category TEXT,
  is_collected BOOLEAN DEFAULT FALSE,
  recipe_ids TEXT NOT NULL, -- JSON array
  FOREIGN KEY (shopping_list_id) REFERENCES shopping_lists(id) ON DELETE CASCADE
);

CREATE INDEX idx_shopping_items_list ON shopping_list_items(shopping_list_id);
CREATE INDEX idx_shopping_items_category ON shopping_list_items(shopping_list_id, category);
```

**Migration 006: Notifications Tables**
```sql
-- File: migrations/006_create_notifications_table.sql

CREATE TABLE notifications (
  id TEXT PRIMARY KEY,
  user_id TEXT NOT NULL,
  notification_type TEXT NOT NULL,
  recipe_id TEXT NOT NULL,
  meal_date TEXT NOT NULL,
  meal_type TEXT NOT NULL,
  scheduled_time TEXT NOT NULL,
  sent_at TEXT,
  status TEXT NOT NULL DEFAULT 'pending',
  title TEXT NOT NULL,
  body TEXT NOT NULL,
  action_url TEXT,
  FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
  FOREIGN KEY (recipe_id) REFERENCES recipes(id) ON DELETE CASCADE
);

CREATE INDEX idx_notifications_user_pending ON notifications(user_id, status, scheduled_time);
CREATE INDEX idx_notifications_due ON notifications(status, scheduled_time)
  WHERE status = 'pending' AND scheduled_time <= datetime('now');
```

**Migration 007: Push Subscriptions Table**
```sql
-- File: migrations/007_create_push_subscriptions_table.sql

CREATE TABLE push_subscriptions (
  id TEXT PRIMARY KEY,
  user_id TEXT NOT NULL,
  endpoint TEXT NOT NULL,
  p256dh_key TEXT NOT NULL,
  auth_key TEXT NOT NULL,
  created_at TEXT NOT NULL,
  last_used_at TEXT,
  FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
  UNIQUE(user_id, endpoint)
);

CREATE INDEX idx_push_subscriptions_user ON push_subscriptions(user_id);
```

### Integration Points

**Evento Event Bus**:
- **Shopping Crate Subscriptions**:
  - `MealPlanGenerated` → triggers `GenerateShoppingList` command
  - `MealReplaced` → triggers `UpdateShoppingList` command
- **Notifications Crate Subscriptions**:
  - `MealPlanGenerated` → triggers multiple `ScheduleReminder` commands (one per meal with advance prep)
  - `MealReplaced` → cancels old reminders, schedules new reminders

**Read Model Queries**:
- Shopping crate reads `meal_assignments` and `recipes` tables (cross-crate queries allowed for read models)
- Notifications crate reads `recipes` table for advance_prep_hours and titles
- Both crates write to their own read model tables only

**Background Workers**:
- Notification scheduler runs as tokio task spawned in `main.rs`
- Worker lifetime tied to application process (restarts with app)
- No separate worker process required (simplifies deployment)

---

## Acceptance Criteria (Authoritative)

### Story 1: Shopping List Generation

**Given** a user has generated a meal plan with 7 days and 14 meals (breakfast and dinner)
**When** the meal plan generation completes
**Then** a shopping list is automatically generated for the week
**And** the shopping list contains all ingredients from all 14 recipes
**And** ingredients with the same name and unit are aggregated (e.g., "chicken breast 2lbs" + "chicken breast 1lb" = "chicken breast 3lbs")
**And** the shopping list is accessible at GET /shopping
**And** shopping list generation completes within 2 seconds

**Technical Verification**:
- `ShoppingListGenerated` event exists in evento stream with meal_plan_id
- `shopping_lists` table has row with correct week_start_date
- `shopping_list_items` table has rows with aggregated quantities
- Unit test verifies aggregation logic for 3 recipes with overlapping ingredients

---

### Story 2: Category-Based Ingredient Grouping

**Given** a shopping list has been generated with 30 ingredients
**When** the user views the shopping list
**Then** ingredients are grouped by category: Produce, Dairy, Meat, Pantry, Spices, Baking, Other
**And** each category is collapsible/expandable for easy navigation
**And** at least 95% of common ingredients are assigned to correct categories

**Technical Verification**:
- Categorization algorithm assigns categories to all 30 ingredients
- Template renders category groups with headings
- Unit test verifies category assignment for 50 common ingredients (produce: onions, tomatoes; dairy: milk, cheese; etc.)
- Manual testing confirms category accuracy for edge cases

---

### Story 3: Multi-Week Shopping List Access

**Given** a user has generated meal plans for 4 consecutive weeks
**When** the user navigates to GET /shopping/multi-week
**Then** shopping lists for all 4 weeks are displayed in an accordion
**And** each week shows the week start date and total item count
**And** the user can expand any week to view categorized ingredients
**And** page loads within 3 seconds

**Technical Verification**:
- Query returns shopping lists for 4 weeks in single database query
- Template renders accordion with 4 sections
- Integration test verifies multi-week query performance
- E2E test navigates to multi-week view, asserts 4 weeks visible

---

### Story 4: Shopping List Updates on Meal Replacement

**Given** a user has an active meal plan with shopping list for the week
**When** the user replaces Thursday dinner (Chicken Tikka Masala → Quick Stir Fry)
**Then** the shopping list is automatically updated within 3 seconds
**And** ingredients unique to Chicken Tikka Masala are removed (yogurt, garam masala)
**And** ingredients unique to Quick Stir Fry are added (soy sauce, bell peppers)
**And** shared ingredients remain with adjusted quantities (chicken breast 2lbs → 1lb)

**Technical Verification**:
- `MealReplaced` event triggers `UpdateShoppingList` command
- `ShoppingListUpdated` event written to evento stream
- Read model reflects new ingredient list within 3 seconds
- Integration test verifies meal replacement triggers shopping list update

---

### Story 5: Shopping List Item Completion Tracking

**Given** a user is viewing a shopping list with 20 items
**When** the user taps the checkbox next to "chicken breast 2lbs"
**Then** the item is marked as collected with a strike-through
**And** the checkbox is checked
**And** the UI updates within 500ms without full page reload
**And** the collected state persists if the user refreshes the page

**Technical Verification**:
- POST to /shopping/:list_id/item/:item_id/collect returns HTML fragment
- TwinSpark swaps DOM element with updated state
- `ItemCollected` event written to evento stream
- Read model `shopping_list_items.is_collected` set to TRUE
- E2E test: Click checkbox, assert strike-through class applied

---

### Story 6: Push Notification Subscription

**Given** a user is logged in and has enabled browser notifications
**When** the user navigates to GET /notifications
**And** clicks "Enable Prep Reminders" button
**Then** browser requests notification permission
**And** if granted, browser generates push subscription
**And** subscription is sent to POST /notifications/subscribe
**And** subscription is stored in push_subscriptions table
**And** user sees success message: "Successfully subscribed to notifications!"

**Technical Verification**:
- JavaScript calls browser PushManager.subscribe() API
- POST request includes endpoint, p256dh_key, auth_key
- `PushSubscriptionCreated` event written to evento stream
- Read model `push_subscriptions` table has row with user_id and endpoint
- E2E test: Click button, mock browser API, verify subscription stored

---

### Story 7: Advance Preparation Reminder Scheduling

**Given** a user generates meal plan with Thursday dinner requiring 24-hour marinade (Chicken Tikka Masala)
**When** the meal plan is generated
**Then** a prep reminder is scheduled for Wednesday 9am (24 hours before Thursday 6pm dinner)
**And** the reminder is stored in notifications table with status='pending'
**And** the notification body says "Marinate chicken tonight for Thursday dinner: Chicken Tikka Masala"

**Technical Verification**:
- `MealPlanGenerated` event triggers `ScheduleReminder` command
- `ReminderScheduled` event written to evento stream
- Read model `notifications` table has row with correct scheduled_time
- Unit test verifies `calculate_reminder_time()` function: meal Thursday 6pm, 24h prep → Wednesday 9am
- Integration test verifies notification created with correct body text

---

### Story 8: Background Notification Delivery

**Given** a prep reminder is scheduled for 9am and current time is 9:02am
**And** the user has an active push subscription
**When** the background worker polls for due notifications
**Then** the reminder is sent via Web Push API within 5 minutes
**And** the notification appears on the user's device with title, body, and action buttons
**And** the notification status is updated to 'sent' in the database
**And** the sent_at timestamp is recorded

**Technical Verification**:
- Background worker queries notifications with scheduled_time <= now AND status='pending'
- Worker invokes Web Push API with VAPID signature
- `ReminderSent` event written to evento stream
- Read model `notifications.status` updated to 'sent'
- Integration test: Mock Web Push API, verify event emitted and status updated
- Manual test: Real browser receives notification

---

### Story 9: Notification Delivery Resilience

**Given** a notification is scheduled for 9am
**And** the Web Push API returns 500 Internal Server Error on first attempt
**When** the background worker retries the notification
**Then** the notification is retried up to 3 times with exponential backoff (1s, 2s, 4s)
**And** if all retries fail, status is set to 'failed'
**And** the error is logged with notification ID and error message

**Technical Verification**:
- Worker retry logic implemented with exponential backoff
- After 3 failures, notification marked as 'failed'
- Tracing logs contain error details
- Unit test: Mock failing Web Push API, verify retry attempts
- Integration test: Verify failed notification status after retries

---

### Story 10: Prep Task Completion Tracking

**Given** a user has received a prep reminder notification
**When** the user opens the notification and clicks "View Recipe"
**Then** the recipe detail page opens
**And** a prep task checklist is displayed: "Marinate chicken (4 hours)"
**And** the user can check off the task
**And** the task completion is saved to the notifications table

**Technical Verification**:
- Notification action button has URL: /recipes/:id?notification_id=:notif_id
- Recipe detail page queries notification by ID
- UI renders prep task with checkbox
- POST /notifications/:id/dismiss marks task complete
- `ReminderDismissed` event written to evento stream
- Read model `notifications.status` updated to 'dismissed'
- E2E test: Open notification, click action, check task, verify status updated

---

## Traceability Mapping

### PRD to Technical Implementation Mapping

**FR-8: Shopping List Generation**
- **Implementation**: `shopping` crate with `ShoppingListAggregate`
- **Commands**: `GenerateShoppingList`
- **Events**: `ShoppingListGenerated`
- **Tables**: `shopping_lists`, `shopping_list_items`
- **HTTP Endpoint**: GET /shopping (displays current week)
- **Acceptance Criteria**: Story 1

**FR-8: Ingredient Aggregation**
- **Implementation**: `aggregation.rs` module with `aggregate_ingredients()` function
- **Algorithm**: Normalize names/units, group by key, sum quantities
- **Testing**: Unit tests for aggregation logic with various ingredient combinations
- **Acceptance Criteria**: Story 1, Story 4

**FR-8: Category Grouping**
- **Implementation**: `categorization.rs` module with `assign_category()` function
- **Categories**: produce, dairy, meat, pantry, spices, baking, other
- **Algorithm**: Keyword matching against predefined lists
- **Acceptance Criteria**: Story 2

**FR-9: Multi-Week Shopping List Access**
- **Implementation**: `get_multi_week_shopping_lists()` query function
- **HTTP Endpoint**: GET /shopping/multi-week?start_week=DATE&num_weeks=N
- **Template**: `templates/pages/shopping-multi-week.html` with accordion UI
- **Acceptance Criteria**: Story 3

**FR-9: Shopping List Updates**
- **Implementation**: evento subscription on `MealReplaced` event triggers `UpdateShoppingList` command
- **Events**: `ShoppingListUpdated`
- **Workflow**: Regenerate shopping list from updated meal assignments
- **Acceptance Criteria**: Story 4

**FR-10: Advance Preparation Reminders**
- **Implementation**: `notifications` crate with `NotificationAggregate`
- **Commands**: `ScheduleReminder`, `SendReminder`
- **Events**: `ReminderScheduled`, `ReminderSent`
- **Background Worker**: `NotificationScheduler` tokio task
- **Acceptance Criteria**: Story 7, Story 8

**FR-10: Push Notification Integration**
- **Implementation**: `push.rs` module with `send_web_push()` function
- **Library**: `web-push` crate (0.10+)
- **Authentication**: VAPID keys (generated via `scripts/generate-vapid.sh`)
- **HTTP Endpoint**: POST /notifications/subscribe (stores browser push endpoint)
- **Tables**: `push_subscriptions`
- **Acceptance Criteria**: Story 6

**FR-10: Prep Task Tracking**
- **Implementation**: `DismissNotification` command marks notification as completed
- **Events**: `ReminderDismissed`
- **HTTP Endpoint**: POST /notifications/:id/dismiss
- **UI**: Checkbox in recipe detail page for pending prep tasks
- **Acceptance Criteria**: Story 10

### Epic Dependency Mapping

**Epic 2 (Recipe Management) → Epic 4**:
- Epic 4 reads `recipes.ingredients` (JSON array)
- Epic 4 reads `recipes.advance_prep_hours` for notification scheduling
- Epic 4 reads `recipes.title` for notification body text

**Epic 3 (Meal Planning) → Epic 4**:
- Epic 4 subscribes to `MealPlanGenerated` event
- Epic 4 subscribes to `MealReplaced` event
- Epic 4 reads `meal_plans` and `meal_assignments` tables

**Epic 1 (User Management) → Epic 4**:
- Epic 4 enforces user-scoped queries (multi-tenancy)
- Epic 4 stores push subscriptions per user_id
- Epic 4 delivers notifications to authenticated users only

### Story Dependencies (Implementation Order)

**Phase 1: Shopping Foundation**
1. Story 1: Shopping List Generation (core functionality)
2. Story 2: Category-Based Grouping (enhances Story 1)
3. Story 5: Item Completion Tracking (interactive feature)

**Phase 2: Multi-Week & Updates**
4. Story 3: Multi-Week Access (extends Story 1)
5. Story 4: Shopping List Updates (event-driven update)

**Phase 3: Notifications Infrastructure**
6. Story 6: Push Subscription (enables notifications)
7. Story 7: Reminder Scheduling (core notification logic)
8. Story 8: Background Delivery (sends scheduled notifications)

**Phase 4: Resilience & Tracking**
9. Story 9: Delivery Resilience (error handling for Story 8)
10. Story 10: Task Completion Tracking (user interaction with notifications)

**Critical Path**: Story 1 → Story 6 → Story 7 → Story 8 (end-to-end value delivery)

---

## Risks, Assumptions, Open Questions

### Risks

**Risk 1: Ingredient Aggregation Accuracy**
- **Description**: Aggregating ingredients with different units (cups vs liters, lbs vs grams) may produce incorrect quantities if unit conversion not implemented
- **Impact**: High - Users receive incorrect shopping lists, leading to under/over-purchasing ingredients
- **Mitigation**:
  - MVP: Keep ingredients with different units separate (do not aggregate across units)
  - Phase 2: Implement unit conversion library (cups to ml, lbs to grams, etc.)
  - Validation: Unit tests verify non-aggregation of different units
- **Owner**: Shopping crate developer
- **Status**: Mitigated (MVP uses separate items)

**Risk 2: Category Assignment Accuracy**
- **Description**: Keyword-based category assignment may misclassify ingredients (e.g., "coconut milk" → dairy instead of pantry)
- **Impact**: Medium - Reduced shopping efficiency, users need to scan multiple categories
- **Mitigation**:
  - Comprehensive keyword lists covering 95% of common ingredients
  - "Other" category for unrecognized ingredients (users can still shop, just less organized)
  - Phase 2: Machine learning classification or manual overrides
- **Owner**: Shopping crate developer
- **Status**: Accepted (95% target in acceptance criteria)

**Risk 3: Web Push Browser Compatibility**
- **Description**: iOS Safari only supports Web Push API starting version 16.4 (March 2023), older iOS users cannot receive notifications
- **Impact**: Medium - Reduces notification reach for iOS users on older devices
- **Mitigation**:
  - Feature detection in JavaScript (hide notification button if unsupported)
  - Fallback: Email reminders (future enhancement)
  - Target audience: Assuming 85% of users on supported browsers (per Can I Use data)
- **Owner**: Notifications crate developer
- **Status**: Accepted (documented limitation)

**Risk 4: Background Worker Reliability**
- **Description**: If background worker crashes or gets stuck, notifications won't be delivered until worker restarts
- **Impact**: High - Missed prep reminders lead to failed recipe execution (core value proposition)
- **Mitigation**:
  - Kubernetes liveness probe restarts worker if unhealthy (30-second check interval)
  - At-least-once delivery: Poll continuously, due notifications eventually sent
  - Monitoring: Alert if worker hasn't processed notifications in 10 minutes
  - Retry logic: Failed notifications retried on next poll cycle
- **Owner**: Notifications crate developer + DevOps
- **Status**: Mitigated (robust restart and retry mechanisms)

**Risk 5: Web Push API Rate Limits**
- **Description**: Push services (Google FCM, Mozilla Autopush) may rate limit requests if too many notifications sent quickly
- **Impact**: Low - Only affects users with many meal plans or during bulk operations
- **Mitigation**:
  - Batch processing: Send up to 100 notifications per poll (spread load)
  - Respect retry-after headers from push services
  - Exponential backoff on errors
- **Owner**: Notifications crate developer
- **Status**: Accepted (unlikely to hit limits at MVP scale)

### Assumptions

**Assumption 1: Ingredient Data Quality**
- **Assumption**: Users enter recipe ingredients in structured format (quantity, unit, name) via recipe creation form
- **Validation**: Recipe crate enforces ingredient schema (Epic 2)
- **Risk if False**: Malformed ingredient data breaks aggregation algorithm
- **Mitigation**: Input validation in recipe form, JSON schema validation

**Assumption 2: Meal Times Default**
- **Assumption**: Meal times default to standard times (breakfast 8am, lunch 12pm, dinner 6pm) for notification scheduling
- **Validation**: Hardcoded in `calculate_reminder_time()` function
- **Risk if False**: Notifications sent at suboptimal times for users with different schedules
- **Future Enhancement**: User-configurable meal times (Epic 3 extension)

**Assumption 3: Single Active Meal Plan**
- **Assumption**: Users have one active meal plan per week (no overlapping plans)
- **Validation**: Meal planning crate enforces single active plan (Epic 3)
- **Risk if False**: Shopping list generation ambiguous if multiple plans for same week
- **Mitigation**: Query most recent meal plan if duplicates found

**Assumption 4: User Timezone**
- **Assumption**: All users in same timezone as server (UTC) for MVP
- **Validation**: No timezone field in user profile
- **Risk if False**: Notifications sent at wrong local time for international users
- **Future Enhancement**: Store user timezone in profile, convert scheduled_time to UTC

**Assumption 5: Web Push Subscription Persistence**
- **Assumption**: Browser push subscriptions remain valid indefinitely unless user unsubscribes
- **Validation**: Monitor 410 Gone responses from push services
- **Risk if False**: Notifications fail silently if subscription expired
- **Mitigation**: Clean up expired subscriptions (410 Gone → delete from database)

**Assumption 6: Shopping List Per Week**
- **Assumption**: One shopping list per week (Monday-Sunday), no mid-week shopping
- **Validation**: Shopping list keyed by week_start_date
- **Risk if False**: Users shop multiple times per week, need partial lists
- **Future Enhancement**: Daily shopping lists or dynamic date ranges

### Open Questions

**Question 1: Unit Conversion Precision**
- **Question**: Should unit conversion be precise (1 cup = 236.588 ml) or approximate (1 cup ≈ 240 ml)?
- **Context**: Precise conversion creates decimal quantities (2.36588 liters), approximate is cleaner (2.4 liters)
- **Decision Needed**: Product decision on precision vs. usability trade-off
- **Stakeholder**: Product owner
- **Timeline**: Before Phase 2 implementation (unit conversion feature)

**Question 2: Notification Timing Customization**
- **Question**: Should users customize notification timing (e.g., prefer 7am instead of 9am)?
- **Context**: MVP uses fixed 9am timing for morning reminders
- **Trade-off**: Flexibility vs. complexity (settings UI, per-user scheduling logic)
- **Decision Needed**: Post-MVP based on user feedback (do users complain about timing?)
- **Stakeholder**: Product owner
- **Timeline**: After MVP launch, review in 3-month retrospective

**Question 3: Shared Shopping Lists**
- **Question**: Should family members share shopping lists (one user generates plan, other user collects items)?
- **Context**: MVP has per-user shopping lists, no sharing
- **Trade-off**: Collaboration feature vs. multi-user access control complexity
- **Decision Needed**: Product decision on target persona (individual vs. family cooking)
- **Stakeholder**: Product owner
- **Timeline**: Future epic (Family Management)

**Question 4: Offline Shopping List Access**
- **Question**: Should shopping list be fully cached in service worker for offline access in grocery store?
- **Context**: PWA architecture supports offline, but shopping list may be stale if meal plan changes
- **Trade-off**: Offline convenience vs. data freshness
- **Decision Needed**: Technical decision on cache strategy (cache-first vs. network-first)
- **Stakeholder**: Tech lead
- **Timeline**: Epic 5 (PWA implementation)

**Question 5: Recipe Ingredient Deduplication Logic**
- **Question**: How to handle duplicate ingredients within a single recipe (e.g., "onions" in steps 1 and 3)?
- **Context**: Current logic aggregates across recipes, but not within a single recipe
- **Trade-off**: Data quality (clean recipe entry) vs. resilience (handle messy data)
- **Decision Needed**: Technical decision on preprocessing during recipe creation vs. runtime deduplication
- **Stakeholder**: Tech lead + Recipe crate developer
- **Timeline**: Before Story 1 implementation

---

## Test Strategy Summary

### Unit Tests (Domain Logic)

**Shopping Crate Tests**:
- `aggregation.rs`:
  - Test: Aggregate ingredients with same name/unit (sum quantities)
  - Test: Keep separate ingredients with different units (no conversion in MVP)
  - Test: Normalize ingredient names (uppercase, plural, whitespace)
  - Test: Normalize units (cups/cup/c → cups, lbs/lb/pound → lbs)
  - Test: Handle edge cases (empty ingredient list, single ingredient, 50+ ingredients)
- `categorization.rs`:
  - Test: Assign categories to 50 common ingredients (verify 95% accuracy)
  - Test: Fallback to "other" for unrecognized ingredients
  - Test: Case-insensitive matching (Chicken vs. chicken)

**Notifications Crate Tests**:
- `scheduler.rs`:
  - Test: Calculate reminder time for various advance prep hours (4h, 24h, 48h)
  - Test: Morning reminder scheduled for 9am on correct day
  - Test: Handle edge case: meal on Monday, 48h prep → reminder on Saturday 9am
- `push.rs`:
  - Test: Generate VAPID signature with valid keys
  - Test: Build Web Push payload with title, body, action buttons
  - Test: Handle push API errors (timeout, 410 Gone, 500 error)

**Aggregate Tests**:
- `ShoppingListAggregate`:
  - Test: GenerateShoppingList command creates aggregate with items
  - Test: MarkItemCollected command updates item state
  - Test: Event sourcing: Replay events to rebuild aggregate state
- `NotificationAggregate`:
  - Test: ScheduleReminder command creates notification with pending status
  - Test: SendReminder command updates status to sent
  - Test: DismissNotification command updates status to dismissed

### Integration Tests (HTTP Endpoints)

**Shopping Endpoints**:
- Test: POST /plan/generate → Shopping list auto-generated (verify database row)
- Test: GET /shopping → Returns HTML with categorized items
- Test: POST /shopping/:list_id/item/:item_id/collect → Updates item, returns HTML fragment
- Test: GET /shopping/multi-week → Returns 4 weeks of shopping lists

**Notification Endpoints**:
- Test: POST /notifications/subscribe → Stores push subscription in database
- Test: GET /notifications → Returns notification history
- Test: POST /notifications/:id/dismiss → Updates notification status

**Background Worker Tests**:
- Test: Worker polls database for due notifications
- Test: Worker sends push notifications via mocked Web Push API
- Test: Worker retries on failure, updates status to 'failed' after 3 attempts
- Test: Worker handles empty results (no due notifications)

### E2E Tests (Playwright)

**Shopping Flow E2E**:
- Test: User generates meal plan → Shopping list appears at /shopping
- Test: User views shopping list → Sees categories (Produce, Dairy, Meat)
- Test: User checks off item → Item strike-through, checkbox checked, persists on refresh
- Test: User navigates to multi-week view → Sees 4 weeks in accordion

**Notification Flow E2E**:
- Test: User enables notifications → Browser permission prompt → Subscription stored
- Test: User generates meal plan with advance prep recipe → Notification scheduled (verify database)
- Test: Background worker runs (trigger manually) → Notification sent (mock push API)
- Test: User opens notification → Recipe page opens with prep task checklist

**Error Handling E2E**:
- Test: User without push subscription → Notification not sent, error logged
- Test: Web Push API failure → Notification retried, eventually marked failed
- Test: User dismisses notification → Status updated, no longer shown in pending list

### Performance Tests

**Load Test 1: Shopping List Generation**
- **Scenario**: 10 concurrent users generate meal plans (21 meals each)
- **Measurement**: Latency to complete shopping list generation
- **Acceptance**: 95th percentile < 2 seconds

**Load Test 2: Item Collection**
- **Scenario**: 100 concurrent users mark items as collected
- **Measurement**: HTTP response time
- **Acceptance**: 99th percentile < 500ms

**Load Test 3: Notification Delivery**
- **Scenario**: 500 notifications scheduled for same time
- **Measurement**: Time to deliver all notifications
- **Acceptance**: All notifications sent within 10 minutes

### Manual Test Cases

**Manual Test 1: Category Accuracy Validation**
- **Procedure**: Generate shopping list with 30 diverse ingredients
- **Verification**: Manually verify category assignments match expectations
- **Acceptance**: 28+ out of 30 correct (93%+ accuracy)

**Manual Test 2: Real Browser Push Notification**
- **Procedure**: Subscribe to notifications in Chrome/Firefox/Safari
- **Action**: Manually trigger notification from database (update scheduled_time to past)
- **Verification**: Notification appears on device with correct title, body, action buttons
- **Acceptance**: Notification received within 2 minutes

**Manual Test 3: Offline Shopping List Access**
- **Procedure**: Load shopping list, enable airplane mode, refresh page
- **Verification**: Service worker serves cached shopping list
- **Acceptance**: Page loads offline (Epic 5 dependency)

### Test Coverage Goals

- **Unit Tests**: 90% code coverage for `shopping` and `notifications` crates (domain logic must be thoroughly tested)
- **Integration Tests**: All HTTP endpoints covered with happy path and error cases
- **E2E Tests**: Critical user flows (generate shopping list, mark items, enable notifications, receive notification)
- **Performance Tests**: Baseline measurements for all performance acceptance criteria

---

## Appendix: Event Schema Reference

### Shopping Domain Events

**Event: ShoppingListGenerated**
```rust
{
    "event_type": "ShoppingListGenerated",
    "aggregator_id": "shopping-list-uuid",
    "aggregator_type": "ShoppingListAggregate",
    "version": 1,
    "timestamp": "2025-10-14T12:34:56Z",
    "metadata": {
        "user_id": "user-uuid",
        "correlation_id": "meal-plan-uuid"
    },
    "data": {
        "user_id": "user-uuid",
        "meal_plan_id": "meal-plan-uuid",
        "week_start_date": "2025-10-14",
        "items": [
            {
                "ingredient_name": "chicken breast",
                "quantity": 3.0,
                "unit": "lbs",
                "category": "meat",
                "recipe_ids": ["recipe-1", "recipe-2"]
            },
            {
                "ingredient_name": "onions",
                "quantity": 5.0,
                "unit": "whole",
                "category": "produce",
                "recipe_ids": ["recipe-1", "recipe-3", "recipe-4"]
            }
        ],
        "item_count": 2
    }
}
```

**Event: ItemCollected**
```rust
{
    "event_type": "ItemCollected",
    "aggregator_id": "shopping-list-uuid",
    "aggregator_type": "ShoppingListAggregate",
    "version": 2,
    "timestamp": "2025-10-14T15:23:11Z",
    "metadata": {
        "user_id": "user-uuid"
    },
    "data": {
        "item_id": "item-uuid",
        "is_collected": true,
        "collected_at": "2025-10-14T15:23:11Z"
    }
}
```

### Notification Domain Events

**Event: ReminderScheduled**
```rust
{
    "event_type": "ReminderScheduled",
    "aggregator_id": "notification-uuid",
    "aggregator_type": "NotificationAggregate",
    "version": 1,
    "timestamp": "2025-10-14T12:34:56Z",
    "metadata": {
        "user_id": "user-uuid",
        "correlation_id": "meal-plan-uuid"
    },
    "data": {
        "user_id": "user-uuid",
        "notification_type": "prep_reminder",
        "recipe_id": "recipe-uuid",
        "meal_date": "2025-10-16",
        "meal_type": "dinner",
        "scheduled_time": "2025-10-15T09:00:00Z",
        "title": "Prep Reminder",
        "body": "Marinate chicken tonight for Thursday dinner: Chicken Tikka Masala",
        "action_url": "/recipes/recipe-uuid"
    }
}
```

**Event: ReminderSent**
```rust
{
    "event_type": "ReminderSent",
    "aggregator_id": "notification-uuid",
    "aggregator_type": "NotificationAggregate",
    "version": 2,
    "timestamp": "2025-10-15T09:02:34Z",
    "metadata": {
        "user_id": "user-uuid"
    },
    "data": {
        "sent_at": "2025-10-15T09:02:34Z"
    }
}
```

**Event: PushSubscriptionCreated**
```rust
{
    "event_type": "PushSubscriptionCreated",
    "aggregator_id": "subscription-uuid",
    "aggregator_type": "PushSubscriptionAggregate",
    "version": 1,
    "timestamp": "2025-10-14T10:15:22Z",
    "metadata": {
        "user_id": "user-uuid"
    },
    "data": {
        "user_id": "user-uuid",
        "endpoint": "https://fcm.googleapis.com/fcm/send/...",
        "p256dh_key": "base64-encoded-key...",
        "auth_key": "base64-encoded-key...",
        "created_at": "2025-10-14T10:15:22Z"
    }
}
```

---

**Document Status**: Draft - Ready for Review
**Next Steps**:
1. Review with team for technical feasibility
2. Validate aggregation algorithm with sample data
3. Confirm Web Push API integration approach
4. Approve and begin Story 1 implementation

**Prepared by**: Jonathan (Architecture Agent)
**Review Date**: 2025-10-11
**Epic Owner**: TBD
**Estimated Effort**: 6-8 stories, 3-4 weeks (2 developers)

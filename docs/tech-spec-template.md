# Technical Specification: [Epic/Feature Name]

**Project:** imkitchen
**Epic:** [Epic ID] - [Epic Name]
**Date:** [YYYY-MM-DD]
**Author:** [Your Name]
**Status:** [Draft | Review | Approved | Implemented]
**Version:** 1.0

---

## Table of Contents

1. [Overview](#1-overview)
2. [Architecture & Design](#2-architecture--design)
3. [Data Models](#3-data-models)
4. [API Endpoints](#4-api-endpoints)
5. [Business Logic](#5-business-logic)
6. [UI/UX Specifications](#6-uiux-specifications)
7. [Testing Strategy](#7-testing-strategy)
8. [Security & Performance](#8-security--performance)
9. [Implementation Plan](#9-implementation-plan)
10. [Open Questions](#10-open-questions)

---

## 1. Overview

### 1.1 Purpose

**Problem Statement:** [Describe the problem this epic solves]

**User Story:**
> As a [user persona], I want to [action] so that [benefit].

**Example:**
> As a home cooking enthusiast, I want to generate automated weekly meal plans so that I can reduce meal planning time and increase recipe variety without mental overhead.

### 1.2 Success Criteria

**Functional Requirements:**
- [ ] Requirement 1 (e.g., User can generate meal plan in <10 seconds)
- [ ] Requirement 2 (e.g., Meal plan respects dietary restrictions)
- [ ] Requirement 3 (e.g., Algorithm assigns recipes to appropriate days)

**Non-Functional Requirements:**
- [ ] Performance: [e.g., Response time <500ms for meal plan generation]
- [ ] Scalability: [e.g., Support 10K concurrent users]
- [ ] Reliability: [e.g., 99.9% uptime]
- [ ] Security: [e.g., User data encrypted at rest]
- [ ] Accessibility: [e.g., WCAG 2.1 AA compliant]

### 1.3 Out of Scope

**Explicitly Not Included:**
- [Feature X] - Deferred to future epic
- [Feature Y] - Technical limitation
- [Feature Z] - Product decision

---

## 2. Architecture & Design

### 2.1 System Architecture

**Architecture Pattern:** [e.g., Event-Sourced CQRS with Page-Specific Read Models]

**Component Diagram:**
```
┌─────────────────────────────────────────────────────────┐
│                  HTTP Layer (Axum)                      │
│  ┌────────────┐  ┌────────────┐  ┌────────────┐       │
│  │ Route 1    │  │ Route 2    │  │ Route 3    │       │
│  └─────┬──────┘  └─────┬──────┘  └─────┬──────┘       │
└────────┼────────────────┼────────────────┼──────────────┘
         │                │                │
┌────────▼────────────────▼────────────────▼──────────────┐
│              Domain Crates (Business Logic)              │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  │
│  │ Aggregate A  │  │ Aggregate B  │  │ Aggregate C  │  │
│  │ Commands     │  │ Commands     │  │ Commands     │  │
│  │ Events       │  │ Events       │  │ Events       │  │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘  │
└─────────┼──────────────────┼──────────────────┼─────────┘
          │                  │                  │
┌─────────▼──────────────────▼──────────────────▼─────────┐
│                 evento (Event Store)                     │
│  ┌──────────────┐      ┌──────────────┐                │
│  │ Event Stream │ ───> │ Subscriptions│                │
│  │   (SQLite)   │      │(Projections) │                │
│  └──────────────┘      └──────┬───────┘                │
└─────────────────────────────────┼──────────────────────┘
                                  │
┌─────────────────────────────────▼──────────────────────┐
│           Page-Specific Read Models (SQLite)            │
│  ┌────────────┐  ┌────────────┐  ┌────────────┐       │
│  │ Page 1 RM  │  │ Page 2 RM  │  │ Page 3 RM  │       │
│  └────────────┘  └────────────┘  └────────────┘       │
└────────────────────────────────────────────────────────┘
```

### 2.2 Domain Model

**Aggregates:**
- **[Aggregate Name]**: [Brief description of aggregate responsibility]
  - Root Entity: [Entity name]
  - Value Objects: [List value objects]
  - Business Rules: [Key invariants this aggregate enforces]

**Example:**
- **Recipe Aggregate**: Manages recipe lifecycle, favoriting, and sharing
  - Root Entity: Recipe
  - Value Objects: Ingredient, Instruction, RecipeType
  - Business Rules:
    - Free tier limited to 10 recipes
    - Recipe must have at least 1 ingredient and 1 instruction
    - Recipe type required for meal planning (appetizer/main_course/dessert)

### 2.3 Event Flow

**Domain Events:**

```rust
// Event 1: [EventName]
#[derive(evento::AggregatorName, bincode::Encode, bincode::Decode)]
struct EventName {
    field1: Type,
    field2: Type,
    // ...
}

// Emitted when: [Condition]
// Triggers projections: [List affected read models]
```

**Event Sequence Diagram:**
```
User → Handler → Aggregate → evento → Projections → Read Models
  │       │          │          │          │            │
  │──(1)──┤          │          │          │            │
  │       │──(2)─────┤          │          │            │
  │       │          │──(3)─────┤          │            │
  │       │          │          │──(4)─────┤            │
  │       │          │          │          │──(5)───────┤
  │       │◄─────────────────────────────────────────(6)│
  │◄──────┤ (Redirect/Response)                         │

Legend:
(1) HTTP Request (POST/PUT/DELETE)
(2) Invoke domain command
(3) Emit event to event stream
(4) Trigger subscriptions
(5) Update page-specific read models
(6) Query read model for response
```

---

## 3. Data Models

### 3.1 Aggregate State

**Aggregate:** [Name]

```rust
#[derive(Default, Serialize, Deserialize, bincode::Encode, bincode::Decode, Clone, Debug)]
pub struct [AggregateName] {
    // State fields (rebuilt from events)
    pub id: String,
    pub field1: Type,
    pub field2: Type,
    // ...
}

// Business logic methods (NOT in handlers)
impl [AggregateName] {
    pub fn validate_command(&self, cmd: &CommandType) -> Result<(), DomainError> {
        // Business rules validation
    }
}
```

**Event Handlers:**

```rust
#[evento::aggregator]
impl [AggregateName] {
    async fn event_name_handler(
        &mut self,
        event: EventDetails<EventName>,
    ) -> anyhow::Result<()> {
        // Update aggregate state from event
        self.field1 = event.data.field1;
        Ok(())
    }
}
```

### 3.2 Page-Specific Read Models

**Purpose:** Each page can have **one or more** dedicated read model tables, each serving a specific concern on that page.

**Guidelines for Multiple Read Models:**
- Create separate tables when concerns differ (content vs filters vs metrics)
- Different update frequencies justify separate tables
- Performance optimization (avoid complex joins/aggregations)

**Example:** Recipe Library Page
- `recipe_list` - Recipe cards for display
- `recipe_filter_counts` - Filter facet counts ("Simple: 12, Favorite: 7")
- Both serve same page, different concerns

#### Read Model 1: [Page Name] - [Concern] Read Model

**Table Name:** `[table_name]`

**Purpose:** Displays [what data] on [which page]

**Schema:**
```sql
CREATE TABLE [table_name] (
  id TEXT PRIMARY KEY,
  user_id TEXT NOT NULL,
  field1 TYPE NOT NULL,
  field2 TYPE,
  -- Include ONLY fields needed for this page
  created_at TEXT NOT NULL,
  FOREIGN KEY (user_id) REFERENCES users(id)
);

CREATE INDEX idx_[table_name]_user ON [table_name](user_id);
CREATE INDEX idx_[table_name]_[frequent_query_field] ON [table_name]([field]);
```

**Projection Handler:**
```rust
#[evento::handler([Aggregate])]
pub async fn project_to_[page]_read_model<E: evento::Executor>(
    context: &evento::Context<'_, E>,
    event: EventDetails<EventName>,
) -> anyhow::Result<()> {
    sqlx::query!(
        "INSERT INTO [table_name] (id, user_id, field1, field2, created_at)
         VALUES (?, ?, ?, ?, ?)",
        event.aggregator_id,
        event.metadata.user_id,
        event.data.field1,
        event.data.field2,
        chrono::Utc::now().to_rfc3339()
    )
    .execute(context.executor.pool())
    .await?;

    Ok(())
}
```

**Query Function:**
```rust
pub async fn get_[page]_data(
    pool: &SqlitePool,
    user_id: &str,
) -> Result<Vec<[ReadModelStruct]>, Error> {
    sqlx::query_as!(
        [ReadModelStruct],
        "SELECT field1, field2, field3 FROM [table_name] WHERE user_id = ?",
        user_id
    )
    .fetch_all(pool)
    .await
}
```

#### Read Model Mapping Table

**Note:** Pages can have multiple read models (separate tables for different concerns)

| Page/Route | Read Model Tables | Data Included | Updated By Events |
|------------|------------------|---------------|-------------------|
| /[route1]  | [table1]<br>[table1_filters] | Content<br>Filter counts | [Event1, Event2]<br>[Event1, Event3] |
| /[route2]  | [table2]         | [fields]      | [Event3]          |
| /[route3]  | [table3]<br>[table3_stats] | Content<br>Statistics | [Event1, Event4]<br>[Event1, Event5] |

**Example - Recipe Library:**
- `recipe_list` → Recipe cards (title, image, complexity)
- `recipe_filter_counts` → Filter facets ("Simple: 12, Moderate: 8")

### 3.3 Form Data Consistency

**When to Use `evento::load`:**

Forms requiring pre-population with current authoritative state MUST use `evento::load` to fetch the aggregate directly, NOT read models.

**Example - Edit Form:**
```rust
pub async fn edit_[entity]_form_handler(
    auth: Auth,
    Path(entity_id): Path<String>,
    State(executor): State<evento::Executor>,
) -> Result<impl IntoResponse, AppError> {
    // Load aggregate for TRUSTED form data
    let entity = evento::load::<[Aggregate]>(&entity_id)
        .run(&executor)
        .await?;

    // Verify ownership
    if entity.user_id != auth.user_id {
        return Err(AppError::Forbidden);
    }

    // Pass aggregate state to template (authoritative source)
    Ok(HtmlResponse(Edit[Entity]Template { entity }))
}
```

**Rationale:**
- Prevents race conditions (form shows stale data from read model)
- Guarantees consistency between display and validation
- Acceptable latency for edit forms (not read-heavy list pages)

---

## 4. API Endpoints

### 4.1 Route Definitions

**Route:** `[METHOD] /[path]`

**Purpose:** [What this endpoint does]

**Authentication:** [Required | Optional | Public]

**Request:**
```rust
// Form/JSON structure
#[derive(Deserialize, Validate)]
struct [RequestType] {
    #[validate(length(min = 3, max = 200))]
    field1: String,

    #[validate(range(min = 1))]
    field2: u32,
}
```

**Response:**
- **Success (200/303):** [Description]
- **Validation Error (422):** Re-render form with inline errors
- **Auth Error (401):** Redirect to /login
- **Not Found (404):** Error page

**Handler Implementation:**
```rust
pub async fn [handler_name](
    auth: Auth,
    Form(form): Form<[RequestType]>,
    State(executor): State<evento::Executor>,
) -> Result<impl IntoResponse, AppError> {
    // 1. Validate form structure
    form.validate()?;

    // 2. Invoke domain command (business logic in aggregate)
    let entity_id = evento::create::<[Aggregate]>()
        .data(&[Event] { /* ... */ })?
        .metadata(&auth.user_id)?
        .commit(&executor)
        .await?;

    // 3. Redirect (PRG pattern)
    Ok(Redirect::to(&format!("/[path]/{}", entity_id)))
}
```

### 4.2 Endpoint Table

| Method | Route | Handler | Query Read Model | Command Aggregate |
|--------|-------|---------|------------------|-------------------|
| GET    | /[path] | [handler] | [table_name] | N/A |
| POST   | /[path] | [handler] | N/A | [Aggregate] |
| GET    | /[path]/:id | [handler] | [table_name] | N/A |
| PUT    | /[path]/:id | [handler] | N/A | [Aggregate] |
| DELETE | /[path]/:id | [handler] | N/A | [Aggregate] |

---

## 5. Business Logic

### 5.1 Domain Rules

**Business Rule 1:** [Rule description]
- **Enforced by:** [Aggregate name]
- **Validation:** [How it's validated]
- **Error:** [Error type if violated]

**Example:**
- **Rule:** Free tier users limited to 10 recipes
- **Enforced by:** User aggregate
- **Validation:** Count recipes before allowing creation
- **Error:** `UserError::RecipeLimitReached`

### 5.2 Command Handlers

**Command:** [CommandName]

**Purpose:** [What this command accomplishes]

**Input:**
```rust
pub struct [CommandName] {
    pub field1: Type,
    pub field2: Type,
}
```

**Validation:**
```rust
impl [Aggregate] {
    pub fn validate_[command](&self, cmd: &[CommandName]) -> Result<(), DomainError> {
        // Business rule 1
        if self.field1 < cmd.field2 {
            return Err(DomainError::ValidationFailed("reason"));
        }

        // Business rule 2
        // ...

        Ok(())
    }
}
```

**Event Emission:**
```rust
// In handler (thin orchestration layer)
evento::create::<[Aggregate]>()
    .data(&[Event] {
        field1: cmd.field1,
        field2: cmd.field2,
    })?
    .metadata(&user_id)?
    .commit(&executor)
    .await?;
```

**Critical:** Business logic resides in aggregate, NOT in handlers. Handlers orchestrate only.

---

## 6. UI/UX Specifications

### 6.1 Templates

**Template:** `templates/pages/[page-name].html`

**Purpose:** Renders [what view]

**Data Model:**
```rust
#[derive(Template)]
#[template(path = "pages/[page-name].html")]
pub struct [TemplateName] {
    pub data: [ReadModelType],
    pub user: User,
}
```

**Template Structure:**
```html
{% extends "base.html" %}

{% block title %}[Page Title]{% endblock %}

{% block content %}
<div class="container">
  <h1>[Heading]</h1>

  {% for item in data %}
    {% include "components/[component].html" %}
  {% endfor %}
</div>
{% endblock %}
```

### 6.2 TwinSpark Interactions

**Progressive Enhancement:** AJAX behaviors via TwinSpark attributes

**Example - Replace Meal Slot:**
```html
<form ts-req="/plan/meal/{{ meal.id }}/replace"
      ts-req-method="POST"
      ts-target="#meal-slot-{{ meal.id }}"
      ts-swap="outerHTML">
  <select name="recipe_id">
    {% for recipe in recipes %}
    <option value="{{ recipe.id }}">{{ recipe.title }}</option>
    {% endfor %}
  </select>
  <button type="submit">Replace</button>
</form>
```

**Server Response (HTML Fragment):**
```html
<div id="meal-slot-{{ meal.id }}" class="meal-slot">
  <h4>{{ new_recipe.title }}</h4>
  <span class="prep-indicator">Prep Required</span>
</div>
```

### 6.3 Accessibility Requirements

**WCAG 2.1 Level AA Compliance:**
- [ ] Color contrast 4.5:1 (normal text), 7:1 (Kitchen Mode)
- [ ] Keyboard navigation (Tab, Enter, Escape)
- [ ] ARIA labels for icon-only buttons
- [ ] Form labels explicit (`<label for="input-id">`)
- [ ] Focus indicators visible (2px outline, 4px offset)
- [ ] Screen reader compatible (semantic HTML, ARIA landmarks)

---

## 7. Testing Strategy

### 7.1 Unit Tests

**Aggregate Logic Tests:**

```rust
#[test]
fn test_[business_rule]() {
    let aggregate = [Aggregate]::new(/* ... */);

    // Test business rule validation
    let cmd = [Command] { /* violates rule */ };
    let result = aggregate.validate_[command](&cmd);

    assert!(matches!(result, Err(DomainError::[ErrorType])));
}
```

### 7.2 Projection Tests (with `unsafe_oneshot`)

**Read Model Update Tests:**

```rust
#[tokio::test]
async fn test_[event]_updates_[read_model]() {
    let pool = setup_test_db().await;
    let executor = evento::Executor::new(pool.clone());

    // Emit event
    let entity_id = evento::create::<[Aggregate]>()
        .data(&[Event] { /* ... */ })
        .commit(&executor)
        .await?;

    // Process projection synchronously (deterministic testing)
    evento::subscribe("test-projection")
        .aggregator::<[Aggregate]>()
        .handler([projection_handler])
        .unsafe_oneshot(&executor) // Blocks until processed
        .await?;

    // Assert read model updated
    let result = sqlx::query!(
        "SELECT field1 FROM [table_name] WHERE id = ?",
        entity_id
    )
    .fetch_one(&pool)
    .await?;

    assert_eq!(result.field1, "expected_value");
}
```

**Why `unsafe_oneshot`:**
- `run()` processes events asynchronously (eventual consistency)
- `unsafe_oneshot()` blocks until all events processed (deterministic)
- **Only use in tests** - production uses `run()`

### 7.3 Integration Tests

**HTTP Route Tests:**

```rust
#[tokio::test]
async fn test_[route]_endpoint() {
    let app = test_app().await;
    let client = reqwest::Client::new();

    // Login
    let auth_cookie = login(&client, &app.url).await;

    // Make request
    let resp = client.[method](&format!("{}/[path]", app.url))
        .header("Cookie", format!("auth_token={}", auth_cookie))
        .form(&[("field1", "value1")])
        .send()
        .await
        .unwrap();

    // Assert response
    assert_eq!(resp.status(), StatusCode::[EXPECTED]);
}
```

### 7.4 E2E Tests (Playwright)

**User Flow Tests:**

```typescript
test('[feature] user flow', async ({ page }) => {
  // Login
  await page.goto('/login');
  await page.fill('input[name="email"]', 'test@example.com');
  await page.fill('input[name="password"]', 'password123');
  await page.click('button[type="submit"]');

  // Navigate to feature
  await page.waitForURL('/[route]');

  // Interact with feature
  await page.click('[data-testid="action-button"]');

  // Assert result
  await expect(page.locator('[data-testid="result"]')).toBeVisible();
});
```

### 7.5 Coverage Goals

- **Unit Tests:** 80% code coverage (domain logic)
- **Integration Tests:** All HTTP routes covered
- **E2E Tests:** Critical user flows (happy path + error cases)

---

## 8. Security & Performance

### 8.1 Security Considerations

**Authentication:**
- JWT cookie-based auth (HTTP-only, Secure, SameSite=Lax)
- Token expiration: 7 days
- Password hashing: Argon2 (OWASP-recommended)

**Authorization:**
- User ownership verification on all mutations
- Free tier recipe limit enforced in aggregate (10 recipes)
- Premium tier unrestricted

**Input Validation:**
- Server-side validation with `validator` crate
- SQL injection prevention via SQLx parameterized queries
- XSS prevention via Askama auto-escaping

**OWASP Compliance:**
- [List relevant OWASP Top 10 mitigations]

### 8.2 Performance Targets

| Metric | Target | Measurement |
|--------|--------|-------------|
| Page Load Time | <500ms | Time to First Byte (TTFB) |
| Read Model Query | <50ms | Database query latency |
| Command Execution | <200ms | Event write + projection trigger |
| Projection Latency | <100ms | Event → Read model update |

**Optimization Strategies:**
- Page-specific read models (no joins, optimized queries)
- Database indexes on foreign keys and filter columns
- Connection pooling (write pool: 1 connection, read pool: N connections)
- SQLite PRAGMAs (WAL mode, optimized cache)

---

## 9. Implementation Plan

### 9.1 Task Breakdown

**Phase 1: Data Layer**
- [ ] Create migration files for page-specific read models
- [ ] Implement projection handlers
- [ ] Write unit tests for projections (`unsafe_oneshot`)
- [ ] Register subscriptions in main.rs

**Phase 2: Domain Logic**
- [ ] Implement aggregate with event handlers
- [ ] Write business rule validation logic
- [ ] Write unit tests for aggregate logic
- [ ] Document business rules

**Phase 3: HTTP Layer**
- [ ] Implement route handlers (thin orchestration)
- [ ] Create Askama templates
- [ ] Add TwinSpark attributes for AJAX
- [ ] Write integration tests for routes

**Phase 4: UI/UX**
- [ ] Implement Tailwind CSS 4.1+ styling
- [ ] Ensure WCAG 2.1 AA accessibility
- [ ] Add Kitchen Mode support (high contrast)
- [ ] Test responsive breakpoints (mobile/tablet/desktop)

**Phase 5: Testing & Validation**
- [ ] E2E tests (Playwright)
- [ ] Load testing (k6 or Locust)
- [ ] Security audit (OWASP checklist)
- [ ] Accessibility testing (axe-core, screen readers)

**Phase 6: Deployment**
- [ ] Create Docker image
- [ ] Kubernetes manifests
- [ ] CI/CD pipeline (GitHub Actions)
- [ ] Monitoring and observability (OpenTelemetry)

### 9.2 Dependencies

**Blocked by:**
- [Prerequisite feature/epic]
- [Infrastructure requirement]

**Blocks:**
- [Downstream feature/epic]

### 9.3 Estimated Effort

| Phase | Estimated Time | Confidence |
|-------|----------------|------------|
| Phase 1 | [X] days | High/Medium/Low |
| Phase 2 | [X] days | High/Medium/Low |
| Phase 3 | [X] days | High/Medium/Low |
| Phase 4 | [X] days | High/Medium/Low |
| Phase 5 | [X] days | High/Medium/Low |
| Phase 6 | [X] days | High/Medium/Low |
| **Total** | [X] days | - |

---

## 10. Open Questions

**Technical Questions:**
1. [Question 1]?
   - **Answer:** [Resolution or "TBD"]
   - **Owner:** [Name]
   - **Due Date:** [YYYY-MM-DD]

2. [Question 2]?
   - **Answer:** [Resolution or "TBD"]
   - **Owner:** [Name]
   - **Due Date:** [YYYY-MM-DD]

**Product Questions:**
1. [Question 1]?
   - **Answer:** [Resolution or "TBD"]
   - **Owner:** [Product Manager]

**Design Questions:**
1. [Question 1]?
   - **Answer:** [Resolution or "TBD"]
   - **Owner:** [Designer]

---

## Appendix

### A. Related Documents

- **PRD:** `/docs/PRD.md`
- **Solution Architecture:** `/docs/solution-architecture.md`
- **Epic Breakdown:** `/docs/epics.md`
- **UX Specification:** `/docs/ux-specification.md`
- **Migration Plan:** `/docs/read-model-migration-plan.md` (if applicable)

### B. References

- **evento Documentation:** [evento crate docs]
- **SQLx Documentation:** [sqlx crate docs]
- **TwinSpark API:** `/docs/twinspark.md`
- **Tailwind CSS 4.1+:** [Official docs]

### C. Revision History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| [YYYY-MM-DD] | 1.0 | Initial draft | [Name] |
| [YYYY-MM-DD] | 1.1 | [Changes] | [Name] |

---

_Template Version: 1.0_
_Generated by Winston (Architect Agent) - 2025-10-24_

# Technical Specification: Enhanced Meal Planning - Testing & Refinement

Date: 2025-10-27
Author: Jonathan
Epic ID: 10
Status: Approved

---

## Overview

Epic 10 represents the final validation phase of the Enhanced Meal Planning system before production deployment. This epic focuses on comprehensive testing, performance validation, bug resolution, and documentation completion to ensure the multi-week meal planning system meets all quality, performance, and usability requirements defined in previous epics (6-9).

The enhanced meal planning system introduces significant architectural complexity: multi-week plan generation, week-specific regeneration, meal planning preferences with accompaniment controls, and coordinated shopping list generation across weeks. This epic validates that these features function correctly under realistic load conditions, meet performance benchmarks (generation <5s, routes <500ms P95), and provide an intuitive user experience backed by comprehensive documentation.

## Objectives and Scope

### Objectives

1. **Validation Through E2E Testing**: Implement comprehensive Playwright test suite covering all critical user flows for multi-week meal planning (generation, navigation, regeneration, preferences, shopping lists)

2. **Performance Verification**: Conduct load testing and benchmarking to validate system meets PRD performance requirements under realistic usage patterns (100 concurrent users, 50 recipes per user)

3. **Quality Assurance**: Identify and resolve all critical bugs, triage medium/low priority issues, implement regression tests to prevent bug reappearance

4. **Documentation Completion**: Create user-facing documentation (user guides) and developer documentation (API docs, architecture updates) to support production launch and future development

### In Scope

- **E2E Testing (Story 10.1)**:
  - Playwright test suite with TypeScript fixtures for authenticated sessions
  - Critical flow tests: multi-week generation, week navigation, single-week regeneration, full regeneration, preference updates, recipe creation with accompaniments, shopping list access
  - CI integration with <5 minute execution time
  - Test video recording for failure debugging

- **Performance Testing (Story 10.2)**:
  - Load testing with k6 (100 concurrent multi-week generation requests)
  - P95 latency benchmarks: generation <5s, route responses <500ms
  - Database query profiling (N+1 query detection)
  - Memory usage profiling (leak detection, growth bounds)
  - Performance regression tests added to CI

- **Bug Fixing (Story 10.3)**:
  - Critical bug resolution (deployment blockers)
  - Medium bug triage (fix vs defer to future release)
  - Low bug documentation (known issues list)
  - Edge case handling (graceful errors, no crashes)
  - User-friendly error messages (no stack traces)
  - Regression test implementation for all fixed bugs

- **Documentation (Story 10.4)**:
  - User guide: `docs/user-guide-meal-planning.md` covering generation, navigation, regeneration, preferences, accompaniments
  - API documentation: `docs/api/meal-planning-routes.md` with route signatures and examples
  - Architecture updates: "as-built" notes in `solution-architecture.md`
  - README updates with new features
  - Screenshots via Playwright API for consistency
  - Code comments for complex algorithm functions
  - Deployment guide updates (migrations, environment variables)

### Out of Scope

- **Property-based testing**: Deferred to future enhancement (mentioned in architecture but not required for MVP)
- **Advanced load testing**: Beyond 100 concurrent users (sufficient for MVP validation)
- **Accessibility testing automation**: Manual NVDA/VoiceOver testing per sprint is sufficient (axe-core integration deferred)
- **Penetration testing**: Quarterly cadence post-launch, not blocking MVP
- **Internationalization testing**: English-only in MVP
- **Mobile native app testing**: PWA-only in MVP

## System Architecture Alignment

### Architectural Context

Epic 10 validates the event-sourced monolith architecture implemented in Epics 6-9:

**Core Technologies Tested**:
- **Rust 1.90+**: Type-safe domain logic in `crates/meal-plan` aggregate
- **evento 1.3+**: Event sourcing for meal plan generation and regeneration commands
- **Axum 0.8+**: HTTP server handling multi-week meal planning routes
- **SQLite 3.45+**: Database with WAL mode, optimized PRAGMAs (journal_mode, cache_size, synchronous)
- **Askama 0.14+**: Server-side HTML templates for meal plan calendar views
- **TwinSpark**: Progressive enhancement for week navigation and regeneration actions
- **Playwright 1.56+**: Cross-browser E2E testing (Chromium, Firefox, WebKit)

### Component Integration Points

**Domain Crates**:
- `crates/meal-plan`: Multi-week generation algorithm, rotation state, preference application
- `crates/recipe`: Recipe favoriting, accompaniment settings (`can_be_side_dish`, `needs_side_dish`)
- `crates/shopping`: Multi-week shopping list generation from meal plans

**Read Models** (validated via E2E and integration tests):
- `meal_plan_preferences`: User preferences for meal planning (breakfast/lunch/dinner flags, side dish preferences)
- `meal_plans` + `meal_assignments`: Multi-week plan storage with course assignments
- `shopping_lists` + `shopping_list_items`: Per-week shopping lists with aggregated ingredients

**HTTP Routes** (performance tested):
- `POST /plan/generate-multi-week`: Multi-week meal plan generation
- `GET /plan?week=YYYY-MM-DD`: Week-specific calendar view
- `POST /plan/regenerate-week?week=YYYY-MM-DD`: Single week regeneration
- `POST /plan/regenerate-future`: Regenerate all future weeks
- `GET /shopping?week=YYYY-MM-DD`: Week-specific shopping list
- `POST /profile/meal-planning-preferences`: Update preferences

### Database Performance Constraints

**Connection Pool Configuration** (validated via load testing):
- Write Pool: 1 connection max (prevents SQLITE_BUSY errors)
- Read Pool: 5 connections max (concurrent reads with WAL mode)
- Busy timeout: 5000ms (5 seconds for lock acquisition)

**PRAGMA Optimizations** (performance tested):
- `journal_mode = WAL`: Enables concurrent reads/writes (critical for multi-user load testing)
- `cache_size = -20000`: 20MB memory cache for query performance
- `synchronous = NORMAL`: Safe with WAL, improves write performance

### Testing Architecture

**Test Pyramid Implementation**:
1. **Unit Tests**: Domain aggregate logic (evento commands/events) in `crates/*/tests/`
2. **Integration Tests**: HTTP routes with `unsafe_oneshot` for deterministic projection testing in `tests/` directory
3. **E2E Tests**: Playwright tests in `e2e/tests/` directory covering full user flows

**CI/CD Validation** (GitHub Actions):
- Unit + integration test execution (cargo test)
- E2E test execution (Playwright with video recording)
- Code coverage enforcement (80% minimum via cargo-tarpaulin)
- Performance regression detection (benchmark comparisons)

### Observability Integration

**OpenTelemetry Instrumentation** (validated in load testing):
- Distributed tracing: Track meal plan generation latency across aggregate loading, algorithm execution, event commits
- Metrics: Request rates, error rates, P95/P99 latencies
- Structured logging: Correlation IDs for debugging failed test scenarios

## Detailed Design

### Services and Modules

Epic 10 focuses on testing and validation infrastructure rather than new service implementation. The following modules support the testing and documentation deliverables:

| Module | Location | Responsibilities | Inputs | Outputs | Owner |
|--------|----------|------------------|--------|---------|-------|
| **E2E Test Suite** | `e2e/tests/` | Execute critical user flows via browser automation | Playwright config, test fixtures, deployed application | Test results, failure videos, coverage reports | QA Engineer |
| **Performance Testing** | `e2e/performance/` | Load testing and benchmarking | k6 scripts, test data generators | Latency metrics, throughput data, profiling reports | Performance Engineer |
| **Test Fixtures** | `e2e/fixtures/` | Authenticated session setup, test data generation | User credentials, recipe data | Authenticated Playwright contexts, seeded database | QA Engineer |
| **Integration Tests** | `tests/` | HTTP route validation, projection testing with `unsafe_oneshot` | In-memory database, test data | Test pass/fail, code coverage | Backend Developer |
| **Documentation** | `docs/` | User guides, API docs, architecture updates | Architecture diagrams, route signatures, user flows | Markdown documentation, screenshots | Technical Writer |
| **CI/CD Pipeline** | `.github/workflows/` | Automated testing, coverage enforcement, performance regression detection | GitHub Actions triggers, test suites | Build status, deployment artifacts | DevOps |

**Testing Module Dependencies**:
- E2E tests depend on deployed application (local dev server or staging environment)
- Performance tests require realistic data seeding (50 favorite recipes per user, 100 test users)
- Integration tests use isolated SQLite databases with WAL mode enabled
- Documentation uses Playwright screenshot API for consistent visuals

### Data Models and Contracts

Epic 10 does not introduce new data models but validates existing models through comprehensive testing:

**Validated Domain Events** (evento event stream):
```rust
// MealPlan aggregate events (tested via E2E and integration)
MealPlanGenerated {
    user_id: String,
    start_weeks: Vec<String>,    // Monday dates: ["2025-11-03", "2025-11-10", ...]
    assignments: Vec<MealAssignment>,
    rotation_state: RotationState,
    preferences_applied: MealPlanPreferences,
}

WeekRegenerated {
    user_id: String,
    week_start: String,          // Monday date: "2025-11-03"
    new_assignments: Vec<MealAssignment>,
    rotation_state: RotationState,
}

FutureWeeksRegenerated {
    user_id: String,
    regenerated_weeks: Vec<String>, // Monday dates
    new_assignments: Vec<MealAssignment>,
    rotation_state: RotationState,
}

MealPlanPreferencesUpdated {
    user_id: String,
    preferences: MealPlanPreferences,
}
```

**Validated Read Models** (SQLite tables):
```sql
-- Tested via integration tests with unsafe_oneshot
CREATE TABLE meal_plan_preferences (
    user_id TEXT PRIMARY KEY,
    generate_breakfast BOOLEAN NOT NULL DEFAULT true,
    generate_lunch BOOLEAN NOT NULL DEFAULT true,
    generate_dinner BOOLEAN NOT NULL DEFAULT true,
    prefer_side_dishes BOOLEAN NOT NULL DEFAULT false,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id)
);

CREATE TABLE meal_plans (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    start_weeks TEXT NOT NULL,   -- JSON array: ["2025-11-03", "2025-11-10"]
    status TEXT NOT NULL,         -- "active"|"archived"
    rotation_state TEXT,          -- JSON: tracks used recipes
    created_at TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id)
);

CREATE TABLE meal_assignments (
    id TEXT PRIMARY KEY,
    meal_plan_id TEXT NOT NULL,
    week_start TEXT NOT NULL,     -- Monday date: "2025-11-03"
    date TEXT NOT NULL,           -- Full date: "2025-11-04"
    course_type TEXT NOT NULL,    -- "appetizer"|"main_course"|"dessert"
    recipe_id TEXT NOT NULL,
    prep_required BOOLEAN,
    algorithm_reasoning TEXT,     -- "Saturday: more prep time available"
    FOREIGN KEY (meal_plan_id) REFERENCES meal_plans(id),
    FOREIGN KEY (recipe_id) REFERENCES recipes(id)
);

CREATE TABLE shopping_lists (
    id TEXT PRIMARY KEY,
    meal_plan_id TEXT NOT NULL,
    week_start TEXT NOT NULL,     -- Monday date for shopping list
    generated_at TEXT NOT NULL,
    FOREIGN KEY (meal_plan_id) REFERENCES meal_plans(id)
);
```

**Test Data Contracts** (Playwright fixtures):
```typescript
// e2e/fixtures/auth.ts
interface TestUser {
    email: string;
    password: string;
    userId: string;
    authCookie: string; // JWT cookie value
}

// e2e/fixtures/recipes.ts
interface TestRecipe {
    id: string;
    title: string;
    recipe_type: 'appetizer' | 'main_course' | 'dessert';
    prep_time_min: number;
    cook_time_min: number;
    can_be_side_dish: boolean;
    needs_side_dish: boolean;
    is_favorite: boolean;
}

// e2e/fixtures/meal-plan.ts
interface GeneratedMealPlan {
    id: string;
    start_weeks: string[];        // ["2025-11-03", "2025-11-10"]
    assignments: MealAssignment[];
    shopping_lists: ShoppingList[];
}
```

### APIs and Interfaces

Epic 10 validates existing HTTP routes through E2E and performance testing. No new routes are introduced.

**Routes Under Test** (performance benchmarks: P95 <500ms except generation):

```rust
// Meal Planning Routes (primary focus of Epic 10)
POST /plan/generate-multi-week
    Request: Form { num_weeks: u32 }  // Default: 4 weeks
    Response: 303 Redirect to /plan?week={first_monday}
    Performance: P95 <5s (NFR requirement)
    Test Coverage: E2E (Story 10.1), Performance (Story 10.2)

GET /plan?week=YYYY-MM-DD
    Request: Query { week: Option<String> }  // Default: current week
    Response: 200 HTML (Askama template: meal_plan_calendar.html)
    Performance: P95 <500ms
    Test Coverage: E2E (navigation), Performance

POST /plan/regenerate-week?week=YYYY-MM-DD
    Request: Query { week: String }  // Monday date
    Response: 303 Redirect to /plan?week={regenerated_week}
    Performance: P95 <3s (single week faster than multi-week)
    Test Coverage: E2E (Story 10.1), Performance (Story 10.2)

POST /plan/regenerate-future
    Request: None
    Response: 303 Redirect to /plan?week={current_week}
    Performance: P95 <5s (regenerates all future weeks)
    Test Coverage: E2E (Story 10.1), Performance (Story 10.2)

// Meal Planning Preferences Routes
GET /profile/meal-planning-preferences
    Request: None (authenticated user from JWT cookie)
    Response: 200 HTML (Askama template: preferences_form.html)
    Performance: P95 <200ms (simple form load)
    Test Coverage: E2E (Story 10.1)

POST /profile/meal-planning-preferences
    Request: Form {
        generate_breakfast: bool,
        generate_lunch: bool,
        generate_dinner: bool,
        prefer_side_dishes: bool,
    }
    Response: 303 Redirect to /profile
    Performance: P95 <300ms
    Test Coverage: E2E (Story 10.1), Integration tests

// Shopping List Routes
GET /shopping?week=YYYY-MM-DD
    Request: Query { week: Option<String> }  // Default: current week
    Response: 200 HTML (Askama template: shopping_list.html)
    Performance: P95 <500ms
    Test Coverage: E2E (Story 10.1), Performance (Story 10.2)

// Recipe Routes (tested for accompaniment settings)
POST /recipes
    Request: Form {
        title: String,
        recipe_type: String,        // Required: "appetizer"|"main_course"|"dessert"
        can_be_side_dish: bool,     // New accompaniment field
        needs_side_dish: bool,      // New accompaniment field
        // ... other fields
    }
    Response: 303 Redirect to /recipes/{id}
    Performance: P95 <400ms
    Test Coverage: E2E (Story 10.1 - recipe creation with accompaniments)
```

**Error Response Contracts** (validated via integration tests):
```rust
// 422 Unprocessable Entity (validation errors)
{
    "error": "Validation failed",
    "details": {
        "num_weeks": "Must be between 1 and 8"
    }
}

// 404 Not Found (invalid week date)
{
    "error": "Meal plan not found for week 2025-11-32"
}

// 500 Internal Server Error (graceful handling, no stack traces)
{
    "error": "An error occurred. Please try again later.",
    "request_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

**TwinSpark Attributes** (tested via E2E for interactivity):
```html
<!-- Week navigation -->
<button ts-req="GET /plan?week=2025-11-10"
        ts-target="#meal-calendar"
        ts-swap="outerHTML">
    Next Week →
</button>

<!-- Single week regeneration -->
<form ts-req="POST /plan/regenerate-week?week=2025-11-03"
      ts-req-method="POST"
      ts-trigger="submit">
    <button type="submit">Regenerate This Week</button>
</form>
```

### Workflows and Sequencing

#### Workflow 1: E2E Test Execution Flow

```
Developer Pushes Code
  ↓
GitHub Actions Triggered (.github/workflows/test.yml)
  ↓
[Parallel Execution]
  ├─ Unit Tests (cargo test --lib)
  │   └─ Domain aggregate logic tests
  ├─ Integration Tests (cargo test --test '*')
  │   ├─ HTTP route tests
  │   └─ Projection tests with unsafe_oneshot
  └─ E2E Tests (Playwright)
      ↓
    Setup Phase (e2e/fixtures/setup.ts)
      ├─ Start local dev server (cargo run --release)
      ├─ Wait for server ready (health check: GET /health)
      ├─ Seed test database (50 recipes per user, 10 users)
      └─ Generate auth cookies (JWT tokens)
      ↓
    Test Execution (parallel workers)
      ├─ meal-planning.spec.ts
      │   ├─ Test: Generate multi-week meal plan
      │   ├─ Test: Navigate between weeks
      │   ├─ Test: Regenerate single week
      │   └─ Test: Regenerate all future weeks
      ├─ preferences.spec.ts
      │   └─ Test: Update meal planning preferences
      ├─ recipes.spec.ts
      │   └─ Test: Create recipe with accompaniment settings
      └─ shopping.spec.ts
          └─ Test: View shopping list for specific week
      ↓
    Teardown Phase
      ├─ Stop dev server
      ├─ Clean test database
      └─ Archive failure videos (if any)
      ↓
    Report Generation
      ├─ Test results (passed/failed/skipped)
      ├─ Video recordings for failures
      └─ Code coverage report (HTML)
      ↓
  Check: All tests passed?
    ├─ YES → Build succeeds, deployment proceeds
    └─ NO → Build fails, notify developers
```

#### Workflow 2: Performance Testing Flow

```
Performance Engineer Executes k6 Script
  ↓
Load Test Configuration (e2e/performance/load-test.js)
  ├─ Virtual Users: 100 concurrent
  ├─ Duration: 5 minutes sustained load
  ├─ Ramp-up: 30 seconds (0 → 100 users)
  └─ Test Data: 50 favorite recipes per user
  ↓
[Load Generation]
  ├─ Stage 1: Generate Multi-Week Meal Plans
  │   └─ POST /plan/generate-multi-week (num_weeks=4)
  ├─ Stage 2: Navigate Meal Plans
  │   └─ GET /plan?week=YYYY-MM-DD (random weeks)
  ├─ Stage 3: Regenerate Single Weeks
  │   └─ POST /plan/regenerate-week?week=YYYY-MM-DD
  └─ Stage 4: Access Shopping Lists
      └─ GET /shopping?week=YYYY-MM-DD
  ↓
Metrics Collection (OpenTelemetry)
  ├─ Request latencies (P50, P95, P99)
  ├─ Error rates (4xx, 5xx)
  ├─ Database query times (via tracing)
  └─ Memory usage (heap profiling)
  ↓
Performance Analysis
  ├─ Compare against benchmarks:
  │   ├─ Generation P95: <5s (PASS/FAIL)
  │   ├─ Route P95: <500ms (PASS/FAIL)
  │   └─ Error rate: <1% (PASS/FAIL)
  ├─ Database query profiling:
  │   ├─ Identify N+1 queries (SQLx query logs)
  │   └─ Check index usage (EXPLAIN QUERY PLAN)
  └─ Memory profiling:
      ├─ Heap growth analysis (cargo-flamegraph)
      └─ Leak detection (valgrind for C dependencies)
  ↓
Report Generation (docs/performance-report.md)
  ├─ Benchmark results table
  ├─ Latency histograms
  ├─ Bottleneck analysis
  └─ Optimization recommendations
```

#### Workflow 3: Integration Test with unsafe_oneshot

```
Integration Test Execution (tests/meal_plan_tests.rs)
  ↓
Test Setup
  ├─ Create in-memory SQLite database
  ├─ Apply migrations (evento + read models)
  ├─ Configure PRAGMAs (WAL, cache_size, etc.)
  ├─ Initialize evento Executor
  └─ Seed test user and 10 favorite recipes
  ↓
Execute Command (Generate Meal Plan)
  ├─ POST /plan/generate-multi-week (num_weeks=2)
  └─ evento::create::<MealPlan>()
      .data(&MealPlanGenerated { ... })
      .commit(&executor)
      .await
  ↓
Process Projections SYNCHRONOUSLY (critical for deterministic testing)
  └─ evento::subscribe("meal-plan-projections")
      .aggregator::<MealPlan>()
      .handler(project_meal_plan_to_read_models)
      .unsafe_oneshot(&executor)  // Blocks until all events processed
      .await
  ↓
Assert Read Model State
  ├─ Query meal_plans table:
  │   └─ SELECT * FROM meal_plans WHERE user_id = ?
  ├─ Query meal_assignments table:
  │   └─ SELECT * FROM meal_assignments WHERE meal_plan_id = ?
  └─ Assertions:
      ├─ Assert: 2 weeks of assignments (14 days)
      ├─ Assert: 3 courses per day (appetizer, main, dessert)
      ├─ Assert: No duplicate recipes within rotation
      └─ Assert: All recipes match user's favorites
  ↓
Test Cleanup
  └─ Drop in-memory database (automatic on test exit)
```

**Key Testing Pattern**: Use `unsafe_oneshot` instead of `run` for event subscriptions in tests to ensure synchronous, deterministic projection processing. This prevents race conditions where assertions execute before projections complete.

## Non-Functional Requirements

### Performance

**Performance Benchmarks** (validated via Story 10.2):

| Metric | Target | Test Method | Acceptance |
|--------|--------|-------------|------------|
| Multi-week meal plan generation | P95 <5s | k6 load test (100 concurrent users, 50 recipes per user) | P95 latency ≤5000ms |
| Single week regeneration | P95 <3s | k6 load test | P95 latency ≤3000ms |
| Route response times (GET) | P95 <500ms | k6 load test (meal calendar, shopping list) | P95 latency ≤500ms |
| Route response times (POST) | P95 <300ms | k6 load test (preferences update) | P95 latency ≤300ms |
| Database query performance | No N+1 queries | SQL query log analysis | Zero N+1 queries detected |
| Memory usage | Bounded growth | Heap profiling (cargo-flamegraph) | <100MB growth per 1000 requests |
| Error rate under load | <1% | k6 metrics (4xx, 5xx responses) | Error rate <1% |

**E2E Test Performance Requirements** (Story 10.1):
- Test suite execution time: <5 minutes for full suite (parallelized across 4 workers)
- Individual test timeout: 60 seconds per test
- Video recording overhead: <10% performance impact

**CI/CD Performance Requirements**:
- Total CI pipeline duration: <15 minutes (unit tests + integration tests + E2E tests)
- Playwright browser launch: <5 seconds per worker
- Test database seeding: <10 seconds for 100 test recipes

### Security

**Test Data Security**:
- Test user credentials: Use fake data (test@example.com, never real emails)
- JWT secrets: Use dedicated test secret (not production secret)
- Database isolation: In-memory databases for integration tests, isolated test DB for E2E
- No sensitive data in test fixtures: Avoid real PII, credit cards, or secrets

**CI/CD Security**:
- Secrets management: GitHub Secrets for test environment variables
- Test database encryption: Not required (ephemeral test data)
- Code coverage reports: No sensitive data exposure in HTML reports
- Video recordings: Auto-delete after 30 days (GitHub Actions artifacts)

**Error Handling Validation** (Story 10.3):
- No stack traces in user-facing error messages
- Request IDs for debugging without exposing internals
- Graceful degradation: 500 errors render user-friendly HTML (not JSON)

### Reliability/Availability

**Test Reliability Requirements**:
- Flaky test tolerance: 0% (all tests must be deterministic)
- Test isolation: Each test uses isolated database (no shared state)
- Retry logic: No automatic retries (flaky tests must be fixed, not masked)
- Parallel execution: Tests must pass when run in parallel (4 workers)

**CI/CD Reliability**:
- Build reproducibility: Docker-based builds (pinned base images)
- Test data consistency: Seeded data must be deterministic (fixed random seed)
- Playwright reliability: Use `waitForSelector` instead of fixed delays
- Database state: Clean state between tests (migrations + teardown)

**Availability Testing**:
- Graceful error handling: All error states render user-friendly messages
- Offline behavior: PWA offline mode tested via Playwright (network throttling)
- Database unavailability: Test connection failures (mock SQLITE_BUSY errors)

### Observability

**Test Observability Requirements**:

**Playwright Test Reporting**:
- HTML report: Generated for every test run (e2e/playwright-report/)
- Video recordings: Captured for failed tests only (saves storage)
- Screenshots: Captured at failure point
- Trace files: Full timeline, network, console logs (for debugging)
- Test duration metrics: Per-test timing data for performance tracking

**Performance Test Observability**:
- k6 metrics export: JSON output for trend analysis
- Latency histograms: P50, P90, P95, P99 percentiles
- Request distribution: Breakdown by route (generation, navigation, regeneration)
- Error tracking: 4xx/5xx categorization with error messages
- Database query logs: Trace ID correlation for slow queries

**CI/CD Observability**:
- GitHub Actions job summaries: Test pass/fail counts, coverage percentage
- Code coverage trends: Track coverage over time (cargo-tarpaulin HTML report)
- Performance regression detection: Compare P95 latencies against baseline
- Artifact uploads: Test reports, videos, coverage reports (7-day retention)

**OpenTelemetry Integration** (tested via load testing):
- Distributed tracing: Validate spans for meal plan generation (aggregate load → algorithm → event commit)
- Metrics: Counter for test runs, histogram for test durations
- Logs: Structured logging with correlation IDs (link test failures to server logs)

## Dependencies and Integrations

### External Tool Dependencies

| Dependency | Version | Purpose | Installation | License |
|------------|---------|---------|--------------|---------|
| **Playwright** | 1.56+ | E2E browser automation | `npm install -D @playwright/test` | Apache 2.0 |
| **k6** | 0.50+ | Load testing and performance benchmarking | Binary download or Docker | AGPL 3.0 |
| **cargo-tarpaulin** | 0.30+ | Rust code coverage | `cargo install cargo-tarpaulin` | Apache 2.0 / MIT |
| **cargo-flamegraph** | 0.6+ | Memory profiling (heap analysis) | `cargo install flamegraph` | Apache 2.0 / MIT |
| **SQLite CLI** | 3.45+ | Database query profiling (EXPLAIN) | System package | Public Domain |
| **valgrind** | 3.21+ | Memory leak detection (C dependencies) | System package | GPL 2.0 |

### Internal Module Dependencies

**E2E Test Dependencies** (e2e/package.json):
```json
{
  "devDependencies": {
    "@playwright/test": "^1.56.0",
    "typescript": "^5.7.0",
    "dotenv": "^16.4.0"
  }
}
```

**Rust Test Dependencies** (Cargo.toml):
```toml
[dev-dependencies]
tokio-test = "0.4"
sqlx = { version = "0.8", features = ["sqlite", "runtime-tokio"] }
evento = { version = "1.4", features = ["test"] }  # Includes unsafe_oneshot
axum-test = "16.0"  # HTTP integration testing helpers
```

### CI/CD Integration Points

**GitHub Actions Workflow** (.github/workflows/test.yml):
- Trigger: Push to main, pull requests
- Runners: ubuntu-latest (4 concurrent jobs)
- Caching: Cargo dependencies, npm packages, Playwright browsers
- Artifacts: Test reports, videos, coverage HTML

**Deployment Gate Integration**:
- Epic 10 validation required before production deployment
- All E2E tests must pass (Story 10.1)
- Performance benchmarks must meet targets (Story 10.2)
- Zero critical bugs remaining (Story 10.3)
- Documentation complete (Story 10.4)

### Database Dependency

**SQLite Configuration** (validated via tests):
- WAL mode: Concurrent reads during writes (critical for load testing)
- PRAGMAs: `journal_mode=WAL, cache_size=-20000, synchronous=NORMAL, busy_timeout=5000`
- Connection pooling: Write pool (1 connection), Read pool (5 connections)
- Migrations: SQLx migrations applied automatically in test setup

### Browser Compatibility Testing

**Playwright Browser Support** (Story 10.1):
- Chromium: Latest stable (primary test target)
- Firefox: Latest stable (secondary test target)
- WebKit: Latest stable (Safari equivalent, tertiary test target)

**PWA Validation**:
- Service worker caching tested via Chromium (network throttling)
- Offline mode tested via Playwright (navigator.onLine mocking)
- Installation prompts: Not tested in E2E (browser-specific behavior)

## Acceptance Criteria (Authoritative)

### Story 10.1: End-to-End Testing with Playwright

**AC1**: Playwright test suite configured in `e2e/` directory with TypeScript fixtures for authenticated sessions
- Verify: `e2e/playwright.config.ts` exists with parallel workers (4), base URL, auth storage path
- Verify: `e2e/fixtures/auth.ts` provides `authenticatedPage` fixture with JWT cookie

**AC2**: Test coverage for multi-week meal plan generation flow
- Verify: `e2e/tests/meal-planning.spec.ts` includes test: "User generates multi-week meal plan (4 weeks)"
- Verify: Test navigates to `/plan`, submits generation form, asserts success redirect

**AC3**: Test coverage for week navigation
- Verify: Test "User navigates between weeks" clicks "Next Week" button
- Verify: Calendar updates to show next Monday-Sunday range

**AC4**: Test coverage for single week regeneration
- Verify: Test "User regenerates single week" clicks "Regenerate This Week" button
- Verify: Meal assignments change for that week only (other weeks unchanged)

**AC5**: Test coverage for all future weeks regeneration
- Verify: Test "User regenerates all future weeks" clicks "Regenerate All Future Weeks" button
- Verify: All weeks from current week forward are regenerated

**AC6**: Test coverage for meal planning preferences update
- Verify: `e2e/tests/preferences.spec.ts` navigates to `/profile/meal-planning-preferences`
- Verify: Test toggles checkboxes (breakfast, lunch, dinner, side dishes), submits form
- Verify: Preferences saved (next meal plan generation respects preferences)

**AC7**: Test coverage for recipe creation with accompaniment settings
- Verify: `e2e/tests/recipes.spec.ts` navigates to `/recipes/new`
- Verify: Test fills form with `can_be_side_dish` and `needs_side_dish` checkboxes
- Verify: Recipe saved with correct accompaniment settings

**AC8**: Test coverage for shopping list access for specific week
- Verify: `e2e/tests/shopping.spec.ts` navigates to `/shopping?week=2025-11-10`
- Verify: Shopping list displays ingredients for that week only

**AC9**: All E2E tests pass in CI pipeline
- Verify: GitHub Actions workflow executes `npx playwright test` successfully
- Verify: Exit code 0 (all tests passed)

**AC10**: Test execution time <5 minutes
- Verify: GitHub Actions job duration for E2E tests <5 minutes (300 seconds)

### Story 10.2: Performance Testing and Optimization

**AC1**: Load test with 100 concurrent multi-week generation requests
- Verify: k6 script `e2e/performance/load-test.js` configured with 100 virtual users
- Verify: Test runs for 5 minutes sustained load with 30-second ramp-up

**AC2**: P95 generation time <5 seconds
- Verify: k6 output shows `http_req_duration{route=generate-multi-week} P95 < 5000ms`
- Verify: Benchmark meets PRD NFR-1 requirement

**AC3**: P95 route response time <500ms
- Verify: k6 output shows `http_req_duration{route=plan-calendar} P95 < 500ms`
- Verify: k6 output shows `http_req_duration{route=shopping-list} P95 < 500ms`

**AC4**: Database query performance profiled (no N+1 queries)
- Verify: SQL query logs analyzed (SQLx tracing output)
- Verify: Zero N+1 query patterns detected (ingredient loading, recipe fetching)
- Verify: `EXPLAIN QUERY PLAN` output shows index usage for all queries

**AC5**: Memory usage profiled (no leaks, bounded growth)
- Verify: Heap profile generated via `cargo-flamegraph` or valgrind
- Verify: Memory growth <100MB per 1000 requests (linear, bounded)
- Verify: No leaks reported by valgrind (if used for C dependencies)

**AC6**: Performance regression tests added to CI
- Verify: GitHub Actions workflow includes performance benchmarking step
- Verify: Baseline latencies stored (JSON file in repo)
- Verify: CI fails if P95 latency exceeds baseline by >20%

**AC7**: Optimization recommendations documented (if targets not met)
- Verify: `docs/performance-report.md` exists if benchmarks fail
- Verify: Document includes: bottleneck analysis, optimization suggestions, action items

### Story 10.3: Bug Fixing and Edge Case Handling

**AC1**: All critical bugs fixed (blocker for deployment)
- Verify: GitHub Issues labeled "critical" or "blocker" have status "closed"
- Verify: Critical bug list = 0 open issues

**AC2**: Medium bugs triaged (fix or defer to future release)
- Verify: GitHub Issues labeled "medium" have triage decision documented
- Verify: Fixed bugs closed, deferred bugs labeled "future-release" and backlog prioritized

**AC3**: Low bugs documented (known issues list)
- Verify: `docs/known-issues.md` exists with list of low-priority bugs
- Verify: Each bug includes: description, workaround (if any), target fix version

**AC4**: Edge cases handled gracefully (no crashes)
- Verify: Integration tests cover edge cases: 0 favorite recipes, >100 favorite recipes, invalid week dates
- Verify: All edge case tests pass without panics or crashes

**AC5**: Error messages user-friendly (no stack traces shown)
- Verify: E2E tests for error scenarios (generate with 0 recipes) show user-friendly message
- Verify: HTML error pages do not contain Rust panic messages or stack traces
- Verify: Request ID displayed for debugging ("Request ID: 550e8400...")

**AC6**: Regression tests added for fixed bugs
- Verify: Each fixed bug has corresponding unit or integration test
- Verify: Test reproduces original bug (fails on pre-fix code)
- Verify: Test passes on fixed code

**AC7**: Bug fix changelog documented
- Verify: `CHANGELOG.md` updated with "Bug Fixes" section for Epic 10
- Verify: Each fixed bug listed with issue number and brief description

### Story 10.4: Documentation Updates

**AC1**: User guide created: `docs/user-guide-meal-planning.md`
- Verify: File exists with sections: Introduction, Generating Plans, Navigation, Regeneration, Preferences, Accompaniments
- Verify: Non-technical language (no code examples, aimed at end users)

**AC2**: User guide covers all key features
- Verify: Section "Generating Multi-Week Meal Plans" explains how to use generation form
- Verify: Section "Navigating Between Weeks" explains Previous/Next Week buttons
- Verify: Section "Regenerating Meal Plans" covers single week and all future weeks regeneration
- Verify: Section "Setting Meal Planning Preferences" explains breakfast/lunch/dinner toggles and side dish preference
- Verify: Section "Accompaniments" explains `can_be_side_dish` and `needs_side_dish` recipe settings

**AC3**: API documentation updated: `docs/api/meal-planning-routes.md`
- Verify: File exists with route signatures for all meal planning routes
- Verify: Each route includes: HTTP method, path, request params, response codes, example requests

**AC4**: Architecture document updated with "as-built" notes
- Verify: `docs/solution-architecture.md` includes section "Epic 10 Enhancements" or similar
- Verify: Updates describe: multi-week plan structure, rotation state handling, preference application

**AC5**: README.md updated with new features
- Verify: `README.md` includes "Features" section mentioning multi-week meal planning
- Verify: Brief description of preferences and accompaniment settings

**AC6**: Screenshots added to user guide
- Verify: At least 3 screenshots in user guide (meal calendar, preferences form, shopping list)
- Verify: Screenshots captured via Playwright screenshot API (consistent styling)

**AC7**: Code comments added to complex algorithm functions
- Verify: `crates/meal-plan/src/algorithm.rs` includes doc comments for `generate_multi_week`, `apply_rotation_state`
- Verify: Comments explain algorithm logic (not just "what" but "why")

**AC8**: Deployment guide updated (migration steps, env vars)
- Verify: `docs/deployment.md` includes migration steps for Epic 10 schema changes
- Verify: New environment variables documented (if any added for preferences)

## Traceability Mapping

| Acceptance Criteria | Spec Section | Component/API | Test Coverage |
|---------------------|--------------|---------------|---------------|
| **Story 10.1 AC1**: Playwright config with auth fixtures | Testing Architecture | `e2e/fixtures/auth.ts` | E2E test suite setup verification |
| **Story 10.1 AC2**: Multi-week generation test | Workflow 1: E2E Flow | `POST /plan/generate-multi-week` | `meal-planning.spec.ts` |
| **Story 10.1 AC3**: Week navigation test | APIs: GET /plan?week | TwinSpark week navigation | `meal-planning.spec.ts` |
| **Story 10.1 AC4**: Single week regeneration test | APIs: POST /plan/regenerate-week | `crates/meal-plan` aggregate | `meal-planning.spec.ts` |
| **Story 10.1 AC5**: Future weeks regeneration test | APIs: POST /plan/regenerate-future | `crates/meal-plan` aggregate | `meal-planning.spec.ts` |
| **Story 10.1 AC6**: Preferences update test | APIs: POST /profile/meal-planning-preferences | `meal_plan_preferences` read model | `preferences.spec.ts` |
| **Story 10.1 AC7**: Recipe creation with accompaniments | APIs: POST /recipes | `crates/recipe` aggregate | `recipes.spec.ts` |
| **Story 10.1 AC8**: Shopping list specific week test | APIs: GET /shopping?week | `crates/shopping` module | `shopping.spec.ts` |
| **Story 10.1 AC9**: All tests pass in CI | Testing Architecture: CI/CD | GitHub Actions workflow | `.github/workflows/test.yml` |
| **Story 10.1 AC10**: Execution time <5 minutes | Performance: E2E Test Requirements | Parallel test workers | Playwright config (4 workers) |
| **Story 10.2 AC1**: 100 concurrent load test | Workflow 2: Performance Testing | k6 script configuration | `e2e/performance/load-test.js` |
| **Story 10.2 AC2**: P95 generation <5s | Performance: Benchmarks | `POST /plan/generate-multi-week` | k6 metrics validation |
| **Story 10.2 AC3**: P95 routes <500ms | Performance: Benchmarks | `GET /plan`, `GET /shopping` | k6 metrics validation |
| **Story 10.2 AC4**: No N+1 queries | Performance: Database Constraints | SQLite query optimization | SQL query log analysis |
| **Story 10.2 AC5**: Memory profiling | Performance: Memory Usage | Rust heap profiling | cargo-flamegraph output |
| **Story 10.2 AC6**: Regression tests in CI | Performance: CI/CD | GitHub Actions benchmark step | Baseline comparison script |
| **Story 10.2 AC7**: Optimization docs | Performance: Report Generation | `docs/performance-report.md` | Manual review if benchmarks fail |
| **Story 10.3 AC1**: Critical bugs fixed | Reliability: Graceful Errors | All components | GitHub Issues status |
| **Story 10.3 AC2**: Medium bugs triaged | Reliability: Bug Triage | All components | GitHub Issues labels |
| **Story 10.3 AC3**: Known issues documented | Documentation: Known Issues | `docs/known-issues.md` | Manual review |
| **Story 10.3 AC4**: Edge cases handled | Reliability: Edge Case Tests | All aggregates | Integration tests |
| **Story 10.3 AC5**: User-friendly errors | Security: Error Handling | Axum error responses | E2E error scenario tests |
| **Story 10.3 AC6**: Regression tests | Reliability: Test Isolation | All components | Unit/integration tests per bug |
| **Story 10.3 AC7**: Bug fix changelog | Documentation: CHANGELOG | `CHANGELOG.md` | Manual review |
| **Story 10.4 AC1**: User guide created | Documentation: User Guide | `docs/user-guide-meal-planning.md` | Manual review |
| **Story 10.4 AC2**: User guide coverage | Documentation: Feature Coverage | User guide sections | Manual review (completeness check) |
| **Story 10.4 AC3**: API docs updated | Documentation: API Docs | `docs/api/meal-planning-routes.md` | Manual review |
| **Story 10.4 AC4**: Architecture as-built notes | Documentation: Architecture Updates | `docs/solution-architecture.md` | Manual review |
| **Story 10.4 AC5**: README updated | Documentation: README | `README.md` | Manual review |
| **Story 10.4 AC6**: Screenshots added | Documentation: Visual Aids | Playwright screenshot API | User guide screenshot count |
| **Story 10.4 AC7**: Code comments | Documentation: Code Comments | `crates/meal-plan/src/algorithm.rs` | Manual review (rustdoc comments) |
| **Story 10.4 AC8**: Deployment guide updated | Documentation: Deployment | `docs/deployment.md` | Manual review |

## Risks, Assumptions, Open Questions

### Risks

**Risk 1**: E2E tests may be flaky due to async timing issues
- **Likelihood**: Medium
- **Impact**: High (blocks CI, delays deployment)
- **Mitigation**: Use `waitForSelector` instead of fixed delays, implement deterministic test data seeding, avoid `setTimeout` in tests
- **Contingency**: Rerun failed tests with video recording to identify root cause, implement retry logic only for infrastructure failures (not test failures)

**Risk 2**: Performance benchmarks may not meet targets on first attempt
- **Likelihood**: Medium
- **Impact**: Medium (requires optimization iteration)
- **Mitigation**: Profile early (Story 10.2), identify bottlenecks (N+1 queries, missing indexes), optimize before final validation
- **Contingency**: Document optimization recommendations, defer non-critical optimizations to post-MVP, validate that current performance acceptable for 10K user target

**Risk 3**: Critical bugs discovered late in testing cycle
- **Likelihood**: Low
- **Impact**: High (blocks deployment, requires rushed fixes)
- **Mitigation**: Continuous integration testing throughout Epics 6-9, encourage exploratory testing alongside E2E test development
- **Contingency**: Triage ruthlessly (fix critical, defer medium/low), implement hotfix process for showstoppers

**Risk 4**: Documentation may become outdated if implementation changes
- **Likelihood**: Medium
- **Impact**: Low (confusing for users, but not a blocker)
- **Mitigation**: Use Playwright screenshots (auto-update), write documentation in final sprint (after implementation frozen)
- **Contingency**: Schedule documentation review sprint post-deployment, accept minor inconsistencies in MVP

**Risk 5**: Load testing may reveal SQLite scaling limitations
- **Likelihood**: Low (architecture pre-validated for 10K users)
- **Impact**: High (requires database migration)
- **Mitigation**: Validate WAL mode, connection pooling, PRAGMA optimizations early, test with realistic data volume (100 users, 50 recipes each)
- **Contingency**: If SQLite insufficient, plan migration to PostgreSQL (evento supports both), estimate 2-week migration timeline

### Assumptions

**Assumption 1**: 100 concurrent users represent realistic peak load for MVP
- **Validation**: Calculate: 10,000 MAU * 5% concurrent = 500 users, 100 users = 20% of peak (sufficient safety margin)
- **If False**: Increase load test to 500 concurrent users, validate performance still meets targets

**Assumption 2**: Playwright test suite can complete in <5 minutes with parallelization
- **Validation**: Current estimate: 40 tests * 30s avg * (1/4 workers) = 5 minutes
- **If False**: Increase parallelization (8 workers), optimize slow tests, split into critical vs non-critical suites

**Assumption 3**: `unsafe_oneshot` is acceptable for test usage (not production)
- **Validation**: evento documentation confirms unsafe_oneshot is test-only pattern
- **If False**: Implement polling mechanism for projection completion in tests (increases test complexity)

**Assumption 4**: Manual accessibility testing (NVDA/VoiceOver) per sprint is sufficient
- **Validation**: WCAG 2.1 AA compliance does not require automated testing for MVP
- **If False**: Integrate axe-core into Playwright tests for automated accessibility checks

**Assumption 5**: Single dev/QA can complete Epic 10 in 2-week sprint
- **Validation**: Story point estimate: 10.1 (5 pts) + 10.2 (3 pts) + 10.3 (3 pts) + 10.4 (3 pts) = 14 pts (within 2-week sprint)
- **If False**: Prioritize Story 10.1 and 10.2 (critical path), defer 10.4 documentation to parallel effort

### Open Questions

**Question 1**: Should E2E tests run against local dev server or deployed staging environment?
- **Options**: (A) Local dev server (faster, isolated), (B) Staging environment (more realistic)
- **Recommendation**: Local dev server for CI, staging for pre-deployment validation
- **Decision Owner**: DevOps / QA Lead
- **Deadline**: Before Story 10.1 implementation

**Question 2**: What is acceptable code coverage threshold for Epic 10 (testing code itself)?
- **Context**: Tests are excluded from coverage calculation (Cargo.toml `[lib]`)
- **Recommendation**: E2E test fixtures should have 80% coverage (integration tests validate fixtures)
- **Decision Owner**: Tech Lead
- **Deadline**: Before Story 10.1 implementation

**Question 3**: Should performance regression tests block CI or just warn?
- **Options**: (A) Block CI (prevents regressions), (B) Warn only (allows acceptable slowdowns)
- **Recommendation**: Warn if P95 exceeds baseline by 10-20%, block if >20%
- **Decision Owner**: Performance Engineer
- **Deadline**: Before Story 10.2 implementation

**Question 4**: How to handle test data seeding for E2E tests (pre-generated vs dynamic)?
- **Options**: (A) Pre-generated JSON fixtures (fast, deterministic), (B) Dynamic via API (realistic, slower)
- **Recommendation**: Hybrid: Pre-generated user + recipes, dynamic meal plan generation
- **Decision Owner**: QA Lead
- **Deadline**: Before Story 10.1 implementation

## Test Strategy Summary

### Test Levels

**Level 1: Unit Tests** (TDD enforced)
- Scope: Domain aggregate logic in `crates/meal-plan`, `crates/recipe`, `crates/shopping`
- Framework: `cargo test --lib`
- Coverage: 80% minimum (enforced via cargo-tarpaulin in CI)
- Patterns: Pure function tests, evento command/event validation
- Example: Test `generate_multi_week` algorithm logic (rotation state, preference application)

**Level 2: Integration Tests** (HTTP routes + projections)
- Scope: Axum routes, evento projection handlers, database queries
- Framework: `cargo test --test '*'`
- Database: In-memory SQLite with WAL mode
- Key Pattern: `unsafe_oneshot` for deterministic projection testing
- Example: Test `POST /plan/generate-multi-week` → assert `meal_plans` table updated correctly

**Level 3: E2E Tests** (Critical user flows)
- Scope: Full browser automation with Playwright
- Framework: `@playwright/test` (TypeScript)
- Browsers: Chromium (primary), Firefox (secondary), WebKit (tertiary)
- Coverage: Multi-week generation, navigation, regeneration, preferences, accompaniments, shopping lists
- Example: Test complete flow: login → generate plan → navigate weeks → view shopping list

**Level 4: Performance Tests** (Load testing)
- Scope: Multi-user concurrent load validation
- Framework: k6 (JavaScript)
- Test Data: 100 virtual users, 50 recipes per user, 5-minute sustained load
- Metrics: P95 latencies, error rates, database query performance
- Example: 100 concurrent multi-week generation requests, validate P95 <5s

### Test Execution Strategy

**Local Development**:
- Unit tests: Run on save (cargo-watch)
- Integration tests: Run before commit
- E2E tests: Run selectively (critical paths only)
- Performance tests: Not run locally (requires dedicated environment)

**CI Pipeline** (GitHub Actions):
1. **Unit + Integration Tests** (parallel job 1): `cargo test` (all tests)
2. **E2E Tests** (parallel job 2): `npx playwright test` (4 workers)
3. **Code Coverage** (parallel job 3): `cargo tarpaulin --out Html`
4. **Performance Tests** (manual trigger): `k6 run load-test.js`

**Deployment Gates**:
- All unit + integration tests pass: Required for merge to main
- All E2E tests pass: Required for merge to main
- Code coverage ≥80%: Required for merge to main
- Performance benchmarks met: Required for production deployment
- All critical bugs fixed: Required for production deployment

### Coverage Goals

| Test Type | Target Coverage | Measurement | Status Tracking |
|-----------|-----------------|-------------|-----------------|
| Unit Tests | 80% line coverage | cargo-tarpaulin | CI enforcement (blocks merge if <80%) |
| Integration Tests | 100% route coverage | Manual audit | All meal planning routes tested |
| E2E Tests | 100% critical path coverage | Manual audit | All user stories have E2E test |
| Performance Tests | 100% benchmark coverage | Manual audit | All NFR performance targets validated |

### Test Data Management

**Unit Tests**: Inline test data (small, focused)
**Integration Tests**: Seeded via setup functions (10 recipes, 1 user)
**E2E Tests**: Fixtures in `e2e/fixtures/` (50 recipes, 10 users, deterministic)
**Performance Tests**: Generated via script (100 users, 50 recipes per user, randomized but deterministic seed)

### Bug Tracking and Regression Testing

**Process**:
1. Bug reported → Create GitHub Issue with label (critical/medium/low)
2. Bug reproduced → Write failing test (unit or integration level)
3. Bug fixed → Verify test passes
4. Regression test → Keep test in suite (prevents reoccurrence)

**Epic 10 Specific**:
- All bugs discovered during Story 10.3 must have regression tests (AC6)
- Critical bugs require integration or E2E test (unit test insufficient)
- Bug fix changelog maintained in `CHANGELOG.md` (AC7)

---

## Post-Review Follow-ups

This section captures action items identified during Senior Developer Reviews of Epic 10 stories.

### Story 10.1: End-to-End Testing with Playwright

**Initial Review Date**: 2025-10-27
**Final Review Date**: 2025-10-27
**Reviewer**: Jonathan (AI)
**Status**: ✅ **APPROVED FOR MERGE**

**Medium Severity Items** (Must fix before merge):

1. ✅ **Extract test credentials to environment variables**
   - **Issue**: Hardcoded test credentials (`test@example.com` / `password123`) in `e2e/fixtures/auth.setup.ts:22-23`
   - **Recommendation**: Use `process.env.TEST_USER_EMAIL` and `process.env.TEST_USER_PASSWORD`, document in `e2e/README.md`
   - **Owner**: Dev Team
   - **Resolution**: Implemented in `e2e/fixtures/auth.setup.ts:27-28`, documented in `e2e/README.md:14-17`

2. ✅ **Verify /health endpoint exists or implement**
   - **Issue**: CI workflow assumes `/health` endpoint exists (`.github/workflows/test.yml:92`) but not verified
   - **Recommendation**: Implement `GET /health` route in Axum router returning 200 OK `{"status": "ok"}`
   - **Owner**: Backend Dev
   - **Resolution**: Verified existing implementation at `src/routes/health.rs:28-30` with unit test coverage

3. ✅ **Move storageState config out of global use block**
   - **Issue**: Setup project configured to load `storageState` that it creates, causing fragile initialization
   - **Recommendation**: Move `storageState` config from global `use` block to individual browser projects only
   - **Owner**: QA/Dev Team
   - **Resolution**: Corrected in `e2e/playwright.config.ts:39,47,55,65,73,83` - each browser project now explicitly configures storageState

4. ✅ **Document and implement test database seeding strategy**
   - **Issue**: Tests assume test user exists but no seeding step in CI
   - **Recommendation**: Add CI seeding step before server startup, document in `e2e/README.md`
   - **Owner**: Backend Dev, QA
   - **Resolution**: Implemented in `.github/workflows/test.yml:87-102`, documented in `e2e/README.md:23-50`

**Low Severity Items** (Recommended improvements):

5. ✅ Add .auth directory creation in auth.setup.ts (prevents ENOENT errors) - **Resolved**: `e2e/fixtures/auth.setup.ts:41`
6. ✅ Add .gitignore entries for E2E artifacts (`e2e/fixtures/.auth/*.json`, test results) - **Resolved**: `.gitignore:31-33`
7. ✅ Replace `bc -l` with awk in CI coverage check for portability - **Resolved**: `.github/workflows/test.yml:189`
8. ✅ Add individual test timeout (60s) to Playwright config - **Resolved**: `e2e/playwright.config.ts:23`

**Final Verdict**: All 8 action items (4 medium, 4 low severity) have been successfully resolved. Code quality meets production standards with proper security practices, CI reliability improvements, and comprehensive documentation. Story approved for merge to main.


# imkitchen E2E Tests

End-to-end tests for imkitchen using Playwright, covering critical user flows for the enhanced meal planning system.

## Prerequisites

- Node.js 20+
- Playwright browsers installed (`npx playwright install`)
- Application running at `http://localhost:3000` (or set `BASE_URL` environment variable)
- Test database with schema and test user seeded

## Environment Variables

### Required for Authentication

- `TEST_USER_EMAIL` - Email for test user (default: `test@example.com`)
- `TEST_USER_PASSWORD` - Password for test user (default: `password123`)

### Optional Configuration

- `BASE_URL` - Application base URL (default: `http://localhost:3000`)

## Test Database Setup

E2E tests require a test database with:

1. **Schema**: All tables created via SQLx migrations
2. **Test User**: User with email matching `TEST_USER_EMAIL` and password matching `TEST_USER_PASSWORD`

### Local Development Setup

```bash
# 1. Run database migrations
sqlx database create
sqlx migrate run

# 2. Create test user (example using SQL)
sqlite3 imkitchen.db "INSERT INTO users (id, email, password_hash, created_at) VALUES ('test-user-id', 'test@example.com', '\$argon2id\$..hash..', datetime('now'));"

# Or use a seeding script if available:
cargo run --bin seed-test-db
```

### CI Environment

The CI workflow (`.github/workflows/test.yml`) includes a database seeding step that:
- Creates test database
- Runs migrations
- Seeds test user with credentials from secrets

## Running Tests

```bash
# Install dependencies
npm ci

# Install Playwright browsers
npx playwright install --with-deps

# Run all tests (4 parallel workers)
npm test

# Run tests in headed mode (see browser)
npm run test:headed

# Run specific browser only
npm run test:chromium

# Debug tests
npm run test:debug
```

## Test Structure

- `tests/` - Test specifications
  - `meal-planning.spec.ts` - Multi-week generation, navigation, regeneration
  - `preferences.spec.ts` - Meal planning preferences CRUD
  - `recipe.spec.ts` - Recipe creation with accompaniments
  - `shopping.spec.ts` - Shopping list access and week navigation

- `fixtures/` - Test fixtures and setup
  - `auth.setup.ts` - Authentication setup (runs once before all tests)
  - `auth.ts` - Authenticated page fixture
  - `recipes.ts` - 50 deterministic test recipes
  - `.auth/` - Stored authentication state (gitignored)

## Configuration

- `playwright.config.ts` - Main configuration
  - 4 parallel workers for <5min execution time
  - Cross-browser testing (Chromium, Firefox, WebKit, mobile devices)
  - Video recording on failures only
  - Auth storage at `./fixtures/.auth/user.json`

## Test Artifacts

- `test-results/` - Test execution artifacts (videos, traces)
- `playwright-report/` - HTML test report

**Note**: These directories are gitignored. CI uploads artifacts automatically on test failures.

## Troubleshooting

### Authentication Failures

If tests fail with login errors:
1. Verify test user exists in database with correct credentials
2. Check `TEST_USER_EMAIL` and `TEST_USER_PASSWORD` environment variables
3. Ensure `.auth/user.json` is being created in `fixtures/.auth/` directory

### Missing .auth Directory Error

If you see `ENOENT: no such file or directory` for `.auth/user.json`:
- The directory is created automatically by `auth.setup.ts`
- If error persists, manually create: `mkdir -p fixtures/.auth`

### Flaky Tests

Tests use `waitForSelector` and `waitForLoadState('networkidle')` for TwinSpark AJAX updates. If tests are flaky:
1. Check for race conditions in test assertions
2. Increase timeout in `playwright.config.ts` if needed (`timeout: 60000`)
3. Run specific test in headed mode to observe behavior

### CI Failures

If tests pass locally but fail in CI:
1. Verify test database seeding step runs before tests
2. Check health endpoint responds with 200 OK (`/health`)
3. Verify application starts successfully (check server logs)
4. Ensure all environment variables are set in CI secrets

## Architecture Notes

- Tests validate E2E user flows, not implementation details
- TwinSpark progressive enhancement: tests wait for DOM updates after AJAX requests
- Test isolation: each test uses fresh authenticated session, no shared state
- Deterministic test data: 50 recipes with fixed characteristics for reproducible results
- Parallel execution: tests run independently across 4 workers (test order doesn't matter)

## References

- [Playwright Documentation](https://playwright.dev/docs/intro)
- [Story 10.1 Technical Specification](/docs/tech-spec-epic-10.md)
- [Story Context](/docs/story-context-10.1.xml)

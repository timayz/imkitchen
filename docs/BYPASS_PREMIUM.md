# Bypass Premium Feature Flag

## Overview

The `bypass_premium` configuration flag allows you to disable premium restrictions for MVP testing, demos, and development environments. When enabled, all users are treated as premium users, bypassing recipe limits and other premium-only features.

## Use Cases

- **MVP Testing**: Test the full application without subscription setup
- **Demo Environments**: Show all features to potential customers/investors
- **Development**: Faster development without managing test subscriptions
- **CI/CD**: Integration tests run without premium restrictions

## Configuration

### Option 1: Configuration File

Edit `config/default.toml`:

```toml
[features]
bypass_premium = true
```

### Option 2: Environment Variable

```bash
export IMKITCHEN__FEATURES__BYPASS_PREMIUM=true
cargo run
```

Or in `.env`:

```env
IMKITCHEN__FEATURES__BYPASS_PREMIUM=true
```

### Option 3: Command Line (Development)

```bash
IMKITCHEN__FEATURES__BYPASS_PREMIUM=true cargo run
```

## What Gets Bypassed?

When `bypass_premium = true`, the following restrictions are removed:

### ✅ Recipe Limits
- **Free tier**: Limited to 10 recipes → **Bypassed**: Unlimited recipes
- Applies to:
  - Creating new recipes
  - Copying community recipes
  - Batch importing recipes

### ✅ Future Premium Features
As new premium features are added, this flag will bypass those as well.

## Implementation Details

### Architecture

The bypass flag flows through the application like this:

```
Config File/Env Var
    ↓
Config::load()
    ↓
AppState.bypass_premium
    ↓
Route Handlers (pass to domain commands)
    ↓
Domain Commands (check bypass before enforcing limits)
```

### Code Locations

1. **Config**: `src/config.rs`
   - `FeatureConfig` struct with `bypass_premium` field

2. **AppState**: `src/routes/auth.rs`
   - Added `bypass_premium: bool` field

3. **Main**: `src/main.rs`
   - Passes `config.features.bypass_premium` to AppState

4. **Domain Commands**: `crates/recipe/src/commands.rs`
   - `create_recipe()` - Added `bypass_premium` parameter
   - `copy_recipe()` - Added `bypass_premium` parameter
   - `batch_import_recipes()` - Added `bypass_premium` parameter
   - Check: `if !bypass_premium && user.tier != "premium"`

5. **Route Handlers**: `src/routes/recipes.rs`
   - Pass `state.bypass_premium` to all domain command calls

## Security Considerations

### ⚠️ Production Warning

**NEVER enable `bypass_premium` in production!**

```toml
# ❌ BAD - Don't do this in production
[features]
bypass_premium = true
```

This would give all users premium features for free, bypassing your business model.

### ✅ Recommended Production Config

```toml
[features]
bypass_premium = false  # or omit entirely (defaults to false)
```

### Environment-Specific Configuration

Use different config files per environment:

```bash
# Development
cargo run --config=config/development.toml

# Staging (bypass enabled for demos)
cargo run --config=config/staging.toml

# Production (bypass disabled)
cargo run --config=config/production.toml
```

## Testing

### Unit Tests

Tests bypass premium by default for simplicity:

```rust
// In src/lib.rs test setup
let state = AppState {
    // ...
    bypass_premium: true, // Tests bypass premium by default
};
```

### Integration Tests

Recipe domain tests can test both modes:

```rust
// Test free tier limit enforcement
let result = create_recipe(cmd, &user_id, &executor, &pool, false).await;
assert!(matches!(result, Err(RecipeError::RecipeLimitReached)));

// Test bypass mode
let result = create_recipe(cmd, &user_id, &executor, &pool, true).await;
assert!(result.is_ok());
```

## Monitoring

When bypass is enabled, premium restrictions are still logged but not enforced:

```
Premium users bypass all limits (or MVP/demo mode bypass)
```

Look for this log message to verify bypass mode is active.

## FAQ

### Q: Does this affect Stripe integration?
**A:** No. Users can still subscribe to premium via Stripe. The bypass only affects enforcement of limits, not the subscription system itself.

### Q: Can I bypass only for specific users?
**A:** Not with this flag. It's a global setting. To give premium to specific users, use the Stripe subscription system and set their tier to "premium" in the database.

### Q: Does this work in tests?
**A:** Yes! Tests set `bypass_premium = true` by default in `src/lib.rs` for easier testing.

### Q: What happens if I enable bypass with active paid users?
**A:** Paid users continue to work normally (they're already premium). Free users get premium features without payment.

### Q: How do I check if bypass is enabled at runtime?
**A:** Check the config at startup. The flag is logged during application initialization.

## Examples

### Development with Bypass

```bash
# Start app with bypass enabled
IMKITCHEN__FEATURES__BYPASS_PREMIUM=true cargo run

# Create 100 recipes as a free user (normally limited to 10)
# ✅ All succeed
```

### Production without Bypass

```bash
# Start app with bypass disabled (default)
cargo run

# Create 11th recipe as a free user
# ❌ Returns RecipeLimitReached error
```

### Demo Environment

```toml
# config/demo.toml
[features]
bypass_premium = true

[stripe]
# Use test keys for demo
secret_key = "sk_test_..."
```

```bash
cargo run --config=config/demo.toml
```

## Migration Guide

If you have existing code that checks premium status:

### Before (Direct Checks)
```rust
if user.tier != "premium" {
    // enforce limit
}
```

### After (With Bypass Support)
```rust
if !bypass_premium && user.tier != "premium" {
    // enforce limit
}
```

All recipe domain commands have been updated with this pattern.

---

**Last Updated**: 2025-10-23
**Version**: 0.6.0

# Coding Standards

## Critical Fullstack Rules

- **Type Sharing:** Always define shared types in `src/types/` and import from there - prevents frontend/backend interface mismatches
- **API Client Usage:** Never make direct fetch calls - use the centralized API client service for consistent error handling and authentication
- **Environment Variables:** Access only through config objects in `src/lib/config.ts`, never process.env directly - enables proper validation and fallbacks
- **Error Handling:** All API routes must use the standard error handler middleware for consistent error responses and logging
- **Database Access:** Only access database through service layer, never direct Prisma calls in API routes - enables caching and business logic separation
- **Authentication Checks:** Always verify authentication in API routes using middleware, never skip auth validation
- **Voice Command Structure:** Voice commands must follow the predefined schema in `src/types/voice.ts` for consistency
- **Cache Invalidation:** Update relevant cache keys when data changes, use the cache service for coordinated invalidation
- **File Uploads:** Use the storage abstraction layer, never direct cloud provider SDKs - maintains vendor independence
- **State Updates:** Use proper state management patterns, never directly mutate state objects

## Naming Conventions

| Element               | Frontend             | Backend              | Example                   |
| --------------------- | -------------------- | -------------------- | ------------------------- |
| Components            | PascalCase           | -                    | `InventoryList.tsx`       |
| Hooks                 | camelCase with 'use' | -                    | `useInventoryItems.ts`    |
| API Routes            | -                    | kebab-case           | `/api/meal-plans`         |
| Database Tables       | -                    | snake_case           | `inventory_items`         |
| Services              | PascalCase + Service | PascalCase + Service | `InventoryService`        |
| Types/Interfaces      | PascalCase           | PascalCase           | `InventoryItem`           |
| Constants             | SCREAMING_SNAKE_CASE | SCREAMING_SNAKE_CASE | `MAX_RECIPE_TITLE_LENGTH` |
| Environment Variables | SCREAMING_SNAKE_CASE | SCREAMING_SNAKE_CASE | `DATABASE_URL`            |

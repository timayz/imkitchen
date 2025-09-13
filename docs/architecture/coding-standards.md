# Coding Standards

## Critical Fullstack Rules

- **Type Sharing:** Always define shared types in `shared/types` and import consistently across frontend and backend
- **Error Handling:** All API endpoints must use the unified error handling pattern with proper HTTP status codes
- **Database Migrations:** Never modify existing migrations; always create new migration files for schema changes
- **Authentication:** Never bypass authentication middleware; all protected routes must validate JWT tokens
- **Caching Strategy:** Always implement cache invalidation when modifying data that affects cached responses
- **Offline Support:** Critical user flows must degrade gracefully when offline; sync changes when connectivity restored
- **Template Security:** Never use raw HTML insertion in Askama templates; rely on built-in escaping
- **Environment Variables:** Access configuration only through the config module, never directly from `std::env`

## Naming Conventions

| Element | Frontend | Backend | Example |
|---------|----------|---------|---------|
| Templates | snake_case.html | - | `recipe_card.html` |
| Route Handlers | snake_case | snake_case | `create_recipe_handler` |
| API Endpoints | kebab-case | - | `/meal-plans/{id}/confirm` |
| Database Tables | snake_case | snake_case | `cooking_sessions` |
| Struct Names | PascalCase | PascalCase | `CookingSession` |
| Functions | snake_case | snake_case | `parse_recipe_url` |

# Coding Standards

## Critical Fullstack Rules

- **Type Safety Across Boundaries:** All data models must be defined in shared crate with validation
- **Event Sourcing Consistency:** Never mutate aggregates directly - only through Evento create() builder pattern
- **Template Security:** Always use Askama filters for user input, never raw HTML injection
- **TwinSpark Patterns:** All dynamic interactions must use ts-* attributes, no custom JavaScript
- **Error Handling:** All handlers must return Result<T, AppError> with proper error conversion
- **Validation First:** Input validation with validator before any business logic execution
- **Testing Required:** Minimum 90% test coverage, TDD for all new features
- **Crate Boundaries:** Domain crates cannot depend on web crate, only shared types

## Naming Conventions

| Element | Frontend | Backend | Example |
|---------|----------|---------|---------|
| Templates | snake_case.html | - | `weekly_calendar.html` |
| Handlers | snake_case | snake_case | `generate_meal_plan_handler` |
| Commands | PascalCase | PascalCase | `GenerateMealPlanCommand` |
| Events | PascalCase | PascalCase | `MealPlanGeneratedEvent` |
| Crate Names | kebab-case | kebab-case | `imkitchen-meal-planning` |
| Database Tables | snake_case | snake_case | `meal_plans` |

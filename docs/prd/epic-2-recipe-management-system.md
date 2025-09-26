# Epic 2: Recipe Management System

Create a comprehensive recipe management system that enables users to discover, store, organize, and rate recipes within their personal collections. This epic delivers the content foundation necessary for the intelligent meal planning system, including community-driven recipe discovery and quality assessment.

## Story 2.1: Recipe Database and CRUD Operations

As a user,
I want to add, view, edit, and organize recipes,
so that I can build my personal recipe collection for meal planning.

### Acceptance Criteria (TDD Required)

**ALL criteria must be implemented using TDD + DDD + CQRS + ES + Validation + Askama methodology:**

1. **DDD Recipe Domain:** Recipe aggregate with validated Ingredient, Instruction, Category, Difficulty value objects using validator derive macros
2. **Input Validation:** Recipe title length (1-200 chars), ingredient quantities (positive numbers), instruction steps (non-empty), prep/cook times (positive integers)
3. **Evento Event Sourcing:** RecipeCreated, RecipeUpdated, RecipeDeleted, IngredientAdded, InstructionModified events with Evento serialization and audit trail
4. **Evento Commands:** CreateRecipeCommand, UpdateRecipeCommand, DeleteRecipeCommand processed by Evento command handlers with validation
5. **Evento Queries:** RecipeByIdQuery, RecipeSearchQuery, RecipesByUserQuery handled by Evento query handlers with projection optimization
6. **Askama + Tailwind Templates:** RecipeForm.html, RecipeDetail.html, RecipeList.html, IngredientEditor.html with Tailwind styling and type-safe recipe data binding
7. **TwinSpark Recipe Management:** Recipe forms with `ts-req="/recipes"` and `ts-target="#recipe-list"` → Axum handlers → validate commands → render Askama fragments
8. **JavaScript-Free Recipe Editor:** Ingredient additions, instruction editing via TwinSpark attributes without custom JavaScript
9. **Domain Services:** RecipeDifficultyCalculator, IngredientParser, NutritionalCalculator with validated inputs encapsulate complex business logic
10. **Evento Projections:** RecipeListView, RecipeDetailView, RecipeSearchIndex maintained by Evento projection builders and rendered through Askama templates
11. **TwinSpark + Tailwind Examples:** Recipe search with `border-2 border-gray-300 focus:border-blue-500 rounded-lg px-4 py-2` styling and live search functionality
12. **JavaScript-Free Interactions:** Recipe ratings, favoriting, sharing all via TwinSpark HTML attributes and server responses
13. **Evento Version History:** Recipe versioning through Evento event replay - reconstruct any historical state from event streams with Askama rendering
14. **Evento Soft Deletion:** RecipeArchivedEvent and RecipeRestoredEvent handled by Evento with automatic projection updates and template management
15. **TDD Template Testing:** Write template tests first covering recipe forms, ingredient editors, instruction builders, and search results
16. **Evento Search Projections:** Full-text search index built from Evento event streams with real-time updates and Askama-rendered search results
17. **Crate Isolation:** Recipe crate independent of presentation concerns, web crate depends on recipe crate types and interfaces"

## Story 2.2: Personal Recipe Collections and Favorites

As a user,
I want to organize recipes into custom collections and mark favorites,
so that I can easily access recipes that match my preferences and occasions.

### Acceptance Criteria

1. Users can create named recipe collections (e.g., "Quick Weeknight Dinners", "Holiday Meals")
2. Recipes can be added to multiple collections simultaneously
3. Favorite recipe system with quick access from main navigation
4. Collection management includes rename, delete, and reorder capabilities
5. Recipe collections can be set as private or shared with community
6. Collection filtering and sorting by date added, rating, prep time, or difficulty
7. Bulk operations allow moving multiple recipes between collections efficiently
8. Import functionality accepts recipes from URLs or common recipe formats

## Story 2.3: Community Recipe Rating and Review System

As a user,
I want to rate and review recipes from other community members,
so that I can discover high-quality recipes and share my cooking experiences.

### Acceptance Criteria

1. 5-star rating system for recipes with aggregate scoring and review count display
2. Written reviews with optional photos of finished dishes and modifications made
3. Review helpfulness voting system to surface most useful community feedback
4. Recipe rating averages update in real-time with proper statistical weighting
5. Users can edit or delete their own ratings and reviews with change history
6. Review moderation prevents spam and inappropriate content through automated filtering
7. Recipe creators receive notifications of new ratings and reviews with privacy controls
8. Rating distribution visualization shows community opinion spread

## Story 2.4: Recipe Discovery and Browsing

As a user,
I want to discover new recipes from the community,
so that I can expand my cooking repertoire beyond my current collection.

### Acceptance Criteria

1. Recipe browse page with grid/list view toggle and infinite scroll loading
2. Filtering options include rating, difficulty, prep time, dietary restrictions, and meal type
3. Sorting capabilities by newest, most popular, highest rated, and quickest prep time
4. Search functionality with autocomplete suggestions and typo tolerance
5. Recipe detail view shows full recipe information, community ratings, and related recipes
6. "Similar recipes" recommendations based on ingredients and cooking techniques
7. Trending recipes section highlights currently popular community choices
8. Random recipe suggestion feature for culinary exploration and inspiration

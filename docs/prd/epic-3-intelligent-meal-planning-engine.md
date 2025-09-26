# Epic 3: Intelligent Meal Planning Engine

Implement the core intelligent meal planning system with automated weekly meal generation, recipe rotation logic, and visual calendar interface. This epic delivers the primary value proposition of imkitchen by solving timing complexity through intelligent automation.

## Story 3.1: "Fill My Week" Automated Meal Planning

As a user,
I want to automatically generate a complete weekly meal plan,
so that I can eliminate the mental overhead of daily meal decisions.

### Acceptance Criteria

1. "Fill My Week" button generates complete 7-day meal plan in under 3 seconds
2. Algorithm selects recipes from user's favorites and collections based on preferences
3. Meal plan considers dietary restrictions, family size, and available cooking time
4. No recipe is repeated until all recipes in active collections have been used once
5. Difficulty distribution balances complex and simple meals throughout the week
6. Weekend meals can include more complex recipes with longer preparation times
7. Generated plan can be regenerated with different options while maintaining preferences
8. Plan generation respects user-defined meal exclusions and scheduling constraints
9. **Meal Planning Crate:** MealPlan aggregate in imkitchen-meal-planning crate with WeeklySchedule, MealSlot, Recipe references
10. **Evento Event Sourcing:** MealPlanGenerated, MealScheduled events in meal-planning crate managed by Evento with persistence
11. **Evento Commands in Meal Planning Crate:** GenerateMealPlanCommand, ScheduleMealCommand in meal-planning crate with Evento handlers
12. **Evento Queries in Meal Planning Crate:** WeeklyMealPlanQuery, MealCalendarQuery in meal-planning crate with optimized projections
13. **Inter-Crate Dependencies:** Meal-planning crate depends on recipe crate for Recipe types and user crate for preferences
14. **Tailwind Calendar Templates:** MealCalendar.html, WeeklyPlan.html, MealSlot.html with Tailwind grid system and responsive design classes
15. **Cross-Crate Planning Flow:** "Fill My Week" button with `ts-req` attribute → meal-planning crate commands → recipe crate queries → web library fragments
16. **JavaScript-Free Calendar:** Meal calendar interactions (drag-drop, rescheduling) handled via TwinSpark HTML attributes and server responses
17. **CLI Server Integration:** `imkitchen web start` initializes meal planning server with all domain crate dependencies
16. **Domain Services in Meal Planning Crate:** MealPlanningAlgorithm, RecipeRotationEngine, SchedulingOptimizer in meal-planning crate
17. **Evento Projections in Meal Planning Crate:** WeeklyCalendarView, MealScheduleView maintained by meal-planning projection builders
18. **Cross-Crate Event Communication:** Meal planning events propagated to shopping crate for automatic list generation
19. **TDD Crate Testing:** Meal-planning crate algorithm tests separate from web crate template integration tests
20. **Crate Isolation:** Meal-planning crate independent of presentation, web crate depends on meal-planning interfaces

## Story 3.2: Visual Weekly Meal Calendar

As a user,
I want to view my weekly meal plan in a visual calendar format,
so that I can easily understand my cooking schedule and make adjustments.

### Acceptance Criteria

1. Calendar displays 7 days with breakfast, lunch, and dinner slots clearly differentiated
2. Each meal slot shows recipe name, prep time, and color-coded difficulty indicators
3. Advanced preparation requirements are highlighted with timing alerts (e.g., "marinate overnight")
4. Drag-and-drop functionality allows easy meal rescheduling within the week
5. Calendar adapts to mobile screens with swipe navigation between days
6. Empty meal slots display "+" button for manual recipe selection
7. Weekend/weekday styling differences reflect different cooking time availability
8. Calendar can be viewed in daily detail mode with expanded recipe information

## Story 3.3: Recipe Rotation and Variety Management

As a user,
I want the system to ensure recipe variety by rotating through my entire collection,
so that I don't get stuck cooking the same meals repeatedly.

### Acceptance Criteria

1. Rotation algorithm tracks which recipes have been cooked from each collection
2. "Recently cooked" indicator prevents recipes from being selected too frequently
3. Variety scoring considers ingredient overlap to avoid repetitive meals
4. Seasonal recipe suggestions promote timely ingredients and holiday themes
5. User can manually mark recipes as "cooked" to update rotation status
6. Collections can be set as active/inactive for meal planning inclusion
7. Rotation reset option allows starting fresh cycle through all recipes
8. Cooking history displays statistics on recipe usage and favorite patterns

## Story 3.4: Real-Time Meal Plan Adaptation

As a user,
I want my meal plan to adapt when life disruptions occur,
so that I can maintain organized cooking despite schedule changes.

### Acceptance Criteria

1. "Reschedule meal" option automatically suggests alternative recipes with similar prep requirements
2. Emergency meal substitution provides quick 15-30 minute recipe alternatives
3. Ingredient freshness tracking adjusts meal order when perishables need immediate use
4. Weather integration suggests appropriate comfort foods during cold/hot periods
5. Energy level adjustment converts complex meals to simpler alternatives on busy days
6. Shopping list automatically updates when meal plans change
7. Family schedule integration avoids complex meals during busy weeknight periods
8. Plan disruption learning improves future meal scheduling through pattern recognition

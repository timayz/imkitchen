# Epic 2: Automated Meal Planning Engine

**Epic Goal:** Implement the core "Fill My Week" automation and visual meal calendar that solves the primary user pain point of decision fatigue while providing intelligent meal selection from user's curated recipe collection.

## Story 2.1: Basic Meal Calendar Interface

As a home cook,
I want to see my weekly meals in a visual calendar format,
so that I can understand my meal plan at a glance and plan my week effectively.

### Acceptance Criteria
1. Weekly calendar view displays 7 days with breakfast/lunch/dinner slots in responsive grid layout
2. Calendar navigation allows moving between weeks with clear date indicators
3. Empty meal slots display placeholder content inviting meal assignment
4. Calendar adapts to mobile screen sizes with touch-friendly interaction areas
5. Today's date is visually highlighted with distinct styling
6. Calendar loads quickly (<500ms) and handles week transitions smoothly
7. Accessibility features support keyboard navigation and screen readers

## Story 2.2: Manual Meal Assignment

As a home cook,
I want to assign recipes to specific meal slots in my calendar,
so that I can manually plan my meals before using automation features.

### Acceptance Criteria
1. Recipe selection modal allows browsing user's recipe collection with search functionality
2. Drag-and-drop interface enables moving recipes to calendar meal slots
3. Alternative tap-to-assign workflow works on mobile devices without drag capability
4. Assigned meals display recipe name, prep time, and visual thumbnail in calendar slot
5. Meal assignments persist in database and reload correctly on page refresh
6. Validation prevents assigning same recipe multiple times within a week
7. Clear feedback confirms successful meal assignment with visual updates

## Story 2.3: "Fill My Week" Automation Algorithm

As a busy home cook,
I want one-tap automation to fill my weekly meal plan,
so that I can eliminate decision fatigue while ensuring recipe variety.

### Acceptance Criteria
1. "Fill My Week" button triggers algorithm that selects from user's recipe collection
2. Algorithm ensures no duplicate recipes within the same week
3. Meal selection considers basic constraints like recipe complexity distribution
4. Algorithm fills only empty meal slots, preserving manually assigned meals
5. Selection logic rotates through entire recipe collection before repeating recipes
6. Generated meal plan displays immediately in calendar with clear visual feedback
7. Algorithm performance completes selection within 2 seconds for collections up to 100 recipes

## Story 2.4: Meal Plan Editing and Rescheduling

As a home cook,
I want to easily modify my generated meal plan,
so that I can adapt to schedule changes and personal preferences.

### Acceptance Criteria
1. Drag-and-drop rescheduling moves meals between calendar slots with visual feedback
2. Meal removal functionality clears slots and returns recipes to available pool
3. Replace meal option opens recipe selection for direct substitution
4. Undo functionality reverses recent meal plan changes
5. Bulk operations allow clearing entire days or meal types (all lunches, etc.)
6. Changes save automatically with visual confirmation of successful updates
7. Mobile-optimized editing workflow accommodates touch interactions and smaller screens

## Story 2.5: Recipe Collection Management

As a home cook,
I want to curate my recipe collection for meal planning,
so that automation only selects from recipes I actually want to cook.

### Acceptance Criteria
1. Recipe collection page displays all user recipes with inclusion toggles for meal planning
2. Bulk selection tools enable quick inclusion/exclusion of multiple recipes
3. Collection management includes categories or tags for organizing recipes by meal type
4. Search and filter functionality helps users find specific recipes to include/exclude
5. Collection statistics show total included recipes and estimated weeks of variety
6. Import functionality allows adding recipes from public database to personal collection
7. Collection changes immediately affect "Fill My Week" algorithm behavior

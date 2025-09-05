# Epic 2: Automated Meal Planning Engine  

**Expanded Goal:** Implement the core "Fill My Week" functionality with intelligent rotation logic that automatically generates weekly meal plans from user's recipe collection. This epic delivers the primary value proposition of eliminating meal planning cognitive overhead while ensuring users experience their full recipe variety through systematic rotation.

## Story 2.1: Recipe Rotation Algorithm Core
**As a** user with multiple favorite recipes,  
**I want** the system to track which recipes I've used recently,  
**so that** automated meal planning cycles through my full collection without repetition.

### Acceptance Criteria
1. Database schema tracking recipe usage history per user
2. Rotation algorithm ensuring no recipe repeats until all favorites are used
3. Reset mechanism when full collection has been cycled through
4. Manual recipe exclusion option for dietary changes or dislikes
5. Rotation status visibility showing progress through recipe collection
6. Algorithm performance under 500ms for collections up to 100 recipes

## Story 2.2: "Fill My Week" Button Implementation
**As a** busy home cook,  
**I want** to tap one button and receive a complete weekly meal plan,  
**so that** I can eliminate meal planning decision fatigue entirely.

### Acceptance Criteria
1. Prominent "Fill My Week" button on main interface
2. Automatic generation of 7 days of meals (breakfast, lunch, dinner)
3. Integration with rotation algorithm preventing recipe duplication
4. Sub-2-second meal plan generation meeting performance requirements
5. Visual feedback during generation with progress indicators
6. Generated plan immediately accessible and editable

## Story 2.3: Dietary Restriction and Preference Filtering
**As a** user with dietary restrictions,  
**I want** automated meal plans to respect my food allergies and preferences,  
**so that** generated meals are always safe and appealing to eat.

### Acceptance Criteria
1. User profile settings for common allergies (nuts, dairy, gluten, etc.)
2. Preference settings for dietary styles (vegetarian, keto, etc.)
3. Automatic filtering of incompatible recipes during meal generation
4. Warning system if insufficient compatible recipes exist for full week
5. Manual override option for specific restriction exceptions
6. Dietary restriction indicators visible on recipe details

## Story 2.4: Basic Meal Plan Editing and Regeneration
**As a** user reviewing my generated meal plan,  
**I want** to swap individual meals or regenerate specific days,  
**so that** I can customize the automated plan when needed.

### Acceptance Criteria
1. Individual meal slot editing with recipe substitution
2. Single day regeneration maintaining weekly rotation logic
3. Full week regeneration option preserving dietary restrictions
4. Undo functionality for recent meal plan changes
5. Change tracking showing modified versus auto-generated meals
6. Seamless integration with shopping list updates after edits

## Story 2.5: Meal Plan Persistence and History
**As a** user who finds successful meal combinations,  
**I want** to save and review previous meal plans,  
**so that** I can repeat planning approaches that worked well for my schedule.

### Acceptance Criteria
1. Automatic saving of all generated meal plans with timestamps
2. Meal plan history view with week-by-week navigation
3. "Repeat This Week" functionality for successful meal combinations
4. Meal plan favoriting and rating for future reference
5. Search functionality within meal plan history
6. Export option for sharing successful meal plans
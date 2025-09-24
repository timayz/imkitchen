# AI UI Generation Prompts for imkitchen

## Overview

This document contains optimized prompts for AI-powered frontend development tools (v0, Lovable.ai, etc.) to generate imkitchen UI components based on our comprehensive UI/UX specification and Rust fullstack architecture.

## Foundational Context Prompt

Use this as the opening context for all AI UI generation sessions:

```
You are building components for imkitchen, an intelligent meal planning platform that helps home cooks access their full recipe repertoire through automated weekly meal scheduling.

TECH STACK:
- Backend: Rust with Axum 0.8+ web framework
- Frontend: Server-side rendered HTML with Askama 0.14+ templates  
- UI Reactivity: twinspark-js for progressive enhancement
- Styling: Tailwind CSS with mobile-first approach
- Database: SQLite3 with SQLx 0.8+
- Deployment: Single Rust binary PWA (Progressive Web App)

DESIGN SYSTEM:
- Primary Color: #2D5A27 (forest green)
- Secondary Color: #8FBC8F (light green) 
- Accent Color: #FF6B47 (warm orange)
- Typography: Inter font family
- Spacing: 8px base unit system
- Target: WCAG AA accessibility compliance
- Focus: Kitchen environment optimization (large touch targets, high contrast)

USER CONTEXT: 
- Primary users are working parents (28-45) planning meals on mobile devices in kitchen environments
- Core user goal: Eliminate meal planning complexity while accessing 3x more recipe variety
- Key interaction: "Fill My Week" button that generates complete weekly meal plans in <3 seconds
```

## Component Generation Prompts

### 1. Weekly Meal Calendar Dashboard

```
HIGH-LEVEL GOAL:
Create a mobile-first weekly meal calendar dashboard that serves as the primary interface for viewing and managing weekly meal plans with one-touch automation.

DETAILED STEP-BY-STEP INSTRUCTIONS:
1. Create a responsive calendar grid component showing 7 days (current week)
2. Each day displays 3 meal slots: breakfast, lunch, dinner
3. Include prominent "Fill My Week" floating action button (FAB) using primary color #2D5A27
4. Add week navigation with left/right arrows and current week indicator
5. Implement color-coded meal complexity indicators:
   - Green (#4CAF50): Easy meals (≤30 min prep)
   - Yellow (#FF9800): Medium meals (31-60 min prep)  
   - Red (#FF6B47): Complex meals (>60 min prep)
6. Add shopping list quick access button with badge showing item count
7. Include swipe gesture indicators for week navigation
8. Show prep reminder badges on meals requiring advance preparation

CODE EXAMPLES & CONSTRAINTS:
- Use CSS Grid for calendar layout: `grid-template-columns: repeat(7, 1fr)`
- Minimum touch targets: 44x44px for all interactive elements
- FAB positioning: `fixed bottom-6 right-6` with `z-index: 50`
- Meal slot structure: 
  ```html
  <div class="meal-slot bg-white rounded-lg p-3 border-2 border-gray-100">
    <h4 class="text-sm font-medium text-gray-600">Breakfast</h4>
    <p class="text-base font-semibold">Recipe Name</p>
    <span class="inline-block w-3 h-3 rounded-full bg-green-500 mt-1"></span>
  </div>
  ```
- Use Tailwind responsive classes: `sm:`, `md:`, `lg:` for desktop adaptations
- DO NOT include actual recipe data - use placeholder content
- DO NOT add JavaScript functionality - focus on HTML/CSS structure

STRICT SCOPE:
Create only the calendar dashboard component HTML with Tailwind classes. Do NOT create navigation components, authentication, or data fetching logic. Focus on the visual layout and responsive design only.
```

### 2. Recipe Discovery Interface

```
HIGH-LEVEL GOAL:
Build a mobile-optimized recipe discovery interface with search, filtering, and community-driven recipe browsing that encourages recipe collection expansion.

DETAILED STEP-BY-STEP INSTRUCTIONS:  
1. Create search bar component with rounded design and search icon
2. Add filter chip row for: dietary restrictions, prep time, difficulty level, meal type
3. Design recipe card grid layout with 2 columns on mobile, 3-4 on desktop
4. Each recipe card includes: large image, title, star rating, prep time, difficulty badge
5. Add "heart" favorite button overlay on recipe images
6. Include trending recipes section above main grid
7. Implement infinite scroll placeholder (visual loading indicators)
8. Add floating "Create Recipe" button for community contributions
9. Design empty state for no search results with helpful suggestions

CODE EXAMPLES & CONSTRAINTS:
- Search bar structure:
  ```html
  <div class="relative mb-4">
    <input type="search" class="w-full pl-10 pr-4 py-3 border border-gray-200 rounded-xl bg-gray-50" placeholder="Search recipes...">
    <svg class="absolute left-3 top-3 h-5 w-5 text-gray-400"><!-- search icon --></svg>
  </div>
  ```
- Recipe card grid: `grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-4 gap-4`
- Recipe card aspect ratio: `aspect-square` for images
- Star rating display: Use filled/empty star SVGs, show average rating number
- Filter chips: `inline-flex items-center px-3 py-1 rounded-full text-sm bg-gray-100 border border-gray-200`
- Heart button: Absolute positioned `top-2 right-2` with semi-transparent background
- DO NOT implement actual search functionality or data fetching
- DO NOT add complex JavaScript interactions beyond basic CSS hover states

STRICT SCOPE:
Create recipe discovery page layout with search bar, filters, and recipe card grid. Include placeholder content only. Do NOT create recipe detail views or navigation components.
```

### 3. Shopping List Interface

```  
HIGH-LEVEL GOAL:
Design a family-friendly shopping list interface with store section organization, sharing capabilities, and check-off functionality optimized for grocery shopping workflow.

DETAILED STEP-BY-STEP INSTRUCTIONS:
1. Create header with list title, sharing button, and "Add Item" quick action
2. Organize ingredients by store sections: Produce, Dairy, Meat, Pantry, Frozen
3. Each section should be collapsible with item count badge
4. Design checkable list items with quantity, item name, and optional recipe context
5. Add swipe-to-delete gesture visual indicators
6. Include family sharing status indicators (who added each item)
7. Show estimated total cost at bottom
8. Add export options: email, text, shopping apps
9. Design completed items section (checked off) at bottom
10. Include "Clear Completed" bulk action

CODE EXAMPLES & CONSTRAINTS:
- Section headers:
  ```html
  <div class="flex items-center justify-between py-3 px-4 bg-gray-50 border-b">
    <h3 class="font-semibold text-gray-800">Produce</h3>
    <span class="bg-gray-200 text-gray-600 px-2 py-1 rounded-full text-xs">5 items</span>
  </div>
  ```
- Shopping list items:
  ```html
  <div class="flex items-center py-3 px-4 border-b border-gray-100">
    <input type="checkbox" class="h-5 w-5 text-green-600 rounded mr-3">
    <div class="flex-1">
      <p class="font-medium">2 lbs Ground Beef</p>
      <p class="text-sm text-gray-500">for Beef Tacos</p>
    </div>
    <span class="text-gray-400 text-sm">$8.99</span>
  </div>
  ```
- Sharing button: Use share icon with `bg-blue-500 text-white` styling
- Collapsible sections: Use chevron down/up icons, `transition-transform duration-200`
- Total cost styling: Sticky bottom bar with prominent text
- DO NOT implement actual sharing functionality or data persistence
- DO NOT add complex state management for check-off functionality

STRICT SCOPE:  
Create shopping list page layout with section organization and item structure. Use placeholder data for ingredients and costs. Do NOT implement sharing APIs or persistent storage.
```

### 4. Recipe Detail View

```
HIGH-LEVEL GOAL:
Create an immersive recipe detail view optimized for kitchen use with large text, clear instructions, and community interaction features.

DETAILED STEP-BY-STEP INSTRUCTIONS:
1. Design hero section with large recipe image, title, and key metadata (prep/cook time, difficulty, servings)
2. Add action button row: Save to Collection, Rate Recipe, Share, Start Cooking
3. Create tabbed interface: Ingredients, Instructions, Reviews, Nutrition
4. Design ingredients list with checkable items and quantity adjustment controls
5. Format instructions as numbered steps with timing information highlighted
6. Include community reviews section with 5-star ratings and written feedback
7. Add "Similar Recipes" recommendation section at bottom
8. Design prep timeline visual for complex recipes (marinate overnight, etc.)
9. Include cooking mode toggle for distraction-free step-by-step view

CODE EXAMPLES & CONSTRAINTS:
- Hero section layout:
  ```html
  <div class="relative">
    <img src="recipe-image.jpg" alt="Recipe" class="w-full h-64 object-cover rounded-b-xl">
    <div class="absolute bottom-0 left-0 right-0 bg-gradient-to-t from-black/60 to-transparent p-6">
      <h1 class="text-white text-2xl font-bold">Recipe Title</h1>
      <div class="flex items-center mt-2 text-white/90 text-sm">
        <span>30 min</span> <span class="mx-2">•</span> <span>4 servings</span> <span class="mx-2">•</span> <span>Medium</span>
      </div>
    </div>
  </div>
  ```
- Tab navigation: `flex border-b border-gray-200` with active state styling
- Ingredient items: Checkbox + quantity + ingredient name with `flex items-center` layout
- Instruction steps: Large step numbers in circles, clear typography hierarchy
- Star ratings: Use filled star SVGs with `text-yellow-400` color
- Cooking mode: Full-screen overlay with `fixed inset-0 bg-white z-50`
- DO NOT implement actual tab switching logic or cooking timer functionality
- DO NOT add recipe editing capabilities or complex form interactions

STRICT SCOPE:
Create recipe detail page layout with hero image, metadata, tabs, and content sections. Use placeholder recipe data. Do NOT implement interactive functionality like timers or tab switching.
```

### 5. User Profile & Settings Interface

```
HIGH-LEVEL GOAL:
Design a comprehensive user profile and settings interface that manages dietary preferences, family configuration, and meal planning optimization parameters.

DETAILED STEP-BY-STEP INSTRUCTIONS:
1. Create profile header with user avatar, name, and cooking statistics
2. Design settings sections: Personal Info, Dietary Preferences, Family Settings, Notifications
3. Add dietary restriction checkboxes: vegetarian, vegan, gluten-free, dairy-free, nut allergies
4. Include family size slider (1-8 people) with visual indicators
5. Design cooking skill level selector: beginner, intermediate, advanced with descriptions
6. Add time availability toggles for weekdays vs weekends
7. Create notification preferences: prep reminders, meal suggestions, community updates
8. Include data export/import options for GDPR compliance
9. Add account deletion option with clear warning messaging
10. Design achievement badges section showing cooking milestones

CODE EXAMPLES & CONSTRAINTS:
- Profile header:
  ```html
  <div class="bg-gradient-to-r from-green-600 to-green-500 p-6 text-white">
    <div class="flex items-center">
      <img src="avatar.jpg" alt="Profile" class="w-16 h-16 rounded-full border-4 border-white/20">
      <div class="ml-4">
        <h2 class="text-xl font-bold">User Name</h2>
        <p class="text-green-100">42 recipes cooked this month</p>
      </div>
    </div>
  </div>
  ```
- Settings sections: Card-based layout with `bg-white rounded-lg shadow-sm p-4 mb-4`
- Toggle switches: Use `relative inline-block w-10 h-6 bg-gray-200 rounded-full` styling
- Dietary checkboxes: Grid layout with icon + label combinations
- Skill level selector: Radio buttons with descriptive text and icons
- Slider components: Use `input[type="range"]` with custom Tailwind styling
- Danger zone: Red-themed section for destructive actions like account deletion
- DO NOT implement actual form submission or data persistence
- DO NOT add profile image upload functionality

STRICT SCOPE:
Create profile and settings page layout with form controls and visual organization. Use placeholder user data and static form elements. Do NOT implement form validation or submission logic.
```

## Usage Instructions

1. **Start each AI session** with the Foundational Context Prompt
2. **Choose the appropriate component prompt** based on your development priority
3. **Customize the prompt** with specific requirements or constraints for your implementation
4. **Iterate on the output** - generate one component at a time and refine based on results
5. **Test thoroughly** - all AI-generated code requires human review, testing, and refinement for production readiness

## Important Reminders

⚠️ **All AI-generated code will require careful human review, testing, and refinement to be considered production-ready.**

- Verify accessibility compliance (WCAG AA)
- Test responsive design across device sizes  
- Validate HTML semantics and structure
- Ensure Tailwind classes are correct and optimized
- Check for potential security issues in any form handling
- Test keyboard navigation and screen reader compatibility

## Additional Component Prompts Available

The above prompts cover the core user interface components. Additional prompts can be created for:
- Authentication forms (login/register)
- Onboarding flow screens
- Community recipe sharing interface  
- Cooking mode step-by-step view
- Push notification management
- Recipe creation/editing forms

Contact the UX team for additional specialized component prompts as needed.
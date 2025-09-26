# AI Frontend Generation Prompt for IMKitchen

## Complete Project Context

You are building the frontend for **IMKitchen**, an intelligent meal planning platform that eliminates decision fatigue for home cooks through automated weekly meal generation. The platform serves busy families who want to expand their recipe repertoire without planning complexity.

### Tech Stack & Architecture
- **Backend:** Rust with Axum web framework, SQLite database with SQLx
- **Frontend:** Server-side rendered HTML with Askama templates  
- **Styling:** Tailwind CSS 3.0+ utility-first approach
- **Interactivity:** TwinSpark declarative HTML attributes (zero custom JavaScript)
- **Architecture:** Progressive Web App (PWA) with mobile-first design
- **Deployment:** Single binary serving pre-compiled templates

### Design System Foundation
- **Color Palette:** Kitchen-warm colors with high contrast
  - Primary: #F59E0B (amber), Secondary: #10B981 (emerald)  
  - Success: #22C55E, Warning: #F59E0B, Error: #EF4444
  - Neutrals: #64748B, #F8FAFC, #1E293B
- **Typography:** Inter font family, 16px base size, optimized for kitchen readability
- **Components:** Touch-optimized with 44px minimum targets
- **Accessibility:** WCAG 2.1 AA compliance with kitchen environment considerations

---

## High-Level Goal

Create a responsive, kitchen-optimized meal planning dashboard that serves as the primary interface for IMKitchen. The dashboard should enable one-touch weekly meal plan generation, visual calendar navigation, and seamless transitions to recipe discovery and shopping list management.

---

## Detailed Step-by-Step Instructions

### 1. Dashboard Structure & Layout
1. Create the main dashboard template with a mobile-first approach
2. Implement a 7-day calendar grid using Tailwind's `grid grid-cols-1 md:grid-cols-7` responsive system
3. Each day should contain 3 meal slots (breakfast, lunch, dinner) with visual separation
4. Add a prominent "Fill My Week" button using `bg-amber-500 hover:bg-amber-600` styling
5. Include a persistent bottom navigation bar with 5 main sections: Calendar, Discover, Collections, Shopping, Profile

### 2. Meal Calendar Components
1. Design meal slot cards with the following structure:
   - Recipe name (if assigned) with `text-lg font-semibold` styling
   - Prep time indicator with clock icon using `text-sm text-gray-600`
   - Difficulty indicator using color-coded dots (green=easy, yellow=medium, red=hard)
   - Empty state with "+" button for manual recipe addition
2. Implement visual hierarchy using `shadow-sm hover:shadow-md transition-shadow`
3. Add color-coded preparation alerts for advance prep requirements using `bg-orange-100 border-l-4 border-orange-500`

### 3. Mobile-First Responsive Behavior
1. Mobile (320px-767px): Single column daily view with horizontal swipe navigation
2. Tablet (768px-1023px): 3-day view with touch-optimized spacing
3. Desktop (1024px+): Full 7-day grid with hover states and keyboard navigation
4. Implement smooth transitions using `transform transition-transform duration-200`

### 4. TwinSpark Interactivity (No Custom JavaScript)
1. "Fill My Week" button should use `ts-req="/api/generate-meal-plan" ts-target="#weekly-calendar"`
2. Meal slot interactions use `ts-req="/api/meal-details/{id}" ts-target="#meal-modal"`
3. Day navigation uses `ts-req="/api/calendar/day/{date}" ts-target="#calendar-content"`
4. All form submissions use TwinSpark attributes for progressive enhancement

### 5. PWA Features & Performance
1. Add meta tags for PWA manifest and mobile optimization
2. Implement loading states with skeleton screens using `animate-pulse bg-gray-200`
3. Design offline indicators with `bg-yellow-100 border-yellow-400` styling
4. Optimize for kitchen environment with large touch targets and high contrast

---

## Code Examples, Data Structures & Constraints

### Expected Meal Plan Data Structure
```rust
pub struct WeeklyMealPlan {
    pub week_start: Date,
    pub days: Vec<DayMealPlan>,
}

pub struct DayMealPlan {
    pub date: Date,
    pub breakfast: Option<MealSlot>,
    pub lunch: Option<MealSlot>,
    pub dinner: Option<MealSlot>,
}

pub struct MealSlot {
    pub recipe_id: String,
    pub recipe_name: String,
    pub prep_time_minutes: u32,
    pub difficulty: Difficulty, // Easy, Medium, Hard
    pub requires_advance_prep: bool,
}
```

### TwinSpark Attribute Examples
```html
<!-- Meal plan generation -->
<button ts-req="/generate-plan" ts-target="#calendar" 
        class="bg-amber-500 hover:bg-amber-600 text-white font-bold py-4 px-8 rounded-lg">
    Fill My Week
</button>

<!-- Meal slot interaction -->
<div ts-req="/meal/{meal_id}" ts-target="#meal-details"
     class="bg-white rounded-lg shadow-sm p-4 cursor-pointer hover:shadow-md">
    <!-- Meal content -->
</div>
```

### Styling Constraints
- **DO NOT** use custom CSS or JavaScript - only Tailwind classes and TwinSpark attributes
- **DO NOT** implement client-side routing - use server-side navigation
- **DO USE** semantic HTML elements with proper ARIA labels for accessibility
- **DO USE** consistent spacing with Tailwind's spacing scale (space-y-4, gap-6, etc.)
- **DO ENSURE** all interactive elements meet 44px minimum touch target size

### Kitchen Environment Optimizations
- High contrast color combinations for various lighting conditions
- Large, easily tappable buttons with clear visual feedback
- Quick access patterns for time-sensitive cooking scenarios
- Error states that don't interrupt cooking workflow

---

## Strict Scope Definition

### Files You Should Create/Modify:
1. `templates/dashboard.html` - Main dashboard template
2. `templates/components/meal_slot.html` - Reusable meal slot component
3. `templates/components/navigation.html` - Bottom navigation component
4. `templates/layouts/base.html` - Base PWA layout with meta tags
5. Any supporting component templates for modals or overlays

### Files You Should NOT Touch:
- Any Rust backend files in `src/` directory
- Database migration files
- Configuration files (Cargo.toml, etc.)
- Any JavaScript files (the project uses zero custom JS)

### Implementation Requirements:
- All templates must use Askama syntax for Rust integration
- Progressive enhancement: functionality should work without JavaScript
- Mobile-first responsive design with kitchen usability priority
- Comprehensive accessibility support including screen reader compatibility
- Performance optimization for 3G mobile connections

---

## Visual Design Guidelines

### Color Usage Patterns
- **Primary Actions:** `bg-amber-500 hover:bg-amber-600 text-white`
- **Secondary Actions:** `bg-gray-100 hover:bg-gray-200 text-gray-800`
- **Success States:** `bg-emerald-500 text-white` or `text-emerald-600`
- **Warnings:** `bg-orange-100 border-orange-500 text-orange-800`
- **Errors:** `bg-red-100 border-red-500 text-red-800`

### Typography Hierarchy
- **Page Titles:** `text-3xl font-bold text-gray-900`
- **Section Headers:** `text-xl font-semibold text-gray-800`
- **Body Text:** `text-base text-gray-700`
- **Metadata:** `text-sm text-gray-600`
- **Timestamps:** `text-xs text-gray-500`

### Component Patterns
- **Cards:** `bg-white rounded-lg shadow-sm border border-gray-200`
- **Buttons:** Minimum `min-h-12` with `px-6 py-3` padding
- **Form Inputs:** `border-2 border-gray-300 focus:border-amber-500 rounded-lg`
- **Loading States:** `animate-pulse bg-gray-200 rounded`

---

## Important Implementation Notes

**Mobile Kitchen Context:** This interface will be used in kitchen environments with wet hands, varying lighting, and time pressure. Prioritize:
- Large, forgiving touch targets
- Clear visual hierarchy
- Immediate feedback for all interactions
- Graceful degradation when connectivity is poor

**TwinSpark Integration:** All dynamic behavior should use TwinSpark's declarative HTML attributes. Common patterns:
- `ts-req="POST /endpoint"` for form submissions
- `ts-target="#element-id"` for content replacement
- `ts-trigger="click"` or `ts-trigger="change"` for interaction triggers
- `ts-confirm="Are you sure?"` for destructive actions

**Accessibility Priority:** Kitchen environments require enhanced accessibility:
- High contrast ratios (4.5:1 minimum)
- Clear focus indicators for keyboard navigation
- Screen reader support with descriptive ARIA labels
- Voice-friendly interface design

---

## Conclusion & Quality Assurance

This prompt provides comprehensive guidance for generating the IMKitchen dashboard interface. The resulting code should create a production-ready foundation that:

1. **Serves Real User Needs:** Eliminates meal planning friction through intuitive automation
2. **Optimizes for Context:** Kitchen environment usability with mobile-first approach  
3. **Maintains Technical Standards:** Server-side rendering with progressive enhancement
4. **Ensures Accessibility:** WCAG AA compliance with cooking-specific considerations

**⚠️ Critical Reminder:** All AI-generated code requires careful human review, testing, and refinement to be considered production-ready. Specifically validate:
- Askama template syntax compatibility with Rust backend
- TwinSpark attribute accuracy for your specific implementation
- Responsive behavior across all target devices
- Accessibility compliance through automated and manual testing
- Performance optimization for mobile 3G connections

The generated code should serve as a solid foundation that significantly accelerates development while maintaining high quality standards appropriate for a production meal planning application.
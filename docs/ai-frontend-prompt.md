# AI Frontend Prompt - imkitchen PWA

**Generated:** 2025-10-11
**Target Tools:** Vercel v0, Lovable.ai, Cursor, Claude Code
**Tech Stack:** React/Next.js (for prototyping) → Will be converted to Rust (Axum + Askama + TwinSpark)

---

## Project Overview

Build **imkitchen**, a mobile-first Progressive Web App (PWA) for intelligent meal planning and cooking optimization. The platform eliminates meal planning complexity through automated weekly scheduling, advance preparation reminders, and community recipe discovery.

**Core Value Proposition:** Unlock access to complex recipes by automating timing, preparation, and scheduling complexity—enabling home cooks to utilize their full recipe collections without planning burden.

---

## Design System Foundation

### Color Palette

```css
/* Primary Colors */
--primary-500: #2563eb;  /* Brand blue - trust, reliability */
--primary-600: #1d4ed8;  /* Hover/active states */
--primary-400: #3b82f6;  /* Backgrounds, subtle accents */

/* Secondary Colors */
--secondary-500: #f59e0b;  /* Amber - warmth, cooking */
--secondary-600: #d97706;  /* Hover states */

/* Semantic Colors */
--success-500: #10b981;  /* Green - meal completed, prep done */
--warning-500: #f59e0b;  /* Amber - prep required */
--error-500: #ef4444;    /* Red - validation errors */
--info-500: #3b82f6;     /* Blue - informational */

/* Neutral Colors (Kitchen Mode High Contrast) */
--gray-900: #111827;  /* Text primary - 7:1 contrast ratio */
--gray-600: #4b5563;  /* Text secondary */
--gray-400: #9ca3af;  /* Borders */
--gray-100: #f3f4f6;  /* Card backgrounds */
--white: #ffffff;

/* Complexity Badge Colors */
--simple: #10b981;    /* Green */
--moderate: #f59e0b;  /* Amber */
--complex: #ef4444;   /* Red */
```

### Typography

**System Font Stack (Performance Optimized):**
```css
font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto',
             'Helvetica Neue', Arial, sans-serif;
```

**Type Scale (Mobile-First, 16px base):**
- **H1**: 28px (1.75rem), 700 Bold, line-height 1.3
- **H2**: 24px (1.5rem), 600 Semibold, line-height 1.3
- **H3**: 20px (1.25rem), 600 Semibold, line-height 1.4
- **Body Large**: 18px (1.125rem), 400 Regular, line-height 1.6 (Recipe instructions)
- **Body**: 16px (1rem), 400 Regular, line-height 1.5
- **Body Small**: 14px (0.875rem), 400 Regular, line-height 1.5
- **Label**: 14px (0.875rem), 500 Medium, line-height 1.4

**Kitchen Mode:** Add +4px to H1, H2, H3, Body Large, Body for enhanced readability

### Spacing & Layout

**8px Grid System:**
- xs: 4px, sm: 8px, md: 16px, lg: 24px, xl: 32px, 2xl: 48px, 3xl: 64px

**Responsive Breakpoints:**
- Mobile (default): 0-767px
- Tablet (md): 768px-1023px
- Desktop (lg): 1024px+

**Container Widths:**
- Narrow: 640px (forms/modals)
- Medium: 960px (main content)
- Wide: 1280px (dashboard/calendar)

**Touch Targets:** 44x44px minimum (WCAG AAA), 48x48px recommended

### Component Styling

**Border Radius:** 12px (default), 8px (small), 16px (large)

**Elevation (Shadows):**
- Small: `0 1px 3px rgba(0,0,0,0.1), 0 1px 2px rgba(0,0,0,0.06)`
- Medium: `0 4px 6px -1px rgba(0,0,0,0.1), 0 2px 4px -1px rgba(0,0,0,0.06)`
- Large: `0 10px 15px -3px rgba(0,0,0,0.1), 0 4px 6px -2px rgba(0,0,0,0.05)`

---

## User Interface Requirements

### Primary Navigation (Bottom Tab Bar - Mobile)

Fixed bottom navigation with 5 tabs:

1. **🏠 Home** - Dashboard with today's meals
2. **📅 Plan** - Meal planning calendar
3. **📖 Recipes** - My recipe library
4. **🌍 Discover** - Community recipes
5. **🛒 Shop** - Shopping lists

**Design Requirements:**
- Fixed bottom position with safe area insets
- Active tab: Primary-500 color, filled icon
- Inactive tab: Gray-600, outline icon
- Labels visible on all tabs
- Ripple animation on tap
- Minimum 44x44px touch targets

**Desktop Alternative:** Left sidebar navigation (256px width, collapsible to 64px icon-only)

---

## Key Screens & Layouts

### 1. Dashboard (Home)

**Layout - Mobile:**
- Top app bar: "imkitchen" logo, notification bell, profile avatar
- Hero section: "Today's Meals" card
  - Breakfast, Lunch, Dinner cards with recipe image, title, prep indicator
  - "No meal planned" state with CTA
- Prep Tasks section: List of today's prep tasks with checkboxes
- Quick actions: "Generate Meal Plan" FAB (floating action button)
- Recipe variety metrics: "12 recipes this month (+3 vs last month)"

**Layout - Desktop:**
- 3-column grid: Sidebar (left), Today's Meals (center 60%), Prep Tasks + Metrics (right 40%)

**Empty State (No Meal Plan):**
- Illustration + "Add 7+ recipes to generate your first meal plan"
- "Add Recipe" and "Discover Recipes" CTAs

### 2. Meal Planning Calendar

**Layout - Mobile:**
- Week selector with Previous/Next arrows
- Current week displayed vertically (one day per screen, swipe left/right)
- Each day shows 3 meal slots (Breakfast, Lunch, Dinner)
- "Generate Meal Plan" button at top (if no plan exists)
- "Regenerate Meal Plan" button (if plan exists)

**Layout - Desktop:**
- 7-day week grid (Sunday-Saturday)
- 3 rows per day (Breakfast, Lunch, Dinner)
- Hover shows "Replace This Meal" button on each slot

**Meal Slot Component:**
- Recipe thumbnail image (left)
- Recipe title (truncated 1 line)
- Prep + Cook time
- Complexity badge (Simple/Moderate/Complex with color)
- Prep required indicator (⏰ yellow clock icon if advance prep needed)
- Algorithm reasoning tooltip: "Assigned to Saturday: more prep time available"

**States:**
- Empty slot: Dashed border, "Add Meal" button
- Today's slot: Blue border, elevated shadow
- Past slot: Dimmed/grayed out

### 3. Recipe Library (My Recipes)

**Layout - Mobile:**
- Top: Search bar + Filter button
- View toggle: List view / Grid view (2 columns)
- Recipe count: "7/10 recipes" (free tier) or "24 recipes" (premium)
- "Create Recipe" FAB

**Layout - Desktop:**
- Left sidebar: Filters (Collections, Favorites, Complexity, Tags)
- Main area: Grid view (4 columns)

**Recipe Card Component:**
- Square image (placeholder if none)
- Title (truncate 2 lines)
- Complexity badge (top-right corner)
- Prep + Cook time (bottom)
- Favorite heart icon (top-left, toggleable)
- Rating stars (if community recipe)
- Advance prep indicator

**Recipe Card States:**
- Default: Subtle shadow, rounded 12px
- Hover: Elevated shadow
- Favorited: Yellow filled heart
- Selected: Primary-500 border highlight

### 4. Recipe Detail / Creation

**Recipe Detail (View Mode):**

**Mobile - Tabbed Interface:**
- Tab 1: Ingredients (list with quantities/units)
- Tab 2: Instructions (numbered steps)
- Tab 3: Reviews (community ratings, if shared)

**Desktop - Side-by-Side:**
- Left 40%: Ingredients section
- Right 60%: Instructions section

**Header:**
- Recipe image (large, hero)
- Recipe title (H1)
- Complexity badge, Prep + Cook time, Servings
- Action buttons: Edit, Share to Community, Mark Favorite, Delete

**Kitchen Mode Toggle:**
- Large text (+4px)
- High contrast (7:1 ratio)
- Step-by-step mode (one instruction at a time, large "Next" button)
- Screen wake-lock (prevent sleep while cooking)

**Recipe Creation Form:**
- Title input
- Image upload (optional)
- Ingredients list (dynamic add/remove rows):
  - Quantity (number input)
  - Unit (dropdown: cup, tbsp, tsp, oz, lb, g, kg, whole)
  - Ingredient name (text input)
- Instructions (numbered textarea, add/remove steps)
- Prep time, Cook time (number inputs)
- Advance prep requirements (textarea: e.g., "Marinate 4 hours")
- Serving size (number input)
- Privacy toggle: Private / Share to Community

**Validation:**
- Title required (non-empty)
- At least 1 ingredient
- At least 1 instruction step
- Real-time validation on blur, inline error messages

### 5. Community Discovery (Discover)

**Layout:**
- Filter bar: Rating (4+, 3+), Cuisine, Prep Time, Dietary
- Search bar with autocomplete
- Sort: Highest Rated, Most Recent, Most Reviewed

**Community Recipe Feed:**
- Grid view (mobile: 2 cols, tablet: 3 cols, desktop: 4 cols)
- Recipe cards with:
  - Community recipe indicator (small badge)
  - Creator attribution (username below title)
  - Rating stars + review count (4.8 ⭐ 47 reviews)
  - "Add to My Recipes" button on hover

**Recipe Detail (Community):**
- Same layout as personal recipe detail
- Read-only (cannot edit)
- "Add to My Recipes" button (copies to personal library)
- Creator attribution prominent
- Reviews section visible

**Rating & Review:**
- 1-5 star selector
- Optional text review (max 500 characters)
- Submit button
- User can edit/delete own review

### 6. Shopping List

**Layout - Mobile:**
- Week selector dropdown: "This Week", "Next Week", "Week of {date}"
- Category sections (collapsible):
  - Produce (8 items)
  - Dairy (5 items)
  - Meat & Seafood (3 items)
  - Pantry (12 items)
  - Frozen (2 items)
  - Bakery (1 item)
- Progress indicator: "14 of 31 items collected"
- Filter toggle: Show All / Show Remaining / Show Collected

**Shopping List Item:**
- Checkbox (44x44px) with haptic feedback
- Ingredient name + quantity/unit
- Strike-through when checked, 60% opacity
- Checked items move to bottom of category

**Category Header:**
- Category name + item count
- Expand/collapse arrow
- Auto-collapse when all items checked

**Offline Support:**
- PWA service worker caches list
- Checkoff state persists in LocalStorage
- Syncs to server when connectivity restored
- Offline indicator shown when disconnected

### 7. Onboarding Flow (New User)

**3-Step Wizard:**

**Step 1: Dietary Restrictions**
- Title: "Let's personalize your experience"
- Checkboxes: Vegetarian, Vegan, Gluten-free
- Allergens text input (comma-separated)
- Skip button (applies defaults)
- Next button

**Step 2: Household & Skill**
- Household size (number input: 1-10)
- Cooking skill level (radio: Beginner, Intermediate, Advanced)
- Next button

**Step 3: Availability**
- "When do you typically cook dinner on weeknights?"
- Time range picker (default: 6-7pm)
- Duration slider (default: 45 minutes)
- Complete button

**After Onboarding:**
- Redirect to Dashboard
- Show empty state: "Add recipes to get started"

---

## Core Components Specification

### 1. Button Component

**Variants:**
- **Primary**: Solid background (Primary-500), white text, high contrast
- **Secondary**: Outlined (Primary-500 border), Primary-500 text
- **Ghost**: Text only (Primary-500), no border/background
- **Danger**: Solid background (Error-500), white text
- **Icon Button**: Icon only, 44x44px minimum

**States:**
- Default: Base styling
- Hover: Darken background slightly (desktop only)
- Active: Scale to 0.95, spring back (100-150ms)
- Focus: Visible 2px Primary-500 outline, 4px offset
- Disabled: 50% opacity, no pointer events
- Loading: Spinner replacing content

**Sizes:**
- Small: 36px height
- Medium: 44px height (default)
- Large: 56px height

### 2. Recipe Card Component

See "Recipe Library" section above for full specification.

### 3. Meal Slot Component (Calendar)

See "Meal Planning Calendar" section above for full specification.

### 4. Form Input Component

**Variants:**
- Text Input (single line)
- Textarea (multi-line)
- Number Input (with steppers)
- Select Dropdown
- Date/Time Picker

**Structure:**
- Label (always visible, not placeholder)
- Input field
- Helper text (optional, gray)
- Error message (conditional, red with icon)
- Character count (for limited fields)

**States:**
- Default: Gray-400 border, black text
- Focus: Primary-500 border, elevated
- Error: Error-500 border, error message below, `aria-invalid="true"`
- Disabled: Gray-100 background, no interaction
- Valid: Success-500 checkmark icon (after validation)

### 5. Modal/Dialog Component

**Variants:**
- Full Screen (mobile): Takes entire viewport
- Center Modal (desktop): Centered with backdrop, max-width 640px
- Bottom Sheet (mobile): Slides up from bottom

**Structure:**
- Header: Title (H2) + Close button (X icon, top-right)
- Content area: Scrollable if needed
- Footer: Action buttons (Cancel, Confirm)

**Behavior:**
- Open animation: Backdrop fade-in (200ms) + content slide-up (300ms)
- Close animation: Content slide-down + backdrop fade-out (250ms)
- Escape key closes modal
- Focus trap: Tab cycles within modal
- Return focus to trigger element on close

**Accessibility:**
- `role="dialog"` `aria-modal="true"`
- `aria-labelledby` pointing to title
- Focus on first interactive element on open

### 6. Toast Notification Component

**Variants:**
- Success: Green background, checkmark icon
- Error: Red background, X icon
- Warning: Yellow background, alert icon
- Info: Blue background, info icon

**Structure:**
- Icon (left, matching variant)
- Message text (1-2 lines, truncate)
- Optional action button (Undo, Retry)
- Auto-dismiss timer (4 seconds default)

**Position:**
- Mobile: Top of screen (below status bar)
- Desktop: Top-right corner

**Animation:**
- Slide in from top (mobile) or right (desktop) - 300ms
- Auto-dismiss fade-out after 4s - 250ms
- Swipe to dismiss (mobile)

### 7. Navigation Tab Bar (Mobile)

**Structure:**
- 5 navigation items: Home, Plan, Recipes, Discover, Shop
- Each item: Icon + Label
- Fixed bottom position with safe area insets

**States:**
- Active: Primary-500 color, filled icon, medium weight label
- Inactive: Gray-600 color, outline icon, regular weight label
- Tap: Ripple animation

**Accessibility:**
- `role="tablist"` with `role="tab"` items
- Keyboard arrow navigation
- Active tab announced to screen readers

### 8. Loading States

**Spinner:**
- Circular rotation (1s infinite)
- Sizes: 24px (inline/button), 48px (page)

**Skeleton Loader:**
- Pulse animation (1.5s infinite) with shimmer effect
- Placeholder boxes matching content layout
- Use for recipe cards, meal slots while loading

**Progress Bar:**
- Linear indeterminate wave
- Use for meal plan generation, shopping list generation

### 9. Empty States

**Pattern:**
- Illustration or icon (centered)
- Heading: "No [content] yet"
- Description: Brief explanation
- Primary CTA button
- Secondary action (optional)

**Examples:**
- No recipes: "Add recipes to get started" + "Create Recipe" button
- No meal plan: "Generate your first meal plan" + "Add 7 recipes first" guidance
- No shopping list: "Generate meal plan first" CTA

### 10. Calendar Week View Component

**Desktop Layout:**
- 7 columns (days: Sun-Sat)
- 3 rows per day (Breakfast, Lunch, Dinner)
- Day headers: Date + Day name
- Previous/Next week arrows (top)
- "Today" indicator (blue border, elevated)

**Mobile Layout:**
- Vertical stack, one day at a time
- Swipe left/right to navigate days
- Day header prominent
- 3 meal slots stacked vertically

---

## User Flows to Implement

### Flow 1: New User Onboarding → First Meal Plan

1. Landing page → Click "Get Started"
2. Registration form (email, password min 8 chars)
3. Account created, auto-login
4. Onboarding wizard (3 steps: dietary, household, availability)
5. Dashboard empty state: "Add 7+ recipes"
6. Click "Add Recipe" → Recipe creation form
7. Enter recipe details, mark as favorite
8. Repeat until 7 recipes created
9. Dashboard shows "Generate Meal Plan" button
10. Click → Algorithm processing (3-5s spinner)
11. Week calendar displays with meals assigned
12. View shopping list → Success!

### Flow 2: Daily Cooking Experience

1. Morning 9am: Push notification "Prep reminder: Marinate chicken tonight"
2. Tap notification → Recipe detail, prep instructions highlighted
3. Mark prep complete → Checkmark on dashboard
4. Evening 6pm: Cooking reminder notification
5. Open app → Dashboard shows today's meals
6. Click dinner recipe → Recipe detail
7. Toggle Kitchen Mode (large text, high contrast)
8. Step-by-step cooking instructions
9. Complete meal
10. Next day: Rate recipe prompt (1-5 stars)

### Flow 3: Meal Plan Disruption Recovery

1. User realizes schedule conflict (meeting runs late)
2. Open app → Navigate to meal calendar
3. Find conflicting meal slot (Wednesday dinner)
4. Click "Replace This Meal"
5. System suggests 3-5 quick alternatives (<30 min)
6. Select replacement recipe
7. Calendar updates in real-time
8. Shopping list recalculates (new ingredients highlighted)
9. Success!

### Flow 4: Community Recipe Discovery

1. Navigate to Discover tab
2. Filter: "Highly Rated 4+ stars"
3. Browse recipe cards
4. Click interesting recipe
5. View full recipe + community reviews
6. Click "Add to My Recipes"
7. Recipe copied to personal library (check free tier limit)
8. Mark as favorite
9. Included in next meal plan generation

### Flow 5: Shopping List In-Store Use

1. Active meal plan exists → Click "Shopping List"
2. System generates list (<2 sec)
3. Categories displayed (Produce, Dairy, Meat, etc.)
4. User at store → Open shopping list
5. PWA offline mode active (no connectivity needed)
6. Navigate by category (collapsible sections)
7. Tap item to check off → Strike-through + haptic
8. Progress: "14 of 31 items collected"
9. All items collected → Filter "Show Collected" to verify
10. Return home → List auto-syncs to server

---

## Accessibility Requirements (WCAG 2.1 Level AA)

### Mandatory Implementation:

1. **Color Contrast:**
   - Normal text: 4.5:1 minimum
   - Large text: 3:1 minimum
   - Kitchen Mode: 7:1 enhanced (AAA)
   - UI components: 3:1 minimum

2. **Keyboard Navigation:**
   - All interactive elements keyboard accessible (Tab, Enter, Space)
   - Focus indicators visible (2px Primary-500 outline, 4px offset)
   - Skip to main content link
   - Modal focus trap with Escape to close

3. **Screen Reader Support:**
   - Semantic HTML (nav, main, aside, header, footer)
   - Heading hierarchy (h1 → h2 → h3)
   - ARIA labels for icon-only buttons (`aria-label="Mark recipe as favorite"`)
   - ARIA live regions for toasts (`aria-live="polite"`) and errors (`aria-live="assertive"`)
   - Alt text for all images
   - Form labels explicit (`<label for="input-id">`)

4. **Touch Targets:**
   - Minimum 44x44px (WCAG AAA)
   - 8px spacing between adjacent targets
   - Haptic feedback on tap (mobile)

5. **Motion:**
   - Respect `prefers-reduced-motion` media query
   - Disable animations if set: `animation-duration: 0.01ms !important`

6. **Forms:**
   - Labels always visible (not placeholder-only)
   - Required fields: `aria-required="true"` + visual (*)
   - Error identification: Icon + color + text
   - `aria-invalid="true"` + `aria-describedby="error-id"` on errors

---

## Technical Implementation Notes

### State Management

**Global State (Context/Redux):**
- User authentication status
- User profile (dietary restrictions, preferences)
- Active meal plan
- Recipe library
- Shopping list

**Local State:**
- Form inputs
- Modal open/close
- Filter selections
- Kitchen mode toggle

### Data Structures

**Recipe Object:**
```typescript
{
  id: string;
  userId: string;
  title: string;
  image?: string;
  ingredients: Array<{
    quantity: number;
    unit: string;
    name: string;
  }>;
  instructions: Array<{
    stepNumber: number;
    text: string;
    timerMinutes?: number;
  }>;
  prepTimeMinutes: number;
  cookTimeMinutes: number;
  advancePrepText?: string; // e.g., "Marinate 4 hours"
  servingSize: number;
  complexity: 'simple' | 'moderate' | 'complex';
  tags: string[];
  isShared: boolean;
  isFavorite: boolean;
  rating?: number;
  reviewCount?: number;
  createdAt: Date;
}
```

**Meal Plan Object:**
```typescript
{
  id: string;
  userId: string;
  weekStartDate: Date;
  isActive: boolean;
  meals: Array<{
    date: Date;
    mealType: 'breakfast' | 'lunch' | 'dinner';
    recipeId: string;
    prepRequired: boolean;
    algorithmReasoning: string; // e.g., "Saturday: more prep time"
  }>;
  createdAt: Date;
}
```

**Shopping List Object:**
```typescript
{
  id: string;
  userId: string;
  weekStartDate: Date;
  categories: Array<{
    name: 'Produce' | 'Dairy' | 'Meat' | 'Pantry' | 'Frozen' | 'Bakery';
    items: Array<{
      ingredientName: string;
      quantity: number;
      unit: string;
      isChecked: boolean;
      fromRecipeIds: string[];
    }>;
  }>;
}
```

### PWA Requirements

1. **Manifest File:**
   - App name: "imkitchen"
   - Short name: "imkitchen"
   - Display: standalone
   - Theme color: #2563eb (Primary-500)
   - Background color: #ffffff
   - Icons: 192x192, 512x512

2. **Service Worker:**
   - Cache strategy: Network-first for HTML, Cache-first for static assets
   - Offline recipe access (cache recipe pages after first view)
   - Background sync for offline actions (favorite, checkoff)

3. **Installability:**
   - Prompt user to install after 2+ visits
   - "Add to Home Screen" prompt

---

## Animation Specifications

### Duration Standards:
- Micro: 100-150ms (button tap, checkbox)
- Short: 200-300ms (transitions, fades, slides)
- Medium: 400-600ms (complex state changes)

### Key Animations:
- **Button Tap**: Scale to 0.95 (100ms) → spring back (150ms)
- **Modal Open**: Backdrop fade-in (200ms) + content slide-up (300ms)
- **Toast**: Slide-in from top (300ms), auto-dismiss fade (250ms) after 4s
- **Shopping Item Check**: Strike-through (200ms) + fade to 60%
- **Recipe Card Hover**: Elevated shadow transition (200ms)

### Easing Functions:
- Ease-out: Default for exits (cubic-bezier(0, 0, 0.2, 1))
- Ease-in: For entries (cubic-bezier(0.4, 0, 1, 1))
- Spring: For playful feedback (elastic overshoot)

---

## Special Features

### Kitchen Mode

**Activation:** Toggle button in recipe detail view

**Changes:**
- Typography: +4px to all headings and body text
- Contrast: 7:1 ratio (AAA level)
- Layout: Simplified, hide non-essential elements
- Step-by-step mode: One instruction at a time, large "Next" button
- Screen wake-lock: Prevent sleep while cooking (Wake Lock API)

### Voice Control (Future Enhancement)

**Commands:**
- "Next step" - Advance recipe instructions
- "Mark complete" - Complete prep task
- "Start timer" - Begin cooking timer

**Visual Feedback:** Display recognized command on screen

---

## Error States & Edge Cases

### Recipe Creation:
- **Empty title**: Inline error "Recipe title is required"
- **No ingredients**: Prevent save, show "Add at least 1 ingredient"
- **No instructions**: Prevent save, show "Add at least 1 instruction step"

### Meal Plan Generation:
- **< 7 favorite recipes**: Show "Add {X} more recipes to generate meal plan" with "Add Recipe" CTA
- **Algorithm timeout**: Retry button with loading state
- **Network error**: "Offline. Please try again when connected" + queue for retry

### Shopping List:
- **Offline mode**: Display "Offline - Changes will sync when connected" banner
- **Checkoff state lost**: LocalStorage fallback, sync on reconnect
- **Week selector error**: Default to current week

### Community Discovery:
- **Recipe limit reached (free tier)**: Upgrade prompt OR "Delete a recipe first" flow
- **Network error during copy**: Offline queue with retry
- **Recipe removed by author**: "Recipe no longer available" message

---

## Performance Targets

- **Initial Load**: <3 seconds on 3G connection
- **Meal Plan Generation**: <5 seconds for up to 50 recipes
- **Shopping List Generation**: <2 seconds
- **Interaction Response**: <100ms button feedback
- **Lighthouse Score**: >90 for Performance, Accessibility, Best Practices, PWA

---

## Final Notes for AI Generation

**Priorities:**
1. Mobile-first responsive design (most users on mobile)
2. Accessibility compliance (WCAG 2.1 AA) - NOT optional
3. Kitchen-optimized UX (large touch targets, high contrast, simple interactions)
4. Offline-first PWA (service worker, local storage)
5. Performance (lazy loading, code splitting, optimized images)

**Code Quality:**
- TypeScript for type safety
- Component-based architecture (reusable components)
- Semantic HTML (nav, main, aside, etc.)
- ARIA attributes where needed
- Responsive utility classes (Tailwind or custom CSS)

**Testing Reminders:**
- Test on real mobile devices (iOS Safari, Android Chrome)
- Test with screen readers (NVDA, VoiceOver)
- Test keyboard-only navigation
- Test offline mode (service worker)
- Test reduced motion preference

**Remember:** This prototype will be converted to Rust (Axum + Askama + TwinSpark), so keep JavaScript minimal and focus on server-rendered HTML patterns where possible.

---

## Example Prompt for AI Tools

**For Vercel v0 / Lovable.ai:**

"Build a mobile-first PWA called 'imkitchen' for intelligent meal planning. Use the color palette: Primary #2563eb, Success #10b981, Warning #f59e0b, Error #ef4444. System font stack. Bottom tab navigation (Home, Plan, Recipes, Discover, Shop). Key screens: Dashboard with today's meals cards, Meal Calendar (7-day week grid on desktop, vertical stack on mobile), Recipe Library (grid/list toggle), Shopping List (category groups with checkboxes). All components must have 44x44px touch targets, WCAG AA contrast (4.5:1 text, 7:1 kitchen mode), keyboard accessible, ARIA labels. Include recipe card component (image, title, complexity badge, favorite heart), meal slot component (recipe preview, prep indicator, replace button), and shopping list item (checkbox, strike-through when checked). Implement full onboarding flow (3 steps: dietary, household, availability). Make it installable as PWA with offline support. Use Tailwind CSS with 8px spacing grid."

---

**Generated by:** imkitchen UX Specification v1.0
**Last Updated:** 2025-10-11
**Status:** Ready for prototyping

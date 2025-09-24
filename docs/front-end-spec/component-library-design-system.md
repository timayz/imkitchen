# Component Library / Design System

## Design System Approach
**Design System Approach:** Custom component library built on atomic design principles, optimized for mobile-first PWA with kitchen environment considerations

## Core Components

### Meal Card Component
**Purpose:** Display meal information in calendar and list contexts with clear visual hierarchy

**Variants:** 
- Calendar slot (compact)
- Detail view (expanded) 
- Shopping list meal reference (minimal)

**States:** 
- Default, Selected, Completed, Needs Prep, Easy Mode Alternative

**Usage Guidelines:** Always include prep time indicator, use consistent color coding for complexity, ensure 44px minimum touch target

### Recipe Rating Component  
**Purpose:** Capture and display community recipe ratings with 5-star system

**Variants:**
- Input mode (interactive stars)
- Display mode (readonly with aggregate)
- Compact mode (small rating badge)

**States:**
- Not rated, User rated, Average rating, Loading submission

**Usage Guidelines:** Prominent placement on recipe cards, immediate visual feedback on rating submission

### Action Button Component
**Purpose:** Primary call-to-action buttons for key user flows

**Variants:**
- Primary (Fill My Week, Generate List)
- Secondary (Save Recipe, Share)  
- Floating Action Button (context-aware)

**States:**
- Default, Loading, Success, Disabled

**Usage Guidelines:** Maximum one primary action per screen, loading states with progress indication

# Component Library / Design System

**Design System Approach:** Custom design system optimized for kitchen environments and cooking workflows, built on accessibility-first principles with large touch targets and high contrast ratios

## Core Components

### Cooking Timer Component

**Purpose:** Display countdown timers with clear visual hierarchy and multiple simultaneous timer support

**Variants:** 
- Compact (dashboard widget)
- Prominent (cook mode primary)  
- Multi-timer (coordination view)

**States:** Active, Paused, Complete, Warning (<2 min remaining), Critical (<30 sec)

**Usage Guidelines:** Always use high contrast colors, provide both visual and audio alerts, support custom labels

### Recipe Card Component  

**Purpose:** Display recipe information consistently across library, planning, and search contexts

**Variants:**
- Compact (list view)
- Featured (dashboard highlight)
- Detailed (planning selection)

**States:** Default, Hover/Focus, Selected, Cooking, Favorited, Offline Available

**Usage Guidelines:** Include difficulty, time, and dietary indicators, support offline imagery caching

### Step Indicator Component

**Purpose:** Show cooking progress through recipe steps with clear current position

**Variants:**
- Linear (mobile)
- Circular (tablet/desktop)
- Minimal (overlay)

**States:** Completed, Current, Upcoming, Skipped, In Progress

**Usage Guidelines:** Large touch targets for navigation, clear visual hierarchy, works in poor lighting

### Notification Banner Component

**Purpose:** Communicate timing alerts, system status, and critical cooking updates

**Variants:**
- Timing Alert (high priority)
- System Status (medium priority)  
- Tips & Suggestions (low priority)

**States:** Active, Dismissible, Persistent, Action Required

**Usage Guidelines:** Never obscure critical cooking information, provide clear dismiss actions, stack appropriately

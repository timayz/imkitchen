# Component Library / Design System

**Design System Approach:** Custom design system built on Tailwind CSS utility classes with kitchen-focused component patterns and accessibility-first approach

## Core Components

### Button Component

**Purpose:** Primary interaction element with kitchen-optimized sizing and states

**Variants:** Primary, Secondary, Ghost, Danger, Voice-activated

**States:** Default, Hover, Active, Disabled, Loading, Voice-listening

**Usage Guidelines:** Minimum 44px touch targets, high contrast ratios, clear focus indicators for keyboard navigation

### Recipe Card Component

**Purpose:** Display recipe information consistently across search, favorites, and planning contexts

**Variants:** Compact (list view), Standard (grid view), Featured (hero display)

**States:** Default, Hover, Selected, Saved, In-progress

**Usage Guidelines:** Include cooking time, difficulty, and ingredient match indicators prominently

### Timer Component

**Purpose:** Kitchen timer management with multiple simultaneous timer support

**Variants:** Compact, Standard, Full-screen alert

**States:** Inactive, Running, Paused, Completed, Overdue

**Usage Guidelines:** Distinct visual/audio alerts, clear labeling system, emergency stop functionality

### Inventory Item Component

**Purpose:** Display ingredient information with quantity and freshness status

**Variants:** List item, Card view, Quick-add

**States:** Fresh, Expiring-soon, Expired, Low-stock, Out-of-stock

**Usage Guidelines:** Color-coded freshness indicators, swipe gestures for mobile editing

### Voice Control Indicator

**Purpose:** Provide clear feedback for voice interaction states

**Variants:** Listening, Processing, Confirmation, Error

**States:** Inactive, Active, Success, Error

**Usage Guidelines:** Subtle but clear visual feedback, works in bright kitchen lighting

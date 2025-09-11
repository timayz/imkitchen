# Component Library / Design System

**Design System Approach:** Create a custom design system optimized for cooking contexts, building on Tailwind CSS utilities with custom components for timing intelligence and meal planning. Focus on kitchen-friendly interactions (large touch targets, high contrast, readable in various lighting) while maintaining modern, trustworthy aesthetics.

## Core Components

### Meal Card

**Purpose:** Displays meal information across calendar, dashboard, and list views with consistent timing intelligence integration

**Variants:** 
- Calendar slot (compact with timing indicators)
- Dashboard card (expanded with prep status)
- List item (medium with quick actions)
- Preview card (detailed with ratings)

**States:** 
- Empty/placeholder, assigned, prep needed, prep overdue, ready to cook, completed
- Selected/unselected, dragging, error state

**Usage Guidelines:** Always include timing information when available. Use color coding consistently for prep status. Ensure touch targets meet minimum 44px requirement for kitchen use.

### Timing Timeline

**Purpose:** Visual representation of recipe preparation phases and dependencies, core to ImKitchen's value proposition

**Variants:**
- Compact horizontal (recipe cards and calendar)
- Expanded vertical (recipe detail view)
- Interactive (with phase selection and customization)
- Progress view (showing completion during cooking)

**States:**
- Planning (shows estimated timing)
- Active (shows progress and remaining time)
- Completed (shows actual vs. estimated timing)
- Modified (user has adjusted default timing)

**Usage Guidelines:** Always show relative timing (2 hours before, 1 day ahead) rather than absolute times when possible. Use visual hierarchy to emphasize critical timing points.

### Recipe Difficulty Indicator

**Purpose:** Communicate recipe complexity focusing on timing coordination rather than cooking skill

**Variants:**
- Icon only (for compact spaces)
- Icon with label (standard usage)
- Detailed breakdown (prep time, active time, complexity factors)

**States:**
- Simple (minimal prep, straightforward timing)
- Moderate (some advance prep or timing coordination)
- Complex (multiple timing dependencies, extensive prep)

**Usage Guidelines:** Base complexity on timing coordination requirements, not cooking techniques. Include prep time estimates prominently.

### Action Button

**Purpose:** Primary and secondary actions optimized for mobile cooking contexts

**Variants:**
- Primary (Fill My Week, Add Recipe, Start Cooking)
- Secondary (Edit, Share, Favorite)
- Floating Action Button (quick access to core actions)
- Icon button (compact actions in lists and cards)

**States:**
- Default, hover, active, disabled, loading
- Success (with confirmation animation)
- Error (with clear recovery options)

**Usage Guidelines:** Minimum 44px touch targets. Use loading states for any action taking >1 second. Provide clear visual feedback for state changes.

### Navigation Tab Bar

**Purpose:** Bottom navigation optimized for one-handed mobile use with cooking-specific considerations

**Variants:**
- Standard 5-tab layout
- Adaptive layout (hide less critical tabs on smaller screens)
- Badge indicators (for notifications and pending tasks)

**States:**
- Active/inactive tabs with clear visual distinction
- Notification badges with counts
- Contextual states (prep tasks urgent indicator)

**Usage Guidelines:** Keep tab labels short and universally understood. Use icons that test well with users. Ensure navigation works with kitchen gloves.

### Calendar Grid

**Purpose:** Weekly meal planning interface with drag-and-drop capabilities and timing visualization

**Variants:**
- Week view (7 days visible)
- Day view (single day expanded)
- Compact view (for smaller screens)

**States:**
- Empty slots with clear add prompts
- Filled slots with meal and timing information
- Drag states (source, target, invalid drop zones)
- Conflict indicators (timing or dietary issues)

**Usage Guidelines:** Clearly indicate drop zones during drag operations. Use consistent color coding for timing status across all calendar components.

### Prep Task Item

**Purpose:** Individual preparation task display optimized for kitchen workflow management

**Variants:**
- Compact list item (task lists)
- Expanded card (task detail)
- Notification format (push and in-app notifications)

**States:**
- Pending (not yet time to start)
- Ready (time to begin prep)
- In progress (user has started)
- Completed (task finished)
- Overdue (missed optimal timing)

**Usage Guidelines:** Always include estimated duration and deadline information. Use progressive disclosure for task details. Enable quick completion gestures.

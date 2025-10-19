# Story 5.5: Touch-Optimized Interface

Status: Approved

## Story

As a user interacting via touchscreen,
I want touch targets large enough to tap accurately,
so that I avoid mis-taps and frustration.

## Acceptance Criteria

1. All interactive elements (buttons, links, checkboxes) minimum 44x44px tap target
2. Adequate spacing between adjacent tap targets (8px minimum)
3. No hover-dependent interactions (avoid :hover for critical functionality)
4. Touch gestures intuitive: swipe to dismiss, pull to refresh (where appropriate)
5. Haptic feedback on button taps (where browser supports)
6. Long-press menus for contextual actions
7. Scrolling smooth and responsive (no janky scroll performance)
8. Pinch-to-zoom disabled for app UI, enabled for recipe images

## Tasks / Subtasks

- [ ] Task 1: Audit and enforce 44x44px minimum touch targets across all interactive elements (AC: 1)
  - [ ] Subtask 1.1: Update Tailwind config with min-touch-target utility classes (44px min-height/min-width)
  - [ ] Subtask 1.2: Apply min-touch-target classes to all buttons in templates/components/button.html
  - [ ] Subtask 1.3: Apply min-touch-target to navigation links (nav-tabs.html, sidebar navigation)
  - [ ] Subtask 1.4: Increase checkbox/radio input sizes to 24x24px minimum with clickable label area
  - [ ] Subtask 1.5: Update form-field.html inputs to meet 44px height minimum
  - [ ] Subtask 1.6: Audit icon-only buttons and ensure 44x44px dimensions
  - [ ] Subtask 1.7: Test touch targets on physical mobile device (iPhone, Android)

- [ ] Task 2: Implement adequate spacing between adjacent touch targets (AC: 2)
  - [ ] Subtask 2.1: Add gap-2 (8px) minimum between button groups in action bars
  - [ ] Subtask 2.2: Space navigation tabs with p-3 (12px) padding to prevent accidental taps
  - [ ] Subtask 2.3: Add margin-bottom to stacked buttons (mobile layouts)
  - [ ] Subtask 2.4: Test dense UI sections (shopping list items) for adequate spacing
  - [ ] Subtask 2.5: Create Playwright test to measure spacing between interactive elements

- [ ] Task 3: Remove hover-dependent critical functionality (AC: 3)
  - [ ] Subtask 3.1: Audit templates for :hover-only actions (replace with tap/click handlers)
  - [ ] Subtask 3.2: Convert dropdown menus to tap-to-open instead of hover-to-open
  - [ ] Subtask 3.3: Ensure tooltips appear on tap/long-press, not just hover
  - [ ] Subtask 3.4: Add visual :active states for touch feedback on tap
  - [ ] Subtask 3.5: Test all interactions work without mouse (touch-only device testing)

- [ ] Task 4: Implement intuitive touch gestures (AC: 4)
  - [ ] Subtask 4.1: Enable native browser pull-to-refresh (no custom implementation needed - verify works)
  - [ ] Subtask 4.2: Implement swipe-to-navigate for mobile meal calendar (single-day view)
  - [ ] Subtask 4.3: Add swipe-to-dismiss for toast notifications (optional enhancement)
  - [ ] Subtask 4.4: Use touch-action CSS property to control gesture behavior
  - [ ] Subtask 4.5: Test gestures on iOS Safari and Android Chrome

- [ ] Task 5: Add haptic feedback for button taps (AC: 5, optional browser support)
  - [ ] Subtask 5.1: Detect Vibration API support with feature detection
  - [ ] Subtask 5.2: Add vibrate(50) on button click events (subtle 50ms vibration)
  - [ ] Subtask 5.3: Add haptic feedback to critical actions (meal replacement, shopping list checkoff)
  - [ ] Subtask 5.4: Provide user setting to disable haptic feedback (accessibility)
  - [ ] Subtask 5.5: Test haptic feedback on physical devices (iOS, Android)

- [ ] Task 6: Implement long-press contextual menus (AC: 6)
  - [ ] Subtask 6.1: Add long-press detection (300ms threshold) for recipe cards
  - [ ] Subtask 6.2: Show contextual menu: Edit, Delete, View Details options
  - [ ] Subtask 6.3: Prevent context menu from triggering accidental tap
  - [ ] Subtask 6.4: Dismiss menu on tap outside or scroll
  - [ ] Subtask 6.5: Test long-press on various devices (ensure 300ms threshold works)

- [ ] Task 7: Optimize scrolling performance (AC: 7)
  - [ ] Subtask 7.1: Use passive event listeners for scroll events ({ passive: true })
  - [ ] Subtask 7.2: Optimize CSS for GPU-accelerated scrolling (will-change, transform3d)
  - [ ] Subtask 7.3: Reduce layout thrashing (batch DOM reads/writes in scroll handlers)
  - [ ] Subtask 7.4: Test scroll FPS on lower-end Android devices (target 60fps)
  - [ ] Subtask 7.5: Measure scroll jank with Chrome DevTools Performance tab

- [ ] Task 8: Configure pinch-to-zoom behavior (AC: 8)
  - [ ] Subtask 8.1: Set viewport meta tag: user-scalable=no for app UI
  - [ ] Subtask 8.2: Enable user-scalable=yes on recipe image modals/overlays
  - [ ] Subtask 8.3: Use touch-action: pinch-zoom CSS on image containers
  - [ ] Subtask 8.4: Test pinch-to-zoom disabled on forms, enabled on recipe images
  - [ ] Subtask 8.5: Verify iOS Safari and Android Chrome respect pinch-zoom settings

- [ ] Task 9: Comprehensive touch interaction testing (All ACs)
  - [ ] Subtask 9.1: Create Playwright tests for 44px touch target measurements
  - [ ] Subtask 9.2: Create visual regression tests for :active states
  - [ ] Subtask 9.3: Manual testing on physical devices (iPhone SE, Pixel 6)
  - [ ] Subtask 9.4: Test with screen reader + touch (VoiceOver, TalkBack)
  - [ ] Subtask 9.5: Verify WCAG 2.1 Level AA touch target compliance (44x44px minimum)

## Dev Notes

### Architecture Patterns and Constraints

**From Solution Architecture (docs/solution-architecture.md):**
- **Touch Target Sizing (Section 7.4 - Accessibility)**: WCAG 2.1 Level AA requires minimum 44x44px touch targets for all interactive elements
- **Tailwind CSS Configuration**: Define custom utilities `min-h-touch-target` and `min-w-touch-target` in tailwind.config.js
- **Progressive Enhancement**: Touch interactions work without JavaScript, enhanced with TwinSpark for smooth UX
- **Mobile-First Approach**: Touch optimization is core to mobile-first design strategy

**From Tech Spec Epic 5 (docs/tech-spec-epic-5.md):**
- **Module 3: Responsive Design System** - Tailwind config includes touch target utilities:
  ```javascript
  minHeight: {
    'touch-target': '44px', // WCAG 2.1 Level AA
  },
  minWidth: {
    'touch-target': '44px',
  }
  ```
- **Touch Target Optimization Pattern**:
  ```css
  .btn-primary, button, a[role="button"] {
    @apply min-h-touch-target min-w-touch-target;
    padding: 0.75rem 1.5rem; /* Comfortable padding within target */
  }
  ```
- **Haptic Feedback (Vibration API)**: Optional browser feature, requires feature detection
- **Passive Event Listeners**: Required for smooth scrolling (no scroll blocking)

**Touch Gesture Guidelines**:
- **Pull-to-Refresh**: Native browser behavior, no custom implementation needed
- **Swipe Navigation**: Use TwinSpark or minimal JavaScript for mobile calendar day swipe
- **Long-Press**: 300ms threshold standard, show contextual menu
- **Pinch-to-Zoom**: Disabled for app UI (`user-scalable=no`), enabled for images

### Source Tree Components

**Templates to Modify:**
- `templates/components/button.html` - Ensure all button variants meet 44px minimum
- `templates/components/form-field.html` - Update input heights to 44px minimum
- `templates/components/nav-tabs.html` - Increase tap targets for mobile bottom navigation
- `templates/components/recipe-card.html` - Add touch target sizing to action buttons
- `templates/base.html` - Update viewport meta tag for pinch-zoom control

**CSS Files:**
- `tailwind.config.js` - Define min-touch-target utility classes
- `static/css/tailwind.css` - Add custom touch target styles if needed

**JavaScript Enhancements (Optional):**
- `static/js/touch-gestures.js` - Long-press detection, swipe navigation, haptic feedback
- Feature detection for Vibration API: `if ('vibrate' in navigator)`

### Testing Standards Summary

**From Solution Architecture (Section 15 - Testing Strategy):**
- **Unit Tests**: Not applicable (pure CSS/HTML touch targets)
- **Integration Tests**: Verify touch target dimensions in rendered HTML
- **E2E Tests (Playwright)**:
  - Test touch target sizes programmatically (boundingBox >= 44x44)
  - Test swipe gestures on mobile viewport (375px iPhone SE)
  - Test long-press menus on recipe cards
  - Verify haptic feedback API calls (if supported)
  - Test pinch-to-zoom disabled on app UI, enabled on images

**Coverage Goal**: 80% code coverage (applies to touch gesture JavaScript, not CSS)

**TDD Approach**:
1. Write Playwright test verifying all buttons are >= 44x44px
2. Run test - should fail (current buttons may be smaller)
3. Apply min-touch-target classes to button.html
4. Run test - should pass (all buttons now meet WCAG requirement)
5. Refactor button component for clarity

### Project Structure Notes

**Alignment with Unified Project Structure:**
- Touch target styles defined in tailwind.config.js per architecture
- Button component follows existing structure: templates/components/button.html
- Touch gesture JavaScript (if needed) in static/js/ directory
- E2E tests in e2e/tests/touch-optimization.spec.ts

**Naming Conventions (Section 13.3):**
- CSS classes: kebab-case (`.touch-target`, `.haptic-feedback`)
- Tailwind utilities: `min-h-touch-target`, `min-w-touch-target`
- JavaScript functions: camelCase (`enableHapticFeedback`, `detectLongPress`)

**Detected Conflicts:**
- None - touch optimization integrates seamlessly with existing responsive design (Story 5.4)
- Button components already exist, will be enhanced with touch target classes

### References

- [Source: docs/solution-architecture.md#Section 7.4 - Accessibility]
  WCAG 2.1 Level AA touch target minimum: 44x44px
  Touch targets: 44x44px minimum (Tailwind defaults)

- [Source: docs/solution-architecture.md#Section 7.3 - Responsive Design]
  Mobile-first approach with touch-optimized interactions
  No hover-dependent functionality for critical features

- [Source: docs/tech-spec-epic-5.md#Module 3 - Responsive Design System]
  Tailwind config: `minHeight/minWidth: 'touch-target': '44px'`
  Touch target pattern for buttons, links, inputs

- [Source: docs/tech-spec-epic-5.md#Story 5: Touch Optimization]
  Full acceptance criteria and technical implementation details
  Haptic feedback (Vibration API), passive event listeners, pinch-zoom control

- [Source: docs/epics.md#Story 5.5 - Touch-Optimized Interface]
  User story, acceptance criteria, prerequisites
  Touch gestures: swipe, long-press, haptic feedback

- [Source: WCAG 2.1 Level AA - Target Size (2.5.5)]
  Touch targets must be at least 44 CSS pixels in both width and height
  Exception: Inline links within text blocks

## Dev Agent Record

### Context Reference

- `/home/snapiz/projects/github/timayz/imkitchen/docs/story-context-5.5.5.xml` (Generated: 2025-10-19)

### Agent Model Used

claude-sonnet-4-5-20250929

### Debug Log References

### Completion Notes List

### File List

## Change Log

| Date | Author | Change Description |
|------|--------|-------------------|
| 2025-10-19 | Bob (Scrum Master) | Initial story creation from Epic 5, Story 5.5 |

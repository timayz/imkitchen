# Story 5.6: Kitchen-Friendly Display Modes

Status: Approved

## Story

As a user cooking in kitchen,
I want high-contrast, large-text display option,
so that I can read recipes in various lighting conditions.

## Acceptance Criteria

1. "Kitchen Mode" toggle in recipe detail view
2. Kitchen mode increases text size (20px body, 28px headings)
3. High contrast styling: dark text on light background, increased contrast ratio (7:1)
4. Simplified UI: hide non-essential elements, focus on instructions
5. Step-by-step mode: display one instruction at a time with large "Next" button
6. Keep-awake functionality prevents screen from sleeping while cooking
7. Mode persists across recipe views (stored in user preference)
8. Easy toggle to return to normal mode

## Tasks / Subtasks

- [ ] Task 1: Implement Kitchen Mode toggle UI (AC: 1)
  - [ ] Subtask 1.1: Add toggle button to recipe detail page header
  - [ ] Subtask 1.2: Design toggle icon (chef hat or kitchen icon + text label)
  - [ ] Subtask 1.3: Position toggle prominently for easy thumb access on mobile
  - [ ] Subtask 1.4: Add aria-label and role attributes for accessibility
  - [ ] Subtask 1.5: Create E2E test for toggle button presence and functionality

- [ ] Task 2: Increase text sizing in Kitchen Mode (AC: 2)
  - [ ] Subtask 2.1: Create .kitchen-mode CSS class in Tailwind config
  - [ ] Subtask 2.2: Define typography scale: body 20px, headings 28px
  - [ ] Subtask 2.3: Apply .kitchen-mode to body element on toggle
  - [ ] Subtask 2.4: Test text legibility at 3-5 feet distance (kitchen counter scenario)
  - [ ] Subtask 2.5: Verify responsive scaling on mobile/tablet/desktop

- [ ] Task 3: Implement high-contrast color scheme (AC: 3)
  - [ ] Subtask 3.1: Define high-contrast palette: near-black text (#1a1a1a) on white background
  - [ ] Subtask 3.2: Increase border weights and button outlines
  - [ ] Subtask 3.3: Calculate and verify 7:1 contrast ratio with color contrast tool
  - [ ] Subtask 3.4: Remove decorative elements (shadows, gradients) that reduce contrast
  - [ ] Subtask 3.5: Test in various lighting: bright kitchen, dim evening, outdoor daylight

- [ ] Task 4: Simplify UI for focused cooking experience (AC: 4)
  - [ ] Subtask 4.1: Hide navigation sidebar/header in Kitchen Mode
  - [ ] Subtask 4.2: Remove non-essential recipe metadata (ratings, tags, sharing buttons)
  - [ ] Subtask 4.3: Prioritize ingredients list and instructions only
  - [ ] Subtask 4.4: Increase whitespace between elements for visual clarity
  - [ ] Subtask 4.5: Create simplified template partial for kitchen mode view

- [ ] Task 5: Build step-by-step instruction mode (AC: 5)
  - [ ] Subtask 5.1: Create step navigator component (current step indicator, total steps)
  - [ ] Subtask 5.2: Display one instruction step at a time in large type
  - [ ] Subtask 5.3: Add large "Next" button (min 60x60px for easy thumb tap)
  - [ ] Subtask 5.4: Add "Previous" button for navigation back
  - [ ] Subtask 5.5: Support keyboard navigation (arrow keys, spacebar for next)
  - [ ] Subtask 5.6: Add step completion checkmarks for progress tracking
  - [ ] Subtask 5.7: Show ingredient list reference panel (collapsible)

- [ ] Task 6: Implement Keep-Awake functionality (AC: 6)
  - [ ] Subtask 6.1: Detect Wake Lock API support with feature detection
  - [ ] Subtask 6.2: Request wake lock when Kitchen Mode enabled
  - [ ] Subtask 6.3: Release wake lock when Kitchen Mode disabled or user navigates away
  - [ ] Subtask 6.4: Handle wake lock errors gracefully (permission denied, battery saver)
  - [ ] Subtask 6.5: Show visual indicator when wake lock is active
  - [ ] Subtask 6.6: Test on iOS Safari and Android Chrome for wake lock behavior

- [ ] Task 7: Persist Kitchen Mode preference (AC: 7)
  - [ ] Subtask 7.1: Store kitchen_mode_enabled in localStorage
  - [ ] Subtask 7.2: Auto-enable Kitchen Mode on recipe view if preference set
  - [ ] Subtask 7.3: Persist across browser sessions and device switches
  - [ ] Subtask 7.4: Add user setting in Profile page to set default Kitchen Mode
  - [ ] Subtask 7.5: Clear preference on explicit toggle off

- [ ] Task 8: Easy toggle to return to normal mode (AC: 8)
  - [ ] Subtask 8.1: Display "Exit Kitchen Mode" button prominently
  - [ ] Subtask 8.2: Clicking exit restores normal recipe view instantly
  - [ ] Subtask 8.3: Preserve scroll position when toggling modes
  - [ ] Subtask 8.4: Add confirmation dialog if user has progress in step-by-step mode
  - [ ] Subtask 8.5: Test toggle experience for smooth transition

- [ ] Task 9: Comprehensive Kitchen Mode testing (All ACs)
  - [ ] Subtask 9.1: Create Playwright tests for Kitchen Mode toggle and UI changes
  - [ ] Subtask 9.2: Test text size increases (measure font sizes programmatically)
  - [ ] Subtask 9.3: Test contrast ratios with accessibility tools
  - [ ] Subtask 9.4: Test step-by-step navigation (next/previous buttons)
  - [ ] Subtask 9.5: Test Wake Lock API integration (mock API for CI)
  - [ ] Subtask 9.6: Test localStorage persistence across sessions
  - [ ] Subtask 9.7: Manual testing in real kitchen environment (lighting variations)

## Dev Notes

### Architecture Patterns and Constraints

**From Solution Architecture (docs/solution-architecture.md):**
- **Section 7.4 - Accessibility**: Kitchen Mode requires 7:1 contrast ratio for WCAG Level AAA compliance
- **Section 7 - UI/UX Architecture**: Kitchen-friendly display mode with high contrast and large text options
- **Section 8.3 - PWA Offline Strategy**: Kitchen Mode must work offline when recipe cached
- **Wake Lock API**: Browser API to prevent screen from sleeping during active cooking

**From Tech Spec Epic 5 (docs/tech-spec-epic-5.md):**
- **Story 5.6 - Kitchen-Friendly Display Modes**:
  - Kitchen mode CSS class applied to body: `.kitchen-mode`
  - Typography: 20px body, 28px headings (vs normal 16px/24px)
  - High contrast: #1a1a1a text on #ffffff background (7:1 ratio)
  - Simplified UI: hide nav, ratings, tags - focus on ingredients + instructions
  - Step-by-step mode with large next/previous buttons (60x60px minimum)
  - Wake Lock API: `navigator.wakeLock.request('screen')` when enabled
  - LocalStorage key: `kitchen_mode_enabled` (boolean)
  - URL param alternative: `?mode=kitchen` for shareable links

**Kitchen Mode Design Principles**:
1. **Legibility First**: Text must be readable at 3-5 feet distance (typical kitchen counter to user)
2. **No Clutter**: Remove all non-essential UI elements (navigation, social features, decorative elements)
3. **Touch-Optimized**: Large touch targets (60x60px for primary actions) for messy/wet hands
4. **Progressive Disclosure**: Show one instruction step at a time to reduce cognitive load while cooking
5. **Persistent State**: Remember user's Kitchen Mode preference across sessions

### Source Tree Components

**Templates to Create/Modify:**
- `templates/pages/recipe-detail.html` - Add Kitchen Mode toggle button
- `templates/components/kitchen-mode-toggle.html` - NEW: Toggle button component
- `templates/partials/kitchen-mode-view.html` - NEW: Simplified kitchen view template
- `templates/partials/step-by-step-navigator.html` - NEW: Step navigator component
- `templates/base.html` - Conditional .kitchen-mode class on body

**CSS Files:**
- `static/css/tailwind.css` - Add .kitchen-mode utility classes:
  ```css
  @layer components {
    .kitchen-mode {
      --text-base-size: 20px;
      --text-heading-size: 28px;
      --text-color: #1a1a1a;
      --bg-color: #ffffff;
      --contrast-ratio: 7; /* WCAG Level AAA */
    }

    .kitchen-mode body {
      font-size: var(--text-base-size);
      color: var(--text-color);
      background: var(--bg-color);
    }

    .kitchen-mode h1, .kitchen-mode h2 {
      font-size: var(--text-heading-size);
      font-weight: 700;
    }

    .kitchen-mode .touch-target {
      min-width: 60px;
      min-height: 60px; /* Increased from 44px for easier thumb access */
    }

    .kitchen-mode .non-essential {
      display: none; /* Hide navigation, ratings, tags */
    }
  }
  ```

**JavaScript Enhancements:**
- `static/js/kitchen-mode.js` - NEW: Kitchen Mode toggle logic, Wake Lock API integration
  - Toggle kitchen mode on/off
  - Request/release wake lock
  - Manage localStorage preference
  - Step-by-step navigation logic

### Testing Standards Summary

**From Solution Architecture (Section 15 - Testing Strategy):**
- **Unit Tests**: Not applicable (CSS/HTML/localStorage-focused story)
- **Integration Tests**: Verify Kitchen Mode CSS classes applied, localStorage read/write
- **E2E Tests (Playwright)**:
  - Toggle Kitchen Mode on recipe detail page
  - Verify text size increases (measure computed styles)
  - Test contrast ratio with accessibility tools
  - Navigate step-by-step instructions (next/previous)
  - Verify Wake Lock API called (mock API for CI)
  - Test localStorage persistence across page refreshes
  - Manual testing in real kitchen lighting conditions

**Coverage Goal**: 80% code coverage for kitchen-mode.js (Wake Lock logic, toggle state management)

**TDD Approach**:
1. Write Playwright test: toggle Kitchen Mode and verify `.kitchen-mode` class on body
2. Run test - should fail (Kitchen Mode not implemented)
3. Implement Kitchen Mode toggle button and CSS class application
4. Run test - should pass (body has `.kitchen-mode` class)
5. Write test: verify text sizes are 20px/28px in Kitchen Mode
6. Implement CSS typography changes
7. Run test - should pass
8. Refactor for clarity and accessibility

### Project Structure Notes

**Alignment with Unified Project Structure:**
- Kitchen Mode CSS follows existing Tailwind component layer pattern
- JavaScript module in `static/js/kitchen-mode.js` per architecture guidelines
- New template partials in `templates/partials/` directory
- E2E tests in `e2e/tests/kitchen-mode.spec.ts`

**Naming Conventions (Section 13.3):**
- CSS classes: kebab-case (`.kitchen-mode`, `.step-navigator`, `.non-essential`)
- Tailwind utilities: `kitchen-mode`, `kitchen-text-lg`, `kitchen-heading`
- JavaScript functions: camelCase (`enableKitchenMode`, `requestWakeLock`, `toggleMode`)
- LocalStorage keys: snake_case (`kitchen_mode_enabled`)

**Detected Conflicts:**
- None - Kitchen Mode is an additive feature overlay
- Integrates with Story 5.5 touch targets (increases to 60x60px in Kitchen Mode)
- Works with Story 5.3 offline recipe access (no server requests needed)

### References

- [Source: docs/solution-architecture.md#Section 7.4 - Accessibility]
  Kitchen Mode high contrast: 7:1 contrast ratio for WCAG Level AAA
  Large text option: 20px body, 28px headings minimum

- [Source: docs/solution-architecture.md#Section 7 - UI/UX Architecture]
  Kitchen-friendly display mode with high contrast and large text options
  Progressive disclosure for instructions

- [Source: docs/tech-spec-epic-5.md#Story 5.6 - Kitchen-Friendly Display Modes]
  Full acceptance criteria and technical implementation details
  Wake Lock API: prevents screen from sleeping while cooking
  LocalStorage key: kitchen_mode_enabled

- [Source: docs/epics.md#Story 5.6 - Kitchen-Friendly Display Modes]
  User story: cooking in kitchen, read recipes in various lighting
  Step-by-step mode with large "Next" button

- [Source: MDN Web Docs - Screen Wake Lock API]
  https://developer.mozilla.org/en-US/docs/Web/API/Screen_Wake_Lock_API
  Browser support: Chrome 84+, Edge 84+, Safari 16.4+ (iOS), not Firefox

- [Source: WCAG 2.1 Level AAA - Contrast (Enhanced) 1.4.6]
  https://www.w3.org/WAI/WCAG21/Understanding/contrast-enhanced.html
  Contrast ratio of at least 7:1 for normal text

## Dev Agent Record

### Context Reference

- `/home/snapiz/projects/github/timayz/imkitchen/docs/story-context-5.6.xml` (Generated: 2025-10-19)

### Agent Model Used

claude-sonnet-4-5-20250929

### Debug Log References

### Completion Notes List

### File List

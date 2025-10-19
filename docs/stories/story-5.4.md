# Story 5.4: Mobile-Responsive Design

Status: Approved

## Story

As a user on mobile device,
I want optimized interface for small screens,
so that I can use app comfortably on phone.

## Acceptance Criteria

1. Responsive breakpoints: mobile (<768px), tablet (768-1024px), desktop (>1024px)
2. Mobile layout: single-column stacking, full-width cards, bottom navigation
3. Tablet layout: 2-column grid for recipe cards, side navigation
4. Desktop layout: multi-column grid, persistent sidebar navigation
5. Text sizes scale appropriately (16px minimum for body text on mobile)
6. Images responsive with srcset for different screen densities
7. Form inputs sized appropriately for thumb typing
8. Navigation accessible without excessive scrolling

## Tasks / Subtasks

- [ ] Task 1: Configure Tailwind responsive breakpoints and viewport meta tag (AC: 1)
  - [ ] Subtask 1.1: Update tailwind.config.js with custom breakpoints (sm: 768px, md: 1024px)
  - [ ] Subtask 1.2: Add viewport meta tag to base.html template (width=device-width, initial-scale=1)
  - [ ] Subtask 1.3: Test breakpoints across devices using browser DevTools

- [ ] Task 2: Implement mobile-first layout patterns (AC: 2, 5)
  - [ ] Subtask 2.1: Refactor base.html template to use mobile-first single-column layout
  - [ ] Subtask 2.2: Convert recipe cards to full-width on mobile (<768px)
  - [ ] Subtask 2.3: Implement bottom navigation bar component for mobile (fixed position)
  - [ ] Subtask 2.4: Set minimum text size to 16px for body text on mobile (prevents zoom on iOS)
  - [ ] Subtask 2.5: Test stacking behavior on iPhone SE (375px width) and Pixel 5 (393px width)

- [ ] Task 3: Create tablet-specific layout adaptations (AC: 3)
  - [ ] Subtask 3.1: Implement 2-column grid for recipe cards on tablet (768-1024px breakpoint)
  - [ ] Subtask 3.2: Add side navigation component for tablet view (replaces bottom nav)
  - [ ] Subtask 3.3: Adjust padding and margins for tablet screen real estate
  - [ ] Subtask 3.4: Test on iPad (768px) and Surface Pro (912px)

- [ ] Task 4: Build desktop multi-column layouts (AC: 4)
  - [ ] Subtask 4.1: Implement 3-4 column grid for recipe library on desktop (>1024px)
  - [ ] Subtask 4.2: Create persistent sidebar navigation component for desktop
  - [ ] Subtask 4.3: Optimize meal calendar for horizontal week view on desktop
  - [ ] Subtask 4.4: Test on 1920x1080 and 2560x1440 resolutions

- [ ] Task 5: Implement responsive image loading strategies (AC: 6)
  - [ ] Subtask 5.1: Add srcset attributes to recipe images with 1x, 2x, 3x variants
  - [ ] Subtask 5.2: Generate image variants during recipe upload (thumbnail, medium, large)
  - [ ] Subtask 5.3: Use picture element with media queries for art direction (square on mobile, landscape on desktop)
  - [ ] Subtask 5.4: Implement lazy loading (loading="lazy") for below-fold images
  - [ ] Subtask 5.5: Test on retina displays (iPhone 13, MacBook Pro)

- [ ] Task 6: Optimize form inputs for mobile interaction (AC: 7)
  - [ ] Subtask 6.1: Increase input field height to minimum 44px on mobile
  - [ ] Subtask 6.2: Add appropriate input types (type="tel", type="email") for mobile keyboards
  - [ ] Subtask 6.3: Implement larger touch targets for dropdowns and selects (44x44px minimum)
  - [ ] Subtask 6.4: Add autocomplete attributes for autofill support
  - [ ] Subtask 6.5: Test thumb typing ergonomics on various device sizes

- [ ] Task 7: Ensure navigation accessibility without scrolling (AC: 8)
  - [ ] Subtask 7.1: Implement sticky header on mobile with key nav links
  - [ ] Subtask 7.2: Ensure bottom navigation (mobile) is always visible (fixed position)
  - [ ] Subtask 7.3: Add "back to top" button for long pages on mobile
  - [ ] Subtask 7.4: Test navigation reach on iPhone SE (smallest supported device)

- [ ] Task 8: Cross-device testing and refinement (All ACs)
  - [ ] Subtask 8.1: Test on physical devices (iPhone, Android, iPad)
  - [ ] Subtask 8.2: Run Playwright tests with mobile viewport emulation
  - [ ] Subtask 8.3: Validate using Chrome DevTools responsive mode (all breakpoints)
  - [ ] Subtask 8.4: Address layout bugs and edge cases discovered during testing
  - [ ] Subtask 8.5: Verify text readability and touch target sizes meet WCAG 2.1 Level AA

## Dev Notes

### Architecture Patterns and Constraints

**From Solution Architecture (docs/solution-architecture.md):**
- Server-Side Rendering: Askama templates compile responsive classes at build time
- Tailwind CSS responsive utilities: Apply breakpoint prefixes (sm:, md:, lg:) to utility classes
- Mobile-first approach: Base styles for mobile (<768px), enhance for larger screens
- Progressive enhancement: HTML works without CSS, responsive CSS enhances experience
- No client-side routing: Traditional multi-page app with server-rendered HTML

**Responsive Design Pattern (Section 7.3):**
```html
<div class="flex flex-col md:flex-row lg:grid lg:grid-cols-3">
  <!-- Mobile: stack vertically -->
  <!-- Tablet: horizontal flex -->
  <!-- Desktop: 3-column grid -->
</div>
```

**Breakpoints (tailwind.config.js):**
- Mobile: 0-767px (default, no prefix)
- Tablet: 768px-1023px (`md:` prefix)
- Desktop: 1024px+ (`lg:` prefix)

**Navigation Adaptation (Section 7.3):**
- Mobile: Bottom tab bar (fixed position)
- Desktop: Left sidebar with icons + labels

### Source Tree Components

**Templates to Modify:**
- `templates/base.html` - Add viewport meta tag, update navigation patterns
- `templates/components/nav-tabs.html` - Bottom navigation for mobile
- `templates/components/recipe-card.html` - Responsive card layout variants
- `templates/pages/recipe-list.html` - Grid layout with responsive columns
- `templates/pages/meal-calendar.html` - Adapt calendar for mobile (1 day) vs desktop (7 days)

**CSS Configuration:**
- `tailwind.config.js` - Configure responsive breakpoints and design tokens
- `static/css/tailwind.css` - Custom responsive utilities if needed

**Static Assets:**
- `static/images/` - Generate responsive image variants (1x, 2x, 3x)

### Testing Standards Summary

**From Solution Architecture (Section 15):**
- Unit Tests: Not applicable (pure CSS/HTML changes)
- Integration Tests: Verify responsive breakpoints render correctly at different viewport widths
- E2E Tests (Playwright): Test user flows on mobile (iPhone SE 375px), tablet (iPad 768px), desktop (1920px)
  - Navigate recipe library and verify card layout adapts
  - Test bottom navigation on mobile, sidebar navigation on desktop
  - Verify form inputs are usable on mobile (thumb typing, autocomplete)
  - Validate image srcset loads appropriate resolution

**Coverage Goal:** 80% code coverage (applies to route handlers and domain logic, not pure CSS)

**TDD Approach:**
1. Write Playwright test verifying mobile layout (single column, bottom nav)
2. Run test - should fail (current layout not responsive)
3. Implement Tailwind responsive classes in templates
4. Run test - should pass (layout adapts to viewport)
5. Refactor template structure for clarity

### Project Structure Notes

**Alignment with Unified Project Structure:**
- Responsive templates located in `templates/` directory per architecture
- Tailwind config follows standard setup in `tailwind.config.js` at project root
- Static assets served from `static/` via Axum static file handler
- Image variants stored in `static/images/recipes/{recipe_id}/{size}/` structure

**Naming Conventions (Section 13.3):**
- Template files: kebab-case (`recipe-card.html`, `meal-calendar.html`)
- CSS classes: kebab-case (`.recipe-card`, `.btn-primary`, `.meal-slot`)
- Responsive utilities: Tailwind prefixes (`sm:`, `md:`, `lg:`)

**Detected Conflicts:**
- None - responsive design integrates seamlessly with existing server-rendered architecture
- TwinSpark AJAX behaviors remain functional across breakpoints (no conflicts)

### References

- [Source: docs/solution-architecture.md#Section 7.3 - Responsive Design]
  Breakpoints: Mobile 0-767px, Tablet 768-1023px, Desktop 1024px+
  Mobile-first approach with Tailwind responsive utilities

- [Source: docs/solution-architecture.md#Section 7.1 - Component Structure]
  Template hierarchy: base.html → components/ → pages/
  Navigation adaptation patterns for mobile vs desktop

- [Source: docs/tech-spec-epic-5.md#Module 1 - PWA Manifest Configuration]
  Viewport meta tag: `width=device-width, initial-scale=1`
  Orientation preference: portrait-primary for mobile

- [Source: docs/solution-architecture.md#Section 15 - Testing Strategy]
  E2E tests with Playwright: Cross-browser, mobile viewport emulation
  Coverage target: 80% (applies to route handlers, not pure CSS)

- [Source: docs/epics.md#Story 5.4 - Mobile-Responsive Design]
  Full acceptance criteria and technical notes
  Tailwind responsive utilities, mobile-first CSS, viewport meta tag

## Change Log

| Date | Author | Change Description |
|------|--------|-------------------|
| 2025-10-19 | Bob (Scrum Master) | Initial story creation from Epic 5, Story 5.4 |

## Dev Agent Record

### Context Reference

- [Story Context XML](/home/snapiz/projects/github/timayz/imkitchen/docs/story-context-5.5.4.xml) - Generated 2025-10-19

### Agent Model Used

claude-sonnet-4-5-20250929

### Debug Log References

### Completion Notes List

### File List

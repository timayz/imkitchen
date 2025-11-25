# Story 4.2: Week Carousel Navigation (Mobile)

Status: drafted

## Story

As a mobile user,
I want to swipe between weeks in the calendar,
So that I can easily navigate my meal plans on a touchscreen device.

## Acceptance Criteria

1. Mobile view displays one week at a time (7 days in carousel)
2. Swipe gestures navigate forward/backward between weeks
3. Week indicator shows current position (e.g., "Week 2 of 4")
4. Navigation arrows for non-touch devices
5. Smooth animations for week transitions
6. Tests verify swipe gesture handling and navigation

## Tasks / Subtasks

- [ ] Add mobile carousel structure to calendar template (AC: #1)
  - [ ] Modify `templates/pages/mealplan/calendar.html` to include mobile-specific markup
  - [ ] Create carousel container with week sections
  - [ ] Display one week at a time on mobile viewports (<768px)
  - [ ] Use CSS to hide/show weeks based on active index
- [ ] Implement week indicator component (AC: #3)
  - [ ] Create `templates/components/week-indicator.html` displaying "Week X of Y"
  - [ ] Calculate total weeks and current active week index
  - [ ] Style indicator with Tailwind CSS (fixed position at top)
  - [ ] Show/hide indicator on mobile viewports only
- [ ] Add navigation arrow buttons (AC: #4)
  - [ ] Add previous/next arrow buttons to calendar template
  - [ ] Position arrows on sides of carousel container
  - [ ] Implement Twinspark actions for week navigation
  - [ ] Style arrows with Tailwind (touch-optimized 44x44px minimum)
  - [ ] Disable previous arrow on Week 1, next arrow on last week
- [ ] Implement swipe gesture support (AC: #2)
  - [ ] Add minimal JavaScript for touch event handling in `static/js/swipe.js`
  - [ ] Detect swipe left (next week) and swipe right (previous week)
  - [ ] Trigger Twinspark action on swipe completion
  - [ ] Set swipe threshold (minimum distance and velocity)
  - [ ] Prevent vertical scrolling during horizontal swipe
- [ ] Add CSS transition animations (AC: #5)
  - [ ] Implement slide transition using Tailwind transition utilities
  - [ ] Animate week change with translateX transform
  - [ ] Set transition duration (300ms for smooth feel)
  - [ ] Add easing function (ease-in-out)
  - [ ] Ensure animations performant on mobile devices
- [ ] Create responsive breakpoints (AC: #1, #4)
  - [ ] Show carousel view on mobile (<768px)
  - [ ] Show full grid view on desktop (≥768px)
  - [ ] Hide carousel navigation on desktop
  - [ ] Use Tailwind responsive prefixes: `block md:hidden`, `hidden md:grid`
- [ ] Write integration and E2E tests (AC: #6)
  - [ ] Extend `tests/calendar_test.rs` with mobile viewport tests
  - [ ] Test carousel displaying one week at a time
  - [ ] Test week indicator showing correct position
  - [ ] Test arrow navigation (previous/next)
  - [ ] Create E2E test in `tests/e2e/mobile_calendar.spec.ts` for swipe gestures
  - [ ] Verify swipe left/right triggers week change
  - [ ] Test disabled states (first/last week)

## Dev Notes

### Architecture Patterns and Constraints

**Mobile-First Responsive Design:**
- Default mobile view: carousel with single week visible
- Desktop view (≥768px): full grid showing all weeks simultaneously
- Tailwind breakpoints: `block md:hidden` for mobile-only, `hidden md:grid` for desktop-only
- Touch-optimized UI per NFR005: 44x44px minimum tap targets for arrows

**Twinspark Navigation Actions:**
- Arrow buttons use Twinspark `ts-action` for week navigation without full page reload
- Actions update active week index stored in URL hash or query parameter
- Server returns partial HTML update for week indicator (optional optimization)
- No JavaScript framework required - minimal vanilla JS for swipe detection only

**Swipe Gesture Implementation:**
- Minimal JavaScript file (~50 lines) for touch event handling
- Touch start → record initial X position
- Touch move → track horizontal distance and prevent vertical scroll if horizontal swipe detected
- Touch end → if distance > threshold (50px) and velocity sufficient → trigger week change
- Integrates with Twinspark by programmatically triggering arrow button click
- Progressive enhancement: works without JS via arrow buttons

**Week State Management:**
- Active week index stored in URL parameter: `/mealplan?week=2`
- Server renders only active week's meals for mobile viewport
- Client-side navigation updates URL without page reload
- Browser back/forward buttons work naturally with URL parameter approach

**Animation Performance:**
- Use CSS transforms (translateX) for hardware-accelerated animations
- Avoid layout thrashing - animate transform property only
- Transition duration 300ms matches typical mobile gesture feel
- Test on lower-end mobile devices to ensure smooth 60fps

### Project Structure Notes

**Files to Create/Modify:**
- `templates/pages/mealplan/calendar.html` - Add mobile carousel structure (MODIFY)
- `templates/components/week-indicator.html` - Week position indicator (NEW)
- `static/js/swipe.js` - Touch gesture handling (~50 lines) (NEW)
- `static/css/input.css` - Add carousel transition styles if needed (MODIFY)
- `tests/calendar_test.rs` - Add mobile viewport tests (MODIFY)
- `tests/e2e/mobile_calendar.spec.ts` - E2E swipe gesture tests (NEW)

**Minimal JavaScript Justification:**
- CLAUDE.md prefers Twinspark over JavaScript, but swipe gestures require touch event handling
- Scope limited to touch detection only - all state management via Twinspark/server
- Progressive enhancement pattern: full functionality without JS via arrow buttons
- File size <2KB gzipped, no framework dependencies

**Route Handler Changes:**
- `src/routes/mealplan/calendar.rs` modified to accept `?week=N` query parameter
- Render all weeks for desktop, single active week for mobile (based on viewport detection or default)
- Return week count metadata for indicator component

**CSS Strategy:**
- Use Tailwind utilities for responsive display toggles
- Carousel container: `overflow-hidden` with fixed width
- Week sections: `transition-transform duration-300 ease-in-out`
- Arrow buttons: `fixed` positioning on mobile, `hidden` on desktop

**Visual Mockup Alignment:**
- Mobile carousel matches `mockups/calendar-free.html` single-week display
- Week navigation matches mobile-specific UI patterns
- Arrow buttons positioned for one-handed thumb reach

### References

- [Source: docs/epics.md#Story 4.2 - Acceptance Criteria and Prerequisites]
- [Source: docs/PRD.md#NFR001 - Mobile page load performance]
- [Source: docs/PRD.md#NFR005 - Mobile-responsive, touch-optimized interface]
- [Source: docs/PRD.md#NFR006 - Modern mobile browsers support]
- [Source: docs/architecture.md#HTTP Routes - /mealplan route with query parameters]
- [Source: CLAUDE.md#Server-Side Rendering Rules - Twinspark for UI reactivity, minimal JavaScript]
- [Source: CLAUDE.md#Askama Guidelines - Template responsive breakpoints]

## Dev Agent Record

### Context Reference

<!-- Path(s) to story context XML will be added here by context workflow -->

### Agent Model Used

<!-- Model information will be added during implementation -->

### Debug Log References

<!-- Debug logs will be added during implementation -->

### Completion Notes List

<!-- Completion notes will be added during implementation -->

### File List

<!-- Files created/modified will be listed during implementation -->

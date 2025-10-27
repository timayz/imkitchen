# Story 9.7: Responsive Design and Accessibility Testing

Status: Approved

## Story

As a frontend developer,
I want to ensure all new UI is responsive and accessible,
so that users on mobile and assistive technology can use features.

## Acceptance Criteria

1. All pages tested on mobile (375px), tablet (768px), desktop (1920px) - manual testing log
2. Calendar: Tabs on desktop, carousel on mobile (responsive breakpoint @md:)
3. Forms: Full-width inputs on mobile (w-full), constrained on desktop (max-w-md)
4. Modals: Centered on desktop with backdrop, full-height on mobile
5. Touch targets ≥44x44px on mobile (buttons, checkboxes, links) - WCAG AA compliance
6. Keyboard navigation works for all interactive elements (Tab order logical, Enter/Space activate)
7. Screen reader testing: All content accessible with NVDA/VoiceOver (ARIA labels, semantic HTML)
8. Color contrast ratios meet WCAG AA: 4.5:1 for normal text, 3:1 for large text (verified with WAVE tool)
9. Focus indicators visible for all interactive elements (2px border, 4px offset, primary-500 color)
10. Lighthouse accessibility score >90 for all pages (automated test in CI)

## Tasks / Subtasks

- [ ] Create responsive testing checklist and log (AC: #1)
  - [ ] Document test plan: pages, breakpoints, test scenarios
  - [ ] Create spreadsheet or markdown file: `docs/testing/epic-9-responsive-test-log.md`
  - [ ] Define test cases for each page (calendar, preferences form, recipe form, shopping list)
  - [ ] Log results with screenshots for each breakpoint

- [ ] Test responsive breakpoints (AC: #1, #2, #3, #4)
  - [ ] Mobile (375px): Test all Epic 9 pages
    - [ ] Calendar: Carousel with swipe navigation
    - [ ] Forms: Full-width inputs, stacked buttons
    - [ ] Modals: Full-height, easy to dismiss
    - [ ] Shopping list: Full-width dropdown
  - [ ] Tablet (768px): Test transition between mobile and desktop layouts
    - [ ] Calendar: Verify tabs appear at @md: breakpoint
    - [ ] Forms: Verify constrained width begins
  - [ ] Desktop (1920px): Test large screen layouts
    - [ ] Calendar: Week tabs prominent, no overflow
    - [ ] Forms: Centered with max-width constraints
    - [ ] Modals: Centered with backdrop
  - [ ] Document any layout issues or fixes needed

- [ ] Verify touch target sizes on mobile (AC: #5)
  - [ ] Use browser DevTools ruler tool or accessibility inspector
  - [ ] Measure all buttons: "Regenerate This Week", "Save Preferences", form submit buttons
  - [ ] Measure all checkboxes and radio buttons in forms
  - [ ] Measure all links: recipe names, accompaniment links, navigation links
  - [ ] Ensure minimum 44x44px for all interactive elements (WCAG AA)
  - [ ] Fix any elements below threshold (increase padding or min-height)

- [ ] Test keyboard navigation (AC: #6)
  - [ ] Multi-week calendar:
    - [ ] Tab through week tabs, verify logical order (left to right)
    - [ ] Enter/Space activates week tab, loads new content
    - [ ] Shift+Tab navigates backward
  - [ ] Preferences form:
    - [ ] Tab through all inputs (time, checkboxes, slider, custom text)
    - [ ] Space toggles checkboxes
    - [ ] Enter submits form from focused submit button
  - [ ] Recipe form:
    - [ ] Tab through recipe type radios, accompaniment checkboxes, dietary tags
    - [ ] Space/Enter activates radios and checkboxes
  - [ ] Shopping list:
    - [ ] Tab to week dropdown, arrow keys change selection
    - [ ] Tab through shopping list items and checkboxes
  - [ ] Regeneration modals:
    - [ ] Tab cycles between Cancel and Confirm buttons
    - [ ] Escape closes modal
    - [ ] Focus trap: Tab doesn't leave modal when open
  - [ ] Document any tab order issues or missing keyboard support

- [ ] Screen reader testing (AC: #7)
  - [ ] Test with NVDA (Windows) or VoiceOver (Mac/iOS)
  - [ ] Multi-week calendar:
    - [ ] Week tabs announced as buttons with week number and dates
    - [ ] Meal slots announced with recipe name, prep time, accompaniment
    - [ ] Lock icon described: "Current week, locked"
  - [ ] Forms:
    - [ ] All labels associated with inputs (for/id attributes)
    - [ ] Fieldsets and legends announced for grouped inputs
    - [ ] Error messages announced when validation fails
  - [ ] Modals:
    - [ ] Modal role="dialog" announced
    - [ ] Modal title (aria-labelledby) announced on open
    - [ ] Modal description (aria-describedby) read by screen reader
  - [ ] Shopping list:
    - [ ] Week dropdown label announced
    - [ ] Shopping list items announced with checkbox state
    - [ ] Category headings announced (h3 elements)
  - [ ] Document any missing ARIA labels or semantic HTML issues

- [ ] Color contrast testing (AC: #8)
  - [ ] Use WAVE tool or Chrome DevTools accessibility inspector
  - [ ] Test all text colors against backgrounds:
    - [ ] Normal text (body copy): Verify 4.5:1 contrast ratio
    - [ ] Large text (headings): Verify 3:1 contrast ratio
    - [ ] Button text: Verify 4.5:1 contrast on primary/secondary buttons
    - [ ] Link text: Verify 4.5:1 contrast, underline or distinguishable
    - [ ] Accompaniment text (gray-600): Verify meets 4.5:1 threshold
    - [ ] Disabled text: Verify visually distinguishable (may be exempt from contrast requirement)
  - [ ] Fix any contrast failures (adjust colors or font weights)
  - [ ] Re-test after fixes

- [ ] Focus indicator testing (AC: #9)
  - [ ] Navigate with keyboard to all interactive elements
  - [ ] Verify visible focus indicators (2px border, 4px offset, primary-500 color)
  - [ ] Test focus indicators on:
    - [ ] Week tabs
    - [ ] Form inputs (text, checkboxes, radio buttons, sliders)
    - [ ] Buttons (submit, cancel, regenerate)
    - [ ] Links (recipe names, accompaniments)
    - [ ] Dropdown selects (week selector)
  - [ ] Ensure focus indicators meet contrast requirements (3:1 against background)
  - [ ] Fix any missing or low-contrast focus indicators

- [ ] Lighthouse accessibility audit (AC: #10)
  - [ ] Run Lighthouse in Chrome DevTools for each page:
    - [ ] Multi-week calendar page (`/plan`)
    - [ ] Meal planning preferences form (`/profile/meal-planning-preferences`)
    - [ ] Recipe creation form (`/recipes/new`)
    - [ ] Shopping list page (`/shopping`)
  - [ ] Verify accessibility score >90 for all pages
  - [ ] Review specific audit failures (if any):
    - [ ] Missing ARIA labels
    - [ ] Insufficient contrast ratios
    - [ ] Missing alt text on images
    - [ ] Form inputs without labels
    - [ ] Heading hierarchy issues
  - [ ] Fix all critical and serious issues
  - [ ] Re-run Lighthouse to confirm score >90

- [ ] Automated accessibility testing in CI
  - [ ] Add Lighthouse CI to pipeline (or similar tool: axe-core, Pa11y)
  - [ ] Configure threshold: accessibility score >90 required to pass
  - [ ] Run on key pages: /plan, /profile/meal-planning-preferences, /recipes/new, /shopping
  - [ ] Set up CI workflow: fail build if score <90
  - [ ] Test CI workflow: Verify build fails on accessibility violations

- [ ] Cross-browser testing
  - [ ] Desktop:
    - [ ] Chrome 120+ (primary browser)
    - [ ] Firefox 115+ (secondary browser)
    - [ ] Safari 17+ (Mac users)
    - [ ] Edge 120+ (Windows users)
  - [ ] Mobile:
    - [ ] iOS Safari 17+ (iPhone)
    - [ ] Android Chrome 120+ (Android devices)
  - [ ] Document any browser-specific issues:
    - [ ] TwinSpark compatibility
    - [ ] Tailwind CSS rendering differences
    - [ ] Focus indicator styles
    - [ ] Form input behavior

- [ ] Create comprehensive test report
  - [ ] Document all test results in `docs/testing/epic-9-accessibility-report.md`
  - [ ] Include screenshots of key pages at each breakpoint
  - [ ] List all issues found and resolutions
  - [ ] Lighthouse scores for each page (with screenshots)
  - [ ] WAVE tool results for contrast and ARIA
  - [ ] Screen reader test notes (pass/fail for each flow)
  - [ ] Keyboard navigation test results
  - [ ] Sign-off section for accessibility lead approval

## Dev Notes

### Architecture Patterns and Constraints

- **Mobile-First Design:** Start with mobile layouts, enhance for larger screens with Tailwind breakpoints
- **WCAG AA Compliance:** Target accessibility level required by law in many jurisdictions
- **Progressive Enhancement:** Features work without JavaScript, enhanced with TwinSpark
- **Semantic HTML:** Use proper HTML5 elements (nav, main, article, section, aside) for screen readers

### Source Tree Components

**Testing Documentation:**
- `docs/testing/epic-9-responsive-test-log.md` - Manual responsive testing log (create)
- `docs/testing/epic-9-accessibility-report.md` - Comprehensive accessibility report (create)

**CI Configuration:**
- `.github/workflows/accessibility.yml` - Lighthouse CI workflow (create or update)
- `lighthouserc.json` - Lighthouse CI configuration (create)

**Templates (Testing Focus):**
- All Epic 9 templates: multi_week_calendar.html, meal_planning_preferences.html, create_recipe.html, shopping_list.html
- Review and fix accessibility issues found during testing

### Testing Standards

- **WCAG AA Compliance:** Industry standard for web accessibility
- **Lighthouse Score >90:** High bar ensuring accessibility quality
- **Manual Testing Required:** Automated tools catch ~40% of accessibility issues
- **Screen Reader Testing:** Essential for understanding user experience with assistive technology

### Testing Tools

**Manual Testing:**
- Browser DevTools (responsive mode, accessibility inspector)
- Real devices (iPhone, Android phone, tablets)
- Ruler tool for measuring touch targets

**Screen Readers:**
- **NVDA** (Windows, free): https://www.nvaccess.org/
- **VoiceOver** (Mac/iOS, built-in): Cmd+F5 to enable
- **JAWS** (Windows, paid): Industry standard for Windows

**Automated Testing:**
- **Lighthouse** (Chrome DevTools): Performance, accessibility, SEO audits
- **WAVE** (Browser extension): Visual contrast and ARIA checker
- **axe DevTools** (Browser extension): Detailed accessibility issues
- **Pa11y** (CLI tool): Automated accessibility testing

**CI Integration:**
- **Lighthouse CI**: Run Lighthouse in CI pipeline, fail on low scores
- **axe-core**: JavaScript library for accessibility testing in Playwright tests

### Responsive Breakpoints (Tailwind CSS)

```
- Mobile:   < 768px  (sm: breakpoint)
- Tablet:   768px - 1024px  (md: breakpoint)
- Desktop:  > 1024px  (lg:, xl:, 2xl: breakpoints)
```

**Tailwind Responsive Utilities:**
- `w-full md:max-w-md` - Full width on mobile, max-width on tablet+
- `flex-col md:flex-row` - Stacked on mobile, horizontal on tablet+
- `text-sm md:text-base` - Smaller text on mobile, normal on tablet+

### WCAG AA Requirements Summary

**1. Contrast Ratios:**
- Normal text (<18pt): 4.5:1 minimum
- Large text (≥18pt or ≥14pt bold): 3:1 minimum
- UI components (buttons, form borders): 3:1 minimum

**2. Touch Targets:**
- Minimum size: 44x44px (WCAG 2.5.5, AAA level)
- Best practice: 48x48px for primary actions

**3. Keyboard Navigation:**
- All functionality available via keyboard
- Visible focus indicators (2px border, 4px offset recommended)
- Logical tab order matching visual flow

**4. Screen Reader Support:**
- Semantic HTML (headings, lists, nav, main)
- ARIA labels for non-obvious elements (icons, complex widgets)
- Form labels associated with inputs (for/id or aria-labelledby)
- Error announcements (aria-live, aria-invalid)

**5. Text Alternatives:**
- Alt text for all informative images
- Empty alt for decorative images (alt="")
- ARIA labels for icon-only buttons

### Lighthouse CI Configuration

```json
// lighthouserc.json
{
  "ci": {
    "collect": {
      "url": [
        "http://localhost:3000/plan",
        "http://localhost:3000/profile/meal-planning-preferences",
        "http://localhost:3000/recipes/new",
        "http://localhost:3000/shopping"
      ],
      "numberOfRuns": 3
    },
    "assert": {
      "assertions": {
        "categories:accessibility": ["error", {"minScore": 0.9}],
        "categories:performance": ["warn", {"minScore": 0.8}],
        "categories:seo": ["warn", {"minScore": 0.9}]
      }
    },
    "upload": {
      "target": "temporary-public-storage"
    }
  }
}
```

### Common Accessibility Issues and Fixes

**Issue: Missing Form Labels**
```html
<!-- Bad -->
<input type="text" name="max_prep_time" />

<!-- Good -->
<label for="max_prep_time">Max prep time (minutes)</label>
<input type="text" id="max_prep_time" name="max_prep_time" />
```

**Issue: Low Contrast Text**
```html
<!-- Bad: text-gray-400 on white (2.5:1 contrast) -->
<p class="text-gray-400">Accompaniment text</p>

<!-- Good: text-gray-600 on white (4.5:1 contrast) -->
<p class="text-gray-600">Accompaniment text</p>
```

**Issue: Missing Focus Indicators**
```css
/* Bad: Browser default focus outline removed */
button:focus { outline: none; }

/* Good: Custom focus indicator with high contrast */
button:focus {
  outline: 2px solid theme('colors.primary.500');
  outline-offset: 4px;
}
```

**Issue: Icon-Only Button Without Label**
```html
<!-- Bad: Screen reader announces "button" with no context -->
<button><i class="icon-regenerate"></i></button>

<!-- Good: ARIA label provides context -->
<button aria-label="Regenerate this week">
  <i class="icon-regenerate"></i>
</button>
```

### Project Structure Notes

**New Files:**
- `docs/testing/epic-9-responsive-test-log.md` - Responsive testing documentation
- `docs/testing/epic-9-accessibility-report.md` - Accessibility audit report
- `lighthouserc.json` - Lighthouse CI configuration
- `.github/workflows/accessibility.yml` - CI workflow for accessibility tests

**Modified Files:**
- All Epic 9 templates: Fix accessibility issues found during testing
- `tailwind.config.js`: May need to adjust colors for contrast compliance

### References

- [Source: docs/tech-spec-epic-9.md#Acceptance Criteria → Story 9.7 (AC-9.7.1 through AC-9.7.10)]
- [Source: docs/tech-spec-epic-9.md#Non-Functional Requirements → Observability → Accessibility Monitoring]
- [Source: docs/tech-spec-epic-9.md#Test Strategy Summary → Manual Testing → Accessibility Testing]
- [Source: docs/epics.md#Epic 9 → Story 9.7]
- [WCAG 2.1 Guidelines](https://www.w3.org/WAI/WCAG21/quickref/)
- [Tailwind CSS Responsive Design](https://tailwindcss.com/docs/responsive-design)

## Dev Agent Record

### Context Reference

- `/home/snapiz/projects/github/timayz/imkitchen/docs/story-context-9.7.xml`

### Agent Model Used

Claude Sonnet 4.5 (claude-sonnet-4-5-20250929)

### Debug Log References

### Completion Notes List

### File List

# Epic 9: Testing Documentation

This directory contains testing documentation and results for Epic 9: Enhanced Meal Planning Frontend UX Implementation.

## Overview

Story 9.7 implements comprehensive responsive design and accessibility testing infrastructure to ensure WCAG 2.1 Level AA compliance across all Epic 9 pages.

## Files

### Testing Documentation
- **epic-9-responsive-test-log.md** - Manual responsive testing checklist and results log
- **epic-9-accessibility-report.md** - Comprehensive accessibility audit report template

### Test Infrastructure
- **../../tests/accessibility/epic-9-accessibility.spec.ts** - Playwright accessibility tests using axe-core
- **../../playwright.config.ts** - Playwright configuration for e2e and accessibility testing
- **../../lighthouserc.json** - Lighthouse CI configuration with accessibility assertions
- **../../.github/workflows/accessibility.yml** - CI workflow for automated accessibility testing

## Running Tests

### Prerequisites

1. Install Node.js dependencies:
```bash
npm install
```

2. Install Playwright browsers:
```bash
npm run playwright:install
```

3. Build and run the application:
```bash
cargo build --release
cargo run --release
```

### Automated Tests

#### Run all Playwright tests:
```bash
npm test
```

#### Run only accessibility tests:
```bash
npm run test:accessibility
```

#### Run tests in headed mode (see browser):
```bash
npm run test:headed
```

#### Run tests in UI mode (interactive):
```bash
npm run test:ui
```

#### Debug tests:
```bash
npm run test:debug
```

#### Run Lighthouse CI:
```bash
npm run lighthouse
```

### Manual Testing

#### Responsive Design Testing

1. Open browser DevTools (F12)
2. Enable device toolbar (Ctrl+Shift+M / Cmd+Shift+M)
3. Test each page at breakpoints:
   - Mobile: 375px width
   - Tablet: 768px width
   - Desktop: 1920px width
4. Document results in `epic-9-responsive-test-log.md`

#### Touch Target Verification

1. Open browser DevTools → Elements → Accessibility pane
2. Use ruler tool to measure interactive elements
3. Verify all targets ≥44x44px
4. Document measurements in `epic-9-responsive-test-log.md`

#### Keyboard Navigation Testing

1. Use Tab key to navigate through page
2. Verify logical tab order
3. Test Enter/Space to activate elements
4. Test Escape to close modals
5. Verify focus indicators visible
6. Document results in `epic-9-accessibility-report.md`

#### Screen Reader Testing

**NVDA (Windows):**
1. Download from https://www.nvaccess.org/
2. Install and run NVDA
3. Navigate through pages with NVDA active
4. Verify announcements for all interactive elements
5. Document results in `epic-9-accessibility-report.md`

**VoiceOver (Mac/iOS):**
1. Enable VoiceOver (Cmd+F5 on Mac)
2. Navigate through pages with VoiceOver active
3. Verify announcements for all interactive elements
4. Document results in `epic-9-accessibility-report.md`

#### Color Contrast Testing

1. Install WAVE browser extension: https://wave.webaim.org/extension/
2. Run WAVE on each page
3. Review contrast errors and warnings
4. Fix any contrast ratios < 4.5:1 (normal text) or < 3:1 (large text)
5. Document results in `epic-9-accessibility-report.md`

#### Lighthouse Audit

1. Open Chrome DevTools
2. Navigate to Lighthouse tab
3. Select "Accessibility" category
4. Run audit
5. Verify score >90
6. Review and fix any issues
7. Document results in `epic-9-accessibility-report.md`

## CI Integration

### GitHub Actions Workflow

The `.github/workflows/accessibility.yml` workflow runs automatically on:
- Pull requests to `main` branch
- Pushes to `main` branch

### Workflow Jobs

1. **lighthouse-ci** - Runs Lighthouse CI on all Epic 9 pages
   - Asserts accessibility score >90
   - Uploads results as artifacts

2. **axe-core** - Runs Playwright accessibility tests
   - Uses axe-core for WCAG 2.1 AA compliance
   - Tests all Epic 9 pages across multiple viewports
   - Uploads HTML report as artifacts

### Viewing CI Results

1. Navigate to GitHub Actions tab
2. Select the workflow run
3. View job summaries
4. Download artifacts for detailed reports

## Acceptance Criteria

Story 9.7 Acceptance Criteria:
1. ✅ All pages tested on mobile (375px), tablet (768px), desktop (1920px) - manual testing log
2. ⏳ Calendar: Tabs on desktop, carousel on mobile (responsive breakpoint @md:)
3. ⏳ Forms: Full-width inputs on mobile (w-full), constrained on desktop (max-w-md)
4. ⏳ Modals: Centered on desktop with backdrop, full-height on mobile
5. ⏳ Touch targets ≥44x44px on mobile (buttons, checkboxes, links) - WCAG AA compliance
6. ✅ Keyboard navigation works for all interactive elements (Tab order logical, Enter/Space activate)
7. ✅ Screen reader testing: All content accessible with NVDA/VoiceOver (ARIA labels, semantic HTML)
8. ✅ Color contrast ratios meet WCAG AA: 4.5:1 for normal text, 3:1 for large text (verified with WAVE tool)
9. ✅ Focus indicators visible for all interactive elements (2px border, 4px offset, primary-500 color)
10. ✅ Lighthouse accessibility score >90 for all pages (automated test in CI)

✅ = Automated test infrastructure created
⏳ = Manual testing required

## WCAG 2.1 Level AA Requirements

### Contrast Ratios
- Normal text (<18pt): 4.5:1 minimum
- Large text (≥18pt or ≥14pt bold): 3:1 minimum
- UI components (buttons, form borders): 3:1 minimum

### Touch Targets
- Minimum size: 44x44px (WCAG 2.5.5, AAA level)
- Best practice: 48x48px for primary actions

### Keyboard Navigation
- All functionality available via keyboard
- Visible focus indicators (2px border, 4px offset)
- Logical tab order matching visual flow

### Screen Reader Support
- Semantic HTML (headings, lists, nav, main)
- ARIA labels for non-obvious elements (icons, complex widgets)
- Form labels associated with inputs (for/id or aria-labelledby)
- Error announcements (aria-live, aria-invalid)

### Text Alternatives
- Alt text for all informative images
- Empty alt for decorative images (alt="")
- ARIA labels for icon-only buttons

## Tools and Resources

### Testing Tools
- **Lighthouse** - https://developer.chrome.com/docs/lighthouse
- **WAVE** - https://wave.webaim.org/extension/
- **axe DevTools** - https://www.deque.com/axe/devtools/
- **NVDA** - https://www.nvaccess.org/
- **Playwright** - https://playwright.dev/

### Standards and Guidelines
- **WCAG 2.1** - https://www.w3.org/WAI/WCAG21/quickref/
- **WAI-ARIA** - https://www.w3.org/WAI/ARIA/apg/
- **HTML5 Accessibility** - https://www.w3.org/TR/html-aria/

### Related Documentation
- [Epic 9 Technical Specification](../tech-spec-epic-9.md)
- [UX Specification](../ux-specification.md)
- [Tailwind CSS Documentation](https://tailwindcss.com/docs)

## Sign-Off

### Testing Completion Checklist
- [ ] All automated tests passing in CI
- [ ] All manual responsive tests completed
- [ ] All manual screen reader tests completed
- [ ] All manual keyboard navigation tests completed
- [ ] All color contrast issues resolved
- [ ] All touch target sizes verified
- [ ] All cross-browser tests completed
- [ ] Lighthouse score >90 on all pages
- [ ] Comprehensive test report signed off

### Approvals
- **Accessibility Lead:** _____________________ Date: _____
- **Frontend Lead:** _____________________ Date: _____
- **Product Owner:** _____________________ Date: _____

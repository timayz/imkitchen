# Development Backlog

This file tracks action items, tech debt, and follow-up tasks identified during development and code review.

## Format
| Date | Story | Epic | Type | Severity | Owner | Status | Notes |
|------|-------|------|------|----------|-------|--------|-------|

## Action Items

| Date | Story | Epic | Type | Severity | Owner | Status | Notes |
|------|-------|------|------|----------|-------|--------|-------|
| 2025-10-19 | 5.3 | 5 | Bug | Medium | AI Agent | Done | ✅ Added transaction error handling with try-finally and abort/error handlers in `static/js/offline-db.js` (get/put/remove operations) |
| 2025-10-19 | 5.3 | 5 | Enhancement | Medium | AI Agent | Done | ✅ Implemented QuotaExceededError handling with dismissible yellow toast notification in `static/js/offline-db.js` put() function |
| 2025-10-19 | 5.3 | 5 | TechDebt | Low | AI Agent | Done | ✅ Standardized logging: console.debug (info), console.warn (degraded), console.error (critical) across offline modules |
| 2025-10-19 | 5.3 | 5 | Refactoring | Low | AI Agent | Done | ✅ Extracted CSS selectors as SELECTORS constant object in `static/js/shopping-checkoff.js` |
| 2025-10-19 | 5.3 | 5 | Enhancement | Low | AI Agent | Done | ✅ Created comprehensive TypeScript definitions in `types/global.d.ts` for window.offlineDB and window.shoppingCheckoff |
| 2025-10-19 | 5.3 | 5 | Testing | Low | TBD | Open | Add unit test for IndexedDB schema migration path (v1→v2) |

## Legend

**Type:**
- Bug: Code defect or error
- Enhancement: New feature or improvement
- TechDebt: Technical debt requiring refactoring
- Refactoring: Code restructuring without behavior change
- Testing: Missing or inadequate test coverage

**Severity:**
- High: Critical issue, blocks functionality or security risk
- Medium: Important issue, degrades experience or maintainability
- Low: Minor issue, nice-to-have improvement

**Status:**
- Open: Not yet started
- InProgress: Currently being worked on
- Done: Completed and verified
- Wontfix: Decided not to implement

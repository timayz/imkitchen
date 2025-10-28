# Engineering Backlog

This backlog collects cross-cutting or future action items that emerge from reviews and planning.

Routing guidance:

- Use this file for non-urgent optimizations, refactors, or follow-ups that span multiple stories/epics.
- Must-fix items to ship a story belong in that story's `Tasks / Subtasks`.
- Same-epic improvements may also be captured under the epic Tech Spec `Post-Review Follow-ups` section.

| Date | Story | Epic | Type | Severity | Owner | Status | Notes |
| ---- | ----- | ---- | ---- | -------- | ----- | ------ | ----- |
| 2025-10-27 | 10.1 | 10 | TechDebt | Med | Dev Team | Resolved | Extract test credentials to environment variables in `e2e/fixtures/auth.setup.ts:22-23` |
| 2025-10-27 | 10.1 | 10 | Bug | Med | Backend Dev | Resolved | Verify /health endpoint exists or implement in Axum router (CI health check dependency) - verified existing implementation |
| 2025-10-27 | 10.1 | 10 | TechDebt | Med | QA/Dev Team | Resolved | Move storageState config out of global use block in `e2e/playwright.config.ts:23` |
| 2025-10-27 | 10.1 | 10 | TechDebt | Med | Backend Dev, QA | Resolved | Document and implement test database seeding strategy for CI (`.github/workflows/test.yml`, `e2e/README.md`) |
| 2025-10-27 | 10.1 | 10 | Enhancement | Low | QA/Dev Team | Resolved | Add .auth directory creation in `e2e/fixtures/auth.setup.ts:8` |
| 2025-10-27 | 10.1 | 10 | TechDebt | Low | Dev Team | Resolved | Add .gitignore entries for E2E artifacts (`e2e/fixtures/.auth/*.json`, `e2e/test-results/`, `e2e/playwright-report/`) |
| 2025-10-27 | 10.1 | 10 | TechDebt | Low | DevOps | Resolved | Replace bc with shell arithmetic in CI coverage check (`.github/workflows/test.yml:171`) |
| 2025-10-27 | 10.1 | 10 | Enhancement | Low | QA/Dev Team | Resolved | Add individual test timeout (60s) to Playwright config `use` block |

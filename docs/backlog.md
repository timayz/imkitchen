# Engineering Backlog

This backlog collects cross-cutting or future action items that emerge from reviews and planning.

Routing guidance:

- Use this file for non-urgent optimizations, refactors, or follow-ups that span multiple stories/epics.
- Must-fix items to ship a story belong in that story's `Tasks / Subtasks`.
- Same-epic improvements may also be captured under the epic Tech Spec `Post-Review Follow-ups` section.

| Date | Story | Epic | Type | Severity | Owner | Status | Notes |
| ---- | ----- | ---- | ---- | -------- | ----- | ------ | ----- |
| 2025-11-01 | 1.2 | 1 | Testing | High | Dev | Open | Implement Task 8: Comprehensive Authentication Tests - Create tests/auth_test.rs with 8+ tests covering all ACs (AC #8, BLOCKER) |
| 2025-11-01 | 1.2 | 1 | Enhancement | Medium | Dev | Open | Add Password Complexity Validation - Update RegisterUserInput validator to require uppercase/lowercase/number per Architecture.md (crates/imkitchen-user/src/command.rs:16-21) |
| 2025-11-01 | 1.2 | 1 | Process | Low | SM | Open | Create Story Context for Future Stories - Generate context files for Epic 1 stories for better consistency |
| 2025-11-01 | 1.2 | 1 | Enhancement | Low | Dev | Open | Improve get_user_status Query Function - Consider adding aggregate status check for failed registrations (src/queries/user.rs:151-177) |
| 2025-11-01 | 1.3 | 1 | TechDebt | Low | Future maintainer | Open | Add inline comment to migration explaining JSON format for dietary_restrictions (migrations/queries/20251101230002_user_profiles.sql:4) |
| 2025-11-01 | 1.3 | 1 | Enhancement | Medium | Security enhancement epic | Open | Consider logging failed authentication attempts in AuthUser extractor for security monitoring (src/auth/jwt.rs:93) |
| 2025-11-01 | 1.3 | 1 | Enhancement | Low | UX polish epic | Open | Surface specific validation errors to users in profile route handler for better UX (src/routes/auth/profile.rs:140-156) |
| 2025-11-02 | 1.4 | 1 | Performance | Medium | Dev | Resolved | ✅ Fixed N+1 Query - LEFT JOIN user_profiles optimization reduces admin page from 22 to 2 queries (91% reduction) (src/queries/user.rs:397-400) |
| 2025-11-02 | 1.4 | 1 | Bug | Medium | Dev | Resolved | ✅ TwinSpark Attributes Correct - Investigation confirmed ts-req-selector is required for <tr> element extraction, correctly implemented |
| 2025-11-02 | 1.4 | 1 | Testing | Low | TBD | Open | Add E2E Middleware Test - Add HTTP-level test verifying /admin/users returns 403 for non-admin users (tests/admin_test.rs, AC #2) - Optional enhancement, command-level auth test exists |
| 2025-11-02 | 1.4 | 1 | Enhancement | Low | UX polish epic | Open | Format Timestamp Display - Add human-readable date format instead of raw Unix timestamp (templates/pages/admin/users.html:102) - Optional UX enhancement |

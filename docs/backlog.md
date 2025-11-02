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

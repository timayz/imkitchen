# Branch Protection Configuration

This document describes the branch protection rules required for the `main` branch as defined in Story 0.3 (AC #5).

## Required Configuration

### GitHub Repository Settings

Navigate to: **Settings > Branches > Branch protection rules** and configure the following for the `main` branch:

1. **Require a pull request before merging**
   - Enable: `Require a pull request before merging`
   - Enable: `Require approvals` (set to 1 approval if working in a team)

2. **Require status checks to pass before merging**
   - Enable: `Require status checks to pass before merging`
   - Enable: `Require branches to be up to date before merging`
   - Required status checks:
     - `lint` (Lint (Clippy))
     - `format` (Format (rustfmt))
     - `unit-tests` (Unit Tests)
     - `integration-tests` (Integration Tests)
     - `coverage` (Code Coverage)

3. **Additional protections**
   - Enable: `Do not allow bypassing the above settings`
   - Enable: `Restrict who can push to matching branches` (optional, for team environments)
   - **Disable**: `Allow force pushes` (must be unchecked)

## Verification

After configuration, verify:
1. Create a test PR with a failing test → CI should fail → Merge button should be disabled
2. Fix the test → CI should pass → Merge button should be enabled

## Rationale

These protections ensure:
- AC #1: CI pipeline runs on every PR (enforced by required status checks)
- AC #5: PR cannot merge if CI fails (enforced by required status checks)
- Code quality: All code must pass linting, formatting, and tests before merge
- Team collaboration: Peer review required before merge (if team environment)

## See Also

- [Story 0.3: CI/CD Pipeline Configuration](../stories/story-0.3.md)
- [GitHub Actions CI Workflow](../.github/workflows/ci.yml)

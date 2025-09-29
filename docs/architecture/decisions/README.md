# Architecture Decision Records (ADRs)

This directory contains Architecture Decision Records (ADRs) for IMKitchen, documenting the key architectural decisions made during the project's development.

## What are ADRs?

Architecture Decision Records are documents that capture important architectural decisions made along with their context and consequences. They help teams understand why certain technology choices were made and provide historical context for future decisions.

## ADR Format

Each ADR follows this structure:
- **Title**: Short descriptive title
- **Status**: Proposed, Accepted, Deprecated, Superseded
- **Context**: What is the issue that we're seeing
- **Decision**: What is the change that we're proposing/making
- **Consequences**: What becomes easier or more difficult to do

## Current ADRs

| ADR | Title | Status | Date |
|-----|-------|--------|------|
| [001](001-rust-askama-server-side-rendering.md) | Use Rust with Askama for Server-Side Rendering | Accepted | 2025-09-29 |
| [002](002-sqlite-event-sourcing-data-persistence.md) | SQLite with Event Sourcing for Data Persistence | Accepted | 2025-09-29 |
| [003](003-bounded-context-crate-organization.md) | Bounded Context Crate Organization | Accepted | 2025-09-29 |
| [004](004-twinspark-progressive-enhancement.md) | TwinSpark for Progressive JavaScript Enhancement | Accepted | 2025-09-29 |
| [005](005-session-based-authentication-management.md) | Session-Based Authentication and Authorization | Accepted | 2025-09-29 |

## ADR Lifecycle

1. **Proposed**: ADR is drafted and under discussion
2. **Accepted**: ADR is approved and being implemented
3. **Deprecated**: ADR is no longer relevant but kept for historical reference
4. **Superseded**: ADR has been replaced by a newer decision

## Creating New ADRs

When making significant architectural decisions:

1. Copy the [ADR template](template.md)
2. Number the ADR sequentially (006, 007, etc.)
3. Fill in all sections thoroughly
4. Discuss with team before marking as "Accepted"
5. Update this index with the new ADR

## ADR Guidelines

- **Be specific**: Include concrete technical details
- **Explain context**: Why was this decision needed?
- **Consider alternatives**: What other options were evaluated?
- **Document consequences**: Both positive and negative impacts
- **Keep it concise**: Focus on the essential information
- **Update when superseded**: Mark old ADRs as deprecated/superseded
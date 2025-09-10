# Introduction

This document outlines the complete fullstack architecture for **imkitchen**, including backend systems, frontend implementation, and their integration. It serves as the single source of truth for AI-driven development, ensuring consistency across the entire technology stack.

This unified approach combines what would traditionally be separate backend and frontend architecture documents, streamlining the development process for modern fullstack applications where these concerns are increasingly intertwined.

## Starter Template or Existing Project

Based on the PRD technical assumptions, **imkitchen** is a greenfield project with specific technology preferences:

- **Frontend Framework:** Lynx.js for cross-platform mobile development (as specified in PRD)
- **Backend Technology:** Go-based API services for performance-critical scheduling algorithms
- **Database:** PostgreSQL + Redis for relational data and caching
- **Mobile-First Approach:** Cross-platform targeting iOS/Android with responsive web interface

**No existing starter template** is prescribed, making this a custom architecture design. However, I recommend evaluating:

1. **Lynx.js Project Templates** - If available, for rapid cross-platform mobile setup
2. **Go Web API Starters** - Popular frameworks like Gin, Echo, or Fiber with PostgreSQL integration
3. **Monorepo Templates** - Nx or Turborepo configurations for managing mobile + API + shared packages

**Decision:** Custom architecture without relying on specific starters, allowing optimal technology integration for the unique "Fill My Week" automation requirements.

## Change Log
| Date | Version | Description | Author |
|------|---------|-------------|---------|
| 2025-09-06 | 1.0 | Initial fullstack architecture based on PRD and frontend specifications | Winston (Architect) |

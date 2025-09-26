# High Level Architecture

## Technical Summary

IMKitchen employs a **Rust-based modular monolithic architecture** with bounded context crates, deployed as a single CLI binary. The **Axum web server** serves **Askama-rendered HTML templates** with **TwinSpark declarative interactivity**, eliminating traditional API complexity. **DDD + CQRS + Event Sourcing** via **Evento 1.1+** provides robust state management across **SQLite event stores** per bounded context. The **Progressive Web App (PWA)** delivers kitchen-optimized mobile experiences with **Tailwind CSS styling** and **offline functionality**. This architecture achieves the PRD goals of intelligent meal planning automation while maintaining type safety, performance, and developer productivity through Rust's ecosystem.

## Platform and Infrastructure Choice

**Platform:** Containerized deployment with Docker + Kubernetes orchestration  
**Key Services:** Single Rust binary, SQLite databases per bounded context, SMTP service integration, static asset serving  
**Deployment Host and Regions:** Multi-cloud capable (AWS/GCP/Azure) with primary deployment in US-East for performance

## Repository Structure

**Structure:** Cargo workspace monorepo with bounded context crates  
**Monorepo Tool:** Cargo workspaces (native Rust tooling)  
**Package Organization:** Domain-driven crate separation with shared libraries and clear dependency boundaries

## High Level Architecture Diagram

```mermaid
graph TD
    A[Mobile/Web Users] --> B[CDN/Load Balancer]
    B --> C[Axum Web Server]
    C --> D[Askama Templates]
    C --> E[TwinSpark Handlers]
    
    C --> F[imkitchen-user Crate]
    C --> G[imkitchen-recipe Crate] 
    C --> H[imkitchen-meal-planning Crate]
    C --> I[imkitchen-shopping Crate]
    C --> J[imkitchen-notification Crate]
    
    F --> F1[SQLite Event Store - Users]
    G --> G1[SQLite Event Store - Recipes]
    H --> H1[SQLite Event Store - Meal Plans]
    I --> I1[SQLite Event Store - Shopping]
    J --> J1[SQLite Event Store - Notifications]
    
    J --> K[SMTP Service]
    C --> L[Static Assets/Tailwind CSS]
    
    M[Evento Event Bus] --> F
    M --> G
    M --> H
    M --> I
    M --> J
```

## Architectural Patterns

- **Domain-Driven Design (DDD):** Bounded contexts as separate crates with ubiquitous language - _Rationale:_ Clear business domain separation and independent evolution of meal planning, recipe management, and user management concerns
- **Event Sourcing with Evento:** Immutable event streams as single source of truth with aggregate replay - _Rationale:_ Complete audit trails for meal planning decisions, temporal queries for user behavior analysis, and reliable state reconstruction from events
- **Server-Side Rendering (SSR):** Askama templates with progressive enhancement - _Rationale:_ Optimal mobile performance, SEO benefits, and reduced client-side complexity for kitchen environments
- **Progressive Web App (PWA):** Installable with offline capabilities - _Rationale:_ Native app experience for kitchen usage with unreliable connectivity
- **Modular Monolith:** Single binary with crate boundaries - _Rationale:_ Type safety across boundaries, simplified deployment, while maintaining domain separation

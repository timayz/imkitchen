# Information Architecture (IA)

## Site Map / Screen Inventory

```mermaid
graph TD
    A[Landing/Login] --> B[Dashboard]
    B --> C[Recipe Library]
    B --> D[Meal Planner]
    B --> E[Shopping List]
    B --> F[Cook Mode]
    B --> G[Settings]
    
    C --> C1[Search/Filter]
    C --> C2[Recipe Detail]
    C --> C3[Add Recipe]
    C --> C4[Import Recipe]
    
    D --> D1[Weekly View]
    D --> D2[AI Suggestions]
    D --> D3[Manual Planning]
    D --> D4[Plan History]
    
    E --> E1[Current List]
    E --> E2[Shopping History]
    E --> E3[Store Layout]
    
    F --> F1[Active Recipe]
    F --> F2[Timer Central]
    F --> F3[Multi-dish View]
    F --> F4[Recipe Notes]
    
    G --> G1[Profile]
    G --> G2[Dietary Prefs]
    G --> G3[Notifications]
    G --> G4[Kitchen Setup]
    
    C2 --> F
    D1 --> E
    D1 --> F
    E1 --> F
```

## Navigation Structure

**Primary Navigation:** Bottom tab bar (mobile) / Left sidebar (desktop) with 5 core sections:
- Dashboard (home icon) - Central hub and today's focus
- Recipes (book icon) - Library and recipe management
- Planning (calendar icon) - Meal planning and scheduling
- Shopping (cart icon) - Shopping lists and grocery management  
- Cook (chef hat icon) - Active cooking mode and timers

**Secondary Navigation:** Context-aware top navigation and floating action buttons based on current task phase

**Breadcrumb Strategy:** Minimal breadcrumbs only in Cook Mode to show recipe → step progression; elsewhere rely on clear back/close actions

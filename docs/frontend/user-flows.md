# User Flows

## Flow 1: "Fill My Week" Automation

**User Goal:** Generate a complete weekly meal plan in under 30 seconds to eliminate decision fatigue

**Entry Points:** 
- Dashboard "Fill My Week" button
- Empty calendar state prompt
- Weekly planning reminder notification

**Success Criteria:** User has 7 days of meals assigned with visible prep timing indicators

### Flow Diagram

```mermaid
graph TD
    A[User taps Fill My Week] --> B{Recipe collection empty?}
    B -->|Yes| C[Onboard with curated recipes]
    B -->|No| D[Check current week assignments]
    
    C --> C1[Select cuisine preferences]
    C1 --> C2[Import starter recipes]
    C2 --> D
    
    D --> E{Any meals already assigned?}
    E -->|Yes| F[Preserve existing assignments]
    E -->|No| G[Start with empty calendar]
    
    F --> H[Fill only empty slots]
    G --> H
    H --> I[Apply rotation algorithm]
    I --> J[Consider complexity distribution]
    J --> K[Generate meal assignments]
    K --> L[Display populated calendar]
    L --> M{User satisfied?}
    M -->|Yes| N[Save meal plan]
    M -->|No| O[Offer regenerate options]
    O --> P[Regenerate with constraints]
    P --> L
    N --> Q[Show prep timeline preview]
    Q --> R[Enable notifications]
    R --> S[Success: Ready to cook!]
```

### Edge Cases & Error Handling:
- Recipe collection too small (< 7 recipes): Prompt to add more or accept repeats
- All recipes too complex for week: Suggest simpler alternatives or spread complexity
- User dietary restrictions conflict: Filter incompatible recipes automatically
- Technical failure during generation: Graceful degradation with manual assignment option
- Network offline: Use cached recipes and sync when reconnected

## Flow 2: Timing Intelligence Workflow

**User Goal:** Successfully coordinate complex recipe preparation through automated notifications and task management

**Entry Points:**
- Meal calendar showing upcoming complex recipes
- Notification prompt for advance preparation
- Recipe detail view timing timeline

**Success Criteria:** User completes all preparation steps on time and cooks meal successfully

### Flow Diagram

```mermaid
graph TD
    A[Complex meal scheduled] --> B[System calculates prep timeline]
    B --> C[Schedule advance notifications]
    C --> D[Send first prep reminder]
    D --> E[User receives notification]
    E --> F{User available?}
    F -->|Yes| G[Open prep task detail]
    F -->|No| H[Snooze with smart suggestions]
    
    G --> I[Review preparation steps]
    I --> J[Start preparation task]
    J --> K[Mark steps complete]
    K --> L{All steps done?}
    L -->|No| M[Continue with next step]
    L -->|Yes| N[Mark task complete]
    
    M --> K
    N --> O[Update timing intelligence]
    O --> P[Schedule next prep reminder]
    P --> Q{More prep needed?}
    Q -->|Yes| D
    Q -->|No| R[Ready for cooking day]
    
    H --> H1[Suggest alternative timing]
    H1 --> H2[Reschedule notifications]
    H2 --> D
    
    R --> S[Cooking day arrives]
    S --> T[Final cooking instructions]
    T --> U[Success tracking]
```

### Edge Cases & Error Handling:
- User misses critical prep window: Suggest recipe modifications or substitutions
- Preparation takes longer than estimated: Learn and adjust future timing
- User reports timing inaccuracy: Collect feedback and update algorithm
- Notification delivery failure: Use multiple delivery methods and in-app fallbacks
- Life disrupts schedule: Intelligent rescheduling with minimal user input

## Flow 3: Community Recipe Discovery

**User Goal:** Find new recipes with confidence in execution success based on community feedback

**Entry Points:**
- Discover tab exploration
- Search for specific cuisine or dish
- Trending recipe notifications
- Similar recipe suggestions

**Success Criteria:** User adds new recipe to collection and successfully cooks it

### Flow Diagram

```mermaid
graph TD
    A[Enter Discover section] --> B[Browse trending/featured]
    B --> C{Specific search intent?}
    C -->|Yes| D[Use search with filters]
    C -->|No| E[Explore curated categories]
    
    D --> F[Apply filters: cuisine, time, difficulty]
    E --> F
    F --> G[Browse recipe cards]
    G --> H[Tap recipe for preview]
    H --> I[View timing requirements]
    I --> J[Read execution reviews]
    J --> K{Recipe looks good?}
    K -->|No| L[Back to browse]
    K -->|Yes| M[Preview full recipe]
    
    L --> G
    M --> N[Check ingredient availability]
    N --> O[View timing timeline]
    O --> P{Add to collection?}
    P -->|Yes| Q[Import to personal recipes]
    P -->|No| R[Save for later consideration]
    
    Q --> S[Choose meal planning inclusion]
    S --> T[Recipe available for Fill My Week]
    T --> U[Success: Expanded repertoire]
    
    R --> V[Add to wishlist]
    V --> W[Return to discovery]
```

### Edge Cases & Error Handling:
- No recipes match search criteria: Suggest broader search or alternative cuisines
- Recipe has poor timing reviews: Display warnings and alternative suggestions
- Ingredient unavailability: Suggest substitutions or seasonal alternatives
- Network issues during import: Queue for later sync with clear status indication
- User reaches collection limits: Prompt for curation or premium upgrade

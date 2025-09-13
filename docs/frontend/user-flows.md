# User Flows

## Recipe Import & Management Flow

**User Goal:** Add new recipes to personal collection from various sources

**Entry Points:** Recipe Library "Add Recipe" button, URL sharing into app, dashboard quick actions

**Success Criteria:** Recipe successfully parsed, stored, and available in searchable library

### Flow Diagram

```mermaid
graph TD
    A[Start: Add Recipe] --> B{Import Method}
    B --> C[URL Import]
    B --> D[Manual Entry]
    B --> E[Photo Scan]
    
    C --> C1[Parse Recipe URL]
    C1 --> C2{Parsing Success?}
    C2 -->|Yes| F[Review & Edit]
    C2 -->|No| C3[Manual Fallback]
    C3 --> F
    
    D --> D1[Form Entry]
    D1 --> F
    
    E --> E1[OCR Processing]
    E1 --> E2{Text Detected?}
    E2 -->|Yes| F
    E2 -->|No| D1
    
    F --> G[Save Recipe]
    G --> H[Add to Library]
    H --> I[Success Confirmation]
```

### Edge Cases & Error Handling:
- URL parsing failures trigger manual entry mode with pre-filled detected text
- Duplicate recipe detection offers merge/replace options
- Network failures during import save partial data locally for retry
- Image upload failures provide camera retry and skip options

**Notes:** Import flow emphasizes quick success with graceful fallback options for parsing failures

## Meal Planning Flow

**User Goal:** Generate and customize weekly meal plan based on preferences and constraints

**Entry Points:** Dashboard meal planning widget, dedicated Planning tab, empty meal plan state

**Success Criteria:** Complete weekly meal plan with balanced nutrition, optimized ingredients, and timing feasibility

### Flow Diagram

```mermaid
graph TD
    A[Start: Plan Meals] --> B[Preferences Check]
    B --> B1{Preferences Set?}
    B1 -->|No| C[Quick Setup]
    B1 -->|Yes| D[Generate Plan]
    
    C --> C1[Dietary Restrictions]
    C1 --> C2[Household Size]  
    C2 --> C3[Cooking Time]
    C3 --> C4[Cuisine Prefs]
    C4 --> D
    
    D --> E[AI Plan Generation]
    E --> F[Present Plan]
    F --> G{User Satisfied?}
    
    G -->|No| H[Customize Plan]
    G -->|Yes| K[Confirm Plan]
    
    H --> H1{Change Type}
    H1 --> H2[Swap Recipe]
    H1 --> H3[Add Custom Meal]
    H1 --> H4[Remove Meal]
    H1 --> H5[Regenerate Day]
    
    H2 --> I[Recipe Suggestions]
    H3 --> J[Recipe Search]
    H4 --> I
    H5 --> I
    I --> F
    J --> F
    
    K --> L[Generate Shopping List]
    L --> M[Plan Saved]
```

### Edge Cases & Error Handling:
- No suitable recipes found for constraints offers relaxed criteria options
- Conflicting dietary preferences show trade-off explanations
- Plan generation failures provide manual planning tools
- Ingredient conflicts in customization trigger optimization suggestions

**Notes:** Flow balances AI automation with user control, always allowing manual override

## Cook Mode Flow

**User Goal:** Execute recipe(s) successfully with timing coordination and step-by-step guidance

**Entry Points:** Recipe detail "Start Cooking", meal plan "Cook Now", dashboard active cooking widget

**Success Criteria:** All dishes completed within timing targets with successful coordination

### Flow Diagram

```mermaid
graph TD
    A[Start: Cook Mode] --> B{Single or Multi-dish?}
    B --> C[Single Recipe]
    B --> D[Multi-dish Menu]
    
    C --> E[Recipe Prep View]
    D --> F[Timing Coordination]
    F --> G[Start Times Calculated]
    G --> H[Begin First Recipe]
    
    E --> I[Ingredients Check]
    H --> I
    I --> J{All Ingredients?}
    J -->|No| K[Substitution Suggestions]
    J -->|Yes| L[Start Prep Steps]
    K --> L
    
    L --> M[Step-by-step Guide]
    M --> N[Timer Management]
    N --> O{Step Complete?}
    O -->|No| P[Continue Step]
    O -->|Yes| Q{More Steps?}
    
    P --> N
    Q -->|Yes| R[Next Step]
    Q -->|No| S[Recipe Complete]
    
    R --> M
    S --> T{More Recipes?}
    T -->|Yes| U[Continue Multi-dish]
    T -->|No| V[All Complete]
    
    U --> M
    V --> W[Rate & Review]
    W --> X[Cooking Session End]
```

### Edge Cases & Error Handling:
- Timer failures provide manual time tracking and notifications
- Recipe modifications during cooking recalculate all dependent timings
- Emergency pause stops all timers and provides restart options
- Missing ingredients mid-cooking offer substitution or adaptation guidance

**Notes:** Cook Mode prioritizes clear progression and timing accuracy above all other features

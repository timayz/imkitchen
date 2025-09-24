# Information Architecture (IA)

## Site Map / Screen Inventory

```mermaid
graph TD
    A[Landing/Auth] --> B[Weekly Calendar Dashboard]
    A --> C[Onboarding Flow]
    
    B --> B1[Fill My Week Action]
    B --> B2[Daily Meal Detail]
    B --> B3[Meal Rescheduling]
    B --> B4[Weekly Shopping List]
    
    B --> D[Recipe Discovery]
    D --> D1[Browse Community]
    D --> D2[Search & Filter]
    D --> D3[Recipe Detail View]
    D --> D4[My Collections]
    
    B --> E[My Profile]
    E --> E1[Dietary Preferences]
    E --> E2[Family Settings] 
    E --> E3[Cooking Skill Level]
    E --> E4[Notification Preferences]
    
    B --> F[Community Hub]
    F --> F1[Recipe Sharing]
    F --> F2[Meal Plan Inspiration]
    F --> F3[Cooking Challenges]
    F --> F4[Success Stories]
    
    B2 --> G[Daily Prep Guide]
    G --> G1[Morning Reminders]
    G --> G2[Prep Checklist]
    G --> G3[Easy Mode Alternatives]
    
    B4 --> H[Shopping Features]
    H --> H1[Store Section Grouping]
    H --> H2[Family Sharing]
    H --> H3[Purchase History]
```

## Navigation Structure

**Primary Navigation:** Bottom tab bar (mobile) with 4 core sections:
- Home (Weekly Calendar) - primary dashboard
- Discover (Recipe browsing and community)  
- Lists (Shopping lists and meal prep)
- Profile (Settings and preferences)

**Secondary Navigation:** Context-aware action buttons and swipe gestures:
- Calendar: Swipe between weeks, tap dates for daily view
- Recipe Detail: Save to collections, rate, share actions
- Shopping: Check off items, share list, add custom items

**Breadcrumb Strategy:** Minimal breadcrumbs due to mobile-first design; rely on clear screen titles, back buttons, and contextual navigation cues

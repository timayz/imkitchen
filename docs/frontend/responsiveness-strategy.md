# Responsiveness Strategy

## Breakpoints

| Breakpoint | Min Width | Max Width | Target Devices |
|------------|-----------|-----------|----------------|
| Mobile | 320px | 767px | Smartphones, kitchen displays |
| Tablet | 768px | 1023px | Tablets, small laptops |
| Desktop | 1024px | 1439px | Laptops, desktop monitors |
| Wide | 1440px | - | Large monitors, kitchen displays |

## Adaptation Patterns

**Layout Changes:** Single column (mobile) → multi-column (tablet) → sidebar navigation (desktop) → dashboard layout (wide)

**Navigation Changes:** Bottom tabs (mobile) → side navigation (tablet+) → persistent sidebar with breadcrumbs (desktop+)

**Content Priority:** Timer/cooking content always prioritized, secondary features collapsed on mobile, progressive enhancement for larger screens

**Interaction Changes:** Touch-first design scales to mouse/keyboard, gestures supplement click interactions, voice integration available across all breakpoints

# Animation & Micro-interactions

## Motion Principles

**Purposeful Motion:** Every animation serves a functional purpose - providing feedback, guiding attention, or reducing cognitive load. No decorative animations that might distract during cooking.

**Kitchen-Appropriate Timing:** Animations are fast enough to feel responsive but slow enough to be perceived clearly in high-stress cooking moments. Default to slightly slower timing than typical mobile apps.

**Reduced Motion Respect:** Full support for prefers-reduced-motion with meaningful alternatives that maintain functionality without motion.

**Battery Awareness:** Lightweight animations that don't drain device batteries during extended cooking sessions.

## Key Animations

- **Fill My Week Generation:** Subtle card shuffle animation showing meal selection process (Duration: 800ms, Easing: ease-out)

- **Meal Card Assignment:** Smooth slide-in with gentle bounce when meals are added to calendar slots (Duration: 400ms, Easing: cubic-bezier(0.34, 1.56, 0.64, 1))

- **Prep Task Completion:** Satisfying checkmark animation with subtle scale and color transition (Duration: 300ms, Easing: ease-in-out)

- **Timing Countdown:** Smooth progress bar animation for active prep timers with pulsing for urgency (Duration: 1000ms, Easing: linear)

- **Calendar Navigation:** Horizontal slide transition between weeks with momentum-based easing (Duration: 350ms, Easing: ease-out)

- **Recipe Card Flip:** 3D flip animation for showing recipe details vs. timing information (Duration: 500ms, Easing: ease-in-out)

- **Drag and Drop Feedback:** Real-time visual feedback during meal rescheduling with drop zone highlighting (Duration: immediate, Easing: ease-out)

- **Error State Recovery:** Gentle shake animation for invalid actions with clear recovery guidance (Duration: 600ms, Easing: ease-in-out)

- **Loading States:** Skeleton screens with subtle shimmer effect during content loading (Duration: 1200ms, Easing: ease-in-out)

- **Success Confirmations:** Brief scale-up with color transition for successful actions like saving recipes (Duration: 200ms, Easing: ease-out)

- **Notification Entrance:** Slide-down from top with gentle bounce for timing alerts (Duration: 450ms, Easing: cubic-bezier(0.68, -0.55, 0.265, 1.55))

- **Page Transitions:** Fade with subtle scale for navigation between major sections (Duration: 250ms, Easing: ease-in-out)

# imkitchen AI Frontend Generation Prompt

## Master Prompt for AI Frontend Development Tools

The following prompt has been optimized for use with AI-powered frontend development tools such as Vercel v0, Lovable.ai, or similar code generation platforms. This prompt follows the structured four-part framework for maximum effectiveness.

---

## Copy-Pasteable AI Generation Prompt

```
# FOUNDATIONAL CONTEXT

You are building components for **imkitchen**, a comprehensive kitchen management platform that helps home cooks reduce food waste, streamline meal planning, and optimize cooking workflows. 

**Tech Stack:**
- Next.js 14+ with App Router and TypeScript
- Tailwind CSS for styling with 8px base grid system
- React hooks for state management
- Progressive Web App (PWA) capabilities
- Multi-language support (English, Spanish, French, German)
- Voice interaction capabilities

**Target Users:** 
- Primary: Organized home cooks (ages 28-45) who are tech-comfortable but need approachable complexity
- Secondary: Busy professionals (ages 25-40) who prioritize speed and convenience

**Core Design Principles:**
1. Kitchen-First Design - Every interaction considers wet hands, messy surfaces, and time pressure
2. Progressive Intelligence - System learns user behavior without overwhelming
3. Multi-Modal Accessibility - Voice, touch, and visual interaction patterns
4. 44px minimum touch targets for kitchen use
5. High contrast ratios (4.5:1 minimum) for various lighting conditions

**Visual Style:**
- Color Palette: Primary #FF6B35 (warm orange), Secondary #2ECC71 (fresh green), Warning #F1C40F (amber), Error #E74C3C (red)
- Typography: Inter for UI (highly legible), Merriweather for content warmth
- Spacing: 8px base unit system (4px, 8px, 16px, 24px, 32px, 48px)
- Aesthetic: Clean, food-focused, warm and approachable with clear functionality emphasis

# HIGH-LEVEL GOAL

Create a responsive [SPECIFIC COMPONENT NAME] component for the imkitchen platform that emphasizes usability in kitchen environments with mobile-first design and accessibility compliance.

# DETAILED STEP-BY-STEP INSTRUCTIONS

1. Create a new React component file named `[ComponentName].tsx` using TypeScript
2. Implement mobile-first responsive design with breakpoints at 768px (tablet) and 1024px (desktop)
3. Use Tailwind CSS classes exclusively for styling following the 8px spacing system
4. Ensure all interactive elements have minimum 44px touch targets
5. Add proper ARIA labels and semantic HTML for screen reader accessibility
6. Include keyboard navigation support with visible focus indicators
7. Implement loading states and error handling with user-friendly messages
8. Add voice interaction hints where applicable (microphone icons, voice status indicators)
9. Use the specified color palette for consistent brand experience
10. Include hover states for desktop and touch-friendly active states for mobile
11. Add proper TypeScript interfaces for all props and state
12. Implement proper error boundaries and fallback UI

# CODE EXAMPLES, DATA STRUCTURES & CONSTRAINTS

**Color Classes (use these exact Tailwind classes):**
```css
/* Primary Actions */
.bg-orange-500 { background: #FF6B35 }
.text-orange-500 { color: #FF6B35 }

/* Success States */
.bg-green-500 { background: #2ECC71 }
.text-green-500 { color: #2ECC71 }

/* Warning States */
.bg-yellow-500 { background: #F1C40F }
.text-yellow-500 { color: #F1C40F }

/* Error States */
.bg-red-500 { background: #E74C3C }
.text-red-500 { color: #E74C3C }
```

**Typography Classes:**
```css
/* Headlines */
.text-2xl.font-bold { /* H1 equivalent */ }
.text-xl.font-semibold { /* H2 equivalent */ }
.text-lg.font-semibold { /* H3 equivalent */ }

/* Body Text */
.text-base.font-normal { /* Standard body text */ }
.text-sm.font-normal { /* Small text */ }
```

**Spacing System (use these classes):**
- `p-1` (4px), `p-2` (8px), `p-4` (16px), `p-6` (24px), `p-8` (32px), `p-12` (48px)
- `m-1` (4px), `m-2` (8px), `m-4` (16px), `m-6` (24px), `m-8` (32px), `m-12` (48px)
- `gap-1` through `gap-12` for flexbox/grid spacing

**Required Props Interface Example:**
```typescript
interface ComponentProps {
  title: string;
  onAction?: () => void;
  isLoading?: boolean;
  variant?: 'primary' | 'secondary' | 'danger';
  className?: string;
}
```

**Voice Interaction Pattern:**
```jsx
// Include voice status indicator when applicable
{isVoiceActive && (
  <div className="flex items-center gap-2 text-blue-500">
    <Mic className="w-4 h-4 animate-pulse" />
    <span className="text-sm">Listening...</span>
  </div>
)}
```

**DO NOT:**
- Use any CSS-in-JS libraries or styled-components
- Implement complex state management (use props and simple useState)
- Add external dependencies beyond what's specified
- Use colors outside the defined palette
- Create touch targets smaller than 44px
- Use fixed positioning that might interfere with mobile keyboards

**DO:**
- Use semantic HTML elements (button, nav, main, section, etc.)
- Include proper alt text for any images
- Add loading and error states for all data-dependent components
- Use React.forwardRef if the component needs ref forwarding
- Include proper TypeScript types for all props and events

# STRICT SCOPE DEFINITION

**You should ONLY:**
- Create the single requested component file
- Use standard React hooks (useState, useEffect, useMemo, useCallback)
- Import icons from a standard icon library like Lucide React
- Apply Tailwind classes for styling

**You should NOT:**
- Modify any existing components or pages
- Create or modify API endpoints or server-side code
- Add new dependencies to package.json
- Modify global styles or configuration files
- Create multiple files unless specifically requested
- Implement authentication or complex business logic

**Testing Considerations:**
- Ensure component renders correctly on mobile (320px width minimum)
- Verify all interactive elements are accessible via keyboard
- Test with screen reader simulation (proper ARIA labels)
- Validate color contrast meets WCAG AA standards
- Check that component works with and without JavaScript enabled

**Output Requirements:**
- Provide complete, production-ready TypeScript React component
- Include comprehensive prop types and interfaces
- Add JSDoc comments for complex functions
- Ensure component is self-contained and reusable
- Include example usage in comments
```

---

## Prompt Structure Explanation

This prompt follows the four-part structured framework for optimal AI code generation:

### 1. High-Level Goal
Clearly establishes the objective of creating a kitchen-optimized React component for the imkitchen platform.

### 2. Detailed Instructions
Provides 12 specific, sequential steps that guide the AI through the complete component creation process, from file creation to accessibility compliance.

### 3. Code Examples & Constraints
Includes:
- Exact Tailwind CSS classes for brand colors and typography
- TypeScript interface patterns
- Voice interaction UI patterns
- Comprehensive DO/DON'T lists to prevent common issues

### 4. Strict Scope Definition
Explicitly defines what the AI should and should not modify, preventing unintended changes to the broader codebase while ensuring focus on the specific component request.

## Usage Instructions

1. **Replace Placeholders**: Update `[SPECIFIC COMPONENT NAME]` and `[ComponentName]` with your actual component requirements
2. **Customize Instructions**: Modify the step-by-step instructions based on your specific component needs
3. **Provide Context**: Add any additional context about the specific functionality or user interactions required
4. **Iterate**: Use the generated component as a starting point and refine through follow-up prompts

## Important Reminders

**⚠️ All AI-generated code requires careful human review, testing, and refinement to be considered production-ready.**

The generated components should be:
- Thoroughly tested across different screen sizes and devices
- Validated for accessibility compliance
- Reviewed for security considerations
- Integrated carefully into the existing codebase
- Performance-tested, especially for mobile devices in kitchen environments

This prompt framework ensures consistency with imkitchen's design system while providing the AI with sufficient context to generate high-quality, kitchen-optimized React components.

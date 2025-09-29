# ADR-004: TwinSpark for Progressive JavaScript Enhancement

## Status
Accepted

## Context
IMKitchen operates in kitchen environments where:

- **Devices vary widely**: From modern tablets to older resistant touchscreen devices
- **Network connectivity is inconsistent**: WiFi may be spotty in kitchen environments
- **JavaScript execution varies**: Performance ranges from fast to very slow
- **Reliability is critical**: Kitchen operations cannot wait for JavaScript to load
- **Progressive enhancement is essential**: Core functionality must work without JavaScript

The application needs interactive features like:
- Real-time cooking timers
- Dynamic ingredient calculations
- Live search and filtering
- Responsive form interactions
- Progressive Web App (PWA) capabilities

Traditional approaches:
- **No JavaScript**: Limited interactivity, poor user experience
- **Heavy JavaScript frameworks**: Poor performance on low-end devices, complexity
- **jQuery/Vanilla JS**: Manual DOM manipulation, maintenance overhead
- **HTMX**: Server-round trips for every interaction, network dependency

## Decision
We will use **TwinSpark** for progressive JavaScript enhancement, providing:

1. **HTML-driven interactions**: Define behavior in HTML attributes
2. **Progressive enhancement**: Full functionality without JavaScript, enhanced with JS
3. **Minimal payload**: ~10KB JavaScript library with no dependencies
4. **Server-side rendering**: Primary UI generated on server with Askama templates
5. **Selective enhancement**: JavaScript enhances specific interactions, not entire UI

## Alternatives Considered

### React/Vue.js Single Page Application
**Pros:**
- Rich ecosystem and community
- Component-based architecture
- Excellent development tools

**Cons:**
- Large bundle sizes (100KB+ minimum)
- Requires JavaScript for basic functionality
- Complex build processes and tooling
- Poor performance on low-end devices
- SEO and accessibility challenges
- High maintenance overhead

### HTMX with Server-Side Rendering
**Pros:**
- HTML-centric approach
- Good progressive enhancement
- Simple mental model

**Cons:**
- Every interaction requires server round-trip
- Network dependency for all dynamic behavior
- Limited offline capabilities
- Less control over client-side behavior
- Larger payload for complex interactions

### Vanilla JavaScript with Manual DOM Manipulation
**Pros:**
- No framework dependencies
- Full control over behavior
- Minimal payload

**Cons:**
- High development overhead
- Manual event handling and DOM updates
- No standard patterns for common interactions
- Maintenance complexity as features grow
- Browser compatibility challenges

### jQuery for DOM Manipulation
**Pros:**
- Familiar to many developers
- Simplified DOM manipulation
- Good browser compatibility

**Cons:**
- Large library size (~80KB)
- Outdated paradigms
- Encourages imperative programming
- No progressive enhancement patterns
- Performance overhead

### Alpine.js for Reactive Behavior
**Pros:**
- Small size (~25KB)
- HTML-driven reactive behavior
- Good progressive enhancement

**Cons:**
- Still requires JavaScript for basic functionality
- Limited ecosystem
- Less suitable for complex interactions
- Another framework dependency

## Consequences

### Positive
- **Minimal JavaScript Payload**: ~10KB library with no external dependencies
- **Progressive Enhancement**: Full functionality without JavaScript, enhanced experience with JS
- **HTML-Driven Development**: Behavior defined in HTML attributes, close to markup
- **Kitchen Environment Optimized**: Works reliably on slow devices and poor networks
- **Server-Side Rendering**: Fast initial page loads with Askama templates
- **Selective Enhancement**: Only enhance specific interactions that benefit from client-side behavior
- **Maintenance Simplicity**: Minimal JavaScript complexity, easier debugging and updates
- **SEO and Accessibility**: Search engines and screen readers work with server-rendered HTML

### Negative
- **Limited Ecosystem**: Smaller community compared to major frameworks
- **Learning Curve**: Team needs to learn TwinSpark patterns and limitations
- **Complex Interactions**: Some advanced UI patterns may require additional JavaScript
- **Framework Lock-in**: TwinSpark-specific patterns may not transfer to other projects

### Risks
- **Framework Maturity**: TwinSpark is less mature than established frameworks
  - *Mitigation*: Contribute to project, maintain fallback to vanilla JavaScript
- **Community Support**: Smaller community for troubleshooting and resources
  - *Mitigation*: Document patterns internally, contribute to community knowledge
- **Feature Limitations**: May need additional JavaScript for complex features
  - *Mitigation*: Use progressive enhancement, add vanilla JavaScript when needed

## Implementation Notes

### TwinSpark Integration
```html
<!-- Example: Enhanced form with client-side validation -->
<form 
    action="/recipes/create" 
    method="post"
    ts-req="/api/recipes/validate"
    ts-trigger="input"
    ts-target="#validation-feedback"
>
    <input 
        type="text" 
        name="recipe_name" 
        required 
        minlength="3"
        ts-validate="recipe-name"
    />
    <div id="validation-feedback"></div>
    <button type="submit">Create Recipe</button>
</form>
```

### Server-Side Template Foundation
```rust
// Askama template with TwinSpark enhancement points
#[derive(Template)]
#[template(path = "pages/recipe_form.html")]
pub struct RecipeFormTemplate {
    pub recipe: Option<Recipe>,
    pub ingredients: Vec<Ingredient>,
    pub validation_errors: ValidationErrors,
}
```

```html
<!-- recipe_form.html template -->
<div class="recipe-form-container">
    <!-- Core functionality works without JavaScript -->
    <form action="/recipes/create" method="post" class="recipe-form">
        <!-- Enhanced with TwinSpark for better UX -->
        <div 
            class="ingredient-list"
            ts-data="ingredients: {{ ingredients | json }}"
            ts-controller="ingredient-calculator"
        >
            {% for ingredient in ingredients %}
            <div class="ingredient-item">
                <input 
                    type="text" 
                    value="{{ ingredient.name }}"
                    ts-action="calculate-totals"
                />
                <input 
                    type="number" 
                    value="{{ ingredient.amount }}"
                    ts-action="calculate-totals"
                    ts-target="#recipe-totals"
                />
            </div>
            {% endfor %}
        </div>
        
        <!-- Real-time calculated totals (enhanced) -->
        <div id="recipe-totals" class="recipe-totals">
            <!-- Server-rendered initial values -->
            <span>Calories: {{ recipe.total_calories }}</span>
            <span>Prep Time: {{ recipe.prep_time_minutes }}min</span>
        </div>
        
        <button type="submit">Save Recipe</button>
    </form>
</div>
```

### Progressive Enhancement Patterns

#### Pattern 1: Real-time Validation
```html
<!-- Works without JavaScript: server-side validation on submit -->
<!-- Enhanced with JavaScript: real-time feedback -->
<input 
    type="email" 
    name="email" 
    required
    ts-validate="email"
    ts-target="#email-feedback"
    aria-describedby="email-feedback"
/>
<div id="email-feedback" role="alert">
    <!-- Server-rendered validation errors -->
    {% if validation_errors.email %}
        <span class="error">{{ validation_errors.email }}</span>
    {% endif %}
</div>
```

#### Pattern 2: Dynamic Content Loading
```html
<!-- Works without JavaScript: full page navigation -->
<!-- Enhanced with JavaScript: partial content updates -->
<nav class="recipe-filters">
    <a 
        href="/recipes?category=appetizers"
        ts-req="/api/recipes?category=appetizers"
        ts-target="#recipe-list"
        ts-push-url
    >
        Appetizers
    </a>
</nav>

<div id="recipe-list">
    <!-- Server-rendered recipe list -->
    {% for recipe in recipes %}
        {% include "partials/recipe_card.html" %}
    {% endfor %}
</div>
```

#### Pattern 3: Interactive Timers
```html
<!-- Works without JavaScript: static timer display -->
<!-- Enhanced with JavaScript: live countdown -->
<div 
    class="cooking-timer"
    ts-controller="timer"
    ts-data="duration: {{ timer.duration_seconds }}, started_at: {{ timer.started_at }}"
>
    <div class="timer-display">
        <span ts-target="minutes">{{ timer.remaining_minutes }}</span>:
        <span ts-target="seconds">{{ timer.remaining_seconds }}</span>
    </div>
    
    <button 
        ts-action="start-timer"
        {% if timer.is_running %}disabled{% endif %}
    >
        Start Timer
    </button>
    
    <button 
        ts-action="pause-timer"
        {% if not timer.is_running %}disabled{% endif %}
    >
        Pause Timer
    </button>
</div>
```

### JavaScript Controller Implementation
```javascript
// Custom controllers for complex behavior
TwinSpark.register('timer', {
    connect() {
        this.duration = parseInt(this.data.get('duration'));
        this.startedAt = new Date(this.data.get('started_at'));
        this.updateInterval = null;
        
        if (this.isRunning()) {
            this.startUpdating();
        }
    },
    
    startTimer() {
        this.startedAt = new Date();
        this.data.set('started_at', this.startedAt.toISOString());
        this.startUpdating();
        this.toggleButtons();
    },
    
    pauseTimer() {
        clearInterval(this.updateInterval);
        this.updateInterval = null;
        this.toggleButtons();
    },
    
    startUpdating() {
        this.updateInterval = setInterval(() => {
            this.updateDisplay();
        }, 1000);
    },
    
    updateDisplay() {
        const elapsed = Math.floor((new Date() - this.startedAt) / 1000);
        const remaining = Math.max(0, this.duration - elapsed);
        
        const minutes = Math.floor(remaining / 60);
        const seconds = remaining % 60;
        
        this.targets.minutes.textContent = minutes.toString().padStart(2, '0');
        this.targets.seconds.textContent = seconds.toString().padStart(2, '0');
        
        if (remaining === 0) {
            this.timerFinished();
        }
    },
    
    timerFinished() {
        clearInterval(this.updateInterval);
        // Trigger server notification or sound alert
        this.element.classList.add('timer-finished');
        // Optional: Send completion event to server
        fetch('/api/timers/completed', {
            method: 'POST',
            body: JSON.stringify({ timer_id: this.data.get('timer_id') })
        });
    }
});
```

### API Endpoints for Enhancement
```rust
// API endpoints that support both full page and partial updates
#[axum::debug_handler]
pub async fn get_recipes(
    Query(params): Query<RecipeQuery>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, AppError> {
    let recipes = recipe_service.search_recipes(params).await?;
    
    // Check if request is from TwinSpark (partial update)
    if headers.get("TS-Request").is_some() {
        // Return partial HTML for enhancement
        let template = RecipeListPartialTemplate { recipes };
        Ok(Html(template.render()?))
    } else {
        // Return full page for progressive enhancement
        let template = RecipeListPageTemplate { recipes, /* other page data */ };
        Ok(Html(template.render()?))
    }
}
```

### Performance Optimization
```html
<!-- Lazy loading for non-critical enhancements -->
<div 
    class="advanced-recipe-editor"
    ts-lazy
    ts-src="/static/js/recipe-editor.js"
>
    <!-- Fallback content without JavaScript -->
    <textarea name="recipe_instructions" rows="10">
        {{ recipe.instructions }}
    </textarea>
</div>

<!-- Critical enhancements loaded immediately -->
<script src="/static/js/twinspark.min.js" defer></script>
<script src="/static/js/core-enhancements.js" defer></script>
```

### Testing Strategy
```javascript
// Testing enhanced behavior
describe('Recipe Timer Enhancement', () => {
    beforeEach(() => {
        // Set up DOM with server-rendered content
        document.body.innerHTML = `
            <div ts-controller="timer" ts-data="duration: 300">
                <span ts-target="minutes">05</span>:
                <span ts-target="seconds">00</span>
                <button ts-action="start-timer">Start</button>
            </div>
        `;
        
        TwinSpark.start();
    });
    
    test('timer displays correctly without JavaScript', () => {
        // Test server-rendered content
        expect(document.querySelector('[ts-target="minutes"]').textContent).toBe('05');
        expect(document.querySelector('[ts-target="seconds"]').textContent).toBe('00');
    });
    
    test('timer starts when enhanced with JavaScript', () => {
        const startButton = document.querySelector('[ts-action="start-timer"]');
        startButton.click();
        
        // Test that JavaScript enhancement works
        expect(startButton.disabled).toBe(true);
    });
});
```

## References
- [TwinSpark Documentation](https://github.com/kasta-ua/twinspark)
- [Progressive Enhancement Principles](https://developer.mozilla.org/en-US/docs/Glossary/Progressive_Enhancement)
- [HTML-over-the-Wire](https://hotwired.dev/)
- [Kitchen Environment Web Development Best Practices](../requirements/kitchen-environments.md)
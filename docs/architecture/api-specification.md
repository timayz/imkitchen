# API Specification

**Note:** IMKitchen uses **TwinSpark server-side rendering** instead of traditional REST APIs. All interactions occur through HTML form submissions and server-rendered template fragments.

## TwinSpark Endpoint Patterns

```rust
// TwinSpark Server Response Pattern
use axum::response::Html;

// TwinSpark works by replacing HTML content based on ts-target selectors
// Server returns complete HTML fragments that replace target elements

pub struct TwinSparkHandler;

impl TwinSparkHandler {
    pub fn render_fragment<T: askama::Template>(template: T) -> Html<String> {
        Html(template.render().unwrap())
    }
}

// Example: Meal Plan Generation
// Client HTML with TwinSpark attributes:
// <button ts-req="/meal-plans/generate" ts-target="#weekly-calendar" ts-swap="outerHTML">
//   Fill My Week
// </button>
// 
// Server endpoint returns HTML fragment that replaces #weekly-calendar element
```

## Key TwinSpark Endpoint Patterns

```html
<!-- Askama Template: weekly_calendar.html -->
<!DOCTYPE html>
<html>
<head>
    <title>IMKitchen - Weekly Meal Plan</title>
    <script src="/static/js/twinspark.js"></script>
    <link href="/static/css/tailwind.css" rel="stylesheet">
</head>
<body>
    <div id="weekly-calendar" class="meal-calendar">
        <!-- Meal Plan Generation Form -->
        <form ts-req="/meal-plans/generate" 
              ts-target="#weekly-calendar">
            <input type="hidden" name="week_start" value="{{week_start}}">
            <button type="submit" 
                    class="bg-amber-500 hover:bg-amber-600 text-white font-bold py-4 px-8 rounded-lg">
                Fill My Week
            </button>
        </form>

        <!-- Weekly Calendar Grid -->
        <div class="grid grid-cols-7 gap-4 mt-6">
            {% for meal_slot in meal_slots %}
            <div class="meal-slot bg-white rounded-lg shadow-sm p-4"
                 ts-req="/meals/{{meal_slot.recipe_id}}/details"
                 ts-target="#meal-details"
                 ts-trigger="click">
                {% if meal_slot.recipe_id %}
                    <h3>{{meal_slot.recipe_name}}</h3>
                    <p class="text-sm text-gray-600">{{meal_slot.prep_time}} min</p>
                {% else %}
                    <div class="text-gray-400 text-center">
                        <span class="text-2xl">+</span>
                        <p>Add Meal</p>
                    </div>
                {% endif %}
            </div>
            {% endfor %}
        </div>

        <!-- Live Recipe Search -->
        <div class="mt-8">
            <input type="text" 
                   ts-req="/recipes/search" 
                   ts-target="#search-results"
                   ts-trigger="keyup changed delay:500ms"
                   placeholder="Search recipes..."
                   class="w-full border-2 border-gray-300 rounded-lg px-4 py-2">
            
            <div id="search-results" class="mt-4">
                <!-- Search results populated via TwinSpark -->
            </div>
        </div>
    </div>

    <!-- Meal Details Modal -->
    <div id="meal-details" class="hidden">
        <!-- Populated via TwinSpark when meal slot clicked -->
    </div>
</body>
</html>
```

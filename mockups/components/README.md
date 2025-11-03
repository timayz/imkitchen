# UI Components

This folder contains reusable UI components for the imkitchen application.

## Components

### Modals (`modal.html`)

Interactive modal dialogs with various use cases:

- **Basic Modal** - Simple modal with header, body, and footer
- **Confirmation Modal** - Confirmation dialog with warning icon
- **Form Modal** - Modal containing a form with scrollable content
- **Loading Modal** - Modal with spinner for async operations
- **Success Modal** - Success confirmation with checkmark icon
- **Large Modal** - Full-screen on mobile, large dialog on desktop

#### Features:
- Responsive design (full-screen on mobile, centered on desktop)
- Close button in header
- Stacked action buttons on mobile
- Scrollable content for long forms
- Dark overlay background

#### Usage:
```html
<!-- Modal Trigger -->
<button onclick="document.getElementById('modal-id').classList.remove('hidden')">
    Show Modal
</button>

<!-- Modal Structure -->
<div id="modal-id" class="hidden fixed inset-0 bg-black bg-opacity-50 z-50 flex items-center justify-center p-4">
    <div class="bg-white rounded-lg shadow-xl max-w-md w-full">
        <!-- Header -->
        <div class="flex items-center justify-between p-6 border-b">
            <h3 class="text-xl font-bold">Modal Title</h3>
            <button onclick="document.getElementById('modal-id').classList.add('hidden')">
                <!-- Close icon -->
            </button>
        </div>
        <!-- Body -->
        <div class="p-6">
            <!-- Content here -->
        </div>
        <!-- Footer -->
        <div class="flex gap-3 p-6 border-t bg-gray-50">
            <button class="flex-1 px-4 py-2 bg-gray-200 text-gray-700 rounded-lg">
                Cancel
            </button>
            <button class="flex-1 px-4 py-2 bg-green-600 text-white rounded-lg">
                Confirm
            </button>
        </div>
    </div>
</div>
```

### Alerts (`alerts.html`)

Alert components for different message types and states:

#### Alert Types:

1. **Success Alerts**
   - Basic success message
   - Success with loading state (processing)
   - Success with action button

2. **Error Alerts**
   - Basic error message
   - Error with loading state (retrying)
   - Error with validation list

3. **Warning Alerts**
   - Basic warning message
   - Warning with loading state
   - Warning with action button (upgrade)

4. **Info Alerts**
   - Basic info message
   - Info with loading state (syncing)
   - Info with progress bar

5. **Compact Alerts (Toast Style)**
   - Success toast
   - Error toast
   - Loading toast

6. **Banner Alerts (Full Width)**
   - Info banner
   - Success banner

#### Features:
- Color-coded by type (green=success, red=error, yellow=warning, blue=info)
- Icons for visual clarity
- Loading spinners for async operations
- Progress bars for upload/download states
- Dismissible with close button
- Responsive layouts (stacked on mobile)
- Action buttons where appropriate
- Auto-dismiss functionality (toast example)

#### Usage:

**Basic Alert:**
```html
<div class="bg-green-50 border border-green-200 rounded-lg p-4">
    <div class="flex items-start gap-3">
        <!-- Icon -->
        <svg class="w-5 h-5 text-green-600">...</svg>
        <!-- Content -->
        <div class="flex-1">
            <div class="font-semibold text-green-800">Success!</div>
            <div class="text-sm text-green-700 mt-1">Message here</div>
        </div>
        <!-- Close button -->
        <button class="text-green-600">...</button>
    </div>
</div>
```

**Alert with Loading:**
```html
<div class="bg-blue-50 border border-blue-200 rounded-lg p-4">
    <div class="flex items-start gap-3">
        <!-- Spinner -->
        <div class="animate-spin rounded-full h-5 w-5 border-2 border-blue-200 border-t-blue-600"></div>
        <!-- Content -->
        <div class="flex-1">
            <div class="font-semibold text-blue-800">Processing...</div>
            <div class="text-sm text-blue-700 mt-1">Please wait...</div>
        </div>
    </div>
</div>
```

**Toast (Fixed Position):**
```html
<!-- Container -->
<div class="fixed bottom-4 right-4 z-50 space-y-2">
    <!-- Toast -->
    <div class="bg-white border-l-4 border-green-500 rounded-lg shadow-lg p-4 max-w-sm">
        <div class="flex items-center gap-3">
            <svg class="w-5 h-5 text-green-600">...</svg>
            <div class="flex-1 text-sm font-semibold">Message</div>
            <button>...</button>
        </div>
    </div>
</div>
```

## Color Schemes

### Success
- Background: `bg-green-50`
- Border: `border-green-200`
- Text: `text-green-800` (heading), `text-green-700` (body)
- Icon: `text-green-600`
- Button: `bg-green-600 hover:bg-green-700`

### Error
- Background: `bg-red-50`
- Border: `border-red-200`
- Text: `text-red-800` (heading), `text-red-700` (body)
- Icon: `text-red-600`
- Button: `bg-red-600 hover:bg-red-700`

### Warning
- Background: `bg-yellow-50`
- Border: `border-yellow-200`
- Text: `text-yellow-800` (heading), `text-yellow-700` (body)
- Icon: `text-yellow-600`
- Button: `bg-yellow-600 hover:bg-yellow-700`

### Info
- Background: `bg-blue-50`
- Border: `border-blue-200`
- Text: `text-blue-800` (heading), `text-blue-700` (body)
- Icon: `text-blue-600`
- Button: `bg-blue-600 hover:bg-blue-700`

## Responsive Design

All components are mobile-first and responsive:

- **Mobile (default)**: Stacked layouts, full-width elements
- **Tablet (md:)**: Horizontal layouts where appropriate
- **Desktop (lg:)**: Optimized spacing and sizing

## Accessibility

- Semantic HTML structure
- Proper color contrast ratios
- Focus states on interactive elements
- Close buttons with clear hit areas
- Screen reader friendly (use aria-labels as needed)

## Examples

View the HTML files directly in a browser to see interactive examples of all components:

- `modal.html` - All modal variations
- `alerts.html` - All alert variations

## Integration

To use these components in your pages:

1. Copy the HTML structure
2. Adjust IDs and onclick handlers as needed
3. Modify content and styling to match your use case
4. Add any necessary JavaScript for dynamic behavior

## Dependencies

- Tailwind CSS 4.1+ (via CDN)
- No JavaScript framework required
- Pure HTML/CSS with vanilla JavaScript for interactions

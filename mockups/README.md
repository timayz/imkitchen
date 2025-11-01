# imkitchen HTML Mockups

Complete set of HTML mockups for the imkitchen meal planning application.

## Overview

This folder contains **17 fully interactive HTML pages** built with Tailwind CSS 4.1+ (via CDN). All pages are static HTML with realistic dummy data, ready to open directly in your browser for validation and testing.

## Quick Start

1. Open any HTML file in your browser (no server required)
2. Navigate between pages using the links
3. All styling is done via Tailwind CDN (no build step needed)

## Page Inventory

### Public Pages (3 pages)

**1. Landing Page** - `index.html`
- Hero section with value proposition
- Feature showcase (meal planning, accompaniments, shopping)
- How it works (3-step process)
- Pricing comparison (Free vs Premium)
- Testimonials and social proof
- Footer with links

**2. Login Page** - `login.html`
- Email/password form
- "Remember me" option
- Forgot password link
- Demo account quick access links

**3. Register Page** - `register.html`
- Full sign-up form with account info
- Dietary restrictions preferences
- Household size and cuisine variety settings
- Terms acceptance checkbox

### Authenticated User Pages (9 pages)

**4. Dashboard (Free Tier)** - `dashboard-free.html`
- Shows nearest day's meals (only if in accessible Week 1)
- Upgrade banner for premium
- Quick actions (calendar, shopping list, regenerate)
- Recipe favorites status (8/10 limit)
- Premium upsell cards

**5. Dashboard (Premium)** - `dashboard-premium.html`
- Shows nearest day's meals (from any week)
- Full access indicators
- This week at a glance preview
- No upgrade banners

**6. Meal Calendar (Free)** - `calendar-free.html`
- Week 1 fully visible with all meals
- Weeks 2-5 locked with upgrade prompts
- 7-day grid with appetizer, main, dessert per day
- Regenerate all weeks button
- Premium upsell section

**7. Meal Calendar (Premium)** - `calendar-premium.html`
- All 5 weeks accessible
- Week navigation tabs
- Full 7-day grid for each week
- Week preview sections
- No restrictions

**8. Recipe Create/Edit Form** - `recipe-create.html`
- Recipe type selector (all 4 types with icons)
- Basic info (name, description, prep/cook time)
- Dynamic ingredient list
- Dynamic instruction steps
- Dietary restrictions checkboxes
- Cuisine type dropdown with custom option
- Main course specific: accepts accompaniments toggle
- Advance prep instructions
- Community sharing toggle

**9. My Recipes List** - `recipes-list.html`
- Stats cards (total, favorited, shared, community)
- Favorites limit warning (Free tier: 8/10)
- Filters (search, type, cuisine, sort)
- Recipe grid with all 4 types displayed
- Recipe cards showing:
  - Type badge (color-coded)
  - Dietary tags
  - Cook time
  - Community indicator
  - Edit/View buttons

**10. Recipe Detail View** - `recipe-detail.html`
- Full recipe display with hero image area
- Ingredients with checkboxes
- Numbered instruction steps
- Rating summary (stars, distribution)
- Write review form
- Review list with user avatars
- Suggested accompaniments sidebar
- Quick info sidebar
- Recipe stats (views, favorites, reviews)

**11. Community Recipes** - `community.html`
- Community stats banner
- Trending recipes section (4 featured)
- Advanced filters (search, type, cuisine, dietary)
- Recipe grid with ratings and creator info
- Favorite button on each card
- Pagination

**12. Recipe Import** - `import.html`
- Drag-and-drop upload area
- JSON schema documentation
- Required/optional fields reference
- Import progress UI (hidden by default)
- Real-time stats (imported, failed, remaining)
- Import summary with error details
- Duplicate detection warnings
- Download example files

### Utility Pages (3 pages)

**13. Shopping List** - `shopping-list.html`
- Week selector (Week 1 visible, others locked for Free)
- Organized by category (proteins, vegetables, dairy, bakery, pantry)
- Checkboxes for each item
- Quantity and recipe count per item
- Progress tracker
- Email/print/reset actions
- Recipes included list
- Premium upsell

**14. User Profile/Settings** - `profile.html`
- Account info form (name, email, username)
- Meal preferences (dietary restrictions, household size, cuisine variety slider)
- Subscription management with upgrade CTA
- Notification toggles (4 types with switches)
- Change password form
- Danger zone (delete account)

**15. Contact Us** - `contact.html`
- Contact form (name, email, subject dropdown, message)
- Subject categories (general, support, billing, feature, bug, etc.)
- Contact information sidebar
- Quick help links
- Social media links
- FAQ section (4 common questions)

### Admin Pages (2 pages)

**16. Admin - User Management** - `admin-users.html`
- Admin-only dark navigation
- User stats dashboard (total, premium, active, suspended)
- Filters (search, account type, status, sort)
- User table with:
  - User info (avatar, name, email, username)
  - Status badges
  - Account type badges
  - Recipe count
  - Action buttons (edit, view, suspend, reactivate, delete)
- Pagination

**17. Admin - Contact Inbox** - `admin-contact.html`
- Admin-only navigation
- Message stats (total, unread, today, avg response time)
- Filters (search, status, subject, sort)
- Message list with:
  - Unread/read/resolved status
  - Subject category badges
  - Message preview
  - Quick actions (mark read, resolve, view details)
- Checkbox selection
- Pagination

## Navigation Flow

### User Journey - Free Tier
```
index.html (Landing)
  → register.html (Sign up)
    → dashboard-free.html (Home after login)
      → calendar-free.html (View Week 1 only)
        → shopping-list.html (Week 1 shopping)
      → recipes-list.html (My recipes, 8/10 favorites)
        → recipe-create.html (Add new recipe)
        → recipe-detail.html (View recipe)
      → community.html (Browse community recipes)
      → import.html (Bulk import)
      → profile.html (Settings & upgrade to premium)
```

### User Journey - Premium Tier
```
index.html (Landing)
  → login.html
    → dashboard-premium.html (Full access home)
      → calendar-premium.html (View all 5 weeks)
        → shopping-list.html (All weeks accessible)
      → recipes-list.html (Unlimited favorites)
      → profile.html (Manage subscription)
```

### Admin Journey
```
login.html (Admin login)
  → admin-users.html (User management)
  → admin-contact.html (Message inbox)
```

## Key Features Demonstrated

### Freemium Model
- **Free Tier:**
  - Week 1 visibility only
  - 10 favorite recipes maximum
  - Upgrade prompts throughout
  - All features accessible but limited

- **Premium Tier:**
  - Full month visibility (all 5 weeks)
  - Unlimited favorite recipes
  - No upgrade prompts
  - Priority support badge

### Recipe System
- 4 Recipe Types: Appetizer, Main Course, Dessert, Accompaniment
- Color-coded type badges (blue, orange, pink, purple)
- Dietary restriction tags
- Community sharing toggle
- Rating and review system

### Meal Planning
- Week-based generation (5 weeks)
- Current week locking indicator
- 3 courses per day (appetizer, main, dessert)
- Accompaniment pairing (e.g., rice with curry)
- Advance prep reminders
- Empty slots when insufficient recipes

### Admin Features
- User management (view, edit, suspend, delete)
- Account type indicators (Free, Premium, Admin)
- Contact message management (read, resolve, reply)
- Stats dashboards
- Bulk actions

## Visual Design

### Color Scheme
- **Primary:** Green (#059669 - green-600)
- **Accent:** Purple/Pink gradient (Premium features)
- **Status Colors:**
  - Blue: Appetizer
  - Orange: Main Course
  - Pink: Dessert
  - Purple: Accompaniment
  - Yellow: Warnings/Locked
  - Red: Errors/Danger

### Typography
- Headings: Bold, large sizes (text-3xl, text-4xl)
- Body: Regular, readable (text-sm, text-base)
- Badges: Uppercase, small (text-xs uppercase)

### Components
- Cards: White background, rounded-lg, shadow-lg
- Buttons: Rounded-lg with hover states
- Forms: Border with focus ring (ring-2 ring-green-500)
- Tables: Alternating hover states
- Badges: Rounded-full for status, rounded for tags

## Technology Stack

- **HTML5:** Semantic markup
- **Tailwind CSS 4.1+:** Via CDN (no config needed)
- **No JavaScript framework:** Pure HTML with inline onclick for demos
- **No build process:** Open directly in browser

## Browser Compatibility

All pages use modern CSS via Tailwind CDN:
- Chrome 90+
- Firefox 88+
- Safari 14+
- Edge 90+

## Responsive Design

All pages are mobile-responsive with:
- Grid layouts (md:grid-cols-2, lg:grid-cols-3)
- Flexible containers
- Breakpoints: sm, md, lg
- Touch-friendly buttons and inputs

## Notes for Development

### What's NOT Implemented (Static Mockups)
- No real form submission (uses `method="get"` for demo)
- No actual authentication
- No database connections
- No real-time updates
- No actual file uploads
- JavaScript is minimal (alert() for button demos)

### What IS Demonstrated
- Complete UI for all user flows
- Freemium restrictions and upgrade paths
- All form fields with proper types and validation attributes
- Realistic dummy data throughout
- Responsive layouts
- Accessible markup (semantic HTML, labels, ARIA where needed)

## Next Steps for Development

1. **Backend Integration:**
   - Replace static HTML with Askama templates
   - Add Axum route handlers
   - Implement Evento commands for state changes
   - Add query handlers for projections

2. **Database:**
   - Create migration files for all tables
   - Separate read/write databases (CQRS)
   - Implement event sourcing with Evento

3. **Authentication:**
   - Implement JWT cookie-based auth
   - Add session management
   - Role-based access control (user, admin)

4. **File Uploads:**
   - Implement JSON import validation
   - Add file size checks
   - Batch processing with async handlers

5. **Real-time Features:**
   - Twinspark for reactive UI updates
   - Polling for async operations (meal generation)
   - WebSocket for admin notifications

## File Structure
```
mockups/
├── index.html                  # Landing page
├── login.html                  # Login
├── register.html               # Sign up
├── dashboard-free.html         # Free tier dashboard
├── dashboard-premium.html      # Premium dashboard
├── calendar-free.html          # Free tier calendar
├── calendar-premium.html       # Premium calendar
├── recipe-create.html          # Recipe form
├── recipes-list.html           # Recipe library
├── recipe-detail.html          # Single recipe view
├── community.html              # Community recipes
├── import.html                 # Bulk import
├── shopping-list.html          # Weekly shopping lists
├── profile.html                # User settings
├── contact.html                # Contact form
├── admin-users.html            # Admin user management
├── admin-contact.html          # Admin message inbox
└── README.md                   # This file
```

## Questions or Feedback?

Review each page in your browser to validate:
- Visual design matches your brief
- All features are represented
- User flows make sense
- Freemium restrictions are clear
- Admin capabilities are sufficient

Open `index.html` to start your review!

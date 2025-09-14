# Database Schema

```sql
-- Users and Households
CREATE TABLE households (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    settings JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE NOT NULL,
    name VARCHAR(255) NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    dietary_preferences TEXT[],
    allergies TEXT[],
    household_id UUID REFERENCES households(id),
    language VARCHAR(5) DEFAULT 'en',
    timezone VARCHAR(50) DEFAULT 'UTC',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Inventory Management
CREATE TYPE storage_location AS ENUM ('pantry', 'refrigerator', 'freezer');
CREATE TYPE inventory_category AS ENUM ('proteins', 'vegetables', 'fruits', 'grains', 'dairy', 'spices', 'condiments', 'beverages', 'baking', 'frozen');

CREATE TABLE inventory_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    quantity DECIMAL(10,2) NOT NULL,
    unit VARCHAR(50) NOT NULL,
    category inventory_category NOT NULL,
    location storage_location NOT NULL,
    expiration_date DATE,
    purchase_date DATE DEFAULT CURRENT_DATE,
    estimated_cost DECIMAL(10,2),
    household_id UUID REFERENCES households(id),
    added_by UUID REFERENCES users(id),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Recipes
CREATE TYPE difficulty_level AS ENUM ('easy', 'medium', 'hard');
CREATE TYPE recipe_source AS ENUM ('user-created', 'imported', 'api-external');

CREATE TABLE recipes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    external_id VARCHAR(255), -- For API-sourced recipes
    title VARCHAR(500) NOT NULL,
    description TEXT,
    ingredients JSONB NOT NULL, -- Array of RecipeIngredient objects
    instructions JSONB NOT NULL, -- Array of RecipeStep objects
    cooking_time INTEGER, -- minutes
    prep_time INTEGER, -- minutes
    difficulty difficulty_level,
    servings INTEGER,
    cuisine VARCHAR(100),
    tags TEXT[],
    image_url VARCHAR(500),
    nutrition_info JSONB,
    source recipe_source DEFAULT 'user-created',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE TABLE user_recipe_ratings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id),
    recipe_id UUID REFERENCES recipes(id),
    rating INTEGER CHECK (rating >= 1 AND rating <= 5),
    review TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(user_id, recipe_id)
);

-- Meal Planning
CREATE TYPE meal_type AS ENUM ('breakfast', 'lunch', 'dinner', 'snack');
CREATE TYPE meal_status AS ENUM ('planned', 'in-progress', 'completed', 'skipped');

CREATE TABLE meal_plans (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    start_date DATE NOT NULL,
    end_date DATE NOT NULL,
    household_id UUID REFERENCES households(id),
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE TABLE meal_plan_entries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    meal_plan_id UUID REFERENCES meal_plans(id),
    date DATE NOT NULL,
    meal_type meal_type NOT NULL,
    recipe_id UUID REFERENCES recipes(id),
    servings INTEGER DEFAULT 1,
    notes TEXT,
    assigned_cook UUID REFERENCES users(id),
    status meal_status DEFAULT 'planned',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Shopping Lists
CREATE TYPE shopping_list_status AS ENUM ('active', 'completed', 'archived');
CREATE TYPE store_category AS ENUM ('produce', 'dairy', 'meat', 'frozen', 'pantry', 'bakery', 'other');

CREATE TABLE shopping_lists (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    status shopping_list_status DEFAULT 'active',
    generated_from UUID[], -- Array of meal_plan IDs
    estimated_total DECIMAL(10,2),
    household_id UUID REFERENCES households(id),
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE TABLE shopping_list_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    shopping_list_id UUID REFERENCES shopping_lists(id),
    name VARCHAR(255) NOT NULL,
    quantity DECIMAL(10,2) NOT NULL,
    unit VARCHAR(50) NOT NULL,
    category store_category DEFAULT 'other',
    purchased BOOLEAN DEFAULT FALSE,
    estimated_price DECIMAL(10,2),
    actual_price DECIMAL(10,2),
    notes TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Indexes for performance
CREATE INDEX idx_inventory_household_location ON inventory_items(household_id, location);
CREATE INDEX idx_inventory_expiration ON inventory_items(expiration_date) WHERE expiration_date IS NOT NULL;
CREATE INDEX idx_recipes_source ON recipes(source);
CREATE INDEX idx_recipes_external_id ON recipes(external_id) WHERE external_id IS NOT NULL;
CREATE INDEX idx_meal_plan_entries_date ON meal_plan_entries(date);
CREATE INDEX idx_shopping_lists_household_status ON shopping_lists(household_id, status);

-- Full-text search for recipes
CREATE INDEX idx_recipes_search ON recipes USING gin(to_tsvector('english', title || ' ' || description));
```

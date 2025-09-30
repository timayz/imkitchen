-- Recipe Database Schema Migration
-- Story 2.1: Recipe Database and CRUD Operations

-- Main recipes table
CREATE TABLE IF NOT EXISTS recipes (
    recipe_id TEXT PRIMARY KEY,
    title TEXT NOT NULL CHECK (length(title) >= 1 AND length(title) <= 200),
    prep_time_minutes INTEGER NOT NULL CHECK (prep_time_minutes > 0),
    cook_time_minutes INTEGER NOT NULL CHECK (cook_time_minutes > 0),
    difficulty TEXT NOT NULL CHECK (difficulty IN ('Easy', 'Medium', 'Hard')),
    category TEXT NOT NULL CHECK (category IN ('Appetizer', 'Main', 'Dessert', 'Beverage', 'Bread', 'Soup', 'Salad')),
    rating REAL DEFAULT 0.0 CHECK (rating >= 0.0 AND rating <= 5.0),
    review_count INTEGER DEFAULT 0 CHECK (review_count >= 0),
    created_by TEXT NOT NULL,
    is_public BOOLEAN NOT NULL DEFAULT FALSE,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME,
    
    -- Foreign key to user_profiles table
    FOREIGN KEY (created_by) REFERENCES user_profiles(id) ON DELETE CASCADE
);

-- Recipe ingredients table
CREATE TABLE IF NOT EXISTS recipe_ingredients (
    ingredient_id INTEGER PRIMARY KEY AUTOINCREMENT,
    recipe_id TEXT NOT NULL,
    ingredient_name TEXT NOT NULL CHECK (length(ingredient_name) >= 1),
    quantity DECIMAL(10, 3) NOT NULL CHECK (quantity > 0),
    unit TEXT NOT NULL CHECK (length(unit) >= 1),
    notes TEXT,
    sort_order INTEGER NOT NULL DEFAULT 0,
    
    FOREIGN KEY (recipe_id) REFERENCES recipes(recipe_id) ON DELETE CASCADE
);

-- Recipe instructions table
CREATE TABLE IF NOT EXISTS recipe_instructions (
    instruction_id INTEGER PRIMARY KEY AUTOINCREMENT,
    recipe_id TEXT NOT NULL,
    step_number INTEGER NOT NULL CHECK (step_number > 0),
    instruction_text TEXT NOT NULL CHECK (length(trim(instruction_text)) > 0),
    estimated_minutes INTEGER CHECK (estimated_minutes > 0),
    
    FOREIGN KEY (recipe_id) REFERENCES recipes(recipe_id) ON DELETE CASCADE,
    UNIQUE (recipe_id, step_number)
);

-- Recipe tags table (normalized many-to-many relationship)
CREATE TABLE IF NOT EXISTS recipe_tags (
    recipe_id TEXT NOT NULL,
    tag TEXT NOT NULL CHECK (length(trim(tag)) > 0),
    
    PRIMARY KEY (recipe_id, tag),
    FOREIGN KEY (recipe_id) REFERENCES recipes(recipe_id) ON DELETE CASCADE
);

-- Full-text search virtual table for recipes
CREATE VIRTUAL TABLE IF NOT EXISTS recipe_search USING fts5(
    recipe_id UNINDEXED,
    title,
    ingredients_text,
    instructions_text,
    tags_text,
    category UNINDEXED,
    difficulty UNINDEXED,
    prep_time_minutes UNINDEXED,
    cook_time_minutes UNINDEXED,
    is_public UNINDEXED,
    created_by UNINDEXED,
    rating UNINDEXED,
    review_count UNINDEXED,
    created_at UNINDEXED,
    content='recipes',
    content_rowid='rowid'
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_recipes_created_by ON recipes(created_by);
CREATE INDEX IF NOT EXISTS idx_recipes_category ON recipes(category);
CREATE INDEX IF NOT EXISTS idx_recipes_difficulty ON recipes(difficulty);
CREATE INDEX IF NOT EXISTS idx_recipes_is_public ON recipes(is_public);
CREATE INDEX IF NOT EXISTS idx_recipes_rating ON recipes(rating DESC);
CREATE INDEX IF NOT EXISTS idx_recipes_created_at ON recipes(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_recipes_prep_time ON recipes(prep_time_minutes);
CREATE INDEX IF NOT EXISTS idx_recipes_cook_time ON recipes(cook_time_minutes);

CREATE INDEX IF NOT EXISTS idx_recipe_ingredients_recipe_id ON recipe_ingredients(recipe_id);
CREATE INDEX IF NOT EXISTS idx_recipe_ingredients_sort_order ON recipe_ingredients(recipe_id, sort_order);

CREATE INDEX IF NOT EXISTS idx_recipe_instructions_recipe_id ON recipe_instructions(recipe_id);
CREATE INDEX IF NOT EXISTS idx_recipe_instructions_step ON recipe_instructions(recipe_id, step_number);

CREATE INDEX IF NOT EXISTS idx_recipe_tags_tag ON recipe_tags(tag);

-- Triggers to maintain FTS search index
CREATE TRIGGER IF NOT EXISTS recipe_search_insert AFTER INSERT ON recipes BEGIN
    INSERT INTO recipe_search(
        recipe_id, title, ingredients_text, instructions_text, tags_text,
        category, difficulty, prep_time_minutes, cook_time_minutes,
        is_public, created_by, rating, review_count, created_at
    ) VALUES (
        NEW.recipe_id, NEW.title, '', '', '',
        NEW.category, NEW.difficulty, NEW.prep_time_minutes, NEW.cook_time_minutes,
        NEW.is_public, NEW.created_by, NEW.rating, NEW.review_count, NEW.created_at
    );
END;

CREATE TRIGGER IF NOT EXISTS recipe_search_update AFTER UPDATE ON recipes BEGIN
    UPDATE recipe_search SET
        title = NEW.title,
        category = NEW.category,
        difficulty = NEW.difficulty,
        prep_time_minutes = NEW.prep_time_minutes,
        cook_time_minutes = NEW.cook_time_minutes,
        is_public = NEW.is_public,
        rating = NEW.rating,
        review_count = NEW.review_count
    WHERE recipe_id = NEW.recipe_id;
END;

CREATE TRIGGER IF NOT EXISTS recipe_search_delete AFTER DELETE ON recipes BEGIN
    DELETE FROM recipe_search WHERE recipe_id = OLD.recipe_id;
END;

-- Function to rebuild search text when ingredients change
CREATE TRIGGER IF NOT EXISTS recipe_search_ingredients_update AFTER INSERT ON recipe_ingredients BEGIN
    UPDATE recipe_search SET ingredients_text = (
        SELECT GROUP_CONCAT(ingredient_name || ' ' || quantity || ' ' || unit, ' ')
        FROM recipe_ingredients 
        WHERE recipe_id = NEW.recipe_id
        ORDER BY sort_order
    ) WHERE recipe_id = NEW.recipe_id;
END;

CREATE TRIGGER IF NOT EXISTS recipe_search_ingredients_update_on_change AFTER UPDATE ON recipe_ingredients BEGIN
    UPDATE recipe_search SET ingredients_text = (
        SELECT GROUP_CONCAT(ingredient_name || ' ' || quantity || ' ' || unit, ' ')
        FROM recipe_ingredients 
        WHERE recipe_id = NEW.recipe_id
        ORDER BY sort_order
    ) WHERE recipe_id = NEW.recipe_id;
END;

CREATE TRIGGER IF NOT EXISTS recipe_search_ingredients_delete AFTER DELETE ON recipe_ingredients BEGIN
    UPDATE recipe_search SET ingredients_text = (
        SELECT COALESCE(GROUP_CONCAT(ingredient_name || ' ' || quantity || ' ' || unit, ' '), '')
        FROM recipe_ingredients 
        WHERE recipe_id = OLD.recipe_id
        ORDER BY sort_order
    ) WHERE recipe_id = OLD.recipe_id;
END;

-- Function to rebuild search text when instructions change
CREATE TRIGGER IF NOT EXISTS recipe_search_instructions_update AFTER INSERT ON recipe_instructions BEGIN
    UPDATE recipe_search SET instructions_text = (
        SELECT GROUP_CONCAT(instruction_text, ' ')
        FROM recipe_instructions 
        WHERE recipe_id = NEW.recipe_id
        ORDER BY step_number
    ) WHERE recipe_id = NEW.recipe_id;
END;

CREATE TRIGGER IF NOT EXISTS recipe_search_instructions_update_on_change AFTER UPDATE ON recipe_instructions BEGIN
    UPDATE recipe_search SET instructions_text = (
        SELECT GROUP_CONCAT(instruction_text, ' ')
        FROM recipe_instructions 
        WHERE recipe_id = NEW.recipe_id
        ORDER BY step_number
    ) WHERE recipe_id = NEW.recipe_id;
END;

CREATE TRIGGER IF NOT EXISTS recipe_search_instructions_delete AFTER DELETE ON recipe_instructions BEGIN
    UPDATE recipe_search SET instructions_text = (
        SELECT COALESCE(GROUP_CONCAT(instruction_text, ' '), '')
        FROM recipe_instructions 
        WHERE recipe_id = OLD.recipe_id
        ORDER BY step_number
    ) WHERE recipe_id = OLD.recipe_id;
END;

-- Function to rebuild search text when tags change
CREATE TRIGGER IF NOT EXISTS recipe_search_tags_update AFTER INSERT ON recipe_tags BEGIN
    UPDATE recipe_search SET tags_text = (
        SELECT GROUP_CONCAT(tag, ' ')
        FROM recipe_tags 
        WHERE recipe_id = NEW.recipe_id
    ) WHERE recipe_id = NEW.recipe_id;
END;

CREATE TRIGGER IF NOT EXISTS recipe_search_tags_delete AFTER DELETE ON recipe_tags BEGIN
    UPDATE recipe_search SET tags_text = (
        SELECT COALESCE(GROUP_CONCAT(tag, ' '), '')
        FROM recipe_tags 
        WHERE recipe_id = OLD.recipe_id
    ) WHERE recipe_id = OLD.recipe_id;
END;
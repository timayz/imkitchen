-- Collections Database Schema Migration
-- Story 2.2: Personal Recipe Collections and Favorites

-- Main collections table
CREATE TABLE IF NOT EXISTS collections (
    collection_id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    name TEXT NOT NULL CHECK (length(name) >= 1 AND length(name) <= 100),
    description TEXT CHECK (description IS NULL OR length(description) <= 500),
    privacy TEXT NOT NULL CHECK (privacy IN ('Private', 'Shared', 'Public')) DEFAULT 'Private',
    is_archived BOOLEAN NOT NULL DEFAULT FALSE,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Foreign key to users table
    FOREIGN KEY (user_id) REFERENCES users(user_id) ON DELETE CASCADE
);

-- Collection recipes many-to-many relationship table
CREATE TABLE IF NOT EXISTS collection_recipes (
    collection_id TEXT NOT NULL,
    recipe_id TEXT NOT NULL,
    added_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    sort_order INTEGER NOT NULL DEFAULT 0,
    
    PRIMARY KEY (collection_id, recipe_id),
    FOREIGN KEY (collection_id) REFERENCES collections(collection_id) ON DELETE CASCADE,
    FOREIGN KEY (recipe_id) REFERENCES recipes(recipe_id) ON DELETE CASCADE
);

-- User favorites table (special system collection for quick access)
CREATE TABLE IF NOT EXISTS user_favorites (
    user_id TEXT NOT NULL,
    recipe_id TEXT NOT NULL,
    favorited_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    PRIMARY KEY (user_id, recipe_id),
    FOREIGN KEY (user_id) REFERENCES users(user_id) ON DELETE CASCADE,
    FOREIGN KEY (recipe_id) REFERENCES recipes(recipe_id) ON DELETE CASCADE
);

-- Collection sharing table (for shared collections)
CREATE TABLE IF NOT EXISTS collection_shares (
    collection_id TEXT NOT NULL,
    shared_with_user_id TEXT NOT NULL,
    shared_by_user_id TEXT NOT NULL,
    shared_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    can_edit BOOLEAN NOT NULL DEFAULT FALSE,
    
    PRIMARY KEY (collection_id, shared_with_user_id),
    FOREIGN KEY (collection_id) REFERENCES collections(collection_id) ON DELETE CASCADE,
    FOREIGN KEY (shared_with_user_id) REFERENCES users(user_id) ON DELETE CASCADE,
    FOREIGN KEY (shared_by_user_id) REFERENCES users(user_id) ON DELETE CASCADE
);

-- Full-text search virtual table for collections
CREATE VIRTUAL TABLE IF NOT EXISTS collection_search USING fts5(
    collection_id UNINDEXED,
    name,
    description,
    recipe_titles,
    recipe_tags,
    user_id UNINDEXED,
    privacy UNINDEXED,
    recipe_count UNINDEXED,
    created_at UNINDEXED,
    is_archived UNINDEXED,
    content='collections',
    content_rowid='rowid'
);

-- Performance indexes for collections
CREATE INDEX IF NOT EXISTS idx_collections_user_id ON collections(user_id);
CREATE INDEX IF NOT EXISTS idx_collections_privacy ON collections(privacy);
CREATE INDEX IF NOT EXISTS idx_collections_created_at ON collections(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_collections_updated_at ON collections(updated_at DESC);
CREATE INDEX IF NOT EXISTS idx_collections_name ON collections(name);
CREATE INDEX IF NOT EXISTS idx_collections_archived ON collections(is_archived);

-- Performance indexes for collection_recipes
CREATE INDEX IF NOT EXISTS idx_collection_recipes_collection_id ON collection_recipes(collection_id);
CREATE INDEX IF NOT EXISTS idx_collection_recipes_recipe_id ON collection_recipes(recipe_id);
CREATE INDEX IF NOT EXISTS idx_collection_recipes_added_at ON collection_recipes(added_at DESC);
CREATE INDEX IF NOT EXISTS idx_collection_recipes_sort_order ON collection_recipes(collection_id, sort_order);

-- Performance indexes for user_favorites
CREATE INDEX IF NOT EXISTS idx_user_favorites_user_id ON user_favorites(user_id);
CREATE INDEX IF NOT EXISTS idx_user_favorites_recipe_id ON user_favorites(recipe_id);
CREATE INDEX IF NOT EXISTS idx_user_favorites_favorited_at ON user_favorites(favorited_at DESC);

-- Performance indexes for collection_shares
CREATE INDEX IF NOT EXISTS idx_collection_shares_shared_with ON collection_shares(shared_with_user_id);
CREATE INDEX IF NOT EXISTS idx_collection_shares_shared_by ON collection_shares(shared_by_user_id);
CREATE INDEX IF NOT EXISTS idx_collection_shares_shared_at ON collection_shares(shared_at DESC);

-- Triggers to maintain FTS search index for collections
CREATE TRIGGER IF NOT EXISTS collection_search_insert AFTER INSERT ON collections BEGIN
    INSERT INTO collection_search(
        collection_id, name, description, recipe_titles, recipe_tags,
        user_id, privacy, recipe_count, created_at, is_archived
    ) VALUES (
        NEW.collection_id, NEW.name, COALESCE(NEW.description, ''), '', '',
        NEW.user_id, NEW.privacy, 0, NEW.created_at, NEW.is_archived
    );
END;

CREATE TRIGGER IF NOT EXISTS collection_search_update AFTER UPDATE ON collections BEGIN
    UPDATE collection_search SET
        name = NEW.name,
        description = COALESCE(NEW.description, ''),
        user_id = NEW.user_id,
        privacy = NEW.privacy,
        is_archived = NEW.is_archived
    WHERE collection_id = NEW.collection_id;
END;

CREATE TRIGGER IF NOT EXISTS collection_search_delete AFTER DELETE ON collections BEGIN
    DELETE FROM collection_search WHERE collection_id = OLD.collection_id;
END;

-- Trigger to update collection updated_at timestamp
CREATE TRIGGER IF NOT EXISTS collections_updated_at AFTER UPDATE ON collections FOR EACH ROW BEGIN
    UPDATE collections SET updated_at = CURRENT_TIMESTAMP WHERE collection_id = NEW.collection_id;
END;

-- Trigger to update collection updated_at when recipes are added/removed
CREATE TRIGGER IF NOT EXISTS collection_recipes_update_collection_timestamp 
AFTER INSERT ON collection_recipes FOR EACH ROW BEGIN
    UPDATE collections SET updated_at = CURRENT_TIMESTAMP WHERE collection_id = NEW.collection_id;
END;

CREATE TRIGGER IF NOT EXISTS collection_recipes_delete_update_collection_timestamp 
AFTER DELETE ON collection_recipes FOR EACH ROW BEGIN
    UPDATE collections SET updated_at = CURRENT_TIMESTAMP WHERE collection_id = OLD.collection_id;
END;

-- Function to rebuild search text when collection recipes change
CREATE TRIGGER IF NOT EXISTS collection_search_recipes_update AFTER INSERT ON collection_recipes BEGIN
    UPDATE collection_search SET 
        recipe_count = (
            SELECT COUNT(*) 
            FROM collection_recipes 
            WHERE collection_id = NEW.collection_id
        ),
        recipe_titles = (
            SELECT GROUP_CONCAT(r.title, ' ')
            FROM collection_recipes cr
            JOIN recipes r ON cr.recipe_id = r.recipe_id
            WHERE cr.collection_id = NEW.collection_id
            ORDER BY cr.sort_order, cr.added_at
        ),
        recipe_tags = (
            SELECT GROUP_CONCAT(DISTINCT rt.tag, ' ')
            FROM collection_recipes cr
            JOIN recipe_tags rt ON cr.recipe_id = rt.recipe_id
            WHERE cr.collection_id = NEW.collection_id
        )
    WHERE collection_id = NEW.collection_id;
END;

CREATE TRIGGER IF NOT EXISTS collection_search_recipes_delete AFTER DELETE ON collection_recipes BEGIN
    UPDATE collection_search SET 
        recipe_count = (
            SELECT COUNT(*) 
            FROM collection_recipes 
            WHERE collection_id = OLD.collection_id
        ),
        recipe_titles = (
            SELECT COALESCE(GROUP_CONCAT(r.title, ' '), '')
            FROM collection_recipes cr
            JOIN recipes r ON cr.recipe_id = r.recipe_id
            WHERE cr.collection_id = OLD.collection_id
            ORDER BY cr.sort_order, cr.added_at
        ),
        recipe_tags = (
            SELECT COALESCE(GROUP_CONCAT(DISTINCT rt.tag, ' '), '')
            FROM collection_recipes cr
            JOIN recipe_tags rt ON cr.recipe_id = rt.recipe_id
            WHERE cr.collection_id = OLD.collection_id
        )
    WHERE collection_id = OLD.collection_id;
END;

-- Constraint to enforce user collection limit (50 collections per user)
-- Note: This is implemented as a check in application logic rather than database constraint
-- due to SQLite limitations with complex constraints

-- Constraint to enforce collection recipe limit (1000 recipes per collection)
-- Note: This is implemented as a check in application logic rather than database constraint
-- due to SQLite limitations with complex constraints

-- View for collection analytics (optional, for performance)
CREATE VIEW IF NOT EXISTS collection_analytics AS
SELECT 
    c.collection_id,
    c.user_id,
    c.name,
    c.privacy,
    c.is_archived,
    c.created_at,
    c.updated_at,
    COUNT(cr.recipe_id) as recipe_count,
    COALESCE(AVG(CASE r.difficulty 
        WHEN 'Easy' THEN 1 
        WHEN 'Medium' THEN 2 
        WHEN 'Hard' THEN 3 
        END), 0) as avg_difficulty_score,
    COALESCE(AVG(r.prep_time_minutes + r.cook_time_minutes), 0) as avg_total_time,
    COALESCE(AVG(r.rating), 0) as avg_rating,
    COUNT(DISTINCT r.category) as category_count
FROM collections c
LEFT JOIN collection_recipes cr ON c.collection_id = cr.collection_id
LEFT JOIN recipes r ON cr.recipe_id = r.recipe_id
WHERE c.is_archived = FALSE
GROUP BY c.collection_id, c.user_id, c.name, c.privacy, c.created_at, c.updated_at;

-- View for user favorites with recipe details
CREATE VIEW IF NOT EXISTS user_favorites_detail AS
SELECT 
    uf.user_id,
    uf.recipe_id,
    uf.favorited_at,
    r.title,
    r.difficulty,
    r.category,
    r.prep_time_minutes + r.cook_time_minutes as total_time_minutes,
    r.rating,
    r.review_count,
    r.is_public,
    r.created_by
FROM user_favorites uf
JOIN recipes r ON uf.recipe_id = r.recipe_id
ORDER BY uf.favorited_at DESC;
-- Migration 005: Create recipes table with full-text search and indexes
-- Create recipes table with all required fields
CREATE TABLE IF NOT EXISTS recipes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    external_id VARCHAR(255), -- from URL import sources
    external_source VARCHAR(50), -- 'spoonacular', 'edamam', 'user_generated'
    title VARCHAR(255) NOT NULL,
    description TEXT,
    image_url TEXT,
    source_url TEXT,
    prep_time INTEGER NOT NULL, -- minutes
    cook_time INTEGER NOT NULL, -- minutes
    total_time INTEGER GENERATED ALWAYS AS (prep_time + cook_time) STORED,
    meal_type VARCHAR(20)[] CHECK (meal_type <@ ARRAY['breakfast', 'lunch', 'dinner', 'snack']),
    complexity VARCHAR(20) CHECK (complexity IN ('simple', 'moderate', 'complex')),
    cuisine_type VARCHAR(50),
    servings INTEGER DEFAULT 4,
    ingredients JSONB NOT NULL,
    instructions JSONB NOT NULL,
    dietary_labels TEXT[] DEFAULT '{}',
    is_public BOOLEAN DEFAULT false,
    average_rating DECIMAL(3,2) DEFAULT 0.0,
    total_ratings INTEGER DEFAULT 0,
    difficulty_score INTEGER,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    deleted_at TIMESTAMP WITH TIME ZONE
);

-- Create indexes for search and filtering performance
-- GIN index for meal type array filtering
CREATE INDEX IF NOT EXISTS idx_recipes_meal_type ON recipes USING GIN (meal_type);

-- B-tree index for complexity filtering
CREATE INDEX IF NOT EXISTS idx_recipes_complexity ON recipes (complexity);

-- B-tree index for prep time filtering
CREATE INDEX IF NOT EXISTS idx_recipes_total_time ON recipes (total_time);

-- GIN index for full-text search on title and description
CREATE INDEX IF NOT EXISTS idx_recipes_fulltext ON recipes USING GIN (to_tsvector('english', COALESCE(title, '') || ' ' || COALESCE(description, '')));

-- Index for user_id (for user's recipe queries)
CREATE INDEX IF NOT EXISTS idx_recipes_user_id ON recipes (user_id);

-- Index for created_at (for sorting by newest)
CREATE INDEX IF NOT EXISTS idx_recipes_created_at ON recipes (created_at DESC);

-- Index for average_rating (for sorting by rating)
CREATE INDEX IF NOT EXISTS idx_recipes_rating ON recipes (average_rating DESC);

-- GIN index for dietary labels array
CREATE INDEX IF NOT EXISTS idx_recipes_dietary_labels ON recipes USING GIN (dietary_labels);

-- Index for public recipes
CREATE INDEX IF NOT EXISTS idx_recipes_public ON recipes (is_public, created_at DESC) WHERE is_public = true;

-- Partial index for non-deleted recipes
CREATE INDEX IF NOT EXISTS idx_recipes_active ON recipes (created_at DESC) WHERE deleted_at IS NULL;

-- Composite index for common search patterns
CREATE INDEX IF NOT EXISTS idx_recipes_search_combo ON recipes (user_id, is_public, deleted_at, created_at DESC);

-- Create updated_at trigger
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_recipes_updated_at BEFORE UPDATE ON recipes FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Create advanced search function
CREATE OR REPLACE FUNCTION search_recipes(
    p_user_id UUID,
    p_query TEXT DEFAULT NULL,
    p_meal_types TEXT[] DEFAULT NULL,
    p_complexity TEXT[] DEFAULT NULL,
    p_max_prep_time INTEGER DEFAULT NULL,
    p_max_cook_time INTEGER DEFAULT NULL,
    p_max_total_time INTEGER DEFAULT NULL,
    p_cuisine_type TEXT DEFAULT NULL,
    p_dietary_labels TEXT[] DEFAULT NULL,
    p_include_public BOOLEAN DEFAULT true,
    p_sort_by TEXT DEFAULT 'created_at',
    p_sort_order TEXT DEFAULT 'desc',
    p_limit INTEGER DEFAULT 20,
    p_offset INTEGER DEFAULT 0
)
RETURNS TABLE (
    id UUID,
    user_id UUID,
    title VARCHAR(255),
    description TEXT,
    image_url TEXT,
    prep_time INTEGER,
    cook_time INTEGER,
    total_time INTEGER,
    meal_type VARCHAR(20)[],
    complexity VARCHAR(20),
    cuisine_type VARCHAR(50),
    servings INTEGER,
    ingredients JSONB,
    instructions JSONB,
    dietary_labels TEXT[],
    is_public BOOLEAN,
    average_rating DECIMAL(3,2),
    total_ratings INTEGER,
    created_at TIMESTAMP WITH TIME ZONE,
    updated_at TIMESTAMP WITH TIME ZONE,
    rank_score REAL
) AS $$
DECLARE
    sql_query TEXT;
    where_conditions TEXT[] := ARRAY[]::TEXT[];
    order_clause TEXT;
BEGIN
    -- Build WHERE conditions
    where_conditions := where_conditions || 'deleted_at IS NULL';
    
    -- User access control
    if p_include_public then
        where_conditions := where_conditions || format('(user_id = %L OR is_public = true)', p_user_id);
    else
        where_conditions := where_conditions || format('user_id = %L', p_user_id);
    end if;
    
    -- Text search
    if p_query IS NOT NULL AND p_query != '' then
        where_conditions := where_conditions || format('to_tsvector(''english'', COALESCE(title, '''') || '' '' || COALESCE(description, '''')) @@ plainto_tsquery(''english'', %L)', p_query);
    end if;
    
    -- Meal type filter
    if p_meal_types IS NOT NULL AND array_length(p_meal_types, 1) > 0 then
        where_conditions := where_conditions || format('meal_type && %L', p_meal_types);
    end if;
    
    -- Complexity filter
    if p_complexity IS NOT NULL AND array_length(p_complexity, 1) > 0 then
        where_conditions := where_conditions || format('complexity = ANY(%L)', p_complexity);
    end if;
    
    -- Time filters
    if p_max_prep_time IS NOT NULL then
        where_conditions := where_conditions || format('prep_time <= %s', p_max_prep_time);
    end if;
    
    if p_max_cook_time IS NOT NULL then
        where_conditions := where_conditions || format('cook_time <= %s', p_max_cook_time);
    end if;
    
    if p_max_total_time IS NOT NULL then
        where_conditions := where_conditions || format('total_time <= %s', p_max_total_time);
    end if;
    
    -- Cuisine type filter
    if p_cuisine_type IS NOT NULL then
        where_conditions := where_conditions || format('cuisine_type = %L', p_cuisine_type);
    end if;
    
    -- Dietary labels filter
    if p_dietary_labels IS NOT NULL AND array_length(p_dietary_labels, 1) > 0 then
        where_conditions := where_conditions || format('dietary_labels && %L', p_dietary_labels);
    end if;
    
    -- Build ORDER BY clause
    CASE p_sort_by
        WHEN 'created_at' THEN order_clause := 'created_at ' || p_sort_order;
        WHEN 'updated_at' THEN order_clause := 'updated_at ' || p_sort_order;
        WHEN 'total_time' THEN order_clause := 'total_time ' || p_sort_order;
        WHEN 'average_rating' THEN order_clause := 'average_rating ' || p_sort_order || ', total_ratings DESC';
        WHEN 'title' THEN order_clause := 'title ' || p_sort_order;
        ELSE order_clause := 'created_at DESC';
    END CASE;
    
    -- Add rank score for text search
    if p_query IS NOT NULL AND p_query != '' then
        order_clause := 'ts_rank(to_tsvector(''english'', COALESCE(title, '''') || '' '' || COALESCE(description, '''')), plainto_tsquery(''english'', ''' || p_query || ''')) DESC, ' || order_clause;
    end if;
    
    -- Build final query
    sql_query := format('
        SELECT r.id, r.user_id, r.title, r.description, r.image_url,
               r.prep_time, r.cook_time, r.total_time, r.meal_type,
               r.complexity, r.cuisine_type, r.servings, r.ingredients,
               r.instructions, r.dietary_labels, r.is_public,
               r.average_rating, r.total_ratings, r.created_at, r.updated_at,
               CASE WHEN %L IS NOT NULL AND %L != '''' THEN
                   ts_rank(to_tsvector(''english'', COALESCE(r.title, '''') || '' '' || COALESCE(r.description, '''')), plainto_tsquery(''english'', %L))
               ELSE 0.0 END as rank_score
        FROM recipes r
        WHERE %s
        ORDER BY %s
        LIMIT %s OFFSET %s',
        p_query, p_query, p_query,
        array_to_string(where_conditions, ' AND '),
        order_clause,
        p_limit, p_offset
    );
    
    RETURN QUERY EXECUTE sql_query;
END;
$$ LANGUAGE plpgsql;
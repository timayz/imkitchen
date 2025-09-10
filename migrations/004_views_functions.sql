-- Database Views and Functions Migration
-- This migration creates views and functions for common queries

-- Active User Meal Plans View
CREATE VIEW active_user_meal_plans AS
SELECT 
    mp.*,
    u.email,
    u.dietary_restrictions,
    u.cooking_skill_level,
    EXTRACT(EPOCH FROM (NOW() - mp.created_at))/3600 AS hours_since_creation,
    jsonb_array_length(COALESCE(mp.meals->'monday', '[]'::jsonb)) + 
    jsonb_array_length(COALESCE(mp.meals->'tuesday', '[]'::jsonb)) + 
    jsonb_array_length(COALESCE(mp.meals->'wednesday', '[]'::jsonb)) + 
    jsonb_array_length(COALESCE(mp.meals->'thursday', '[]'::jsonb)) + 
    jsonb_array_length(COALESCE(mp.meals->'friday', '[]'::jsonb)) + 
    jsonb_array_length(COALESCE(mp.meals->'saturday', '[]'::jsonb)) + 
    jsonb_array_length(COALESCE(mp.meals->'sunday', '[]'::jsonb)) AS total_planned_meals
FROM meal_plans mp
JOIN users u ON mp.user_id = u.id
WHERE mp.status = 'active'
    AND u.deleted_at IS NULL
    AND mp.week_start <= CURRENT_DATE
    AND mp.week_end >= CURRENT_DATE;

-- Recipe Recommendations View
CREATE VIEW recipe_recommendations AS
SELECT 
    r.*,
    COALESCE(rpm.algorithm_score, 0.0) as algorithm_score,
    COALESCE(rpm.completion_rate, 0.0) as completion_rate,
    CASE 
        WHEN r.total_ratings > 50 THEN r.average_rating
        WHEN r.total_ratings > 10 THEN (r.average_rating * r.total_ratings + 3.5 * 10) / (r.total_ratings + 10)
        ELSE 3.5 -- default rating for new recipes
    END AS weighted_rating
FROM recipes r
LEFT JOIN (
    SELECT 
        recipe_id,
        AVG(algorithm_score) as algorithm_score,
        AVG(completion_rate) as completion_rate
    FROM recipe_performance_metrics 
    WHERE date >= CURRENT_DATE - INTERVAL '30 days'
    GROUP BY recipe_id
) rpm ON r.id = rpm.recipe_id
WHERE r.deleted_at IS NULL;

-- Update Recipe Rating Aggregates Function
CREATE OR REPLACE FUNCTION update_recipe_rating_aggregates()
RETURNS TRIGGER AS $$
BEGIN
    -- Update recipe aggregates when rating is inserted/updated
    UPDATE recipes 
    SET 
        average_rating = (
            SELECT ROUND(AVG(overall_rating)::numeric, 2)
            FROM recipe_ratings 
            WHERE recipe_id = COALESCE(NEW.recipe_id, OLD.recipe_id)
        ),
        total_ratings = (
            SELECT COUNT(*)
            FROM recipe_ratings 
            WHERE recipe_id = COALESCE(NEW.recipe_id, OLD.recipe_id)
        ),
        updated_at = NOW()
    WHERE id = COALESCE(NEW.recipe_id, OLD.recipe_id);
    
    RETURN COALESCE(NEW, OLD);
END;
$$ LANGUAGE plpgsql;

-- Trigger for automatic rating aggregation
CREATE TRIGGER trigger_update_recipe_ratings
    AFTER INSERT OR UPDATE OR DELETE ON recipe_ratings
    FOR EACH ROW
    EXECUTE FUNCTION update_recipe_rating_aggregates();

-- Search Recipes Function
CREATE OR REPLACE FUNCTION search_recipes(
    search_query TEXT DEFAULT NULL,
    dietary_restrictions TEXT[] DEFAULT '{}',
    max_cook_time INTEGER DEFAULT NULL,
    meal_types TEXT[] DEFAULT '{}',
    complexity_levels TEXT[] DEFAULT '{}',
    min_rating DECIMAL DEFAULT 0.0,
    limit_count INTEGER DEFAULT 50,
    offset_count INTEGER DEFAULT 0
)
RETURNS TABLE(
    recipe_id UUID,
    title VARCHAR,
    image_url TEXT,
    total_time INTEGER,
    complexity VARCHAR,
    average_rating DECIMAL,
    relevance_score REAL
) AS $$
BEGIN
    RETURN QUERY
    SELECT 
        r.id,
        r.title,
        r.image_url,
        r.total_time,
        r.complexity,
        r.average_rating,
        CASE 
            WHEN search_query IS NOT NULL THEN 
                ts_rank(to_tsvector('english', r.title || ' ' || COALESCE(r.description, '')), 
                        plainto_tsquery('english', search_query))
            ELSE 1.0
        END::REAL as relevance_score
    FROM recipes r
    WHERE r.deleted_at IS NULL
        AND (search_query IS NULL OR 
             to_tsvector('english', r.title || ' ' || COALESCE(r.description, '')) @@ 
             plainto_tsquery('english', search_query))
        AND (array_length(dietary_restrictions, 1) IS NULL OR 
             r.dietary_labels && dietary_restrictions)
        AND (max_cook_time IS NULL OR r.total_time <= max_cook_time)
        AND (array_length(meal_types, 1) IS NULL OR r.meal_type && meal_types)
        AND (array_length(complexity_levels, 1) IS NULL OR r.complexity = ANY(complexity_levels))
        AND r.average_rating >= min_rating
    ORDER BY relevance_score DESC, r.average_rating DESC
    LIMIT limit_count
    OFFSET offset_count;
END;
$$ LANGUAGE plpgsql;
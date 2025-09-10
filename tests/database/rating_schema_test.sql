-- Database Schema Tests for Community Rating System
-- These tests validate the database migration and constraints

BEGIN;

-- Test 1: Verify new columns exist in recipes table
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'recipes' 
        AND column_name IN ('is_public', 'is_community', 'rating_distribution')
    ) THEN
        RAISE EXCEPTION 'Required columns missing from recipes table';
    END IF;
END $$;

-- Test 2: Verify new columns exist in recipe_ratings table  
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'recipe_ratings' 
        AND column_name IN ('moderation_status', 'flagged_reason', 'review_text_length')
    ) THEN
        RAISE EXCEPTION 'Required columns missing from recipe_ratings table';
    END IF;
END $$;

-- Test 3: Test rating aggregation function
INSERT INTO recipes (id, title, prep_time, cook_time, ingredients, instructions, is_community, is_public) 
VALUES ('550e8400-e29b-41d4-a716-446655440001', 'Test Recipe', 30, 45, '[]'::jsonb, '[]'::jsonb, true, true);

INSERT INTO users (id, email) 
VALUES ('550e8400-e29b-41d4-a716-446655440002', 'test@example.com');

INSERT INTO recipe_ratings (id, recipe_id, user_id, overall_rating, moderation_status)
VALUES 
    ('550e8400-e29b-41d4-a716-446655440003', '550e8400-e29b-41d4-a716-446655440001', '550e8400-e29b-41d4-a716-446655440002', 4, 'approved');

-- Verify aggregation worked
DO $$
DECLARE
    avg_rating DECIMAL;
    total_count INTEGER;
    distribution JSONB;
BEGIN
    SELECT average_rating, total_ratings, rating_distribution 
    INTO avg_rating, total_count, distribution
    FROM recipes 
    WHERE id = '550e8400-e29b-41d4-a716-446655440001';
    
    IF avg_rating != 4.00 OR total_count != 1 THEN
        RAISE EXCEPTION 'Rating aggregation failed: avg=%, count=%', avg_rating, total_count;
    END IF;
    
    IF (distribution->>'4')::int != 1 THEN
        RAISE EXCEPTION 'Rating distribution incorrect: %', distribution;
    END IF;
END $$;

-- Test 4: Test rating distribution with multiple ratings
INSERT INTO users (id, email) VALUES ('550e8400-e29b-41d4-a716-446655440004', 'test2@example.com');
INSERT INTO users (id, email) VALUES ('550e8400-e29b-41d4-a716-446655440005', 'test3@example.com');

INSERT INTO recipe_ratings (recipe_id, user_id, overall_rating, moderation_status)
VALUES 
    ('550e8400-e29b-41d4-a716-446655440001', '550e8400-e29b-41d4-a716-446655440004', 5, 'approved'),
    ('550e8400-e29b-41d4-a716-446655440001', '550e8400-e29b-41d4-a716-446655440005', 3, 'approved');

-- Verify multiple ratings aggregation
DO $$
DECLARE
    avg_rating DECIMAL;
    total_count INTEGER;
    distribution JSONB;
BEGIN
    SELECT average_rating, total_ratings, rating_distribution 
    INTO avg_rating, total_count, distribution
    FROM recipes 
    WHERE id = '550e8400-e29b-41d4-a716-446655440001';
    
    IF avg_rating != 4.00 OR total_count != 3 THEN
        RAISE EXCEPTION 'Multiple rating aggregation failed: avg=%, count=%', avg_rating, total_count;
    END IF;
    
    IF (distribution->>'3')::int != 1 OR (distribution->>'4')::int != 1 OR (distribution->>'5')::int != 1 THEN
        RAISE EXCEPTION 'Rating distribution incorrect: %', distribution;
    END IF;
END $$;

-- Test 5: Test moderation status filtering
INSERT INTO recipe_ratings (recipe_id, user_id, overall_rating, moderation_status)
VALUES ('550e8400-e29b-41d4-a716-446655440001', '550e8400-e29b-41d4-a716-446655440002', 1, 'rejected');

-- Verify rejected ratings are not included in aggregation
DO $$
DECLARE
    total_count INTEGER;
BEGIN
    SELECT total_ratings INTO total_count
    FROM recipes 
    WHERE id = '550e8400-e29b-41d4-a716-446655440001';
    
    IF total_count != 3 THEN
        RAISE EXCEPTION 'Rejected ratings should not be counted: count=%', total_count;
    END IF;
END $$;

-- Test 6: Test recommendation score function
DO $$
DECLARE
    score DECIMAL;
BEGIN
    SELECT calculate_recipe_recommendation_score(4.5, 10, true) INTO score;
    
    IF score <= 0 THEN
        RAISE EXCEPTION 'Recommendation score calculation failed: %', score;
    END IF;
END $$;

-- Test 7: Test community recipes view
DO $$
DECLARE
    recipe_count INTEGER;
BEGIN
    SELECT COUNT(*) INTO recipe_count
    FROM community_recipes_ranked
    WHERE id = '550e8400-e29b-41d4-a716-446655440001';
    
    IF recipe_count != 1 THEN
        RAISE EXCEPTION 'Community recipes view not working: count=%', recipe_count;
    END IF;
END $$;

-- Test 8: Test user rating history view
DO $$
DECLARE
    rating_count INTEGER;
BEGIN
    SELECT COUNT(*) INTO rating_count
    FROM user_rating_history
    WHERE user_id = '550e8400-e29b-41d4-a716-446655440002'
    AND recipe_id = '550e8400-e29b-41d4-a716-446655440001';
    
    IF rating_count != 1 THEN
        RAISE EXCEPTION 'User rating history view not working: count=%', rating_count;
    END IF;
END $$;

-- Test 9: Test constraints
DO $$
BEGIN
    -- Test review text length constraint
    BEGIN
        INSERT INTO recipe_ratings (recipe_id, user_id, overall_rating, review_text)
        VALUES ('550e8400-e29b-41d4-a716-446655440001', '550e8400-e29b-41d4-a716-446655440004', 5, REPEAT('a', 501));
        RAISE EXCEPTION 'Review text length constraint not enforced';
    EXCEPTION
        WHEN check_violation THEN NULL; -- Expected
    END;
    
    -- Test unique constraint
    BEGIN
        INSERT INTO recipe_ratings (recipe_id, user_id, overall_rating)
        VALUES ('550e8400-e29b-41d4-a716-446655440001', '550e8400-e29b-41d4-a716-446655440002', 5);
        RAISE EXCEPTION 'Unique constraint not enforced';
    EXCEPTION
        WHEN unique_violation THEN NULL; -- Expected
    END;
END $$;

-- Cleanup test data
DELETE FROM recipe_ratings WHERE recipe_id = '550e8400-e29b-41d4-a716-446655440001';
DELETE FROM recipes WHERE id = '550e8400-e29b-41d4-a716-446655440001';
DELETE FROM users WHERE id IN ('550e8400-e29b-41d4-a716-446655440002', '550e8400-e29b-41d4-a716-446655440004', '550e8400-e29b-41d4-a716-446655440005');

ROLLBACK;

-- If we get here, all tests passed
SELECT 'All database schema tests passed!' as result;
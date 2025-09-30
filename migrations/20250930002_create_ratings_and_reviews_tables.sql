-- Recipe Rating and Review System Schema Migration
-- Story 2.3: Community Recipe Rating and Review System

-- Enable foreign key constraints
PRAGMA foreign_keys = ON;

-- Recipe Ratings Table
-- Stores individual star ratings from users for recipes
CREATE TABLE IF NOT EXISTS recipe_ratings (
    rating_id TEXT PRIMARY KEY,  -- UUID
    recipe_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    star_rating INTEGER NOT NULL CHECK (star_rating >= 1 AND star_rating <= 5),
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME,
    
    -- Foreign key constraints
    FOREIGN KEY (recipe_id) REFERENCES recipes(recipe_id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES user_profiles(id) ON DELETE CASCADE,
    
    -- Ensure one rating per user per recipe
    UNIQUE(recipe_id, user_id)
);

-- Recipe Reviews Table
-- Stores text reviews with photos from users for recipes
CREATE TABLE IF NOT EXISTS recipe_reviews (
    review_id TEXT PRIMARY KEY,  -- UUID
    recipe_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    rating_id TEXT NOT NULL,  -- Links to the rating this review is associated with
    review_text TEXT NOT NULL CHECK (length(trim(review_text)) >= 10 AND length(review_text) <= 2000),
    photos TEXT,  -- JSON array of photo URLs/paths
    
    -- Moderation fields
    moderation_status TEXT NOT NULL DEFAULT 'pending' CHECK (
        moderation_status IN ('pending', 'approved', 'rejected', 'flagged')
    ),
    moderation_reason TEXT,
    moderated_by TEXT,
    moderated_at DATETIME,
    
    -- Auto-moderation fields
    spam_score REAL DEFAULT 0.0 CHECK (spam_score >= 0.0 AND spam_score <= 1.0),
    sentiment_score REAL DEFAULT 0.5 CHECK (sentiment_score >= 0.0 AND sentiment_score <= 1.0),
    auto_approved BOOLEAN DEFAULT FALSE,
    
    -- Timestamps
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME,
    
    -- Foreign key constraints
    FOREIGN KEY (recipe_id) REFERENCES recipes(recipe_id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES user_profiles(id) ON DELETE CASCADE,
    FOREIGN KEY (rating_id) REFERENCES recipe_ratings(rating_id) ON DELETE CASCADE,
    FOREIGN KEY (moderated_by) REFERENCES user_profiles(id) ON DELETE SET NULL,
    
    -- Ensure one review per rating (and implicitly per user per recipe)
    UNIQUE(rating_id)
);

-- Review Helpfulness Votes Table
-- Tracks which users found which reviews helpful
CREATE TABLE IF NOT EXISTS review_helpfulness_votes (
    vote_id INTEGER PRIMARY KEY AUTOINCREMENT,
    review_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    is_helpful BOOLEAN NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME,
    
    -- Foreign key constraints
    FOREIGN KEY (review_id) REFERENCES recipe_reviews(review_id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES user_profiles(id) ON DELETE CASCADE,
    
    -- Ensure one vote per user per review
    UNIQUE(review_id, user_id)
);

-- Review Flags Table
-- Stores user reports of inappropriate reviews
CREATE TABLE IF NOT EXISTS review_flags (
    flag_id INTEGER PRIMARY KEY AUTOINCREMENT,
    review_id TEXT NOT NULL,
    flagged_by TEXT NOT NULL,
    flag_reason TEXT NOT NULL CHECK (length(trim(flag_reason)) >= 10),
    flag_category TEXT CHECK (flag_category IN ('spam', 'inappropriate', 'fake', 'offensive', 'other')),
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    resolved BOOLEAN DEFAULT FALSE,
    resolved_by TEXT,
    resolved_at DATETIME,
    resolution_action TEXT CHECK (resolution_action IN ('dismissed', 'warning', 'removed', 'banned')),
    
    -- Foreign key constraints
    FOREIGN KEY (review_id) REFERENCES recipe_reviews(review_id) ON DELETE CASCADE,
    FOREIGN KEY (flagged_by) REFERENCES user_profiles(id) ON DELETE CASCADE,
    FOREIGN KEY (resolved_by) REFERENCES user_profiles(id) ON DELETE SET NULL,
    
    -- Prevent duplicate flags from same user for same review
    UNIQUE(review_id, flagged_by)
);

-- Recipe Rating Aggregates Table (Projection/Cache)
-- Stores computed rating statistics for fast retrieval
CREATE TABLE IF NOT EXISTS recipe_rating_aggregates (
    recipe_id TEXT PRIMARY KEY,
    average_rating REAL NOT NULL DEFAULT 0.0 CHECK (average_rating >= 0.0 AND average_rating <= 5.0),
    weighted_average REAL NOT NULL DEFAULT 0.0 CHECK (weighted_average >= 0.0 AND weighted_average <= 5.0),
    total_ratings INTEGER NOT NULL DEFAULT 0 CHECK (total_ratings >= 0),
    
    -- Rating distribution (count for each star level)
    rating_1_count INTEGER NOT NULL DEFAULT 0 CHECK (rating_1_count >= 0),
    rating_2_count INTEGER NOT NULL DEFAULT 0 CHECK (rating_2_count >= 0),
    rating_3_count INTEGER NOT NULL DEFAULT 0 CHECK (rating_3_count >= 0),
    rating_4_count INTEGER NOT NULL DEFAULT 0 CHECK (rating_4_count >= 0),
    rating_5_count INTEGER NOT NULL DEFAULT 0 CHECK (rating_5_count >= 0),
    
    -- Quality metrics
    confidence_score REAL NOT NULL DEFAULT 0.0 CHECK (confidence_score >= 0.0 AND confidence_score <= 1.0),
    recommendation_percentage REAL NOT NULL DEFAULT 0.0 CHECK (recommendation_percentage >= 0.0 AND recommendation_percentage <= 100.0),
    
    -- Review statistics
    total_reviews INTEGER NOT NULL DEFAULT 0 CHECK (total_reviews >= 0),
    approved_reviews INTEGER NOT NULL DEFAULT 0 CHECK (approved_reviews >= 0),
    reviews_with_photos INTEGER NOT NULL DEFAULT 0 CHECK (reviews_with_photos >= 0),
    average_helpfulness REAL NOT NULL DEFAULT 0.0,
    
    -- Timestamps
    last_updated DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Foreign key constraint
    FOREIGN KEY (recipe_id) REFERENCES recipes(recipe_id) ON DELETE CASCADE
);

-- Performance Indexes
CREATE INDEX IF NOT EXISTS idx_recipe_ratings_recipe_id ON recipe_ratings(recipe_id);
CREATE INDEX IF NOT EXISTS idx_recipe_ratings_user_id ON recipe_ratings(user_id);
CREATE INDEX IF NOT EXISTS idx_recipe_ratings_created_at ON recipe_ratings(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_recipe_ratings_star_rating ON recipe_ratings(star_rating);

CREATE INDEX IF NOT EXISTS idx_recipe_reviews_recipe_id ON recipe_reviews(recipe_id);
CREATE INDEX IF NOT EXISTS idx_recipe_reviews_user_id ON recipe_reviews(user_id);
CREATE INDEX IF NOT EXISTS idx_recipe_reviews_rating_id ON recipe_reviews(rating_id);
CREATE INDEX IF NOT EXISTS idx_recipe_reviews_moderation_status ON recipe_reviews(moderation_status);
CREATE INDEX IF NOT EXISTS idx_recipe_reviews_created_at ON recipe_reviews(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_recipe_reviews_auto_approved ON recipe_reviews(auto_approved);

CREATE INDEX IF NOT EXISTS idx_review_helpfulness_votes_review_id ON review_helpfulness_votes(review_id);
CREATE INDEX IF NOT EXISTS idx_review_helpfulness_votes_user_id ON review_helpfulness_votes(user_id);

CREATE INDEX IF NOT EXISTS idx_review_flags_review_id ON review_flags(review_id);
CREATE INDEX IF NOT EXISTS idx_review_flags_flagged_by ON review_flags(flagged_by);
CREATE INDEX IF NOT EXISTS idx_review_flags_resolved ON review_flags(resolved);
CREATE INDEX IF NOT EXISTS idx_review_flags_created_at ON review_flags(created_at DESC);

-- Triggers for maintaining rating aggregates

-- Trigger: Update aggregates when new rating is added
CREATE TRIGGER IF NOT EXISTS update_rating_aggregate_on_insert 
AFTER INSERT ON recipe_ratings 
BEGIN
    INSERT OR REPLACE INTO recipe_rating_aggregates (
        recipe_id,
        total_ratings,
        rating_1_count,
        rating_2_count,
        rating_3_count,
        rating_4_count,
        rating_5_count,
        average_rating,
        weighted_average,
        confidence_score,
        recommendation_percentage,
        last_updated
    )
    SELECT 
        recipe_id,
        COUNT(*) as total_ratings,
        SUM(CASE WHEN star_rating = 1 THEN 1 ELSE 0 END) as rating_1_count,
        SUM(CASE WHEN star_rating = 2 THEN 1 ELSE 0 END) as rating_2_count,
        SUM(CASE WHEN star_rating = 3 THEN 1 ELSE 0 END) as rating_3_count,
        SUM(CASE WHEN star_rating = 4 THEN 1 ELSE 0 END) as rating_4_count,
        SUM(CASE WHEN star_rating = 5 THEN 1 ELSE 0 END) as rating_5_count,
        AVG(CAST(star_rating AS REAL)) as average_rating,
        -- Bayesian weighted average with global average of 3.0
        (AVG(CAST(star_rating AS REAL)) * COUNT(*) + 3.0 * 10) / (COUNT(*) + 10) as weighted_average,
        -- Confidence score based on sample size
        CASE 
            WHEN COUNT(*) = 0 THEN 0.0
            WHEN COUNT(*) <= 5 THEN 0.3
            WHEN COUNT(*) <= 15 THEN 0.6
            WHEN COUNT(*) <= 50 THEN 0.8
            ELSE 0.95
        END as confidence_score,
        -- Recommendation percentage (4+ stars)
        CASE 
            WHEN COUNT(*) = 0 THEN 0.0
            ELSE (SUM(CASE WHEN star_rating >= 4 THEN 1 ELSE 0 END) * 100.0 / COUNT(*))
        END as recommendation_percentage,
        CURRENT_TIMESTAMP
    FROM recipe_ratings 
    WHERE recipe_id = NEW.recipe_id;
    
    -- Update the main recipes table rating
    UPDATE recipes 
    SET rating = (SELECT weighted_average FROM recipe_rating_aggregates WHERE recipe_id = NEW.recipe_id),
        review_count = (SELECT total_ratings FROM recipe_rating_aggregates WHERE recipe_id = NEW.recipe_id)
    WHERE recipe_id = NEW.recipe_id;
END;

-- Trigger: Update aggregates when rating is updated
CREATE TRIGGER IF NOT EXISTS update_rating_aggregate_on_update 
AFTER UPDATE ON recipe_ratings 
BEGIN
    INSERT OR REPLACE INTO recipe_rating_aggregates (
        recipe_id,
        total_ratings,
        rating_1_count,
        rating_2_count,
        rating_3_count,
        rating_4_count,
        rating_5_count,
        average_rating,
        weighted_average,
        confidence_score,
        recommendation_percentage,
        last_updated
    )
    SELECT 
        recipe_id,
        COUNT(*) as total_ratings,
        SUM(CASE WHEN star_rating = 1 THEN 1 ELSE 0 END) as rating_1_count,
        SUM(CASE WHEN star_rating = 2 THEN 1 ELSE 0 END) as rating_2_count,
        SUM(CASE WHEN star_rating = 3 THEN 1 ELSE 0 END) as rating_3_count,
        SUM(CASE WHEN star_rating = 4 THEN 1 ELSE 0 END) as rating_4_count,
        SUM(CASE WHEN star_rating = 5 THEN 1 ELSE 0 END) as rating_5_count,
        AVG(CAST(star_rating AS REAL)) as average_rating,
        (AVG(CAST(star_rating AS REAL)) * COUNT(*) + 3.0 * 10) / (COUNT(*) + 10) as weighted_average,
        CASE 
            WHEN COUNT(*) = 0 THEN 0.0
            WHEN COUNT(*) <= 5 THEN 0.3
            WHEN COUNT(*) <= 15 THEN 0.6
            WHEN COUNT(*) <= 50 THEN 0.8
            ELSE 0.95
        END as confidence_score,
        CASE 
            WHEN COUNT(*) = 0 THEN 0.0
            ELSE (SUM(CASE WHEN star_rating >= 4 THEN 1 ELSE 0 END) * 100.0 / COUNT(*))
        END as recommendation_percentage,
        CURRENT_TIMESTAMP
    FROM recipe_ratings 
    WHERE recipe_id = NEW.recipe_id;
    
    UPDATE recipes 
    SET rating = (SELECT weighted_average FROM recipe_rating_aggregates WHERE recipe_id = NEW.recipe_id),
        review_count = (SELECT total_ratings FROM recipe_rating_aggregates WHERE recipe_id = NEW.recipe_id)
    WHERE recipe_id = NEW.recipe_id;
END;

-- Trigger: Update aggregates when rating is deleted
CREATE TRIGGER IF NOT EXISTS update_rating_aggregate_on_delete 
AFTER DELETE ON recipe_ratings 
BEGIN
    INSERT OR REPLACE INTO recipe_rating_aggregates (
        recipe_id,
        total_ratings,
        rating_1_count,
        rating_2_count,
        rating_3_count,
        rating_4_count,
        rating_5_count,
        average_rating,
        weighted_average,
        confidence_score,
        recommendation_percentage,
        last_updated
    )
    SELECT 
        recipe_id,
        COUNT(*) as total_ratings,
        SUM(CASE WHEN star_rating = 1 THEN 1 ELSE 0 END) as rating_1_count,
        SUM(CASE WHEN star_rating = 2 THEN 1 ELSE 0 END) as rating_2_count,
        SUM(CASE WHEN star_rating = 3 THEN 1 ELSE 0 END) as rating_3_count,
        SUM(CASE WHEN star_rating = 4 THEN 1 ELSE 0 END) as rating_4_count,
        SUM(CASE WHEN star_rating = 5 THEN 1 ELSE 0 END) as rating_5_count,
        COALESCE(AVG(CAST(star_rating AS REAL)), 0.0) as average_rating,
        CASE 
            WHEN COUNT(*) = 0 THEN 0.0
            ELSE (AVG(CAST(star_rating AS REAL)) * COUNT(*) + 3.0 * 10) / (COUNT(*) + 10)
        END as weighted_average,
        CASE 
            WHEN COUNT(*) = 0 THEN 0.0
            WHEN COUNT(*) <= 5 THEN 0.3
            WHEN COUNT(*) <= 15 THEN 0.6
            WHEN COUNT(*) <= 50 THEN 0.8
            ELSE 0.95
        END as confidence_score,
        CASE 
            WHEN COUNT(*) = 0 THEN 0.0
            ELSE (SUM(CASE WHEN star_rating >= 4 THEN 1 ELSE 0 END) * 100.0 / COUNT(*))
        END as recommendation_percentage,
        CURRENT_TIMESTAMP
    FROM recipe_ratings 
    WHERE recipe_id = OLD.recipe_id;
    
    UPDATE recipes 
    SET rating = COALESCE((SELECT weighted_average FROM recipe_rating_aggregates WHERE recipe_id = OLD.recipe_id), 0.0),
        review_count = COALESCE((SELECT total_ratings FROM recipe_rating_aggregates WHERE recipe_id = OLD.recipe_id), 0)
    WHERE recipe_id = OLD.recipe_id;
END;

-- Trigger: Update review statistics when review status changes
CREATE TRIGGER IF NOT EXISTS update_review_stats_on_insert 
AFTER INSERT ON recipe_reviews 
BEGIN
    UPDATE recipe_rating_aggregates 
    SET 
        total_reviews = (
            SELECT COUNT(*) 
            FROM recipe_reviews 
            WHERE recipe_id = NEW.recipe_id
        ),
        approved_reviews = (
            SELECT COUNT(*) 
            FROM recipe_reviews 
            WHERE recipe_id = NEW.recipe_id AND moderation_status = 'approved'
        ),
        reviews_with_photos = (
            SELECT COUNT(*) 
            FROM recipe_reviews 
            WHERE recipe_id = NEW.recipe_id 
            AND photos IS NOT NULL 
            AND photos != '[]' 
            AND trim(photos) != ''
        ),
        last_updated = CURRENT_TIMESTAMP
    WHERE recipe_id = NEW.recipe_id;
END;

CREATE TRIGGER IF NOT EXISTS update_review_stats_on_update 
AFTER UPDATE ON recipe_reviews 
BEGIN
    UPDATE recipe_rating_aggregates 
    SET 
        total_reviews = (
            SELECT COUNT(*) 
            FROM recipe_reviews 
            WHERE recipe_id = NEW.recipe_id
        ),
        approved_reviews = (
            SELECT COUNT(*) 
            FROM recipe_reviews 
            WHERE recipe_id = NEW.recipe_id AND moderation_status = 'approved'
        ),
        reviews_with_photos = (
            SELECT COUNT(*) 
            FROM recipe_reviews 
            WHERE recipe_id = NEW.recipe_id 
            AND photos IS NOT NULL 
            AND photos != '[]' 
            AND trim(photos) != ''
        ),
        last_updated = CURRENT_TIMESTAMP
    WHERE recipe_id = NEW.recipe_id;
END;

CREATE TRIGGER IF NOT EXISTS update_review_stats_on_delete 
AFTER DELETE ON recipe_reviews 
BEGIN
    UPDATE recipe_rating_aggregates 
    SET 
        total_reviews = (
            SELECT COUNT(*) 
            FROM recipe_reviews 
            WHERE recipe_id = OLD.recipe_id
        ),
        approved_reviews = (
            SELECT COUNT(*) 
            FROM recipe_reviews 
            WHERE recipe_id = OLD.recipe_id AND moderation_status = 'approved'
        ),
        reviews_with_photos = (
            SELECT COUNT(*) 
            FROM recipe_reviews 
            WHERE recipe_id = OLD.recipe_id 
            AND photos IS NOT NULL 
            AND photos != '[]' 
            AND trim(photos) != ''
        ),
        last_updated = CURRENT_TIMESTAMP
    WHERE recipe_id = OLD.recipe_id;
END;

-- Trigger: Update helpfulness statistics when votes change
CREATE TRIGGER IF NOT EXISTS update_helpfulness_stats_on_vote_insert 
AFTER INSERT ON review_helpfulness_votes 
BEGIN
    UPDATE recipe_rating_aggregates 
    SET 
        average_helpfulness = (
            SELECT COALESCE(AVG(
                CASE WHEN helpful_count - unhelpful_count > 0 
                     THEN helpful_count - unhelpful_count 
                     ELSE 0 END
            ), 0.0)
            FROM (
                SELECT 
                    r.review_id,
                    SUM(CASE WHEN v.is_helpful = 1 THEN 1 ELSE 0 END) as helpful_count,
                    SUM(CASE WHEN v.is_helpful = 0 THEN 1 ELSE 0 END) as unhelpful_count
                FROM recipe_reviews r
                LEFT JOIN review_helpfulness_votes v ON r.review_id = v.review_id
                WHERE r.recipe_id = (
                    SELECT recipe_id FROM recipe_reviews WHERE review_id = NEW.review_id
                )
                GROUP BY r.review_id
            )
        ),
        last_updated = CURRENT_TIMESTAMP
    WHERE recipe_id = (
        SELECT recipe_id FROM recipe_reviews WHERE review_id = NEW.review_id
    );
END;

CREATE TRIGGER IF NOT EXISTS update_helpfulness_stats_on_vote_update 
AFTER UPDATE ON review_helpfulness_votes 
BEGIN
    UPDATE recipe_rating_aggregates 
    SET 
        average_helpfulness = (
            SELECT COALESCE(AVG(
                CASE WHEN helpful_count - unhelpful_count > 0 
                     THEN helpful_count - unhelpful_count 
                     ELSE 0 END
            ), 0.0)
            FROM (
                SELECT 
                    r.review_id,
                    SUM(CASE WHEN v.is_helpful = 1 THEN 1 ELSE 0 END) as helpful_count,
                    SUM(CASE WHEN v.is_helpful = 0 THEN 1 ELSE 0 END) as unhelpful_count
                FROM recipe_reviews r
                LEFT JOIN review_helpfulness_votes v ON r.review_id = v.review_id
                WHERE r.recipe_id = (
                    SELECT recipe_id FROM recipe_reviews WHERE review_id = NEW.review_id
                )
                GROUP BY r.review_id
            )
        ),
        last_updated = CURRENT_TIMESTAMP
    WHERE recipe_id = (
        SELECT recipe_id FROM recipe_reviews WHERE review_id = NEW.review_id
    );
END;

CREATE TRIGGER IF NOT EXISTS update_helpfulness_stats_on_vote_delete 
AFTER DELETE ON review_helpfulness_votes 
BEGIN
    UPDATE recipe_rating_aggregates 
    SET 
        average_helpfulness = (
            SELECT COALESCE(AVG(
                CASE WHEN helpful_count - unhelpful_count > 0 
                     THEN helpful_count - unhelpful_count 
                     ELSE 0 END
            ), 0.0)
            FROM (
                SELECT 
                    r.review_id,
                    SUM(CASE WHEN v.is_helpful = 1 THEN 1 ELSE 0 END) as helpful_count,
                    SUM(CASE WHEN v.is_helpful = 0 THEN 1 ELSE 0 END) as unhelpful_count
                FROM recipe_reviews r
                LEFT JOIN review_helpfulness_votes v ON r.review_id = v.review_id
                WHERE r.recipe_id = (
                    SELECT recipe_id FROM recipe_reviews WHERE review_id = OLD.review_id
                )
                GROUP BY r.review_id
            )
        ),
        last_updated = CURRENT_TIMESTAMP
    WHERE recipe_id = (
        SELECT recipe_id FROM recipe_reviews WHERE review_id = OLD.review_id
    );
END;

-- Create aggregate entries for existing recipes
INSERT OR IGNORE INTO recipe_rating_aggregates (recipe_id, last_updated)
SELECT recipe_id, CURRENT_TIMESTAMP FROM recipes;
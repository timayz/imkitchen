-- Supporting Tables Migration
-- This migration creates supporting tables for shopping lists and user preference tracking

-- Shopping Lists table
CREATE TABLE shopping_lists (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    meal_plan_id UUID REFERENCES meal_plans(id) ON DELETE SET NULL,
    
    -- List Data
    name VARCHAR(255) NOT NULL DEFAULT 'Weekly Shopping List',
    items JSONB NOT NULL, -- [{"name": "flour", "quantity": 2, "unit": "lbs", "category": "baking", "checked": false}]
    
    -- Organization
    store_preferences JSONB, -- aisle ordering, preferred brands
    estimated_total_cost DECIMAL(10,2),
    
    -- Status
    status VARCHAR(20) DEFAULT 'active' CHECK (status IN ('active', 'completed', 'archived')),
    completed_at TIMESTAMP WITH TIME ZONE,
    
    -- Metadata
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Indexes for shopping_lists table
CREATE INDEX idx_shopping_lists_user_id ON shopping_lists(user_id);
CREATE INDEX idx_shopping_lists_meal_plan_id ON shopping_lists(meal_plan_id);
CREATE INDEX idx_shopping_lists_status ON shopping_lists(status);
CREATE INDEX idx_shopping_lists_created_at ON shopping_lists(created_at DESC);

-- User Preferences History table
CREATE TABLE user_preference_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- Preference Tracking
    preference_type VARCHAR(50) NOT NULL, -- 'dietary_restriction_added', 'skill_level_updated'
    old_value JSONB,
    new_value JSONB,
    change_reason VARCHAR(255), -- 'onboarding', 'user_update', 'algorithm_learning'
    
    -- Context
    triggered_by VARCHAR(50), -- 'user_action', 'feedback_analysis', 'preference_quiz'
    confidence_score DECIMAL(5,4), -- algorithm confidence in learned preferences
    
    -- Metadata
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Indexes for user_preference_history table
CREATE INDEX idx_user_pref_history_user_id ON user_preference_history(user_id);
CREATE INDEX idx_user_pref_history_type ON user_preference_history(preference_type);
CREATE INDEX idx_user_pref_history_created_at ON user_preference_history(created_at DESC);
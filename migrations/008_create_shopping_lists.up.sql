-- Create shopping_lists table
CREATE TABLE IF NOT EXISTS shopping_lists (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    meal_plan_id UUID REFERENCES meal_plans(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'completed', 'archived')),
    generated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create shopping_items table
CREATE TABLE IF NOT EXISTS shopping_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    shopping_list_id UUID NOT NULL REFERENCES shopping_lists(id) ON DELETE CASCADE,
    ingredient_name VARCHAR(255) NOT NULL,
    amount DECIMAL(10,3) NOT NULL DEFAULT 0,
    unit VARCHAR(100) NOT NULL,
    category VARCHAR(50) NOT NULL DEFAULT 'other' CHECK (category IN ('produce', 'dairy', 'pantry', 'protein', 'other')),
    is_completed BOOLEAN NOT NULL DEFAULT FALSE,
    notes TEXT NULL,
    recipe_sources UUID[] NULL,
    estimated_cost DECIMAL(10,2) NULL,
    completed_at TIMESTAMPTZ NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_shopping_lists_user_id ON shopping_lists(user_id);
CREATE INDEX IF NOT EXISTS idx_shopping_lists_meal_plan_id ON shopping_lists(meal_plan_id);
CREATE INDEX IF NOT EXISTS idx_shopping_lists_status ON shopping_lists(status);
CREATE INDEX IF NOT EXISTS idx_shopping_lists_created_at ON shopping_lists(created_at DESC);

CREATE INDEX IF NOT EXISTS idx_shopping_items_list_id ON shopping_items(shopping_list_id);
CREATE INDEX IF NOT EXISTS idx_shopping_items_category ON shopping_items(category);
CREATE INDEX IF NOT EXISTS idx_shopping_items_completed ON shopping_items(is_completed);
CREATE INDEX IF NOT EXISTS idx_shopping_items_ingredient_name ON shopping_items(ingredient_name);

-- Create updated_at triggers
CREATE OR REPLACE FUNCTION update_shopping_lists_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION update_shopping_items_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_shopping_lists_updated_at
    BEFORE UPDATE ON shopping_lists
    FOR EACH ROW
    EXECUTE FUNCTION update_shopping_lists_updated_at();

CREATE TRIGGER trigger_shopping_items_updated_at
    BEFORE UPDATE ON shopping_items
    FOR EACH ROW
    EXECUTE FUNCTION update_shopping_items_updated_at();

-- Create view for shopping list summaries
CREATE OR REPLACE VIEW shopping_list_summaries AS
SELECT 
    sl.id,
    sl.user_id,
    sl.meal_plan_id,
    sl.name,
    sl.status,
    sl.generated_at,
    sl.completed_at,
    sl.created_at,
    sl.updated_at,
    COUNT(si.id) as total_items,
    COUNT(CASE WHEN si.is_completed THEN 1 END) as completed_items,
    ROUND(
        COUNT(CASE WHEN si.is_completed THEN 1 END)::NUMERIC / 
        NULLIF(COUNT(si.id), 0) * 100, 
        2
    ) as completion_percentage
FROM shopping_lists sl
LEFT JOIN shopping_items si ON sl.id = si.shopping_list_id
GROUP BY sl.id, sl.user_id, sl.meal_plan_id, sl.name, sl.status, 
         sl.generated_at, sl.completed_at, sl.created_at, sl.updated_at;
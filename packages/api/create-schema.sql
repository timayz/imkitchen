-- ImKitchen Database Schema
-- Manual migration for NixOS Prisma compatibility

-- Users table
CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY,
    email TEXT UNIQUE NOT NULL,
    name TEXT,
    image TEXT,
    "emailVerified" TIMESTAMP WITH TIME ZONE,
    preferences JSONB,
    "notificationSettings" JSONB,
    "createdAt" TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    "lastActiveAt" TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- NextAuth.js tables
CREATE TABLE IF NOT EXISTS accounts (
    id TEXT PRIMARY KEY,
    "userId" TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    type TEXT NOT NULL,
    provider TEXT NOT NULL,
    "providerAccountId" TEXT NOT NULL,
    refresh_token TEXT,
    access_token TEXT,
    expires_at INTEGER,
    token_type TEXT,
    scope TEXT,
    id_token TEXT,
    session_state TEXT,
    UNIQUE(provider, "providerAccountId")
);

CREATE TABLE IF NOT EXISTS sessions (
    id TEXT PRIMARY KEY,
    "sessionToken" TEXT UNIQUE NOT NULL,
    "userId" TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    expires TIMESTAMP WITH TIME ZONE NOT NULL
);

CREATE TABLE IF NOT EXISTS verificationtokens (
    identifier TEXT NOT NULL,
    token TEXT UNIQUE NOT NULL,
    expires TIMESTAMP WITH TIME ZONE NOT NULL,
    UNIQUE(identifier, token)
);

-- Recipes table
CREATE TABLE IF NOT EXISTS recipes (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    description TEXT,
    ingredients JSONB NOT NULL,
    instructions JSONB NOT NULL,
    "prepTimeMinutes" INTEGER NOT NULL,
    "cookTimeMinutes" INTEGER NOT NULL,
    "totalTimeMinutes" INTEGER NOT NULL,
    servings INTEGER DEFAULT 4,
    difficulty TEXT,
    cuisine TEXT,
    "dietaryTags" TEXT[],
    "nutritionData" JSONB,
    "imageUrl" TEXT,
    "sourceUrl" TEXT,
    "isPublic" BOOLEAN DEFAULT false,
    "userId" TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    "createdAt" TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    "updatedAt" TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Meal Plans table
CREATE TABLE IF NOT EXISTS meal_plans (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    description TEXT,
    "startDate" TIMESTAMP WITH TIME ZONE NOT NULL,
    "endDate" TIMESTAMP WITH TIME ZONE NOT NULL,
    "userId" TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    "createdAt" TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    "updatedAt" TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Meals table
CREATE TABLE IF NOT EXISTS meals (
    id TEXT PRIMARY KEY,
    "mealPlanId" TEXT NOT NULL REFERENCES meal_plans(id) ON DELETE CASCADE,
    "recipeId" TEXT NOT NULL REFERENCES recipes(id),
    "scheduledDate" TIMESTAMP WITH TIME ZONE NOT NULL,
    "mealType" TEXT NOT NULL,
    "targetServingTime" TIMESTAMP WITH TIME ZONE,
    servings INTEGER DEFAULT 4,
    "timingSchedule" JSONB,
    notes TEXT,
    completed BOOLEAN DEFAULT false,
    "createdAt" TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Shopping Lists table
CREATE TABLE IF NOT EXISTS shopping_lists (
    id TEXT PRIMARY KEY,
    "mealPlanId" TEXT NOT NULL REFERENCES meal_plans(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    items JSONB NOT NULL,
    completed BOOLEAN DEFAULT false,
    "createdAt" TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    "completedAt" TIMESTAMP WITH TIME ZONE
);

-- Timing Notifications table
CREATE TABLE IF NOT EXISTS timing_notifications (
    id TEXT PRIMARY KEY,
    "mealId" TEXT NOT NULL REFERENCES meals(id) ON DELETE CASCADE,
    "userId" TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    "notificationType" TEXT NOT NULL,
    "scheduledTime" TIMESTAMP WITH TIME ZONE NOT NULL,
    status TEXT NOT NULL,
    "deliveryResults" JSONB,
    "createdAt" TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_recipes_userId ON recipes("userId");
CREATE INDEX IF NOT EXISTS idx_recipes_isPublic ON recipes("isPublic");
CREATE INDEX IF NOT EXISTS idx_meal_plans_userId ON meal_plans("userId");
CREATE INDEX IF NOT EXISTS idx_meal_plans_dates ON meal_plans("startDate", "endDate");
CREATE INDEX IF NOT EXISTS idx_meals_mealPlanId ON meals("mealPlanId");
CREATE INDEX IF NOT EXISTS idx_meals_scheduledDate ON meals("scheduledDate");
CREATE INDEX IF NOT EXISTS idx_timing_notifications_scheduledTime ON timing_notifications("scheduledTime");
CREATE INDEX IF NOT EXISTS idx_timing_notifications_status ON timing_notifications(status);

-- Create trigger for updating updatedAt timestamps
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW."updatedAt" = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Apply triggers
DROP TRIGGER IF EXISTS update_recipes_updated_at ON recipes;
CREATE TRIGGER update_recipes_updated_at BEFORE UPDATE ON recipes FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_meal_plans_updated_at ON meal_plans;
CREATE TRIGGER update_meal_plans_updated_at BEFORE UPDATE ON meal_plans FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
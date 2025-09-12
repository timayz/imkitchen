import { Pool } from 'pg';

// Alternative database client for NixOS compatibility
const pool = new Pool({
  connectionString: process.env.DATABASE_URL,
});

// Helper function to generate CUID-like IDs
function generateId(): string {
  return 'ck' + Math.random().toString(36).substring(2, 15) + Math.random().toString(36).substring(2, 15);
}

// Example database operations
export class DatabaseOperations {
  // User operations
  static async createUser(email: string, name?: string) {
    const id = generateId();
    const query = `
      INSERT INTO users (id, email, name) 
      VALUES ($1, $2, $3) 
      RETURNING *
    `;
    const result = await pool.query(query, [id, email, name]);
    return result.rows[0];
  }

  static async getUserByEmail(email: string) {
    const query = 'SELECT * FROM users WHERE email = $1';
    const result = await pool.query(query, [email]);
    return result.rows[0];
  }

  static async getAllUsers() {
    const query = 'SELECT * FROM users ORDER BY "createdAt" DESC';
    const result = await pool.query(query);
    return result.rows;
  }

  // Recipe operations
  static async createRecipe(recipe: {
    title: string;
    description?: string;
    ingredients: any[];
    instructions: any[];
    prepTimeMinutes: number;
    cookTimeMinutes: number;
    totalTimeMinutes: number;
    userId: string;
    servings?: number;
    difficulty?: string;
    cuisine?: string;
    dietaryTags?: string[];
  }) {
    const id = generateId();
    const query = `
      INSERT INTO recipes (
        id, title, description, ingredients, instructions, 
        "prepTimeMinutes", "cookTimeMinutes", "totalTimeMinutes", 
        "userId", servings, difficulty, cuisine, "dietaryTags"
      ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
      RETURNING *
    `;
    
    const values = [
      id,
      recipe.title,
      recipe.description || null,
      JSON.stringify(recipe.ingredients),
      JSON.stringify(recipe.instructions),
      recipe.prepTimeMinutes,
      recipe.cookTimeMinutes,
      recipe.totalTimeMinutes,
      recipe.userId,
      recipe.servings || 4,
      recipe.difficulty || null,
      recipe.cuisine || null,
      recipe.dietaryTags || []
    ];

    const result = await pool.query(query, values);
    return result.rows[0];
  }

  static async getRecipesByUser(userId: string) {
    const query = `
      SELECT r.*, u.name as user_name 
      FROM recipes r
      JOIN users u ON r."userId" = u.id
      WHERE r."userId" = $1
      ORDER BY r."createdAt" DESC
    `;
    const result = await pool.query(query, [userId]);
    return result.rows;
  }

  // Meal plan operations
  static async createMealPlan(mealPlan: {
    title: string;
    description?: string;
    startDate: Date;
    endDate: Date;
    userId: string;
  }) {
    const id = generateId();
    const query = `
      INSERT INTO meal_plans (id, title, description, "startDate", "endDate", "userId")
      VALUES ($1, $2, $3, $4, $5, $6)
      RETURNING *
    `;
    
    const result = await pool.query(query, [
      id,
      mealPlan.title,
      mealPlan.description || null,
      mealPlan.startDate,
      mealPlan.endDate,
      mealPlan.userId
    ]);
    return result.rows[0];
  }

  static async getMealPlanWithMeals(mealPlanId: string) {
    const query = `
      SELECT 
        mp.*,
        json_agg(
          json_build_object(
            'id', m.id,
            'scheduledDate', m."scheduledDate",
            'mealType', m."mealType",
            'servings', m.servings,
            'completed', m.completed,
            'recipe', json_build_object(
              'id', r.id,
              'title', r.title,
              'prepTimeMinutes', r."prepTimeMinutes",
              'cookTimeMinutes', r."cookTimeMinutes"
            )
          )
        ) as meals
      FROM meal_plans mp
      LEFT JOIN meals m ON mp.id = m."mealPlanId"
      LEFT JOIN recipes r ON m."recipeId" = r.id
      WHERE mp.id = $1
      GROUP BY mp.id
    `;
    
    const result = await pool.query(query, [mealPlanId]);
    return result.rows[0];
  }

  // Close connection
  static async close() {
    await pool.end();
  }
}

// Usage examples
async function examples() {
  try {
    console.log('🚀 Running database examples...');
    
    // Create a user
    const user = await DatabaseOperations.createUser(
      'chef@imkitchen.com',
      'Chef Smith'
    );
    console.log('Created user:', user);

    // Create a recipe
    const recipe = await DatabaseOperations.createRecipe({
      title: 'Spaghetti Carbonara',
      description: 'Classic Italian pasta dish',
      ingredients: [
        { name: 'Spaghetti', amount: '400g' },
        { name: 'Eggs', amount: '4 large' },
        { name: 'Pancetta', amount: '200g' },
        { name: 'Parmesan cheese', amount: '100g' }
      ],
      instructions: [
        { step: 1, text: 'Boil pasta in salted water', timing: '10-12 minutes' },
        { step: 2, text: 'Cook pancetta until crispy', timing: '5 minutes' },
        { step: 3, text: 'Mix eggs and cheese', timing: '2 minutes' },
        { step: 4, text: 'Combine all ingredients', timing: '3 minutes' }
      ],
      prepTimeMinutes: 15,
      cookTimeMinutes: 20,
      totalTimeMinutes: 35,
      userId: user.id,
      difficulty: 'Medium',
      cuisine: 'Italian',
      dietaryTags: ['gluten-containing']
    });
    console.log('Created recipe:', recipe.title);

    // Create a meal plan
    const mealPlan = await DatabaseOperations.createMealPlan({
      title: 'Weekly Dinner Plan',
      description: 'Italian-themed week',
      startDate: new Date(),
      endDate: new Date(Date.now() + 7 * 24 * 60 * 60 * 1000),
      userId: user.id
    });
    console.log('Created meal plan:', mealPlan.title);

    // Get all users
    const users = await DatabaseOperations.getAllUsers();
    console.log('Total users:', users.length);

    console.log('✅ Database examples completed!');
  } catch (error) {
    console.error('❌ Error:', error);
  }
}

// Run examples if this file is executed directly
if (require.main === module) {
  examples()
    .then(() => DatabaseOperations.close())
    .catch(console.error);
}
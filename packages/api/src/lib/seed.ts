import { PrismaClient } from '@prisma/client';

const prisma = new PrismaClient();

async function main() {
  console.log('🌱 Starting database seed...');

  // Create test user
  const user = await prisma.user.upsert({
    where: { email: 'test@imkitchen.com' },
    update: {},
    create: {
      email: 'test@imkitchen.com',
      name: 'Test User',
      preferences: {
        dietaryRestrictions: [],
        skillLevel: 'intermediate',
      },
    },
  });

  console.log(`👤 Created user: ${user.email}`);

  // Create sample recipes
  const recipes = [
    {
      title: 'Classic Spaghetti Carbonara',
      description: 'Traditional Italian pasta dish with eggs, cheese, and pancetta',
      ingredients: [
        { name: 'Spaghetti', amount: 400, unit: 'g' },
        { name: 'Pancetta', amount: 200, unit: 'g' },
        { name: 'Eggs', amount: 4, unit: 'large' },
        { name: 'Pecorino Romano', amount: 100, unit: 'g' },
        { name: 'Black pepper', amount: 1, unit: 'tsp' },
      ],
      instructions: [
        { step: 1, description: 'Cook spaghetti according to package directions', duration: 10 },
        { step: 2, description: 'Crisp pancetta in a large skillet', duration: 5 },
        { step: 3, description: 'Whisk eggs with cheese and pepper', duration: 2 },
        { step: 4, description: 'Combine hot pasta with pancetta and egg mixture', duration: 3 },
      ],
      prepTimeMinutes: 10,
      cookTimeMinutes: 15,
      totalTimeMinutes: 25,
      servings: 4,
      difficulty: 'Medium',
      cuisine: 'Italian',
      dietaryTags: [],
    },
    {
      title: 'Quinoa Buddha Bowl',
      description: 'Healthy and colorful grain bowl with roasted vegetables',
      ingredients: [
        { name: 'Quinoa', amount: 1, unit: 'cup' },
        { name: 'Sweet potato', amount: 1, unit: 'large' },
        { name: 'Broccoli', amount: 1, unit: 'head' },
        { name: 'Chickpeas', amount: 1, unit: 'can' },
        { name: 'Tahini', amount: 3, unit: 'tbsp' },
        { name: 'Lemon', amount: 1, unit: 'whole' },
      ],
      instructions: [
        { step: 1, description: 'Cook quinoa according to package directions', duration: 15 },
        { step: 2, description: 'Roast sweet potato and broccoli', duration: 25 },
        { step: 3, description: 'Heat chickpeas with spices', duration: 5 },
        { step: 4, description: 'Make tahini dressing', duration: 5 },
        { step: 5, description: 'Assemble bowls and serve', duration: 5 },
      ],
      prepTimeMinutes: 15,
      cookTimeMinutes: 30,
      totalTimeMinutes: 45,
      servings: 2,
      difficulty: 'Easy',
      cuisine: 'Mediterranean',
      dietaryTags: ['Vegetarian', 'Vegan', 'Gluten-Free'],
    },
    {
      title: 'Chicken Stir Fry',
      description: 'Quick and easy chicken stir fry with vegetables',
      ingredients: [
        { name: 'Chicken breast', amount: 500, unit: 'g' },
        { name: 'Bell peppers', amount: 2, unit: 'whole' },
        { name: 'Onion', amount: 1, unit: 'large' },
        { name: 'Soy sauce', amount: 3, unit: 'tbsp' },
        { name: 'Garlic', amount: 3, unit: 'cloves' },
        { name: 'Ginger', amount: 1, unit: 'inch' },
      ],
      instructions: [
        { step: 1, description: 'Cut chicken and vegetables', duration: 10 },
        { step: 2, description: 'Heat wok or large skillet', duration: 2 },
        { step: 3, description: 'Cook chicken until done', duration: 8 },
        { step: 4, description: 'Add vegetables and stir fry', duration: 5 },
        { step: 5, description: 'Add sauce and finish', duration: 2 },
      ],
      prepTimeMinutes: 15,
      cookTimeMinutes: 15,
      totalTimeMinutes: 30,
      servings: 4,
      difficulty: 'Easy',
      cuisine: 'Asian',
      dietaryTags: ['Gluten-Free'],
    },
  ];

  for (const recipeData of recipes) {
    const recipe = await prisma.recipe.create({
      data: {
        ...recipeData,
        userId: user.id,
        ingredients: recipeData.ingredients,
        instructions: recipeData.instructions,
      },
    });
    console.log(`🍳 Created recipe: ${recipe.title}`);
  }

  console.log('✅ Database seed completed!');
}

main()
  .catch((e) => {
    console.error('❌ Seed failed:', e);
    process.exit(1);
  })
  .finally(async () => {
    await prisma.$disconnect();
  });
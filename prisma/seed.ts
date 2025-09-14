import { PrismaClient, DietaryPreference, Language } from '@prisma/client';
import bcrypt from 'bcryptjs';

const prisma = new PrismaClient();

// Utility function to create password hash
async function hashPassword(password: string): Promise<string> {
  const saltRounds = 12;
  return bcrypt.hash(password, saltRounds);
}

// Create development households
async function createHouseholds() {
  console.log('Creating households...');
  
  const households = [
    {
      id: '550e8400-e29b-41d4-a716-446655440001',
      name: 'The Johnson Family',
      settings: {
        defaultMeasurementUnit: 'imperial',
        sharedInventory: true,
        mealPlanningAccess: 'all-members',
        notificationPreferences: {
          expirationAlerts: true,
          mealReminders: true,
          shoppingListUpdates: true,
        },
      },
    },
    {
      id: '550e8400-e29b-41d4-a716-446655440002',
      name: 'Rodriguez Household',
      settings: {
        defaultMeasurementUnit: 'metric',
        sharedInventory: true,
        mealPlanningAccess: 'owner',
        notificationPreferences: {
          expirationAlerts: true,
          mealReminders: false,
          shoppingListUpdates: true,
        },
      },
    },
    {
      id: '550e8400-e29b-41d4-a716-446655440003',
      name: 'Single User - Alex',
      settings: {
        defaultMeasurementUnit: 'metric',
        sharedInventory: false,
        mealPlanningAccess: 'owner',
        notificationPreferences: {
          expirationAlerts: true,
          mealReminders: true,
          shoppingListUpdates: false,
        },
      },
    },
  ];

  for (const household of households) {
    await prisma.household.upsert({
      where: { id: household.id },
      update: {},
      create: household,
    });
  }

  console.log(`Created ${households.length} households`);
  return households;
}

// Create development users
async function createUsers(households: any[]) {
  console.log('Creating users...');
  
  const users = [
    // Johnson Family
    {
      id: '550e8400-e29b-41d4-a716-446655440101',
      email: 'john.johnson@example.com',
      name: 'John Johnson',
      passwordHash: await hashPassword('password123'),
      dietaryPreferences: [DietaryPreference.GLUTEN_FREE],
      allergies: ['nuts', 'shellfish'],
      householdId: households[0].id,
      language: Language.EN,
      timezone: 'America/New_York',
    },
    {
      id: '550e8400-e29b-41d4-a716-446655440102',
      email: 'sarah.johnson@example.com',
      name: 'Sarah Johnson',
      passwordHash: await hashPassword('password123'),
      dietaryPreferences: [DietaryPreference.VEGETARIAN],
      allergies: ['dairy'],
      householdId: households[0].id,
      language: Language.EN,
      timezone: 'America/New_York',
    },
    // Rodriguez Household
    {
      id: '550e8400-e29b-41d4-a716-446655440201',
      email: 'carlos.rodriguez@example.com',
      name: 'Carlos Rodriguez',
      passwordHash: await hashPassword('password123'),
      dietaryPreferences: [],
      allergies: [],
      householdId: households[1].id,
      language: Language.ES,
      timezone: 'America/Mexico_City',
    },
    {
      id: '550e8400-e29b-41d4-a716-446655440202',
      email: 'maria.rodriguez@example.com',
      name: 'Maria Rodriguez',
      passwordHash: await hashPassword('password123'),
      dietaryPreferences: [DietaryPreference.DAIRY_FREE],
      allergies: ['eggs'],
      householdId: households[1].id,
      language: Language.ES,
      timezone: 'America/Mexico_City',
    },
    // Single User
    {
      id: '550e8400-e29b-41d4-a716-446655440301',
      email: 'alex.smith@example.com',
      name: 'Alex Smith',
      passwordHash: await hashPassword('password123'),
      dietaryPreferences: [DietaryPreference.VEGAN, DietaryPreference.GLUTEN_FREE],
      allergies: ['soy'],
      householdId: households[2].id,
      language: Language.EN,
      timezone: 'America/Los_Angeles',
    },
  ];

  for (const user of users) {
    await prisma.user.upsert({
      where: { id: user.id },
      update: {},
      create: user,
    });
  }

  console.log(`Created ${users.length} users`);
  return users;
}

// Create sample sessions
async function createSessions(users: any[]) {
  console.log('Creating sample sessions...');
  
  const sessions = [
    {
      id: '550e8400-e29b-41d4-a716-446655440401',
      userId: users[0].id, // John Johnson
      token: 'session_token_john_' + Date.now(),
      expiresAt: new Date(Date.now() + 30 * 24 * 60 * 60 * 1000), // 30 days
    },
    {
      id: '550e8400-e29b-41d4-a716-446655440402',
      userId: users[2].id, // Carlos Rodriguez
      token: 'session_token_carlos_' + Date.now(),
      expiresAt: new Date(Date.now() + 7 * 24 * 60 * 60 * 1000), // 7 days
    },
  ];

  for (const session of sessions) {
    await prisma.session.upsert({
      where: { id: session.id },
      update: {},
      create: session,
    });
  }

  console.log(`Created ${sessions.length} sessions`);
  return sessions;
}

// Utility functions for creating test data
export async function createTestHousehold(name: string, settings = {}) {
  return prisma.household.create({
    data: {
      name,
      settings: {
        defaultMeasurementUnit: 'metric',
        sharedInventory: true,
        mealPlanningAccess: 'all-members',
        ...settings,
      },
    },
  });
}

export async function createTestUser(
  email: string,
  name: string,
  householdId: string,
  options: {
    password?: string;
    dietaryPreferences?: DietaryPreference[];
    allergies?: string[];
    language?: Language;
    timezone?: string;
  } = {}
) {
  const {
    password = 'testpassword',
    dietaryPreferences = [],
    allergies = [],
    language = Language.EN,
    timezone = 'UTC',
  } = options;

  return prisma.user.create({
    data: {
      email,
      name,
      passwordHash: await hashPassword(password),
      dietaryPreferences,
      allergies,
      householdId,
      language,
      timezone,
    },
  });
}

// Main seeding function
async function main() {
  console.log('🌱 Starting database seeding...');
  
  try {
    // Create households first
    const households = await createHouseholds();
    
    // Create users
    const users = await createUsers(households);
    
    // Create sessions
    const sessions = await createSessions(users);
    
    console.log('✅ Database seeding completed successfully!');
    console.log(`Summary:
    - ${households.length} households created
    - ${users.length} users created
    - ${sessions.length} sessions created`);
    
  } catch (error) {
    console.error('❌ Error seeding database:', error);
    throw error;
  }
}

// Run the seeding function
main()
  .catch((e) => {
    console.error(e);
    process.exit(1);
  })
  .finally(async () => {
    await prisma.$disconnect();
  });
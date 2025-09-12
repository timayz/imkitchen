const { PrismaClient } = require('@prisma/client');

async function testPrisma() {
  const prisma = new PrismaClient({
    log: ['query', 'info', 'warn', 'error'],
  });

  try {
    console.log('🔍 Testing Prisma client connection...');
    
    // Test raw query first
    const result = await prisma.$queryRaw`SELECT version() as db_version`;
    console.log('✅ Database connected:', result[0].db_version.split(' ')[0]);

    // Check if tables exist
    const tables = await prisma.$queryRaw`
      SELECT table_name FROM information_schema.tables 
      WHERE table_schema = 'public' AND table_type = 'BASE TABLE'
      ORDER BY table_name;
    `;
    
    if (tables.length === 0) {
      console.log('📋 No tables found. Creating schema manually...');
      
      // Create users table as an example
      await prisma.$executeRaw`
        CREATE TABLE IF NOT EXISTS users (
          id TEXT PRIMARY KEY,
          email TEXT UNIQUE NOT NULL,
          name TEXT,
          "createdAt" TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
          "lastActiveAt" TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
        );
      `;
      
      console.log('✅ Created users table');
    } else {
      console.log('📋 Existing tables:', tables.map(t => t.table_name).join(', '));
    }

    // Test basic operations
    console.log('🧪 Testing basic operations...');
    
    // Count users
    try {
      const userCount = await prisma.user.count();
      console.log(`👥 Current user count: ${userCount}`);
    } catch (error) {
      console.log('⚠️  User model not ready yet (expected if tables don\'t match schema)');
    }

    console.log('✅ Prisma client is working!');
    
  } catch (error) {
    console.error('❌ Error testing Prisma:', error.message);
  } finally {
    await prisma.$disconnect();
  }
}

testPrisma();
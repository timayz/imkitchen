// Environment variables must be set before any modules import them
process.env = { 
  ...process.env, 
  NODE_ENV: 'development',
  DATABASE_URL: 'postgresql://test:test@localhost:5432/imkitchen_test',
  REDIS_URL: 'redis://localhost:6379',
  NEXTAUTH_SECRET: 'test-secret-for-testing-environment-which-is-at-least-32-chars',
  NEXTAUTH_URL: 'http://localhost:3000',
  NEXT_PUBLIC_APP_URL: 'http://localhost:3000',
  NEXT_PUBLIC_API_URL: 'http://localhost:3000/api',
  LOG_LEVEL: 'error'
};
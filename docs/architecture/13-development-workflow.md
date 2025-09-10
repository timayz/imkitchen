# 13. Development Workflow

## Git Workflow Strategy

### Branch Strategy (GitFlow)
- **main**: Production-ready code
- **develop**: Integration branch for features
- **feature/\***: Individual feature development
- **release/\***: Release preparation
- **hotfix/\***: Critical production fixes

### Commit Convention
```
type(scope): description

feat(mobile): add meal plan generation progress indicator
fix(backend): resolve race condition in recipe caching
docs(architecture): update deployment section
test(mobile): add unit tests for MealPlanGrid component
refactor(backend): optimize database query performance
```

## CI/CD Pipeline

### GitHub Actions Workflow
```yaml
# .github/workflows/ci.yml
name: Continuous Integration

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main, develop]

jobs:
  test:
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:15
        env:
          POSTGRES_PASSWORD: postgres
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5

    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: '18'
          cache: 'npm'
      
      - name: Install dependencies
        run: npm ci
      
      - name: Run linting
        run: npm run lint
        
      - name: Run type checking  
        run: npm run type-check
        
      - name: Run unit tests
        run: npm run test:unit
        
      - name: Run integration tests
        run: npm run test:integration
        env:
          DATABASE_URL: postgresql://postgres:postgres@localhost:5432/test

  build:
    needs: test
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/main'
    
    steps:
      - uses: actions/checkout@v4
      - name: Build Docker images
        run: |
          docker build -t imkitchen-backend -f Dockerfile.backend .
          docker build -t imkitchen-mobile -f Dockerfile.frontend .
      
      - name: Push to registry
        run: |
          echo "${{ secrets.DOCKER_PASSWORD }}" | docker login -u "${{ secrets.DOCKER_USERNAME }}" --password-stdin
          docker push imkitchen-backend:latest
          docker push imkitchen-mobile:latest
```

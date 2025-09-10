# API Specification

## REST API Specification

```yaml
openapi: 3.0.0
info:
  title: imkitchen API
  version: 1.0.0
  description: Intelligent meal planning automation API supporting cross-platform mobile and web applications
servers:
  - url: https://api.imkitchen.app/v1
    description: Production API
  - url: https://staging-api.imkitchen.app/v1
    description: Staging API
  - url: http://localhost:8080/v1
    description: Local development

security:
  - BearerAuth: []

paths:
  # Authentication Endpoints
  /auth/login:
    post:
      tags: [Authentication]
      summary: User login
      requestBody:
        required: true
        content:
          application/json:
            schema:
              type: object
              required: [email, password]
              properties:
                email:
                  type: string
                  format: email
                password:
                  type: string
                  minLength: 8
      responses:
        '200':
          description: Login successful
          content:
            application/json:
              schema:
                type: object
                properties:
                  accessToken:
                    type: string
                  refreshToken:
                    type: string
                  user:
                    $ref: '#/components/schemas/User'

  # Core Meal Planning - "Fill My Week" Automation
  /meal-plans/generate:
    post:
      tags: [Meal Planning]
      summary: Generate automated weekly meal plan
      description: Core "Fill My Week" automation endpoint - must complete in <2 seconds
      requestBody:
        required: false
        content:
          application/json:
            schema:
              type: object
              properties:
                weekStartDate:
                  type: string
                  format: date
                  description: Monday of target week, defaults to current week
                preferences:
                  type: object
                  properties:
                    maxPrepTimePerMeal:
                      type: integer
                      description: Maximum prep time in minutes
                    preferredComplexity:
                      type: array
                      items:
                        type: string
                        enum: [simple, moderate, complex]
      responses:
        '200':
          description: Meal plan generated successfully
          content:
            application/json:
              schema:
                type: object
                properties:
                  mealPlan:
                    $ref: '#/components/schemas/MealPlan'
                  shoppingList:
                    $ref: '#/components/schemas/ShoppingList'
                  generationTime:
                    type: number
                    description: Generation time in milliseconds

  # Recipe Management
  /recipes:
    get:
      tags: [Recipes]
      summary: Get user's recipe collection
      parameters:
        - name: search
          in: query
          schema:
            type: string
        - name: mealType
          in: query
          schema:
            type: string
            enum: [breakfast, lunch, dinner, snack]
        - name: complexity
          in: query
          schema:
            type: string
            enum: [simple, moderate, complex]
      responses:
        '200':
          description: Recipe collection
          content:
            application/json:
              schema:
                type: object
                properties:
                  recipes:
                    type: array
                    items:
                      $ref: '#/components/schemas/Recipe'
                  total:
                    type: integer

    post:
      tags: [Recipes]
      summary: Create new recipe
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/RecipeCreate'
      responses:
        '201':
          description: Recipe created
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Recipe'

  # Community Features
  /community/recipes:
    get:
      tags: [Community]
      summary: Browse public community recipes
      parameters:
        - name: sortBy
          in: query
          schema:
            type: string
            enum: [rating, recent, popular]
            default: rating
        - name: minRating
          in: query
          schema:
            type: number
            minimum: 1
            maximum: 5
      responses:
        '200':
          description: Community recipes
          content:
            application/json:
              schema:
                type: array
                items:
                  allOf:
                    - $ref: '#/components/schemas/Recipe'
                    - type: object
                      properties:
                        averageRating:
                          type: number
                        ratingCount:
                          type: integer

components:
  securitySchemes:
    BearerAuth:
      type: http
      scheme: bearer
      bearerFormat: JWT

  schemas:
    User:
      type: object
      properties:
        id:
          type: string
          format: uuid
        email:
          type: string
          format: email
        displayName:
          type: string
        dietaryRestrictions:
          type: array
          items:
            type: string
        cookingSkillLevel:
          type: string
          enum: [beginner, intermediate, advanced]
        preferredMealComplexity:
          type: string
          enum: [simple, moderate, complex]

    Recipe:
      type: object
      properties:
        id:
          type: string
          format: uuid
        title:
          type: string
        ingredients:
          type: array
          items:
            type: object
            properties:
              name:
                type: string
              amount:
                type: number
              unit:
                type: string
              category:
                type: string
                enum: [produce, dairy, pantry, protein, other]
        prepTime:
          type: integer
        cookTime:
          type: integer
        complexity:
          type: string
          enum: [simple, moderate, complex]
        mealType:
          type: string
          enum: [breakfast, lunch, dinner, snack]

    MealPlan:
      type: object
      properties:
        id:
          type: string
          format: uuid
        weekStartDate:
          type: string
          format: date
        generationType:
          type: string
          enum: [automated, manual, mixed]
        entries:
          type: array
          items:
            $ref: '#/components/schemas/MealPlanEntry'

    MealPlanEntry:
      type: object
      properties:
        id:
          type: string
          format: uuid
        recipeId:
          type: string
          format: uuid
        recipe:
          $ref: '#/components/schemas/Recipe'
        date:
          type: string
          format: date
        mealType:
          type: string
          enum: [breakfast, lunch, dinner]
        isManualOverride:
          type: boolean

    ShoppingList:
      type: object
      properties:
        categories:
          type: object
          properties:
            produce:
              type: array
              items:
                type: object
                properties:
                  name:
                    type: string
                  totalAmount:
                    type: number
                  unit:
                    type: string
            dairy:
              type: array
            pantry:
              type: array
            protein:
              type: array

    RecipeCreate:
      type: object
      required: [title, ingredients, prepTime, cookTime, complexity, mealType]
      properties:
        title:
          type: string
        ingredients:
          type: array
        prepTime:
          type: integer
        cookTime:
          type: integer
        complexity:
          type: string
          enum: [simple, moderate, complex]
        mealType:
          type: string
          enum: [breakfast, lunch, dinner, snack]
```

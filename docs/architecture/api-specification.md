# API Specification

Based on the REST API choice from Tech Stack, here's the OpenAPI 3.0 specification for imkitchen's backend services:

## REST API Specification

```yaml
openapi: 3.0.0
info:
  title: imkitchen API
  version: 1.0.0
  description: Automated meal planning and recipe management API
servers:
  - url: https://api.imkitchen.app
    description: Production API server
  - url: https://staging-api.imkitchen.app
    description: Staging API server

paths:
  # Authentication
  /auth/login:
    post:
      summary: User login
      requestBody:
        required: true
        content:
          application/json:
            schema:
              type: object
              properties:
                email:
                  type: string
                  format: email
                password:
                  type: string
              required: [email, password]
      responses:
        '200':
          description: Login successful
          content:
            application/json:
              schema:
                type: object
                properties:
                  token:
                    type: string
                  user:
                    $ref: '#/components/schemas/User'
        '401':
          $ref: '#/components/responses/Unauthorized'

  /auth/register:
    post:
      summary: User registration
      requestBody:
        required: true
        content:
          application/json:
            schema:
              type: object
              properties:
                email:
                  type: string
                  format: email
                password:
                  type: string
                  minLength: 8
                displayName:
                  type: string
              required: [email, password, displayName]
      responses:
        '201':
          description: Registration successful
          content:
            application/json:
              schema:
                type: object
                properties:
                  token:
                    type: string
                  user:
                    $ref: '#/components/schemas/User'
        '400':
          $ref: '#/components/responses/BadRequest'

  # Recipes
  /recipes:
    get:
      summary: Get recipes with filtering
      security:
        - bearerAuth: []
      parameters:
        - name: category
          in: query
          schema:
            $ref: '#/components/schemas/RecipeCategory'
        - name: difficulty
          in: query
          schema:
            $ref: '#/components/schemas/DifficultyLevel'
        - name: max_time
          in: query
          schema:
            type: integer
        - name: is_public
          in: query
          schema:
            type: boolean
        - name: limit
          in: query
          schema:
            type: integer
            default: 20
        - name: offset
          in: query
          schema:
            type: integer
            default: 0
      responses:
        '200':
          description: Recipes retrieved successfully
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
      summary: Create new recipe
      security:
        - bearerAuth: []
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/CreateRecipeRequest'
      responses:
        '201':
          description: Recipe created successfully
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Recipe'

  /recipes/{id}:
    get:
      summary: Get recipe by ID
      security:
        - bearerAuth: []
      parameters:
        - name: id
          in: path
          required: true
          schema:
            type: string
            format: uuid
      responses:
        '200':
          description: Recipe retrieved successfully
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Recipe'
        '404':
          $ref: '#/components/responses/NotFound'

  # Meal Planning - Core "Fill My Week" functionality
  /meal-plans/fill-my-week:
    post:
      summary: Generate automated weekly meal plan
      security:
        - bearerAuth: []
      requestBody:
        required: true
        content:
          application/json:
            schema:
              type: object
              properties:
                weekStartDate:
                  type: string
                  format: date
                preferences:
                  type: object
                  properties:
                    maxComplexity:
                      $ref: '#/components/schemas/DifficultyLevel'
                    excludeRecipes:
                      type: array
                      items:
                        type: string
                        format: uuid
              required: [weekStartDate]
      responses:
        '201':
          description: Meal plan generated successfully
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/MealPlan'
        '400':
          $ref: '#/components/responses/BadRequest'

  /meal-plans:
    get:
      summary: Get user's meal plans
      security:
        - bearerAuth: []
      parameters:
        - name: limit
          in: query
          schema:
            type: integer
            default: 10
      responses:
        '200':
          description: Meal plans retrieved successfully
          content:
            application/json:
              schema:
                type: array
                items:
                  $ref: '#/components/schemas/MealPlan'

  /meal-plans/{id}/slots/{slotId}:
    put:
      summary: Update individual meal slot
      security:
        - bearerAuth: []
      parameters:
        - name: id
          in: path
          required: true
          schema:
            type: string
            format: uuid
        - name: slotId
          in: path
          required: true
          schema:
            type: string
            format: uuid
      requestBody:
        required: true
        content:
          application/json:
            schema:
              type: object
              properties:
                recipeId:
                  type: string
                  format: uuid
                customMealName:
                  type: string
      responses:
        '200':
          description: Meal slot updated successfully
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/MealPlanSlot'

  # Shopping Lists
  /shopping-lists/generate:
    post:
      summary: Generate shopping list from meal plan
      security:
        - bearerAuth: []
      requestBody:
        required: true
        content:
          application/json:
            schema:
              type: object
              properties:
                mealPlanId:
                  type: string
                  format: uuid
              required: [mealPlanId]
      responses:
        '201':
          description: Shopping list generated successfully
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/ShoppingList'

components:
  securitySchemes:
    bearerAuth:
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
        createdAt:
          type: string
          format: date-time
        dietaryRestrictions:
          type: array
          items:
            $ref: '#/components/schemas/DietaryRestriction'

    Recipe:
      type: object
      properties:
        id:
          type: string
          format: uuid
        title:
          type: string
        description:
          type: string
        prepTimeMinutes:
          type: integer
        cookTimeMinutes:
          type: integer
        totalTimeMinutes:
          type: integer
        difficultyLevel:
          $ref: '#/components/schemas/DifficultyLevel'
        servings:
          type: integer
        category:
          $ref: '#/components/schemas/RecipeCategory'
        ingredients:
          type: array
          items:
            $ref: '#/components/schemas/RecipeIngredient'
        instructions:
          type: array
          items:
            $ref: '#/components/schemas/RecipeStep'
        isPublic:
          type: boolean
        imageUrl:
          type: string
          format: uri
        communityRating:
          type: number
          minimum: 1
          maximum: 5

    MealPlan:
      type: object
      properties:
        id:
          type: string
          format: uuid
        userId:
          type: string
          format: uuid
        weekStartDate:
          type: string
          format: date
        createdAt:
          type: string
          format: date-time
        isCurrent:
          type: boolean
        generationMethod:
          $ref: '#/components/schemas/GenerationMethod'
        slots:
          type: array
          items:
            $ref: '#/components/schemas/MealPlanSlot'

    MealPlanSlot:
      type: object
      properties:
        id:
          type: string
          format: uuid
        date:
          type: string
          format: date
        mealType:
          $ref: '#/components/schemas/MealType'
        recipeId:
          type: string
          format: uuid
        customMealName:
          type: string
        isCompleted:
          type: boolean

    ShoppingList:
      type: object
      properties:
        id:
          type: string
          format: uuid
        mealPlanId:
          type: string
          format: uuid
        items:
          type: array
          items:
            $ref: '#/components/schemas/ShoppingListItem'
        createdAt:
          type: string
          format: date-time

    # Enums and smaller objects
    DifficultyLevel:
      type: string
      enum: [easy, medium, hard]

    RecipeCategory:
      type: string
      enum: [breakfast, lunch, dinner, snack, dessert]

    MealType:
      type: string
      enum: [breakfast, lunch, dinner]

    GenerationMethod:
      type: string
      enum: [auto_fill_my_week, manual, partial_auto]

    DietaryRestriction:
      type: string
      enum: [vegetarian, vegan, gluten_free, dairy_free, nut_free, keto, paleo]

  responses:
    BadRequest:
      description: Invalid request parameters
      content:
        application/json:
          schema:
            type: object
            properties:
              error:
                type: string
              details:
                type: object

    Unauthorized:
      description: Authentication required
      content:
        application/json:
          schema:
            type: object
            properties:
              error:
                type: string
                example: "Authentication required"

    NotFound:
      description: Resource not found
      content:
        application/json:
          schema:
            type: object
            properties:
              error:
                type: string
                example: "Resource not found"
```
# API Specification

## REST API Specification

```yaml
openapi: 3.0.0
info:
  title: imkitchen API
  version: 1.0.0
  description: Intelligent meal planning platform API
servers:
  - url: https://api.imkitchen.app
    description: Production server
  - url: http://localhost:3000
    description: Development server

paths:
  /api/auth/register:
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
                name:
                  type: string
                familySize:
                  type: integer
                  minimum: 1
                  maximum: 8
      responses:
        201:
          description: User created successfully
        400:
          description: Validation error

  /api/auth/login:
    post:
      summary: User authentication
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
      responses:
        200:
          description: Authentication successful
        401:
          description: Invalid credentials

  /api/meal-plans:
    post:
      summary: Generate weekly meal plan
      security:
        - sessionAuth: []
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
                      type: string
                      enum: [easy, medium, hard]
                    avoidRecentRecipes:
                      type: boolean
      responses:
        201:
          description: Meal plan generated
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/MealPlan'
        400:
          description: Invalid request

  /api/recipes:
    get:
      summary: Browse community recipes
      parameters:
        - name: category
          in: query
          schema:
            type: string
        - name: difficulty
          in: query
          schema:
            type: string
            enum: [easy, medium, hard]
        - name: maxPrepTime
          in: query
          schema:
            type: integer
      responses:
        200:
          description: Recipe list
          content:
            application/json:
              schema:
                type: array
                items:
                  $ref: '#/components/schemas/Recipe'

    post:
      summary: Create new recipe
      security:
        - sessionAuth: []
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/Recipe'
      responses:
        201:
          description: Recipe created

  /api/recipes/{id}/rate:
    post:
      summary: Rate a recipe
      security:
        - sessionAuth: []
      parameters:
        - name: id
          in: path
          required: true
          schema:
            type: string
      requestBody:
        required: true
        content:
          application/json:
            schema:
              type: object
              properties:
                rating:
                  type: integer
                  minimum: 1
                  maximum: 5
                comment:
                  type: string
      responses:
        200:
          description: Rating submitted

  /api/shopping-lists/{mealPlanId}:
    get:
      summary: Get shopping list for meal plan
      security:
        - sessionAuth: []
      parameters:
        - name: mealPlanId
          in: path
          required: true
          schema:
            type: string
      responses:
        200:
          description: Shopping list
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/ShoppingList'

    post:
      summary: Generate shopping list
      security:
        - sessionAuth: []
      parameters:
        - name: mealPlanId
          in: path
          required: true
          schema:
            type: string
      responses:
        201:
          description: Shopping list generated

components:
  securitySchemes:
    sessionAuth:
      type: apiKey
      in: cookie
      name: session_id
  schemas:
    User:
      type: object
      properties:
        id:
          type: string
        email:
          type: string
        name:
          type: string
        familySize:
          type: integer
        dietaryRestrictions:
          type: array
          items:
            type: string
    Recipe:
      type: object
      properties:
        id:
          type: string
        title:
          type: string
        description:
          type: string
        prepTime:
          type: integer
        cookTime:
          type: integer
        difficulty:
          type: string
          enum: [easy, medium, hard]
        ingredients:
          type: array
          items:
            $ref: '#/components/schemas/Ingredient'
    Ingredient:
      type: object
      properties:
        name:
          type: string
        quantity:
          type: number
        unit:
          type: string
    MealPlan:
      type: object
      properties:
        id:
          type: string
        userId:
          type: string
        weekStartDate:
          type: string
        meals:
          type: array
          items:
            $ref: '#/components/schemas/MealPlanEntry'
    MealPlanEntry:
      type: object
      properties:
        dayOfWeek:
          type: integer
        mealType:
          type: string
          enum: [breakfast, lunch, dinner]
        recipeId:
          type: string
    ShoppingList:
      type: object
      properties:
        id:
          type: string
        mealPlanId:
          type: string
        items:
          type: array
          items:
            $ref: '#/components/schemas/ShoppingItem'
    ShoppingItem:
      type: object
      properties:
        name:
          type: string
        quantity:
          type: number
        unit:
          type: string
        category:
          type: string
        purchased:
          type: boolean
```

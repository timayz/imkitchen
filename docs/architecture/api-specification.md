# API Specification

## REST API Specification

```yaml
openapi: 3.0.0
info:
  title: imkitchen API
  version: 1.0.0
  description: Kitchen management platform API for inventory tracking, meal planning, and cooking guidance
servers:
  - url: https://api.imkitchen.com/v1
    description: Production API server
  - url: http://localhost:3000/api
    description: Local development server

paths:
  /auth/login:
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
              required: [email, password]
      responses:
        '200':
          description: Successful authentication
          content:
            application/json:
              schema:
                type: object
                properties:
                  user:
                    $ref: '#/components/schemas/User'
                  token:
                    type: string

  /inventory:
    get:
      summary: Get household inventory
      parameters:
        - name: location
          in: query
          schema:
            type: string
            enum: [pantry, refrigerator, freezer]
        - name: category
          in: query
          schema:
            type: string
      responses:
        '200':
          description: Inventory items list
          content:
            application/json:
              schema:
                type: array
                items:
                  $ref: '#/components/schemas/InventoryItem'
    
    post:
      summary: Add inventory item
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/InventoryItemCreate'
      responses:
        '201':
          description: Item created successfully
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/InventoryItem'

  /inventory/{itemId}:
    put:
      summary: Update inventory item
      parameters:
        - name: itemId
          in: path
          required: true
          schema:
            type: string
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/InventoryItemUpdate'
      responses:
        '200':
          description: Item updated successfully

    delete:
      summary: Remove inventory item
      parameters:
        - name: itemId
          in: path
          required: true
          schema:
            type: string
      responses:
        '204':
          description: Item deleted successfully

  /recipes:
    get:
      summary: Search recipes
      parameters:
        - name: q
          in: query
          description: Search query
          schema:
            type: string
        - name: ingredients
          in: query
          description: Available ingredients
          schema:
            type: array
            items:
              type: string
        - name: cuisine
          in: query
          schema:
            type: string
        - name: maxCookingTime
          in: query
          schema:
            type: integer
      responses:
        '200':
          description: Recipe search results
          content:
            application/json:
              schema:
                type: object
                properties:
                  recipes:
                    type: array
                    items:
                      $ref: '#/components/schemas/Recipe'
                  pagination:
                    $ref: '#/components/schemas/Pagination'

  /recipes/{recipeId}:
    get:
      summary: Get recipe details
      parameters:
        - name: recipeId
          in: path
          required: true
          schema:
            type: string
      responses:
        '200':
          description: Recipe details
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Recipe'

  /meal-plans:
    get:
      summary: Get meal plans
      parameters:
        - name: startDate
          in: query
          schema:
            type: string
            format: date
        - name: endDate
          in: query
          schema:
            type: string
            format: date
      responses:
        '200':
          description: Meal plans list
          content:
            application/json:
              schema:
                type: array
                items:
                  $ref: '#/components/schemas/MealPlan'

    post:
      summary: Create meal plan
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/MealPlanCreate'
      responses:
        '201':
          description: Meal plan created
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/MealPlan'

  /shopping-lists:
    get:
      summary: Get shopping lists
      responses:
        '200':
          description: Shopping lists
          content:
            application/json:
              schema:
                type: array
                items:
                  $ref: '#/components/schemas/ShoppingList'

    post:
      summary: Generate shopping list from meal plan
      requestBody:
        required: true
        content:
          application/json:
            schema:
              type: object
              properties:
                mealPlanId:
                  type: string
                name:
                  type: string
              required: [mealPlanId]
      responses:
        '201':
          description: Shopping list generated
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/ShoppingList'

  /voice/commands:
    post:
      summary: Process voice command
      requestBody:
        required: true
        content:
          application/json:
            schema:
              type: object
              properties:
                command:
                  type: string
                context:
                  type: object
              required: [command]
      responses:
        '200':
          description: Command processed
          content:
            application/json:
              schema:
                type: object
                properties:
                  action:
                    type: string
                  response:
                    type: string
                  data:
                    type: object

components:
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
        dietaryPreferences:
          type: array
          items:
            type: string
        language:
          type: string
    
    InventoryItem:
      type: object
      properties:
        id:
          type: string
        name:
          type: string
        quantity:
          type: number
        unit:
          type: string
        category:
          type: string
        location:
          type: string
        expirationDate:
          type: string
          format: date

    Recipe:
      type: object
      properties:
        id:
          type: string
        title:
          type: string
        description:
          type: string
        cookingTime:
          type: integer
        difficulty:
          type: string
        servings:
          type: integer

  securitySchemes:
    BearerAuth:
      type: http
      scheme: bearer
      bearerFormat: JWT

security:
  - BearerAuth: []
```

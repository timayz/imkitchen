# Import Recipe CLI Command

The `import-recipe` command allows you to import recipes from JSON files into the imkitchen database.

## Usage

```bash
cargo run -- import-recipe --file <PATH_TO_JSON> --email <USER_EMAIL>
```

### Arguments

- `--file`: Path to the JSON file containing recipe data (required)
- `--email`: Email address of the user who will own the recipe (required)

## JSON Format

The JSON file must follow this structure:

```json
{
  "title": "Recipe Name",
  "recipe_type": "main_course",
  "ingredients": [
    {
      "name": "ingredient name",
      "amount": "quantity",
      "unit": "measurement unit or null"
    }
  ],
  "instructions": [
    {
      "step_number": 1,
      "instruction": "Step description"
    }
  ],
  "prep_time_min": 10,
  "cook_time_min": 20,
  "advance_prep_hours": null,
  "serving_size": 4
}
```

### Field Descriptions

- **title** (required, string): Recipe name (3-200 characters)
- **recipe_type** (required, string): Recipe type - must be one of: "appetizer", "main_course", or "dessert"
- **ingredients** (required, array): List of ingredients
  - **name** (required, string): Ingredient name
  - **amount** (required, string): Quantity
  - **unit** (optional, string|null): Measurement unit
- **instructions** (required, array): Step-by-step instructions
  - **step_number** (required, integer): Step order
  - **instruction** (required, string): Step description
- **prep_time_min** (optional, integer|null): Preparation time in minutes
- **cook_time_min** (optional, integer|null): Cooking time in minutes
- **advance_prep_hours** (optional, integer|null): Hours of advance prep needed
- **serving_size** (optional, integer|null): Number of servings

## Example

An example recipe file is provided: `example-recipe.json`

```bash
cargo run -- import-recipe --file example-recipe.json --email user@example.com
```

### Output

On success, you'll see:
```
✅ Recipe imported successfully!
   Recipe ID: uuid-here
   User: user@example.com (user-id)
```

## Error Handling

The command will fail with an error message if:
- The JSON file doesn't exist or can't be read
- The JSON format is invalid
- The user email doesn't exist in the database
- Recipe validation fails (missing required fields, title too short/long, etc.)
- Database connection fails

## Notes

- The user must already exist in the database before importing recipes
- The recipe will be automatically tagged with cuisine type and dietary tags based on ingredients
- Complexity level is calculated automatically based on ingredient count and instruction steps
- The recipe is created using the evento event sourcing system, so all changes are tracked in the event store

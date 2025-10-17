# Recipe Fixtures

This directory contains sample recipe JSON files that can be imported using the `import-recipe` CLI command.

All recipes are sourced from [Tatie Maryse](https://www.tatiemaryse.com), a popular Caribbean cooking blog featuring authentic Martinican and Guadeloupean cuisine.

## Available Recipes

### 1. Haricots Rouges (Caribbean Red Beans)
**File:** `haricots-rouges.json`
**Cuisine:** Caribbean
**Servings:** 6 | **Prep:** 15 min | **Cook:** 45 min | **Advance Prep:** 12 hours

Traditional Caribbean red beans using pressure cooker with aromatic herbs (cive, thyme, bay leaves, parsley) and quatre épices.

### 2. Nouilles au Porc Grillé MADIVIAL
**File:** `nouilles-porc-grille.json`
**Cuisine:** Caribbean-Asian Fusion
**Servings:** 3 | **Prep:** 25 min | **Cook:** 30 min | **Advance Prep:** 1 hour

Grilled pork with egg noodles, vegetables, and herbed butter featuring Caribbean and Asian flavors.

### 3. Nuggets de Poulet, Sauce Curry-Miel
**File:** `nuggets-poulet.json`
**Cuisine:** Caribbean
**Servings:** 4 | **Prep:** 35 min | **Cook:** 40 min | **Advance Prep:** 1 hour

Homemade chicken nuggets with honey-curry sauce and baked potato wedges.

### 4. Curry de Dinde aux Légumes-Pays
**File:** `curry-dinde-legumes.json`
**Cuisine:** Caribbean
**Servings:** 3 | **Prep:** 30 min | **Cook:** 45 min | **Advance Prep:** 40 min

Turkey curry with traditional Caribbean vegetables (chayote, pumpkin, cherry tomatoes) in creamy sauce.

### 5. Croustillants de Poulet MADIVIAL
**File:** `croustillants-poulet.json`
**Cuisine:** Caribbean
**Servings:** 4 | **Prep:** 35 min | **Cook:** 20 min

Crispy chicken and vegetable wraps in brick pastry with banana and bell peppers.

### 6. Salade de Pâtes au Giraumon
**File:** `salade-pates-giraumon.json`
**Cuisine:** Caribbean
**Servings:** 2-3 | **Prep:** 15 min | **Cook:** 12 min

Warm pasta salad with giraumon squash, zucchini, and sun-dried tomatoes.

### 7. Escalope de Dinde Panée aux Cacahuètes
**File:** `escalope-dinde-cacahuetes.json`
**Cuisine:** Caribbean
**Servings:** 4 | **Prep:** 30 min | **Cook:** 15 min

Peanut-crusted turkey escalopes with herb cheese sauce.

### 8. Tomates Farcies au Porc
**File:** `tomates-farcies-porc.json`
**Cuisine:** Caribbean
**Servings:** 4 | **Prep:** 25 min | **Cook:** 25 min

Pork and mushroom stuffed tomatoes baked in tomato sauce.

### 9. Pains Briochés Fourrés aux Herbes
**File:** `pains-brioches-herbes.json`
**Cuisine:** Caribbean
**Servings:** 17 rolls | **Prep:** 50 min | **Cook:** 30 min | **Advance Prep:** 2 hours

Herb-stuffed brioche rolls with caramelized onions and béchamel.

### 10. Flan au Chocolat
**File:** `flan-chocolat.json`
**Cuisine:** Caribbean
**Servings:** 4 | **Prep:** 15 min | **Cook:** 25 min

Creamy chocolate flan with cinnamon and lime zest, baked in a water bath.

## Quick Import Commands

Import all recipes for a user:
```bash
# Replace user@example.com with actual email
for file in fixtures/*.json; do
  cargo run -- import-recipe --file "$file" --email user@example.com
done
```

Import a specific recipe:
```bash
cargo run -- import-recipe \
  --file fixtures/haricots-rouges.json \
  --email user@example.com
```

## JSON Structure

All recipe files follow this structure:

```json
{
  "title": "Recipe Name",
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

## Adding New Recipes

To add a new recipe fixture:

1. Create a new JSON file following the structure above
2. Ensure all required fields are present:
   - `title` (string)
   - `ingredients` (array with at least 1 item)
   - `instructions` (array with at least 1 item)
3. Validate the JSON:
   ```bash
   python3 -c "import json; json.load(open('fixtures/your-recipe.json')); print('✓ Valid')"
   ```
4. Test import:
   ```bash
   cargo run -- import-recipe --file fixtures/your-recipe.json --email test@example.com
   ```
5. Update this README with recipe details

## Notes

- All times are in minutes except `advance_prep_hours` which is in hours
- `unit` can be null for ingredients without specific measurements
- Recipe metadata (cuisine, dietary tags, complexity) is automatically inferred during import
- Recipes are stored using evento event sourcing, preserving full change history

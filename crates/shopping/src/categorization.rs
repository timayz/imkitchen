/// Category for grocery store organization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Category {
    Produce,
    Dairy,
    Meat,
    Pantry,
    Frozen,
    Bakery,
    Other,
}

impl Category {
    pub fn as_str(&self) -> &str {
        match self {
            Category::Produce => "Produce",
            Category::Dairy => "Dairy",
            Category::Meat => "Meat",
            Category::Pantry => "Pantry",
            Category::Frozen => "Frozen",
            Category::Bakery => "Bakery",
            Category::Other => "Other",
        }
    }
}

/// Categorization Service
///
/// Stateless domain service that maps ingredients to grocery store categories
/// using predefined mappings.
///
/// Categories: Produce, Dairy, Meat, Pantry, Frozen, Bakery, Other
///
/// Extensible for future AI-based categorization.
pub struct CategorizationService;

impl CategorizationService {
    /// Categorize an ingredient by name
    ///
    /// Returns the category for the ingredient based on a predefined mapping.
    /// If the ingredient is not found in the mapping, returns Category::Other.
    pub fn categorize(ingredient_name: &str) -> Category {
        let normalized = ingredient_name.trim().to_lowercase();

        // Produce
        if Self::is_produce(&normalized) {
            return Category::Produce;
        }

        // Dairy
        if Self::is_dairy(&normalized) {
            return Category::Dairy;
        }

        // Meat
        if Self::is_meat(&normalized) {
            return Category::Meat;
        }

        // Pantry (dry goods, spices, canned goods)
        if Self::is_pantry(&normalized) {
            return Category::Pantry;
        }

        // Frozen
        if Self::is_frozen(&normalized) {
            return Category::Frozen;
        }

        // Bakery
        if Self::is_bakery(&normalized) {
            return Category::Bakery;
        }

        // Default to Other
        Category::Other
    }

    fn is_produce(name: &str) -> bool {
        matches!(
            name,
            // Vegetables
            "tomato" | "tomatoes"
                | "onion" | "onions"
                | "garlic"
                | "lettuce"
                | "carrot" | "carrots"
                | "celery"
                | "bell pepper" | "bell peppers"
                | "cucumber" | "cucumbers"
                | "zucchini"
                | "broccoli"
                | "cauliflower"
                | "spinach"
                | "kale"
                | "cabbage"
                | "potato" | "potatoes"
                | "sweet potato" | "sweet potatoes"
                | "mushroom" | "mushrooms"
                | "green beans"
                | "peas"
                | "corn"
                | "avocado" | "avocados"
                | "eggplant"
                | "squash"
                | "jalapeño" | "jalapeno"
                | "ginger"
                | "cilantro"
                | "parsley"
                | "basil"
                | "mint"
                | "thyme"
                | "rosemary"
                // Fruits
                | "apple" | "apples"
                | "banana" | "bananas"
                | "orange" | "oranges"
                | "lemon" | "lemons"
                | "lime" | "limes"
                | "strawberry" | "strawberries"
                | "blueberry" | "blueberries"
                | "raspberry" | "raspberries"
                | "grape" | "grapes"
                | "mango" | "mangoes"
                | "pineapple"
                | "watermelon"
        )
    }

    fn is_dairy(name: &str) -> bool {
        matches!(
            name,
            "milk"
                | "cream"
                | "heavy cream"
                | "whipping cream"
                | "sour cream"
                | "butter"
                | "cheese"
                | "cheddar cheese"
                | "mozzarella cheese"
                | "parmesan cheese"
                | "feta cheese"
                | "goat cheese"
                | "cream cheese"
                | "yogurt"
                | "greek yogurt"
                | "cottage cheese"
                | "ricotta cheese"
                | "eggs"
        )
    }

    fn is_meat(name: &str) -> bool {
        matches!(
            name,
            // Poultry
            "chicken"
                | "chicken breast" | "chicken breasts"
                | "chicken thigh" | "chicken thighs"
                | "turkey"
                | "duck"
                // Beef
                | "beef"
                | "ground beef"
                | "steak"
                | "brisket"
                | "roast"
                // Pork
                | "pork"
                | "bacon"
                | "ham"
                | "sausage"
                | "pork chop" | "pork chops"
                // Seafood
                | "fish"
                | "salmon"
                | "tuna"
                | "cod"
                | "tilapia"
                | "shrimp"
                | "prawns"
                | "lobster"
                | "crab"
                | "scallops"
                // Other
                | "lamb"
                | "veal"
        )
    }

    fn is_pantry(name: &str) -> bool {
        matches!(
            name,
            // Grains & Pasta
            "flour"
                | "all-purpose flour"
                | "bread flour"
                | "rice"
                | "white rice"
                | "brown rice"
                | "pasta"
                | "spaghetti"
                | "penne"
                | "oats"
                | "quinoa"
                | "couscous"
                // Baking
                | "sugar"
                | "brown sugar"
                | "powdered sugar"
                | "baking powder"
                | "baking soda"
                | "yeast"
                | "vanilla extract"
                | "cocoa powder"
                | "chocolate chips"
                // Oils & Condiments
                | "olive oil"
                | "vegetable oil"
                | "coconut oil"
                | "vinegar"
                | "balsamic vinegar"
                | "soy sauce"
                | "worcestershire sauce"
                | "ketchup"
                | "mustard"
                | "mayonnaise"
                | "hot sauce"
                // Spices
                | "salt"
                | "pepper"
                | "black pepper"
                | "paprika"
                | "cumin"
                | "coriander"
                | "turmeric"
                | "cinnamon"
                | "nutmeg"
                | "oregano"
                | "chili powder"
                | "cayenne pepper"
                | "garlic powder"
                | "onion powder"
                // Canned/Jarred
                | "tomato sauce"
                | "tomato paste"
                | "canned tomatoes"
                | "chicken broth"
                | "beef broth"
                | "vegetable broth"
                | "beans"
                | "black beans"
                | "kidney beans"
                | "chickpeas"
                | "peanut butter"
                | "jam"
                | "honey"
                | "maple syrup"
                // Nuts & Seeds
                | "almonds"
                | "walnuts"
                | "pecans"
                | "cashews"
                | "peanuts"
                | "sunflower seeds"
                | "chia seeds"
        )
    }

    fn is_frozen(name: &str) -> bool {
        matches!(
            name,
            "frozen vegetables"
                | "frozen peas"
                | "frozen corn"
                | "frozen broccoli"
                | "frozen berries"
                | "frozen strawberries"
                | "frozen blueberries"
                | "ice cream"
                | "frozen pizza"
                | "frozen french fries"
        )
    }

    fn is_bakery(name: &str) -> bool {
        matches!(
            name,
            "bread"
                | "baguette"
                | "ciabatta"
                | "sourdough"
                | "whole wheat bread"
                | "tortillas"
                | "pita bread"
                | "bagels"
                | "croissant"
                | "croissants"
                | "buns"
                | "hamburger buns"
                | "hot dog buns"
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_categorize_produce() {
        assert_eq!(
            CategorizationService::categorize("tomato"),
            Category::Produce
        );
        assert_eq!(
            CategorizationService::categorize("onion"),
            Category::Produce
        );
        assert_eq!(
            CategorizationService::categorize("lettuce"),
            Category::Produce
        );
        assert_eq!(
            CategorizationService::categorize("carrot"),
            Category::Produce
        );
        assert_eq!(
            CategorizationService::categorize("apple"),
            Category::Produce
        );
        assert_eq!(
            CategorizationService::categorize("banana"),
            Category::Produce
        );
    }

    #[test]
    fn test_categorize_dairy() {
        assert_eq!(CategorizationService::categorize("milk"), Category::Dairy);
        assert_eq!(CategorizationService::categorize("cheese"), Category::Dairy);
        assert_eq!(CategorizationService::categorize("yogurt"), Category::Dairy);
        assert_eq!(CategorizationService::categorize("butter"), Category::Dairy);
        assert_eq!(CategorizationService::categorize("cream"), Category::Dairy);
        assert_eq!(CategorizationService::categorize("eggs"), Category::Dairy);
    }

    #[test]
    fn test_categorize_meat() {
        assert_eq!(CategorizationService::categorize("chicken"), Category::Meat);
        assert_eq!(CategorizationService::categorize("beef"), Category::Meat);
        assert_eq!(CategorizationService::categorize("pork"), Category::Meat);
        assert_eq!(CategorizationService::categorize("fish"), Category::Meat);
        assert_eq!(CategorizationService::categorize("salmon"), Category::Meat);
        assert_eq!(CategorizationService::categorize("shrimp"), Category::Meat);
    }

    #[test]
    fn test_categorize_pantry() {
        assert_eq!(CategorizationService::categorize("flour"), Category::Pantry);
        assert_eq!(CategorizationService::categorize("sugar"), Category::Pantry);
        assert_eq!(CategorizationService::categorize("pasta"), Category::Pantry);
        assert_eq!(CategorizationService::categorize("rice"), Category::Pantry);
        assert_eq!(
            CategorizationService::categorize("olive oil"),
            Category::Pantry
        );
        assert_eq!(CategorizationService::categorize("salt"), Category::Pantry);
        assert_eq!(
            CategorizationService::categorize("pepper"),
            Category::Pantry
        );
        assert_eq!(CategorizationService::categorize("cumin"), Category::Pantry);
    }

    #[test]
    fn test_categorize_frozen() {
        assert_eq!(
            CategorizationService::categorize("frozen vegetables"),
            Category::Frozen
        );
        assert_eq!(
            CategorizationService::categorize("ice cream"),
            Category::Frozen
        );
        assert_eq!(
            CategorizationService::categorize("frozen berries"),
            Category::Frozen
        );
    }

    #[test]
    fn test_categorize_bakery() {
        assert_eq!(CategorizationService::categorize("bread"), Category::Bakery);
        assert_eq!(
            CategorizationService::categorize("baguette"),
            Category::Bakery
        );
        assert_eq!(
            CategorizationService::categorize("tortillas"),
            Category::Bakery
        );
        assert_eq!(
            CategorizationService::categorize("bagels"),
            Category::Bakery
        );
    }

    #[test]
    fn test_categorize_unknown() {
        assert_eq!(
            CategorizationService::categorize("unknown_ingredient"),
            Category::Other
        );
        assert_eq!(CategorizationService::categorize("xyz"), Category::Other);
    }

    #[test]
    fn test_categorize_case_insensitive() {
        assert_eq!(
            CategorizationService::categorize("TOMATO"),
            Category::Produce
        );
        assert_eq!(CategorizationService::categorize("Milk"), Category::Dairy);
        assert_eq!(CategorizationService::categorize("ChIcKeN"), Category::Meat);
    }

    #[test]
    fn test_categorize_with_whitespace() {
        assert_eq!(
            CategorizationService::categorize("  tomato  "),
            Category::Produce
        );
        assert_eq!(
            CategorizationService::categorize("  milk  "),
            Category::Dairy
        );
    }

    #[test]
    fn test_category_as_str() {
        assert_eq!(Category::Produce.as_str(), "Produce");
        assert_eq!(Category::Dairy.as_str(), "Dairy");
        assert_eq!(Category::Meat.as_str(), "Meat");
        assert_eq!(Category::Pantry.as_str(), "Pantry");
        assert_eq!(Category::Frozen.as_str(), "Frozen");
        assert_eq!(Category::Bakery.as_str(), "Bakery");
        assert_eq!(Category::Other.as_str(), "Other");
    }

    // Test 50+ common ingredients as required by the story
    #[test]
    fn test_50_common_ingredients() {
        let ingredients = vec![
            // Produce (20)
            ("tomato", Category::Produce),
            ("onion", Category::Produce),
            ("garlic", Category::Produce),
            ("lettuce", Category::Produce),
            ("carrot", Category::Produce),
            ("celery", Category::Produce),
            ("bell pepper", Category::Produce),
            ("cucumber", Category::Produce),
            ("broccoli", Category::Produce),
            ("spinach", Category::Produce),
            ("potato", Category::Produce),
            ("mushroom", Category::Produce),
            ("apple", Category::Produce),
            ("banana", Category::Produce),
            ("lemon", Category::Produce),
            ("lime", Category::Produce),
            ("strawberry", Category::Produce),
            ("avocado", Category::Produce),
            ("ginger", Category::Produce),
            ("cilantro", Category::Produce),
            // Dairy (10)
            ("milk", Category::Dairy),
            ("butter", Category::Dairy),
            ("cheese", Category::Dairy),
            ("cream", Category::Dairy),
            ("yogurt", Category::Dairy),
            ("eggs", Category::Dairy),
            ("cheddar cheese", Category::Dairy),
            ("mozzarella cheese", Category::Dairy),
            ("parmesan cheese", Category::Dairy),
            ("sour cream", Category::Dairy),
            // Meat (10)
            ("chicken", Category::Meat),
            ("beef", Category::Meat),
            ("pork", Category::Meat),
            ("bacon", Category::Meat),
            ("salmon", Category::Meat),
            ("shrimp", Category::Meat),
            ("ground beef", Category::Meat),
            ("chicken breast", Category::Meat),
            ("sausage", Category::Meat),
            ("ham", Category::Meat),
            // Pantry (15)
            ("flour", Category::Pantry),
            ("sugar", Category::Pantry),
            ("rice", Category::Pantry),
            ("pasta", Category::Pantry),
            ("olive oil", Category::Pantry),
            ("salt", Category::Pantry),
            ("pepper", Category::Pantry),
            ("soy sauce", Category::Pantry),
            ("vinegar", Category::Pantry),
            ("baking powder", Category::Pantry),
            ("canned tomatoes", Category::Pantry),
            ("chicken broth", Category::Pantry),
            ("cumin", Category::Pantry),
            ("paprika", Category::Pantry),
            ("garlic powder", Category::Pantry),
            // Bakery (3)
            ("bread", Category::Bakery),
            ("tortillas", Category::Bakery),
            ("bagels", Category::Bakery),
            // Frozen (2)
            ("frozen peas", Category::Frozen),
            ("ice cream", Category::Frozen),
        ];

        assert_eq!(ingredients.len(), 60); // 60 > 50 ✓

        for (ingredient, expected_category) in ingredients {
            assert_eq!(
                CategorizationService::categorize(ingredient),
                expected_category,
                "Ingredient '{}' should be categorized as {:?}",
                ingredient,
                expected_category
            );
        }
    }
}

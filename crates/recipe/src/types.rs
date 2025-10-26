//! Recipe domain types for meal planning enhancements
//!
//! This module contains enum types introduced in Epic 6: Enhanced Meal Planning System
//! to support accompaniment pairing, cuisine tracking, and dietary filtering.

use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

/// Accompaniment category for recipe pairing
///
/// Defines the category of side dishes that can accompany main courses.
/// Main courses specify which accompaniment categories they accept via `preferred_accompaniments`.
/// Accompaniment recipes specify their category via `accompaniment_category`.
///
/// # Example
/// ```
/// use recipe::AccompanimentCategory;
///
/// let tikka_masala_preferences = vec![
///     AccompanimentCategory::Rice,
///     AccompanimentCategory::Pasta,
/// ];
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Encode, Decode)]
#[serde(rename_all = "snake_case")]
pub enum AccompanimentCategory {
    /// Pasta-based sides (spaghetti, penne, etc.)
    Pasta,
    /// Rice-based sides (basmati, jasmine, wild rice, etc.)
    Rice,
    /// Fried potato sides (french fries, wedges, etc.)
    Fries,
    /// Salad sides (green salad, coleslaw, etc.)
    Salad,
    /// Bread sides (garlic bread, naan, baguette, etc.)
    Bread,
    /// Vegetable sides (roasted vegetables, steamed greens, etc.)
    Vegetable,
    /// Other uncategorized sides
    Other,
}

/// Cuisine type for recipe categorization and variety tracking
///
/// Supports 13 predefined cuisines plus a Custom variant for user-defined cuisines.
/// The meal planning algorithm uses cuisine to ensure variety across weekly meal plans
/// (avoid repeating the same cuisine too frequently).
///
/// # Example
/// ```
/// use recipe::Cuisine;
///
/// let indian = Cuisine::Indian;
/// let fusion = Cuisine::Custom("Fusion".to_string());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Encode, Decode)]
#[serde(rename_all = "snake_case")]
pub enum Cuisine {
    /// Italian cuisine (pasta, pizza, risotto, etc.)
    Italian,
    /// Indian cuisine (curry, tikka masala, biryani, etc.)
    Indian,
    /// Mexican cuisine (tacos, enchiladas, quesadillas, etc.)
    Mexican,
    /// Chinese cuisine (stir-fry, dumplings, fried rice, etc.)
    Chinese,
    /// Japanese cuisine (sushi, ramen, teriyaki, etc.)
    Japanese,
    /// French cuisine (coq au vin, ratatouille, soufflé, etc.)
    French,
    /// American cuisine (burgers, BBQ, mac and cheese, etc.)
    American,
    /// Mediterranean cuisine (hummus, falafel, Greek salad, etc.)
    Mediterranean,
    /// Thai cuisine (pad thai, green curry, tom yum, etc.)
    Thai,
    /// Korean cuisine (bibimbap, kimchi, bulgogi, etc.)
    Korean,
    /// Vietnamese cuisine (pho, banh mi, spring rolls, etc.)
    Vietnamese,
    /// Greek cuisine (moussaka, souvlaki, tzatziki, etc.)
    Greek,
    /// Spanish cuisine (paella, tapas, gazpacho, etc.)
    Spanish,
    /// User-defined custom cuisine (e.g., "Fusion", "Regional Brazilian", "Home Cooking")
    Custom(String),
}

/// Dietary tag for recipe filtering and user constraint matching
///
/// Describes dietary properties of recipes (vegetarian, vegan, gluten-free, etc.).
/// The meal planning algorithm filters recipes where all user dietary restrictions
/// match the recipe's dietary tags.
///
/// # Example
/// ```
/// use recipe::DietaryTag;
///
/// let recipe_tags = vec![
///     DietaryTag::Vegetarian,
///     DietaryTag::GlutenFree,
/// ];
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
#[serde(rename_all = "snake_case")]
pub enum DietaryTag {
    /// Vegetarian diet (no meat or fish)
    Vegetarian,
    /// Vegan diet (no animal products: meat, fish, dairy, eggs, honey)
    Vegan,
    /// Gluten-free diet (no wheat, barley, rye, or gluten-containing grains)
    GlutenFree,
    /// Dairy-free diet (no milk, cheese, butter, or dairy products)
    DairyFree,
    /// Nut-free diet (no tree nuts or peanuts)
    NutFree,
    /// Halal diet (Islamic dietary laws)
    Halal,
    /// Kosher diet (Jewish dietary laws)
    Kosher,
}

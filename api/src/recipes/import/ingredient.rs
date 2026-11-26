//! Reads an ingredient as a page writes it, in one line ("150 g de
//! farine"), into how much, of what unit, of what.
//!
//! Only what is read for sure is taken apart: a line that starts with a
//! number, followed or not by a unit this module knows. Anything else stays
//! whole as the name, which is what the owner sees and may rewrite.

use imkitchen_aggregates::recipe::Ingredient;

/// The bounds a recipe holds an ingredient to (`recipes::Ingredient`).
const NAME_MAX: usize = 200;
const QUANTITY_MIN: f64 = 0.001;
const QUANTITY_MAX: f64 = 1_000_000.0;

/// The units a quantity may be followed by, in lowercase, as English and
/// French pages write them.
const UNITS: &[&str] = &[
    "g",
    "gr",
    "kg",
    "mg",
    "ml",
    "cl",
    "dl",
    "l",
    "gram",
    "grams",
    "gramme",
    "grammes",
    "kilo",
    "kilos",
    "liter",
    "liters",
    "litre",
    "litres",
    "oz",
    "ounce",
    "ounces",
    "lb",
    "lbs",
    "pound",
    "pounds",
    "cup",
    "cups",
    "tsp",
    "tbsp",
    "teaspoon",
    "teaspoons",
    "tablespoon",
    "tablespoons",
    "pinch",
    "pinches",
    "clove",
    "cloves",
    "slice",
    "slices",
    "can",
    "cans",
    "stick",
    "sticks",
    "cuillère",
    "cuillères",
    "cuillère à soupe",
    "cuillères à soupe",
    "cuillère à café",
    "cuillères à café",
    "c. à soupe",
    "c. à café",
    "c. à s.",
    "c. à c.",
    "c. à s",
    "c. à c",
    "c à s",
    "c à c",
    "càs",
    "càc",
    "pincée",
    "pincées",
    "sachet",
    "sachets",
    "gousse",
    "gousses",
    "tranche",
    "tranches",
    "verre",
    "verres",
    "tasse",
    "tasses",
    "boîte",
    "boîtes",
    "brin",
    "brins",
    "feuille",
    "feuilles",
    "botte",
    "bottes",
];

/// What links a unit to what it counts: "150 g de farine", "a cup of
/// flour".
const LINKS: &[&str] = &["de ", "d'", "d’", "of "];

/// The ingredient `line` writes, a text without blanks around it.
pub fn parse(line: &str) -> Ingredient {
    taken_apart(line).unwrap_or_else(|| Ingredient {
        name: clip(line, NAME_MAX),
        quantity: None,
        unit: None,
    })
}

fn taken_apart(line: &str) -> Option<Ingredient> {
    let (quantity, rest) = quantity(line)?;
    if !(QUANTITY_MIN..=QUANTITY_MAX).contains(&quantity) {
        return None;
    }
    let (unit, rest) = match unit(rest.trim_start()) {
        Some((unit, rest)) => (Some(unit), rest),
        None => (None, rest),
    };
    let mut name = rest.trim_start();
    // "150g": a unit holds to its number, a name does not ("2oeufs").
    if name.len() == rest.len() && unit.is_none() && !name.is_empty() {
        return None;
    }
    if unit.is_some() {
        name = LINKS
            .iter()
            .find_map(|link| strip_prefix_ignore_case(name, link))
            .unwrap_or(name)
            .trim_start();
    }
    if name.is_empty() {
        return None;
    }
    Some(Ingredient {
        name: clip(name, NAME_MAX),
        quantity: Some(quantity),
        unit: unit.map(str::to_owned),
    })
}

/// The number `text` starts with and what follows it: "2", "1,5", "1/2",
/// "½", "1 1/2". `None` when it starts with none, or with a span ("2-3",
/// "2 à 3"), which is not one quantity.
fn quantity(text: &str) -> Option<(f64, &str)> {
    let (quantity, rest) = match number(text) {
        Some((whole, rest)) => match rest.strip_prefix('/').and_then(integer) {
            Some((divisor, rest)) if divisor > 0.0 => (whole / divisor, rest),
            Some(_) => return None,
            None => match fraction(rest.trim_start()) {
                Some((part, rest)) => (whole + part, rest),
                None => (whole, rest),
            },
        },
        None => fraction(text)?,
    };
    let after = rest.trim_start();
    let span = ["-", "–", "à ", "to ", "ou ", "or "]
        .iter()
        .filter_map(|link| strip_prefix_ignore_case(after, link))
        .any(|rest| rest.trim_start().starts_with(|c: char| c.is_ascii_digit()));
    (!span).then_some((quantity, rest))
}

/// A whole or decimal number, with a point or a comma.
fn number(text: &str) -> Option<(f64, &str)> {
    let (whole, rest) = digits(text)?;
    let decimals = rest
        .strip_prefix(['.', ','])
        .and_then(digits)
        .map(|(decimals, rest)| (format!("{whole}.{decimals}"), rest));
    match decimals {
        Some((number, rest)) => Some((number.parse().ok()?, rest)),
        None => Some((whole.parse().ok()?, rest)),
    }
}

fn integer(text: &str) -> Option<(f64, &str)> {
    let (digits, rest) = digits(text)?;
    Some((digits.parse().ok()?, rest))
}

/// The digits `text` starts with, when it starts with some.
fn digits(text: &str) -> Option<(&str, &str)> {
    let end = text
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(text.len());
    (end > 0).then(|| text.split_at(end))
}

/// A part of one: "1/2" or "½".
fn fraction(text: &str) -> Option<(f64, &str)> {
    if let Some((dividend, rest)) = integer(text) {
        let (divisor, rest) = integer(rest.strip_prefix('/')?)?;
        return (divisor > 0.0).then_some((dividend / divisor, rest));
    }
    let mut chars = text.chars();
    let part = match chars.next()? {
        '½' => 1.0 / 2.0,
        '⅓' => 1.0 / 3.0,
        '⅔' => 2.0 / 3.0,
        '¼' => 1.0 / 4.0,
        '¾' => 3.0 / 4.0,
        '⅛' => 1.0 / 8.0,
        _ => return None,
    };
    Some((part, chars.as_str()))
}

/// The unit `text` starts with, as it writes it, and what follows: the
/// longest of [`UNITS`] that ends where a word does.
fn unit(text: &str) -> Option<(&str, &str)> {
    UNITS
        .iter()
        .filter_map(|unit| {
            let rest = strip_prefix_ignore_case(text, unit)?;
            // "tbsp." is "tbsp".
            let rest = match unit.ends_with('.') {
                true => rest,
                false => rest.strip_prefix('.').unwrap_or(rest),
            };
            let ends = !rest.starts_with(|c: char| c.is_alphanumeric());
            ends.then(|| (&text[..unit_len(text, rest, unit)], rest))
        })
        .max_by_key(|(unit, _)| unit.len())
}

/// How many bytes of `text` the unit takes, without the point `rest` may
/// have lost after it.
fn unit_len(text: &str, rest: &str, unit: &str) -> usize {
    let taken = text.len() - rest.len();
    match !unit.ends_with('.') && text[..taken].ends_with('.') {
        true => taken - 1,
        false => taken,
    }
}

/// What follows `prefix` at the start of `text`, whatever the case.
fn strip_prefix_ignore_case<'a>(text: &'a str, prefix: &str) -> Option<&'a str> {
    let mut chars = text.chars();
    for expected in prefix.chars() {
        if !chars.next()?.to_lowercase().eq(expected.to_lowercase()) {
            return None;
        }
    }
    Some(chars.as_str())
}

/// `text` held to `max` characters.
pub fn clip(text: &str, max: usize) -> String {
    match text.char_indices().nth(max) {
        Some((end, _)) => text[..end].trim_end().to_owned(),
        None => text.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read(line: &str) -> (Option<f64>, Option<String>, String) {
        let ingredient = parse(line);
        (ingredient.quantity, ingredient.unit, ingredient.name)
    }

    #[test]
    fn a_line_is_taken_apart_when_it_reads_for_sure() {
        for (line, quantity, unit, name) in [
            ("150 g de farine", Some(150.0), Some("g"), "farine"),
            ("150g de farine", Some(150.0), Some("g"), "farine"),
            ("150 G de Farine", Some(150.0), Some("G"), "Farine"),
            ("200 ml de lait", Some(200.0), Some("ml"), "lait"),
            ("1 oeuf", Some(1.0), None, "oeuf"),
            (
                "1 cuillère à soupe de sucre blanc",
                Some(1.0),
                Some("cuillère à soupe"),
                "sucre blanc",
            ),
            (
                "1 cuillère à soupe d'huile ou de beurre fondu",
                Some(1.0),
                Some("cuillère à soupe"),
                "huile ou de beurre fondu",
            ),
            (
                "1 bonne cuillère à café de levure",
                Some(1.0),
                None,
                "bonne cuillère à café de levure",
            ),
            ("2 c. à s. d’huile", Some(2.0), Some("c. à s."), "huile"),
            ("1 c à s de vinaigre", Some(1.0), Some("c à s"), "vinaigre"),
            ("1,5 l d'eau", Some(1.5), Some("l"), "eau"),
            ("2.5 cups of flour", Some(2.5), Some("cups"), "flour"),
            ("1/2 cup sugar", Some(0.5), Some("cup"), "sugar"),
            ("1 1/2 cups sugar", Some(1.5), Some("cups"), "sugar"),
            ("1½ cups sugar", Some(1.5), Some("cups"), "sugar"),
            ("¾ tsp salt", Some(0.75), Some("tsp"), "salt"),
            ("2 Tbsp. butter", Some(2.0), Some("Tbsp"), "butter"),
            ("2 large eggs", Some(2.0), None, "large eggs"),
            // A unit ends where a word does.
            ("2 lemons", Some(2.0), None, "lemons"),
            ("3 gousses d'ail", Some(3.0), Some("gousses"), "ail"),
            // Not for sure: left whole.
            ("sel", None, None, "sel"),
            ("Salt and pepper", None, None, "Salt and pepper"),
            ("2-3 carottes", None, None, "2-3 carottes"),
            ("2 à 3 carottes", None, None, "2 à 3 carottes"),
            ("1 or 2 onions", None, None, "1 or 2 onions"),
            ("1/0 cup", None, None, "1/0 cup"),
            ("0 g de sel", None, None, "0 g de sel"),
            ("4", None, None, "4"),
            ("100 g", None, None, "100 g"),
            ("2oeufs", None, None, "2oeufs"),
            ("7up", None, None, "7up"),
        ] {
            let expected = (quantity, unit.map(str::to_owned), name.to_owned());
            assert_eq!(read(line), expected, "{line}");
        }
    }

    #[test]
    fn a_name_is_held_to_its_bound() {
        let long = "é".repeat(NAME_MAX + 50);
        assert_eq!(parse(&long).name.chars().count(), NAME_MAX);
        let ingredient = parse(&format!("2 kg de {long}"));
        assert_eq!(ingredient.quantity, Some(2.0));
        assert_eq!(ingredient.name.chars().count(), NAME_MAX);
    }
}

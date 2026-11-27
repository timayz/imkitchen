//! Reads the recipe a page describes to machines: a schema.org `Recipe`,
//! written as JSON-LD (`<script type="application/ld+json">`) or as
//! microdata (`itemscope`, `itemprop`).
//!
//! Microdata is first turned into what the JSON-LD of the same recipe
//! would be, so that both are read by the one [`found`].

use scraper::{ElementRef, Html, Selector};
use serde_json::{Map, Value};

use super::html::{self, Budget, following, lines};
use super::{Found, INGREDIENTS_MAX, INSTRUCTIONS_MAX, Step};

/// How deep the items of a microdata item are read.
const ITEMS_DEPTH: usize = 8;
/// The microdata properties that are read, of a recipe and of its steps.
const PROPERTIES: &[&str] = &[
    "name",
    "description",
    "recipeIngredient",
    "ingredients",
    "recipeInstructions",
    "itemListElement",
    "text",
    "prepTime",
    "cookTime",
    "totalTime",
    "timeRequired",
    "performTime",
    "duration",
    "recipeYield",
    "yield",
    "image",
    "url",
    "contentUrl",
];
/// Those of them that hold an address: a picture, or where one is.
const ADDRESSES: &[&str] = &["image", "url", "contentUrl"];

/// The recipe `document` describes to machines, when it names one.
pub fn recipe(document: &Html, budget: &mut Budget) -> Option<Found> {
    let named = |found: &Found| !found.title.is_empty();
    json_ld(document)
        .iter()
        .find_map(|data| Some(found(find_recipe(data)?)).filter(named))
        .or_else(|| Some(found(microdata(document, budget)?.as_object()?)).filter(named))
}

/// What the page says in JSON-LD, block by block. A block that is not JSON
/// is skipped.
fn json_ld(document: &Html) -> Vec<Value> {
    let blocks =
        Selector::parse(r#"script[type="application/ld+json"]"#).expect("a valid selector");
    document
        .select(&blocks)
        .filter_map(|block| serde_json::from_str(&block.text().collect::<String>()).ok())
        .collect()
}

/// The first `Recipe` of `data`: itself, one of a list, or one of its
/// `@graph`.
fn find_recipe(data: &Value) -> Option<&Map<String, Value>> {
    match data {
        Value::Array(items) => items.iter().find_map(find_recipe),
        Value::Object(node) if is_recipe(node.get("@type")) => Some(node),
        Value::Object(node) => ["@graph", "mainEntity"]
            .iter()
            .filter_map(|key| node.get(*key))
            .find_map(find_recipe),
        _ => None,
    }
}

fn is_recipe(kind: Option<&Value>) -> bool {
    match kind {
        Some(Value::String(kind)) => is_recipe_type(kind),
        Some(Value::Array(kinds)) => kinds.iter().any(|kind| is_recipe(Some(kind))),
        _ => false,
    }
}

/// Whether one of the types `kinds` names is `Recipe`, short or as an
/// address (`https://schema.org/Recipe`).
fn is_recipe_type(kinds: &str) -> bool {
    kinds
        .split_whitespace()
        .any(|kind| kind.trim_end_matches('/').rsplit('/').next() == Some("Recipe"))
}

/// The first `Recipe` item of the page's microdata, as its JSON-LD would
/// be.
fn microdata(document: &Html, budget: &mut Budget) -> Option<Value> {
    let items = Selector::parse("[itemscope][itemtype]").expect("a valid selector");
    let recipe = document
        .select(&items)
        .find(|item| item.value().attr("itemtype").is_some_and(is_recipe_type))?;
    microdata_item(recipe, 0, budget)
}

/// The properties of the item `scope` opens that are read
/// ([`PROPERTIES`]): those of its descendants that are not within an item
/// of their own, which is read as a value instead. `None` once the budget
/// is spent.
fn microdata_item(scope: ElementRef, depth: usize, budget: &mut Budget) -> Option<Value> {
    let mut item = Map::new();
    let mut next = scope.first_child();
    while let Some(node) = next {
        budget.walk()?;
        let mut inside = true;
        if let Some(element) = ElementRef::wrap(node) {
            let nested = element.value().attr("itemscope").is_some();
            let names = element.value().attr("itemprop").unwrap_or_default();
            // Under the first name that is read: one value, held once.
            let name = names
                .split_whitespace()
                .find(|name| PROPERTIES.contains(name));
            if let Some(name) = name {
                let value = match nested {
                    true if depth < ITEMS_DEPTH => microdata_item(element, depth + 1, budget)?,
                    true => Value::Null,
                    false => Value::String(property(element, name, budget)?),
                };
                add(&mut item, name, value);
            }
            inside = !nested;
        }
        next = following(node, inside, *scope);
    }
    Some(Value::Object(item))
}

/// What the property `name` holds when it is not an item: what it says to
/// machines when it does (`content`, `datetime`, and where it points for an
/// address), its text otherwise.
fn property(element: ElementRef, name: &str, budget: &mut Budget) -> Option<String> {
    let value = element.value();
    let said = value.attr("content").or(value.attr("datetime"));
    let points = || value.attr("src").or(value.attr("href"));
    match said.or_else(|| points().filter(|_| ADDRESSES.contains(&name))) {
        Some(said) => {
            budget.take(said.len())?;
            Some(said.to_owned())
        }
        None => html::text(element, budget),
    }
}

/// Adds a value of the property `name`: one alone as it is, several as a
/// list.
fn add(item: &mut Map<String, Value>, name: &str, value: Value) {
    match item.get_mut(name) {
        Some(Value::Array(values)) => values.push(value),
        Some(first) => *first = Value::Array(vec![first.take(), value]),
        None => {
            item.insert(name.to_owned(), value);
        }
    }
}

/// The text `value` is, or the first one it lists.
fn text(value: &Value) -> Option<&str> {
    match value {
        Value::String(text) => Some(text),
        Value::Array(values) => values.iter().find_map(text),
        _ => None,
    }
}

/// Every line `value` holds: a text, a list of them, or things that have a
/// name.
fn texts(value: &Value, into: &mut Vec<String>) {
    if into.len() >= INGREDIENTS_MAX {
        return;
    }
    match value {
        Value::String(text) => into.extend(lines(text)),
        Value::Array(values) => values.iter().for_each(|value| texts(value, into)),
        Value::Object(node) => {
            if let Some(Value::String(text)) = node.get("name").or(node.get("text")) {
                into.extend(lines(text));
            }
        }
        _ => {}
    }
}

/// How a step says how long it takes, the most usual first.
const STEP_TIMES: &[&str] = &["timeRequired", "performTime", "duration"];

/// The steps `value` holds, in their order: a text of several lines, a
/// list of texts or of `HowToStep`, themselves in `HowToSection` or not. A
/// `HowToStep` that says how long it takes has that to wait.
fn steps(value: &Value, into: &mut Vec<Step>) {
    if into.len() >= INSTRUCTIONS_MAX {
        return;
    }
    match value {
        Value::String(text) => into.extend(lines(text).into_iter().map(Step::from)),
        Value::Array(values) => values.iter().for_each(|value| steps(value, into)),
        Value::Object(node) => match node.get("itemListElement") {
            Some(section) => steps(section, into),
            None => {
                let step = node.get("text").or(node.get("name")).and_then(text);
                let step = step.map(|text| lines(text).join(" ")).unwrap_or_default();
                if !step.is_empty() {
                    into.push(Step {
                        text: step,
                        timer_minutes: STEP_TIMES
                            .iter()
                            .filter_map(|name| node.get(*name))
                            .find_map(minutes),
                    });
                }
            }
        },
        _ => {}
    }
}

/// Where the picture `value` is: an address, the first of a list, or that
/// of an `ImageObject`.
fn image(value: &Value) -> Option<&str> {
    match value {
        Value::String(url) => Some(url.trim()).filter(|url| !url.is_empty()),
        Value::Array(values) => values.iter().find_map(image),
        Value::Object(node) => ["url", "contentUrl"]
            .iter()
            .filter_map(|key| node.get(*key))
            .find_map(image),
        _ => None,
    }
}

/// The minutes of an ISO 8601 duration (`PT1H30M`), at most a week.
fn minutes(value: &Value) -> Option<u32> {
    let duration = text(value)?.trim().to_ascii_uppercase();
    let mut minutes = 0.0;
    let mut number = String::new();
    let mut time = false;
    for part in duration.strip_prefix('P')?.chars() {
        let unit = match part {
            '0'..='9' | '.' => {
                number.push(part);
                continue;
            }
            ',' => {
                number.push('.');
                continue;
            }
            'T' => {
                time = true;
                continue;
            }
            'W' => 10080.0,
            'D' => 1440.0,
            'H' => 60.0,
            'M' if time => 1.0,
            'S' => 1.0 / 60.0,
            _ => return None,
        };
        minutes += number.parse::<f64>().ok()? * unit;
        number.clear();
    }
    number.is_empty().then_some(minutes.round() as u32)
}

/// How many people `value` says the recipe feeds: a number, or the first
/// one of a text ("4 personnes").
fn servings(value: &Value) -> Option<u32> {
    match value {
        Value::Number(number) => Some(number.as_f64()?.round() as u32),
        Value::String(text) => super::first_number(text),
        Value::Array(values) => values.iter().find_map(servings),
        _ => None,
    }
}

/// What the `Recipe` says, as the page writes it.
fn found(recipe: &Map<String, Value>) -> Found {
    let said = |name: &str| recipe.get(name).and_then(text).map(lines);
    let mut ingredients = Vec::new();
    // `ingredients` is what `recipeIngredient` was called before.
    for name in ["recipeIngredient", "ingredients"] {
        if let Some(value) = recipe.get(name).filter(|_| ingredients.is_empty()) {
            texts(value, &mut ingredients);
        }
    }
    let mut instructions = Vec::new();
    if let Some(value) = recipe.get("recipeInstructions") {
        steps(value, &mut instructions);
    }
    let time = |name: &str| recipe.get(name).and_then(minutes);

    Found {
        title: said("name").unwrap_or_default().join(" "),
        description: said("description").unwrap_or_default().join("\n"),
        ingredients,
        instructions,
        prep_minutes: time("prepTime"),
        cook_minutes: time("cookTime"),
        total_minutes: time("totalTime"),
        servings: ["recipeYield", "yield"]
            .iter()
            .filter_map(|name| recipe.get(*name))
            .find_map(servings),
        image: recipe.get("image").and_then(image).map(str::to_owned),
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use imkitchen_aggregates::recipe::{Ingredient, Instruction};
    use serde_json::json;

    use super::*;
    use crate::recipes::import::*;
    use crate::recipes::model::Content;

    /// The recipe `html` describes to machines, as a recipe of ours.
    fn recipe(html: &str) -> Option<Content> {
        let mut budget = Budget::new();
        let document = html::parse(html, &budget)?;
        super::recipe(&document, &mut budget)?.into_content()
    }

    fn page(data: &Value) -> String {
        format!(r#"<html><head><script type="application/ld+json">{data}</script></head></html>"#)
    }

    fn one(name: &str, quantity: Option<f64>, unit: Option<&str>) -> Ingredient {
        Ingredient {
            name: name.to_owned(),
            quantity,
            unit: unit.map(str::to_owned),
        }
    }

    fn step(text: &str, timer_minutes: Option<u32>) -> Instruction {
        Instruction {
            text: text.to_owned(),
            timer_minutes,
        }
    }

    #[test]
    fn a_recipe_is_read_from_json_ld_however_it_is_laid_out() {
        let pancakes = json!({
            "@type": ["Recipe", "NewsArticle"],
            "name": " Pancakes &amp; co ",
            "description": "<p>Easy.</p><p>Quick.</p>",
            "recipeIngredient": ["150 g de farine", "<b>1</b> oeuf", "sel"],
            "recipeInstructions": [
                { "@type": "HowToSection", "name": "The batter", "itemListElement": [
                    { "@type": "HowToStep", "text": "Mix\n everything." },
                ] },
                { "@type": "HowToStep", "name": "Cook." },
                "Eat.",
            ],
            "prepTime": "PT5M",
            "cookTime": "PT1H30M",
            "recipeYield": ["2", "2 personnes"],
        });
        let expected = Content {
            title: "Pancakes & co".to_owned(),
            description: "Easy.\nQuick.".to_owned(),
            ingredients: vec![
                one("farine", Some(150.0), Some("g")),
                one("oeuf", Some(1.0), None),
                one("sel", None, None),
            ],
            instructions: vec![
                step("Mix everything.", None),
                step("Cook.", None),
                step("Eat.", None),
            ],
            prep_minutes: 5,
            cook_minutes: 90,
            servings: 2,
        };

        for data in [
            pancakes.clone(),
            json!([{ "@type": "WebSite", "name": "A site" }, pancakes]),
            json!({ "@context": "https://schema.org", "@graph": [
                { "@type": "BreadcrumbList", "name": "Recipes" },
                pancakes,
            ] }),
            json!({ "@type": "WebPage", "mainEntity": pancakes }),
        ] {
            assert_eq!(recipe(&page(&data)), Some(expected.clone()), "{data}");
        }

        // After a block that is not JSON and one that is not a recipe.
        let html = format!(
            r#"<script type="application/ld+json">{{ nope</script>
               <script type="application/ld+json">{{ "@type": "Organization", "name": "A site" }}</script>
               {}"#,
            page(&pancakes)
        );
        assert_eq!(recipe(&html), Some(expected));
    }

    #[test]
    fn what_a_page_leaves_out_is_left_out() {
        let bare = json!({ "@type": "https://schema.org/Recipe", "name": "Toast" });
        assert_eq!(
            recipe(&page(&bare)),
            Some(Content {
                title: "Toast".to_owned(),
                servings: SERVINGS,
                ..Content::default()
            })
        );

        // One text for all the steps, the old name of the ingredients, and
        // only how long it all takes.
        let old = json!({
            "@type": "Recipe",
            "name": "Toast",
            "ingredients": "2 slices of bread",
            "recipeInstructions": "<ol><li>Toast the bread.</li><li>Butter it.</li></ol>",
            "prepTime": "PT2M",
            "totalTime": "P0DT0H5M",
            "recipeYield": 1,
        });
        let read = recipe(&page(&old)).unwrap();
        assert_eq!(read.ingredients, [one("bread", Some(2.0), Some("slices"))]);
        assert_eq!(
            read.instructions,
            [step("Toast the bread.", None), step("Butter it.", None)]
        );
        assert_eq!((read.prep_minutes, read.cook_minutes), (2, 3));
        assert_eq!(read.servings, 1);
    }

    #[test]
    fn a_recipe_is_read_from_microdata() {
        let html = r#"
            <div itemscope itemtype="https://schema.org/WebPage"><span itemprop="name">A site</span></div>
            <article itemscope itemtype="http://schema.org/Recipe">
              <h1 itemprop="name">Banana   bread</h1>
              <div itemprop="author" itemscope itemtype="http://schema.org/Person">
                <span itemprop="name">Someone</span>
              </div>
              <p itemprop="description">Moist &amp; easy.</p>
              <meta itemprop="prepTime" content="PT15M">
              <time itemprop="cookTime" datetime="PT1H">1 hour</time>
              <span itemprop="recipeYield">Serves 8</span>
              <ul>
                <li itemprop="recipeIngredient">3 ripe bananas</li>
                <li itemprop="recipeIngredient">250 g <a href="/flour">flour</a></li>
              </ul>
              <ol>
                <li itemprop="recipeInstructions" itemscope itemtype="http://schema.org/HowToStep">
                  <span itemprop="text">Mash the bananas.</span>
                </li>
                <li itemprop="recipeInstructions" itemscope itemtype="http://schema.org/HowToStep">
                  <span itemprop="text">Bake.</span>
                </li>
              </ol>
            </article>"#;
        assert_eq!(
            recipe(html),
            Some(Content {
                title: "Banana bread".to_owned(),
                description: "Moist & easy.".to_owned(),
                ingredients: vec![
                    one("ripe bananas", Some(3.0), None),
                    one("flour", Some(250.0), Some("g")),
                ],
                instructions: vec![step("Mash the bananas.", None), step("Bake.", None)],
                prep_minutes: 15,
                cook_minutes: 60,
                servings: 8,
            })
        );

        // The steps as one element, a line each.
        let html = r#"
            <div itemscope itemtype="http://schema.org/Recipe">
              <span itemprop="name">Toast</span>
              <div itemprop="recipeInstructions"><p>Toast the bread.</p><p>Butter it.</p></div>
            </div>"#;
        let read = recipe(html).unwrap();
        assert_eq!(
            read.instructions,
            [step("Toast the bread.", None), step("Butter it.", None)]
        );
    }

    #[test]
    fn a_step_that_says_how_long_to_wait_has_a_timer() {
        let bread = json!({
            "@type": "Recipe",
            "name": "Bread",
            "recipeInstructions": [
                // What the page says of the step, rather than its text.
                { "@type": "HowToStep", "text": "Knead 10 minutes, let it rise.", "timeRequired": "PT1H30M" },
                { "@type": "HowToStep", "text": "Shape it.", "performTime": "PT0M" },
                { "@type": "HowToSection", "name": "Baking", "itemListElement": [
                    { "@type": "HowToStep", "text": "Bake 35 min at 230°C." },
                    { "@type": "HowToDirection", "text": "Let it cool.", "duration": "P30D" },
                ] },
                "Laisser reposer 1 h 30 avant de trancher.",
                "Eat.",
            ],
        });
        assert_eq!(
            recipe(&page(&bread)).unwrap().instructions,
            [
                step("Knead 10 minutes, let it rise.", Some(90)),
                step("Shape it.", None),
                step("Bake 35 min at 230°C.", Some(35)),
                step("Let it cool.", Some(MINUTES_MAX)),
                step("Laisser reposer 1 h 30 avant de trancher.", Some(90)),
                step("Eat.", None),
            ]
        );

        let html = r#"
            <div itemscope itemtype="http://schema.org/Recipe">
              <span itemprop="name">Bread</span>
              <div itemprop="recipeInstructions" itemscope itemtype="http://schema.org/HowToStep">
                <span itemprop="text">Let it rise.</span>
                <meta itemprop="timeRequired" content="PT45M">
              </div>
              <div itemprop="recipeInstructions" itemscope itemtype="http://schema.org/HowToStep">
                <span itemprop="text">Bake 35 minutes.</span>
              </div>
            </div>"#;
        assert_eq!(
            recipe(html).unwrap().instructions,
            [
                step("Let it rise.", Some(45)),
                step("Bake 35 minutes.", Some(35))
            ]
        );
    }

    /// Where the page `html` says the picture of its recipe is.
    fn picture(html: &str) -> Option<String> {
        let mut budget = Budget::new();
        let document = html::parse(html, &budget)?;
        super::recipe(&document, &mut budget)?.image
    }

    #[test]
    fn the_picture_of_a_recipe_is_read_however_it_is_given() {
        let with =
            |given: Value| page(&json!({ "@type": "Recipe", "name": "Toast", "image": given }));
        for given in [
            json!("https://example.com/toast.jpg"),
            json!([
                " https://example.com/toast.jpg ",
                "https://example.com/small.jpg"
            ]),
            json!({ "@type": "ImageObject", "url": "https://example.com/toast.jpg" }),
            json!([{ "@type": "ImageObject", "contentUrl": "https://example.com/toast.jpg" }]),
        ] {
            let read = picture(&with(given.clone()));
            assert_eq!(
                read.as_deref(),
                Some("https://example.com/toast.jpg"),
                "{given}"
            );
        }
        for given in [
            json!(""),
            json!([]),
            json!({ "@type": "ImageObject" }),
            json!(3),
        ] {
            assert_eq!(picture(&with(given.clone())), None, "{given}");
        }
        assert_eq!(
            picture(&page(&json!({ "@type": "Recipe", "name": "Toast" }))),
            None
        );

        // In microdata: where the element points, not what it says.
        let html = r#"
            <div itemscope itemtype="http://schema.org/Recipe">
              <a itemprop="name" href="/toast">Toast</a>
              <img itemprop="image" src="/img/toast.jpg" alt="A toast">
            </div>"#;
        assert_eq!(picture(html).as_deref(), Some("/img/toast.jpg"));
        assert_eq!(recipe(html).unwrap().title, "Toast");
        let html = r#"
            <div itemscope itemtype="http://schema.org/Recipe">
              <span itemprop="name">Toast</span>
              <div itemprop="image" itemscope itemtype="http://schema.org/ImageObject">
                <meta itemprop="url" content="https://example.com/toast.jpg">
              </div>
            </div>"#;
        assert_eq!(
            picture(html).as_deref(),
            Some("https://example.com/toast.jpg")
        );
    }

    #[test]
    fn a_page_without_a_recipe_has_none() {
        for html in [
            "",
            "<html><body><h1>Pancakes</h1><p>Mix everything.</p></body></html>",
            r#"<script type="application/ld+json">{ "@type": "Article", "name": "News" }</script>"#,
            // A recipe nothing names.
            r#"<script type="application/ld+json">{ "@type": "Recipe", "name": " " }</script>"#,
            r#"<div itemscope itemtype="http://schema.org/Person"><b itemprop="name">Me</b></div>"#,
        ] {
            assert_eq!(recipe(html), None, "{html}");
        }
    }

    #[test]
    fn what_is_read_is_held_to_the_bounds_of_a_recipe() {
        let lines: Vec<String> = (0..150).map(|n| format!("{n} g de {n}")).collect();
        let long = json!({
            "@type": "Recipe",
            "name": "t".repeat(300),
            "description": "d".repeat(3000),
            "recipeIngredient": lines,
            "recipeInstructions": std::iter::repeat_n("s".repeat(5000), 150).collect::<Vec<_>>(),
            "prepTime": "P30D",
            "cookTime": "soon",
            "recipeYield": "5000",
        });
        let read = recipe(&page(&long)).unwrap();
        assert_eq!(read.title.len(), TITLE_MAX);
        assert_eq!(read.description.len(), DESCRIPTION_MAX);
        assert_eq!(read.ingredients.len(), INGREDIENTS_MAX);
        assert_eq!(read.instructions.len(), INSTRUCTIONS_MAX);
        assert_eq!(read.instructions[0].text.len(), INSTRUCTION_MAX);
        assert_eq!((read.prep_minutes, read.cook_minutes), (MINUTES_MAX, 0));
        assert_eq!(read.servings, SERVINGS);
    }

    #[test]
    fn a_page_nested_deep_is_read_all_the_same() {
        let deep = 10_000;
        let html = format!(
            r#"<div itemscope itemtype="http://schema.org/Recipe">{}<b itemprop="name">Deep</b>{}</div>"#,
            "<div>".repeat(deep),
            "</div>".repeat(deep)
        );
        assert_eq!(recipe(&html).unwrap().title, "Deep");
    }

    #[test]
    fn a_page_written_to_cost_its_reader_has_no_recipe() {
        // Nested for the parser to take minutes.
        let deep = 400_000;
        let html = format!(
            r#"<div itemscope itemtype="http://schema.org/Recipe">{}<b itemprop="name">Deep</b>"#,
            "<div>".repeat(deep),
        );
        let started = Instant::now();
        assert_eq!(recipe(&html), None);
        assert!(started.elapsed() < Duration::from_secs(20));

        // The same text under a thousand names.
        let nested = 5_000;
        let html = format!(
            r#"<div itemscope itemtype="http://schema.org/Recipe"><b itemprop="name">Wide</b>{}{}{}</div>"#,
            r#"<div itemprop="description">"#.repeat(nested),
            "d".repeat(1024),
            "</div>".repeat(nested)
        );
        assert_eq!(recipe(&html), None);

        // A list without end stops at what a recipe holds.
        let many = json!({
            "@type": "Recipe",
            "name": "Many",
            "recipeIngredient": vec!["<b>1</b> &amp; more"; 200_000],
        });
        assert_eq!(
            recipe(&page(&many)).unwrap().ingredients.len(),
            INGREDIENTS_MAX
        );
    }
}

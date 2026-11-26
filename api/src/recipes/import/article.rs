//! Reads the recipe a page shows to people, when it describes none to
//! machines: a blog, most often, that writes its recipes as an article.
//!
//! This is a guess, made from how such pages are laid out: the ingredients
//! are the lines under a heading that says so ("Ingrédients"), the steps
//! those under one that does ("Préparation", "Instructions"), and a page
//! without both holds no recipe. What the guess takes for a line of the
//! recipe may not be one; its owner rewrites it.

use scraper::{ElementRef, Html, Node, Selector};

use super::html::{self, BREAKS, Budget, NodeRef, following, split};
use super::{Found, first_number};

/// What the heading of the ingredients starts with, in lowercase.
const INGREDIENTS: &[&str] = &["ingrédient", "ingredient"];
/// What the heading of the steps starts with, in lowercase.
const STEPS: &[&str] = &[
    "préparation",
    "preparation",
    "réalisation",
    "étapes",
    "etapes",
    "instructions",
    "directions",
    "method",
    "steps",
    "how to make",
];
/// What a line that says how many people the recipe feeds starts with, in
/// lowercase.
const SERVES: &[&str] = &["pour ", "serves ", "servings", "for ", "makes ", "yield"];
/// The elements whose text is not the page's: what it runs, and what is
/// there to click.
const SKIPPED: &[&str] = &[
    "script", "style", "noscript", "template", "nav", "form", "button", "select", "aside",
    "figure", "iframe", "svg",
];
/// The elements that hold others rather than text: a link right under one
/// is there to go somewhere, not to be read.
const CONTAINERS: &[&str] = &[
    "div", "section", "article", "main", "body", "header", "footer",
];
/// How many headings that say the right thing are tried for the lines under
/// them.
const HEADINGS_TRIED: usize = 10;
/// How many elements a heading may be wrapped in, alone, above what it
/// heads.
const WRAPPERS: usize = 3;
/// How many elements are tried for a time, and how long their text may be.
const TIMES_TRIED: usize = 50;
const TIME_MAX: usize = 40;

/// The recipe `document` shows, when it shows its ingredients and its
/// steps. `None` too once the budget is spent.
pub fn recipe(document: &Html, budget: &mut Budget) -> Option<Found> {
    let mut ingredients = section(document, INGREDIENTS, budget)?;
    let instructions = section(document, STEPS, budget)?;
    if ingredients.is_empty() || instructions.is_empty() {
        return None;
    }
    let feeds = ingredients.iter().find_map(|line| servings(line));
    // What says how many it feeds, and what names a part of the recipe
    // ("Pour la sauce :"), is not an ingredient.
    ingredients.retain(|line| servings(line).is_none() && !line.ends_with(':'));

    Some(Found {
        title: title(document, budget)?,
        description: meta(document, &["description", "og:description"]).unwrap_or_default(),
        ingredients,
        instructions,
        prep_minutes: time(document, r#"[class*="prep"]"#, budget)?,
        cook_minutes: time(document, r#"[class*="cuisson"], [class*="cook"]"#, budget)?,
        total_minutes: None,
        servings: feeds,
        // The picture the page hands the sites that link to it.
        image: meta(document, &["og:image", "twitter:image"]),
    })
}

fn select(selector: &str) -> Selector {
    Selector::parse(selector).expect("a valid selector")
}

/// What the page is called: its first heading, or what it tells the sites
/// that link to it.
fn title(document: &Html, budget: &mut Budget) -> Option<String> {
    let heading = match document.select(&select("h1")).next() {
        Some(heading) => split(&html::text(heading, budget)?).join(" "),
        None => String::new(),
    };
    Some(match heading.is_empty() {
        true => meta(document, &["og:title"]).unwrap_or_default(),
        false => heading,
    })
}

/// What the page says of itself under the first of `names` it has.
fn meta(document: &Html, names: &[&str]) -> Option<String> {
    names.iter().find_map(|name| {
        let named = select(&format!(r#"meta[name="{name}"], meta[property="{name}"]"#));
        let said = document.select(&named).next()?.value().attr("content")?;
        Some(split(said).join(" ")).filter(|said| !said.is_empty())
    })
}

/// The lines under the first heading that starts with one of `words` and
/// has some. None: no such heading.
fn section(document: &Html, words: &[&str], budget: &mut Budget) -> Option<Vec<String>> {
    let mut tried = 0;
    for heading in document.select(&select("h1, h2, h3, h4, h5, h6")) {
        let says = html::text(heading, budget)?.trim_start().to_lowercase();
        if !words.iter().any(|word| says.starts_with(word)) || aside(heading, budget)? {
            continue;
        }
        let lines = below(heading, budget)?;
        tried += 1;
        if !lines.is_empty() || tried == HEADINGS_TRIED {
            return Some(lines);
        }
    }
    Some(Vec::new())
}

/// Whether `element` is within what is not the page's text ([`SKIPPED`]):
/// a menu, say.
fn aside(element: ElementRef, budget: &mut Budget) -> Option<bool> {
    for ancestor in element.ancestors() {
        budget.walk()?;
        if matches!(ancestor.value(), Node::Element(above) if SKIPPED.contains(&above.name())) {
            return Some(true);
        }
    }
    Some(false)
}

/// The lines `heading` heads: what follows it where it stands or, when
/// nothing does, what follows the elements it is wrapped in.
fn below(heading: ElementRef, budget: &mut Budget) -> Option<Vec<String>> {
    let level = level(heading.value().name()).unwrap_or(6);
    let mut start = *heading;
    for _ in 0..WRAPPERS {
        let Some(scope) = start.parent() else {
            break;
        };
        let lines = split(&after(start, scope, level, budget)?);
        if !lines.is_empty() {
            return Some(lines);
        }
        start = scope;
    }
    Some(Vec::new())
}

/// The level of the heading called `name`: 1 for `h1`.
fn level(name: &str) -> Option<u32> {
    match name.as_bytes() {
        [b'h', digit @ b'1'..=b'6'] => Some(u32::from(digit - b'0')),
        _ => None,
    }
}

/// The text that follows `start` within `scope`, gone to the line where the
/// page does, up to the next heading of `level` or above. A heading below
/// it names a part of the section, and is left out.
fn after(start: NodeRef, scope: NodeRef, level: u32, budget: &mut Budget) -> Option<String> {
    let mut text = String::new();
    let mut next = following(start, false, scope);
    while let Some(node) = next {
        budget.walk()?;
        let mut inside = true;
        match node.value() {
            Node::Text(part) => {
                budget.take(part.len())?;
                text.push_str(part);
            }
            Node::Element(element) => {
                let name = element.name();
                match self::level(name) {
                    Some(other) if other <= level => break,
                    Some(_) => inside = false,
                    None => inside = !SKIPPED.contains(&name) && !goes_elsewhere(node),
                }
                if !inside || BREAKS.contains(&name) {
                    text.push('\n');
                }
            }
            _ => {}
        }
        next = following(node, inside, scope);
    }
    Some(text)
}

/// Whether `node` is a link that stands on its own among elements ("See
/// the conversion table"), rather than within a text.
fn goes_elsewhere(node: NodeRef) -> bool {
    let named = |node: NodeRef, names: &[&str]| matches!(node.value(), Node::Element(element) if names.contains(&element.name()));
    let says = |node: Option<NodeRef>| {
        node.is_some_and(|node| matches!(node.value(), Node::Text(text) if !text.trim().is_empty()))
    };
    named(node, &["a"])
        && node
            .parent()
            .is_some_and(|parent| named(parent, CONTAINERS))
        && !says(node.prev_sibling())
        && !says(node.next_sibling())
}

/// How many people `line` says the recipe feeds, when that is what it
/// says: "Pour 4 personnes :".
fn servings(line: &str) -> Option<u32> {
    let says = line.to_lowercase();
    (line.chars().count() <= TIME_MAX && SERVES.iter().any(|word| says.starts_with(word)))
        .then(|| first_number(line))
        .flatten()
}

/// The minutes the first element `selector` finds that says a time, and
/// nothing else, says. None: no such element.
fn time(document: &Html, selector: &str, budget: &mut Budget) -> Option<Option<u32>> {
    for element in document.select(&select(selector)).take(TIMES_TRIED) {
        let mut text = String::new();
        for node in element.descendants() {
            budget.walk()?;
            if let Node::Text(part) = node.value() {
                text.push_str(part.trim());
                text.push(' ');
            }
            if text.len() > TIME_MAX {
                break;
            }
        }
        if let Some(minutes) = spoken_minutes(&text).filter(|_| text.len() <= TIME_MAX) {
            return Some(Some(minutes));
        }
    }
    Some(None)
}

/// The minutes of a time as people write it: "25 min", "1 h 30", "2
/// heures", "1 hour 15 minutes".
fn spoken_minutes(text: &str) -> Option<u32> {
    let says = text.trim().to_lowercase();
    let (first, rest) = number(&says)?;
    let (unit, rest) = word(rest);
    match unit {
        "h" | "hr" | "hrs" | "heure" | "heures" | "hour" | "hours" => {
            let minutes = match number(rest) {
                Some((minutes, rest)) => {
                    let (unit, rest) = word(rest);
                    (is_minutes(unit) || unit.is_empty() && rest.is_empty()).then_some(minutes)?
                }
                None => rest.trim().is_empty().then_some(0)?,
            };
            first.checked_mul(60)?.checked_add(minutes)
        }
        unit if is_minutes(unit) && rest.trim().is_empty() => Some(first),
        _ => None,
    }
}

fn is_minutes(unit: &str) -> bool {
    matches!(unit, "min" | "mins" | "mn" | "minute" | "minutes")
}

/// The whole number `text` starts with, blanks aside, and what follows.
fn number(text: &str) -> Option<(u32, &str)> {
    let text = text.trim_start();
    let end = text
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(text.len());
    Some((text[..end].parse().ok()?, &text[end..]))
}

/// The word `text` starts with, blanks and its point aside, and what
/// follows.
fn word(text: &str) -> (&str, &str) {
    let text = text.trim_start();
    let end = text
        .find(|c: char| !c.is_alphabetic())
        .unwrap_or(text.len());
    let (word, rest) = text.split_at(end);
    (word, rest.strip_prefix('.').unwrap_or(rest))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn recipe(html: &str) -> Option<Found> {
        let mut budget = Budget::new();
        let document = html::parse(html, &budget)?;
        super::recipe(&document, &mut budget)
    }

    #[test]
    fn a_recipe_is_read_from_an_article_that_shows_one() {
        let html = r#"
            <html><head>
              <title>Banana bread | A blog</title>
              <meta name="description" content="Moist &amp; easy.">
              <meta property="og:image" content="/img/banana-bread.jpg">
            </head><body>
            <nav><h2>Ingredients of the month</h2><a href="/">Home</a></nav>
            <article>
              <h1>Banana <em>bread</em></h1>
              <p>My grandmother made it every Sunday.</p>
              <span class="prep-time">Prep: 15 min</span>
              <span class="recipe-prep-time">15 min</span>
              <span class="cook-time"><b>1</b> h 05</span>
              <h2>Ingredients</h2>
              <p>Serves 8</p>
              <ul>
                <li>3 ripe bananas</li>
                <li>250 g <a href="/flour">flour</a></li>
              </ul>
              <h3>For the icing</h3>
              <ul><li>100 g icing sugar</li></ul>
              <a href="/convert">Conversion table</a>
              <figure><img src="a.jpg"><figcaption>The batter</figcaption></figure>
              <h2>Instructions:</h2>
              <ol>
                <li>Mash the bananas &amp; mix.</li>
                <li>Bake.<br>Let it cool.</li>
              </ol>
              <script>track("Instructions")</script>
              <h2>Notes</h2>
              <p>Keeps three days.</p>
            </article>
            </body></html>"#;
        assert_eq!(
            recipe(html),
            Some(Found {
                title: "Banana bread".to_owned(),
                description: "Moist & easy.".to_owned(),
                ingredients: vec![
                    "3 ripe bananas".to_owned(),
                    "250 g flour".to_owned(),
                    "100 g icing sugar".to_owned(),
                ],
                instructions: vec![
                    "Mash the bananas & mix.".to_owned(),
                    "Bake.".to_owned(),
                    "Let it cool.".to_owned(),
                ],
                prep_minutes: Some(15),
                cook_minutes: Some(65),
                total_minutes: None,
                servings: Some(8),
                image: Some("/img/banana-bread.jpg".to_owned()),
            })
        );
    }

    #[test]
    fn a_heading_wrapped_alone_heads_what_follows_its_wrapper() {
        let html = r#"
            <meta property="og:title" content="Toast">
            <div class="title"><div><h3>Ingrédients pour 2 personnes</h3></div></div>
            <div><p>Pour la base :</p><p>2 tranches de pain</p></div>
            <div class="title"><h3>Préparation</h3></div>
            <div><p>Griller le pain.</p></div>"#;
        let found = recipe(html).unwrap();
        assert_eq!(found.title, "Toast");
        assert_eq!(found.ingredients, ["2 tranches de pain"]);
        assert_eq!(found.instructions, ["Griller le pain."]);
        assert_eq!((found.prep_minutes, found.servings), (None, None));
    }

    #[test]
    fn a_page_that_does_not_show_both_holds_no_recipe() {
        for html in [
            "<h1>Pancakes</h1><p>Mix everything.</p>",
            "<h1>Pancakes</h1><h2>Ingredients</h2><ul><li>Flour</li></ul>",
            "<h1>Pancakes</h1><h2>Method</h2><p>Mix everything.</p>",
            // Headings that head nothing.
            "<h1>Pancakes</h1><h2>Ingredients</h2><h2>Method</h2>",
        ] {
            assert_eq!(recipe(html), None, "{html}");
        }
    }

    #[test]
    fn a_time_is_read_as_people_write_it() {
        for (text, minutes) in [
            ("25 min", Some(25)),
            (" 45 minutes ", Some(45)),
            ("30mn", Some(30)),
            ("5 min.", Some(5)),
            ("1 h", Some(60)),
            ("1h30", Some(90)),
            ("1 h 30 min", Some(90)),
            ("2 heures", Some(120)),
            ("1 hour 15 minutes", Some(75)),
            ("1 hr. 5 mins", Some(65)),
            ("Prep: 15 min", None),
            ("25", None),
            ("25 g", None),
            ("1 h 30 g", None),
            ("min", None),
            ("", None),
        ] {
            assert_eq!(spoken_minutes(text), minutes, "{text}");
        }
    }
}

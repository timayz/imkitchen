//! A page as a tree, and what its readers share to walk it.
//!
//! A page is anyone's, and may be written to cost its reader: nested
//! without end, or saying the same megabyte under a thousand names. So
//! reading one has a [`Budget`] of time and of text, past which the page
//! has no recipe, and nothing here recurses as deep as the page is nested.

use std::ops::Deref;
use std::time::{Duration, Instant};

use html5ever::driver;
use html5ever::tendril::TendrilSink;
use scraper::{ElementRef, Html, HtmlTreeSink, Node};

/// How long reading a page may take. A page of a recipe site takes a
/// hundredth of it.
const READ_TIME: Duration = Duration::from_secs(2);
/// How much text may be taken out of a page's tree in all.
const READ_TEXT: usize = 1024 * 1024;
/// How much of a page the parser is given between two looks at the time.
const CHUNK: usize = 4096;
/// The elements a text goes to the line at.
pub const BREAKS: &[&str] = &["br", "p", "li", "div", "tr", "h1", "h2", "h3", "h4"];

/// A node of a page's tree.
pub type NodeRef<'a> = <ElementRef<'a> as Deref>::Target;

/// What is left to spend on reading a page.
pub struct Budget {
    deadline: Instant,
    /// The bytes of text that may still be taken out of the tree.
    text: usize,
    /// The nodes walked so far, to look at the time once in a while.
    nodes: usize,
}

impl Budget {
    pub fn new() -> Self {
        Self {
            deadline: Instant::now() + READ_TIME,
            text: READ_TEXT,
            nodes: 0,
        }
    }

    fn in_time(&self) -> Option<()> {
        (Instant::now() < self.deadline).then_some(())
    }

    /// Counts a node walked. `None` once the time is spent.
    pub fn walk(&mut self) -> Option<()> {
        self.nodes += 1;
        match self.nodes % 1024 {
            0 => self.in_time(),
            _ => Some(()),
        }
    }

    /// Takes `bytes` of text. `None` when there is not as much left.
    pub fn take(&mut self, bytes: usize) -> Option<()> {
        self.text = self.text.checked_sub(bytes)?;
        Some(())
    }
}

/// The page as a tree, given to the parser a piece at a time: what a page
/// written for it costs the parser shows between two pieces.
pub fn parse(html: &str, budget: &Budget) -> Option<Html> {
    let sink = HtmlTreeSink::new(Html::new_document());
    let mut parser = driver::parse_document(sink, Default::default());
    let mut rest = html;
    while !rest.is_empty() {
        let (chunk, after) = rest.split_at(rest.floor_char_boundary(CHUNK));
        parser.process(chunk.into());
        budget.in_time()?;
        rest = after;
    }
    Some(parser.finish())
}

/// The node after `node` in the order of the page, without leaving
/// `scope`: its first child when `inside`, what follows it otherwise.
/// Walked rather than recursed into, however deep the page is nested.
pub fn following<'a>(node: NodeRef<'a>, inside: bool, scope: NodeRef<'a>) -> Option<NodeRef<'a>> {
    if inside && let Some(child) = node.first_child() {
        return Some(child);
    }
    let mut at = node;
    while at.id() != scope.id() {
        if let Some(sibling) = at.next_sibling() {
            return Some(sibling);
        }
        at = at.parent()?;
    }
    None
}

/// The text of `element`, gone to the line where the page does.
pub fn text(element: ElementRef, budget: &mut Budget) -> Option<String> {
    let mut text = String::new();
    for node in element.descendants() {
        budget.walk()?;
        match node.value() {
            Node::Text(part) => {
                budget.take(part.len())?;
                text.push_str(part);
            }
            Node::Element(inner) if BREAKS.contains(&inner.name()) => text.push('\n'),
            _ => {}
        }
    }
    Some(text)
}

/// The lines of `text` that say something, each without its tags, its
/// entities and its blanks in excess: pages put HTML in their data.
pub fn lines(text: &str) -> Vec<String> {
    let plain = match text.contains(['<', '&']) {
        true => plain(text),
        false => text.to_owned(),
    };
    split(&plain)
}

/// The lines of the plain text `text` that say something, each without its
/// blanks in excess.
pub fn split(text: &str) -> Vec<String> {
    text.lines()
        .map(|line| line.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|line| !line.is_empty())
        .collect()
}

/// `html` as the text it shows, gone to the line where it does. The tags
/// are taken out by hand, so that the parser, which resolves the entities,
/// is given nothing to nest.
fn plain(html: &str) -> String {
    let mut text = String::with_capacity(html.len());
    let mut rest = html;
    while let Some(start) = rest.find('<') {
        text.push_str(&rest[..start]);
        let tag = &rest[start + 1..];
        let name = tag.trim_start_matches('/');
        let end = match name.starts_with(|c: char| c.is_ascii_alphabetic()) {
            true => tag.find('>'),
            false => None,
        };
        match end {
            Some(end) => {
                let name = name.split(|c: char| !c.is_ascii_alphanumeric()).next();
                if name.is_some_and(|name| BREAKS.contains(&name.to_ascii_lowercase().as_str())) {
                    text.push('\n');
                }
                rest = &tag[end + 1..];
            }
            // Not a tag: "less than 5 minutes" written with the sign.
            None => {
                text.push_str("&lt;");
                rest = tag;
            }
        }
    }
    text.push_str(rest);
    match text.contains('&') {
        true => Html::parse_fragment(&text).root_element().text().collect(),
        false => text,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_text_is_read_without_its_tags_and_entities() {
        for (text, expected) in [
            ("Mix.", vec!["Mix."]),
            ("  Mix \t well. \n\n Cook.\r\n", vec!["Mix well.", "Cook."]),
            (
                "<p>Mix <b>well</b>.</p><P class='a'>Cook.</P>",
                vec!["Mix well.", "Cook."],
            ),
            ("Mix.<br/>Cook.<br>Eat.", vec!["Mix.", "Cook.", "Eat."]),
            (
                "Salt &amp; pepper, 5&nbsp;g &#233;",
                vec!["Salt & pepper, 5 g é"],
            ),
            ("Cook < 5 min & eat <3", vec!["Cook < 5 min & eat <3"]),
            ("An unclosed <b tag", vec!["An unclosed <b tag"]),
            ("&lt;b&gt; is bold", vec!["<b> is bold"]),
        ] {
            assert_eq!(lines(text), expected, "{text}");
        }
    }
}

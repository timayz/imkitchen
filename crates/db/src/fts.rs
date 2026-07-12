/// Convert a raw user search string into a safe FTS5 MATCH expression.
///
/// Each whitespace-separated token is wrapped as an FTS5 quoted string (embedded
/// `"` doubled) with a trailing `*` for prefix matching, joined by spaces
/// (implicit AND). Wrapping every token in quotes means FTS5 operators and
/// special characters (`"`, `*`, `(`, `)`, `:`, `-`, `AND`, `OR`, `NOT`, `NEAR`)
/// in user input are treated as literal text rather than query syntax, so no
/// input can cause a MATCH syntax error.
///
/// Returns `None` when the input has no tokens, so callers can skip the filter
/// instead of emitting an empty MATCH (which is itself an FTS5 syntax error).
///
/// Note: the returned string must still be bound as a parameter (e.g. via
/// `sea_query`'s `cust_with_values` / a `?` placeholder), never interpolated
/// into SQL — this function makes the value safe for FTS5, not for SQL.
pub fn to_match_query(search: &str) -> Option<String> {
    let query = search
        .split_whitespace()
        .map(|token| format!("\"{}\"*", token.replace('"', "\"\"")))
        .collect::<Vec<_>>()
        .join(" ");

    (!query.is_empty()).then_some(query)
}

#[cfg(test)]
mod tests {
    use super::to_match_query;

    #[test]
    fn plain_word() {
        assert_eq!(to_match_query("chicken").as_deref(), Some("\"chicken\"*"));
    }

    #[test]
    fn multi_word_is_prefix_and() {
        assert_eq!(
            to_match_query("chicken soup").as_deref(),
            Some("\"chicken\"* \"soup\"*"),
        );
    }

    #[test]
    fn collapses_extra_whitespace() {
        assert_eq!(
            to_match_query("  chicken\t soup \n").as_deref(),
            Some("\"chicken\"* \"soup\"*"),
        );
    }

    #[test]
    fn escapes_embedded_double_quote() {
        // `a"b` -> quoted string with the `"` doubled, then prefix `*`.
        assert_eq!(to_match_query("a\"b").as_deref(), Some("\"a\"\"b\"*"));
    }

    #[test]
    fn fts_operators_are_literal() {
        // Operators/keywords must not be interpreted; each token is quoted.
        assert_eq!(
            to_match_query("a AND b").as_deref(),
            Some("\"a\"* \"AND\"* \"b\"*"),
        );
        assert_eq!(to_match_query("-tomato").as_deref(), Some("\"-tomato\"*"));
        assert_eq!(to_match_query("foo:bar").as_deref(), Some("\"foo:bar\"*"));
        assert_eq!(to_match_query("(").as_deref(), Some("\"(\"*"));
        assert_eq!(to_match_query("soup*").as_deref(), Some("\"soup*\"*"));
    }

    #[test]
    fn sql_injection_attempt_is_literal() {
        // Even if this were interpolated it is now inert; bound as a param it is
        // doubly safe. Verify it stays a single quoted token per word.
        assert_eq!(
            to_match_query("x' OR '1'='1").as_deref(),
            Some("\"x'\"* \"OR\"* \"'1'='1\"*"),
        );
    }

    #[test]
    fn empty_and_whitespace_are_none() {
        assert_eq!(to_match_query(""), None);
        assert_eq!(to_match_query("   \t\n "), None);
    }
}

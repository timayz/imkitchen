use std::collections::HashSet;
use std::sync::LazyLock;

/// Set of ambiguous quantity keywords that cannot be precisely measured
static AMBIGUOUS_QUANTITIES: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    let mut set = HashSet::new();
    set.insert("pinch");
    set.insert("a pinch");
    set.insert("dash");
    set.insert("a dash");
    set.insert("to taste");
    set.insert("taste");
    set.insert("handful");
    set.insert("a handful");
    set.insert("some");
    set.insert("sprinkle");
    set.insert("a sprinkle");
    set
});

/// Check if a quantity string is ambiguous (non-numeric)
///
/// Returns true for quantities like:
/// - "a pinch"
/// - "to taste"
/// - "dash"
/// - "handful"
///
/// Returns false for numeric quantities like:
/// - "2"
/// - "1/2"
/// - "1.5"
///
/// # Arguments
/// * `quantity_str` - The quantity string to check
///
/// # Returns
/// * true if ambiguous, false if numeric
pub fn is_ambiguous_quantity(quantity_str: &str) -> bool {
    let normalized = quantity_str.trim().to_lowercase();

    // Check if it's in the ambiguous set
    if AMBIGUOUS_QUANTITIES.contains(normalized.as_str()) {
        return true;
    }

    // Check if it starts with ambiguous keywords
    for keyword in AMBIGUOUS_QUANTITIES.iter() {
        if normalized.starts_with(keyword) || normalized.ends_with(keyword) {
            return true;
        }
    }

    // Try to parse as number - if it fails, it's likely ambiguous
    // Check for numeric patterns: digits, '/', '.'
    let has_numeric = normalized
        .chars()
        .any(|c| c.is_ascii_digit() || c == '/' || c == '.');

    !has_numeric
}

#[cfg(test)]
mod tests {
    use super::*;

    // AC #8: Ambiguous Quantity Detection Tests

    #[test]
    fn test_detect_ambiguous_pinch() {
        assert!(is_ambiguous_quantity("a pinch"));
        assert!(is_ambiguous_quantity("pinch"));
        assert!(is_ambiguous_quantity("  pinch  "));
    }

    #[test]
    fn test_detect_ambiguous_to_taste() {
        assert!(is_ambiguous_quantity("to taste"));
        assert!(is_ambiguous_quantity("To Taste"));
    }

    #[test]
    fn test_detect_ambiguous_dash() {
        assert!(is_ambiguous_quantity("dash"));
        assert!(is_ambiguous_quantity("a dash"));
    }

    #[test]
    fn test_detect_ambiguous_handful() {
        assert!(is_ambiguous_quantity("handful"));
        assert!(is_ambiguous_quantity("a handful"));
    }

    #[test]
    fn test_non_ambiguous_quantities() {
        assert!(!is_ambiguous_quantity("2"));
        assert!(!is_ambiguous_quantity("1/2"));
        assert!(!is_ambiguous_quantity("0.5"));
        assert!(!is_ambiguous_quantity("2 cups"));
        assert!(!is_ambiguous_quantity("1 1/2"));
    }

    #[test]
    fn test_ambiguous_case_insensitive() {
        assert!(is_ambiguous_quantity("PINCH"));
        assert!(is_ambiguous_quantity("To TASTE"));
        assert!(is_ambiguous_quantity("DASH"));
    }
}

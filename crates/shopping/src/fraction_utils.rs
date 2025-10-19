use fraction::Fraction;

/// Parse a quantity string into a Fraction
///
/// Supports formats:
/// - Whole numbers: "2" → 2/1
/// - Pure fractions: "1/2" → 1/2
/// - Mixed fractions: "1 1/2" → 3/2
/// - Decimals: "0.5" → 1/2
///
/// # Arguments
/// * `quantity_str` - The quantity string to parse
///
/// # Returns
/// * Ok(Fraction) - Parsed fraction
/// * Err(String) - Parse error
pub fn parse_quantity(quantity_str: &str) -> Result<Fraction, String> {
    let trimmed = quantity_str.trim();

    // Handle mixed fractions: "1 1/2"
    if trimmed.contains(' ') && trimmed.contains('/') {
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() != 2 {
            return Err(format!("Invalid mixed fraction format: {}", quantity_str));
        }

        let whole: i64 = parts[0]
            .parse()
            .map_err(|_| format!("Invalid whole number: {}", parts[0]))?;

        if whole < 0 {
            return Err("Negative quantities are not allowed".to_string());
        }

        let frac_parts: Vec<&str> = parts[1].split('/').collect();
        if frac_parts.len() != 2 {
            return Err(format!("Invalid fraction format: {}", parts[1]));
        }

        let numerator: i64 = frac_parts[0]
            .parse()
            .map_err(|_| format!("Invalid numerator: {}", frac_parts[0]))?;
        let denominator: i64 = frac_parts[1]
            .parse()
            .map_err(|_| format!("Invalid denominator: {}", frac_parts[1]))?;

        if numerator < 0 {
            return Err("Negative quantities are not allowed".to_string());
        }

        if denominator == 0 {
            return Err("Denominator cannot be zero".to_string());
        }

        if denominator < 0 {
            return Err("Negative denominator is not allowed".to_string());
        }

        let whole_frac = Fraction::new(whole as u64, 1u64);
        let fractional = Fraction::new(numerator as u64, denominator as u64);
        return Ok(whole_frac + fractional);
    }

    // Handle pure fractions: "1/2"
    if trimmed.contains('/') {
        let parts: Vec<&str> = trimmed.split('/').collect();
        if parts.len() != 2 {
            return Err(format!("Invalid fraction format: {}", quantity_str));
        }

        let numerator: i64 = parts[0]
            .parse()
            .map_err(|_| format!("Invalid numerator: {}", parts[0]))?;
        let denominator: i64 = parts[1]
            .parse()
            .map_err(|_| format!("Invalid denominator: {}", parts[1]))?;

        if numerator < 0 {
            return Err("Negative quantities are not allowed".to_string());
        }

        if denominator == 0 {
            return Err("Denominator cannot be zero".to_string());
        }

        if denominator < 0 {
            return Err("Negative denominator is not allowed".to_string());
        }

        return Ok(Fraction::new(numerator as u64, denominator as u64));
    }

    // Handle decimals and whole numbers
    let value: f64 = trimmed
        .parse()
        .map_err(|_| format!("Invalid number format: {}", quantity_str))?;

    if value < 0.0 {
        return Err("Negative quantities are not allowed".to_string());
    }

    Ok(Fraction::from(value))
}

/// Format a Fraction as a human-readable string
///
/// Formats:
/// - Whole numbers: 2/1 → "2"
/// - Mixed fractions: 3/2 → "1 1/2"
/// - Pure fractions: 1/2 → "1/2"
///
/// # Arguments
/// * `fraction` - The fraction to format
///
/// # Returns
/// * String representation of the fraction
pub fn format_quantity(fraction: Fraction) -> String {
    // Handle whole numbers
    if *fraction.denom().unwrap() == 1u64 {
        return format!("{}", fraction.numer().unwrap());
    }

    // Convert to mixed fraction if >= 1
    let numer = *fraction.numer().unwrap() as i64;
    let denom = *fraction.denom().unwrap() as i64;

    if numer >= denom {
        let whole = numer / denom;
        let remainder = numer % denom;

        if remainder == 0 {
            format!("{}", whole)
        } else {
            format!("{} {}/{}", whole, remainder, denom)
        }
    } else {
        format!("{}/{}", numer, denom)
    }
}

/// Round a Fraction to a practical cooking value
///
/// Rounding rules:
/// - < 1: Round to nearest 1/4, 1/3, 1/2
/// - 1-10: Round to nearest 1/2
/// - > 10: Round to nearest whole number
///
/// # Arguments
/// * `quantity` - The quantity to round
///
/// # Returns
/// * Rounded Fraction
pub fn round_to_practical_value(quantity: Fraction) -> Fraction {
    let value = *quantity.numer().unwrap() as f64 / *quantity.denom().unwrap() as f64;

    if value < 1.0 {
        // Round to nearest 1/4, 1/3, 1/2
        let quarters_count = (value * 4.0).round() as i64;
        let thirds_count = (value * 3.0).round() as i64;
        let halves_count = (value * 2.0).round() as i64;

        let quarters_value = quarters_count as f64 / 4.0;
        let thirds_value = thirds_count as f64 / 3.0;
        let halves_value = halves_count as f64 / 2.0;

        // Choose the closest
        let diff_quarters = (value - quarters_value).abs();
        let diff_thirds = (value - thirds_value).abs();
        let diff_halves = (value - halves_value).abs();

        if diff_quarters <= diff_thirds && diff_quarters <= diff_halves {
            Fraction::new(quarters_count as u64, 4u64)
        } else if diff_thirds <= diff_halves {
            Fraction::new(thirds_count as u64, 3u64)
        } else {
            Fraction::new(halves_count as u64, 2u64)
        }
    } else if value < 10.0 {
        // Round to nearest 0.5
        let halves_count = (value * 2.0).round() as i64;
        Fraction::new(halves_count as u64, 2u64)
    } else {
        // Round to nearest whole number
        let whole = value.round() as i64;
        Fraction::new(whole as u64, 1u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // AC #6: Fractional Quantity Handling Tests

    #[test]
    fn test_parse_fraction_pure_fraction() {
        let result = parse_quantity("1/2").unwrap();
        assert_eq!(*result.numer().unwrap(), 1u64);
        assert_eq!(*result.denom().unwrap(), 2u64);
    }

    #[test]
    fn test_parse_fraction_mixed_fraction() {
        let result = parse_quantity("1 1/2").unwrap();
        // 1 1/2 = 3/2
        assert_eq!(*result.numer().unwrap(), 3u64);
        assert_eq!(*result.denom().unwrap(), 2u64);
    }

    #[test]
    fn test_parse_fraction_decimal() {
        let result = parse_quantity("0.5").unwrap();
        // 0.5 should convert to a fraction approximation
        let value = *result.numer().unwrap() as f64 / *result.denom().unwrap() as f64;
        assert!((value - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_parse_fraction_whole_number() {
        let result = parse_quantity("2").unwrap();
        assert_eq!(*result.numer().unwrap(), 2u64);
        assert_eq!(*result.denom().unwrap(), 1u64);
    }

    #[test]
    fn test_parse_fraction_rejects_negative_whole() {
        let result = parse_quantity("-2");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Negative quantities are not allowed"));
    }

    #[test]
    fn test_parse_fraction_rejects_negative_decimal() {
        let result = parse_quantity("-0.5");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Negative quantities are not allowed"));
    }

    #[test]
    fn test_parse_fraction_rejects_negative_pure_fraction() {
        let result = parse_quantity("-1/2");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Negative quantities are not allowed"));
    }

    #[test]
    fn test_parse_fraction_rejects_negative_mixed_fraction() {
        let result = parse_quantity("-1 1/2");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Negative quantities are not allowed"));
    }

    #[test]
    fn test_parse_fraction_rejects_negative_denominator() {
        let result = parse_quantity("1/-2");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Negative denominator is not allowed"));
    }

    #[test]
    fn test_format_fraction_simplify() {
        let fraction = Fraction::new(4u64, 8u64);
        let formatted = format_quantity(fraction);
        // 4/8 simplifies to 1/2
        assert_eq!(formatted, "1/2");
    }

    #[test]
    fn test_format_fraction_mixed() {
        let fraction = Fraction::new(3u64, 2u64);
        let formatted = format_quantity(fraction);
        // 3/2 = 1 1/2
        assert_eq!(formatted, "1 1/2");
    }

    #[test]
    fn test_format_fraction_whole() {
        let fraction = Fraction::new(4u64, 2u64);
        let formatted = format_quantity(fraction);
        // 4/2 = 2
        assert_eq!(formatted, "2");
    }

    #[test]
    fn test_aggregate_fractions_addition() {
        let frac1 = parse_quantity("1/2").unwrap();
        let frac2 = parse_quantity("1/4").unwrap();

        let sum = frac1 + frac2;

        // 1/2 + 1/4 = 3/4
        assert_eq!(format_quantity(sum), "3/4");
    }

    // AC #7: Practical Rounding Tests

    #[test]
    fn test_round_to_quarter_small_quantities() {
        let value = Fraction::from(0.23);
        let rounded = round_to_practical_value(value);
        let formatted = format_quantity(rounded);
        // 0.23 → 1/4 (0.25)
        assert_eq!(formatted, "1/4");
    }

    #[test]
    fn test_round_to_third_small_quantities() {
        let value = Fraction::from(0.34);
        let rounded = round_to_practical_value(value);
        let formatted = format_quantity(rounded);
        // 0.34 → 1/3
        assert_eq!(formatted, "1/3");
    }

    #[test]
    fn test_round_to_half_medium_quantities() {
        let value = Fraction::from(1.7);
        let rounded = round_to_practical_value(value);
        let formatted = format_quantity(rounded);
        // 1.7 → 1.5 (1 1/2)
        assert_eq!(formatted, "1 1/2");
    }

    #[test]
    fn test_round_to_whole_large_quantities() {
        let value = Fraction::from(10.3);
        let rounded = round_to_practical_value(value);
        let formatted = format_quantity(rounded);
        // 10.3 → 10
        assert_eq!(formatted, "10");
    }

    #[test]
    fn test_round_avoid_excessive_precision() {
        let value = Fraction::from(2.347);
        let rounded = round_to_practical_value(value);
        let formatted = format_quantity(rounded);
        // 2.347 → 2.5 (2 1/2)
        assert_eq!(formatted, "2 1/2");
    }
}

pub struct UaInfo {
    pub device: String,
    pub browser: String,
    pub os: String,
    pub is_bot: bool,
}

/// Classifies a User-Agent string. Unparseable UAs are treated as bots: real
/// browsers are all in woothee's dataset, so unknowns are overwhelmingly
/// scripts and scanners.
pub fn classify(user_agent: &str) -> UaInfo {
    match woothee::parser::Parser::new().parse(user_agent) {
        Some(result) => UaInfo {
            is_bot: matches!(result.category, "crawler" | "misc" | "UNKNOWN"),
            device: result.category.to_owned(),
            browser: result.name.to_owned(),
            os: result.os.to_owned(),
        },
        None => UaInfo {
            device: "UNKNOWN".to_owned(),
            browser: "UNKNOWN".to_owned(),
            os: "UNKNOWN".to_owned(),
            is_bot: true,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn desktop_chrome() {
        let ua = classify(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
        );
        assert!(!ua.is_bot);
        assert_eq!(ua.device, "pc");
        assert_eq!(ua.browser, "Chrome");
    }

    #[test]
    fn iphone_safari() {
        let ua = classify(
            "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1",
        );
        assert!(!ua.is_bot);
        assert_eq!(ua.device, "smartphone");
        assert_eq!(ua.browser, "Safari");
    }

    #[test]
    fn googlebot_is_bot() {
        let ua =
            classify("Mozilla/5.0 (compatible; Googlebot/2.1; +http://www.google.com/bot.html)");
        assert!(ua.is_bot);
    }

    #[test]
    fn garbage_is_bot() {
        assert!(classify("curl/8.5.0").is_bot);
        assert!(classify("").is_bot);
    }
}

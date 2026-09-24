/// Collapses a User-Agent into a key that is stable across browser updates.
///
/// Sessions used to be validated by exact-matching the stored User-Agent
/// against the request's. Chrome's reduced User-Agent freezes the minor,
/// build and patch fields to `0.0.0` but **still increments the major
/// version** with every release — roughly every four weeks. So the stored
/// string stopped matching on the next browser update and the user was
/// bounced to `/login`, well before their token would have expired. The same
/// exact match is the per-device dedup key in the login projection, so each
/// browser release also appended a *new* login entry instead of replacing
/// one, growing the stored blob without bound.
///
/// Normalizing both sides fixes both symptoms without changing what is
/// stored, which keeps the persisted event shape and the projection snapshot
/// byte-identical — no `EVENTO_LOCK=update`, no revision bump, no replay, and
/// nobody is logged out by the deploy itself.
///
/// The key deliberately keeps platform and engine identity (Android vs
/// Windows, Chrome vs Firefox, Mobile vs desktop) so a session still cannot
/// move between devices; only version numbers are discarded.
///
/// Unknown tokens are preserved on purpose. `imkitchen-twa` is appended to
/// the stored User-Agent to tell the Android shell's session apart from a
/// browser session in the same Chrome, and a real UA *parser* (such as the
/// `woothee` dependency in `imkitchen-audience`) would discard it as noise.
/// That is why this is deliberately string surgery rather than parsing.
pub fn stable_ua_key(user_agent: &str) -> String {
    let mut key = String::with_capacity(user_agent.len());

    for raw in user_agent.split(|c: char| c.is_whitespace() || matches!(c, ';' | ',' | '(' | ')')) {
        // Drop the version run attached to a product token: `chrome/141.0.0.0`
        // and `mobile/15e148` both reduce to their product name, and `rv:109.0`
        // to `rv`. Split on the first `/` or `:` whose next character is a
        // digit, so tokens like `https://…` survive untouched.
        let token = match raw.find(['/', ':']) {
            Some(i) if raw[i + 1..].starts_with(|c: char| c.is_ascii_digit()) => &raw[..i],
            _ => raw,
        };

        // A token that *starts* with a digit is a bare version — `10` in
        // `Android 10`, `17_5_1` in `CPU iPhone OS 17_5_1`, `14` in
        // `Android 14`. Product names never start with a digit.
        if token.is_empty() || token.starts_with(|c: char| c.is_ascii_digit()) {
            continue;
        }

        if !key.is_empty() {
            key.push(' ');
        }

        key.extend(token.chars().flat_map(|c| c.to_lowercase()));
    }

    key
}

#[cfg(test)]
mod tests {
    use super::stable_ua_key;

    const CHROME_ANDROID_141: &str = "Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/141.0.0.0 Mobile Safari/537.36";
    const CHROME_ANDROID_142: &str = "Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/142.0.0.0 Mobile Safari/537.36";
    const CHROME_WINDOWS: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/141.0.0.0 Safari/537.36";
    const FIREFOX_ANDROID: &str =
        "Mozilla/5.0 (Android 14; Mobile; rv:109.0) Gecko/20100101 Firefox/130.0";
    const SAFARI_IOS: &str = "Mozilla/5.0 (iPhone; CPU iPhone OS 17_5_1 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.5 Mobile/15E148 Safari/604.1";

    /// The regression this function exists for: a Chrome update must not end
    /// the session.
    #[test]
    fn chrome_major_bump_keeps_the_same_key() {
        assert_eq!(
            stable_ua_key(CHROME_ANDROID_141),
            stable_ua_key(CHROME_ANDROID_142)
        );
    }

    #[test]
    fn ios_point_release_keeps_the_same_key() {
        let ios_17_6 = SAFARI_IOS
            .replace("17_5_1", "17_6")
            .replace("Version/17.5", "Version/17.6")
            .replace("Mobile/15E148", "Mobile/15E999");

        assert_eq!(stable_ua_key(SAFARI_IOS), stable_ua_key(&ios_17_6));
    }

    #[test]
    fn different_platforms_keep_different_keys() {
        assert_ne!(
            stable_ua_key(CHROME_ANDROID_141),
            stable_ua_key(CHROME_WINDOWS)
        );
    }

    #[test]
    fn different_engines_keep_different_keys() {
        assert_ne!(
            stable_ua_key(CHROME_ANDROID_141),
            stable_ua_key(FIREFOX_ANDROID)
        );
        assert_ne!(stable_ua_key(CHROME_ANDROID_141), stable_ua_key(SAFARI_IOS));
    }

    /// The native shell's session must not collide with a browser session in
    /// the same Chrome — they share a User-Agent, so the marker is the only
    /// thing telling them apart.
    #[test]
    fn native_marker_distinguishes_the_app_from_the_browser() {
        let native = format!("{CHROME_ANDROID_141} imkitchen-twa/1");

        assert_ne!(stable_ua_key(CHROME_ANDROID_141), stable_ua_key(&native));
        assert!(stable_ua_key(&native).ends_with("imkitchen-twa"));
    }

    /// The marker must survive its own version suffix being bumped, for the
    /// same reason the browser version is stripped.
    #[test]
    fn native_marker_ignores_its_version_suffix() {
        assert_eq!(
            stable_ua_key(&format!("{CHROME_ANDROID_141} imkitchen-twa/1")),
            stable_ua_key(&format!("{CHROME_ANDROID_141} imkitchen-twa/2"))
        );
    }

    #[test]
    fn versions_are_stripped_but_identity_survives() {
        assert_eq!(
            stable_ua_key(CHROME_ANDROID_141),
            "mozilla linux android k applewebkit khtml like gecko chrome mobile safari"
        );
    }

    #[test]
    fn empty_and_whitespace_agents_are_empty() {
        assert_eq!(stable_ua_key(""), "");
        assert_eq!(stable_ua_key("   "), "");
        assert_eq!(stable_ua_key("(;,)"), "");
    }

    /// Two different users on the same device build the same key; that is
    /// fine, because the key is only ever compared within one account's own
    /// login list.
    #[test]
    fn normalization_is_idempotent() {
        let once = stable_ua_key(CHROME_ANDROID_141);

        assert_eq!(stable_ua_key(&once), once);
    }
}

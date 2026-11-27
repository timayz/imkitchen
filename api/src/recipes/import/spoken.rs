//! Reads a time as people write it: "25 min", "1 h 30", "2 heures", "1 hour
//! 15 minutes".

/// The minutes of a time that `text` is, and nothing else.
pub fn minutes(text: &str) -> Option<u32> {
    let says = text.trim().to_lowercase();
    let (minutes, rest) = leading(&says)?;
    rest.trim().is_empty().then_some(minutes)
}

/// The minutes of the first time `text` says somewhere: "Bake 25 minutes at
/// 180°C." says 25. A number that is the end of another ("1.5 hours",
/// "1/2 hour") starts none.
pub fn first_minutes(text: &str) -> Option<u32> {
    let says = text.to_lowercase();
    let mut before = ' ';
    for (at, c) in says.char_indices() {
        let starts = c.is_ascii_digit() && !before.is_ascii_digit();
        if starts
            && !matches!(before, '.' | ',' | '/')
            && let Some((minutes, _)) = leading(&says[at..])
        {
            return Some(minutes);
        }
        before = c;
    }
    None
}

/// The minutes of the time `says` starts with, in lowercase, and what
/// follows it.
fn leading(says: &str) -> Option<(u32, &str)> {
    let (first, rest) = number(says)?;
    let (unit, rest) = word(rest);
    if is_minutes(unit) {
        return Some((first, rest));
    }
    if !is_hours(unit) {
        return None;
    }
    let hours = first.checked_mul(60)?;
    // "1 h 30", "1 hour and 15 minutes".
    let (link, linked) = word(rest);
    let more = match link {
        "et" | "and" => linked,
        _ => rest,
    };
    let minutes = number(more).and_then(|(minutes, after)| {
        let (unit, named) = word(after);
        match unit {
            unit if is_minutes(unit) => Some((minutes, named)),
            // A number alone after the hours is their minutes when it can
            // be: "1 h 30", not "1 h 180°C".
            _ if minutes < 60 => Some((minutes, after)),
            _ => None,
        }
    });
    Some(match minutes {
        Some((minutes, rest)) => (hours.checked_add(minutes)?, rest),
        None => (hours, rest),
    })
}

fn is_hours(unit: &str) -> bool {
    matches!(
        unit,
        "h" | "hr" | "hrs" | "heure" | "heures" | "hour" | "hours"
    )
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

    #[test]
    fn a_time_is_read_as_people_write_it() {
        for (text, read) in [
            ("25 min", Some(25)),
            (" 45 minutes ", Some(45)),
            ("30mn", Some(30)),
            ("5 min.", Some(5)),
            ("1 h", Some(60)),
            ("1h30", Some(90)),
            ("1 h 30 min", Some(90)),
            ("2 heures", Some(120)),
            ("1 hour 15 minutes", Some(75)),
            ("1 hour and 15 minutes", Some(75)),
            ("1 hr. 5 mins", Some(65)),
            ("Prep: 15 min", None),
            ("25", None),
            ("25 g", None),
            ("1 h 30 g", None),
            ("min", None),
            ("", None),
        ] {
            assert_eq!(minutes(text), read, "{text}");
        }
    }

    #[test]
    fn the_first_time_a_text_says_is_found() {
        for (text, read) in [
            ("Bake 25 minutes at 180°C.", Some(25)),
            ("Enfourner à 180°C pendant 1 h 30, puis démouler.", Some(90)),
            ("Laisser reposer 1 h 30 avant de trancher.", Some(90)),
            ("Cuire 1 h 180°C.", Some(60)),
            ("Laisser reposer 1 heure et 30 minutes.", Some(90)),
            ("Cuire 1 h à 180°C.", Some(60)),
            ("Bake 1 hour, 200 g of butter aside.", Some(60)),
            ("Simmer for 10 to 15 min, then rest 5 minutes.", Some(15)),
            ("Cook each one 2-3 minutes a side.", Some(3)),
            ("Laisser cuire 20min.", Some(20)),
            ("Add 250 g of flour and 2 eggs.", None),
            ("Rest 1.5 hours.", None),
            ("Wait 1/2 hour.", None),
            ("Cut in 4 handfuls, mince the garlic.", None),
            ("Mix everything.", None),
            ("", None),
        ] {
            assert_eq!(first_minutes(text), read, "{text}");
        }
    }
}

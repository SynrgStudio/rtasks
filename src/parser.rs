use chrono::{Days, Local, NaiveDate};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedTaskInput {
    pub title: String,
    pub due: Option<String>,
}

pub fn parse_task_input(input: &str) -> ParsedTaskInput {
    parse_task_input_with_today(input, Local::now().date_naive())
}

pub fn parse_task_input_with_today(input: &str, today: NaiveDate) -> ParsedTaskInput {
    let trimmed = input.trim();

    for marker in due_markers(today) {
        if let Some(title) = strip_due_marker(trimmed, marker.phrase) {
            return ParsedTaskInput {
                title,
                due: Some(marker.date.format("%Y-%m-%d").to_string()),
            };
        }
    }

    ParsedTaskInput {
        title: trimmed.to_owned(),
        due: None,
    }
}

fn due_markers(today: NaiveDate) -> Vec<DueMarker> {
    vec![
        DueMarker {
            phrase: "pasado mañana",
            date: today.checked_add_days(Days::new(2)).unwrap_or(today),
        },
        DueMarker {
            phrase: "pasado manana",
            date: today.checked_add_days(Days::new(2)).unwrap_or(today),
        },
        DueMarker {
            phrase: "mañana",
            date: today.checked_add_days(Days::new(1)).unwrap_or(today),
        },
        DueMarker {
            phrase: "manana",
            date: today.checked_add_days(Days::new(1)).unwrap_or(today),
        },
        DueMarker {
            phrase: "hoy",
            date: today,
        },
    ]
}

fn strip_due_marker(input: &str, marker: &str) -> Option<String> {
    let normalized = input.to_lowercase();

    if normalized == marker {
        return Some(String::new());
    }

    if normalized.ends_with(marker) {
        let title_len = input.len().saturating_sub(marker.len());
        let title = input.get(..title_len)?.trim();
        return Some(title.to_owned());
    }

    None
}

struct DueMarker {
    phrase: &'static str,
    date: NaiveDate,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_manana_and_removes_marker() {
        let today = NaiveDate::from_ymd_opt(2026, 4, 25).expect("valid date");

        let parsed = parse_task_input_with_today("comprar pan mañana", today);

        assert_eq!(parsed.title, "comprar pan");
        assert_eq!(parsed.due, Some("2026-04-26".to_owned()));
    }

    #[test]
    fn parses_manana_without_accent() {
        let today = NaiveDate::from_ymd_opt(2026, 4, 25).expect("valid date");

        let parsed = parse_task_input_with_today("comprar pan manana", today);

        assert_eq!(parsed.title, "comprar pan");
        assert_eq!(parsed.due, Some("2026-04-26".to_owned()));
    }

    #[test]
    fn parses_hoy() {
        let today = NaiveDate::from_ymd_opt(2026, 4, 25).expect("valid date");

        let parsed = parse_task_input_with_today("pagar dominio hoy", today);

        assert_eq!(parsed.title, "pagar dominio");
        assert_eq!(parsed.due, Some("2026-04-25".to_owned()));
    }

    #[test]
    fn parses_pasado_manana_before_manana() {
        let today = NaiveDate::from_ymd_opt(2026, 4, 25).expect("valid date");

        let parsed = parse_task_input_with_today("backup pasado mañana", today);

        assert_eq!(parsed.title, "backup");
        assert_eq!(parsed.due, Some("2026-04-27".to_owned()));
    }

    #[test]
    fn parses_pasado_manana_without_accent() {
        let today = NaiveDate::from_ymd_opt(2026, 4, 25).expect("valid date");

        let parsed = parse_task_input_with_today("backup pasado manana", today);

        assert_eq!(parsed.title, "backup");
        assert_eq!(parsed.due, Some("2026-04-27".to_owned()));
    }

    #[test]
    fn leaves_input_without_due_marker_unchanged() {
        let today = NaiveDate::from_ymd_opt(2026, 4, 25).expect("valid date");

        let parsed = parse_task_input_with_today("comprar pan", today);

        assert_eq!(parsed.title, "comprar pan");
        assert_eq!(parsed.due, None);
    }
}

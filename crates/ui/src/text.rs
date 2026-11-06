const STEPS: [(&str, u64); 6] = [
    ("year", 31_536_000),
    ("month", 2_592_000),
    ("week", 604_800),
    ("day", 86_400),
    ("hour", 3_600),
    ("minute", 60),
];

fn js_round(value: f64) -> i64 {
    opsis_types::numbers::truncate::<i64>((value + 0.5).floor())
}

fn named(unit: &str, amount: i64) -> Option<String> {
    let word = match (unit, amount) {
        ("day", -1) => "yesterday",
        ("day", 1) => "tomorrow",
        (_, -1) => return Some(format!("last {unit}")),
        (_, 1) => return Some(format!("next {unit}")),
        (_, 0) => return Some(format!("this {unit}")),
        _ => return None,
    };
    Some(word.to_owned())
}

fn numeric(unit: &str, amount: i64) -> String {
    let count = amount.unsigned_abs();
    let units = if count == 1 {
        unit.to_owned()
    } else {
        format!("{unit}s")
    };
    if amount < 0 {
        format!("{count} {units} ago")
    } else {
        format!("in {count} {units}")
    }
}

pub fn time_ago(at_ms: i64, now_ms: i64) -> String {
    let difference = at_ms - now_ms;
    let sign = if difference < 0 { -1.0 } else { 1.0 };
    let seconds = sign * opsis_types::numbers::float_wide(difference.unsigned_abs()) / 1000.0;
    for (unit, size) in STEPS {
        let size = opsis_types::numbers::float_wide(size);
        if seconds.abs() >= size {
            let amount = js_round(seconds / size);
            let relative = if matches!(unit, "hour" | "minute") {
                None
            } else {
                named(unit, amount)
            };
            return relative.unwrap_or_else(|| numeric(unit, amount));
        }
    }
    "just now".to_owned()
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    const NOW: i64 = 1_800_000_000_000;

    #[test]
    fn reads_like_the_web_editor() {
        assert_eq!(time_ago(NOW - 20_000, NOW), "just now");
        assert_eq!(time_ago(NOW - 60_000, NOW), "1 minute ago");
        assert_eq!(time_ago(NOW - 5 * 60_000, NOW), "5 minutes ago");
        assert_eq!(time_ago(NOW - 3 * 3_600_000, NOW), "3 hours ago");
        assert_eq!(time_ago(NOW - 86_400_000, NOW), "yesterday");
        assert_eq!(time_ago(NOW - 3 * 86_400_000, NOW), "3 days ago");
        assert_eq!(time_ago(NOW - 8 * 86_400_000, NOW), "last week");
        assert_eq!(time_ago(NOW - 400 * 86_400_000, NOW), "last year");
        assert_eq!(time_ago(NOW + 2 * 3_600_000, NOW), "in 2 hours");
    }

    proptest! {
        #[test]
        fn the_past_never_reads_as_the_future(ago in 0i64..10_000_000_000_000) {
            let text = time_ago(NOW - ago, NOW);
            prop_assert!(!text.starts_with("in ") && !text.starts_with("next ") && text != "tomorrow");
        }
    }
}

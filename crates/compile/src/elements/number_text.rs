use crate::elements::shared::{GroupSeparator, NumberFormat};

const COMPACT_UNITS: [(i32, &str); 4] = [(3, "K"), (6, "M"), (9, "B"), (12, "T")];

struct Decimal {
    negative: bool,
    whole: String,
    fraction: String,
}

fn decimal_of(value: f64) -> Decimal {
    let text = format!("{}", value.abs());
    let (whole, fraction) = text.split_once('.').unwrap_or((&text, ""));
    Decimal {
        negative: value.is_sign_negative(),
        whole: whole.to_owned(),
        fraction: fraction.to_owned(),
    }
}

fn add_one(digits: &mut Vec<u8>) {
    for d in digits.iter_mut().rev() {
        if *d == b'9' {
            *d = b'0';
        } else {
            *d += 1;
            return;
        }
    }
    digits.insert(0, b'1');
}

fn rounded(decimal: Decimal, places: usize) -> Decimal {
    if decimal.fraction.len() <= places {
        return decimal;
    }
    let mut digits: Vec<u8> = decimal
        .whole
        .bytes()
        .chain(decimal.fraction.bytes().take(places))
        .collect();
    if decimal.fraction.as_bytes()[places] >= b'5' {
        add_one(&mut digits);
    }
    let split = digits.len() - places;
    let fraction = String::from_utf8_lossy(&digits[split..])
        .trim_end_matches('0')
        .to_owned();
    let whole = String::from_utf8_lossy(&digits[..split]).to_string();
    Decimal {
        negative: decimal.negative,
        whole: if whole.is_empty() {
            "0".to_owned()
        } else {
            whole
        },
        fraction,
    }
}

fn grouped(whole: &str, separator: &str) -> String {
    let mut out = String::new();
    for (i, c) in whole.chars().enumerate() {
        if i > 0 && (whole.len() - i) % 3 == 0 {
            out.push_str(separator);
        }
        out.push(c);
    }
    out
}

fn text_of(decimal: &Decimal, separator: &str) -> String {
    let sign = if decimal.negative { "-" } else { "" };
    let whole = grouped(&decimal.whole, separator);
    if decimal.fraction.is_empty() {
        format!("{sign}{whole}")
    } else {
        format!("{sign}{whole}.{}", decimal.fraction)
    }
}

pub fn format_plain(value: f64, max_fraction: usize) -> String {
    if !value.is_finite() {
        return if value.is_nan() {
            "NaN".to_owned()
        } else if value > 0.0 {
            "∞".to_owned()
        } else {
            "-∞".to_owned()
        };
    }
    text_of(&rounded(decimal_of(value), max_fraction), ",")
}

fn magnitude(decimal: &Decimal) -> i32 {
    if decimal.whole != "0" {
        return i32::try_from(decimal.whole.len()).unwrap_or(i32::MAX) - 1;
    }
    -1
}

pub fn format_compact(value: f64, max_fraction: usize) -> String {
    if !value.is_finite() {
        return format_plain(value, max_fraction);
    }
    let first = rounded(decimal_of(value), max_fraction);
    let mut power = magnitude(&first);
    loop {
        let Some((exponent, suffix)) = COMPACT_UNITS
            .iter()
            .rev()
            .find(|(exponent, _)| power >= *exponent)
        else {
            return text_of(&first, ",");
        };
        let scaled = value / 10_f64.powi(*exponent);
        let shown = rounded(decimal_of(scaled), max_fraction);
        let grown = magnitude(&shown) + exponent;
        if grown > power && COMPACT_UNITS.iter().any(|(e, _)| *e == grown) {
            power = grown;
            continue;
        }
        return format!("{}{suffix}", text_of(&shown, ""));
    }
}

pub fn format_number(value: f64, format: NumberFormat) -> String {
    match format {
        NumberFormat::Compact => format_compact(value, 1),
        NumberFormat::Full => format_plain(value, 2),
    }
}

pub fn format_axis_value(value: f64) -> String {
    format_compact(value, usize::from(value.abs() < 10.0))
}

pub const fn separator_text(separator: GroupSeparator) -> &'static str {
    match separator {
        GroupSeparator::None => "",
        GroupSeparator::Space => "\u{2009}",
        GroupSeparator::Comma => ",",
        GroupSeparator::Dot => ".",
    }
}

pub fn group_digits(digits: &str, separator: GroupSeparator) -> String {
    let mark = separator_text(separator);
    let mut out = String::new();
    let chars: Vec<char> = digits.chars().collect();
    for (i, c) in chars.iter().enumerate() {
        out.push(*c);
        let run = chars[i + 1..]
            .iter()
            .take_while(|d| d.is_ascii_digit())
            .count();
        let word = c.is_ascii_alphanumeric() || *c == '_';
        if word && run > 0 && run % 3 == 0 {
            out.push_str(mark);
        }
    }
    out
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Flat,
}

pub fn read_delta(delta: &str) -> (Direction, String) {
    let trimmed = delta.trim();
    if trimmed.is_empty() {
        return (Direction::Flat, String::new());
    }
    let direction = if trimmed.starts_with('+') {
        Direction::Up
    } else if trimmed.starts_with('-') || trimmed.starts_with('\u{2212}') {
        Direction::Down
    } else {
        Direction::Flat
    };
    (direction, trimmed.to_owned())
}

pub const MINUS: &str = "\u{2212}";

pub use opsis_types::js::js_round;

pub fn round_to(value: f64, digits: i32) -> f64 {
    let scale = 10f64.powi(digits);
    js_round(value * scale) / scale
}

pub fn number(value: f64) -> String {
    if value == 0.0 {
        return "0".to_owned();
    }
    ryu_js::Buffer::new().format(value).to_owned()
}

const TIE_CHECK_DIGITS: usize = 30;

fn exact_tie(magnitude: f64, digits: usize) -> bool {
    let text = format!("{magnitude:.prec$}", prec = digits + TIE_CHECK_DIGITS);
    let fraction = text.split_once('.').map_or("", |(_, f)| f);
    fraction.as_bytes().get(digits) == Some(&b'5')
        && fraction[digits + 1..].bytes().all(|b| b == b'0')
}

pub fn to_fixed(value: f64, digits: usize) -> String {
    if !value.is_finite() {
        return number(value);
    }
    let magnitude = value.abs();
    let rounded = if exact_tie(magnitude, digits) {
        let scale = 10f64.powi(i32::try_from(digits).unwrap_or(i32::MAX));
        let up = ((magnitude * scale).floor() + 1.0) / scale;
        format!("{up:.digits$}")
    } else {
        format!("{magnitude:.digits$}")
    };
    if value < 0.0 {
        format!("-{rounded}")
    } else {
        rounded
    }
}

pub fn decimals(value: f64, digits: i32) -> String {
    number(round_to(value, digits))
}

pub fn grouped(value: f64, max_digits: i32) -> String {
    use num_format::{Locale, ToFormattedString};
    let rounded = round_to(value.abs(), max_digits);
    let whole = (rounded.trunc() as u64).to_formatted_string(&Locale::en);
    match decimals(rounded.fract(), max_digits).strip_prefix("0.") {
        Some(rest) => format!("{whole}.{rest}"),
        None => whole,
    }
}

pub fn sign_of(value: f64, plus: bool) -> &'static str {
    if value < 0.0 {
        MINUS
    } else if plus && value > 0.0 {
        "+"
    } else {
        ""
    }
}

pub fn signed_text(value: f64, digits: Option<i32>, plus: bool) -> String {
    let magnitude = value.abs();
    let text = match digits {
        None => number(magnitude),
        Some(d) => to_fixed(magnitude, usize::try_from(d).unwrap_or(0)),
    };
    if text.parse::<f64>().map(|n| n == 0.0).unwrap_or(false) {
        text
    } else {
        format!("{}{text}", sign_of(value, plus))
    }
}

const SECONDS_PER_MINUTE: f64 = 60.0;
const SECONDS_PER_HOUR: f64 = 3600.0;

fn minutes_and_seconds(seconds: f64) -> String {
    let h = (seconds / SECONDS_PER_HOUR).floor();
    let m = ((seconds - h * SECONDS_PER_HOUR) / SECONDS_PER_MINUTE).floor();
    let rest = round_to(seconds - h * SECONDS_PER_HOUR - m * SECONDS_PER_MINUTE, 3);
    if h == 0.0 {
        return if rest != 0.0 {
            format!("{} min {} s", number(m), number(rest))
        } else {
            format!("{} min", number(m))
        };
    }
    let parts = [
        format!("{} h", number(h)),
        if m != 0.0 {
            format!("{} min", number(m))
        } else {
            String::new()
        },
        if rest != 0.0 {
            format!("{} s", number(rest))
        } else {
            String::new()
        },
    ];
    parts
        .into_iter()
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn format_duration(ms: f64) -> String {
    if !ms.is_finite() {
        return String::new();
    }
    let s = ms / 1000.0;
    if s >= SECONDS_PER_MINUTE {
        return minutes_and_seconds(s.round());
    }
    let shown = if s >= 10.0 {
        round_to(s, 1)
    } else {
        round_to(s, 2)
    };
    format!("{} s", number(shown))
}

pub fn format_precise_duration(ms: f64) -> String {
    if !ms.is_finite() {
        return String::new();
    }
    let s = round_to(ms / 1000.0, 3);
    if s >= SECONDS_PER_MINUTE {
        minutes_and_seconds(s)
    } else {
        format!("{} s", number(s))
    }
}

pub fn plural(n: f64, word: &str) -> String {
    plural_with(n, word, &format!("{word}s"))
}

pub fn plural_with(n: f64, word: &str, many: &str) -> String {
    format!("{} {}", number(n), if n == 1.0 { word } else { many })
}

pub fn percent_text(fraction: f64, max: f64) -> String {
    format!("{}%", number((fraction * 100.0).round().min(max)))
}

pub fn format_bytes(n: f64) -> String {
    const UNITS: [&str; 4] = ["B", "KB", "MB", "GB"];
    if !n.is_finite() || n < 0.0 {
        return String::new();
    }
    let mut value = n;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{} B", number(value.round()))
    } else {
        format!("{} {}", to_fixed(value, 1), UNITS[unit])
    }
}

pub fn humanize(key: &str) -> String {
    use heck::ToSnakeCase;
    let text = key.to_snake_case().replace('_', " ");
    let mut chars = text.chars();
    chars
        .next()
        .map(|c| c.to_uppercase().chain(chars).collect())
        .unwrap_or_default()
}

pub fn pascal_case(key: &str) -> String {
    use heck::ToUpperCamelCase;
    key.to_upper_camel_case()
}

pub fn display_address(url: &str, keep_query: bool) -> String {
    let text = url.trim();
    let lower = text.to_ascii_lowercase();
    let has_scheme = lower.starts_with("http://") || lower.starts_with("https://");
    let parsed = if has_scheme {
        url::Url::parse(text).ok()
    } else {
        None
    };
    let (address, query) = match &parsed {
        Some(u) => {
            let host = match u.port() {
                Some(port) => format!("{}:{port}", u.host_str().unwrap_or("")),
                None => u.host_str().unwrap_or("").to_owned(),
            };
            (
                format!("{host}{}", u.path()),
                if keep_query {
                    u.query().map(|q| format!("?{q}")).unwrap_or_default()
                } else {
                    String::new()
                },
            )
        }
        None => {
            let without = if has_scheme {
                &text[text.find("//").map_or(0, |i| i + 2)..]
            } else {
                text
            };
            (without.to_owned(), String::new())
        }
    };
    let address = address.strip_prefix("www.").unwrap_or(&address);
    let address = address.strip_suffix('/').unwrap_or(address);
    format!("{address}{query}")
}

pub fn truncate(text: &str, max: usize) -> String {
    let one = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if one.chars().count() > max {
        format!("{}…", one.chars().take(max - 1).collect::<String>())
    } else {
        one
    }
}

pub fn single_quoted(text: &str) -> String {
    format!("'{}'", text.replace('\'', "'\\''"))
}

pub fn non_blank_lines(text: &str) -> Vec<String> {
    text.split('\n')
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(str::to_owned)
        .collect()
}

pub fn format_when(time: &str) -> String {
    match time.trim().parse::<jiff::civil::DateTime>() {
        Ok(t) => t.strftime("%-d %b %Y, %H:%M").to_string(),
        Err(_) => time.to_owned(),
    }
}

pub fn is_wall_time(time: &str) -> bool {
    time.trim().parse::<jiff::civil::DateTime>().is_ok()
}

pub fn wall_time_in(time: &str, zone: &str) -> Result<jiff::Timestamp, String> {
    let wall = time
        .trim()
        .parse::<jiff::civil::DateTime>()
        .map_err(|_| format!("“{time}” isn't a date and time."))?;
    let tz = jiff::tz::TimeZone::get(zone).map_err(|_| format!("“{zone}” isn't a time zone."))?;
    wall.to_zoned(tz)
        .map(|z| z.timestamp())
        .map_err(|e| e.to_string())
}

pub fn is_locale(tag: &str) -> bool {
    language_tags::LanguageTag::parse(tag).is_ok_and(|t| t.is_valid())
}

pub fn is_time_zone(zone: &str) -> bool {
    jiff::tz::TimeZone::get(zone).is_ok()
}

pub fn slug_name(text: &str, fallback: &str) -> String {
    let name = slug::slugify(text);
    if name.is_empty() {
        fallback.to_owned()
    } else {
        name
    }
}

pub fn project_folder_name(name: &str) -> String {
    slug_name(name, "demo")
}

pub fn parse_number_text(text: &str) -> Option<f64> {
    let normal = text.trim().replacen(',', ".", 1);
    if normal.is_empty() {
        return None;
    }
    normal.parse::<f64>().ok().filter(|n| n.is_finite())
}

#[cfg(test)]
mod js_number_tests {
    use super::*;

    #[test]
    fn rounds_halves_up_like_math_round() {
        assert_eq!(
            [
                js_round(2.5),
                js_round(-2.5),
                js_round(-2.6),
                js_round(0.49999999999999994)
            ],
            [3.0, -2.0, -3.0, 0.0]
        );
        assert_eq!(round_to(-0.0625, 3), -0.062);
    }

    #[test]
    fn writes_fixed_digits_rounding_exact_ties_away_from_zero() {
        assert_eq!(to_fixed(0.25, 1), "0.3");
        assert_eq!(to_fixed(0.125, 2), "0.13");
        assert_eq!(to_fixed(2.5, 0), "3");
        assert_eq!(to_fixed(-2.5, 0), "-3");
        assert_eq!(to_fixed(0.15, 1), "0.1");
        assert_eq!(to_fixed(1.005, 2), "1.00");
        assert_eq!(to_fixed(-0.0001, 2), "-0.00");
        assert_eq!(to_fixed(1.03125, 4), "1.0313");
        assert_eq!(to_fixed(0.0, 3), "0.000");
    }

    #[test]
    fn writes_numbers_as_javascript_does() {
        assert_eq!(
            [
                number(1.0),
                number(0.1 + 0.2),
                number(1e21),
                number(-0.0),
                number(1e-7)
            ],
            ["1", "0.30000000000000004", "1e+21", "0", "1e-7"]
        );
    }
}

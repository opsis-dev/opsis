use crate::errors::WallTimeError;
pub const MINUS: &str = "\u{2212}";

pub fn round_to(value: f64, digits: i32) -> f64 {
    let scale = 10f64.powi(digits);
    (value * scale).round() / scale
}

pub fn decimals(value: f64, digits: i32) -> String {
    (round_to(value, digits)).to_string()
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
        None => magnitude.to_string(),
        Some(d) => format!("{:.*}", usize::try_from(d).unwrap_or(0), magnitude),
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
            format!("{} min {} s", m, rest)
        } else {
            format!("{} min", m)
        };
    }
    let parts = [
        format!("{} h", h),
        if m != 0.0 {
            format!("{} min", m)
        } else {
            String::new()
        },
        if rest != 0.0 {
            format!("{} s", rest)
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
    format!("{} s", shown)
}

pub fn format_precise_duration(ms: f64) -> String {
    if !ms.is_finite() {
        return String::new();
    }
    let s = round_to(ms / 1000.0, 3);
    if s >= SECONDS_PER_MINUTE {
        minutes_and_seconds(s)
    } else {
        format!("{} s", s)
    }
}

pub fn plural(n: f64, word: &str) -> String {
    plural_with(n, word, &format!("{word}s"))
}

pub fn plural_with(n: f64, word: &str, many: &str) -> String {
    format!("{} {}", n, if n == 1.0 { word } else { many })
}

pub fn percent_text(fraction: f64, max: f64) -> String {
    format!("{}%", ((fraction * 100.0).round().min(max)))
}

pub fn format_bytes(n: u64) -> String {
    const UNITS: [&str; 4] = ["B", "KB", "MB", "GB"];
    let mut value = n as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{} B", value.round())
    } else {
        format!("{value:.1} {}", UNITS[unit])
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

pub fn wall_time_in(time: &str, zone: &str) -> Result<jiff::Timestamp, WallTimeError> {
    let wall = time
        .trim()
        .parse::<jiff::civil::DateTime>()
        .map_err(|_| WallTimeError::NotATime(time.to_owned()))?;
    let tz = jiff::tz::TimeZone::get(zone).map_err(|_| WallTimeError::NotAZone(zone.to_owned()))?;
    Ok(wall.to_zoned(tz)?.timestamp())
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

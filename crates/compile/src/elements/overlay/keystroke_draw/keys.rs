use crate::elements::overlay::keystroke::KeyboardPlatform;

pub const WORD_FONT: f64 = 0.26;
const WORD_CHAR: f64 = 0.6 * WORD_FONT;
const WORD_PAD: f64 = 0.24;
pub const WINDOWS_WORD: f64 = 1.25;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Icon {
    Command,
    Option,
    Shift,
    Control,
    Return,
    Delete,
    Escape,
    Tab,
    Space,
    CapsLock,
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Legend {
    Icon(Icon),
    Char(String),
    Word(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Key {
    pub label: String,
    pub legend: Legend,
    pub units: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum Modifier {
    Fn,
    Ctrl,
    Alt,
    Shift,
    Meta,
}

pub const fn icon_shape(icon: Icon) -> (&'static str, &'static [&'static str]) {
    match icon {
        Icon::Command => (
            "0 0 24 24",
            &["M15 6v12a3 3 0 1 0 3-3H6a3 3 0 1 0 3 3V6a3 3 0 1 0-3 3h12a3 3 0 1 0-3-3"],
        ),
        Icon::Option => ("0 0 24 24", &["M3 3h6l6 18h6", "M14 3h7"]),
        Icon::Shift => (
            "0 0 24 24",
            &[
                "M9 19a1 1 0 0 0 1 1h4a1 1 0 0 0 1-1v-6a1 1 0 0 1 1-1h3.293a.707.707 0 0 0 .5-1.207l-7.086-7.086a1 1 0 0 0-1.414 0l-7.086 7.086a.707.707 0 0 0 .5 1.207H8a1 1 0 0 1 1 1z",
            ],
        ),
        Icon::Control => ("0 0 24 24", &["m18 15-6-6-6 6"]),
        Icon::Return => ("0 0 24 24", &["M20 4v7a4 4 0 0 1-4 4H4", "m9 10-5 5 5 5"]),
        Icon::Delete => (
            "0 0 24 24",
            &[
                "M10 5a2 2 0 0 0-1.344.519l-6.328 5.74a1 1 0 0 0 0 1.481l6.328 5.741A2 2 0 0 0 10 19h10a2 2 0 0 0 2-2V7a2 2 0 0 0-2-2z",
                "m12 9 6 6",
                "m18 9-6 6",
            ],
        ),
        Icon::Escape => (
            "0 0 24 24",
            &["M2 8V2h6", "m2 2 10 10", "M12 2A10 10 0 1 1 2 12"],
        ),
        Icon::Tab => ("0 0 24 24", &["M17 12H3", "m11 18 6-6-6-6", "M21 5v14"]),
        Icon::Space => ("0 6 24 24", &["M22 17v1c0 .5-.5 1-1 1H3c-.5 0-1-.5-1-1v-1"]),
        Icon::CapsLock => (
            "0 0 24 24",
            &[
                "M14 16a1 1 0 0 0 1-1v-2a1 1 0 0 1 1-1h3.293a.707.707 0 0 0 .5-1.207l-6.939-6.939a1.207 1.207 0 0 0-1.708 0l-6.94 6.94a.707.707 0 0 0 .5 1.206H8a1 1 0 0 1 1 1v2a1 1 0 0 0 1 1z",
                "M9 20h6",
            ],
        ),
        Icon::Up => ("0 0 24 24", &["m5 12 7-7 7 7", "M12 19V5"]),
        Icon::Down => ("0 0 24 24", &["M12 5v14", "m19 12-7 7-7-7"]),
        Icon::Left => ("0 0 24 24", &["m12 19-7-7 7-7", "M19 12H5"]),
        Icon::Right => ("0 0 24 24", &["M5 12h14", "m12 5 7 7-7 7"]),
    }
}

fn modifier_of(name: &str) -> Option<Modifier> {
    Some(match name {
        "fn" => Modifier::Fn,
        "ctrl" | "control" => Modifier::Ctrl,
        "alt" | "option" | "opt" => Modifier::Alt,
        "shift" => Modifier::Shift,
        "cmd" | "command" | "meta" | "win" | "super" => Modifier::Meta,
        _ => return None,
    })
}

const fn modifier_order(platform: KeyboardPlatform) -> [Modifier; 5] {
    match platform {
        KeyboardPlatform::Mac => [
            Modifier::Fn,
            Modifier::Ctrl,
            Modifier::Alt,
            Modifier::Shift,
            Modifier::Meta,
        ],
        KeyboardPlatform::Windows => [
            Modifier::Fn,
            Modifier::Meta,
            Modifier::Ctrl,
            Modifier::Alt,
            Modifier::Shift,
        ],
    }
}

fn word_units(text: &str, min: f64) -> f64 {
    let raw = opsis_types::numbers::float(text.chars().count()).mul_add(WORD_CHAR, 2.0 * WORD_PAD);
    min.max(raw.mul_add(4.0, -1e-9).ceil() / 4.0)
}

fn word_key(label: &str, text: &str, platform: KeyboardPlatform, min: f64) -> Key {
    let floor = if platform == KeyboardPlatform::Windows {
        min.max(WINDOWS_WORD)
    } else {
        min
    };
    Key {
        label: label.to_owned(),
        legend: Legend::Word(text.to_owned()),
        units: word_units(text, floor),
    }
}

fn icon_key(label: &str, icon: Icon, units: f64) -> Key {
    Key {
        label: label.to_owned(),
        legend: Legend::Icon(icon),
        units,
    }
}

fn modifier_key(modifier: Modifier, platform: KeyboardPlatform) -> Key {
    if platform == KeyboardPlatform::Windows {
        let word = match modifier {
            Modifier::Fn => "Fn",
            Modifier::Ctrl => "Ctrl",
            Modifier::Alt => "Alt",
            Modifier::Shift => "Shift",
            Modifier::Meta => "Win",
        };
        return word_key(word, word, platform, 1.0);
    }
    match modifier {
        Modifier::Fn => word_key("Fn", "fn", platform, 1.0),
        Modifier::Ctrl => icon_key("Control", Icon::Control, 1.0),
        Modifier::Alt => icon_key("Option", Icon::Option, 1.0),
        Modifier::Shift => icon_key("Shift", Icon::Shift, 1.5),
        Modifier::Meta => icon_key("Command", Icon::Command, 1.0),
    }
}

fn named_key(name: &str, platform: KeyboardPlatform) -> Option<Key> {
    let named = match name {
        "enter" | "return" => "enter",
        "esc" | "escape" => "escape",
        "tab" => "tab",
        "space" => "space",
        "backspace" => "backspace",
        "delete" | "del" => "delete",
        "capslock" => "capsLock",
        "up" | "arrowup" => "up",
        "down" | "arrowdown" => "down",
        "left" | "arrowleft" => "left",
        "right" | "arrowright" => "right",
        "home" => "home",
        "end" => "end",
        "pageup" | "pgup" => "pageUp",
        "pagedown" | "pgdn" => "pageDown",
        _ => return None,
    };
    let arrow = |label: &str, icon| Some(icon_key(label, icon, 1.0));
    match named {
        "up" => return arrow("Up Arrow", Icon::Up),
        "down" => return arrow("Down Arrow", Icon::Down),
        "left" => return arrow("Left Arrow", Icon::Left),
        "right" => return arrow("Right Arrow", Icon::Right),
        _ => {}
    }
    Some(match platform {
        KeyboardPlatform::Mac => match named {
            "enter" => icon_key("Return", Icon::Return, 1.5),
            "escape" => icon_key("Escape", Icon::Escape, 1.0),
            "tab" => icon_key("Tab", Icon::Tab, 1.25),
            "space" => icon_key("Space", Icon::Space, 2.5),
            "backspace" | "delete" => icon_key("Delete", Icon::Delete, 1.25),
            "capsLock" => icon_key("Caps Lock", Icon::CapsLock, 1.5),
            "home" => word_key("Home", "home", platform, 1.0),
            "end" => word_key("End", "end", platform, 1.0),
            "pageUp" => word_key("Page Up", "page up", platform, 1.0),
            _ => word_key("Page Down", "page down", platform, 1.0),
        },
        KeyboardPlatform::Windows => match named {
            "enter" => word_key("Enter", "Enter", platform, WINDOWS_WORD),
            "escape" => word_key("Esc", "Esc", platform, WINDOWS_WORD),
            "tab" => word_key("Tab", "Tab", platform, WINDOWS_WORD),
            "space" => word_key("Space", "Space", platform, 2.5),
            "backspace" => word_key("Backspace", "Backspace", platform, WINDOWS_WORD),
            "delete" => word_key("Delete", "Delete", platform, WINDOWS_WORD),
            "capsLock" => word_key("Caps Lock", "Caps Lock", platform, WINDOWS_WORD),
            "home" => word_key("Home", "Home", platform, WINDOWS_WORD),
            "end" => word_key("End", "End", platform, WINDOWS_WORD),
            "pageUp" => word_key("Page Up", "PgUp", platform, WINDOWS_WORD),
            _ => word_key("Page Down", "PgDn", platform, WINDOWS_WORD),
        },
    })
}

fn character_of(name: &str) -> Option<&'static str> {
    Some(match name {
        "plus" => "+",
        "minus" => "-",
        "comma" => ",",
        "period" => ".",
        "slash" => "/",
        "backslash" => "\\",
        "semicolon" => ";",
        "quote" => "'",
        "backquote" => "`",
        "bracketleft" => "[",
        "bracketright" => "]",
        "equal" => "=",
        _ => return None,
    })
}

fn is_function_key(name: &str) -> Option<String> {
    let digits = name.strip_prefix('f')?;
    let number: u32 = digits.parse().ok()?;
    let canonical = number.to_string() == digits;
    (canonical && (1..=24).contains(&number)).then(|| format!("F{digits}"))
}

fn capitalised(part: &str) -> String {
    let mut chars = part.chars();
    chars.next().map_or_else(String::new, |first| {
        first.to_uppercase().chain(chars).collect()
    })
}

fn key_of(part: &str, platform: KeyboardPlatform) -> Key {
    let name = part.to_lowercase();
    if let Some(key) = named_key(&name, platform) {
        return key;
    }
    let single = (part.chars().count() == 1).then_some(part);
    if let Some(character) = character_of(&name).or(single) {
        let text = character.to_uppercase();
        return Key {
            label: text.clone(),
            legend: Legend::Char(text),
            units: 1.0,
        };
    }
    if let Some(text) = is_function_key(&name) {
        return word_key(&text, &text, platform, 1.0);
    }
    let text = capitalised(part);
    word_key(&text, &text, platform, 1.0)
}

fn split_chord(token: &str) -> Vec<&str> {
    let parts: Vec<&str> = token.split('+').collect();
    let mut names = Vec::new();
    for (index, part) in parts.iter().enumerate() {
        if !part.is_empty() {
            names.push(*part);
        } else if index + 1 == parts.len() && index > 0 && parts[index - 1].is_empty() {
            names.push("+");
        }
    }
    names
}

fn parse_chord(token: &str, platform: KeyboardPlatform) -> Vec<Key> {
    let mut held = Vec::new();
    let mut keys = Vec::new();
    for part in split_chord(token) {
        let name = part.to_lowercase();
        let modifier = if name == "mod" {
            Some(if platform == KeyboardPlatform::Mac {
                Modifier::Meta
            } else {
                Modifier::Ctrl
            })
        } else {
            modifier_of(&name)
        };
        match modifier {
            Some(m) => held.push(m),
            None => keys.push(key_of(part, platform)),
        }
    }
    modifier_order(platform)
        .into_iter()
        .filter(|m| held.contains(m))
        .map(|m| modifier_key(m, platform))
        .chain(keys)
        .collect()
}

pub fn parse_keys(keys: &str, platform: KeyboardPlatform) -> Vec<Vec<Key>> {
    keys.split_whitespace()
        .map(|token| parse_chord(token, platform))
        .filter(|chord| !chord.is_empty())
        .collect()
}

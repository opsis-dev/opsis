const QT_SHIFT: u32 = 0x0200_0000;
const QT_CONTROL: u32 = 0x0400_0000;
const QT_ALT: u32 = 0x0800_0000;
const QT_META: u32 = 0x1000_0000;

const DOM_ALT: i64 = 1;
const DOM_CONTROL: i64 = 2;
const DOM_META: i64 = 4;
const DOM_SHIFT: i64 = 8;

const LOCATION_STANDARD: i64 = 0;
const LOCATION_LEFT: i64 = 1;
const LOCATION_KEYPAD: i64 = 3;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DomKey {
    pub key: String,
    pub code: String,
    pub key_code: i64,
    pub location: i64,
}

pub const fn dom_modifiers(qt: u32, mac: bool) -> i64 {
    let (control, meta) = if mac {
        (QT_META, QT_CONTROL)
    } else {
        (QT_CONTROL, QT_META)
    };
    let mut bits = 0;
    if qt & QT_ALT != 0 {
        bits |= DOM_ALT;
    }
    if qt & control != 0 {
        bits |= DOM_CONTROL;
    }
    if qt & meta != 0 {
        bits |= DOM_META;
    }
    if qt & QT_SHIFT != 0 {
        bits |= DOM_SHIFT;
    }
    bits
}

const NAMED: [(u32, &str, &str, i64); 31] = [
    (0x0100_0000, "Escape", "Escape", 27),
    (0x0100_0001, "Tab", "Tab", 9),
    (0x0100_0002, "Tab", "Tab", 9),
    (0x0100_0003, "Backspace", "Backspace", 8),
    (0x0100_0004, "Enter", "Enter", 13),
    (0x0100_0005, "Enter", "NumpadEnter", 13),
    (0x0100_0006, "Insert", "Insert", 45),
    (0x0100_0007, "Delete", "Delete", 46),
    (0x0100_0008, "Pause", "Pause", 19),
    (0x0100_0010, "Home", "Home", 36),
    (0x0100_0011, "End", "End", 35),
    (0x0100_0012, "ArrowLeft", "ArrowLeft", 37),
    (0x0100_0013, "ArrowUp", "ArrowUp", 38),
    (0x0100_0014, "ArrowRight", "ArrowRight", 39),
    (0x0100_0015, "ArrowDown", "ArrowDown", 40),
    (0x0100_0016, "PageUp", "PageUp", 33),
    (0x0100_0017, "PageDown", "PageDown", 34),
    (0x0100_0020, "Shift", "ShiftLeft", 16),
    (0x0100_0021, "Control", "ControlLeft", 17),
    (0x0100_0022, "Meta", "MetaLeft", 91),
    (0x0100_0023, "Alt", "AltLeft", 18),
    (0x0100_0024, "CapsLock", "CapsLock", 20),
    (0x20, " ", "Space", 32),
    (0x3b, ";", "Semicolon", 186),
    (0x3d, "=", "Equal", 187),
    (0x2c, ",", "Comma", 188),
    (0x2d, "-", "Minus", 189),
    (0x2e, ".", "Period", 190),
    (0x2f, "/", "Slash", 191),
    (0x60, "`", "Backquote", 192),
    (0x27, "'", "Quote", 222),
];

const BRACKETS: [(u32, &str, &str, i64); 3] = [
    (0x5b, "[", "BracketLeft", 219),
    (0x5c, "\\", "Backslash", 220),
    (0x5d, "]", "BracketRight", 221),
];

const QT_F1: u32 = 0x0100_0030;
const FUNCTION_KEYS: u32 = 12;
const DOM_F1: i64 = 112;

fn named(qt_key: u32) -> Option<(&'static str, &'static str, i64)> {
    NAMED
        .iter()
        .chain(BRACKETS.iter())
        .find(|(qt, ..)| *qt == qt_key)
        .map(|&(_, key, code, key_code)| (key, code, key_code))
}

const fn modifier_key(qt_key: u32) -> bool {
    matches!(qt_key, 0x0100_0020..=0x0100_0023)
}

pub fn dom_key(qt_key: u32, text: &str, keypad: bool) -> DomKey {
    let typed = text.chars().next().filter(|c| !c.is_control());
    let location = if keypad {
        LOCATION_KEYPAD
    } else if modifier_key(qt_key) {
        LOCATION_LEFT
    } else {
        LOCATION_STANDARD
    };
    if let Some(letter) = char::from_u32(qt_key).filter(char::is_ascii_uppercase) {
        return DomKey {
            key: typed.map_or_else(|| letter.to_ascii_lowercase().to_string(), String::from),
            code: format!("Key{letter}"),
            key_code: i64::from(qt_key),
            location,
        };
    }
    if let Some(digit) = char::from_u32(qt_key).filter(char::is_ascii_digit) {
        return DomKey {
            key: typed.map_or_else(|| digit.to_string(), String::from),
            code: if keypad {
                format!("Numpad{digit}")
            } else {
                format!("Digit{digit}")
            },
            key_code: i64::from(qt_key),
            location,
        };
    }
    if (QT_F1..QT_F1 + FUNCTION_KEYS).contains(&qt_key) {
        let n = qt_key - QT_F1;
        return DomKey {
            key: format!("F{}", n + 1),
            code: format!("F{}", n + 1),
            key_code: DOM_F1 + i64::from(n),
            location,
        };
    }
    if let Some((key, code, key_code)) = named(qt_key) {
        let key = if key.chars().count() == 1 {
            typed.map_or_else(|| key.to_owned(), String::from)
        } else {
            key.to_owned()
        };
        return DomKey {
            key,
            code: code.to_owned(),
            key_code,
            location,
        };
    }
    DomKey {
        key: typed.map_or_else(|| "Unidentified".to_owned(), String::from),
        code: String::new(),
        key_code: 0,
        location,
    }
}

pub fn typed_text(key: &DomKey, text: &str, modifiers: i64) -> String {
    if key.key == "Enter" {
        return "\r".to_owned();
    }
    let commanded = modifiers & (DOM_CONTROL | DOM_META) != 0;
    if commanded || text.chars().any(char::is_control) {
        return String::new();
    }
    text.to_owned()
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::{dom_key, dom_modifiers, typed_text};

    proptest! {
        #[test]
        fn a_letter_keeps_its_code_and_types_its_text(letter in 0u32..26, shifted in any::<bool>()) {
            let upper = char::from_u32(u32::from(b'A') + letter).unwrap();
            let text = if shifted { upper.to_string() } else { upper.to_ascii_lowercase().to_string() };
            let key = dom_key(u32::from(upper), &text, false);
            prop_assert_eq!(&key.code, &format!("Key{upper}"));
            prop_assert_eq!(key.key_code, i64::from(u32::from(upper)));
            prop_assert_eq!(&key.key, &text);
            prop_assert_eq!(typed_text(&key, &text, 0), text);
        }

        #[test]
        fn modifiers_map_one_to_one(bits in 0u32..16, mac in any::<bool>()) {
            let qt = ((bits & 1) * 0x0200_0000)
                | (((bits >> 1) & 1) * 0x0400_0000)
                | (((bits >> 2) & 1) * 0x0800_0000)
                | (((bits >> 3) & 1) * 0x1000_0000);
            let dom = dom_modifiers(qt, mac);
            prop_assert_eq!(dom.count_ones(), bits.count_ones());
            prop_assert_eq!(dom & 8 != 0, bits & 1 != 0);
            prop_assert_eq!(dom & 1 != 0, bits & 4 != 0);
        }

        #[test]
        fn a_command_types_nothing(letter in 0u32..26, command in 1i64..8) {
            let upper = char::from_u32(u32::from(b'A') + letter).unwrap();
            let key = dom_key(u32::from(upper), "x", false);
            prop_assert_eq!(typed_text(&key, "x", command & 6), if command & 6 == 0 { "x" } else { "" });
        }
    }

    #[test]
    fn named_keys_read_as_the_browser_names_them() {
        let enter = dom_key(0x0100_0004, "\r", false);
        assert_eq!(
            (enter.key.as_str(), enter.code.as_str(), enter.key_code),
            ("Enter", "Enter", 13)
        );
        assert_eq!(typed_text(&enter, "\r", 0), "\r");
        let left = dom_key(0x0100_0012, "", false);
        assert_eq!((left.key.as_str(), left.key_code), ("ArrowLeft", 37));
        assert_eq!(dom_key(0x0100_0031, "", false).key, "F2");
    }
}

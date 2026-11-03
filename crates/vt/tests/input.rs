use opsis_vt::{CellPoint, KeyAction, KeyMods, MouseAction, MouseButton, Terminal, VtError};
use proptest::prelude::*;

const COLS: u16 = 80;
const ROWS: u16 = 24;
const ESC: u8 = 0x1b;

fn terminal(setup: &str) -> Terminal {
    let mut t = Terminal::new(COLS, ROWS, 100).expect("a terminal");
    t.set_cell_size(10, 20);
    t.feed(setup.as_bytes());
    t
}

const fn mods(shift: bool, ctrl: bool, alt: bool) -> KeyMods {
    KeyMods {
        shift,
        ctrl,
        alt,
        super_key: false,
    }
}

fn press(t: &mut Terminal, key: &str, with: KeyMods) -> Vec<u8> {
    t.key_bytes(key, with, KeyAction::Press)
        .expect("a known key")
}

fn letter() -> impl Strategy<Value = u8> {
    b'a'..=b'z'
}

fn cell() -> impl Strategy<Value = CellPoint> {
    (0..COLS, 0..ROWS).prop_map(|(col, row)| CellPoint { col, row })
}

proptest! {
    #[test]
    fn types_printable_characters_as_themselves(ch in 0x20u8..0x7f) {
        let mut t = terminal("");
        let text = char::from(ch).to_string();
        prop_assert_eq!(press(&mut t, &text, KeyMods::NONE), text.into_bytes());
    }

    #[test]
    fn shifts_letters_to_capitals(ch in letter()) {
        let mut t = terminal("");
        let typed = press(&mut t, &char::from(ch).to_string(), mods(true, false, false));
        prop_assert_eq!(typed, vec![ch.to_ascii_uppercase()]);
    }

    #[test]
    fn turns_control_letters_into_control_bytes(ch in letter().prop_filter("ctrl+i and ctrl+m stay apart from tab and enter", |c| *c != b'i' && *c != b'm')) {
        let mut t = terminal("");
        prop_assert_eq!(press(&mut t, &char::from(ch).to_string(), mods(false, true, false)), vec![ch & 0x1f]);
    }

    #[test]
    fn prefixes_alt_letters_with_escape_when_the_program_asks(ch in letter()) {
        let mut t = terminal("\x1b[?1036h");
        prop_assert_eq!(press(&mut t, &char::from(ch).to_string(), mods(false, false, true)), vec![ESC, ch]);
    }

    #[test]
    fn writes_arrows_in_the_cursor_mode_the_program_set(arrow in 0usize..4, application in any::<bool>()) {
        let (name, last) = [("ArrowUp", b'A'), ("ArrowDown", b'B'), ("ArrowRight", b'C'), ("ArrowLeft", b'D')][arrow];
        let mut t = terminal(if application { "\x1b[?1h" } else { "" });
        let introducer = if application { b'O' } else { b'[' };
        prop_assert_eq!(press(&mut t, name, KeyMods::NONE), vec![ESC, introducer, last]);
    }

    #[test]
    fn reports_clicks_at_their_cell_in_the_sgr_format(at in cell(), right in any::<bool>()) {
        let mut t = terminal("\x1b[?1000h\x1b[?1006h");
        let (button, code) = if right { (MouseButton::Right, 2) } else { (MouseButton::Left, 0) };
        let down = t.mouse_bytes(MouseAction::Press, button, KeyMods::NONE, at);
        let up = t.mouse_bytes(MouseAction::Release, button, KeyMods::NONE, at);
        let place = format!("{code};{};{}", at.col + 1, at.row + 1);
        prop_assert_eq!(String::from_utf8(down).unwrap(), format!("\x1b[<{place}M"));
        prop_assert_eq!(String::from_utf8(up).unwrap(), format!("\x1b[<{place}m"));
    }

    #[test]
    fn reports_clicks_in_the_old_format_until_the_program_picks_another(at in (0u16..80, 0u16..24)) {
        let at = CellPoint { col: at.0, row: at.1 };
        let mut t = terminal("\x1b[?1000h");
        let down = t.mouse_bytes(MouseAction::Press, MouseButton::Left, KeyMods::NONE, at);
        let col = u8::try_from(32 + 1 + at.col).unwrap();
        let row = u8::try_from(32 + 1 + at.row).unwrap();
        prop_assert_eq!(down, vec![ESC, b'[', b'M', 32, col, row]);
    }

    #[test]
    fn sends_nothing_from_the_mouse_until_the_program_turns_it_on(at in cell()) {
        let mut t = terminal("");
        prop_assert!(!t.mouse_reporting());
        prop_assert_eq!(t.mouse_bytes(MouseAction::Press, MouseButton::Left, KeyMods::NONE, at), b"");
    }

    #[test]
    fn frames_a_bracketed_paste_and_never_lets_it_close_early(text in ".{0,80}") {
        let mut t = terminal("\x1b[?2004h");
        let pasted = t.paste_bytes(&text);
        let open = b"\x1b[200~";
        let close = b"\x1b[201~";
        prop_assert!(pasted.starts_with(open) && pasted.ends_with(close));
        let inner = &pasted[open.len()..pasted.len() - close.len()];
        prop_assert!(!inner.contains(&ESC));
    }

    #[test]
    fn pastes_plain_text_without_escapes_or_line_feeds(text in ".{0,80}") {
        let mut t = terminal("");
        let pasted = t.paste_bytes(&text);
        prop_assert!(!pasted.contains(&ESC) && !pasted.contains(&b'\n'));
        prop_assert_eq!(t.paste_is_safe(&text), !text.contains(['\r', '\n']) && !text.contains("\x1b[201~"));
    }
}

#[test]
fn writes_the_named_keys_programs_expect() {
    let mut t = terminal("");
    for (name, bytes) in [
        ("Enter", &b"\r"[..]),
        ("Tab", b"\t"),
        ("Backspace", b"\x7f"),
        ("Escape", b"\x1b"),
        ("Space", b" "),
        ("Home", b"\x1b[H"),
        ("Delete", b"\x1b[3~"),
        ("F5", b"\x1b[15~"),
    ] {
        assert_eq!(press(&mut t, name, KeyMods::NONE), bytes, "{name}");
    }
    assert_eq!(
        t.key_bytes("a", KeyMods::NONE, KeyAction::Release).unwrap(),
        b""
    );
    assert!(matches!(
        t.key_bytes("NoSuchKey", KeyMods::NONE, KeyAction::Press),
        Err(VtError::UnknownKey(name)) if name == "NoSuchKey"
    ));
}

#[test]
fn follows_the_keyboard_protocols_a_program_turns_on() {
    let mut kitty = terminal("\x1b[>1u");
    assert_eq!(press(&mut kitty, "Escape", KeyMods::NONE), b"\x1b[27u");
    let mut legacy = terminal("");
    assert_eq!(
        press(&mut legacy, "i", mods(false, true, false)),
        b"\x1b[105;5u"
    );
    assert_eq!(
        press(&mut kitty, "c", mods(false, true, false)),
        b"\x1b[99;5u"
    );
    let mut other_keys = terminal("\x1b[>4;2m");
    assert_eq!(
        press(&mut other_keys, "a", mods(false, true, false)),
        b"\x1b[27;5;97~"
    );
}

#[test]
fn turns_the_wheel_and_drags_only_where_the_program_listens() {
    let at = CellPoint { col: 9, row: 4 };
    let mut t = terminal("\x1b[?1000h\x1b[?1006h");
    assert_eq!(
        t.mouse_bytes(MouseAction::Press, MouseButton::WheelUp, KeyMods::NONE, at),
        b"\x1b[<64;10;5M"
    );
    assert_eq!(
        t.mouse_bytes(
            MouseAction::Press,
            MouseButton::WheelDown,
            KeyMods::NONE,
            at
        ),
        b"\x1b[<65;10;5M"
    );
    assert_eq!(
        t.mouse_bytes(MouseAction::Motion, MouseButton::None, KeyMods::NONE, at),
        b""
    );
    let mut any = terminal("\x1b[?1003h\x1b[?1006h");
    assert_eq!(
        any.mouse_bytes(MouseAction::Motion, MouseButton::None, KeyMods::NONE, at),
        b"\x1b[<35;10;5M"
    );
    assert_eq!(
        any.mouse_bytes(MouseAction::Motion, MouseButton::None, KeyMods::NONE, at),
        b""
    );
}

#[test]
fn reports_focus_and_the_pointer_shape_the_program_asks_for() {
    let mut t = terminal("");
    assert_eq!(t.focus_bytes(true), b"");
    t.feed(b"\x1b[?1004h");
    assert_eq!(t.focus_bytes(true), b"\x1b[I");
    assert_eq!(t.focus_bytes(false), b"\x1b[O");
    assert_eq!(t.pointer_shape(), "text");
    t.feed(b"\x1b]22;pointer\x1b\\");
    assert_eq!(t.pointer_shape(), "pointer");
    t.feed(b"\x1b]22;ew-resize\x1b\\");
    assert_eq!(t.pointer_shape(), "ew-resize");
}

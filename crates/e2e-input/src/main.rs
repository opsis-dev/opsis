use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::os::fd::AsFd;
use std::process::ExitCode;
use std::time::{Duration, Instant};

use wayland_client::globals::{GlobalList, GlobalListContents, registry_queue_init};
use wayland_client::protocol::wl_keyboard::{KeyState, KeymapFormat};
use wayland_client::protocol::wl_pointer::ButtonState;
use wayland_client::protocol::wl_registry::WlRegistry;
use wayland_client::protocol::wl_seat::WlSeat;
use wayland_client::{Connection, Dispatch, EventQueue, QueueHandle, delegate_noop};
use wayland_protocols_misc::zwp_virtual_keyboard_v1::client::zwp_virtual_keyboard_manager_v1::ZwpVirtualKeyboardManagerV1;
use wayland_protocols_misc::zwp_virtual_keyboard_v1::client::zwp_virtual_keyboard_v1::ZwpVirtualKeyboardV1;
use wayland_protocols_wlr::virtual_pointer::v1::client::zwlr_virtual_pointer_manager_v1::ZwlrVirtualPointerManagerV1;
use wayland_protocols_wlr::virtual_pointer::v1::client::zwlr_virtual_pointer_v1::ZwlrVirtualPointerV1;

const KEYMAP: &str = "xkb_keymap {\n  xkb_keycodes { include \"evdev+aliases(qwerty)\" };\n  xkb_types { include \"complete\" };\n  xkb_compat { include \"complete\" };\n  xkb_symbols { include \"pc+us+inet(evdev)\" };\n};\n";
const SETTLE: Duration = Duration::from_secs(1);
const POLL: Duration = Duration::from_millis(20);
const BUTTON_LEFT: u32 = 0x110;
const BUTTON_RIGHT: u32 = 0x111;
const SHIFT: u32 = 1;
const CONTROL: u32 = 1 << 2;
const ALT: u32 = 1 << 3;
const SUPER: u32 = 1 << 6;
const MODIFIER_KEYS: [(u32, u32); 4] = [(SHIFT, 42), (CONTROL, 29), (ALT, 56), (SUPER, 125)];
const KEY_SPACE: u32 = 57;
const ROWS: [(&str, &str, u32); 4] = [
    ("1234567890-=", "!@#$%^&*()_+", 2),
    ("qwertyuiop[]", "QWERTYUIOP{}", 16),
    ("asdfghjkl;'`", "ASDFGHJKL:\"~", 30),
    ("\\zxcvbnm,./", "|ZXCVBNM<>?", 43),
];

#[derive(Debug)]
struct State;

impl Dispatch<WlRegistry, GlobalListContents> for State {
    fn event(
        _: &mut Self,
        _: &WlRegistry,
        _: <WlRegistry as wayland_client::Proxy>::Event,
        _: &GlobalListContents,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

delegate_noop!(State: ignore WlSeat);
delegate_noop!(State: ZwlrVirtualPointerManagerV1);
delegate_noop!(State: ZwlrVirtualPointerV1);
delegate_noop!(State: ZwpVirtualKeyboardManagerV1);
delegate_noop!(State: ZwpVirtualKeyboardV1);

#[derive(Debug, thiserror::Error)]
enum InputError {
    #[error("can't reach the compositor: {0}")]
    Connect(#[from] wayland_client::ConnectError),
    #[error("the compositor lacks {0}")]
    Missing(&'static str),
    #[error("talking to the compositor failed: {0}")]
    Dispatch(#[from] wayland_client::DispatchError),
    #[error("reading the compositor's globals failed: {0}")]
    Globals(#[from] wayland_client::globals::GlobalError),
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("can't read the request “{0}”")]
    Request(String),
}

#[derive(Debug)]
struct Devices {
    pointer: ZwlrVirtualPointerV1,
    keyboard: ZwpVirtualKeyboardV1,
    width: u32,
    height: u32,
    started: Instant,
}

impl Devices {
    fn now(&self) -> u32 {
        u32::try_from(self.started.elapsed().as_millis()).unwrap_or(u32::MAX)
    }

    fn move_to(&self, x: u32, y: u32) {
        self.pointer.motion_absolute(
            self.now(),
            x.min(self.width.saturating_sub(1)),
            y.min(self.height.saturating_sub(1)),
            self.width,
            self.height,
        );
        self.pointer.frame();
    }

    fn button(&self, button: u32, state: ButtonState) {
        self.pointer.button(self.now(), button, state);
        self.pointer.frame();
    }

    fn key(&self, code: u32, mask: u32) {
        let held: Vec<u32> = MODIFIER_KEYS
            .into_iter()
            .filter(|(bit, _)| mask & bit != 0)
            .map(|(_, key)| key)
            .collect();
        for modifier in &held {
            self.keyboard
                .key(self.now(), *modifier, KeyState::Pressed.into());
        }
        self.keyboard.modifiers(mask, 0, 0, 0);
        self.keyboard
            .key(self.now(), code, KeyState::Pressed.into());
        self.keyboard
            .key(self.now(), code, KeyState::Released.into());
        for modifier in held.iter().rev() {
            self.keyboard
                .key(self.now(), *modifier, KeyState::Released.into());
        }
        self.keyboard.modifiers(0, 0, 0, 0);
    }
}

fn char_key(c: char) -> Option<(u32, bool)> {
    if c == ' ' {
        return Some((KEY_SPACE, false));
    }
    ROWS.into_iter().find_map(|(plain, shifted, first)| {
        plain
            .chars()
            .zip(shifted.chars())
            .zip(first..)
            .find_map(|((lower, upper), code)| {
                if c == lower {
                    Some((code, false))
                } else if c == upper {
                    Some((code, true))
                } else {
                    None
                }
            })
    })
}

fn function_key(name: &str) -> Option<u32> {
    let number: u32 = name.strip_prefix('F')?.parse().ok()?;
    match number {
        1..=10 => Some(58 + number),
        11 => Some(87),
        12 => Some(88),
        _ => None,
    }
}

fn named_key(name: &str) -> Option<u32> {
    let code = match name {
        "Escape" => 1,
        "BackSpace" => 14,
        "Tab" => 15,
        "Return" => 28,
        "space" => KEY_SPACE,
        "Home" => 102,
        "Up" => 103,
        "Prior" => 104,
        "Left" => 105,
        "Right" => 106,
        "End" => 107,
        "Down" => 108,
        "Next" => 109,
        "Insert" => 110,
        "Delete" => 111,
        "Menu" => 127,
        _ => {
            let mut chars = name.chars();
            return match (chars.next(), chars.next()) {
                (Some(only), None) => char_key(only.to_ascii_lowercase()).map(|(code, _)| code),
                _ => function_key(name),
            };
        }
    };
    Some(code)
}

fn combo(text: &str) -> Option<(u32, u32)> {
    let mut parts: Vec<&str> = text.split('+').collect();
    let key = parts.pop()?;
    let mut mask = 0;
    for part in parts {
        mask |= match part {
            "shift" => SHIFT,
            "ctrl" => CONTROL,
            "alt" => ALT,
            "super" => SUPER,
            _ => return None,
        };
    }
    Some((named_key(key)?, mask))
}

fn number(word: Option<&str>, line: &str) -> Result<u32, InputError> {
    word.and_then(|w| w.parse().ok())
        .ok_or_else(|| InputError::Request(line.to_owned()))
}

fn run(devices: &Devices, line: &str) -> Result<(), InputError> {
    let mut words = line.split_whitespace();
    let verb = words.next().unwrap_or_default();
    match verb {
        "move" | "click" | "right-click" | "press" | "release" => {
            let x = number(words.next(), line)?;
            let y = number(words.next(), line)?;
            devices.move_to(x, y);
            let button = if verb == "right-click" {
                BUTTON_RIGHT
            } else {
                BUTTON_LEFT
            };
            if matches!(verb, "click" | "right-click" | "press") {
                devices.button(button, ButtonState::Pressed);
            }
            if matches!(verb, "click" | "right-click" | "release") {
                devices.button(button, ButtonState::Released);
            }
        }
        "key" => {
            let (code, mask) = words
                .next()
                .and_then(combo)
                .ok_or_else(|| InputError::Request(line.to_owned()))?;
            devices.key(code, mask);
        }
        "type" => {
            let text = line.strip_prefix("type ").unwrap_or_default();
            for c in text.chars() {
                let (code, shifted) =
                    char_key(c).ok_or_else(|| InputError::Request(line.to_owned()))?;
                devices.key(code, if shifted { SHIFT } else { 0 });
            }
        }
        _ => return Err(InputError::Request(line.to_owned())),
    }
    Ok(())
}

fn attach(
    queue: &mut EventQueue<State>,
    globals: &GlobalList,
    width: u32,
    height: u32,
) -> Result<Devices, InputError> {
    let qh = queue.handle();
    let seat: WlSeat = globals
        .bind(&qh, 1..=7, ())
        .map_err(|_| InputError::Missing("a seat"))?;
    let pointers: ZwlrVirtualPointerManagerV1 = globals
        .bind(&qh, 1..=2, ())
        .map_err(|_| InputError::Missing("virtual pointers"))?;
    let keyboards: ZwpVirtualKeyboardManagerV1 = globals
        .bind(&qh, 1..=1, ())
        .map_err(|_| InputError::Missing("virtual keyboards"))?;
    let pointer = pointers.create_virtual_pointer(Some(&seat), &qh, ());
    let keyboard = keyboards.create_virtual_keyboard(&seat, &qh, ());
    let mut keymap = tempfile::tempfile()?;
    keymap.write_all(KEYMAP.as_bytes())?;
    keymap.write_all(&[0])?;
    let size = u32::try_from(KEYMAP.len() + 1).unwrap_or(u32::MAX);
    keyboard.keymap(KeymapFormat::XkbV1.into(), keymap.as_fd(), size);
    queue.roundtrip(&mut State)?;
    Ok(Devices {
        pointer,
        keyboard,
        width,
        height,
        started: Instant::now(),
    })
}

fn follow(
    queue: &mut EventQueue<State>,
    devices: &Devices,
    requests: &mut File,
) -> Result<(), InputError> {
    let mut read = 0;
    let mut pending = String::new();
    loop {
        requests.seek(SeekFrom::Start(read))?;
        let mut fresh = String::new();
        let got = requests.read_to_string(&mut fresh)?;
        read += u64::try_from(got).unwrap_or_default();
        pending.push_str(&fresh);
        while let Some(end) = pending.find('\n') {
            let line: String = pending.drain(..=end).collect();
            let line = line.trim();
            if line == "quit" {
                return Ok(());
            }
            if !line.is_empty() {
                run(devices, line)?;
                queue.roundtrip(&mut State)?;
                println!("done {line}");
            }
        }
        std::thread::sleep(POLL);
    }
}

fn start() -> Result<(), InputError> {
    let mut args = std::env::args().skip(1);
    let usage = "usage: opsis-e2e-input <requests file> <width> <height>";
    let path = args
        .next()
        .ok_or_else(|| InputError::Request(usage.to_owned()))?;
    let width = number(args.next().as_deref(), usage)?;
    let height = number(args.next().as_deref(), usage)?;
    let connection = Connection::connect_to_env()?;
    let (globals, mut queue) = registry_queue_init::<State>(&connection)?;
    let devices = attach(&mut queue, &globals, width, height)?;
    std::thread::sleep(SETTLE);
    queue.roundtrip(&mut State)?;
    let mut requests = File::options()
        .read(true)
        .create(true)
        .append(true)
        .open(path)?;
    println!("ready");
    follow(&mut queue, &devices, &mut requests)
}

fn main() -> ExitCode {
    match start() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("opsis-e2e-input: {error}");
            ExitCode::FAILURE
        }
    }
}

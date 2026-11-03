use std::path::{Path, PathBuf};

use opsis_compile::nodes::camera::{SHOT_TYPE, SPEED_RAMP_TYPE, WINDOW_FRAME_TYPE};
use opsis_compile::nodes::terminal::TERMINAL_LOOK_TYPE;
use opsis_terminal::tape::{
    Command, Key, Tape, WAIT_TIMEOUT_MS, parse, parse_with_sources, session_tape, speed_entry,
    tape_steps, write,
};
use opsis_types::terminal::{LookIn, ShellChoice, TerminalAction};
use pretty_assertions::assert_eq;
use proptest::prelude::*;

fn tapes(dir: &Path, found: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).expect("the tapes folder reads") {
        let path = entry.expect("an entry reads").path();
        if path.is_dir() {
            tapes(&path, found);
        } else if path.extension().is_some_and(|e| e == "tape") {
            found.push(path);
        }
    }
}

fn example_tapes() -> Vec<PathBuf> {
    let mut found = Vec::new();
    tapes(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/tapes"),
        &mut found,
    );
    found.sort();
    found
}

#[test]
fn every_example_tape_parses_and_writes_back() {
    let tapes = example_tapes();
    assert!(tapes.len() > 100, "found {} tapes", tapes.len());
    for path in tapes {
        let text = std::fs::read_to_string(&path).expect("the tape reads");
        let parsed = parse(&text);
        if path.parent().is_some_and(|p| p.ends_with("errors")) && path.ends_with("parser.tape") {
            assert!(parsed.is_err(), "{} should fail", path.display());
            continue;
        }
        let tape = parsed.unwrap_or_else(|e| panic!("{}: {}", path.display(), e[0]));
        assert_eq!(parse(&write(&tape)), Ok(tape), "{}", path.display());
    }
}

#[test]
fn errors_name_their_line_and_column() {
    let text = "\nType Enter\nType \"echo 'Hello, World!'\" Enter\nFoo\nSleep Bar";
    let errors = parse(text).expect_err("the tape has errors");
    let places: Vec<(usize, usize)> = errors.iter().map(|e| (e.line, e.column)).collect();
    assert_eq!(places, vec![(2, 6), (4, 1), (5, 1), (5, 7)]);
    assert_eq!(
        errors[1].to_string(),
        "Line 4, column 1: “Foo” isn't a command"
    );
}

#[test]
fn a_bare_word_is_typed_like_a_string() {
    let tape = parse("Type o\nType hello world\n").expect("the tape parses");
    assert_eq!(
        tape.commands,
        vec![
            Command::Type {
                speed_ms: None,
                text: "o".to_owned()
            },
            Command::Type {
                speed_ms: None,
                text: "hello world".to_owned()
            },
        ]
    );
}

#[test]
fn times_read_in_seconds_unless_a_unit_says_otherwise() {
    let tape = parse("Sleep 2\nSleep 500ms\nSleep 1.5s\nType@0.1 \"x\"\nSet TypingSpeed 75ms\n")
        .expect("the tape parses");
    assert_eq!(
        tape.commands,
        vec![
            Command::Sleep { ms: 2000 },
            Command::Sleep { ms: 500 },
            Command::Sleep { ms: 1500 },
            Command::Type {
                speed_ms: Some(100),
                text: "x".to_owned()
            },
            Command::Set {
                name: "TypingSpeed".to_owned(),
                value: "75ms".to_owned()
            },
        ]
    );
}

fn text() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9 .,:;!?#$&*()=+<>{}|~/\\-\"'`]{0,24}".prop_filter("not every quote", |t| {
        !(t.contains('"') && t.contains('\'') && t.contains('`'))
    })
}

fn word() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,8}"
}

fn time() -> impl Strategy<Value = u32> {
    0u32..100_000
}

fn key() -> impl Strategy<Value = Key> {
    prop::sample::select(Key::ALL.to_vec())
}

fn setting() -> impl Strategy<Value = Command> {
    prop_oneof![
        (8u32..64).prop_map(|n| ("FontSize", n.to_string())),
        time().prop_map(|n| ("TypingSpeed", format!("{n}ms"))),
        time().prop_map(|n| ("WaitTimeout", format!("{n}ms"))),
        Just(("Theme", "Catppuccin Mocha".to_owned())),
        Just(("Theme", "{\"background\": \"#000000\"}".to_owned())),
        Just(("WindowBar", "Colorful".to_owned())),
        Just(("LoopOffset", "20%".to_owned())),
        Just(("LineHeight", "1.2".to_owned())),
        Just(("Shell", "bash".to_owned())),
    ]
    .prop_map(|(name, value)| Command::Set {
        name: name.to_owned(),
        value,
    })
}

fn command() -> impl Strategy<Value = Command> {
    prop_oneof![
        (prop::option::of(time()), text())
            .prop_map(|(speed_ms, text)| Command::Type { speed_ms, text }),
        (key(), prop::option::of(time()), 1u32..20).prop_map(|(key, speed_ms, repeat)| {
            Command::Key {
                key,
                speed_ms,
                repeat,
            }
        }),
        prop::collection::vec(prop_oneof![word(), Just("Alt".to_owned())], 1..3)
            .prop_map(|keys| Command::Ctrl { keys }),
        word().prop_map(|key| Command::Alt { key }),
        word().prop_map(|key| Command::Shift { key }),
        time().prop_map(|ms| Command::Sleep { ms }),
        Just(Command::Hide),
        Just(Command::Show),
        (
            prop::sample::select(vec![LookIn::Line, LookIn::Screen]),
            prop::option::of(time()),
            prop::option::of("[a-z>$ ]{1,8}")
        )
            .prop_map(|(scope, timeout_ms, pattern)| Command::Wait {
                scope,
                timeout_ms,
                pattern
            }),
        word().prop_map(|program| Command::Require { program }),
        text().prop_map(|path| Command::Screenshot { path }),
        text().prop_map(|text| Command::Copy { text }),
        Just(Command::Paste),
        (word(), text()).prop_map(|(name, value)| Command::Env { name, value }),
        text().prop_map(|path| Command::Output { path }),
        setting(),
    ]
}

proptest! {
    #[test]
    fn written_tapes_parse_back_to_themselves(commands in prop::collection::vec(command(), 0..24)) {
        let tape = Tape { commands };
        prop_assert_eq!(parse(&write(&tape)), Ok(tape));
    }
}

#[test]
fn settings_shape_the_session_and_the_style() {
    let tape = parse(
        r#"Set Shell "bash"
Set FontSize 32
Set Width 1200
Set Height 600
Set Theme "Dracula"
Set WindowBar Colorful
Set BorderRadius 10
Set Margin 60
Set PlaybackSpeed 2
Set TypingSpeed 75ms
Set FontFamily "Fira Code"
Set Framerate 24
Output demo.gif
Type "ls"
Set TypingSpeed 10ms
Type "pwd"
"#,
    )
    .expect("the tape parses");
    let mapped = tape_steps(&tape, "play");
    assert_eq!(mapped.session.shell, ShellChoice::Own);
    assert_eq!(mapped.session.typing_ms, 75);
    assert_eq!((mapped.session.cols, mapped.session.rows), (47, 10));
    assert_eq!(mapped.speed, Some(2.0));
    assert_eq!(
        mapped.ignored,
        vec!["Set FontFamily", "Set Framerate", "Output"]
    );
    let kinds: Vec<&str> = mapped.style.iter().map(|e| e.kind.as_str()).collect();
    assert_eq!(
        kinds,
        vec![TERMINAL_LOOK_TYPE, WINDOW_FRAME_TYPE, SHOT_TYPE]
    );
    assert_eq!(mapped.style[0].params["theme"], "Dracula");
    assert_eq!(mapped.style[0].params["fontSize"], 32);
    assert_eq!(mapped.style[1].params["frameTheme"], "dark");
    assert_eq!(mapped.style[2].params["cornerRadius"], 10);
    assert_eq!(mapped.style[2].params["windowSize"], 90);
    let types: Vec<&TerminalAction> = mapped.session.steps.iter().map(|s| &s.action).collect();
    assert_eq!(
        types,
        vec![
            &TerminalAction::Type {
                text: "ls".to_owned(),
                speed_ms: None
            },
            &TerminalAction::Type {
                text: "pwd".to_owned(),
                speed_ms: Some(10)
            },
        ]
    );
    assert_eq!(mapped.session.steps[0].label, "Type \"ls\"");
    assert_eq!(speed_entry("play", 2.0, 4000).kind, SPEED_RAMP_TYPE);
}

#[test]
fn a_theme_written_out_belongs_to_the_take_and_an_unknown_name_warns() {
    let tape = parse(
        "Set Theme { \"name\": \"Mine\", \"background\": \"#101010\", \"red\": \"#ff0000\" }\n",
    )
    .expect("the tape parses");
    let mapped = tape_steps(&tape, "play");
    let theme = mapped.theme.expect("the tape has its own theme");
    assert_eq!(
        (
            theme.name.as_str(),
            theme.bg.as_str(),
            theme.palette[1].as_str()
        ),
        ("Mine", "#101010", "#ff0000")
    );
    assert_eq!(mapped.style.len(), 0);
    let unknown = tape_steps(
        &parse("Set Theme \"No such theme\"\n").expect("parses"),
        "play",
    );
    assert_eq!(unknown.warnings.len(), 1);
}

#[test]
fn a_wait_without_a_pattern_waits_for_the_prompt() {
    let plain = tape_steps(&parse("Wait\nWait@3s\n").expect("parses"), "play");
    assert_eq!(
        plain.session.steps[0].action,
        TerminalAction::WaitPrompt {
            limit_ms: WAIT_TIMEOUT_MS
        }
    );
    assert_eq!(
        plain.session.steps[1].action,
        TerminalAction::WaitPrompt { limit_ms: 3000 }
    );
    let patterned = tape_steps(
        &parse("Set WaitPattern \"done\"\nSet WaitTimeout 2s\nWait+Screen\n").expect("parses"),
        "play",
    );
    assert_eq!(
        patterned.session.steps[0].action,
        TerminalAction::WaitText {
            pattern: "done".to_owned(),
            scope: LookIn::Screen,
            limit_ms: 2000
        }
    );
}

#[test]
fn commands_that_map_both_ways_come_back_from_the_steps() {
    let text = r#"Type "echo hello"
Enter
Backspace 3
Up
Ctrl+C
Ctrl+Alt+X
Alt+b
Shift+Tab
Sleep 500ms
Hide
Show
Wait
Wait+Screen@3000ms /ready/
Copy "copied"
Paste
Env HELLO "world"
Require git
ScrollUp 4
ScrollDown 2
"#;
    let tape = parse(text).expect("the tape parses");
    let mapped = tape_steps(&tape, "play");
    let written = session_tape(&mapped.session);
    assert_eq!(written.skipped, Vec::<String>::new());
    let commands: Vec<Command> = written
        .tape
        .commands
        .into_iter()
        .filter(|c| !matches!(c, Command::Set { .. }))
        .collect();
    assert_eq!(commands, tape.commands);
}

#[test]
fn sourced_tapes_play_inline() {
    let read = |path: &str| match path {
        "setup.tape" => Ok("Type \"cd demo\"\nOutput other.gif\nEnter\n".to_owned()),
        "nested.tape" => Ok("Source \"setup.tape\"\n".to_owned()),
        "broken.tape" => Ok("Foo\n".to_owned()),
        _ => Err("Not in the project folder".to_owned()),
    };
    let sourced =
        parse_with_sources("Source \"setup.tape\"\nType \"ls\"\n", &read).expect("it parses");
    assert_eq!(sourced.files, vec!["setup.tape"]);
    assert_eq!(sourced.tape.commands.len(), 3);
    assert!(
        !sourced
            .tape
            .commands
            .iter()
            .any(|c| matches!(c, Command::Output { .. }))
    );
    for (tape, message) in [
        (
            "Source \"nested.tape\"\n",
            "nested.tape: A tape it plays plays another tape",
        ),
        (
            "Source \"broken.tape\"\n",
            "broken.tape: Line 1, column 1: “Foo” isn't a command",
        ),
        (
            "Source \"missing.tape\"\n",
            "missing.tape: Not in the project folder",
        ),
    ] {
        let errors = parse_with_sources(tape, &read).expect_err("it fails");
        assert_eq!(errors[0].message, message);
    }
}

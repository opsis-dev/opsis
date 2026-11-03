use std::time::Duration;

use opsis_terminal::live::TapeRecorder;
use opsis_terminal::tape::{parse, write};
use opsis_types::terminal::TerminalInput;
use pretty_assertions::assert_eq;

fn text(t: &str) -> TerminalInput {
    TerminalInput::Text { text: t.to_owned() }
}

fn key(name: &str) -> TerminalInput {
    TerminalInput::Key {
        key: name.to_owned(),
        ctrl: false,
        alt: false,
        shift: false,
    }
}

fn ctrl(name: &str) -> TerminalInput {
    TerminalInput::Key {
        key: name.to_owned(),
        ctrl: true,
        alt: false,
        shift: false,
    }
}

const fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

#[test]
fn typing_keys_and_pauses_become_a_tape() {
    let mut recorder = TapeRecorder::default();
    let inputs = [
        (0, text("l")),
        (80, text("s")),
        (160, key("Enter")),
        (1400, key("ArrowUp")),
        (1450, key("ArrowUp")),
        (1500, ctrl("c")),
        (
            1600,
            TerminalInput::Paste {
                text: "echo \"hi\"".to_owned(),
            },
        ),
        (1700, key("F5")),
        (
            1750,
            TerminalInput::Wheel {
                col: 1,
                row: 1,
                notches: 2,
            },
        ),
    ];
    for (at, input) in &inputs {
        recorder.input(ms(*at), input);
    }
    assert_eq!(recorder.skipped(), 1);
    let tape = recorder.finish(ms(3000), 80, 24);
    assert_eq!(
        write(&tape),
        "Set Columns 80\nSet Rows 24\nType \"ls\"\nEnter\nSleep 1240ms\nUp 2\nCtrl+c\nCopy 'echo \"hi\"'\nPaste\nSleep 1300ms\n"
    );
    assert_eq!(parse(&write(&tape)), Ok(tape));
}

#[test]
fn a_pause_splits_typing_into_two_commands() {
    let mut recorder = TapeRecorder::default();
    recorder.input(ms(0), &text("git"));
    recorder.input(ms(900), &text(" status"));
    let tape = recorder.finish(ms(950), 100, 30);
    assert_eq!(
        write(&tape),
        "Set Columns 100\nSet Rows 30\nType \"git\"\nSleep 900ms\nType \" status\"\n"
    );
}

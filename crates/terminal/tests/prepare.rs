use std::time::Duration;

use opsis_compile::terminal::default_theme;
use opsis_terminal::cast::{Cast, Event, Header, Version};
use opsis_terminal::colours::ThemeColours;
use opsis_terminal::frames::{Frames, Redaction, read, write};
use opsis_terminal::prepare::{Mask, MaskArea, MaskLook, PrepareInput, frame_count, prepare};
use opsis_terminal::timeline::{END, HIDE, SHOW, START};
use proptest::prelude::*;
use regex::Regex;

const FPS: u32 = 30;

const fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

fn cast(events: Vec<Event>) -> Cast {
    let mut events = events;
    events.sort_by_key(|e| e.time);
    Cast {
        version: Version::Three,
        header: Header {
            term_cols: 40,
            term_rows: 6,
            ..Header::default()
        },
        events,
    }
}

fn out(t: u64, text: &str) -> Event {
    Event::output(ms(t), text.to_owned())
}

fn mark(t: u64, label: &str) -> Event {
    Event::marker(ms(t), label.to_owned())
}

fn frames_of(cast: &Cast, masks: &[Mask]) -> Frames {
    let theme = default_theme();
    prepare(&PrepareInput {
        cast,
        fps: FPS,
        theme: &theme,
        masks,
    })
    .unwrap()
}

fn screen_text(frames: &Frames, index: usize) -> String {
    frames.frames[index]
        .row_indices
        .iter()
        .map(|&i| {
            frames.row_table[usize::try_from(i).unwrap()]
                .iter()
                .map(|r| r.text.as_str())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn makes_one_frame_per_film_frame_and_one_more() {
    let take = cast(vec![mark(0, START), out(100, "hello"), mark(1000, END)]);
    let frames = frames_of(&take, &[]);
    assert_eq!(frames.frames.len(), 31);
    assert_eq!(frame_count(ms(1001), FPS), 32);
    assert!(screen_text(&frames, 30).contains("hello"));
    assert!(!screen_text(&frames, 0).contains("hello"));
}

#[test]
fn gives_hidden_parts_no_frames_but_keeps_what_they_drew() {
    let take = cast(vec![
        mark(0, START),
        out(100, "seen "),
        mark(500, HIDE),
        out(600, "\x1b[2J\x1b[Hsetup done "),
        mark(5500, SHOW),
        out(5600, "after"),
        mark(6000, END),
    ]);
    let frames = frames_of(&take, &[]);
    assert_eq!(frames.frames.len(), 31);
    let at_show = screen_text(&frames, 15);
    assert!(at_show.contains("setup done") && !at_show.contains("seen"));
    assert!(screen_text(&frames, 30).contains("after"));
}

#[test]
fn never_shows_half_of_a_synchronized_frame() {
    let take = cast(vec![
        mark(0, START),
        out(50, "old"),
        out(200, "\x1b[?2026h\x1b[2J\x1b[Hhalf"),
        out(700, " whole\x1b[?2026l"),
        mark(1000, END),
    ]);
    let frames = frames_of(&take, &[]);
    for index in 0..frames.frames.len() {
        let text = screen_text(&frames, index);
        assert!(
            !text.contains("half") || text.contains("whole"),
            "{index}: {text}"
        );
    }
    assert!(screen_text(&frames, 30).contains("half whole"));
}

#[test]
fn lets_go_of_a_hold_the_program_never_ends() {
    let take = cast(vec![
        mark(0, START),
        out(100, "\x1b[?2026hstuck"),
        mark(3000, END),
    ]);
    let frames = frames_of(&take, &[]);
    assert!(!screen_text(&frames, 15).contains("stuck"));
    assert!(screen_text(&frames, 90).contains("stuck"));
}

#[test]
fn never_leaves_masked_text_in_any_frame() {
    let take = cast(vec![
        mark(0, START),
        out(100, "token: sk-12345 ok\r\n"),
        out(400, "again sk-99999\r\n"),
        out(500, "split sk-77777777777777777777777777777777777777"),
        mark(1000, END),
    ]);
    let masks = [Mask {
        areas: vec![],
        patterns: vec![Regex::new(r"sk-[0-9]+").unwrap()],
        from: Duration::ZERO,
        until: None,
        look: MaskLook::Solid(0x00ff_0000),
    }];
    let frames = frames_of(&take, &masks);
    for index in 0..frames.frames.len() {
        let text = screen_text(&frames, index);
        assert!(
            !text.contains("sk-") && !text.contains("123") && !text.contains("777"),
            "{text}"
        );
    }
    let last = &frames.frames[30];
    let redacted = last
        .row_indices
        .iter()
        .flat_map(|&i| &frames.row_table[usize::try_from(i).unwrap()])
        .filter(|r| r.redaction == Redaction::Solid)
        .count();
    assert!(redacted >= 3);
    assert!(screen_text(&frames, 30).contains("ok"));
}

#[test]
fn masks_whole_cells_under_an_area_only_while_it_is_on() {
    let take = cast(vec![mark(0, START), out(0, "abcdefghij"), mark(1000, END)]);
    let masks = [Mask {
        areas: vec![MaskArea {
            x: 0.0,
            y: 0.0,
            width: 0.11,
            height: 0.1,
        }],
        patterns: vec![],
        from: ms(500),
        until: None,
        look: MaskLook::Blur,
    }];
    let frames = frames_of(&take, &masks);
    assert!(screen_text(&frames, 5).starts_with("abcdefghij"));
    assert!(screen_text(&frames, 30).starts_with("fghij"));
}

#[test]
fn follows_palette_changes_from_the_frame_they_happen() {
    let theme = ThemeColours::of(&default_theme());
    let take = cast(vec![
        mark(0, START),
        out(0, "\x1b[31mred\x1b[0m"),
        out(500, "\x1b]4;1;rgb:12/34/56\x1b\\"),
        mark(1000, END),
    ]);
    let frames = frames_of(&take, &[]);
    let first_fg = |index: usize| {
        let row = frames.frames[index].row_indices[0];
        frames.row_table[usize::try_from(row).unwrap()][0].fg
    };
    assert_eq!(first_fg(3), theme.palette[1]);
    assert_eq!(first_fg(30), 0x0012_3456);
}

proptest! {
    #[test]
    fn reads_back_the_frames_file_it_writes(chunks in proptest::collection::vec(("[ -~]{0,30}", 0u8..8), 1..8)) {
        let mut events = vec![mark(0, START)];
        let mut t = 0;
        for (text, colour) in chunks {
            t += 90;
            events.push(out(t, &format!("\x1b[3{colour}m{text}\r\n")));
        }
        events.push(mark(t + 100, END));
        let frames = frames_of(&cast(events), &[]);
        let bytes = write(&frames).unwrap();
        prop_assert_eq!(read(&bytes).unwrap(), frames);
    }
}

proptest! {
    #[test]
    fn a_swatch_reads_back_as_the_colours_it_holds(colours in prop::collection::btree_set(0u32..0x0100_0000, 1..3000)) {
        let colours: Vec<u32> = colours.into_iter().collect();
        let made = opsis_terminal::colours::swatch(&colours);
        prop_assert_eq!(made.rgb.len(), usize::try_from(made.width * made.height * 3).unwrap());
        prop_assert!(colours.len() <= usize::try_from(made.width * made.height).unwrap());
        let map = opsis_terminal::colours::read_swatch(&colours, &made.rgb);
        prop_assert!(colours.iter().all(|c| map.get(c) == Some(c)));
    }

    #[test]
    fn recolouring_reaches_every_colour_a_take_shows(shift in 1u32..255) {
        let made = cast(vec![
            out(0, "\u{1b}[31mred\u{1b}[42mgreen\u{1b}[0m plain\u{1b}[4:3;58;2;9;9;9mline"),
            out(400, "\u{1b}]11;#203040\u{7}more"),
        ]);
        let mut frames = frames_of(&made, &[]);
        let colours = frames.colours();
        let map: std::collections::HashMap<u32, u32> =
            colours.iter().map(|c| (*c, (c + shift) & 0x00ff_ffff)).collect();
        frames.recolour(&map);
        let after = frames.colours();
        let expected: std::collections::BTreeSet<u32> = map.values().copied().collect();
        prop_assert_eq!(after.into_iter().collect::<std::collections::BTreeSet<u32>>(), expected);
    }
}

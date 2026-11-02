use opsis_vt::{
    CellWidth, Colour, Cursor, Event, Flags, Rgb, Terminal, Underline, VtError, check_abi,
};
use proptest::prelude::*;

const COLS: u16 = 80;
const ROWS: u16 = 24;

type Picture = (
    Vec<Vec<(String, Colour, Colour, Flags, Underline, Colour, CellWidth)>>,
    Cursor,
);

fn terminal() -> Terminal {
    Terminal::new(COLS, ROWS, 1000).expect("a terminal")
}

fn picture(terminal: &mut Terminal) -> Picture {
    let frame = terminal.frame().expect("a frame");
    let rows = frame
        .row_iter()
        .map(|row| {
            row.cells()
                .map(|cell| {
                    (
                        cell.text().to_owned(),
                        cell.fg(),
                        cell.bg(),
                        cell.flags(),
                        cell.underline(),
                        cell.underline_colour(),
                        cell.width(),
                    )
                })
                .collect()
        })
        .collect();
    (rows, frame.cursor())
}

fn row_text(terminal: &mut Terminal, y: u16) -> String {
    terminal
        .frame()
        .expect("a frame")
        .row(y)
        .expect("a row")
        .text()
}

fn colour() -> impl Strategy<Value = Colour> {
    prop_oneof![
        Just(Colour::Default),
        any::<u8>().prop_map(Colour::Palette),
        any::<(u8, u8, u8)>().prop_map(|(r, g, b)| Colour::Rgb(Rgb { r, g, b })),
    ]
}

fn underline() -> impl Strategy<Value = Underline> {
    prop_oneof![
        Just(Underline::None),
        Just(Underline::Single),
        Just(Underline::Double),
        Just(Underline::Curly),
        Just(Underline::Dotted),
        Just(Underline::Dashed),
    ]
}

fn flags() -> impl Strategy<Value = Flags> {
    any::<[bool; 8]>().prop_map(|on| Flags {
        bold: on[0],
        italic: on[1],
        faint: on[2],
        blink: on[3],
        inverse: on[4],
        invisible: on[5],
        strikethrough: on[6],
        overline: on[7],
        hyperlink: false,
    })
}

fn colour_parameters(base: u8, colour: Colour) -> Option<String> {
    match colour {
        Colour::Default => None,
        Colour::Palette(index) => Some(format!("{base};5;{index}")),
        Colour::Rgb(Rgb { r, g, b }) => Some(format!("{base};2;{r};{g};{b}")),
    }
}

fn sgr(
    fg: Colour,
    bg: Colour,
    flags: Flags,
    underline: Underline,
    underline_colour: Colour,
) -> String {
    let switches = [
        (flags.bold, "1"),
        (flags.faint, "2"),
        (flags.italic, "3"),
        (flags.blink, "5"),
        (flags.inverse, "7"),
        (flags.invisible, "8"),
        (flags.strikethrough, "9"),
        (flags.overline, "53"),
    ];
    let mut attributes = vec!["0".to_owned()];
    attributes.extend(
        switches
            .iter()
            .filter(|(on, _)| *on)
            .map(|(_, code)| (*code).to_owned()),
    );
    let style = match underline {
        Underline::None => None,
        Underline::Single => Some(1),
        Underline::Double => Some(2),
        Underline::Curly => Some(3),
        Underline::Dotted => Some(4),
        Underline::Dashed => Some(5),
    };
    attributes.extend(style.map(|code| format!("4:{code}")));
    let colours: Vec<String> = [
        colour_parameters(38, fg),
        colour_parameters(48, bg),
        colour_parameters(58, underline_colour),
    ]
    .into_iter()
    .flatten()
    .collect();
    let flags = format!("\x1b[{}m", attributes.join(";"));
    if colours.is_empty() {
        flags
    } else {
        format!("{flags}\x1b[{}m", colours.join(";"))
    }
}

fn sequence() -> impl Strategy<Value = Vec<u8>> {
    prop_oneof![
        "[ -~]{1,30}".prop_map(String::into_bytes),
        any::<String>().prop_map(String::into_bytes),
        proptest::collection::vec(any::<u8>(), 1..20),
        (1u16..30, 1u16..90).prop_map(|(row, col)| format!("\x1b[{row};{col}H").into_bytes()),
        (colour(), colour(), flags(), underline(), colour())
            .prop_map(|(fg, bg, on, line, under)| sgr(fg, bg, on, line, under).into_bytes()),
        Just(b"\r\n".to_vec()),
        Just(b"\x1b[2J".to_vec()),
        Just(b"\x1b[K".to_vec()),
        "[a-z ]{0,20}".prop_map(|title| format!("\x1b]0;{title}\x07").into_bytes()),
        Just("日本語 e\u{301} 👨\u{200d}👩\u{200d}👧".as_bytes().to_vec()),
    ]
}

proptest! {
    #[test]
    fn printable_ascii_shorter_than_a_row_is_the_row_text(text in "[ -~]{0,79}") {
        let mut terminal = terminal();
        terminal.feed(text.as_bytes());
        prop_assert_eq!(row_text(&mut terminal, 0), text.trim_end());
    }

    #[test]
    fn feeding_in_chunks_draws_what_feeding_at_once_draws(
        pieces in proptest::collection::vec(sequence(), 1..40),
        cuts in proptest::collection::vec(any::<prop::sample::Index>(), 0..20),
    ) {
        let bytes: Vec<u8> = pieces.concat();
        let mut whole = terminal();
        whole.feed(&bytes);

        let mut points: Vec<usize> = cuts.iter().map(|cut| cut.index(bytes.len() + 1)).collect();
        points.push(0);
        points.push(bytes.len());
        points.sort_unstable();
        let mut chunked = terminal();
        for pair in points.windows(2) {
            chunked.feed(&bytes[pair[0]..pair[1]]);
        }
        prop_assert_eq!(picture(&mut chunked), picture(&mut whole));
    }

    #[test]
    fn sgr_attributes_come_back_on_the_cell(
        fg in colour(),
        bg in colour(),
        on in flags(),
        line in underline(),
        under in colour(),
    ) {
        let mut terminal = terminal();
        terminal.feed(format!("{}X", sgr(fg, bg, on, line, under)).as_bytes());
        let frame = terminal.frame().expect("a frame");
        let cell = frame.row(0).and_then(|row| row.cell(0)).expect("a cell");
        prop_assert_eq!(cell.text(), "X");
        prop_assert_eq!(cell.fg(), fg);
        prop_assert_eq!(cell.bg(), bg);
        prop_assert_eq!(cell.flags(), on);
        prop_assert_eq!(cell.underline(), line);
        prop_assert_eq!(cell.underline_colour(), under);
    }

    #[test]
    fn wide_characters_take_two_cells(wide in prop_oneof![0x4e00u32..0x9fff, 0x1f600u32..0x1f64f]) {
        let character = char::from_u32(wide).expect("a character");
        let mut terminal = terminal();
        terminal.feed(format!("{character}a").as_bytes());
        let frame = terminal.frame().expect("a frame");
        let row = frame.row(0).expect("a row");
        let cells: Vec<_> = row.cells().take(3).collect();
        prop_assert_eq!(cells[0].text(), character.to_string());
        prop_assert_eq!(cells[0].width(), CellWidth::Wide);
        prop_assert_eq!(cells[1].width(), CellWidth::Spacer);
        prop_assert_eq!(cells[2].text(), "a");
    }

    #[test]
    fn combining_marks_stay_in_their_cell(
        base in "[a-z]",
        marks in proptest::collection::vec(0x300u32..0x36f, 1..24),
    ) {
        let cluster: String = base
            .chars()
            .chain(marks.iter().filter_map(|&mark| char::from_u32(mark)))
            .collect();
        let mut terminal = terminal();
        terminal.feed(format!("{cluster}z").as_bytes());
        let frame = terminal.frame().expect("a frame");
        let row = frame.row(0).expect("a row");
        prop_assert_eq!(row.cell(0).expect("a cell").text(), cluster.as_str());
        prop_assert_eq!(row.cell(1).expect("a cell").text(), "z");
    }

    #[test]
    fn zwj_sequences_stay_in_one_cell(people in proptest::collection::vec(0x1f466u32..=0x1f469, 2..6)) {
        let cluster = people
            .iter()
            .filter_map(|&person| char::from_u32(person))
            .map(String::from)
            .collect::<Vec<_>>()
            .join("\u{200d}");
        let mut terminal = terminal();
        terminal.feed(format!("{cluster}z").as_bytes());
        let frame = terminal.frame().expect("a frame");
        let row = frame.row(0).expect("a row");
        prop_assert_eq!(row.cell(0).expect("a cell").text(), cluster.as_str());
        prop_assert_eq!(row.cell(0).expect("a cell").width(), CellWidth::Wide);
        prop_assert_eq!(row.cell(2).expect("a cell").text(), "z");
    }

    #[test]
    fn position_reports_and_device_attributes_are_answered(row in 1u16..=ROWS, col in 1u16..=COLS) {
        let mut terminal = terminal();
        terminal.feed(format!("\x1b[{row};{col}H\x1b[6n").as_bytes());
        prop_assert_eq!(terminal.take_replies(), format!("\x1b[{row};{col}R").into_bytes());
        terminal.feed(b"\x1b[c");
        let reply = terminal.take_replies();
        prop_assert!(reply.starts_with(b"\x1b[?") && reply.ends_with(b"c"));
        prop_assert!(terminal.take_replies().is_empty());
    }

    #[test]
    fn a_cursor_walk_ends_where_the_moves_say(
        moves in proptest::collection::vec((0u8..5, 1u16..120, 1u16..120), 0..40),
    ) {
        let mut terminal = terminal();
        let (mut x, mut y) = (0u16, 0u16);
        for (kind, a, b) in moves {
            let sequence = match kind {
                0 => {
                    y = y.saturating_sub(a);
                    format!("\x1b[{a}A")
                }
                1 => {
                    y = (y + a).min(ROWS - 1);
                    format!("\x1b[{a}B")
                }
                2 => {
                    x = (x + a).min(COLS - 1);
                    format!("\x1b[{a}C")
                }
                3 => {
                    x = x.saturating_sub(a);
                    format!("\x1b[{a}D")
                }
                _ => {
                    y = a.min(ROWS) - 1;
                    x = b.min(COLS) - 1;
                    format!("\x1b[{a};{b}H")
                }
            };
            terminal.feed(sequence.as_bytes());
        }
        let cursor = terminal.frame().expect("a frame").cursor();
        prop_assert_eq!((cursor.x, cursor.y), (x, y));
    }

    #[test]
    fn scrolling_keeps_the_last_rows(count in 1usize..80) {
        let lines: Vec<String> = (0..count).map(|line| format!("line {line}")).collect();
        let mut terminal = terminal();
        terminal.feed(lines.join("\r\n").as_bytes());
        let shown: Vec<String> = terminal.frame().expect("a frame").row_iter().map(|row| row.text()).collect();
        let first = count.saturating_sub(usize::from(ROWS));
        prop_assert_eq!(&shown[..count - first], &lines[first..]);
        prop_assert!(shown[count - first..].iter().all(String::is_empty));
    }

    #[test]
    fn prompt_marks_report_each_command_and_its_exit_code(codes in proptest::collection::vec(0i32..=255, 1..8)) {
        let mut terminal = terminal();
        let mut expected = Vec::new();
        for code in &codes {
            terminal.feed(
                format!("\x1b]133;A\x07$ \x1b]133;B\x07run\r\n\x1b]133;C\x07out\r\n\x1b]133;D;{code}\x07").as_bytes(),
            );
            expected.extend([Event::Prompt, Event::Input, Event::Command, Event::Done { code: Some(*code) }]);
        }
        prop_assert_eq!(terminal.take_events(), expected);
    }

    #[test]
    fn a_render_hold_freezes_the_frame_until_it_ends(before in "[a-z]{1,20}", after in "[A-Z]{1,20}", release in any::<bool>()) {
        let mut terminal = terminal();
        terminal.feed(before.as_bytes());
        terminal.feed(b"\x1b[?2026h");
        terminal.feed(after.as_bytes());
        {
            let frame = terminal.frame().expect("a frame");
            prop_assert!(frame.held());
            prop_assert_eq!(frame.row(0).expect("a row").text(), before.as_str());
        }
        if release {
            terminal.release_hold();
            prop_assert_eq!(terminal.take_events(), vec![Event::HoldBegin]);
        } else {
            terminal.feed(b"\x1b[?2026l");
            prop_assert_eq!(terminal.take_events(), vec![Event::HoldBegin, Event::HoldEnd]);
        }
        let frame = terminal.frame().expect("a frame");
        prop_assert!(!frame.held());
        prop_assert_eq!(frame.row(0).expect("a row").text(), format!("{before}{after}"));
    }

    #[test]
    fn every_row_has_a_cell_per_column_after_a_resize(
        text in "[ -~]{0,200}",
        sizes in proptest::collection::vec((1u16..200, 1u16..60), 1..6),
    ) {
        let mut terminal = terminal();
        terminal.feed(text.as_bytes());
        for (cols, rows) in sizes {
            terminal.resize(cols, rows).expect("a resize");
            let frame = terminal.frame().expect("a frame");
            prop_assert_eq!((frame.cols(), frame.rows()), (cols, rows));
            prop_assert_eq!(frame.row_iter().count(), usize::from(rows));
            prop_assert!(frame.row_iter().all(|row| row.len() == usize::from(cols)));
        }
    }

    #[test]
    fn a_palette_change_is_reported_once(index in any::<u8>(), r in any::<u8>(), g in any::<u8>(), b in any::<u8>()) {
        let mut terminal = terminal();
        prop_assert!(!terminal.frame().expect("a frame").palette_changed());
        prop_assert!(terminal.palette().is_none());
        terminal.feed(format!("\x1b]4;{index};rgb:{r:02x}/{g:02x}/{b:02x}\x07").as_bytes());
        prop_assert!(terminal.frame().expect("a frame").palette_changed());
        prop_assert!(!terminal.frame().expect("a frame").palette_changed());
        let palette = terminal.palette().expect("a palette");
        prop_assert_eq!(palette.colours[usize::from(index)], Some(Rgb { r, g, b }));
        prop_assert_eq!(palette.colours.iter().filter(|colour| colour.is_some()).count(), 1);
    }
}

#[test]
fn the_engine_and_opsis_agree_on_every_struct() {
    assert!(check_abi().is_ok());
}

#[test]
fn a_terminal_needs_a_size() {
    assert!(matches!(
        Terminal::new(0, 24, 0),
        Err(VtError::Size { cols: 0, rows: 24 })
    ));
    assert!(matches!(
        Terminal::new(80, 0, 0),
        Err(VtError::Size { cols: 80, rows: 0 })
    ));
}

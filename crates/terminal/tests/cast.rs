use std::collections::BTreeMap;
use std::time::Duration;

use opsis_terminal::cast::{
    CastColour, CastTheme, Event, EventData, Header, accelerate, limit_idle_time, read, read_path,
    write_v3,
};
use proptest::prelude::*;
use serde_json::Value;

const fn colour(r: u8, g: u8, b: u8) -> CastColour {
    CastColour::new(r, g, b)
}

#[test]
fn open_v1_minimal() {
    let cast = read_path("tests/casts/minimal-v1.json").unwrap();
    assert_eq!(cast.version, 1);
    assert_eq!((cast.header.term_cols, cast.header.term_rows), (100, 50));
    assert!(cast.header.term_theme.is_none());
    assert_eq!(cast.events[0].time, Duration::from_millis(1230));
    assert!(matches!(cast.events[0].data, EventData::Output(ref s) if s == "hello"));
}

#[test]
fn open_v1_full() {
    let cast = read_path("tests/casts/full-v1.json").unwrap();
    assert_eq!(cast.version, 1);
    assert_eq!((cast.header.term_cols, cast.header.term_rows), (100, 50));
    let expected_env = BTreeMap::from([
        ("SHELL".to_owned(), "/bin/bash".to_owned()),
        ("TERM".to_owned(), "xterm-256color".to_owned()),
    ]);
    assert_eq!(cast.header.env.unwrap(), expected_env);
    assert_eq!(cast.events[0].time, Duration::from_micros(1));
    assert!(matches!(cast.events[0].data, EventData::Output(ref s) if s == "ż"));
    assert_eq!(cast.events[1].time, Duration::from_micros(10_000_001));
    assert!(matches!(cast.events[1].data, EventData::Output(ref s) if s == "ółć"));
    assert_eq!(cast.events[2].time, Duration::from_micros(10_500_001));
    assert!(matches!(cast.events[2].data, EventData::Output(ref s) if s == "\r\n"));
}

#[test]
fn open_v1_with_nulls_in_header() {
    let cast = read_path("tests/casts/nulls-v1.json").unwrap();
    assert_eq!(cast.version, 1);
    let expected_env = BTreeMap::from([("SHELL".to_owned(), "/bin/bash".to_owned())]);
    assert_eq!(cast.header.env.unwrap(), expected_env);
}

#[test]
fn open_v2_minimal() {
    let cast = read_path("tests/casts/minimal-v2.cast").unwrap();
    assert_eq!(cast.version, 2);
    assert_eq!((cast.header.term_cols, cast.header.term_rows), (100, 50));
    assert!(cast.header.term_theme.is_none());
    assert_eq!(cast.events[0].time, Duration::from_millis(1230));
    assert!(matches!(cast.events[0].data, EventData::Output(ref s) if s == "hello"));
}

#[test]
fn open_v2_full() {
    let cast = read_path("tests/casts/full-v2.cast").unwrap();
    let events = &cast.events[..5];
    let theme = cast.header.term_theme.clone().unwrap();
    assert_eq!(cast.version, 2);
    assert_eq!((cast.header.term_cols, cast.header.term_rows), (100, 50));
    assert_eq!(cast.header.timestamp, Some(1_509_091_818));
    assert_eq!(theme.fg, colour(0, 0, 0));
    assert_eq!(theme.bg, colour(0xff, 0xff, 0xff));
    assert_eq!(theme.palette[0], colour(0x24, 0x1f, 0x31));
    let expected_env = BTreeMap::from([
        ("SHELL".to_owned(), "/bin/bash".to_owned()),
        ("TERM".to_owned(), "xterm-256color".to_owned()),
    ]);
    assert_eq!(cast.header.env.unwrap(), expected_env);
    assert_eq!(events[0].time, Duration::from_micros(1));
    assert!(matches!(events[0].data, EventData::Output(ref s) if s == "ż"));
    assert_eq!(events[1].time, Duration::from_micros(1_000_000));
    assert!(matches!(events[1].data, EventData::Output(ref s) if s == "ółć"));
    assert_eq!(events[2].time, Duration::from_millis(2300));
    assert!(matches!(events[2].data, EventData::Input(ref s) if s == "\n"));
    assert_eq!(events[3].time, Duration::from_micros(5_600_001));
    assert!(matches!(events[3].data, EventData::Resize(80, 40)));
    assert_eq!(events[4].time, Duration::from_millis(10500));
    assert!(matches!(events[4].data, EventData::Output(ref s) if s == "\r\n"));
}

#[test]
fn open_v2_with_nulls_in_header() {
    let cast = read_path("tests/casts/nulls-v2.cast").unwrap();
    assert_eq!(cast.version, 2);
    let expected_env = BTreeMap::from([("SHELL".to_owned(), "/bin/bash".to_owned())]);
    assert_eq!(cast.header.env.unwrap(), expected_env);
}

#[test]
fn open_v3_minimal() {
    let cast = read_path("tests/casts/minimal-v3.cast").unwrap();
    assert_eq!(cast.version, 3);
    assert_eq!((cast.header.term_cols, cast.header.term_rows), (100, 50));
    assert!(cast.header.term_theme.is_none());
    assert_eq!(cast.events[0].time, Duration::from_millis(1230));
    assert!(matches!(cast.events[0].data, EventData::Output(ref s) if s == "hello"));
}

#[test]
fn open_v3_full() {
    let cast = read_path("tests/casts/full-v3.cast").unwrap();
    let events = &cast.events[..5];
    let theme = cast.header.term_theme.clone().unwrap();
    assert_eq!(cast.version, 3);
    assert_eq!((cast.header.term_cols, cast.header.term_rows), (100, 50));
    assert_eq!(cast.header.timestamp, Some(1_509_091_818));
    assert_eq!(theme.fg, colour(0, 0, 0));
    assert_eq!(theme.bg, colour(0xff, 0xff, 0xff));
    assert_eq!(theme.palette[0], colour(0x24, 0x1f, 0x31));
    assert_eq!(events[0].time, Duration::from_micros(1));
    assert!(matches!(events[0].data, EventData::Output(ref s) if s == "ż"));
    assert_eq!(events[1].time, Duration::from_micros(1_000_001));
    assert!(matches!(events[1].data, EventData::Output(ref s) if s == "ółć"));
    assert_eq!(events[2].time, Duration::from_micros(1_300_001));
    assert!(matches!(events[2].data, EventData::Input(ref s) if s == "\n"));
    assert_eq!(events[3].time, Duration::from_micros(2_900_002));
    assert!(matches!(events[3].data, EventData::Resize(80, 40)));
    assert_eq!(events[4].time, Duration::from_micros(13_400_002));
    assert!(matches!(events[4].data, EventData::Output(ref s) if s == "\r\n"));
}

#[test]
fn refuses_an_empty_file_and_one_of_another_kind() {
    assert!(read("").is_err());
    assert!(read("{\"version\": 9}").is_err());
    assert!(
        read("{\"version\":3,\"term\":{\"cols\":80,\"rows\":24}}\n[1.0, \"r\", \"80\"]").is_err()
    );
}

fn parse(text: &str) -> Vec<Value> {
    text.split('\n')
        .filter(|s| !s.is_empty())
        .map(serde_json::from_str::<Value>)
        .collect::<serde_json::Result<Vec<_>>>()
        .unwrap()
}

#[test]
fn header_encoding() {
    let env = BTreeMap::from([
        ("SHELL".to_owned(), "/usr/bin/fish".to_owned()),
        ("TERM".to_owned(), "xterm256-color".to_owned()),
    ]);
    let theme = CastTheme {
        fg: colour(0, 1, 2),
        bg: colour(0, 100, 200),
        palette: (0..16u8)
            .map(|i| colour(i * 10, i * 10 + 1, i * 10 + 2))
            .collect(),
    };
    let header = Header {
        timestamp: Some(1_704_719_152),
        idle_time_limit: Some(1.5),
        command: Some("/bin/bash".to_owned()),
        title: Some("Demo".to_owned()),
        env: Some(env),
        term_theme: Some(theme),
        ..Header::default()
    };
    let lines = parse(&write_v3(&header, &[]));
    assert_eq!(lines[0]["version"], 3);
    assert_eq!(lines[0]["term"]["cols"], 80);
    assert_eq!(lines[0]["term"]["rows"], 24);
    assert_eq!(lines[0]["timestamp"], 1_704_719_152);
    assert_eq!(lines[0]["idle_time_limit"], 1.5);
    assert_eq!(lines[0]["command"], "/bin/bash");
    assert_eq!(lines[0]["title"], "Demo");
    assert_eq!(lines[0]["env"]["SHELL"], "/usr/bin/fish");
    assert_eq!(lines[0]["term"]["theme"]["fg"], "#000102");
    assert_eq!(lines[0]["term"]["theme"]["bg"], "#0064c8");
    assert_eq!(
        lines[0]["term"]["theme"]["palette"],
        "#000102:#0a0b0c:#141516:#1e1f20:#28292a:#323334:#3c3d3e:#464748:#505152:#5a5b5c:#646566:#6e6f70:#78797a:#828384:#8c8d8e:#969798"
    );
}

#[test]
fn encoder() {
    let events = [
        Event::output(Duration::from_micros(1_000_000), "hello\r\n".to_owned()),
        Event::output(Duration::from_micros(1_000_001), "world".to_owned()),
        Event::input(Duration::from_micros(2_000_002), " ".to_owned()),
        Event::resize(Duration::from_micros(3_000_003), (100, 40)),
        Event::marker(Duration::from_millis(3500), "opsis:start".to_owned()),
        Event::output(Duration::from_micros(4_000_004), "żółć".to_owned()),
        Event::exit(Duration::from_micros(5_000_000), 0),
    ];
    let lines = parse(&write_v3(&Header::default(), &events));
    assert_eq!(lines[1], serde_json::json!([1.0, "o", "hello\r\n"]));
    assert_eq!(lines[2], serde_json::json!([0.0, "o", "world"]));
    assert_eq!(lines[3], serde_json::json!([1.0, "i", " "]));
    assert_eq!(lines[4], serde_json::json!([1.0, "r", "100x40"]));
    assert_eq!(lines[5], serde_json::json!([0.5, "m", "opsis:start"]));
    assert_eq!(lines[6], serde_json::json!([0.5, "o", "żółć"]));
    assert_eq!(lines[7], serde_json::json!([1.0, "x", "0"]));
}

fn output(events: &[Event]) -> Vec<(Duration, String)> {
    events
        .iter()
        .filter_map(|e| match &e.data {
            EventData::Output(data) => Some((e.time, data.clone())),
            _ => None,
        })
        .collect()
}

#[test]
fn accelerates() {
    let events: Vec<Event> = [(0u64, "foo"), (20, "bar"), (50, "baz")]
        .map(|(t, o)| Event::output(Duration::from_micros(t), o.to_owned()))
        .into();
    let out = output(&accelerate(events, 2.0));
    assert_eq!(out[0], (Duration::from_micros(0), "foo".to_owned()));
    assert_eq!(out[1], (Duration::from_micros(10), "bar".to_owned()));
    assert_eq!(out[2], (Duration::from_micros(25), "baz".to_owned()));
}

#[test]
fn limits_idle_time() {
    let events: Vec<Event> = [
        (0, "foo"),
        (1_000_000, "bar"),
        (3_500_000, "baz"),
        (4_000_000, "qux"),
        (7_500_000, "quux"),
    ]
    .map(|(t, o)| Event::output(Duration::from_micros(t), o.to_owned()))
    .into();
    let out = output(&limit_idle_time(events, Duration::from_secs(2)));
    let times: Vec<u64> = out
        .iter()
        .map(|(t, _)| u64::try_from(t.as_micros()).unwrap())
        .collect();
    assert_eq!(times, [0, 1_000_000, 3_000_000, 3_500_000, 5_500_000]);
}

fn event() -> impl Strategy<Value = (u64, EventData)> {
    let data = prop_oneof![
        ".{0,12}".prop_map(EventData::Output),
        ".{0,4}".prop_map(EventData::Input),
        (1u16..400, 1u16..200).prop_map(|(c, r)| EventData::Resize(c, r)),
        "[a-z:]{0,12}".prop_map(EventData::Marker),
        any::<i32>().prop_map(EventData::Exit),
    ];
    (0u64..5_000_000, data)
}

proptest! {
    #[test]
    fn reads_back_what_it_writes_within_half_a_millisecond(steps in proptest::collection::vec(event(), 0..40)) {
        let mut time = Duration::ZERO;
        let events: Vec<Event> = steps
            .into_iter()
            .map(|(gap, data)| {
                time += Duration::from_micros(gap);
                Event { time, data }
            })
            .collect();
        let back = read(&write_v3(&Header::default(), &events)).unwrap();
        prop_assert_eq!(back.events.len(), events.len());
        for (made, read) in events.iter().zip(&back.events) {
            prop_assert_eq!(&made.data, &read.data);
            let gap = made.time.abs_diff(read.time);
            prop_assert!(gap <= Duration::from_micros(500), "{gap:?}");
        }
    }
}

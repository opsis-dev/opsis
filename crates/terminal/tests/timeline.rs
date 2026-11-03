use std::time::Duration;

use opsis_terminal::cast::Event;
use opsis_terminal::timeline::{END, HIDE, SHOW, START, STEP_PREFIX, Timeline};
use proptest::prelude::*;

const fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

fn marked(times: &[(u64, &str)], last: u64) -> Vec<Event> {
    let mut events: Vec<Event> = times
        .iter()
        .map(|(t, m)| Event::marker(ms(*t), (*m).to_owned()))
        .collect();
    events.push(Event::output(ms(last), "x".to_owned()));
    events.sort_by_key(|e| e.time);
    events
}

#[test]
fn starts_at_start_recording_skips_hidden_parts_and_ends_at_the_end_marker() {
    let events = marked(
        &[(500, START), (1500, HIDE), (2500, SHOW), (4000, END)],
        6000,
    );
    let t = Timeline::from_events(&events);
    assert_eq!(t.length(), ms(2500));
    assert_eq!(t.visible(ms(400)), None);
    assert_eq!(t.visible(ms(2000)), None);
    assert_eq!(t.visible(ms(5000)), None);
    assert_eq!(t.visible(ms(1000)), Some(ms(500)));
    assert_eq!(t.visible(ms(3000)), Some(ms(1500)));
    assert_eq!(t.raw(ms(1000)), ms(2500));
}

#[test]
fn starts_with_the_first_step_without_a_start_marker() {
    let events = marked(&[(700, &format!("{STEP_PREFIX}a"))], 2000);
    let t = Timeline::from_events(&events);
    assert_eq!((t.start(), t.end()), (ms(700), ms(2000)));
}

fn markers() -> impl Strategy<Value = Vec<(u64, bool)>> {
    proptest::collection::vec((0u64..10_000, any::<bool>()), 0..12)
}

proptest! {
    #[test]
    fn keeps_spans_ordered_and_maps_visible_times_back(points in markers(), start in 0u64..3000, end in 5000u64..12_000) {
        let mut times: Vec<(u64, &str)> = points.iter().map(|(t, hide)| (*t, if *hide { HIDE } else { SHOW })).collect();
        times.push((start, START));
        times.push((end, END));
        let events = marked(&times, 12_000);
        let t = Timeline::from_events(&events);
        let spans = t.spans();
        for pair in spans.windows(2) {
            prop_assert!(pair[0].to <= pair[1].from);
        }
        let total: Duration = spans.iter().map(|s| s.length()).sum();
        prop_assert_eq!(total, t.length());
        let mut last = Duration::ZERO;
        for step in 0..=100u64 {
            let visible = t.length() * u32::try_from(step).unwrap() / 100;
            let raw = t.raw(visible);
            prop_assert!(raw >= last);
            last = raw;
            prop_assert_eq!(t.visible(raw), Some(visible));
        }
        for raw in (0..12_000u64).step_by(37).map(ms) {
            let inside = spans.iter().any(|s| raw >= s.from && raw <= s.to);
            prop_assert_eq!(t.visible(raw).is_some(), inside);
        }
    }
}

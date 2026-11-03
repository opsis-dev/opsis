use std::path::Path;
use std::time::Duration;

use opsis_terminal::cast::{Cast, Event, Header, Version, read_path};
use opsis_terminal::import::{ImportError, import_cast};
use opsis_types::actions::ActionKind;
use pretty_assertions::assert_eq;
use proptest::prelude::*;

fn sample(name: &str) -> Cast {
    read_path(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/casts")
            .join(name),
    )
    .unwrap()
}

fn gaps(events: &[Event]) -> Vec<Duration> {
    events
        .windows(2)
        .map(|w| w[1].time.saturating_sub(w[0].time))
        .collect()
}

const fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

fn cast_of(events: Vec<Event>) -> Cast {
    Cast {
        version: Version::Three,
        header: Header::default(),
        events,
    }
}

#[test]
fn imports_a_cast_with_its_typing_titles_caret_and_pause_limit() {
    let cast = sample("demo.cast");
    let imported = import_cast(cast.clone(), None).unwrap();
    assert!(
        gaps(&imported.events)
            .iter()
            .all(|g| *g <= Duration::from_secs(2))
    );
    assert!(imported.length < cast.duration());
    assert_eq!(imported.header.idle_time_limit, None);
    assert!(
        imported
            .log
            .actions
            .iter()
            .any(|a| a.kind == ActionKind::Type)
    );
    assert_ne!(imported.log.titles.len(), 0);
    assert!(imported.log.caret_track.len() > 1);
    assert_eq!(
        (imported.log.viewport.width, imported.log.viewport.height),
        (75, 18)
    );
    let shorter = import_cast(cast, Some(ms(300))).unwrap();
    assert!(gaps(&shorter.events).iter().all(|g| *g <= ms(300)));
    assert!(shorter.length < imported.length);
}

#[test]
fn typing_keys_and_markers_become_actions() {
    let cast = cast_of(vec![
        Event::output(ms(0), "$ ".to_owned()),
        Event::input(ms(100), "l".to_owned()),
        Event::output(ms(110), "l".to_owned()),
        Event::input(ms(200), "s".to_owned()),
        Event::output(ms(210), "s".to_owned()),
        Event::input(ms(400), "\r".to_owned()),
        Event::output(ms(420), "\r\nfile\r\n$ ".to_owned()),
        Event::marker(ms(500), "Listing".to_owned()),
        Event::input(ms(600), "\u{3}".to_owned()),
        Event::output(ms(700), "^C".to_owned()),
    ]);
    let imported = import_cast(cast, None).unwrap();
    let actions: Vec<(ActionKind, Option<&str>, Option<&str>)> = imported
        .log
        .actions
        .iter()
        .map(|a| (a.kind, a.text.as_deref(), a.label.as_deref()))
        .collect();
    assert_eq!(
        actions,
        vec![
            (ActionKind::Type, Some("ls"), None),
            (ActionKind::Key, None, Some("Enter")),
            (ActionKind::Chapter, None, Some("Listing")),
            (ActionKind::Key, None, Some("Control+C")),
        ]
    );
    let typed = &imported.log.actions[0];
    assert_eq!(typed.key_times.as_deref(), Some(&[100.0, 200.0][..]));
    let rect = typed.rect.unwrap();
    assert_eq!((rect.x, rect.y, rect.w), (2.0, 0.0, 2.0));
}

#[test]
fn the_files_own_colours_become_the_takes_theme() {
    let imported = import_cast(sample("full-v3.cast"), None).unwrap();
    let theme = imported.theme.unwrap();
    assert_eq!(
        (theme.fg.as_str(), theme.bg.as_str()),
        ("#000000", "#ffffff")
    );
    assert_eq!(theme.palette[1], "#c01c28");
}

#[test]
fn refuses_a_cast_that_shows_nothing() {
    let cast = cast_of(vec![Event::input(ms(10), "x".to_owned())]);
    assert!(matches!(
        import_cast(cast, None),
        Err(ImportError::NoOutput)
    ));
}

proptest! {
    #[test]
    fn no_pause_outlasts_the_limit(steps in prop::collection::vec(0u64..5000, 1..40), limit in 1u64..3000) {
        let mut at = 0;
        let events: Vec<Event> = steps
            .iter()
            .map(|gap| {
                at += gap;
                Event::output(ms(at), "x".to_owned())
            })
            .collect();
        let imported = import_cast(cast_of(events), Some(ms(limit))).unwrap();
        prop_assert!(gaps(&imported.events).iter().all(|g| *g <= ms(limit)));
    }
}

use std::time::Duration;

use opsis_types::actions::{Action, CaretSample, CursorSample, TitleChange};
use opsis_types::numbers::millis;

use crate::cast::{Event, Utf8Decoder};

#[derive(Debug, Default)]
pub struct Recording {
    pub events: Vec<Event>,
    pub actions: Vec<Action>,
    pub caret: Vec<CaretSample>,
    pub titles: Vec<TitleChange>,
    pub pointer: Vec<CursorSample>,
    decoder: Utf8Decoder,
}

impl Recording {
    pub fn output(&mut self, at: Duration, bytes: &[u8]) {
        let text = self.decoder.feed(bytes);
        if !text.is_empty() {
            self.events.push(Event::output(at, text));
        }
    }

    pub fn input(&mut self, at: Duration, bytes: &[u8]) {
        self.events.push(Event::input(
            at,
            String::from_utf8_lossy(bytes).into_owned(),
        ));
    }

    pub fn marker(&mut self, at: Duration, label: impl Into<String>) {
        self.events.push(Event::marker(at, label.into()));
    }

    pub fn caret(&mut self, at: Duration, col: u16, row: u16) {
        if self
            .caret
            .last()
            .is_some_and(|c| c.col == col && c.row == row)
        {
            return;
        }
        self.caret.push(CaretSample {
            t: millis(at),
            col,
            row,
        });
    }

    pub fn title(&mut self, at: Duration, title: String) {
        if self.titles.last().is_some_and(|t| t.title == title) {
            return;
        }
        self.titles.push(TitleChange {
            t: millis(at),
            title,
        });
    }

    pub fn finish_text(&mut self, at: Duration) {
        let rest = self.decoder.finish();
        if !rest.is_empty() {
            self.events.push(Event::output(at, rest));
        }
    }

    pub fn sorted_events(&mut self) -> Vec<Event> {
        let mut events = std::mem::take(&mut self.events);
        events.sort_by_key(|e| e.time);
        events
    }
}

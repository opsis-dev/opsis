use std::time::Duration;

use opsis_types::actions::{Action, ActionKind, ClickButton, CursorSample, Point, Rect};
use opsis_types::numbers::millis;
use opsis_types::terminal::{
    CellTarget, LookIn, PointerButton, TerminalAction, TerminalSessionValue, TerminalStep, WaitFor,
};
use opsis_vt::{
    CellPoint, Event as VtEvent, KeyAction, KeyMods, MouseAction, MouseButton, VtError,
};
use regex::Regex;

use crate::input::{InputError, find_on_screen, mouse_button, parse_chord, target_cell};
use crate::recording::Recording;
use crate::session::{Chunk, Session, SessionError};
use crate::timeline::{CHAPTER_PREFIX, HIDE, SCREENSHOT, SCROLL_PREFIX, SHOW, START, STEP_PREFIX};

const QUIET: Duration = Duration::from_millis(300);
const FIRST_PROMPT_LIMIT: Duration = Duration::from_secs(5);
const LAST_QUIET_LIMIT: Duration = Duration::from_secs(5);
const PUMP: Duration = Duration::from_millis(10);
const POINTER_SAMPLE: Duration = Duration::from_millis(16);
const DOUBLE_CLICK_GAP: Duration = Duration::from_millis(80);
const PRESS_HOLD: Duration = Duration::from_millis(60);
const CELL_MIDDLE: f64 = 0.5;

#[derive(Debug, thiserror::Error)]
pub enum DriverError {
    #[error(transparent)]
    Session(#[from] SessionError),
    #[error(transparent)]
    Terminal(#[from] VtError),
    #[error(transparent)]
    Input(#[from] InputError),
    #[error("{0}")]
    Timeout(String),
    #[error("“{pattern}” isn't a pattern Opsis can search for: {reason}")]
    Pattern { pattern: String, reason: String },
    #[error("Cancelled")]
    Cancelled,
}

pub struct Driver<'a> {
    pub session: Session,
    pub rec: Recording,
    value: &'a TerminalSessionValue,
    cancelled: &'a dyn Fn() -> bool,
    log: &'a mut dyn FnMut(String),
    last_output: Duration,
    marks_seen: bool,
    inputs: u64,
    dones: u64,
    at_prompt: bool,
    clipboard: String,
    pointer: Option<(f64, f64)>,
}

impl std::fmt::Debug for Driver<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Driver")
            .field("session", &self.session)
            .field("rec", &self.rec)
            .field("value", &self.value)
            .finish_non_exhaustive()
    }
}

fn seconds(ms: u32) -> String {
    let s = f64::from(ms) / 1000.0;
    format!("{s}")
}

fn ms(value: u32) -> Duration {
    Duration::from_millis(u64::from(value))
}

fn cell_point(cell: CellPoint) -> Point {
    Point {
        x: f64::from(cell.col) + CELL_MIDDLE,
        y: f64::from(cell.row) + CELL_MIDDLE,
    }
}

fn cell_rect(col: u16, row: u16, cols: u16, rows: u16) -> Rect {
    Rect {
        x: f64::from(col),
        y: f64::from(row),
        w: f64::from(cols.max(1)),
        h: f64::from(rows.max(1)),
    }
}

impl<'a> Driver<'a> {
    pub fn new(
        session: Session,
        value: &'a TerminalSessionValue,
        cancelled: &'a dyn Fn() -> bool,
        log: &'a mut dyn FnMut(String),
    ) -> Self {
        Self {
            session,
            rec: Recording::default(),
            value,
            cancelled,
            log,
            last_output: Duration::ZERO,
            marks_seen: false,
            inputs: 0,
            dones: 0,
            at_prompt: false,
            clipboard: String::new(),
            pointer: None,
        }
    }

    pub fn now(&self) -> Duration {
        self.session.now()
    }

    pub fn absorb(&mut self, chunk: Chunk) {
        self.rec.output(chunk.at, &chunk.bytes);
        self.last_output = chunk.at;
        for event in chunk.events {
            match event {
                VtEvent::Prompt => self.marks_seen = true,
                VtEvent::Input => {
                    self.marks_seen = true;
                    self.at_prompt = true;
                    self.inputs += 1;
                }
                VtEvent::Done { .. } => self.dones += 1,
                VtEvent::Title => {
                    let title = self.session.terminal().title();
                    self.rec.title(chunk.at, title);
                }
                _ => {}
            }
        }
        let cursor = self.session.terminal().frame().map(|f| f.cursor());
        if let Ok(cursor) = cursor {
            self.rec.caret(chunk.at, cursor.x, cursor.y);
        }
    }

    fn pump(&mut self, wait: Duration) {
        if let Some(chunk) = self.session.next_chunk(wait) {
            self.absorb(chunk);
        }
    }

    fn wait_until(
        &mut self,
        limit: Duration,
        mut done: impl FnMut(&mut Self) -> bool,
    ) -> Result<bool, DriverError> {
        let deadline = self.now() + limit;
        loop {
            if done(self) {
                return Ok(true);
            }
            if (self.cancelled)() {
                return Err(DriverError::Cancelled);
            }
            let now = self.now();
            if now >= deadline {
                return Ok(false);
            }
            self.pump(PUMP.min(deadline.saturating_sub(now)));
        }
    }

    pub fn sleep(&mut self, length: Duration) -> Result<(), DriverError> {
        self.wait_until(length, |_| false).map(|_| ())
    }

    pub fn wait_quiet(&mut self, limit: Duration) -> Result<bool, DriverError> {
        self.wait_until(limit, |d| d.now().saturating_sub(d.last_output) >= QUIET)
    }

    pub fn wait_first_prompt(&mut self) -> Result<(), DriverError> {
        self.wait_until(FIRST_PROMPT_LIMIT, |d| {
            d.inputs > 0
                || (!d.marks_seen
                    && d.now().saturating_sub(d.last_output) >= QUIET
                    && d.last_output > Duration::ZERO)
        })?;
        Ok(())
    }

    pub fn wait_last(&mut self) -> Result<(), DriverError> {
        self.wait_quiet(LAST_QUIET_LIMIT)?;
        Ok(())
    }

    fn send(&mut self, bytes: &[u8]) -> Result<Duration, DriverError> {
        let at = self.session.write(bytes)?;
        if bytes.contains(&b'\r') {
            self.at_prompt = false;
        }
        self.rec.input(at, bytes);
        Ok(at)
    }

    pub fn marker(&mut self, label: impl Into<String>) {
        let at = self.now();
        self.rec.marker(at, label);
    }

    fn key(&mut self, key: &str, mods: KeyMods) -> Result<Duration, DriverError> {
        let bytes = self
            .session
            .terminal()
            .key_bytes(key, mods, KeyAction::Press)?;
        self.send(&bytes)
    }

    fn type_text(&mut self, text: &str, interval: Duration) -> Result<Vec<f64>, DriverError> {
        let mut times = Vec::new();
        for ch in text.chars() {
            let key = match ch {
                '\n' => "Enter".to_owned(),
                '\t' => "Tab".to_owned(),
                other => other.to_string(),
            };
            let at = self.key(&key, KeyMods::NONE)?;
            times.push(millis(at));
            self.sleep(interval)?;
        }
        Ok(times)
    }

    fn caret(&self) -> Result<CellPoint, DriverError> {
        let cursor = self.session.terminal().frame()?.cursor();
        Ok(CellPoint {
            col: cursor.x,
            row: cursor.y,
        })
    }

    fn size(&self) -> Result<(u16, u16), DriverError> {
        let mut t = self.session.terminal();
        let frame = t.frame()?;
        Ok((frame.cols(), frame.rows()))
    }

    fn typed_action(
        &mut self,
        step: &TerminalStep,
        text: &str,
        interval: Duration,
    ) -> Result<Action, DriverError> {
        let before = self.caret()?;
        let t = millis(self.now());
        let key_times = self.type_text(text, interval)?;
        let after = self.caret()?;
        let (cols, _) = self.size()?;
        let rect = if after.row == before.row {
            cell_rect(
                before.col,
                before.row,
                after.col.saturating_sub(before.col),
                1,
            )
        } else {
            cell_rect(
                0,
                before.row,
                cols,
                after.row.saturating_sub(before.row) + 1,
            )
        };
        let mut action = Action::new(step.id.clone(), ActionKind::Type, t);
        action.t_end = key_times.last().copied();
        action.text = Some(text.to_owned());
        action.label = Some(step.label.clone());
        action.key_times = Some(key_times);
        action.point = Some(Point {
            x: rect.x + rect.w / 2.0,
            y: rect.y + rect.h / 2.0,
        });
        action.rect = Some(rect);
        action.node_id = Some(step.node_id.clone());
        Ok(action)
    }

    fn typing_interval(&self, speed_ms: Option<u32>) -> Duration {
        ms(speed_ms.unwrap_or(self.value.typing_ms))
    }

    fn run_command(
        &mut self,
        step: &TerminalStep,
        command: &str,
        wait: WaitFor,
        limit_ms: u32,
        zoom_output: bool,
    ) -> Result<(), DriverError> {
        let interval = self.typing_interval(None);
        let typed = self.typed_action(step, command, interval)?;
        self.rec.actions.push(typed);
        let dones = self.dones;
        let from_event = self.rec.events.len();
        self.key("Enter", KeyMods::NONE)?;
        let limit = ms(limit_ms);
        let finished = if wait == WaitFor::Prompt && self.marks_seen {
            self.wait_until(limit, |d| d.dones > dones)?
        } else {
            self.wait_quiet(limit)?
        };
        if !finished {
            return Err(DriverError::Timeout(format!(
                "“{command}” didn't finish within {} s",
                seconds(limit_ms)
            )));
        }
        if zoom_output {
            self.output_beat(step, from_event)?;
        }
        Ok(())
    }

    fn output_beat(&mut self, step: &TerminalStep, from_event: usize) -> Result<(), DriverError> {
        let lines = self.rec.events[from_event..]
            .iter()
            .filter_map(|e| match &e.data {
                crate::cast::EventData::Output(text) => Some(text.matches('\n').count()),
                _ => None,
            })
            .sum::<usize>()
            .saturating_sub(1);
        if lines == 0 {
            return Ok(());
        }
        let prompt = self.caret()?;
        let (cols, _) = self.size()?;
        let lines = u16::try_from(lines).unwrap_or(u16::MAX).min(prompt.row);
        if lines == 0 {
            return Ok(());
        }
        let rect = cell_rect(0, prompt.row - lines, cols, lines);
        let mut beat = Action::new(
            format!("{}:output", step.id),
            ActionKind::Focus,
            millis(self.now()),
        );
        beat.point = Some(Point {
            x: rect.x + rect.w / 2.0,
            y: rect.y + rect.h / 2.0,
        });
        beat.rect = Some(rect);
        beat.node_id = Some(step.node_id.clone());
        self.rec.actions.push(beat);
        Ok(())
    }

    fn press_keys(
        &mut self,
        step: &TerminalStep,
        keys: &str,
        repeat: u32,
    ) -> Result<(), DriverError> {
        let chord = parse_chord(keys)?;
        let interval = self.typing_interval(None);
        let mut times = Vec::new();
        for i in 0..repeat.max(1) {
            if i > 0 {
                self.sleep(interval)?;
            }
            let at = self.key(&chord.key, chord.mods)?;
            times.push(millis(at));
        }
        let mut action = Action::new(step.id.clone(), ActionKind::Key, times[0]);
        action.label = Some(keys.to_owned());
        action.t_end = times.last().copied();
        action.key_times = Some(times);
        action.node_id = Some(step.node_id.clone());
        self.rec.actions.push(action);
        Ok(())
    }

    fn target(&self, target: &CellTarget) -> Result<CellPoint, DriverError> {
        let mut t = self.session.terminal();
        let frame = t.frame()?;
        Ok(target_cell(frame, target)?)
    }

    fn pointer_sample(&mut self, at: Duration, x: f64, y: f64) {
        let shape = self.session.terminal().pointer_shape();
        self.rec.pointer.push(CursorSample {
            t: millis(at),
            x,
            y,
            c: Some(shape),
        });
    }

    fn glide(&mut self, to: CellPoint, button: MouseButton) -> Result<(), DriverError> {
        let end = cell_point(to);
        let (cols, rows) = self.size()?;
        let start = self
            .pointer
            .unwrap_or_else(|| (f64::from(cols) / 2.0, f64::from(rows) / 2.0));
        let length = ms(self.value.pointer_ms);
        let began = self.now();
        let steps = (length.as_millis() / POINTER_SAMPLE.as_millis()).max(1);
        let steps = u32::try_from(steps).unwrap_or(u32::MAX);
        for i in 1..=steps {
            let share = f64::from(i) / f64::from(steps);
            let eased = share * share * 2.0f64.mul_add(-share, 3.0);
            let x = (end.x - start.0).mul_add(eased, start.0);
            let y = (end.y - start.1).mul_add(eased, start.1);
            let cell = CellPoint {
                col: opsis_types::numbers::truncate(x.clamp(0.0, f64::from(cols) - 1.0)),
                row: opsis_types::numbers::truncate(y.clamp(0.0, f64::from(rows) - 1.0)),
            };
            let bytes = self.session.terminal().mouse_bytes(
                MouseAction::Motion,
                button,
                KeyMods::NONE,
                cell,
            );
            if !bytes.is_empty() {
                self.send(&bytes)?;
            }
            let at = self.now();
            self.pointer_sample(at, x, y);
            let next = began + POINTER_SAMPLE * i;
            self.sleep(next.saturating_sub(self.now()))?;
        }
        self.pointer = Some((end.x, end.y));
        Ok(())
    }

    fn mouse(
        &mut self,
        action: MouseAction,
        button: MouseButton,
        at: CellPoint,
    ) -> Result<Duration, DriverError> {
        let bytes = self
            .session
            .terminal()
            .mouse_bytes(action, button, KeyMods::NONE, at);
        if bytes.is_empty() {
            return Ok(self.now());
        }
        self.send(&bytes)
    }

    fn reports_mouse(&mut self, what: &str) -> bool {
        let on = self.session.terminal().mouse_reporting();
        if !on {
            (self.log)(format!(
                "The program in the terminal isn't reading the mouse, so the {what} did nothing"
            ));
        }
        on
    }

    fn click(
        &mut self,
        step: &TerminalStep,
        target: &CellTarget,
        button: PointerButton,
        double: bool,
    ) -> Result<(), DriverError> {
        let cell = self.target(target)?;
        self.glide(cell, MouseButton::None)?;
        let reports = self.reports_mouse("click");
        let pressed = mouse_button(button);
        let down = self.now();
        let mut second = None;
        for i in 0..if double { 2 } else { 1 } {
            if i == 1 {
                self.sleep(DOUBLE_CLICK_GAP)?;
                second = Some(millis(self.now()));
            }
            if reports {
                self.mouse(MouseAction::Press, pressed, cell)?;
            }
            self.sleep(PRESS_HOLD)?;
            if reports {
                self.mouse(MouseAction::Release, pressed, cell)?;
            }
        }
        let mut action = Action::new(step.id.clone(), ActionKind::Click, millis(self.now()));
        action.t_down = Some(millis(down));
        action.t_second_down = second;
        action.button = (button == PointerButton::Right).then_some(ClickButton::Right);
        action.point = Some(cell_point(cell));
        action.rect = Some(cell_rect(cell.col, cell.row, 1, 1));
        action.label = Some(step.label.clone());
        action.node_id = Some(step.node_id.clone());
        self.rec.actions.push(action);
        Ok(())
    }

    fn drag(
        &mut self,
        step: &TerminalStep,
        from: &CellTarget,
        to: &CellTarget,
    ) -> Result<(), DriverError> {
        let start = self.target(from)?;
        let end = self.target(to)?;
        self.glide(start, MouseButton::None)?;
        let reports = self.reports_mouse("drag");
        let down = self.now();
        if reports {
            self.mouse(MouseAction::Press, MouseButton::Left, start)?;
        }
        self.sleep(PRESS_HOLD)?;
        let from_sample = self.rec.pointer.len();
        self.glide(
            end,
            if reports {
                MouseButton::Left
            } else {
                MouseButton::None
            },
        )?;
        if reports {
            self.mouse(MouseAction::Release, MouseButton::Left, end)?;
        }
        let path = self.rec.pointer[from_sample..]
            .iter()
            .map(|s| Point { x: s.x, y: s.y })
            .collect();
        let mut action = Action::new(step.id.clone(), ActionKind::Drag, millis(self.now()));
        action.t_down = Some(millis(down));
        action.point = Some(cell_point(start));
        action.rect = Some(cell_rect(start.col, start.row, 1, 1));
        action.drop = Some(cell_rect(end.col, end.row, 1, 1));
        action.path = Some(path);
        action.node_id = Some(step.node_id.clone());
        self.rec.actions.push(action);
        Ok(())
    }

    fn wheel(
        &mut self,
        step: &TerminalStep,
        target: &CellTarget,
        notches: i32,
    ) -> Result<(), DriverError> {
        let cell = self.target(target)?;
        self.glide(cell, MouseButton::None)?;
        let reports = self.reports_mouse("wheel turn");
        let button = if notches < 0 {
            MouseButton::WheelUp
        } else {
            MouseButton::WheelDown
        };
        let t = self.now();
        for _ in 0..notches.unsigned_abs() {
            if reports {
                self.mouse(MouseAction::Press, button, cell)?;
            }
            self.sleep(DOUBLE_CLICK_GAP)?;
        }
        let mut action = Action::new(step.id.clone(), ActionKind::Scroll, millis(t));
        action.t_end = Some(millis(self.now()));
        action.point = Some(cell_point(cell));
        action.node_id = Some(step.node_id.clone());
        self.rec.actions.push(action);
        Ok(())
    }

    fn find_pattern(
        &self,
        pattern: &Regex,
        scope: LookIn,
    ) -> Result<Option<crate::input::Found>, DriverError> {
        let mut t = self.session.terminal();
        let frame = t.frame()?;
        let row = frame.cursor().y;
        let found = find_on_screen(frame, &|line| {
            pattern.find(line).map(|m| (m.start(), m.end()))
        });
        Ok(found.filter(|f| scope == LookIn::Screen || f.row == row))
    }

    fn wait_prompt(&mut self, limit_ms: u32) -> Result<(), DriverError> {
        let limit = ms(limit_ms);
        let back = if self.marks_seen {
            self.wait_until(limit, |d| d.at_prompt)?
        } else {
            self.wait_quiet(limit)?
        };
        if back {
            return Ok(());
        }
        Err(DriverError::Timeout(format!(
            "The prompt didn't come back within {} s",
            seconds(limit_ms)
        )))
    }

    fn wait_text(
        &mut self,
        step: &TerminalStep,
        pattern: &str,
        scope: LookIn,
        limit_ms: u32,
    ) -> Result<(), DriverError> {
        let regex = Regex::new(pattern).map_err(|e| DriverError::Pattern {
            pattern: pattern.to_owned(),
            reason: e.to_string(),
        })?;
        let mut found = None;
        let mut failure = None;
        let seen = self.wait_until(ms(limit_ms), |d| match d.find_pattern(&regex, scope) {
            Ok(Some(f)) => {
                found = Some(f);
                true
            }
            Ok(None) => false,
            Err(e) => {
                failure = Some(e);
                true
            }
        })?;
        if let Some(e) = failure {
            return Err(e);
        }
        let Some(found) = found.filter(|_| seen) else {
            return Err(DriverError::Timeout(format!(
                "“{pattern}” didn't appear within {} s",
                seconds(limit_ms)
            )));
        };
        let rect = cell_rect(found.from, found.row, found.to - found.from + 1, 1);
        let mut beat = Action::new(step.id.clone(), ActionKind::Focus, millis(self.now()));
        beat.point = Some(Point {
            x: rect.x + rect.w / 2.0,
            y: rect.y + rect.h / 2.0,
        });
        beat.rect = Some(rect);
        beat.node_id = Some(step.node_id.clone());
        self.rec.actions.push(beat);
        Ok(())
    }

    pub fn play(&mut self, step: &TerminalStep) -> Result<(), DriverError> {
        self.marker(format!("{STEP_PREFIX}{}", step.node_id));
        match &step.action {
            TerminalAction::Run {
                command,
                wait,
                limit_ms,
                zoom_output,
            } => {
                self.run_command(step, command, *wait, *limit_ms, *zoom_output)?;
            }
            TerminalAction::Type { text, speed_ms } => {
                let interval = self.typing_interval(*speed_ms);
                let action = self.typed_action(step, text, interval)?;
                self.rec.actions.push(action);
            }
            TerminalAction::Keys { keys, repeat } => self.press_keys(step, keys, *repeat)?,
            TerminalAction::Click {
                target,
                button,
                double,
            } => self.click(step, target, *button, *double)?,
            TerminalAction::Drag { from, to } => self.drag(step, from, to)?,
            TerminalAction::Wheel { target, notches } => self.wheel(step, target, *notches)?,
            TerminalAction::WaitText {
                pattern,
                scope,
                limit_ms,
            } => self.wait_text(step, pattern, *scope, *limit_ms)?,
            TerminalAction::WaitPrompt { limit_ms } => self.wait_prompt(*limit_ms)?,
            TerminalAction::Sleep { ms: length } => {
                self.sleep(ms(*length))?;
                return Ok(());
            }
            TerminalAction::StartRecording => self.marker(START),
            TerminalAction::Hide => self.marker(HIDE),
            TerminalAction::Show => self.marker(SHOW),
            TerminalAction::Chapter { title } => {
                self.marker(format!("{CHAPTER_PREFIX}{title}"));
                let mut chapter =
                    Action::new(step.id.clone(), ActionKind::Chapter, millis(self.now()));
                chapter.label = Some(title.clone());
                chapter.node_id = Some(step.node_id.clone());
                self.rec.actions.push(chapter);
            }
            TerminalAction::Clear => {
                self.key(
                    "l",
                    KeyMods {
                        ctrl: true,
                        ..KeyMods::NONE
                    },
                )?;
            }
            TerminalAction::Scroll { lines } => {
                self.session.terminal().scroll(*lines);
                self.marker(format!("{SCROLL_PREFIX}{lines}"));
            }
            TerminalAction::Copy { text } => self.clipboard.clone_from(text),
            TerminalAction::Paste => {
                let text = self.clipboard.clone();
                let bytes = self.session.terminal().paste_bytes(&text);
                self.send(&bytes)?;
            }
            TerminalAction::Screenshot => self.marker(SCREENSHOT),
            TerminalAction::Env { .. } | TerminalAction::Require { .. } => {}
        }
        self.sleep(ms(step.wait_after_ms))
    }

    pub const fn pointer_used(&self) -> bool {
        !self.rec.pointer.is_empty()
    }

    pub fn mark_end(&mut self, label: &str) {
        self.marker(label);
    }

    pub fn into_parts(self) -> (Session, Recording) {
        (self.session, self.rec)
    }
}

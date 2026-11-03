use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use opsis_engine::files::write_json;
use opsis_types::actions::{Action, ActionLog, CaretSample, CursorSample, TitleChange, Viewport};
use opsis_types::numbers::millis;
use opsis_types::terminal::{EnvVar, TerminalAction, TerminalSessionValue};

use crate::cast::{Event, Header, write_v3};
use crate::driver::{Driver, DriverError};
use crate::session::{Session, SessionError};
use crate::shell::{ShellError, require, shell_command};
use crate::timeline::{END, Timeline};

pub const CAST_FILE: &str = "take.cast";
pub const LOG_FILE: &str = "take.log.json";
const EXIT_LINE: &[u8] = b"exit\r";
const TERM_TYPE: &str = "xterm-256color";
const END_GAP: Duration = Duration::from_millis(50);

#[derive(Debug, thiserror::Error)]
pub enum TakeError {
    #[error(transparent)]
    Shell(#[from] ShellError),
    #[error(transparent)]
    Session(#[from] SessionError),
    #[error(transparent)]
    Driver(#[from] DriverError),
    #[error("Couldn't save the take: {0}")]
    Save(#[from] std::io::Error),
    #[error("The folder “{0}” doesn't exist")]
    Folder(String),
}

pub struct TakeOptions<'a> {
    pub session: &'a TerminalSessionValue,
    pub project_dir: &'a Path,
    pub temp: &'a Path,
    pub name: &'a str,
    pub pause_before_ms: u32,
    pub keep_after_ms: u32,
    pub cancelled: &'a dyn Fn() -> bool,
    pub log: &'a mut dyn FnMut(String),
}

impl std::fmt::Debug for TakeOptions<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TakeOptions")
            .field("session", &self.session)
            .field("project_dir", &self.project_dir)
            .field("temp", &self.temp)
            .field("name", &self.name)
            .finish_non_exhaustive()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Take {
    pub header: Header,
    pub events: Vec<Event>,
    pub log: ActionLog,
    pub length: Duration,
}

fn folder(project_dir: &Path, folder: &str) -> Result<PathBuf, TakeError> {
    let path = if folder.trim().is_empty() {
        project_dir.to_path_buf()
    } else {
        project_dir.join(folder.trim())
    };
    if path.is_dir() {
        Ok(path)
    } else {
        Err(TakeError::Folder(folder.trim().to_owned()))
    }
}

fn environment(value: &TerminalSessionValue) -> Vec<EnvVar> {
    let mut vars = value.env.clone();
    for step in &value.steps {
        if let TerminalAction::Env { name, value } = &step.action {
            vars.push(EnvVar {
                name: name.clone(),
                value: value.clone(),
            });
        }
    }
    vars
}

fn check_programs(value: &TerminalSessionValue) -> Result<(), ShellError> {
    for step in &value.steps {
        if let TerminalAction::Require { program } = &step.action {
            require(program)?;
        }
    }
    Ok(())
}

pub fn record_take(options: &mut TakeOptions<'_>) -> Result<Take, TakeError> {
    let value = options.session;
    check_programs(value)?;
    let cwd = folder(options.project_dir, &value.folder)?;
    let command = shell_command(value.shell, &cwd, options.temp, &environment(value))?;
    let session = Session::start(&command, value.cols, value.rows)?;
    let mut driver = Driver::new(session, value, options.cancelled, &mut *options.log);
    driver.wait_first_prompt()?;
    driver.sleep(Duration::from_millis(u64::from(options.pause_before_ms)))?;
    for step in &value.steps {
        driver.play(step)?;
    }
    driver.wait_last()?;
    driver.sleep(Duration::from_millis(u64::from(options.keep_after_ms)))?;
    driver.mark_end(END);
    driver.sleep(END_GAP)?;
    let pointer_used = driver.pointer_used();
    let (session, mut rec) = driver.into_parts();
    let finished = session.finish(EXIT_LINE);
    for chunk in finished.rest {
        rec.output(chunk.at, &chunk.bytes);
    }
    rec.finish_text(finished.ended);
    if let Some(code) = finished.exit {
        rec.events.push(Event::exit(finished.ended, code));
    }
    let events = rec.sorted_events();
    let timeline = Timeline::from_events(&events);
    let header = Header {
        term_cols: value.cols,
        term_rows: value.rows,
        term_type: Some(TERM_TYPE.to_owned()),
        timestamp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .ok()
            .map(|d| d.as_secs()),
        command: command
            .program
            .file_name()
            .map(|n| n.to_string_lossy().into_owned()),
        title: Some(options.name.to_owned()).filter(|n| !n.trim().is_empty()),
        ..Header::default()
    };
    let log = ActionLog {
        viewport: Viewport {
            width: u32::from(value.cols),
            height: u32::from(value.rows),
        },
        dpr: 1.0,
        t0: 0.0,
        duration_ms: millis(timeline.length()),
        actions: visible_actions(&rec.actions, &timeline),
        cursor_track: pointer_used.then(|| visible_pointer(&rec.pointer, &timeline)),
        addresses: Vec::new(),
        caret_track: visible_caret(&rec.caret, &timeline),
        titles: visible_titles(&rec.titles, &timeline),
        cells: None,
    };
    Ok(Take {
        header,
        length: timeline.length(),
        events,
        log,
    })
}

pub fn save_take(dir: &Path, take: &Take) -> Result<(), TakeError> {
    std::fs::create_dir_all(dir)?;
    opsis_engine::files::atomic_write(
        &dir.join(CAST_FILE),
        write_v3(&take.header, &take.events).as_bytes(),
    )?;
    write_json(&dir.join(LOG_FILE), &take.log)?;
    Ok(())
}

fn from_ms(t: f64) -> Duration {
    Duration::from_secs_f64(t.max(0.0) / 1000.0)
}

fn visible_ms(timeline: &Timeline, t: f64) -> Option<f64> {
    timeline.visible(from_ms(t)).map(millis)
}

pub fn visible_actions(actions: &[Action], timeline: &Timeline) -> Vec<Action> {
    actions
        .iter()
        .filter_map(|a| {
            let t = visible_ms(timeline, a.t)?;
            let map = |v: Option<f64>| v.and_then(|t| visible_ms(timeline, t));
            Some(Action {
                t,
                t_end: map(a.t_end),
                t_depart: map(a.t_depart),
                t_down: map(a.t_down),
                t_second_down: map(a.t_second_down),
                key_times: a.key_times.as_ref().map(|times| {
                    times
                        .iter()
                        .filter_map(|t| visible_ms(timeline, *t))
                        .collect()
                }),
                ..a.clone()
            })
        })
        .collect()
}

fn visible_samples<T: Clone>(
    samples: &[T],
    timeline: &Timeline,
    time: impl Fn(&T) -> f64,
    at: impl Fn(T, f64) -> T,
) -> Vec<T> {
    let start = millis(timeline.start());
    let mut out: Vec<T> = Vec::new();
    if let Some(before) = samples.iter().rev().find(|s| time(s) < start) {
        out.push(at(before.clone(), 0.0));
    }
    out.extend(
        samples
            .iter()
            .filter_map(|s| visible_ms(timeline, time(s)).map(|t| at(s.clone(), t))),
    );
    out
}

pub fn visible_caret(samples: &[CaretSample], timeline: &Timeline) -> Vec<CaretSample> {
    visible_samples(samples, timeline, |s| s.t, |s, t| CaretSample { t, ..s })
}

pub fn visible_titles(samples: &[TitleChange], timeline: &Timeline) -> Vec<TitleChange> {
    visible_samples(samples, timeline, |s| s.t, |s, t| TitleChange { t, ..s })
}

fn visible_pointer(samples: &[CursorSample], timeline: &Timeline) -> Vec<CursorSample> {
    visible_samples(samples, timeline, |s| s.t, |s, t| CursorSample { t, ..s })
}

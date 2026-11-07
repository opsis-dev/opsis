use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use opsis_audio::output::{Output, Preview};
use opsis_footage::chain::FootageSource;
use opsis_footage::decoder::{Decoder, FootageTools};
use opsis_media::grade::GradeChain;
use opsis_qt::{FrameRef, PlayerView};
use opsis_session::Session;
use opsis_session::state::AppState;

const LARGEST_WIDTH: u32 = 1280;
const SILENT_DB: f64 = -120.0;
const FRAME_WAIT: Duration = Duration::from_millis(15);
const MICROS: f64 = 1_000_000.0;

#[derive(Debug, Default)]
struct Clock {
    anchor: f64,
    started: Option<Instant>,
    rate: f64,
}

impl Clock {
    fn at(&self) -> f64 {
        self.started.map_or(self.anchor, |started| {
            started
                .elapsed()
                .as_secs_f64()
                .mul_add(self.rate, self.anchor)
        })
    }
}

#[derive(Debug, Default)]
struct Shown {
    serial: u64,
    width: u32,
    height: u32,
    rgb: Arc<Vec<u8>>,
}

#[derive(Debug, Default)]
struct Opened {
    ready: bool,
    failed: bool,
    has_video: bool,
    has_audio: bool,
    sound_ready: bool,
    duration: f64,
    width: u32,
    height: u32,
}

#[derive(Debug)]
struct Player {
    id: u64,
    file: PathBuf,
    temp: PathBuf,
    opened: Mutex<Opened>,
    clock: Mutex<Clock>,
    shown: Mutex<Shown>,
    audio: Mutex<Output>,
    generation: AtomicU64,
    volume: Mutex<(f64, bool)>,
}

#[derive(Debug, Clone)]
pub struct Players {
    open: Arc<Mutex<HashMap<u64, Arc<Player>>>>,
    next: Arc<AtomicU64>,
}

impl Default for Players {
    fn default() -> Self {
        Self {
            open: Arc::default(),
            next: Arc::new(AtomicU64::new(1)),
        }
    }
}

pub fn clock_text(seconds: f64, guide: f64) -> String {
    let negative = seconds < 0.0;
    let seconds = if seconds.is_finite() {
        seconds.abs()
    } else {
        0.0
    };
    let whole = opsis_types::numbers::truncate::<u64>(seconds.floor());
    let guide = if guide.is_finite() {
        guide.max(0.0)
    } else {
        0.0
    };
    let guide_whole = opsis_types::numbers::truncate::<u64>(guide.floor());
    let (h, m, s) = (whole / 3600, whole / 60 % 60, whole % 60);
    let hours = h > 0 || guide_whole / 3600 > 0;
    let sign = if negative { "-" } else { "" };
    if hours {
        format!("{sign}{h}:{m:02}:{s:02}")
    } else if guide_whole / 60 % 60 >= 10 {
        format!("{sign}{m:02}:{s:02}")
    } else {
        format!("{sign}{m}:{s:02}")
    }
}

fn volume_db(volume: f64, muted: bool) -> f64 {
    if muted || volume <= 0.0 {
        return SILENT_DB;
    }
    (20.0 * volume.min(1.0).log10()).max(SILENT_DB)
}

fn plain_source(file: &Path) -> FootageSource {
    FootageSource {
        file: file.to_owned(),
        grade: GradeChain {
            chain: format!("[0:v]scale=w=trunc(min({LARGEST_WIDTH}\\,iw)/2)*2:h=-2,setsar=1[out]"),
            extra_inputs: Vec::new(),
        },
        redactions: Vec::new(),
        trim_us: 0,
        loop_us: None,
    }
}

fn micros(seconds: f64) -> i64 {
    opsis_types::numbers::truncate::<i64>((seconds.max(0.0) * MICROS).round())
}

impl Player {
    fn position(&self) -> f64 {
        let duration = self.opened.lock().expect("player").duration;
        let heard = self.audio.lock().expect("player sound").clock();
        let at = heard.unwrap_or_else(|| self.clock.lock().expect("player clock").at());
        if duration > 0.0 {
            at.clamp(0.0, duration)
        } else {
            at.max(0.0)
        }
    }

    fn playing(&self) -> bool {
        self.clock.lock().expect("player clock").started.is_some()
    }

    fn stopped_at_end(&self) {
        let duration = self.opened.lock().expect("player").duration;
        if duration > 0.0 && self.playing() && self.position() >= duration {
            self.pause();
            self.clock.lock().expect("player clock").anchor = duration;
        }
    }

    fn pause(&self) {
        let at = self.position();
        {
            let mut clock = self.clock.lock().expect("player clock");
            clock.anchor = at;
            clock.started = None;
        }
        self.audio.lock().expect("player sound").pause();
        self.generation.fetch_add(1, Ordering::SeqCst);
    }

    fn store(&self, width: u32, height: u32, rgb: Vec<u8>) {
        let mut shown = self.shown.lock().expect("player frame");
        shown.serial += 1;
        shown.width = width;
        shown.height = height;
        shown.rgb = Arc::new(rgb);
        drop(shown);
        opsis_qt::wake(self.id);
    }

    fn changed(&self, change: impl FnOnce(&mut Opened)) {
        change(&mut self.opened.lock().expect("player"));
        opsis_qt::wake(self.id);
    }

    fn tools(&self, ffmpeg: &Path) -> FootageTools {
        FootageTools {
            ffmpeg: ffmpeg.to_owned(),
            temp: self.temp.clone(),
        }
    }

    fn decode(self: &Arc<Self>, ffmpeg: PathBuf, from: f64, follow: bool) {
        let generation = self.generation.fetch_add(1, Ordering::SeqCst) + 1;
        if !self.opened.lock().expect("player").has_video {
            return;
        }
        let player = Arc::clone(self);
        let spawned = std::thread::Builder::new()
            .name("player frames".to_owned())
            .spawn(move || player.frames(&ffmpeg, from, follow, generation));
        if let Err(error) = spawned {
            log::warn!("starting the player's frames: {error}");
        }
    }

    fn current(&self, generation: u64) -> bool {
        self.generation.load(Ordering::SeqCst) == generation
    }

    fn frames(&self, ffmpeg: &Path, from: f64, follow: bool, generation: u64) {
        if let Err(error) = std::fs::create_dir_all(&self.temp) {
            log::warn!("making the player's temp folder: {error}");
            return;
        }
        let mut decoder =
            match Decoder::start(&self.tools(ffmpeg), &plain_source(&self.file), micros(from)) {
                Ok(decoder) => decoder,
                Err(error) => {
                    log::warn!("starting the player's decoder: {error}");
                    return;
                }
            };
        loop {
            if !self.current(generation) {
                return;
            }
            let frame = match decoder.next_frame() {
                Ok(Some(frame)) => frame,
                Ok(None) => return,
                Err(error) => {
                    log::warn!("decoding the player's frames: {error}");
                    return;
                }
            };
            let at = u64::try_from(frame.pts_us.max(0))
                .map_or(0.0, opsis_types::numbers::float_wide)
                / MICROS;
            if follow {
                while self.current(generation) && self.position() < at {
                    std::thread::sleep(FRAME_WAIT);
                }
                if !self.current(generation) {
                    return;
                }
            } else if at + 1.0 / 30.0 < from {
                continue;
            }
            let (width, height) = frame.image.dimensions();
            self.store(width, height, frame.image.into_raw());
            if !follow {
                return;
            }
        }
    }
}

fn ffmpeg_of(session: &Session) -> Option<PathBuf> {
    opsis_session::engine::engine_files(session.state::<AppState>()).ffmpeg
}

fn temp_of(session: &Session, id: u64) -> PathBuf {
    let paths = &session.state::<AppState>().paths;
    paths.temp_root.join(opsis_engine::temp::child_temp_name(
        opsis_engine::temp::ChildKind::Analysis,
        &format!("player{id}"),
        std::process::id(),
    ))
}

const fn hidden() -> PlayerView {
    PlayerView {
        ready: false,
        failed: true,
        has_video: false,
        has_audio: false,
        sound_ready: false,
        playing: false,
        duration: 0.0,
        position: 0.0,
        width: 0,
        height: 0,
        serial: 0,
    }
}

impl Players {
    fn get(&self, id: u64) -> Option<Arc<Player>> {
        self.open.lock().expect("players").get(&id).cloned()
    }

    pub fn open(&self, session: &Session, handle: &tokio::runtime::Handle, file: &str) -> u64 {
        let id = self.next.fetch_add(1, Ordering::SeqCst);
        let player = Arc::new(Player {
            id,
            file: PathBuf::from(file),
            temp: temp_of(session, id),
            opened: Mutex::default(),
            clock: Mutex::new(Clock {
                rate: 1.0,
                ..Clock::default()
            }),
            shown: Mutex::default(),
            audio: Mutex::new(Output::new()),
            generation: AtomicU64::new(0),
            volume: Mutex::new((1.0, false)),
        });
        self.open
            .lock()
            .expect("players")
            .insert(id, Arc::clone(&player));
        let session = session.clone();
        handle.spawn(async move { load(&session, &player).await });
        id
    }

    pub fn close(&self, id: u64) {
        let Some(player) = self.open.lock().expect("players").remove(&id) else {
            return;
        };
        player.pause();
        opsis_engine::temp::remove_child_temp(&player.temp);
    }

    pub fn close_all(&self) {
        let ids: Vec<u64> = self.open.lock().expect("players").keys().copied().collect();
        for id in ids {
            self.close(id);
        }
    }

    pub fn view(&self, id: u64) -> PlayerView {
        let Some(player) = self.get(id) else {
            return hidden();
        };
        player.stopped_at_end();
        let opened = player.opened.lock().expect("player");
        let (ready, failed, has_video, has_audio, sound_ready, duration, width, height) = (
            opened.ready,
            opened.failed,
            opened.has_video,
            opened.has_audio,
            opened.sound_ready,
            opened.duration,
            opened.width,
            opened.height,
        );
        drop(opened);
        PlayerView {
            ready,
            failed,
            has_video,
            has_audio,
            sound_ready,
            playing: player.playing(),
            duration,
            position: player.position(),
            width,
            height,
            serial: player.shown.lock().expect("player frame").serial,
        }
    }

    pub fn frame(&self, id: u64) -> FrameRef {
        let Some(player) = self.get(id) else {
            return FrameRef::empty();
        };
        let shown = player.shown.lock().expect("player frame");
        FrameRef::rgb(
            Arc::clone(&shown.rgb),
            shown.width,
            shown.height,
            shown.serial,
        )
    }

    pub fn play(&self, session: &Session, id: u64) {
        let Some(player) = self.get(id) else {
            return;
        };
        if !player.opened.lock().expect("player").ready || player.playing() {
            return;
        }
        let duration = player.opened.lock().expect("player").duration;
        let mut from = player.position();
        if duration > 0.0 && from >= duration {
            from = 0.0;
        }
        let rate = {
            let mut clock = player.clock.lock().expect("player clock");
            clock.anchor = from;
            clock.started = Some(Instant::now());
            clock.rate
        };
        player.audio.lock().expect("player sound").play(from, rate);
        if let Some(ffmpeg) = ffmpeg_of(session) {
            player.decode(ffmpeg, from, true);
        }
    }

    pub fn pause(&self, id: u64) {
        if let Some(player) = self.get(id) {
            player.pause();
        }
    }

    pub fn seek(&self, session: &Session, id: u64, seconds: f64) {
        let Some(player) = self.get(id) else {
            return;
        };
        let duration = player.opened.lock().expect("player").duration;
        let to = if duration > 0.0 {
            seconds.clamp(0.0, duration)
        } else {
            seconds.max(0.0)
        };
        let playing = player.playing();
        {
            let mut clock = player.clock.lock().expect("player clock");
            clock.anchor = to;
            if playing {
                clock.started = Some(Instant::now());
            }
        }
        {
            let mut audio = player.audio.lock().expect("player sound");
            if playing {
                let rate = player.clock.lock().expect("player clock").rate;
                audio.play(to, rate);
            } else {
                audio.seek(to);
            }
        }
        if let Some(ffmpeg) = ffmpeg_of(session) {
            player.decode(ffmpeg, to, playing);
        }
    }

    pub fn set_rate(&self, session: &Session, id: u64, rate: f64) {
        let Some(player) = self.get(id) else {
            return;
        };
        let at = player.position();
        let playing = player.playing();
        {
            let mut clock = player.clock.lock().expect("player clock");
            clock.anchor = at;
            clock.rate = rate;
            if playing {
                clock.started = Some(Instant::now());
            }
        }
        if playing {
            player.audio.lock().expect("player sound").play(at, rate);
            if let Some(ffmpeg) = ffmpeg_of(session) {
                player.decode(ffmpeg, at, true);
            }
        }
    }

    pub fn set_volume(&self, id: u64, volume: f64, muted: bool) {
        let Some(player) = self.get(id) else {
            return;
        };
        *player.volume.lock().expect("player volume") = (volume, muted);
        player
            .audio
            .lock()
            .expect("player sound")
            .set_preview(Preview {
                gain_db: volume_db(volume, muted),
                ..Preview::default()
            });
    }

    pub fn retry(&self, session: &Session, handle: &tokio::runtime::Handle, id: u64) {
        let Some(player) = self.get(id) else {
            return;
        };
        player.pause();
        *player.opened.lock().expect("player") = Opened::default();
        let session = session.clone();
        handle.spawn(async move { load(&session, &player).await });
    }
}

async fn load(session: &Session, player: &Arc<Player>) {
    let file = player.file.clone();
    let probed = tokio::task::spawn_blocking(move || opsis_probe::probe(&file)).await;
    let probe = match probed {
        Ok(Ok(probe)) => probe,
        Ok(Err(error)) => {
            log::info!(
                "the player couldn't read {}: {error}",
                player.file.display()
            );
            player.changed(|opened| opened.failed = true);
            return;
        }
        Err(error) => {
            log::warn!("reading the player's file: {error}");
            player.changed(|opened| opened.failed = true);
            return;
        }
    };
    player.changed(|opened| {
        opened.has_video = probe.video.is_some();
        opened.has_audio = probe.audio.is_some();
        opened.duration = probe.duration_ms.map_or(0.0, |ms| f64::from(ms) / 1000.0);
        if let Some(video) = &probe.video {
            opened.width = video.width;
            opened.height = video.height;
        }
        opened.ready = true;
    });
    if let Some(ffmpeg) = ffmpeg_of(session) {
        player.decode(ffmpeg, 0.0, false);
    }
    if probe.audio.is_none() {
        return;
    }
    let tools = opsis_session::engine::engine_tools(session.state::<AppState>());
    match opsis_audio::decode::decode_stem(tools.as_ref(), &player.file, &player.temp).await {
        Ok(samples) => {
            let (volume, muted) = *player.volume.lock().expect("player volume");
            let mut audio = player.audio.lock().expect("player sound");
            audio.set_samples(Some(Arc::new(samples)));
            audio.set_preview(Preview {
                gain_db: volume_db(volume, muted),
                ..Preview::default()
            });
            drop(audio);
            if player.playing() {
                let rate = player.clock.lock().expect("player clock").rate;
                let at = player.clock.lock().expect("player clock").at();
                player.audio.lock().expect("player sound").play(at, rate);
            }
            player.changed(|opened| opened.sound_ready = true);
        }
        Err(error) => {
            log::warn!("decoding the player's sound: {error}");
            opsis_qt::wake(player.id);
        }
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::clock_text;

    #[test]
    fn times_read_as_the_player_writes_them() {
        assert_eq!(clock_text(5.4, 12.0), "0:05");
        assert_eq!(clock_text(65.0, 700.0), "01:05");
        assert_eq!(clock_text(3725.0, 3725.0), "1:02:05");
        assert_eq!(clock_text(5.0, 4000.0), "0:00:05");
    }

    proptest! {
        #[test]
        fn a_time_reads_back_as_its_whole_seconds(seconds in 0u32..360_000, guide in 0u32..360_000) {
            let text = clock_text(f64::from(seconds) + 0.5, f64::from(guide));
            let parts: Vec<u64> = text.split(':').map(|p| p.parse().unwrap()).collect();
            let total = parts.iter().fold(0, |sum, part| sum * 60 + part);
            let seconds = u64::from(seconds);
            prop_assert_eq!(total, seconds);
            prop_assert!(parts.iter().skip(1).all(|p| *p < 60));
        }
    }
}

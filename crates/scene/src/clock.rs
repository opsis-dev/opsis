pub const AUTHORED_FPS: u32 = 30;

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Clock {
    pub frame: f64,
    pub fps: u32,
    pub duration_in_frames: f64,
}

fn boundary_slack(frame: f64, from: f64, length: f64) -> f64 {
    (length / 2.0).min(f64::EPSILON * frame.abs().max(from.abs()) * 4.0)
}

fn loop_start(frame: f64, length: f64, total: f64) -> f64 {
    let times = (total / length).ceil();
    let loops = frame / length;
    let nearest = js_round(loops);
    let at_boundary = (loops - nearest).abs() <= f64::EPSILON * 1.0_f64.max(loops.abs()) * 4.0;
    let iteration =
        0.0_f64.max((times - 1.0).min(if at_boundary { nearest } else { loops.floor() }));
    let start = if at_boundary {
        frame
    } else {
        iteration * length
    };
    0.0_f64.max(start.min(length * (times - 1.0)))
}

pub fn js_round(value: f64) -> f64 {
    let below = value.floor();
    if value - below >= 0.5 {
        below + 1.0
    } else {
        below
    }
}

impl Clock {
    pub const fn new(frame: f64, fps: u32, duration_in_frames: f64) -> Self {
        Self {
            frame,
            fps,
            duration_in_frames,
        }
    }

    pub fn seconds(&self) -> f64 {
        self.frame / f64::from(self.fps)
    }

    pub fn sequence(&self, from: f64, duration_in_frames: Option<f64>) -> Option<Self> {
        let duration = duration_in_frames.unwrap_or(f64::INFINITY);
        let frame = self.frame - from;
        let length = 0.0_f64.max(duration.min(self.duration_in_frames - from));
        let slack = boundary_slack(self.frame, from, duration);
        if frame < -slack || frame - duration >= -slack {
            return None;
        }
        Some(Self {
            frame,
            fps: self.fps,
            duration_in_frames: length,
        })
    }

    #[must_use]
    pub const fn freeze(&self, frame: f64) -> Self {
        Self {
            frame,
            fps: self.fps,
            duration_in_frames: self.duration_in_frames,
        }
    }

    pub fn loop_every(&self, duration_in_frames: f64) -> Option<Self> {
        let from = loop_start(self.frame, duration_in_frames, self.duration_in_frames);
        self.sequence(from, Some(duration_in_frames))
    }

    #[must_use]
    pub fn authored(&self) -> Self {
        let fps = f64::from(self.fps);
        let authored = f64::from(AUTHORED_FPS);
        Self {
            frame: (self.frame * authored) / fps,
            fps: AUTHORED_FPS,
            duration_in_frames: (self.duration_in_frames * authored) / fps,
        }
    }

    pub fn real_frames(&self, authored_frames: f64) -> f64 {
        (authored_frames * f64::from(self.fps)) / f64::from(AUTHORED_FPS)
    }
}

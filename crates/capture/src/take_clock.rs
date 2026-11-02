use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use opsis_types::numbers;

use crate::clock::wall_ms;

const SAME_TIME_MS: f64 = 1e-6;

pub type TickFuture<'a> = Pin<Box<dyn Future<Output = ()> + Send + 'a>>;

pub trait Stepper: Send + Sync {
    fn frame_ms(&self) -> f64;
    fn tick(&self, frame: u64, at: f64) -> TickFuture<'_>;
}

enum Mode {
    Real,
    Stepped {
        stepper: Arc<dyn Stepper>,
        origin: f64,
        frame: u64,
    },
    Stopped {
        at: f64,
    },
}

impl Mode {
    fn now(&self) -> f64 {
        match self {
            Self::Real => wall_ms(),
            Self::Stepped {
                stepper,
                origin,
                frame,
            } => frame_time(*origin, *frame, stepper.frame_ms()),
            Self::Stopped { at } => *at,
        }
    }
}

pub const fn frame_time(origin: f64, frame: u64, frame_ms: f64) -> f64 {
    numbers::float_wide(frame).mul_add(frame_ms, origin)
}

#[derive(Clone)]
pub struct TakeClock {
    mode: Arc<Mutex<Mode>>,
    turn: Arc<tokio::sync::Mutex<()>>,
}

impl fmt::Debug for TakeClock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TakeClock")
            .field("turn", &self.turn)
            .finish_non_exhaustive()
    }
}

impl Default for TakeClock {
    fn default() -> Self {
        Self {
            mode: Arc::new(Mutex::new(Mode::Real)),
            turn: Arc::default(),
        }
    }
}

impl TakeClock {
    fn mode(&self) -> MutexGuard<'_, Mode> {
        self.mode.lock().expect("take clock")
    }

    pub fn now(&self) -> f64 {
        self.mode().now()
    }

    pub fn is_stepped(&self) -> bool {
        matches!(*self.mode(), Mode::Stepped { .. })
    }

    pub fn step_from(&self, origin: f64, stepper: Arc<dyn Stepper>) {
        *self.mode() = Mode::Stepped {
            stepper,
            origin,
            frame: 0,
        };
    }

    pub fn stop(&self) {
        let mut mode = self.mode();
        if matches!(*mode, Mode::Stepped { .. }) {
            *mode = Mode::Stopped { at: mode.now() };
        }
    }

    async fn tick(&self) -> bool {
        let _turn = self.turn.lock().await;
        let (stepper, frame, at) = match &*self.mode() {
            Mode::Stepped {
                stepper,
                origin,
                frame,
            } => {
                let next = frame + 1;
                (
                    Arc::clone(stepper),
                    next,
                    frame_time(*origin, next, stepper.frame_ms()),
                )
            }
            _ => return false,
        };
        stepper.tick(frame, at).await;
        if let Mode::Stepped { frame: current, .. } = &mut *self.mode() {
            *current = frame;
        }
        true
    }

    pub async fn wait(&self, duration: Duration) {
        let ms = duration.as_secs_f64() * 1000.0;
        let frame_ms = match &*self.mode() {
            Mode::Real => None,
            Mode::Stepped { stepper, .. } => Some(stepper.frame_ms()),
            Mode::Stopped { .. } => return,
        };
        let Some(frame_ms) = frame_ms else {
            tokio::time::sleep(duration).await;
            return;
        };
        if ms <= 0.0 {
            return;
        }
        let frames = (ms / frame_ms).round().max(1.0);
        self.sleep_until(frames.mul_add(frame_ms, self.now())).await;
    }

    pub async fn sleep_until(&self, at: f64) {
        if matches!(*self.mode(), Mode::Real) {
            let left = at - wall_ms();
            if left > 0.0 {
                tokio::time::sleep(Duration::from_secs_f64(left / 1000.0)).await;
            }
            return;
        }
        while self.now() + SAME_TIME_MS < at {
            if !self.tick().await {
                return;
            }
        }
    }

    pub async fn until<T, F, Fut>(
        &self,
        timeout: Duration,
        poll: Duration,
        mut check: F,
    ) -> Option<T>
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = Option<T>>,
    {
        let start = self.now();
        let timeout_ms = timeout.as_secs_f64() * 1000.0;
        loop {
            if let Some(found) = check().await {
                return Some(found);
            }
            if self.now() - start + SAME_TIME_MS >= timeout_ms
                || matches!(*self.mode(), Mode::Stopped { .. })
            {
                return None;
            }
            self.wait(poll).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::*;

    struct Counted {
        frame_ms: f64,
        ticks: AtomicU64,
        last_at: Mutex<f64>,
    }

    impl Stepper for Counted {
        fn frame_ms(&self) -> f64 {
            self.frame_ms
        }

        fn tick(&self, frame: u64, at: f64) -> TickFuture<'_> {
            Box::pin(async move {
                tokio::task::yield_now().await;
                self.ticks.fetch_add(1, Ordering::SeqCst);
                assert_eq!(self.ticks.load(Ordering::SeqCst), frame);
                *self.last_at.lock().unwrap() = at;
            })
        }
    }

    fn stepped(fps: f64) -> (TakeClock, Arc<Counted>) {
        let clock = TakeClock::default();
        let counted = Arc::new(Counted {
            frame_ms: 1000.0 / fps,
            ticks: AtomicU64::new(0),
            last_at: Mutex::new(0.0),
        });
        clock.step_from(1000.0, Arc::clone(&counted) as Arc<dyn Stepper>);
        (clock, counted)
    }

    #[tokio::test]
    async fn a_wait_is_whole_frames_and_never_less_than_one() {
        let (clock, counted) = stepped(30.0);
        clock.wait(Duration::from_millis(220)).await;
        assert_eq!(counted.ticks.load(Ordering::SeqCst), 7);
        clock.wait(Duration::from_millis(8)).await;
        assert_eq!(counted.ticks.load(Ordering::SeqCst), 8);
        clock.wait(Duration::ZERO).await;
        assert_eq!(counted.ticks.load(Ordering::SeqCst), 8);
        assert_eq!(clock.now(), frame_time(1000.0, 8, 1000.0 / 30.0));
        assert_eq!(*counted.last_at.lock().unwrap(), clock.now());
    }

    #[tokio::test]
    async fn sleeping_until_a_time_already_passed_takes_no_frame() {
        let (clock, counted) = stepped(30.0);
        let start = clock.now();
        for i in 0..8 {
            clock
                .sleep_until(f64::mul_add(f64::from(i), 16.0, start))
                .await;
        }
        assert_eq!(counted.ticks.load(Ordering::SeqCst), 4);
    }

    #[tokio::test]
    async fn two_waits_at_once_share_their_frames() {
        let (clock, counted) = stepped(25.0);
        tokio::join!(
            clock.wait(Duration::from_millis(400)),
            clock.wait(Duration::from_millis(200))
        );
        assert_eq!(counted.ticks.load(Ordering::SeqCst), 10);
    }

    #[tokio::test]
    async fn waits_for_a_check_in_clock_time() {
        let (clock, counted) = stepped(30.0);
        let found = clock
            .until(Duration::from_secs(1), Duration::from_millis(16), || {
                let frame = counted.ticks.load(Ordering::SeqCst);
                async move { (frame >= 5).then_some(frame) }
            })
            .await;
        assert_eq!(found, Some(5));
        let never: Option<()> = clock
            .until(
                Duration::from_secs(1),
                Duration::from_millis(16),
                || async { None },
            )
            .await;
        assert_eq!(never, None);
        assert_eq!(counted.ticks.load(Ordering::SeqCst), 35);
    }

    #[tokio::test]
    async fn a_stopped_clock_keeps_its_time_and_never_waits() {
        let (clock, counted) = stepped(30.0);
        clock.wait(Duration::from_millis(100)).await;
        let at = clock.now();
        clock.stop();
        clock.wait(Duration::from_secs(60)).await;
        assert_eq!(clock.now(), at);
        assert_eq!(counted.ticks.load(Ordering::SeqCst), 3);
    }
}

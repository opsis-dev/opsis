use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use crate::clock::wall_time;

const NANOS_PER_SECOND: u128 = 1_000_000_000;

pub type TickFuture<'a> = Pin<Box<dyn Future<Output = ()> + Send + 'a>>;

pub trait Stepper: Send + Sync {
    fn fps(&self) -> u32;
    fn tick(&self, frame: u64, at: Duration) -> TickFuture<'_>;
}

enum Mode {
    Real,
    Stepped {
        stepper: Arc<dyn Stepper>,
        origin: Duration,
        frame: u64,
    },
    Stopped {
        at: Duration,
    },
}

impl Mode {
    fn now(&self) -> Duration {
        match self {
            Self::Real => wall_time(),
            Self::Stepped {
                stepper,
                origin,
                frame,
            } => frame_time(*origin, *frame, stepper.fps()),
            Self::Stopped { at } => *at,
        }
    }
}

pub fn frames_span(frames: u64, fps: u32) -> Duration {
    let nanos = u128::from(frames) * NANOS_PER_SECOND / u128::from(fps.max(1));
    Duration::from_nanos(u64::try_from(nanos).unwrap_or(u64::MAX))
}

pub fn frame_time(origin: Duration, frame: u64, fps: u32) -> Duration {
    origin + frames_span(frame, fps)
}

pub fn frames_in(duration: Duration, fps: u32) -> u64 {
    let scaled = duration.as_nanos() * u128::from(fps) + NANOS_PER_SECOND / 2;
    u64::try_from(scaled / NANOS_PER_SECOND).unwrap_or(u64::MAX)
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

    pub fn now(&self) -> Duration {
        self.mode().now()
    }

    pub fn is_stepped(&self) -> bool {
        matches!(*self.mode(), Mode::Stepped { .. })
    }

    fn frame(&self) -> Option<u64> {
        match &*self.mode() {
            Mode::Stepped { frame, .. } => Some(*frame),
            _ => None,
        }
    }

    pub fn step_from(&self, origin: Duration, stepper: Arc<dyn Stepper>) {
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
                    frame_time(*origin, next, stepper.fps()),
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
        let fps = match &*self.mode() {
            Mode::Real => None,
            Mode::Stepped { stepper, .. } => Some(stepper.fps()),
            Mode::Stopped { .. } => return,
        };
        let Some(fps) = fps else {
            tokio::time::sleep(duration).await;
            return;
        };
        if duration.is_zero() {
            return;
        }
        let Some(start) = self.frame() else {
            return;
        };
        let target = start.saturating_add(frames_in(duration, fps).max(1));
        while self.frame().is_some_and(|frame| frame < target) {
            if !self.tick().await {
                return;
            }
        }
    }

    pub async fn sleep_until(&self, at: Duration) {
        if matches!(*self.mode(), Mode::Real) {
            let left = at.saturating_sub(wall_time());
            if !left.is_zero() {
                tokio::time::sleep(left).await;
            }
            return;
        }
        while self.now() < at {
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
        let deadline = self.now() + timeout;
        loop {
            if let Some(found) = check().await {
                return Some(found);
            }
            if self.now() >= deadline || matches!(*self.mode(), Mode::Stopped { .. }) {
                return None;
            }
            self.wait(poll).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU64, Ordering};

    use proptest::prelude::*;

    use super::*;

    struct Counted {
        fps: u32,
        ticks: AtomicU64,
        last_at: Mutex<Duration>,
    }

    impl Stepper for Counted {
        fn fps(&self) -> u32 {
            self.fps
        }

        fn tick(&self, frame: u64, at: Duration) -> TickFuture<'_> {
            Box::pin(async move {
                tokio::task::yield_now().await;
                self.ticks.fetch_add(1, Ordering::SeqCst);
                assert_eq!(self.ticks.load(Ordering::SeqCst), frame);
                *self.last_at.lock().unwrap() = at;
            })
        }
    }

    const ORIGIN: Duration = Duration::from_secs(1);

    fn stepped(fps: u32) -> (TakeClock, Arc<Counted>) {
        let clock = TakeClock::default();
        let counted = Arc::new(Counted {
            fps,
            ticks: AtomicU64::new(0),
            last_at: Mutex::new(Duration::ZERO),
        });
        clock.step_from(ORIGIN, Arc::clone(&counted) as Arc<dyn Stepper>);
        (clock, counted)
    }

    proptest! {
        #[test]
        fn frame_times_never_drift_from_the_frame_rate(
            fps in prop::sample::select(vec![12u32, 24, 25, 30, 48, 50, 60, 120]),
            frame in 0u64..10_000_000,
        ) {
            let at = frames_span(frame, fps);
            let exact = u128::from(frame) * NANOS_PER_SECOND;
            let scaled = at.as_nanos() * u128::from(fps);
            prop_assert!(scaled <= exact && exact < scaled + u128::from(fps));
        }

        #[test]
        fn a_span_of_whole_frames_holds_that_many_frames(
            fps in prop::sample::select(vec![12u32, 24, 25, 30, 48, 50, 60, 120]),
            frames in 0u64..10_000_000,
        ) {
            prop_assert_eq!(frames_in(frames_span(frames, fps), fps), frames);
        }
    }

    #[tokio::test]
    async fn a_wait_is_whole_frames_and_never_less_than_one() {
        let (clock, counted) = stepped(30);
        clock.wait(Duration::from_millis(220)).await;
        assert_eq!(counted.ticks.load(Ordering::SeqCst), 7);
        clock.wait(Duration::from_millis(8)).await;
        assert_eq!(counted.ticks.load(Ordering::SeqCst), 8);
        clock.wait(Duration::ZERO).await;
        assert_eq!(counted.ticks.load(Ordering::SeqCst), 8);
        assert_eq!(clock.now(), frame_time(ORIGIN, 8, 30));
        assert_eq!(*counted.last_at.lock().unwrap(), clock.now());
    }

    #[tokio::test]
    async fn sleeping_until_a_time_already_passed_takes_no_frame() {
        let (clock, counted) = stepped(30);
        let start = clock.now();
        for i in 0..8 {
            clock
                .sleep_until(start + Duration::from_millis(16) * i)
                .await;
        }
        assert_eq!(counted.ticks.load(Ordering::SeqCst), 4);
    }

    #[tokio::test]
    async fn two_waits_at_once_share_their_frames() {
        let (clock, counted) = stepped(25);
        tokio::join!(
            clock.wait(Duration::from_millis(400)),
            clock.wait(Duration::from_millis(200))
        );
        assert_eq!(counted.ticks.load(Ordering::SeqCst), 10);
    }

    #[tokio::test]
    async fn waits_for_a_check_in_clock_time() {
        let (clock, counted) = stepped(30);
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
        let (clock, counted) = stepped(30);
        clock.wait(Duration::from_millis(100)).await;
        let at = clock.now();
        clock.stop();
        clock.wait(Duration::from_secs(60)).await;
        assert_eq!(clock.now(), at);
        assert_eq!(counted.ticks.load(Ordering::SeqCst), 3);
    }
}

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use opsis_render::chunks::{Schedule, Worker, render_in_order};
use proptest::prelude::*;

struct Painter {
    fail_at: Option<u32>,
    drawn: Arc<AtomicUsize>,
}

impl Worker for Painter {
    type Error = String;

    fn draw(&mut self, frame: u32) -> Result<Vec<u8>, String> {
        if self.fail_at == Some(frame) {
            return Err(format!("frame {frame}"));
        }
        self.drawn.fetch_add(1, Ordering::SeqCst);
        Ok(frame.to_le_bytes().to_vec())
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn frames_reach_the_sink_once_each_in_order(
        count in 0u32..200,
        workers in 1usize..6,
        held in 1usize..12,
    ) {
        let drawn = Arc::new(AtomicUsize::new(0));
        let frames: Vec<u32> = (0..count).map(|f| f * 2).collect();
        let mut seen = Vec::new();
        let mut most_waiting = 0;
        let schedule = Schedule { workers, held };
        let done: Result<(), String> = render_in_order(
            &frames,
            schedule,
            &|| Painter { fail_at: None, drawn: Arc::clone(&drawn) },
            &mut |frame, picture| {
                most_waiting = most_waiting.max(drawn.load(Ordering::SeqCst) - seen.len());
                assert_eq!(picture, frame.to_le_bytes().to_vec());
                seen.push(frame);
                Ok(())
            },
        );
        prop_assert!(done.is_ok());
        prop_assert_eq!(&seen, &frames);
        prop_assert!(most_waiting <= held + workers, "{most_waiting} drawn ahead with {held} held");
    }

    #[test]
    fn a_failing_frame_stops_the_render_with_its_error(count in 2u32..100, workers in 1usize..5, fail in 0u32..100) {
        let fail = fail % count;
        let frames: Vec<u32> = (0..count).collect();
        let mut seen = Vec::new();
        let done = render_in_order(
            &frames,
            Schedule { workers, held: 4 },
            &|| Painter { fail_at: Some(fail), drawn: Arc::new(AtomicUsize::new(0)) },
            &mut |frame, _| {
                seen.push(frame);
                Ok(())
            },
        );
        prop_assert_eq!(done, Err(format!("frame {fail}")));
        prop_assert!(seen.iter().all(|f| *f < fail));
    }
}

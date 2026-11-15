use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Condvar, Mutex, MutexGuard, PoisonError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Schedule {
    pub workers: usize,
    pub held: usize,
}

pub trait Worker {
    type Error;
    fn draw(&mut self, frame: u32) -> Result<Vec<u8>, Self::Error>;
}

#[derive(Debug)]
struct Shared<E> {
    ready: BTreeMap<usize, Vec<u8>>,
    next: usize,
    failed: Option<E>,
    stopped: bool,
}

impl<E> Shared<E> {
    const fn halted(&self) -> bool {
        self.stopped || self.failed.is_some()
    }
}

#[derive(Debug)]
struct Board<E> {
    shared: Mutex<Shared<E>>,
    room: Condvar,
    arrived: Condvar,
    held: usize,
}

impl<E> Board<E> {
    fn lock(&self) -> MutexGuard<'_, Shared<E>> {
        self.shared.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn wait_for_room(&self, position: usize) -> bool {
        let mut shared = self.lock();
        while !shared.halted() && position >= shared.next + self.held {
            shared = self
                .room
                .wait(shared)
                .unwrap_or_else(PoisonError::into_inner);
        }
        !shared.halted()
    }

    fn deliver(&self, position: usize, drawn: Result<Vec<u8>, E>) -> bool {
        let mut shared = self.lock();
        let kept = match drawn {
            Ok(picture) => {
                shared.ready.insert(position, picture);
                true
            }
            Err(error) => {
                if shared.failed.is_none() {
                    shared.failed = Some(error);
                }
                self.room.notify_all();
                false
            }
        };
        self.arrived.notify_all();
        kept
    }

    fn take(&self, position: usize) -> Option<Vec<u8>> {
        let mut shared = self.lock();
        loop {
            if let Some(picture) = shared.ready.remove(&position) {
                return Some(picture);
            }
            if shared.halted() {
                return None;
            }
            shared = self
                .arrived
                .wait(shared)
                .unwrap_or_else(PoisonError::into_inner);
        }
    }

    fn advance(&self, next: usize) {
        self.lock().next = next;
        self.room.notify_all();
    }

    fn stop(&self) {
        self.lock().stopped = true;
        self.room.notify_all();
    }
}

fn work<W: Worker>(worker: &mut W, frames: &[u32], taken: &AtomicUsize, board: &Board<W::Error>) {
    loop {
        let position = taken.fetch_add(1, Ordering::Relaxed);
        let Some(frame) = frames.get(position) else {
            return;
        };
        if !board.wait_for_room(position) || !board.deliver(position, worker.draw(*frame)) {
            return;
        }
    }
}

pub fn render_in_order<W, E>(
    frames: &[u32],
    schedule: Schedule,
    make: &(dyn Fn() -> W + Sync),
    sink: &mut dyn FnMut(u32, Vec<u8>) -> Result<(), E>,
) -> Result<(), E>
where
    W: Worker<Error = E>,
    E: Send,
{
    let taken = AtomicUsize::new(0);
    let board = Board {
        shared: Mutex::new(Shared {
            ready: BTreeMap::new(),
            next: 0,
            failed: None,
            stopped: false,
        }),
        room: Condvar::new(),
        arrived: Condvar::new(),
        held: schedule.held.max(1),
    };
    std::thread::scope(|scope| {
        for _ in 0..schedule.workers.max(1) {
            scope.spawn(|| work(&mut make(), frames, &taken, &board));
        }
        let mut result = Ok(());
        for (position, frame) in frames.iter().enumerate() {
            let Some(picture) = board.take(position) else {
                break;
            };
            if let Err(error) = sink(*frame, picture) {
                result = Err(error);
                board.stop();
                break;
            }
            board.advance(position + 1);
        }
        board.stop();
        let failed = board.lock().failed.take();
        match failed {
            Some(error) if result.is_ok() => Err(error),
            _ => result,
        }
    })
}

use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use opsis_types::script::ScriptLineKind;

pub type Sink = Rc<dyn Fn(ScriptLineKind, &str)>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stream {
    Out,
    Err,
}

impl Stream {
    const fn kind(self) -> ScriptLineKind {
        match self {
            Self::Out => ScriptLineKind::Print,
            Self::Err => ScriptLineKind::Warn,
        }
    }

    const fn slot(self) -> usize {
        match self {
            Self::Out => 0,
            Self::Err => 1,
        }
    }
}

pub struct Streams {
    sink: Sink,
    pending: RefCell<[String; 2]>,
}

impl fmt::Debug for Streams {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Streams")
            .field("pending", &self.pending)
            .finish_non_exhaustive()
    }
}

impl Streams {
    pub fn new(sink: Sink) -> Rc<Self> {
        Rc::new(Self {
            sink,
            pending: RefCell::default(),
        })
    }

    pub fn line(&self, kind: ScriptLineKind, text: &str) {
        for line in text.split('\n') {
            (self.sink)(kind, line.strip_suffix('\r').unwrap_or(line));
        }
    }

    pub fn write(&self, stream: Stream, text: &str) {
        let complete = {
            let mut pending = self.pending.borrow_mut();
            let buffer = &mut pending[stream.slot()];
            buffer.push_str(text);
            buffer.rfind('\n').map(|end| {
                let rest = buffer.split_off(end + 1);
                let mut done = std::mem::replace(buffer, rest);
                done.pop();
                done
            })
        };
        if let Some(done) = complete {
            self.line(stream.kind(), &done);
        }
    }

    pub fn flush(&self, stream: Stream) {
        let rest = std::mem::take(&mut self.pending.borrow_mut()[stream.slot()]);
        if !rest.is_empty() {
            self.line(stream.kind(), &rest);
        }
    }

    pub fn flush_all(&self) {
        self.flush(Stream::Out);
        self.flush(Stream::Err);
    }
}

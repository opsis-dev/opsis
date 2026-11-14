use opsis_compile::elements::terminal::view::{View, screen_size, terminal_view};
use opsis_layout::fonts::Fonts;
use opsis_layout::measure::Measurer;
use opsis_layout::{Images, Viewport, lay_out};
use opsis_paint::{Cache, Resources, paint};
use opsis_skia::{Rgba, Surface};
use opsis_types::fonts::FontFamilyDef;
use opsis_types::numbers::whole;
use opsis_types::terminal::TerminalLook;
use opsis_types::terminal_frames::Frames;

use crate::{Files, FilmError, register_fallbacks, register_family};

const VIEW_ID: &str = "live";

pub struct TerminalPainter<'a> {
    fonts: Fonts,
    files: Files<'a>,
    cache: Cache,
}

impl std::fmt::Debug for TerminalPainter<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TerminalPainter")
            .field("cache", &self.cache)
            .finish_non_exhaustive()
    }
}

const fn no_sizes(_: &str) -> Option<(f64, f64)> {
    None
}

impl<'a> TerminalPainter<'a> {
    pub fn new(files: Files<'a>, families: &[FontFamilyDef]) -> Result<Self, FilmError> {
        let mut fonts = Fonts::new();
        for family in families {
            register_family(&mut fonts, files, &family.family, family)?;
        }
        register_fallbacks(&mut fonts, files)?;
        Ok(Self {
            fonts,
            files,
            cache: Cache::default(),
        })
    }

    pub fn paint(&mut self, frames: &Frames, look: &TerminalLook) -> Result<Surface, FilmError> {
        let frame = frames.frames.first().ok_or(FilmError::EmptyTerminal)?;
        let size = screen_size(look, frame.cols, frame.rows);
        let viewport = Viewport {
            width: size.width,
            height: size.height,
        };
        let root = {
            let measurer = Measurer::new(&mut self.fonts, viewport);
            terminal_view(&View {
                frames,
                frame,
                look,
                cursor_on: true,
                measure: &measurer,
                id: VIEW_ID,
            })
        };
        self.fonts.next_frame();
        let laid = lay_out(
            &root,
            viewport,
            &mut self.fonts,
            &Images { size_of: &no_sizes },
        );
        let mut surface = Surface::new(
            whole(size.width.ceil()).max(1),
            whole(size.height.ceil()).max(1),
        )?;
        surface.clear(Rgba::TRANSPARENT);
        let mut cache = std::mem::take(&mut self.cache);
        cache.next_frame();
        let mut resources = Resources::with_cache(self.files, self.fonts.faces(), cache);
        paint(&root, &laid, viewport, &mut surface, &mut resources);
        self.cache = resources.into_cache();
        Ok(surface)
    }
}

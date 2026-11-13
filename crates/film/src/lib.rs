use std::cell::RefCell;
use std::collections::HashMap;

use opsis_compile::elements::library::FilmLibrary;
use opsis_compile::fonts::bundled_fonts;
use opsis_layout::fonts::{FontError, Fonts};
use opsis_layout::measure::Measurer;
use opsis_layout::{Images, Viewport, lay_out};
use opsis_paint::{Frames, Resources, paint};
use opsis_scene::drawing::Font;
use opsis_scene::measure::{AnimationTiming, Measure, MeasuredBox};
use opsis_scene::node::Node;
use opsis_scene::timeline::{DrawError, draw_film};
use opsis_skia::{Matrix44, Rgba, Surface, SurfaceError};
use opsis_types::fonts::FontFamilyDef;
use opsis_types::numbers::{exact_eq, narrow, whole};
use opsis_types::timeline::TimelineSpec;

pub const FALLBACK_FAMILIES: [&str; 2] = ["opsis-fallback", "opsis-terminal-symbols-2"];

pub type Files<'a> = &'a (dyn Fn(&str) -> Option<Vec<u8>> + Sync);
pub type Sizes<'a> = &'a dyn Fn(&str) -> Option<(f64, f64)>;

#[derive(Debug, thiserror::Error)]
pub enum FilmError {
    #[error("the font file {0} could not be read")]
    MissingFont(String),
    #[error(transparent)]
    Font(#[from] FontError),
    #[error(transparent)]
    Draw(#[from] DrawError),
    #[error(transparent)]
    Surface(#[from] SurfaceError),
}

pub struct Film<'a> {
    spec: &'a TimelineSpec,
    fonts: Fonts,
    files: Files<'a>,
    sizes: Sizes<'a>,
    frames: Option<Frames<'a>>,
    animations: RefCell<HashMap<String, Option<AnimationTiming>>>,
}

struct FilmMeasure<'f, 'a> {
    layout: Measurer<'f>,
    files: Files<'a>,
    animations: &'f RefCell<HashMap<String, Option<AnimationTiming>>>,
}

fn animation_timing(bytes: &[u8]) -> Option<AnimationTiming> {
    let json = opsis_paint::lottie::animation_json(bytes)?;
    let value: serde_json::Value = serde_json::from_slice(&json).ok()?;
    Some(AnimationTiming {
        in_point: value
            .get("ip")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(0.0),
        out_point: value.get("op")?.as_f64()?,
        frame_rate: value.get("fr")?.as_f64()?,
    })
}

impl Measure for FilmMeasure<'_, '_> {
    fn text_width(&self, text: &str, font: &Font, letter_spacing: f64) -> f64 {
        self.layout.text_width(text, font, letter_spacing)
    }

    fn boxes(&self, root: &Node) -> Vec<MeasuredBox> {
        self.layout.boxes(root)
    }

    fn text_ink(&self, text: &str, font: &Font) -> (f64, f64) {
        self.layout.text_ink(text, font)
    }

    fn animation(&self, source: &str) -> Option<AnimationTiming> {
        if let Some(known) = self.animations.borrow().get(source) {
            return *known;
        }
        let timing = (self.files)(source).and_then(|bytes| animation_timing(&bytes));
        self.animations
            .borrow_mut()
            .insert(source.to_owned(), timing);
        timing
    }
}

impl std::fmt::Debug for Film<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Film")
            .field("spec", &self.spec)
            .field("fonts", &self.fonts)
            .finish_non_exhaustive()
    }
}

fn subset_of(file: &str) -> Option<&str> {
    let stem = file.rsplit('/').next()?.strip_suffix(".woff2")?;
    let rest = stem.strip_prefix("noto-sans-")?;
    let mut parts = rest.rsplitn(3, '-');
    parts.next()?;
    parts.next()?;
    parts.next()
}

fn register_family(
    fonts: &mut Fonts,
    files: Files<'_>,
    name: &str,
    family: &FontFamilyDef,
) -> Result<(), FilmError> {
    for face in &family.faces {
        let bytes = files(&face.file).ok_or_else(|| FilmError::MissingFont(face.file.clone()))?;
        fonts.register(name, &bytes)?;
    }
    Ok(())
}

fn register_fallbacks(fonts: &mut Fonts, files: Files<'_>) -> Result<(), FilmError> {
    for id in FALLBACK_FAMILIES {
        let Some(family) = bundled_fonts().iter().find(|f| f.id == id) else {
            continue;
        };
        for face in &family.faces {
            let Some(subset) = subset_of(&face.file) else {
                continue;
            };
            let bytes =
                files(&face.file).ok_or_else(|| FilmError::MissingFont(face.file.clone()))?;
            fonts.register_fallback(subset, &bytes)?;
        }
    }
    Ok(())
}

pub fn scaled(size: u32, scale: f64) -> u32 {
    whole((f64::from(size) * scale).round()).max(1)
}

impl<'a> Film<'a> {
    pub fn new(
        spec: &'a TimelineSpec,
        files: Files<'a>,
        sizes: Sizes<'a>,
    ) -> Result<Self, FilmError> {
        let mut fonts = Fonts::new();
        for (name, family) in spec.fonts.iter().flat_map(|f| &f.families) {
            register_family(&mut fonts, files, name, family)?;
        }
        register_fallbacks(&mut fonts, files)?;
        Ok(Self {
            spec,
            fonts,
            files,
            sizes,
            frames: None,
            animations: RefCell::new(HashMap::new()),
        })
    }

    #[must_use]
    pub fn with_frames(mut self, frames: Frames<'a>) -> Self {
        self.frames = Some(frames);
        self
    }

    pub fn frame(&mut self, frame: f64) -> Result<Surface, FilmError> {
        self.scaled_frame(frame, 1.0)
    }

    pub fn scaled_frame(&mut self, frame: f64, scale: f64) -> Result<Surface, FilmError> {
        let (width, height) = (self.spec.width, self.spec.height);
        let viewport = Viewport {
            width: f64::from(width),
            height: f64::from(height),
        };
        let root = {
            let measurer = FilmMeasure {
                layout: Measurer::new(&mut self.fonts, viewport),
                files: self.files,
                animations: &self.animations,
            };
            draw_film(&FilmLibrary { measure: &measurer }, self.spec, frame)?
        };
        let laid = lay_out(
            &root,
            viewport,
            &mut self.fonts,
            &Images {
                size_of: self.sizes,
            },
        );
        let mut surface = Surface::new(scaled(width, scale), scaled(height, scale))?;
        if !exact_eq(scale, 1.0) {
            let s = narrow(scale);
            surface = surface.with_base(Matrix44::row_major([
                s, 0.0, 0.0, 0.0, 0.0, s, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
            ]));
        }
        surface.clear(Rgba::new(0.0, 0.0, 0.0, 1.0));
        let mut resources = Resources::new(self.files, self.fonts.faces());
        if let Some(frames) = self.frames {
            resources = resources.with_frames(frames);
        }
        paint(&root, &laid, viewport, &mut surface, &mut resources);
        Ok(surface)
    }
}

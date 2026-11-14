use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use opsis_compile::elements::library::FilmLibrary;
use opsis_compile::fonts::bundled_fonts;
use opsis_effects::{Effects, Setting, has_program};
use opsis_filament::Backend;
use opsis_layout::fonts::{FontError, Fonts};
use opsis_layout::measure::Measurer;
use opsis_layout::{Images, Viewport, lay_out};
use opsis_paint::{Frames, Pass, Programs, Resources, paint};
use opsis_scene::drawing::Font;
use opsis_scene::measure::{AnimationTiming, Measure, MeasuredBox};
use opsis_scene::node::{Node, Stage3d};
use opsis_scene::timeline::{DrawError, draw_film};
use opsis_skia::{Matrix44, Rgba, Surface, SurfaceError};
use opsis_stage3d::{Environment as StageEnvironment, Materials};
use opsis_types::fonts::FontFamilyDef;
use opsis_types::numbers::{exact_eq, narrow, whole};
use opsis_types::terminal_frames::{Frames as TerminalFrames, read as read_frames};
use opsis_types::timeline::TimelineSpec;

pub const FALLBACK_FAMILIES: [&str; 3] = [
    "opsis-fallback",
    "opsis-terminal-symbols-2",
    "opsis-fallback-dejavu",
];

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
    terminals: RefCell<HashMap<String, Option<Arc<TerminalFrames>>>>,
    effects: Option<Effects>,
    effects_tried: bool,
    staging: Staging,
}

const ROOM_REFLECTIONS: &str = "app:environments/room/ibl.ktx";
const ROOM_IRRADIANCE: &str = "app:environments/room/sh.json";

#[derive(Debug, Default)]
struct Staging {
    materials: Materials,
    environment: std::cell::OnceCell<Option<(Vec<u8>, Vec<f32>)>>,
}

struct FilmPrograms<'e, 'a> {
    effects: &'e mut Option<Effects>,
    tried: &'e mut bool,
    staging: &'e mut Staging,
    files: Files<'a>,
}

fn room_environment(files: Files<'_>) -> Option<(Vec<u8>, Vec<f32>)> {
    let reflections = files(ROOM_REFLECTIONS)?;
    let coefficients: Vec<[f32; 3]> = serde_json::from_slice(&files(ROOM_IRRADIANCE)?).ok()?;
    Some((reflections, coefficients.into_iter().flatten().collect()))
}

impl FilmPrograms<'_, '_> {
    fn engine(&mut self) -> Option<&mut Effects> {
        if self.effects.is_none() && !*self.tried {
            *self.tried = true;
            *self.effects = Effects::new(Backend::Vulkan)
                .map_err(|e| log::warn!("shader effects: {e}"))
                .ok();
        }
        self.effects.as_mut()
    }
}

impl Programs for FilmPrograms<'_, '_> {
    fn run(&mut self, pass: &Pass<'_>) -> Option<Vec<u8>> {
        if !has_program(pass.program) {
            return None;
        }
        let effects = self.engine()?;
        let inputs: Vec<&[u8]> = pass.inputs.iter().map(Vec::as_slice).collect();
        let settings: Vec<Setting> = pass
            .uniforms
            .iter()
            .map(|s| Setting {
                name: s.name.clone(),
                values: s.values.clone(),
                columns: s.columns,
            })
            .collect();
        effects
            .run(pass.program, &inputs, pass.size, &settings)
            .map_err(|e| log::warn!("the {} effect: {e}", pass.program))
            .ok()
    }

    fn stage(&mut self, stage: &Stage3d, size: (u32, u32)) -> Option<Vec<u8>> {
        let files = self.files;
        let environment = self
            .staging
            .environment
            .get_or_init(|| room_environment(files))
            .clone();
        let mut materials = std::mem::take(&mut self.staging.materials);
        let drawn = self.engine().map(|effects| {
            opsis_stage3d::render(
                effects.filament(),
                &mut materials,
                stage,
                size,
                environment
                    .as_ref()
                    .map(|(reflections, irradiance)| StageEnvironment {
                        reflections,
                        irradiance,
                    }),
            )
        });
        self.staging.materials = materials;
        drawn?.map_err(|e| log::warn!("a 3D stage: {e}")).ok()
    }
}

struct FilmMeasure<'f, 'a> {
    layout: Measurer<'f>,
    files: Files<'a>,
    animations: &'f RefCell<HashMap<String, Option<AnimationTiming>>>,
    terminals: &'f RefCell<HashMap<String, Option<Arc<TerminalFrames>>>>,
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

    fn terminal(&self, source: &str) -> Option<Arc<TerminalFrames>> {
        if let Some(known) = self.terminals.borrow().get(source) {
            return known.clone();
        }
        let frames = (self.files)(source)
            .and_then(|bytes| read_frames(&bytes).ok())
            .map(Arc::new);
        self.terminals
            .borrow_mut()
            .insert(source.to_owned(), frames.clone());
        frames
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
    let mut parts = stem.rsplitn(3, '-');
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
            terminals: RefCell::new(HashMap::new()),
            effects: None,
            effects_tried: false,
            staging: Staging::default(),
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
                terminals: &self.terminals,
            };
            draw_film(
                &FilmLibrary {
                    measure: &measurer,
                    reframe: self.spec.reframe.unwrap_or_default(),
                },
                self.spec,
                frame,
            )?
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
        let mut programs = FilmPrograms {
            effects: &mut self.effects,
            tried: &mut self.effects_tried,
            staging: &mut self.staging,
            files: self.files,
        };
        let mut resources =
            Resources::new(self.files, self.fonts.faces()).with_programs(&mut programs);
        if let Some(frames) = self.frames {
            resources = resources.with_frames(frames);
        }
        paint(&root, &laid, viewport, &mut surface, &mut resources);
        Ok(surface)
    }
}

mod terminal;
mod vector;

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use opsis_compile::elements::library::FilmLibrary;
use opsis_compile::fonts::bundled_fonts;
use opsis_layout::fonts::{FontError, Fonts};
use opsis_layout::measure::Measurer;
use opsis_layout::{Images, Viewport, lay_out};
use opsis_paint::{Cache, Frames, Pass, Programs, Resources, SharedCache, paint};
use opsis_renderer::{Renderer, RendererError, Setting as RendererSetting, StageInputs};
use opsis_scene::drawing::Font;
use opsis_scene::measure::{AnimationTiming, Measure, MeasuredBox};
use opsis_scene::node::{Node, Stage3d};
use opsis_scene::timeline::{DrawError, draw_film};
use opsis_scene::typeface::Typeface;
use opsis_scene::vector::Vector;
use opsis_skia::{BlendMode, Matrix44, Paint, RectF, Rgba, Sampling, Surface, SurfaceError};
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
pub type FootageTimes<'a> = &'a dyn Fn(&str, f64) -> f64;

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
    #[error("the terminal picture has no frame")]
    EmptyTerminal,
}

pub use opsis_renderer::{Driver, renderer_program};

pub use crate::terminal::TerminalPainter;

pub struct Film<'a> {
    spec: &'a TimelineSpec,
    fonts: Fonts,
    files: Files<'a>,
    sizes: Sizes<'a>,
    frames: Option<Frames<'a>>,
    footage_times: Option<FootageTimes<'a>>,
    shares_cache: bool,
    animations: RefCell<HashMap<String, Option<AnimationTiming>>>,
    terminals: RefCell<HashMap<String, Option<Arc<TerminalFrames>>>>,
    typefaces: RefCell<HashMap<String, Option<Arc<Typeface>>>>,
    vectors: RefCell<HashMap<String, Option<Arc<Vector>>>>,
    drawing: Drawing,
    renderer: Option<Renderer>,
    renderer_tried: bool,
    environment: std::cell::OnceCell<Option<(Vec<u8>, Vec<f32>)>>,
    cache: Cache,
    known_sizes: RefCell<HashMap<String, Option<(f64, f64)>>>,
    last: Option<Shown>,
}

struct Shown {
    hash: u64,
    scale: u64,
    picture: opsis_skia::Image,
}

fn scene_hash(root: &opsis_scene::node::Node) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    root.hash(&mut hasher);
    hasher.finish()
}

const ROOM_REFLECTIONS: &str = "app:environments/room/ibl.ktx";
const ROOM_IRRADIANCE: &str = "app:environments/room/sh.json";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Drawing {
    pub program: Option<PathBuf>,
    pub driver: Driver,
    pub temp: PathBuf,
}

impl Default for Drawing {
    fn default() -> Self {
        Self {
            program: renderer_program(),
            driver: Driver::Card,
            temp: PathBuf::new(),
        }
    }
}

struct FilmPrograms<'e, 'a> {
    drawing: &'e Drawing,
    renderer: &'e mut Option<Renderer>,
    tried: &'e mut bool,
    environment: &'e std::cell::OnceCell<Option<(Vec<u8>, Vec<f32>)>>,
    files: Files<'a>,
}

fn room_environment(files: Files<'_>) -> Option<(Vec<u8>, Vec<f32>)> {
    let reflections = files(ROOM_REFLECTIONS)?;
    let coefficients: Vec<[f32; 3]> = serde_json::from_slice(&files(ROOM_IRRADIANCE)?).ok()?;
    Some((reflections, coefficients.into_iter().flatten().collect()))
}

impl FilmPrograms<'_, '_> {
    fn renderer(&mut self) -> Option<&mut Renderer> {
        if self.renderer.is_none() && !*self.tried {
            *self.tried = true;
            *self.renderer = self
                .drawing
                .program
                .as_deref()
                .ok_or(RendererError::Missing)
                .and_then(|program| {
                    Renderer::start(program, &self.drawing.driver, &self.drawing.temp)
                })
                .map_err(|e| log::warn!("shader effects and 3D: {e}"))
                .ok();
        }
        self.renderer.as_mut()
    }

    fn failed(&mut self, what: &str, error: &RendererError) {
        log::warn!("{what}: {error}");
        if !matches!(error, RendererError::Failed(_)) {
            *self.renderer = None;
        }
    }
}

impl Programs for FilmPrograms<'_, '_> {
    fn run(&mut self, pass: &Pass<'_>) -> Option<Vec<u8>> {
        let inputs: Vec<&[u8]> = pass.inputs.iter().map(Vec::as_slice).collect();
        let settings: Vec<RendererSetting> = pass
            .uniforms
            .iter()
            .map(|s| RendererSetting {
                name: s.name.clone(),
                values: s.values.clone(),
                columns: s.columns,
            })
            .collect();
        let drawn = self
            .renderer()?
            .run(pass.program, &inputs, pass.size, &settings);
        drawn
            .map_err(|e| self.failed(&format!("the {} effect", pass.program), &e))
            .ok()
    }

    fn stage(
        &mut self,
        stage: &Stage3d,
        size: (u32, u32),
        textures: &[Option<opsis_paint::Picture>],
    ) -> Option<Vec<u8>> {
        let pictures: Vec<Option<(&[u8], u32, u32)>> = textures
            .iter()
            .map(|t| t.as_ref().map(|p| (p.rgba.as_slice(), p.width, p.height)))
            .collect();
        let files = self.files;
        let environment = self.environment.get_or_init(|| room_environment(files));
        let inputs = StageInputs {
            stage,
            size,
            pictures: &pictures,
            files: &|source: &str| files(source),
            environment: environment
                .as_ref()
                .map(|(reflections, irradiance)| (reflections.as_slice(), irradiance.as_slice())),
        };
        let drawn = self.renderer()?.stage(&inputs);
        drawn.map_err(|e| self.failed("a 3D stage", &e)).ok()
    }
}

struct FilmMeasure<'f, 'a> {
    layout: Measurer<'f>,
    files: Files<'a>,
    footage_times: Option<FootageTimes<'a>>,
    animations: &'f RefCell<HashMap<String, Option<AnimationTiming>>>,
    terminals: &'f RefCell<HashMap<String, Option<Arc<TerminalFrames>>>>,
    sizes: Sizes<'a>,
    typefaces: &'f RefCell<HashMap<String, Option<Arc<Typeface>>>>,
    vectors: &'f RefCell<HashMap<String, Option<Arc<Vector>>>>,
}

fn animation_timing(bytes: &[u8]) -> Option<AnimationTiming> {
    let json = opsis_probe::lottie::animation_json(bytes)?;
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

    fn picture_size(&self, source: &str) -> Option<(f64, f64)> {
        (self.sizes)(source)
    }

    fn footage_time(&self, base: &str, micros: f64) -> f64 {
        self.footage_times
            .map_or(micros, |snapped| snapped(base, micros))
    }

    fn typeface(&self, source: &str) -> Option<Arc<Typeface>> {
        if let Some(known) = self.typefaces.borrow().get(source) {
            return known.clone();
        }
        let typeface = (self.files)(source)
            .and_then(|bytes| Typeface::parse(&bytes))
            .map(Arc::new);
        self.typefaces
            .borrow_mut()
            .insert(source.to_owned(), typeface.clone());
        typeface
    }

    fn vector(&self, source: &str) -> Option<Arc<Vector>> {
        if let Some(known) = self.vectors.borrow().get(source) {
            return known.clone();
        }
        let vector = (self.files)(source)
            .and_then(|bytes| vector::read(&bytes))
            .map(Arc::new);
        self.vectors
            .borrow_mut()
            .insert(source.to_owned(), vector.clone());
        vector
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
            footage_times: None,
            shares_cache: false,
            animations: RefCell::new(HashMap::new()),
            terminals: RefCell::new(HashMap::new()),
            typefaces: RefCell::new(HashMap::new()),
            vectors: RefCell::new(HashMap::new()),
            drawing: Drawing::default(),
            renderer: None,
            renderer_tried: false,
            environment: std::cell::OnceCell::new(),
            cache: Cache::default(),
            known_sizes: RefCell::new(HashMap::new()),
            last: None,
        })
    }

    #[must_use]
    pub fn with_drawing(mut self, drawing: Drawing) -> Self {
        self.drawing = drawing;
        self
    }

    #[must_use]
    pub fn with_frames(mut self, frames: Frames<'a>) -> Self {
        self.frames = Some(frames);
        self
    }

    #[must_use]
    pub fn with_footage_times(mut self, footage_times: FootageTimes<'a>) -> Self {
        self.footage_times = Some(footage_times);
        self
    }

    #[must_use]
    pub fn with_shared(mut self, shared: Arc<SharedCache>) -> Self {
        self.cache = Cache::with_shared(shared);
        self.shares_cache = true;
        self
    }

    pub fn frame(&mut self, frame: f64) -> Result<Surface, FilmError> {
        self.scaled_frame(frame, 1.0)
    }

    fn viewport(&self) -> Viewport {
        Viewport {
            width: f64::from(self.spec.width),
            height: f64::from(self.spec.height),
        }
    }

    fn drawn_scene(
        &mut self,
        frame: f64,
        size_of: Sizes<'_>,
    ) -> Result<opsis_scene::node::Node, FilmError> {
        let viewport = self.viewport();
        let measurer = FilmMeasure {
            layout: Measurer::new(&mut self.fonts, viewport),
            files: self.files,
            footage_times: self.footage_times,
            animations: &self.animations,
            terminals: &self.terminals,
            sizes: size_of,
            typefaces: &self.typefaces,
            vectors: &self.vectors,
        };
        Ok(draw_film(
            &FilmLibrary {
                measure: &measurer,
                reframe: self.spec.reframe.unwrap_or_default(),
            },
            self.spec,
            frame,
        )?)
    }

    pub fn scene(&mut self, frame: f64) -> Result<opsis_scene::node::Node, FilmError> {
        let sizes = self.sizes;
        self.drawn_scene(frame, sizes)
    }

    pub fn scaled_frame(&mut self, frame: f64, scale: f64) -> Result<Surface, FilmError> {
        let (width, height) = (self.spec.width, self.spec.height);
        let viewport = self.viewport();
        let known_sizes = std::mem::take(&mut self.known_sizes);
        let sizes = self.sizes;
        let size_of = |source: &str| {
            if let Some(known) = known_sizes.borrow().get(source) {
                return *known;
            }
            let size = sizes(source);
            known_sizes.borrow_mut().insert(source.to_owned(), size);
            size
        };
        let drawn = self.drawn_scene(frame, &size_of);
        let root = match drawn {
            Ok(root) => root,
            Err(error) => {
                self.known_sizes = known_sizes;
                return Err(error);
            }
        };
        let hash = scene_hash(&root);
        if let Some(last) = &self.last
            && last.hash == hash
            && last.scale == scale.to_bits()
        {
            self.known_sizes = known_sizes;
            let mut surface = Surface::new(last.picture.width(), last.picture.height())?;
            let full = RectF::new(
                0.0,
                0.0,
                narrow(f64::from(last.picture.width())),
                narrow(f64::from(last.picture.height())),
            );
            let mut paint = Paint::new();
            paint.set_blend(BlendMode::Src);
            surface.draw_image_rect(&last.picture, (full, full), Sampling::Nearest, &paint);
            return Ok(surface);
        }
        self.fonts.next_frame();
        let laid = lay_out(
            &root,
            viewport,
            &mut self.fonts,
            &Images { size_of: &size_of },
        );
        self.known_sizes = known_sizes;
        let mut surface = Surface::new(scaled(width, scale), scaled(height, scale))?;
        if !exact_eq(scale, 1.0) {
            let s = narrow(scale);
            surface = surface.with_base(Matrix44::row_major([
                s, 0.0, 0.0, 0.0, 0.0, s, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
            ]));
        }
        surface.clear(Rgba::new(0.0, 0.0, 0.0, 1.0));
        let mut programs = FilmPrograms {
            drawing: &self.drawing,
            renderer: &mut self.renderer,
            tried: &mut self.renderer_tried,
            environment: &self.environment,
            files: self.files,
        };
        let mut cache = std::mem::take(&mut self.cache);
        cache.next_frame();
        if self.shares_cache {
            cache.stamp_with(u64::from(whole(frame)));
        }
        let mut resources = Resources::with_cache(self.files, self.fonts.faces(), cache)
            .with_programs(&mut programs);
        if let Some(frames) = self.frames {
            resources = resources.with_frames(frames);
        }
        paint(&root, &laid, viewport, &mut surface, &mut resources);
        self.cache = resources.into_cache();
        self.last = surface.snapshot().map(|picture| Shown {
            hash,
            scale: scale.to_bits(),
            picture,
        });
        Ok(surface)
    }
}

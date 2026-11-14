use std::collections::HashMap;
use std::sync::Arc;

use opsis_layout::output::TextRun;
use opsis_skia::{Axis, Image, Typeface};

type TypefaceKey = (u64, u32, Vec<([u8; 4], u32)>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Picture {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

pub type Frames<'a> = &'a (dyn Fn(&str) -> Option<Picture> + Sync);

#[derive(Debug, Clone, PartialEq)]
pub struct Setting {
    pub name: String,
    pub values: Vec<f32>,
    pub columns: usize,
}

impl Setting {
    pub fn of(name: &str, values: Vec<f32>) -> Self {
        Self {
            name: name.to_owned(),
            columns: values.len(),
            values,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Pass<'p> {
    pub program: &'p str,
    pub inputs: &'p [Vec<u8>],
    pub size: (u32, u32),
    pub uniforms: &'p [Setting],
}

pub trait Programs {
    fn run(&mut self, pass: &Pass<'_>) -> Option<Vec<u8>>;

    fn stage(
        &mut self,
        _stage: &opsis_scene::node::Stage3d,
        _size: (u32, u32),
        _textures: &[Option<Picture>],
    ) -> Option<Vec<u8>> {
        None
    }
}

const KEPT_FRAMES: u64 = 2;

struct Kept<T> {
    value: T,
    used: u64,
}

type Shelf<K, T> = HashMap<K, Kept<Option<T>>>;

fn kept<K: std::hash::Hash + Eq, T>(
    shelf: &mut Shelf<K, T>,
    key: K,
    frame: u64,
    make: impl FnOnce(&K) -> Option<T>,
) -> Option<&mut T> {
    let entry = shelf.entry(key).or_insert_with_key(|key| Kept {
        value: make(key),
        used: frame,
    });
    entry.used = frame;
    entry.value.as_mut()
}

fn swept<K, T>(shelf: &mut Shelf<K, T>, frame: u64) {
    shelf.retain(|_, kept| kept.used + KEPT_FRAMES >= frame);
}

type FontIds = HashMap<String, Vec<usvg::fontdb::ID>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LayerRegion {
    pub left: i32,
    pub top: i32,
    pub width: u32,
    pub height: u32,
}

impl LayerRegion {
    fn right(self) -> i64 {
        i64::from(self.left) + i64::from(self.width)
    }

    fn bottom(self) -> i64 {
        i64::from(self.top) + i64::from(self.height)
    }

    pub fn holds(self, other: Self) -> bool {
        self.left <= other.left
            && self.top <= other.top
            && self.right() >= other.right()
            && self.bottom() >= other.bottom()
    }
}

#[derive(Default)]
pub struct Cache {
    fonts: Option<(Arc<usvg::fontdb::Database>, Arc<FontIds>)>,
    typefaces: Shelf<TypefaceKey, Typeface>,
    images: Shelf<String, Image>,
    trees: Shelf<String, usvg::Tree>,
    animations: Shelf<String, opsis_skia::Lottie>,
    layers: Shelf<u64, (LayerRegion, Image)>,
    layers_drawn: u64,
    frame: u64,
}

impl std::fmt::Debug for Cache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Cache")
            .field("typefaces", &self.typefaces.len())
            .field("images", &self.images.len())
            .field("trees", &self.trees.len())
            .field("animations", &self.animations.len())
            .field("layers", &self.layers.len())
            .field("layers_drawn", &self.layers_drawn)
            .field("frame", &self.frame)
            .finish_non_exhaustive()
    }
}

impl Cache {
    pub const fn layers_drawn(&self) -> u64 {
        self.layers_drawn
    }

    pub fn next_frame(&mut self) {
        self.frame += 1;
        let frame = self.frame;
        swept(&mut self.typefaces, frame);
        swept(&mut self.images, frame);
        swept(&mut self.trees, frame);
        swept(&mut self.animations, frame);
        swept(&mut self.layers, frame);
    }

    fn fonts(
        &mut self,
        faces: &[(String, Arc<Vec<u8>>)],
    ) -> (Arc<usvg::fontdb::Database>, Arc<FontIds>) {
        self.fonts
            .get_or_insert_with(|| {
                let mut database = usvg::fontdb::Database::new();
                let mut by_name: FontIds = HashMap::new();
                for (name, bytes) in faces {
                    let source: Arc<dyn AsRef<[u8]> + Send + Sync> = bytes.clone();
                    let ids = database.load_font_source(usvg::fontdb::Source::Binary(source));
                    by_name.entry(name.clone()).or_default().extend(ids);
                }
                (Arc::new(database), Arc::new(by_name))
            })
            .clone()
    }
}

pub struct Resources<'a> {
    files: &'a (dyn Fn(&str) -> Option<Vec<u8>> + Sync),
    frames: Option<Frames<'a>>,
    programs: Option<&'a mut dyn Programs>,
    cache: Cache,
    options: usvg::Options<'a>,
}

fn picture_kind(source: &str, bytes: Vec<u8>) -> Option<usvg::ImageKind> {
    let extension = std::path::Path::new(source)
        .extension()?
        .to_str()?
        .to_ascii_lowercase();
    let data = Arc::new(bytes);
    match extension.as_str() {
        "jpg" | "jpeg" => Some(usvg::ImageKind::JPEG(data)),
        "png" => Some(usvg::ImageKind::PNG(data)),
        "webp" => Some(usvg::ImageKind::WEBP(data)),
        "gif" => Some(usvg::ImageKind::GIF(data)),
        "svg" => usvg::Tree::from_data(&data, &usvg::Options::default())
            .ok()
            .map(usvg::ImageKind::SVG),
        _ => None,
    }
}

fn options<'a>(
    faces: &[(String, Arc<Vec<u8>>)],
    (database, by_name): (Arc<usvg::fontdb::Database>, Arc<FontIds>),
    files: &'a (dyn Fn(&str) -> Option<Vec<u8>> + Sync),
) -> usvg::Options<'a> {
    let fallback: Option<usvg::fontdb::ID> = faces
        .first()
        .and_then(|(name, _)| by_name.get(name))
        .and_then(|ids| ids.first().copied());
    let select_font: usvg::FontSelectionFn<'static> = Box::new(move |font, db| {
        let wanted = match font.style() {
            usvg::FontStyle::Normal => usvg::fontdb::Style::Normal,
            usvg::FontStyle::Italic => usvg::fontdb::Style::Italic,
            usvg::FontStyle::Oblique => usvg::fontdb::Style::Oblique,
        };
        for family in font.families() {
            let usvg::FontFamily::Named(name) = family else {
                continue;
            };
            let Some(ids) = by_name.get(&name.to_lowercase()) else {
                continue;
            };
            let best = ids.iter().copied().min_by_key(|id| {
                db.face(*id).map_or(u16::MAX, |face| {
                    let style = u16::from(face.style != wanted) * 1000;
                    style + face.weight.0.abs_diff(font.weight())
                })
            });
            if best.is_some() {
                return best;
            }
        }
        fallback
    });
    let resolve_string: usvg::ImageHrefStringResolverFn<'a> =
        Box::new(move |href, _| picture_kind(href, files(href)?));
    usvg::Options {
        image_href_resolver: usvg::ImageHrefResolver {
            resolve_data: usvg::ImageHrefResolver::default_data_resolver(),
            resolve_string,
        },
        fontdb: database,
        font_resolver: usvg::FontResolver {
            select_font,
            select_fallback: usvg::FontResolver::default_fallback_selector(),
        },
        ..usvg::Options::default()
    }
}

impl std::fmt::Debug for Resources<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Resources")
            .field("cache", &self.cache)
            .finish_non_exhaustive()
    }
}

impl<'a> Resources<'a> {
    pub fn new(
        files: &'a (dyn Fn(&str) -> Option<Vec<u8>> + Sync),
        faces: &[(String, Arc<Vec<u8>>)],
    ) -> Self {
        Self::with_cache(files, faces, Cache::default())
    }

    pub fn with_cache(
        files: &'a (dyn Fn(&str) -> Option<Vec<u8>> + Sync),
        faces: &[(String, Arc<Vec<u8>>)],
        mut cache: Cache,
    ) -> Self {
        let fonts = cache.fonts(faces);
        Self {
            options: options(faces, fonts, files),
            files,
            frames: None,
            programs: None,
            cache,
        }
    }

    pub fn into_cache(self) -> Cache {
        self.cache
    }

    pub fn layer(&mut self, key: u64, needed: LayerRegion) -> Option<(LayerRegion, &Image)> {
        let frame = self.cache.frame;
        let kept = self.cache.layers.get_mut(&key)?;
        let (region, image) = kept.value.as_ref()?;
        if !region.holds(needed) {
            return None;
        }
        kept.used = frame;
        Some((*region, image))
    }

    pub fn keep_layer(&mut self, key: u64, region: LayerRegion, image: Image) {
        self.cache.layers_drawn += 1;
        self.cache.layers.insert(
            key,
            Kept {
                value: Some((region, image)),
                used: self.cache.frame,
            },
        );
    }

    #[must_use]
    pub fn with_frames(mut self, frames: Frames<'a>) -> Self {
        self.frames = Some(frames);
        self
    }

    #[must_use]
    pub fn with_programs(mut self, programs: &'a mut dyn Programs) -> Self {
        self.programs = Some(programs);
        self
    }

    pub fn run(&mut self, pass: &Pass<'_>) -> Option<Vec<u8>> {
        self.programs.as_deref_mut()?.run(pass)
    }

    pub fn stage(
        &mut self,
        stage: &opsis_scene::node::Stage3d,
        size: (u32, u32),
        textures: &[Option<Picture>],
    ) -> Option<Vec<u8>> {
        self.programs.as_deref_mut()?.stage(stage, size, textures)
    }

    pub fn texture(&mut self, source: &str) -> Option<Picture> {
        if Self::is_svg(source) {
            let tree = self.svg(source)?;
            let size = tree.size();
            let mut decode = |bytes: &[u8]| Image::decode(bytes);
            let image = crate::svg::render_picture(
                tree.root(),
                opsis_skia::RectF::new(0.0, 0.0, size.width(), size.height()),
                &mut decode,
            )?;
            return picture_of(&image);
        }
        picture_of(self.image(source)?)
    }

    pub fn typeface(&mut self, run: &TextRun) -> Option<&Typeface> {
        let key: TypefaceKey = (
            run.font.data.id(),
            run.font.index,
            run.variations
                .iter()
                .map(|(tag, value)| (*tag, value.to_bits()))
                .collect(),
        );
        kept(&mut self.cache.typefaces, key, self.cache.frame, |_| {
            let axes: Vec<Axis> = run
                .variations
                .iter()
                .map(|(tag, value)| Axis::new(*tag, *value))
                .collect();
            Typeface::from_bytes(run.font.data.data(), run.font.index, &axes)
        })
        .map(|t| &*t)
    }

    pub fn is_svg(source: &str) -> bool {
        source.starts_with("data:image/svg+xml")
            || std::path::Path::new(source)
                .extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("svg"))
    }

    pub fn svg(&mut self, source: &str) -> Option<&usvg::Tree> {
        let files = self.files;
        let options = &self.options;
        kept(
            &mut self.cache.trees,
            source.to_owned(),
            self.cache.frame,
            |_| {
                let bytes = source
                    .strip_prefix("data:")
                    .map_or_else(|| files(source), data_bytes)?;
                usvg::Tree::from_data(&bytes, options).ok()
            },
        )
        .map(|t| &*t)
    }

    pub const fn options(&self) -> &usvg::Options<'a> {
        &self.options
    }

    pub fn markup(&mut self, document: String) -> Option<&usvg::Tree> {
        let options = &self.options;
        kept(&mut self.cache.trees, document, self.cache.frame, |text| {
            usvg::Tree::from_str(text, options).ok()
        })
        .map(|t| &*t)
    }

    pub fn animation(&mut self, source: &str) -> Option<&mut opsis_skia::Lottie> {
        let files = self.files;
        kept(
            &mut self.cache.animations,
            source.to_owned(),
            self.cache.frame,
            |_| {
                let json = opsis_probe::lottie::animation_json(&files(source)?)?;
                opsis_skia::Lottie::from_json(&json)
            },
        )
    }

    pub fn image(&mut self, source: &str) -> Option<&Image> {
        let files = self.files;
        let frames = self.frames;
        kept(
            &mut self.cache.images,
            source.to_owned(),
            self.cache.frame,
            |_| {
                if let Some(picture) = frames.and_then(|frames| frames(source)) {
                    return Image::from_rgba(picture.width, picture.height, &picture.rgba);
                }
                source
                    .strip_prefix("data:")
                    .map_or_else(|| files(source), data_bytes)
                    .and_then(|bytes| Image::decode(&bytes))
            },
        )
        .map(|i| &*i)
    }
}

fn picture_of(image: &Image) -> Option<Picture> {
    let (width, height) = (image.width(), image.height());
    let mut surface = opsis_skia::Surface::new(width, height).ok()?;
    surface.clear(opsis_skia::Rgba::TRANSPARENT);
    let full = opsis_skia::RectF::new(
        0.0,
        0.0,
        opsis_types::numbers::narrow(f64::from(width)),
        opsis_types::numbers::narrow(f64::from(height)),
    );
    let mut paint = opsis_skia::Paint::new();
    paint.set_blend(opsis_skia::BlendMode::Src);
    surface.draw_image_rect(image, (full, full), opsis_skia::Sampling::Nearest, &paint);
    Some(Picture {
        width,
        height,
        rgba: surface.rgba(),
    })
}

fn hex_value(digit: u8) -> Option<u8> {
    char::from(digit)
        .to_digit(16)
        .and_then(|d| u8::try_from(d).ok())
}

fn percent_decoded(text: &str) -> Vec<u8> {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let pair = (
            bytes.get(i + 1).copied().and_then(hex_value),
            bytes.get(i + 2).copied().and_then(hex_value),
        );
        match (bytes[i], pair) {
            (b'%', (Some(high), Some(low))) => {
                out.push(high * 16 + low);
                i += 3;
            }
            (byte, _) => {
                out.push(byte);
                i += 1;
            }
        }
    }
    out
}

fn data_bytes(rest: &str) -> Option<Vec<u8>> {
    use base64::Engine as _;
    let (header, payload) = rest.split_once(',')?;
    if header.ends_with(";base64") {
        base64::engine::general_purpose::STANDARD
            .decode(payload.trim())
            .ok()
    } else {
        Some(percent_decoded(payload))
    }
}

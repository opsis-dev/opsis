use std::collections::HashMap;
use std::sync::Arc;

use opsis_layout::output::TextRun;
use opsis_skia::{Axis, Image, Typeface};

type TypefaceKey = (u64, u32, Vec<([u8; 4], u32)>);

pub struct Resources<'a> {
    files: &'a (dyn Fn(&str) -> Option<Vec<u8>> + Sync),
    typefaces: HashMap<TypefaceKey, Option<Typeface>>,
    images: HashMap<String, Option<Image>>,
    trees: HashMap<String, Option<usvg::Tree>>,
    animations: HashMap<String, Option<opsis_skia::Lottie>>,
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
    files: &'a (dyn Fn(&str) -> Option<Vec<u8>> + Sync),
) -> usvg::Options<'a> {
    let mut database = usvg::fontdb::Database::new();
    let mut by_name: HashMap<String, Vec<usvg::fontdb::ID>> = HashMap::new();
    for (name, bytes) in faces {
        let source: Arc<dyn AsRef<[u8]> + Send + Sync> = bytes.clone();
        let ids = database.load_font_source(usvg::fontdb::Source::Binary(source));
        by_name.entry(name.clone()).or_default().extend(ids);
    }
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
        fontdb: Arc::new(database),
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
            .field("typefaces", &self.typefaces.len())
            .field("images", &self.images.len())
            .finish_non_exhaustive()
    }
}

impl<'a> Resources<'a> {
    pub fn new(
        files: &'a (dyn Fn(&str) -> Option<Vec<u8>> + Sync),
        faces: &[(String, Arc<Vec<u8>>)],
    ) -> Self {
        Self {
            options: options(faces, files),
            files,
            typefaces: HashMap::new(),
            images: HashMap::new(),
            trees: HashMap::new(),
            animations: HashMap::new(),
        }
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
        self.typefaces
            .entry(key)
            .or_insert_with(|| {
                let axes: Vec<Axis> = run
                    .variations
                    .iter()
                    .map(|(tag, value)| Axis::new(*tag, *value))
                    .collect();
                Typeface::from_bytes(run.font.data.data(), run.font.index, &axes)
            })
            .as_ref()
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
        self.trees
            .entry(source.to_owned())
            .or_insert_with(|| {
                let bytes = source
                    .strip_prefix("data:")
                    .map_or_else(|| files(source), data_bytes)?;
                usvg::Tree::from_data(&bytes, options).ok()
            })
            .as_ref()
    }

    pub const fn options(&self) -> &usvg::Options<'a> {
        &self.options
    }

    pub fn markup(&mut self, document: String) -> Option<&usvg::Tree> {
        let options = &self.options;
        self.trees
            .entry(document)
            .or_insert_with_key(|text| usvg::Tree::from_str(text, options).ok())
            .as_ref()
    }

    pub fn animation(&mut self, source: &str) -> Option<&mut opsis_skia::Lottie> {
        let files = self.files;
        self.animations
            .entry(source.to_owned())
            .or_insert_with(|| {
                let json = crate::lottie::animation_json(&files(source)?)?;
                opsis_skia::Lottie::from_json(&json)
            })
            .as_mut()
    }

    pub fn image(&mut self, source: &str) -> Option<&Image> {
        let files = self.files;
        self.images
            .entry(source.to_owned())
            .or_insert_with(|| {
                source
                    .strip_prefix("data:")
                    .map_or_else(|| files(source), data_bytes)
                    .and_then(|bytes| Image::decode(&bytes))
            })
            .as_ref()
    }
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

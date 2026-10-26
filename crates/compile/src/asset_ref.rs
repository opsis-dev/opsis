use std::sync::LazyLock;

use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, utf8_percent_encode};
use regex::Regex;

pub const PROJECT_SCHEME: &str = "project:";
pub const APP_SCHEME: &str = "app:";
pub const LIBRARY_SCHEME: &str = "library:";
pub const STATIC_PROJECT_BASE: &str = "static:";

const EXTERNAL_SCHEMES: [&str; 4] = ["http:", "https:", "data:", "blob:"];

const COMPONENT: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'_')
    .remove(b'.')
    .remove(b'!')
    .remove(b'~')
    .remove(b'*')
    .remove(b'\'')
    .remove(b'(')
    .remove(b')');

static ABSOLUTE_PATH: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(/|[A-Za-z]:[\\/]|\\\\)").expect("pattern"));

pub fn join_url(base: &str, rel: &str) -> String {
    let encoded: Vec<String> = rel
        .split('/')
        .map(|p| utf8_percent_encode(p, COMPONENT).to_string())
        .collect();
    if base.ends_with('/') {
        format!("{base}{}", encoded.join("/"))
    } else {
        format!("{base}/{}", encoded.join("/"))
    }
}

pub fn is_project_ref(r: &str) -> bool {
    r.starts_with(PROJECT_SCHEME)
}

pub fn is_external_ref(r: &str) -> bool {
    EXTERNAL_SCHEMES.iter().any(|s| r.starts_with(s))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RefTarget {
    Project(String),
    App(String),
    External(String),
    Absolute(String),
}

pub fn ref_target(r: &str) -> RefTarget {
    if let Some(rest) = r.strip_prefix(PROJECT_SCHEME) {
        return RefTarget::Project(rest.trim_start_matches('/').to_owned());
    }
    if let Some(rest) = r.strip_prefix(APP_SCHEME) {
        return RefTarget::App(rest.to_owned());
    }
    if is_external_ref(r) {
        return RefTarget::External(r.to_owned());
    }
    if ABSOLUTE_PATH.is_match(r) {
        return RefTarget::Absolute(r.to_owned());
    }
    RefTarget::Project(r.to_owned())
}

pub fn project_rel_path(r: &str) -> Option<String> {
    match ref_target(r) {
        RefTarget::Project(rel) => Some(rel),
        _ => None,
    }
}

pub fn stays_in_project(r: &str) -> bool {
    project_rel_path(r).is_some_and(|rel| !rel.split(['/', '\\']).any(|p| p == ".."))
}

pub fn file_version_key(r: &str) -> Option<String> {
    let rel = project_rel_path(r.trim())?;
    let parts: Vec<&str> = rel
        .split(['/', '\\'])
        .filter(|p| !p.is_empty() && *p != ".")
        .collect();
    Some(parts.join("/"))
}

pub fn library_ref(rel: &str) -> String {
    format!("{LIBRARY_SCHEME}{rel}")
}

pub fn library_rel_path(r: &str) -> Option<&str> {
    r.strip_prefix(LIBRARY_SCHEME)
}

pub fn extension_of(file: &str) -> String {
    let name = file.rsplit('/').next().unwrap_or(file);
    match name.rfind('.') {
        Some(dot) if dot > 0 => name[dot + 1..].to_lowercase(),
        _ => String::new(),
    }
}

pub fn base_name(r: &str) -> String {
    let trimmed = r.trim_end_matches('/');
    match trimmed.rsplit('/').next() {
        Some(last) if !last.is_empty() => last.to_owned(),
        _ => r.to_owned(),
    }
}

pub fn project_ref(path: &str) -> String {
    if path.is_empty() || is_project_ref(path) || !matches!(ref_target(path), RefTarget::Project(_))
    {
        path.to_owned()
    } else {
        format!("{PROJECT_SCHEME}{path}")
    }
}

pub fn mime_subtype(t: &str) -> &str {
    t.rsplit('/').next().unwrap_or(t)
}

pub const IMAGE_EXTENSIONS: [&str; 6] = ["png", "jpg", "jpeg", "webp", "gif", "svg"];
pub const AUDIO_EXTENSIONS: [&str; 14] = [
    "mp3", "m4a", "aac", "wav", "ogg", "flac", "opus", "oga", "weba", "aif", "aiff", "caf", "wma",
    "mka",
];

pub fn kind_from_path(path: &str) -> opsis_types::values::MediaKind {
    use opsis_types::values::MediaKind;
    let ext = extension_of(path);
    if IMAGE_EXTENSIONS.contains(&ext.as_str()) {
        MediaKind::Image
    } else if AUDIO_EXTENSIONS.contains(&ext.as_str()) {
        MediaKind::Audio
    } else {
        MediaKind::Video
    }
}

pub const PREPARED_SCHEME: &str = "prepared:";
pub const PREPARED_FOLDER: &str = ".opsis/prepared";

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PreparedKind {
    Typeface { weight: f64 },
    Model,
    Svg,
    Texture,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PreparedRef {
    pub kind: PreparedKind,
    pub source: RefTarget,
}

fn weight_text(weight: f64) -> String {
    if weight.fract() == 0.0 {
        format!("{}", weight as i64)
    } else {
        format!("{weight}")
    }
}

impl PreparedRef {
    pub fn new(kind: PreparedKind, source: &str) -> Option<Self> {
        let source = match ref_target(source) {
            RefTarget::Project(rel) if is_project_ref(source) && stays_in_project(source) => {
                RefTarget::Project(rel)
            }
            RefTarget::App(rel) if !rel.split(['/', '\\']).any(|p| p == "..") => {
                RefTarget::App(rel)
            }
            _ => return None,
        };
        Some(Self { kind, source })
    }

    pub fn path(&self) -> String {
        let kind = match self.kind {
            PreparedKind::Typeface { weight } => format!("typeface/{}", weight_text(weight)),
            PreparedKind::Model => "model".to_owned(),
            PreparedKind::Svg => "svg".to_owned(),
            PreparedKind::Texture => "texture".to_owned(),
        };
        let source = match &self.source {
            RefTarget::App(rel) => format!("app/{rel}"),
            RefTarget::Project(rel) => format!("project/{rel}"),
            RefTarget::External(r) | RefTarget::Absolute(r) => r.clone(),
        };
        format!("{kind}/{source}")
    }

    pub fn to_ref(&self) -> String {
        format!("{PREPARED_SCHEME}{}", self.path())
    }

    pub fn parse(r: &str) -> Option<Self> {
        Self::from_path(r.strip_prefix(PREPARED_SCHEME)?)
    }

    pub fn from_path(path: &str) -> Option<Self> {
        let (kind, rest) = path.split_once('/')?;
        let (kind, rest) = match kind {
            "typeface" => {
                let (weight, rest) = rest.split_once('/')?;
                let weight = weight.parse().ok().filter(|w: &f64| w.is_finite())?;
                (PreparedKind::Typeface { weight }, rest)
            }
            "model" => (PreparedKind::Model, rest),
            "svg" => (PreparedKind::Svg, rest),
            "texture" => (PreparedKind::Texture, rest),
            _ => return None,
        };
        let (scheme, rel) = rest.split_once('/')?;
        let source = match scheme {
            "project" => format!("{PROJECT_SCHEME}{rel}"),
            "app" => format!("{APP_SCHEME}{rel}"),
            _ => return None,
        };
        Self::new(kind, &source)
    }
}

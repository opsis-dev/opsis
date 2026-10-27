use indexmap::IndexMap;
use opsis_types::fonts::ProjectFonts;
use opsis_types::timeline::{TimelineItem, TimelineSpec, TimelineTrack};
use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, utf8_percent_encode};
use serde_json::{Map, Value};

use crate::asset_ref::{
    PREPARED_FOLDER, PREPARED_SCHEME, PROJECT_SCHEME, PreparedRef, RefTarget, is_project_ref,
    join_url, project_rel_path,
};
use crate::fonts::film_fonts;

const MAX_PROP_DEPTH: u32 = 3;

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

struct Host<'a> {
    base: &'a str,
    versions: Option<&'a IndexMap<String, String>>,
}

impl Host<'_> {
    fn versioned(&self, url: String, source: &str) -> String {
        let version = self.versions.and_then(|versions| versions.get(source));
        match version {
            Some(version) => format!("{url}?v={}", utf8_percent_encode(version, COMPONENT)),
            None => url,
        }
    }

    fn prepared(&self, path: &str) -> String {
        let source = PreparedRef::from_path(path).and_then(|p| match p.source {
            RefTarget::Project(rel) => Some(format!("{PROJECT_SCHEME}{rel}")),
            _ => None,
        });
        self.versioned(
            join_url(self.base, &format!("{PREPARED_FOLDER}/{path}")),
            source.as_deref().unwrap_or_default(),
        )
    }

    fn resolve(&self, text: &str) -> Option<String> {
        if let Some(path) = text.strip_prefix(PREPARED_SCHEME) {
            return Some(self.prepared(path));
        }
        if !is_project_ref(text) {
            return None;
        }
        let rel = project_rel_path(text)?;
        Some(self.versioned(join_url(self.base, &rel), text))
    }

    fn string(&self, text: &mut String) {
        if let Some(resolved) = self.resolve(text) {
            *text = resolved;
        }
    }

    fn value(&self, value: &mut Value, depth: u32) {
        match value {
            Value::String(text) => self.string(text),
            Value::Array(items) if depth <= MAX_PROP_DEPTH => {
                items.iter_mut().for_each(|v| self.value(v, depth + 1));
            }
            Value::Object(map) if depth <= MAX_PROP_DEPTH && !map.contains_key("tracks") => {
                map.values_mut().for_each(|v| self.value(v, depth + 1));
            }
            _ => {}
        }
    }

    fn props(&self, props: &mut Map<String, Value>) {
        props.values_mut().for_each(|v| self.value(v, 1));
    }

    fn tracks(&self, tracks: &mut [TimelineTrack]) {
        for item in tracks.iter_mut().flat_map(|t| t.items.iter_mut()) {
            match item {
                TimelineItem::Element { props, .. } => self.props(props),
                TimelineItem::Recording { props, .. } => {
                    self.string(&mut props.src);
                    if let Some(proxy) = props.proxy_src.as_mut() {
                        self.string(proxy);
                    }
                    self.string(&mut props.backdrop);
                }
                TimelineItem::Group { props, .. } => {
                    if let Some(wrapper) = props.wrapper.as_mut() {
                        self.props(&mut wrapper.props);
                    }
                    if let Some(sequencer) = props.sequencer.as_mut() {
                        self.props(&mut sequencer.props);
                    }
                    self.tracks(&mut props.tracks);
                }
            }
        }
    }
}

pub fn hosted(spec: &TimelineSpec) -> TimelineSpec {
    let mut out = spec.clone();
    if out.fonts.is_none() {
        out.fonts = Some(film_fonts(&ProjectFonts::default(), &out.tracks));
    }
    let Some(assets) = out.assets.take() else {
        return out;
    };
    let Some(base) = assets.project_base.as_deref() else {
        return out;
    };
    let host = Host {
        base,
        versions: assets.versions.as_ref(),
    };
    host.tracks(&mut out.tracks);
    for sound in &mut out.audio {
        host.string(&mut sound.src);
    }
    if let Some(stem) = out.mix.as_mut() {
        host.string(&mut stem.src);
    }
    if let Some(fonts) = out.fonts.as_mut() {
        for face in fonts.families.values_mut().flat_map(|f| f.faces.iter_mut()) {
            host.string(&mut face.file);
        }
    }
    out
}

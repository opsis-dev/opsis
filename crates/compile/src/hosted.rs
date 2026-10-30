use opsis_types::fonts::ProjectFonts;
use opsis_types::timeline::{TimelineItem, TimelineSpec, TimelineTrack};
use serde_json::{Map, Value};

use crate::asset_ref::{
    PREPARED_FOLDER, PREPARED_SCHEME, is_project_ref, join_url, project_rel_path,
};
use crate::fonts::film_fonts;

const MAX_PROP_DEPTH: u32 = 3;

struct Host<'a> {
    base: &'a str,
}

impl Host<'_> {
    fn prepared(&self, path: &str) -> String {
        join_url(self.base, &format!("{PREPARED_FOLDER}/{path}"))
    }

    fn resolve(&self, text: &str) -> Option<String> {
        if let Some(path) = text.strip_prefix(PREPARED_SCHEME) {
            return Some(self.prepared(path));
        }
        if !is_project_ref(text) {
            return None;
        }
        let rel = project_rel_path(text)?;
        Some(join_url(self.base, &rel))
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
    let host = Host { base };
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

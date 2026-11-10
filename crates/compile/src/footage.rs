use opsis_types::recording::{FootageChain, ScreenProps};
use opsis_types::terminal::TerminalTheme;
use opsis_types::timeline::{TimelineItem, TimelineTrack};
use serde::Serialize;
use serde_json::{Map, Value};

use crate::clip::{IMAGE, VIDEO};
use crate::elements::element_meta;
use crate::mix::plan::lookup_key;

const VIDEO_KIND: &str = "video";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FootageUse {
    pub id: String,
    pub src: String,
    pub chain: FootageChain,
    pub trim_ms: u32,
    pub looping: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub terminal: Option<TerminalTheme>,
}

pub const TERMINAL_FRAMES: &str = "frames";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemFootage {
    pub prop: String,
    pub used: FootageUse,
}

fn identified(src: &str, chain: FootageChain, trim_ms: u32, looping: bool) -> FootageUse {
    let mut used = FootageUse {
        id: String::new(),
        src: src.to_owned(),
        chain,
        trim_ms,
        looping,
        terminal: None,
    };
    used.id = lookup_key(&used);
    used
}

fn chain_prop(props: &Map<String, Value>) -> FootageChain {
    props
        .get("footage")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default()
}

fn file<'a>(props: &'a Map<String, Value>, prop: &str) -> Option<&'a str> {
    props
        .get(prop)
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
}

fn trim_of(props: &Map<String, Value>) -> u32 {
    props
        .get("trimBeforeMs")
        .and_then(Value::as_u64)
        .and_then(|ms| u32::try_from(ms).ok())
        .unwrap_or(0)
}

fn own_footage(component: &str, props: &Map<String, Value>) -> Option<FootageUse> {
    let src = file(props, "src")?;
    let chain = chain_prop(props);
    match component {
        VIDEO => {
            let looping = props.get("loop").and_then(Value::as_bool).unwrap_or(false);
            Some(identified(src, chain, trim_of(props), looping))
        }
        IMAGE if !chain.is_empty() => Some(identified(src, chain, 0, false)),
        _ => None,
    }
}

pub fn element_footage(component: &str, props: &Map<String, Value>) -> Vec<ItemFootage> {
    let mut found: Vec<ItemFootage> = own_footage(component, props)
        .map(|used| ItemFootage {
            prop: "src".to_owned(),
            used,
        })
        .into_iter()
        .collect();
    let kinds = element_meta(component).map(|meta| &meta.kind_props);
    for (kind, setting) in kinds.into_iter().flatten() {
        let is_video = props.get(kind).and_then(Value::as_str) == Some(VIDEO_KIND);
        if let Some(src) = file(props, setting).filter(|_| is_video)
            && !found.iter().any(|f| &f.prop == setting)
        {
            found.push(ItemFootage {
                prop: setting.clone(),
                used: identified(src, FootageChain::default(), 0, false),
            });
        }
    }
    found
}

fn screen_footage(screen: &ScreenProps) -> Option<ItemFootage> {
    match screen {
        ScreenProps::Footage { src, chain, .. } if !src.is_empty() => Some(ItemFootage {
            prop: String::new(),
            used: identified(src, chain.clone(), 0, false),
        }),
        ScreenProps::Footage { .. } => None,
        ScreenProps::Terminal {
            cast, look, chain, ..
        } => {
            let mut used = identified(cast, chain.clone(), 0, false);
            used.terminal = Some(look.theme.clone());
            used.id = lookup_key(&used);
            Some(ItemFootage {
                prop: String::new(),
                used,
            })
        }
    }
}

pub fn item_footage(item: &TimelineItem) -> Vec<ItemFootage> {
    match item {
        TimelineItem::Recording { props, .. } => {
            screen_footage(&props.screen).into_iter().collect()
        }
        TimelineItem::Element {
            component, props, ..
        } => element_footage(component, props),
        TimelineItem::Group { .. } => Vec::new(),
    }
}

fn collect(tracks: &[TimelineTrack], found: &mut Vec<FootageUse>) {
    for item in tracks.iter().flat_map(|t| &t.items) {
        for footage in item_footage(item) {
            if !found.iter().any(|f| f.id == footage.used.id) {
                found.push(footage.used);
            }
        }
        if let TimelineItem::Group { props, .. } = item {
            collect(&props.tracks, found);
        }
    }
}

pub fn footage_uses(tracks: &[TimelineTrack]) -> Vec<FootageUse> {
    let mut found = Vec::new();
    collect(tracks, &mut found);
    found
}

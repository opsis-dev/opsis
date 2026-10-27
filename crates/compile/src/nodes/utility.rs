use opsis_types::cards::Icon;
use opsis_types::looks::NodeLook;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::card::truncate_card;
use crate::graph::{GROUP_TYPE, NOTE_TYPE};
use crate::node::{CardCtx, Category, NodeDef, Phase, missing, row};
use crate::params::Params;

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct Group {
    #[default = "Group"]
    #[schemars(description = "Title")]
    pub title: String,
    #[default = "#a78bfa"]
    #[schemars(description = "Colour", extend("x-widget" = "color"))]
    pub color: String,
}

impl Params for Group {}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct Note {
    #[default = "Notes…"]
    #[schemars(description = "Text", extend("x-widget" = "textarea"))]
    pub text: String,
}

impl Params for Note {}

pub fn nodes() -> Vec<NodeDef> {
    vec![
        NodeDef::of::<Group>(GROUP_TYPE, "Group", Category::Utility, Phase::Compile)
            .describe("Boxes nodes together. Select nodes, then Group.")
            .hidden()
            .look(NodeLook::of("lucide:group")),
        NodeDef::of::<Note>(NOTE_TYPE, "Note", Category::Utility, Phase::Compile)
            .describe("A sticky note on the canvas. Not part of the video.")
            .card_height(140.0)
            .card(|ctx: &CardCtx<'_, Note>| {
                let first = ctx
                    .params
                    .text
                    .split('\n')
                    .find(|line| !line.trim().is_empty());
                vec![Some(match first {
                    Some(line) => row(Icon::StickyNote, truncate_card(line.trim())),
                    None => missing(Icon::StickyNote, "Write a note"),
                })]
            })
            .look(NodeLook::of("lucide:sticky-note")),
    ]
}

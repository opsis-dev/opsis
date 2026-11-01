use opsis_macros::settings;
use opsis_types::cards::Icon;
use opsis_types::looks::NodeLook;

use crate::card_rows::truncate_card;
use crate::graph::{GROUP_TYPE, NOTE_TYPE};
use crate::node::{CardCtx, Category, NodeDef, Phase, missing, row};

#[settings(eq)]
pub struct Group {
    #[setting("Title", default = "Group")]
    pub title: String,
    #[setting("Colour", default = "#a78bfa", widget = "color")]
    pub color: String,
}

#[settings(eq)]
pub struct Note {
    #[setting("Text", default = "Notes…", widget = "textarea")]
    pub text: String,
    #[setting("Colour", default = "#fcd34d", widget = "color", check = colour)]
    pub color: String,
}

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

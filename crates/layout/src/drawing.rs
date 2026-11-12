use opsis_scene::drawing::{Baseline, Command, Drawing, Font, TextAlign};
use opsis_scene::style::FontStyle;
use parley::PositionedLayoutItem;

use crate::computed::{LineSpacing, TextStyle};
use crate::inline::{font_extent, probe};
use crate::output::{TextRun, run_of};
use crate::tree::Tree;

const HANGING_SHARE: f64 = 0.8;

#[derive(Debug, Clone, Default)]
struct State {
    font: Font,
    align: TextAlign,
    baseline: Baseline,
}

pub fn style_of(font: &Font) -> TextStyle {
    let mut style = TextStyle::initial();
    style.families.clone_from(&font.families);
    style.font_size = font.size.0;
    style.font_weight = font.weight.0;
    style.font_style = if font.italic {
        FontStyle::Italic
    } else {
        FontStyle::Normal
    };
    style.line_height = LineSpacing::Normal;
    style
}

fn runs_for(tree: &mut Tree<'_>, state: &State, text: &str, (x, y): (f64, f64)) -> Vec<TextRun> {
    let style = style_of(&state.font);
    let paragraph = probe(tree, &style, text);
    let width = f64::from(paragraph.width());
    let extent = font_extent(tree, &style);
    let (ascent, descent) = (f64::from(extent.ascent), f64::from(extent.descent));
    let dx = match state.align {
        TextAlign::Start | TextAlign::Left => 0.0,
        TextAlign::Center => -width / 2.0,
        TextAlign::End | TextAlign::Right => -width,
    };
    let dy = match state.baseline {
        Baseline::Alphabetic => 0.0,
        Baseline::Top => ascent,
        Baseline::Hanging => ascent * HANGING_SHARE,
        Baseline::Middle => (ascent - descent) / 2.0,
        Baseline::Ideographic | Baseline::Bottom => -descent,
    };
    let Some(line) = paragraph.lines().next() else {
        return Vec::new();
    };
    line.items()
        .filter_map(|item| match item {
            PositionedLayoutItem::GlyphRun(run) => {
                Some(run_of(&run, &style, (x + dx, y + dy), 0.0))
            }
            PositionedLayoutItem::InlineBox(_) => None,
        })
        .collect()
}

pub fn shaped(tree: &mut Tree<'_>, drawing: &Drawing) -> Vec<(usize, Vec<TextRun>)> {
    let mut state = State::default();
    let mut stack: Vec<State> = Vec::new();
    let mut out = Vec::new();
    for (index, command) in drawing.commands.iter().enumerate() {
        match command {
            Command::Save => stack.push(state.clone()),
            Command::Restore => {
                if let Some(saved) = stack.pop() {
                    state = saved;
                }
            }
            Command::Font(font) => state.font = font.clone(),
            Command::TextAlign(align) => state.align = *align,
            Command::TextBaseline(baseline) => state.baseline = *baseline,
            Command::FillText { text, at } | Command::StrokeText { text, at } => {
                out.push((index, runs_for(tree, &state, text, (at.0.0, at.1.0))));
            }
            _ => {}
        }
    }
    out
}

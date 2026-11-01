use opsis_types::numbers;

use crate::node::{MAX_CARD_ROWS, NodeDef};

pub const CARD_WIDTH: f64 = 256.0;
pub const CARD_GAP: f64 = 32.0;
pub const COLUMN_WIDTH: f64 = 300.0;
pub const ROW_HEIGHT: f64 = 170.0;
const HEADER: f64 = 50.0;
const PORT_ROW: f64 = 22.0;
const CARD_ROW: f64 = 20.0;
const PADDING: f64 = 16.0;
const UNKNOWN_HEIGHT: f64 = 120.0;

pub fn estimate_card_height(def: Option<&NodeDef>) -> f64 {
    let Some(def) = def else {
        return UNKNOWN_HEIGHT;
    };
    if let Some(height) = def.card_height {
        return height;
    }
    let inputs = def.input_ports(std::iter::empty()).len();
    let ports = numbers::float(inputs.max(def.outputs.len()));
    let rows = if def.has_card() {
        numbers::float(MAX_CARD_ROWS)
    } else {
        0.0
    };
    HEADER + ports * PORT_ROW + rows * CARD_ROW + PADDING
}

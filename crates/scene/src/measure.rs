use std::sync::Arc;

use opsis_types::terminal_frames::Frames;

use crate::drawing::Font;
use crate::node::Node;
use crate::typeface::Typeface;
use crate::vector::Vector;

#[derive(Debug, Clone, PartialEq)]
pub struct MeasuredBox {
    pub path: Vec<usize>,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AnimationTiming {
    pub in_point: f64,
    pub out_point: f64,
    pub frame_rate: f64,
}

pub trait Measure {
    fn text_width(&self, text: &str, font: &Font, letter_spacing: f64) -> f64;

    fn animation(&self, _source: &str) -> Option<AnimationTiming> {
        None
    }

    fn boxes(&self, _root: &Node) -> Vec<MeasuredBox> {
        Vec::new()
    }

    fn text_ink(&self, _text: &str, _font: &Font) -> (f64, f64) {
        (0.0, 0.0)
    }

    fn terminal(&self, _source: &str) -> Option<Arc<Frames>> {
        None
    }

    fn picture_size(&self, _source: &str) -> Option<(f64, f64)> {
        None
    }

    fn footage_time(&self, _base: &str, micros: f64) -> f64 {
        micros
    }

    fn typeface(&self, _source: &str) -> Option<Arc<Typeface>> {
        None
    }

    fn vector(&self, _source: &str) -> Option<Arc<Vector>> {
        None
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Unmeasured;

impl Measure for Unmeasured {
    fn text_width(&self, _: &str, _: &Font, _: f64) -> f64 {
        0.0
    }
}

use crate::node::Segment2d;

#[derive(Debug, Clone, PartialEq)]
pub struct VectorStroke {
    pub color: String,
    pub width: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VectorPath {
    pub fill: Option<String>,
    pub stroke: Option<VectorStroke>,
    pub even_odd: bool,
    pub sub_paths: Vec<Vec<Segment2d>>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Vector {
    pub paths: Vec<VectorPath>,
}

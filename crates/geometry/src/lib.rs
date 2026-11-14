pub mod outline;
pub mod primitives;
pub mod tube;

use opsis_scene::node::Shape3d;

use crate::outline::{extruded, flat, plane};
pub use crate::primitives::Geometry;
use crate::primitives::{box_geometry, rounded_box, sphere, torus};
use crate::tube::tube;

pub fn mesh_of(shape: &Shape3d) -> Geometry {
    match shape {
        Shape3d::Sphere {
            radius,
            width_segments,
            height_segments,
        } => sphere(radius.0, *width_segments, *height_segments),
        Shape3d::Torus {
            radius,
            tube,
            radial_segments,
            tubular_segments,
        } => torus(radius.0, tube.0, *radial_segments, *tubular_segments),
        Shape3d::RoundedBox {
            size,
            segments,
            radius,
        } => rounded_box([size[0].0, size[1].0, size[2].0], *segments, radius.0),
        Shape3d::Plane { width, height } => plane(width.0, height.0),
        Shape3d::Box { size } => box_geometry([size[0].0, size[1].0, size[2].0], [1, 1, 1]),
        Shape3d::Flat {
            outline,
            curve_segments,
            uv_span,
            uv_offset,
        } => flat(
            outline,
            *curve_segments,
            [uv_span[0].0, uv_span[1].0],
            [uv_offset[0].0, uv_offset[1].0],
        ),
        Shape3d::Extruded {
            outlines,
            curve_segments,
            depth,
            bevel,
        } => extruded(outlines, *curve_segments, depth.0, bevel.as_ref()),
        Shape3d::Tube {
            points,
            closed,
            tubular_segments,
            radius,
            radial_segments,
        } => {
            let points: Vec<[f64; 3]> = points.iter().map(|p| p.map(|n| n.0)).collect();
            tube(
                &points,
                *closed,
                *tubular_segments,
                radius.0,
                *radial_segments,
            )
        }
    }
}

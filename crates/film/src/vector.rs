use opsis_scene::node::Segment2d;
use opsis_scene::values::Num;
use opsis_scene::vector::{Vector, VectorPath, VectorStroke};
use usvg::tiny_skia_path::{PathSegment, Point};

fn hex(paint: &usvg::Paint) -> String {
    match paint {
        usvg::Paint::Color(c) => format!("#{:02x}{:02x}{:02x}", c.red, c.green, c.blue),
        _ => "#ffffff".to_owned(),
    }
}

#[expect(
    clippy::suboptimal_flops,
    reason = "an SVG transform maps a point this way"
)]
fn mapped(transform: usvg::Transform, point: Point) -> [Num; 2] {
    let (x, y) = (f64::from(point.x), f64::from(point.y));
    [
        Num(f64::from(transform.sx) * x + f64::from(transform.kx) * y + f64::from(transform.tx)),
        Num(f64::from(transform.ky) * x + f64::from(transform.sy) * y + f64::from(transform.ty)),
    ]
}

fn sub_paths(path: &usvg::Path) -> Vec<Vec<Segment2d>> {
    let transform = path.abs_transform();
    let mut out: Vec<Vec<Segment2d>> = Vec::new();
    let mut start = None;
    let mut last = None;
    for segment in path.data().segments() {
        match segment {
            PathSegment::MoveTo(p) => {
                let p = mapped(transform, p);
                out.push(vec![Segment2d::Move(p)]);
                (start, last) = (Some(p), Some(p));
            }
            PathSegment::LineTo(p) => {
                let p = mapped(transform, p);
                if let Some(current) = out.last_mut() {
                    current.push(Segment2d::Line(p));
                }
                last = Some(p);
            }
            PathSegment::QuadTo(c, p) => {
                let p = mapped(transform, p);
                if let Some(current) = out.last_mut() {
                    current.push(Segment2d::Quad(mapped(transform, c), p));
                }
                last = Some(p);
            }
            PathSegment::CubicTo(a, b, p) => {
                let p = mapped(transform, p);
                if let Some(current) = out.last_mut() {
                    current.push(Segment2d::Cubic(
                        mapped(transform, a),
                        mapped(transform, b),
                        p,
                    ));
                }
                last = Some(p);
            }
            PathSegment::Close => {
                if let (Some(first), Some(current)) = (start, out.last_mut())
                    && last != Some(first)
                {
                    current.push(Segment2d::Line(first));
                }
                last = start;
            }
        }
    }
    out
}

fn collect(group: &usvg::Group, into: &mut Vec<VectorPath>) {
    for node in group.children() {
        match node {
            usvg::Node::Group(inner) => collect(inner, into),
            usvg::Node::Path(path) => into.push(VectorPath {
                fill: path.fill().map(|f| hex(f.paint())),
                stroke: path.stroke().map(|s| VectorStroke {
                    color: hex(s.paint()),
                    width: f64::from(s.width().get()),
                }),
                even_odd: path
                    .fill()
                    .is_some_and(|f| f.rule() == usvg::FillRule::EvenOdd),
                sub_paths: sub_paths(path),
            }),
            _ => {}
        }
    }
}

pub fn read(bytes: &[u8]) -> Option<Vector> {
    let tree = usvg::Tree::from_data(bytes, &usvg::Options::default()).ok()?;
    let mut paths = Vec::new();
    collect(tree.root(), &mut paths);
    Some(Vector { paths })
}

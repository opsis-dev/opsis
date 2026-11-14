#![expect(
    clippy::suboptimal_flops,
    clippy::many_single_char_names,
    reason = "a line-by-line port of three.js's curve path and tube geometry, whose JavaScript rounds every step and names its terms so; the meshes have to match it"
)]

use std::f64::consts::PI;

use opsis_types::numbers::{float, narrow};

use crate::primitives::Geometry;

type Vec3 = [f64; 3];

const ARC_DIVISIONS: usize = 200;
const TANGENT_DELTA: f64 = 0.0001;

fn sub(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot(a: Vec3, b: Vec3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: Vec3, b: Vec3) -> Vec3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn length(v: Vec3) -> f64 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

fn normalized(v: Vec3) -> Vec3 {
    let l = length(v);
    let l = if l > 0.0 { l } else { 1.0 };
    [v[0] / l, v[1] / l, v[2] / l]
}

fn distance(a: Vec3, b: Vec3) -> f64 {
    let d = sub(a, b);
    (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
}

#[derive(Debug, Clone, Copy)]
struct Line {
    from: Vec3,
    to: Vec3,
}

impl Line {
    fn point(&self, t: f64) -> Vec3 {
        if opsis_types::numbers::exact_eq(t, 1.0) {
            return self.to;
        }
        let d = sub(self.to, self.from);
        [
            d[0] * t + self.from[0],
            d[1] * t + self.from[1],
            d[2] * t + self.from[2],
        ]
    }

    fn length(&self) -> f64 {
        let mut last = self.point(0.0);
        let mut sum = 0.0;
        for p in 1..=ARC_DIVISIONS {
            let current = self.point(float(p) / float(ARC_DIVISIONS));
            sum += distance(current, last);
            last = current;
        }
        sum
    }
}

struct Polyline {
    lines: Vec<Line>,
    line_lengths: Vec<f64>,
    curve_lengths: Vec<f64>,
    arc_lengths: Vec<f64>,
}

impl Polyline {
    fn new(lines: Vec<Line>) -> Self {
        let line_lengths: Vec<f64> = lines.iter().map(Line::length).collect();
        let mut sums = 0.0;
        let curve_lengths = line_lengths
            .iter()
            .map(|l| {
                sums += l;
                sums
            })
            .collect();
        let mut path = Self {
            lines,
            line_lengths,
            curve_lengths,
            arc_lengths: Vec::new(),
        };
        let mut cache = vec![0.0];
        let mut last = path.point(0.0);
        let mut sum = 0.0;
        for p in 1..=ARC_DIVISIONS {
            let current = path.point(float(p) / float(ARC_DIVISIONS));
            sum += distance(current, last);
            cache.push(sum);
            last = current;
        }
        path.arc_lengths = cache;
        path
    }

    fn total(&self) -> f64 {
        self.curve_lengths.last().copied().unwrap_or(0.0)
    }

    fn point(&self, t: f64) -> Vec3 {
        let d = t * self.total();
        for (i, end) in self.curve_lengths.iter().enumerate() {
            if *end >= d {
                let diff = end - d;
                let segment = self.line_lengths[i];
                let u = if opsis_types::numbers::exact_eq(segment, 0.0) {
                    0.0
                } else {
                    1.0 - diff / segment
                };
                return self.lines[i].point(u);
            }
        }
        self.lines.last().map_or([0.0; 3], |l| l.to)
    }

    fn u_to_t(&self, u: f64) -> f64 {
        let lengths = &self.arc_lengths;
        let il = lengths.len();
        let target = u * lengths[il - 1];
        let mut low: isize = 0;
        let mut high = isize::try_from(il).unwrap_or(0) - 1;
        while low <= high {
            let i = low + (high - low) / 2;
            let comparison = lengths[usize::try_from(i).unwrap_or(0)] - target;
            if comparison < 0.0 {
                low = i + 1;
            } else if comparison > 0.0 {
                high = i - 1;
            } else {
                high = i;
                break;
            }
        }
        let i = usize::try_from(high).unwrap_or(0);
        if opsis_types::numbers::exact_eq(lengths[i], target) {
            return float(i) / float(il - 1);
        }
        let before = lengths[i];
        let after = lengths[i + 1];
        let fraction = (target - before) / (after - before);
        (float(i) + fraction) / float(il - 1)
    }

    fn point_at(&self, u: f64) -> Vec3 {
        self.point(self.u_to_t(u))
    }

    fn tangent_at(&self, u: f64) -> Vec3 {
        let t = self.u_to_t(u);
        let t1 = (t - TANGENT_DELTA).max(0.0);
        let t2 = (t + TANGENT_DELTA).min(1.0);
        normalized(sub(self.point(t2), self.point(t1)))
    }
}

fn rotated(v: Vec3, axis: Vec3, angle: f64) -> Vec3 {
    let c = angle.cos();
    let s = angle.sin();
    let t = 1.0 - c;
    let [x, y, z] = axis;
    let tx = t * x;
    let ty = t * y;
    let m = [
        [tx * x + c, tx * y - s * z, tx * z + s * y],
        [tx * y + s * z, ty * y + c, ty * z - s * x],
        [tx * z - s * y, ty * z + s * x, t * z * z + c],
    ];
    [
        m[0][0] * v[0] + m[0][1] * v[1] + m[0][2] * v[2],
        m[1][0] * v[0] + m[1][1] * v[1] + m[1][2] * v[2],
        m[2][0] * v[0] + m[2][1] * v[1] + m[2][2] * v[2],
    ]
}

struct Frames {
    normals: Vec<Vec3>,
    binormals: Vec<Vec3>,
}

fn frenet_frames(path: &Polyline, segments: usize, closed: bool) -> Frames {
    let tangents: Vec<Vec3> = (0..=segments)
        .map(|i| path.tangent_at(float(i) / float(segments)))
        .collect();
    let [tx, ty, tz] = tangents[0].map(f64::abs);
    let normal = if tz <= tx.min(ty) {
        [0.0, 0.0, 1.0]
    } else if ty <= tx {
        [0.0, 1.0, 0.0]
    } else {
        [1.0, 0.0, 0.0]
    };
    let mut vec = normalized(cross(tangents[0], normal));
    let mut normals = vec![cross(tangents[0], vec)];
    let mut binormals = vec![cross(tangents[0], normals[0])];
    for i in 1..=segments {
        let mut n = normals[i - 1];
        vec = cross(tangents[i - 1], tangents[i]);
        if length(vec) > f64::EPSILON {
            vec = normalized(vec);
            let theta = dot(tangents[i - 1], tangents[i]).clamp(-1.0, 1.0).acos();
            n = rotated(n, vec, theta);
        }
        normals.push(n);
        binormals.push(cross(tangents[i], n));
    }
    if closed {
        let mut theta = dot(normals[0], normals[segments]).clamp(-1.0, 1.0).acos();
        theta /= float(segments);
        if dot(tangents[0], cross(normals[0], normals[segments])) > 0.0 {
            theta = -theta;
        }
        for i in 1..=segments {
            normals[i] = rotated(normals[i], tangents[i], theta * float(i));
            binormals[i] = cross(tangents[i], normals[i]);
        }
    }
    Frames { normals, binormals }
}

pub fn tube(
    points: &[Vec3],
    closed: bool,
    tubular_segments: u32,
    radius: f64,
    radial_segments: u32,
) -> Geometry {
    let mut lines: Vec<Line> = points
        .windows(2)
        .map(|w| Line {
            from: w[0],
            to: w[1],
        })
        .collect();
    if closed && let (Some(first), Some(last)) = (points.first(), points.last()) {
        lines.push(Line {
            from: *last,
            to: *first,
        });
    }
    let mut geometry = Geometry::default();
    if lines.is_empty() {
        return geometry;
    }
    let path = Polyline::new(lines);
    let segments = usize::try_from(tubular_segments).unwrap_or(1).max(1);
    let radial = usize::try_from(radial_segments).unwrap_or(1).max(1);
    let frames = frenet_frames(&path, segments, closed);
    let rings: Vec<usize> = (0..segments)
        .chain(std::iter::once(if closed { 0 } else { segments }))
        .collect();
    for (ring, &i) in rings.iter().enumerate() {
        let p = path.point_at(float(i) / float(segments));
        let n = frames.normals[i];
        let b = frames.binormals[i];
        for j in 0..=radial {
            let v = float(j) / float(radial) * PI * 2.0;
            let sin = v.sin();
            let cos = -v.cos();
            let normal = normalized([
                cos * n[0] + sin * b[0],
                cos * n[1] + sin * b[1],
                cos * n[2] + sin * b[2],
            ]);
            geometry.positions.extend(
                [
                    p[0] + radius * normal[0],
                    p[1] + radius * normal[1],
                    p[2] + radius * normal[2],
                ]
                .map(narrow),
            );
            geometry.normals.extend(normal.map(narrow));
            geometry
                .uvs
                .extend([float(ring) / float(segments), float(j) / float(radial)].map(narrow));
        }
    }
    let row = radial + 1;
    for j in 1..=segments {
        for i in 1..=radial {
            let a = row * (j - 1) + (i - 1);
            let b = row * j + (i - 1);
            let c = row * j + i;
            let d = row * (j - 1) + i;
            geometry
                .indices
                .extend([a, b, d, b, c, d].map(|k| u32::try_from(k).unwrap_or(0)));
        }
    }
    geometry
}

#![expect(
    clippy::suboptimal_flops,
    clippy::imprecise_flops,
    clippy::many_single_char_names,
    reason = "a line-by-line port of three.js's shape and extrude geometry, whose JavaScript rounds every step and names its points so; the meshes have to match it"
)]

use std::f64::consts::FRAC_PI_2;

use opsis_scene::node::{Bevel3d, Outline2d, Segment2d};
use opsis_types::numbers::{float, narrow};

use crate::primitives::Geometry;

type Point = [f64; 2];

const fn pair(p: [opsis_scene::values::Num; 2]) -> Point {
    [p[0].0, p[1].0]
}

fn quadratic(t: f64, p0: f64, p1: f64, p2: f64) -> f64 {
    let k = 1.0 - t;
    k * k * p0 + 2.0 * (1.0 - t) * t * p1 + t * t * p2
}

fn cubic(t: f64, p0: f64, p1: f64, p2: f64, p3: f64) -> f64 {
    let k = 1.0 - t;
    k * k * k * p0 + 3.0 * k * k * t * p1 + 3.0 * (1.0 - t) * t * t * p2 + t * t * t * p3
}

fn same(a: Point, b: Point) -> bool {
    opsis_types::numbers::exact_eq(a[0], b[0]) && opsis_types::numbers::exact_eq(a[1], b[1])
}

pub fn path_points(path: &[Segment2d], divisions: u32) -> Vec<Point> {
    let mut points: Vec<Point> = Vec::new();
    let mut current: Point = [0.0, 0.0];
    let push = |point: Point, points: &mut Vec<Point>| {
        if points.last().is_some_and(|last| same(*last, point)) {
            return;
        }
        points.push(point);
    };
    let steps = divisions.max(1);
    for segment in path {
        match *segment {
            Segment2d::Move(to) => current = pair(to),
            Segment2d::Line(to) => {
                let to = pair(to);
                push(current, &mut points);
                push(to, &mut points);
                current = to;
            }
            Segment2d::Quad(control, to) => {
                let (c, to) = (pair(control), pair(to));
                for d in 0..=steps {
                    let t = f64::from(d) / f64::from(steps);
                    push(
                        [
                            quadratic(t, current[0], c[0], to[0]),
                            quadratic(t, current[1], c[1], to[1]),
                        ],
                        &mut points,
                    );
                }
                current = to;
            }
            Segment2d::Cubic(first, second, to) => {
                let (a, b, to) = (pair(first), pair(second), pair(to));
                for d in 0..=steps {
                    let t = f64::from(d) / f64::from(steps);
                    push(
                        [
                            cubic(t, current[0], a[0], b[0], to[0]),
                            cubic(t, current[1], a[1], b[1], to[1]),
                        ],
                        &mut points,
                    );
                }
                current = to;
            }
        }
    }
    points
}

fn area(contour: &[Point]) -> f64 {
    let n = contour.len();
    let mut a = 0.0;
    let mut p = n.wrapping_sub(1);
    for q in 0..n {
        a += contour[p][0] * contour[q][1] - contour[q][0] * contour[p][1];
        p = q;
    }
    a * 0.5
}

fn is_clockwise(points: &[Point]) -> bool {
    area(points) < 0.0
}

fn without_duplicate_end(points: &mut Vec<Point>) {
    let l = points.len();
    if l > 2 && same(points[l - 1], points[0]) {
        points.pop();
    }
}

fn triangulate(contour: &mut Vec<Point>, holes: &mut [Vec<Point>]) -> Vec<[usize; 3]> {
    without_duplicate_end(contour);
    let mut data: Vec<Point> = contour.clone();
    let mut hole_indices: Vec<usize> = Vec::new();
    for hole in holes.iter_mut() {
        without_duplicate_end(hole);
        hole_indices.push(data.len());
        data.extend(hole.iter().copied());
    }
    let mut triangles: Vec<usize> = Vec::new();
    earcut::Earcut::new().earcut(data, &hole_indices, &mut triangles);
    triangles.as_chunks::<3>().0.to_vec()
}

struct Extracted {
    shape: Vec<Point>,
    holes: Vec<Vec<Point>>,
}

fn extracted(outline: &Outline2d, divisions: u32) -> Extracted {
    Extracted {
        shape: path_points(&outline.shape, divisions),
        holes: outline
            .holes
            .iter()
            .map(|hole| path_points(hole, divisions))
            .collect(),
    }
}

pub fn flat(outline: &Outline2d, curve_segments: u32, span: Point, offset: Point) -> Geometry {
    let Extracted {
        shape: mut vertices,
        mut holes,
    } = extracted(outline, curve_segments);
    if !is_clockwise(&vertices) {
        vertices.reverse();
    }
    for hole in &mut holes {
        if is_clockwise(hole) {
            hole.reverse();
        }
    }
    let faces = triangulate(&mut vertices, &mut holes);
    for hole in &holes {
        vertices.extend(hole.iter().copied());
    }
    let mut geometry = Geometry::default();
    for vertex in &vertices {
        geometry
            .positions
            .extend([vertex[0], vertex[1], 0.0].map(narrow));
        geometry.normals.extend([0.0f32, 0.0, 1.0]);
        geometry.uvs.extend(
            [
                vertex[0] / span[0] + offset[0],
                vertex[1] / span[1] + offset[1],
            ]
            .map(narrow),
        );
    }
    for face in faces {
        geometry
            .indices
            .extend(face.map(|i| u32::try_from(i).unwrap_or(0)));
    }
    geometry
}

pub fn plane(width: f64, height: f64) -> Geometry {
    let (half_w, half_h) = (width / 2.0, height / 2.0);
    let mut geometry = Geometry::default();
    for iy in 0..2u32 {
        let y = f64::from(iy) * height - half_h;
        for ix in 0..2u32 {
            let x = f64::from(ix) * width - half_w;
            geometry.positions.extend([x, -y, 0.0].map(narrow));
            geometry.normals.extend([0.0f32, 0.0, 1.0]);
            geometry
                .uvs
                .extend([f64::from(ix), 1.0 - f64::from(iy)].map(narrow));
        }
    }
    geometry.indices.extend([0, 2, 1, 2, 3, 1]);
    geometry
}

fn merge_overlapping(points: &mut Vec<Point>) {
    const THRESHOLD_SQ: f64 = 1e-10 * 1e-10;
    let Some(&first) = points.first() else {
        return;
    };
    let mut previous = first;
    let mut i = 1;
    while i <= points.len() && !points.is_empty() {
        let index = i % points.len();
        let current = points[index];
        let dx = current[0] - previous[0];
        let dy = current[1] - previous[1];
        let distance_sq = dx * dx + dy * dy;
        let scale = current[0]
            .abs()
            .max(current[1].abs())
            .max(previous[0].abs())
            .max(previous[1].abs());
        if distance_sq <= THRESHOLD_SQ * scale * scale {
            points.remove(index);
            continue;
        }
        previous = current;
        i += 1;
    }
}

fn bevel_vector(point: Point, previous: Point, next: Point) -> Point {
    let v_prev = [point[0] - previous[0], point[1] - previous[1]];
    let v_next = [next[0] - point[0], next[1] - point[1]];
    let v_prev_lensq = v_prev[0] * v_prev[0] + v_prev[1] * v_prev[1];
    let collinear = v_prev[0] * v_next[1] - v_prev[1] * v_next[0];
    let (trans, shrink_by) = if collinear.abs() > f64::EPSILON {
        let v_prev_len = v_prev_lensq.sqrt();
        let v_next_len = (v_next[0] * v_next[0] + v_next[1] * v_next[1]).sqrt();
        let prev_shift = [
            previous[0] - v_prev[1] / v_prev_len,
            previous[1] + v_prev[0] / v_prev_len,
        ];
        let next_shift = [
            next[0] - v_next[1] / v_next_len,
            next[1] + v_next[0] / v_next_len,
        ];
        let sf = ((next_shift[0] - prev_shift[0]) * v_next[1]
            - (next_shift[1] - prev_shift[1]) * v_next[0])
            / (v_prev[0] * v_next[1] - v_prev[1] * v_next[0]);
        let trans = [
            prev_shift[0] + v_prev[0] * sf - point[0],
            prev_shift[1] + v_prev[1] * sf - point[1],
        ];
        let trans_lensq = trans[0] * trans[0] + trans[1] * trans[1];
        if trans_lensq <= 2.0 {
            return trans;
        }
        (trans, (trans_lensq / 2.0).sqrt())
    } else {
        let same_direction = if v_prev[0] > f64::EPSILON {
            v_next[0] > f64::EPSILON
        } else if v_prev[0] < -f64::EPSILON {
            v_next[0] < -f64::EPSILON
        } else {
            opsis_types::numbers::exact_eq(sign(v_prev[1]), sign(v_next[1]))
        };
        if same_direction {
            ([-v_prev[1], v_prev[0]], v_prev_lensq.sqrt())
        } else {
            (v_prev, (v_prev_lensq / 2.0).sqrt())
        }
    };
    [trans[0] / shrink_by, trans[1] / shrink_by]
}

fn sign(value: f64) -> f64 {
    if value > 0.0 {
        1.0
    } else if value < 0.0 {
        -1.0
    } else {
        value
    }
}

fn movements(contour: &[Point]) -> Vec<Point> {
    let n = contour.len();
    (0..n)
        .map(|i| {
            let j = if i == 0 { n - 1 } else { i - 1 };
            let k = if i + 1 == n { 0 } else { i + 1 };
            bevel_vector(contour[i], contour[j], contour[k])
        })
        .collect()
}

fn moved(point: Point, by: Point, size: f64) -> Point {
    [point[0] + by[0] * size, point[1] + by[1] * size]
}

struct Built {
    placeholder: Vec<[f64; 3]>,
    positions: Vec<[f32; 3]>,
}

impl Built {
    fn v(&mut self, point: Point, z: f64) {
        self.placeholder.push([point[0], point[1], z]);
    }

    fn vertex(&mut self, index: usize) {
        self.positions.push(self.placeholder[index].map(narrow));
    }

    fn f3(&mut self, a: usize, b: usize, c: usize) {
        self.vertex(a);
        self.vertex(b);
        self.vertex(c);
    }

    fn f4(&mut self, a: usize, b: usize, c: usize, d: usize) {
        self.f3(a, b, d);
        self.f3(b, c, d);
    }
}

fn face_normal(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> [f32; 3] {
    let [a, b, c] = [a, b, c].map(|p| p.map(f64::from));
    let cb = [c[0] - b[0], c[1] - b[1], c[2] - b[2]];
    let ab = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    let n = [
        cb[1] * ab[2] - cb[2] * ab[1],
        cb[2] * ab[0] - cb[0] * ab[2],
        cb[0] * ab[1] - cb[1] * ab[0],
    ];
    let length = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
    let length = if length > 0.0 { length } else { 1.0 };
    n.map(|c| narrow(c / length))
}

pub fn extruded(
    outlines: &[Outline2d],
    curve_segments: u32,
    depth: f64,
    bevel: Option<&Bevel3d>,
) -> Geometry {
    let mut positions: Vec<[f32; 3]> = Vec::new();
    for outline in outlines {
        positions.extend(extruded_positions(outline, curve_segments, depth, bevel));
    }
    let mut geometry = Geometry::default();
    for (i, triangle) in positions.as_chunks::<3>().0.iter().enumerate() {
        let normal = face_normal(triangle[0], triangle[1], triangle[2]);
        for p in triangle {
            geometry.positions.extend(*p);
            geometry.normals.extend(normal);
            geometry.uvs.extend([p[0], p[1]]);
        }
        let base = u32::try_from(i * 3).unwrap_or(0);
        geometry.indices.extend([base, base + 1, base + 2]);
    }
    geometry
}

fn extruded_positions(
    outline: &Outline2d,
    curve_segments: u32,
    depth: f64,
    bevel: Option<&Bevel3d>,
) -> Vec<[f32; 3]> {
    let steps = 1usize;
    let (bevel_segments, thickness, size, offset) = bevel.map_or((0, 0.0, 0.0, 0.0), |b| {
        (
            usize::try_from(b.segments).unwrap_or(0),
            b.thickness.0,
            b.size.0,
            b.offset.0,
        )
    });
    let bevel_enabled = bevel.is_some();
    let Extracted {
        shape: mut contour,
        mut holes,
    } = extracted(outline, curve_segments);
    if !is_clockwise(&contour) {
        contour.reverse();
        for hole in &mut holes {
            if is_clockwise(hole) {
                hole.reverse();
            }
        }
    }
    merge_overlapping(&mut contour);
    for hole in &mut holes {
        merge_overlapping(hole);
    }
    let mut vertices = contour.clone();
    for hole in &holes {
        vertices.extend(hole.iter().copied());
    }
    let vlen = vertices.len();
    let contour_moves = movements(&contour);
    let holes_moves: Vec<Vec<Point>> = holes.iter().map(|h| movements(h)).collect();
    let mut vertex_moves = contour_moves.clone();
    for moves in &holes_moves {
        vertex_moves.extend(moves.iter().copied());
    }
    let mut built = Built {
        placeholder: Vec::new(),
        positions: Vec::new(),
    };
    let bevel_ring = |b: usize| {
        let t = float(b) / float(bevel_segments);
        (
            thickness * (t * FRAC_PI_2).cos(),
            size * (t * FRAC_PI_2).sin() + offset,
        )
    };
    let faces = if bevel_segments == 0 {
        triangulate(&mut contour.clone(), &mut holes.clone())
    } else {
        let mut contracted: Vec<Point> = Vec::new();
        let mut expanded: Vec<Vec<Point>> = Vec::new();
        for b in 0..bevel_segments {
            let (z, bs) = bevel_ring(b);
            for (point, by) in contour.iter().zip(&contour_moves) {
                let vert = moved(*point, *by, bs);
                built.v(vert, -z);
                if b == 0 {
                    contracted.push(vert);
                }
            }
            for (hole, moves) in holes.iter().zip(&holes_moves) {
                let mut ring = Vec::new();
                for (point, by) in hole.iter().zip(moves) {
                    let vert = moved(*point, *by, bs);
                    built.v(vert, -z);
                    if b == 0 {
                        ring.push(vert);
                    }
                }
                if b == 0 {
                    expanded.push(ring);
                }
            }
        }
        triangulate(&mut contracted, &mut expanded)
    };
    let bs = size + offset;
    for s in 0..=steps {
        let z = if s == 0 {
            0.0
        } else {
            depth / float(steps) * float(s)
        };
        for (point, by) in vertices.iter().zip(&vertex_moves) {
            let vert = if bevel_enabled {
                moved(*point, *by, bs)
            } else {
                *point
            };
            built.v(vert, z);
        }
    }
    for b in (0..bevel_segments).rev() {
        let (z, bs) = bevel_ring(b);
        for (point, by) in contour.iter().zip(&contour_moves) {
            built.v(moved(*point, *by, bs), depth + z);
        }
        for (hole, moves) in holes.iter().zip(&holes_moves) {
            for (point, by) in hole.iter().zip(moves) {
                built.v(moved(*point, *by, bs), depth + z);
            }
        }
    }
    let top = if bevel_enabled {
        steps + bevel_segments * 2
    } else {
        steps
    };
    for face in &faces {
        built.f3(face[2], face[1], face[0]);
    }
    for face in &faces {
        built.f3(
            face[0] + vlen * top,
            face[1] + vlen * top,
            face[2] + vlen * top,
        );
    }
    let layers = steps + bevel_segments * 2;
    let mut layer_offset = 0;
    for ring in std::iter::once(&contour).chain(holes.iter()) {
        let n = ring.len();
        for i in (0..n).rev() {
            let j = i;
            let k = if i == 0 { n - 1 } else { i - 1 };
            for s in 0..layers {
                let (low, high) = (vlen * s, vlen * (s + 1));
                built.f4(
                    layer_offset + j + low,
                    layer_offset + k + low,
                    layer_offset + k + high,
                    layer_offset + j + high,
                );
            }
        }
        layer_offset += n;
    }
    built.positions
}

const CLASSIFY_DIVISIONS: u32 = 12;

#[derive(Debug, Clone, Copy, PartialEq)]
struct Box2 {
    min: Point,
    max: Point,
}

impl Box2 {
    fn of(points: &[Point]) -> Self {
        points.iter().fold(
            Self {
                min: [f64::INFINITY, f64::INFINITY],
                max: [f64::NEG_INFINITY, f64::NEG_INFINITY],
            },
            |b, p| Self {
                min: [b.min[0].min(p[0]), b.min[1].min(p[1])],
                max: [b.max[0].max(p[0]), b.max[1].max(p[1])],
            },
        )
    }

    fn contains(&self, other: &Self) -> bool {
        self.min[0] <= other.min[0]
            && other.max[0] <= self.max[0]
            && self.min[1] <= other.min[1]
            && other.max[1] <= self.max[1]
    }
}

fn point_in_polygon(p: Point, polygon: &[Point]) -> bool {
    let mut inside = false;
    let n = polygon.len();
    let mut j = n.wrapping_sub(1);
    for i in 0..n {
        let a = polygon[i];
        let b = polygon[j];
        if (a[1] > p[1]) != (b[1] > p[1])
            && p[0] < (b[0] - a[0]) * (p[1] - a[1]) / (b[1] - a[1]) + a[0]
        {
            inside = !inside;
        }
        j = i;
    }
    inside
}

fn interior_point(polygon: &[Point], bounds: &Box2) -> Point {
    let mut point = [
        f64::midpoint(bounds.min[0], bounds.max[0]),
        f64::midpoint(bounds.min[1], bounds.max[1]),
    ];
    if point_in_polygon(point, polygon) {
        return point;
    }
    let y = point[1];
    let n = polygon.len();
    let mut intercepts: Vec<f64> = (0..n)
        .filter_map(|i| {
            let a = polygon[i];
            let b = polygon[(i + 1) % n];
            ((a[1] > y) != (b[1] > y)).then(|| a[0] + (y - a[1]) * (b[0] - a[0]) / (b[1] - a[1]))
        })
        .collect();
    if intercepts.len() > 1 {
        intercepts.sort_by(f64::total_cmp);
        point[0] = f64::midpoint(intercepts[0], intercepts[1]);
    }
    point
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Role {
    Outer,
    Hole,
}

struct Entry<'p> {
    path: &'p [Segment2d],
    points: Vec<Point>,
    bounds: Box2,
    interior: Point,
    abs_area: f64,
    winding: i32,
    container: Option<usize>,
    exclude: bool,
    role: Option<Role>,
}

pub fn shapes_of(sub_paths: &[Vec<Segment2d>], even_odd: bool) -> Vec<Outline2d> {
    let inside = |w: i32| if even_odd { w & 1 != 0 } else { w != 0 };
    let mut entries: Vec<Entry<'_>> = sub_paths
        .iter()
        .filter_map(|path| {
            let points = path_points(path, CLASSIFY_DIVISIONS);
            if points.len() < 3 {
                return None;
            }
            let a = area(&points);
            if opsis_types::numbers::exact_eq(a, 0.0) {
                return None;
            }
            let bounds = Box2::of(&points);
            Some(Entry {
                path,
                interior: interior_point(&points, &bounds),
                points,
                bounds,
                abs_area: a.abs(),
                winding: if a < 0.0 { -1 } else { 1 },
                container: None,
                exclude: false,
                role: None,
            })
        })
        .collect();
    entries.sort_by(|a, b| b.abs_area.total_cmp(&a.abs_area));
    for i in 0..entries.len() {
        let mut container_winding = 0;
        for j in (0..i).rev() {
            let candidate = &entries[j];
            if !candidate.bounds.contains(&entries[i].bounds)
                || !point_in_polygon(entries[i].interior, &candidate.points)
            {
                continue;
            }
            let container = if candidate.exclude {
                candidate.container
            } else {
                Some(j)
            };
            container_winding = candidate.winding;
            entries[i].container = container;
            entries[i].winding += container_winding;
            break;
        }
        if inside(entries[i].winding) == inside(container_winding) {
            entries[i].exclude = true;
        }
    }
    for i in 0..entries.len() {
        if entries[i].exclude {
            continue;
        }
        let role = match entries[i].container {
            None => Role::Outer,
            Some(c) if entries[c].role == Some(Role::Hole) => Role::Outer,
            Some(_) => Role::Hole,
        };
        entries[i].role = Some(role);
    }
    let mut shapes: Vec<Outline2d> = Vec::new();
    let mut shape_of: Vec<Option<usize>> = vec![None; entries.len()];
    for (i, entry) in entries.iter().enumerate() {
        if entry.exclude || entry.role != Some(Role::Outer) {
            continue;
        }
        shape_of[i] = Some(shapes.len());
        shapes.push(Outline2d {
            shape: entry.path.to_vec(),
            holes: Vec::new(),
        });
    }
    for entry in &entries {
        if entry.exclude || entry.role != Some(Role::Hole) {
            continue;
        }
        if let Some(shape) = entry.container.and_then(|c| shape_of[c]) {
            shapes[shape].holes.push(entry.path.to_vec());
        }
    }
    shapes
}

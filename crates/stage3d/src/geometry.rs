#![expect(
    clippy::suboptimal_flops,
    clippy::many_single_char_names,
    reason = "a line-by-line port of three.js's geometry, whose JavaScript rounds every step and names its points so; the meshes have to match it"
)]

use std::f64::consts::{FRAC_PI_4, PI, TAU};

use opsis_types::numbers::narrow;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Geometry {
    pub positions: Vec<f32>,
    pub normals: Vec<f32>,
    pub uvs: Vec<f32>,
    pub indices: Vec<u32>,
}

impl Geometry {
    fn push(&mut self, position: [f64; 3], normal: [f64; 3], uv: [f64; 2]) {
        self.positions.extend(position.map(narrow));
        self.normals.extend(normal.map(narrow));
        self.uvs.extend(uv.map(narrow));
    }

    pub const fn vertex_count(&self) -> usize {
        self.positions.len() / 3
    }
}

fn normalized(v: [f64; 3]) -> [f64; 3] {
    let length = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if length == 0.0 {
        return v;
    }
    v.map(|c| c / length)
}

pub fn sphere(radius: f64, width_segments: u32, height_segments: u32) -> Geometry {
    let width_segments = width_segments.max(3);
    let height_segments = height_segments.max(2);
    let mut geometry = Geometry::default();
    let mut grid: Vec<Vec<u32>> = Vec::new();
    let mut next = 0u32;
    for iy in 0..=height_segments {
        let v = f64::from(iy) / f64::from(height_segments);
        let theta = v * PI;
        let y = radius * theta.cos();
        let ring = (radius * radius - y * y).sqrt();
        let u_offset = if iy == 0 {
            0.5 / f64::from(width_segments)
        } else if iy == height_segments {
            -0.5 / f64::from(width_segments)
        } else {
            0.0
        };
        let mut row = Vec::new();
        for ix in 0..=width_segments {
            let u = f64::from(ix) / f64::from(width_segments);
            let phi = u * TAU;
            let vertex = [-ring * phi.cos(), y, ring * phi.sin()];
            geometry.push(vertex, normalized(vertex), [u + u_offset, 1.0 - v]);
            row.push(next);
            next += 1;
        }
        grid.push(row);
    }
    for iy in 0..height_segments {
        for ix in 0..width_segments {
            let (row, below) = (
                &grid[usize::try_from(iy).unwrap_or(0)],
                &grid[usize::try_from(iy + 1).unwrap_or(0)],
            );
            let at = |r: &Vec<u32>, i: u32| r[usize::try_from(i).unwrap_or(0)];
            let (a, b, c, d) = (
                at(row, ix + 1),
                at(row, ix),
                at(below, ix),
                at(below, ix + 1),
            );
            if iy != 0 {
                geometry.indices.extend([a, b, d]);
            }
            if iy != height_segments - 1 {
                geometry.indices.extend([b, c, d]);
            }
        }
    }
    geometry
}

pub fn torus(radius: f64, tube: f64, radial_segments: u32, tubular_segments: u32) -> Geometry {
    let mut geometry = Geometry::default();
    for j in 0..=radial_segments {
        let v = f64::from(j) / f64::from(radial_segments) * TAU;
        for i in 0..=tubular_segments {
            let u = f64::from(i) / f64::from(tubular_segments) * TAU;
            let vertex = [
                (radius + tube * v.cos()) * u.cos(),
                (radius + tube * v.cos()) * u.sin(),
                tube * v.sin(),
            ];
            let center = [radius * u.cos(), radius * u.sin(), 0.0];
            let normal = normalized([
                vertex[0] - center[0],
                vertex[1] - center[1],
                vertex[2] - center[2],
            ]);
            geometry.push(
                vertex,
                normal,
                [
                    f64::from(i) / f64::from(tubular_segments),
                    f64::from(j) / f64::from(radial_segments),
                ],
            );
        }
    }
    for j in 1..=radial_segments {
        for i in 1..=tubular_segments {
            let a = (tubular_segments + 1) * j + i - 1;
            let b = (tubular_segments + 1) * (j - 1) + i - 1;
            let c = (tubular_segments + 1) * (j - 1) + i;
            let d = (tubular_segments + 1) * j + i;
            geometry.indices.extend([a, b, d, b, c, d]);
        }
    }
    geometry
}

#[derive(Debug, Clone, Copy)]
struct Plane {
    u: usize,
    v: usize,
    w: usize,
    udir: f64,
    vdir: f64,
    width: f64,
    height: f64,
    depth: f64,
    grid_x: u32,
    grid_y: u32,
}

fn build_plane(geometry: &mut Geometry, p: Plane) {
    let first = u32::try_from(geometry.vertex_count()).unwrap_or(0);
    let segment_width = p.width / f64::from(p.grid_x);
    let segment_height = p.height / f64::from(p.grid_y);
    let grid_x1 = p.grid_x + 1;
    for iy in 0..=p.grid_y {
        let y = f64::from(iy) * segment_height - p.height / 2.0;
        for ix in 0..=p.grid_x {
            let x = f64::from(ix) * segment_width - p.width / 2.0;
            let mut vertex = [0.0; 3];
            vertex[p.u] = x * p.udir;
            vertex[p.v] = y * p.vdir;
            vertex[p.w] = p.depth / 2.0;
            let mut normal = [0.0; 3];
            normal[p.w] = if p.depth > 0.0 { 1.0 } else { -1.0 };
            geometry.push(
                vertex,
                normal,
                [
                    f64::from(ix) / f64::from(p.grid_x),
                    1.0 - f64::from(iy) / f64::from(p.grid_y),
                ],
            );
        }
    }
    for iy in 0..p.grid_y {
        for ix in 0..p.grid_x {
            let a = first + ix + grid_x1 * iy;
            let b = first + ix + grid_x1 * (iy + 1);
            let c = first + (ix + 1) + grid_x1 * (iy + 1);
            let d = first + (ix + 1) + grid_x1 * iy;
            geometry.indices.extend([a, b, d, b, c, d]);
        }
    }
}

pub fn box_geometry(size: [f64; 3], segments: [u32; 3]) -> Geometry {
    let [width, height, depth] = size;
    let [ws, hs, ds] = segments;
    let mut geometry = Geometry::default();
    let plane = |u, v, w, udir, vdir, dims: (f64, f64, f64), grid: (u32, u32)| Plane {
        u,
        v,
        w,
        udir,
        vdir,
        width: dims.0,
        height: dims.1,
        depth: dims.2,
        grid_x: grid.0,
        grid_y: grid.1,
    };
    for p in [
        plane(2, 1, 0, -1.0, -1.0, (depth, height, width), (ds, hs)),
        plane(2, 1, 0, 1.0, -1.0, (depth, height, -width), (ds, hs)),
        plane(0, 2, 1, 1.0, 1.0, (width, depth, height), (ws, ds)),
        plane(0, 2, 1, 1.0, -1.0, (width, depth, -height), (ws, ds)),
        plane(0, 1, 2, 1.0, -1.0, (width, height, depth), (ws, hs)),
        plane(0, 1, 2, -1.0, -1.0, (width, height, -depth), (ws, hs)),
    ] {
        build_plane(&mut geometry, p);
    }
    geometry
}

fn non_indexed(geometry: &Geometry) -> Geometry {
    let mut flat = Geometry::default();
    for &i in &geometry.indices {
        let at = usize::try_from(i).unwrap_or(0);
        flat.positions
            .extend_from_slice(&geometry.positions[at * 3..at * 3 + 3]);
        flat.normals
            .extend_from_slice(&geometry.normals[at * 3..at * 3 + 3]);
        flat.uvs
            .extend_from_slice(&geometry.uvs[at * 2..at * 2 + 2]);
    }
    flat.indices = (0..u32::try_from(flat.vertex_count()).unwrap_or(0)).collect();
    flat
}

fn angle_between(a: [f64; 3], b: [f64; 3]) -> f64 {
    let denominator = ((a[0] * a[0] + a[1] * a[1] + a[2] * a[2])
        * (b[0] * b[0] + b[1] * b[1] + b[2] * b[2]))
        .sqrt();
    if denominator == 0.0 {
        return std::f64::consts::FRAC_PI_2;
    }
    let theta = (a[0] * b[0] + a[1] * b[1] + a[2] * b[2]) / denominator;
    theta.clamp(-1.0, 1.0).acos()
}

fn rounded_uv(
    face: [f64; 3],
    normal: [f64; 3],
    uv_axis: usize,
    projection_axis: usize,
    radius: f64,
    side: f64,
) -> f64 {
    let arc = TAU * radius / 4.0;
    let center = (side - 2.0 * radius).max(0.0);
    let mut flat = normal;
    flat[projection_axis] = 0.0;
    let flat = normalized(flat);
    let arc_ratio = 0.5 * arc / (arc + center);
    let angle_ratio = 1.0 - angle_between(flat, face) / FRAC_PI_4;
    if flat[uv_axis].signum() > 0.0 && flat[uv_axis] != 0.0 {
        angle_ratio * arc_ratio
    } else {
        center / (arc + center) + arc_ratio + arc_ratio * (1.0 - angle_ratio)
    }
}

fn js_sign(v: f64) -> f64 {
    if v > 0.0 {
        1.0
    } else if v < 0.0 {
        -1.0
    } else {
        0.0
    }
}

pub fn rounded_box(size: [f64; 3], segments: u32, radius: f64) -> Geometry {
    let [width, height, depth] = size;
    let total = segments * 2 + 1;
    let radius = radius.min(width / 2.0).min(height / 2.0).min(depth / 2.0);
    let base = box_geometry([1.0, 1.0, 1.0], [total, total, total]);
    if total == 1 {
        return base;
    }
    let mut flat = non_indexed(&base);
    let half = [
        width / 2.0 - radius,
        height / 2.0 - radius,
        depth / 2.0 - radius,
    ];
    let half_segment = 0.5 / f64::from(total);
    let face_floats = flat.positions.len() / 6;
    let count = flat.vertex_count();
    for vertex in 0..count {
        let i = vertex * 3;
        let position = [
            f64::from(flat.positions[i]),
            f64::from(flat.positions[i + 1]),
            f64::from(flat.positions[i + 2]),
        ];
        let normal = normalized([
            position[0] - js_sign(position[0]) * half_segment,
            position[1] - js_sign(position[1]) * half_segment,
            position[2] - js_sign(position[2]) * half_segment,
        ]);
        for axis in 0..3 {
            flat.positions[i + axis] =
                narrow(half[axis] * js_sign(position[axis]) + normal[axis] * radius);
            flat.normals[i + axis] = narrow(normal[axis]);
        }
        let face = i / face_floats;
        let (ax, ay, az) = (0, 1, 2);
        let uv = match face {
            0 => [
                rounded_uv([1.0, 0.0, 0.0], normal, az, ay, radius, depth),
                1.0 - rounded_uv([1.0, 0.0, 0.0], normal, ay, az, radius, height),
            ],
            1 => [
                1.0 - rounded_uv([-1.0, 0.0, 0.0], normal, az, ay, radius, depth),
                1.0 - rounded_uv([-1.0, 0.0, 0.0], normal, ay, az, radius, height),
            ],
            2 => [
                1.0 - rounded_uv([0.0, 1.0, 0.0], normal, ax, az, radius, width),
                rounded_uv([0.0, 1.0, 0.0], normal, az, ax, radius, depth),
            ],
            3 => [
                1.0 - rounded_uv([0.0, -1.0, 0.0], normal, ax, az, radius, width),
                1.0 - rounded_uv([0.0, -1.0, 0.0], normal, az, ax, radius, depth),
            ],
            4 => [
                1.0 - rounded_uv([0.0, 0.0, 1.0], normal, ax, ay, radius, width),
                1.0 - rounded_uv([0.0, 0.0, 1.0], normal, ay, ax, radius, height),
            ],
            _ => [
                rounded_uv([0.0, 0.0, -1.0], normal, ax, ay, radius, width),
                1.0 - rounded_uv([0.0, 0.0, -1.0], normal, ay, ax, radius, height),
            ],
        };
        let j = vertex * 2;
        flat.uvs[j] = narrow(uv[0]);
        flat.uvs[j + 1] = narrow(uv[1]);
    }
    flat
}

#![expect(
    clippy::suboptimal_flops,
    reason = "each step rounds on its own, and the recorded goldens hold what that draws"
)]

use opsis_scene::clock::js_round;
use opsis_scene::random::js_number_text;
use opsis_types::numbers::whole;

const LIGHT: u8 = 1;
const HEAVY: u8 = 2;
const DOUBLE: u8 = 3;
const SHADE_LIGHT: f64 = 0.25;
const SHADE_MEDIUM: f64 = 0.5;
const SHADE_DARK: f64 = 0.75;
const ARC_TENSION: f64 = 0.25;
const BEZIER_CIRCLE: f64 = (std::f64::consts::SQRT_2 - 1.0) * 4.0 / 3.0;
const UNDERLINE_EM: f64 = 0.05;
const WHOLE_PIXEL: f64 = 1e-6;
const ONE_EIGHTH: f64 = 0.125;
const ONE_THIRD: f64 = 1.0 / 3.0;
const TWO_THIRDS: f64 = 2.0 / 3.0;
const QUARTERS: [f64; 5] = [0.0, 0.25, 0.5, 0.75, 1.0];
const EIGHTHS: [f64; 9] = [0.0, 0.125, 0.25, 0.375, 0.5, 0.625, 0.75, 0.875, 1.0];
const FULL_ALPHA: f64 = 255.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cell {
    pub width: f64,
    pub height: f64,
    pub thickness: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Seg {
    Move(f64, f64),
    Line(f64, f64),
    Curve([f64; 6]),
    Close,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Path(pub Vec<Seg>);

impl Path {
    #[expect(
        clippy::many_single_char_names,
        reason = "the names are the parts of each box-drawing glyph, one term per stroke"
    )]
    fn map(&self, x: impl Fn(f64) -> f64, y: impl Fn(f64) -> f64) -> Self {
        Self(
            self.0
                .iter()
                .map(|seg| match *seg {
                    Seg::Move(a, b) => Seg::Move(x(a), y(b)),
                    Seg::Line(a, b) => Seg::Line(x(a), y(b)),
                    Seg::Curve([a, b, c, d, e, f]) => {
                        Seg::Curve([x(a), y(b), x(c), y(d), x(e), y(f)])
                    }
                    Seg::Close => Seg::Close,
                })
                .collect(),
        )
    }

    #[must_use]
    pub fn moved(&self, dx: f64, dy: f64) -> Self {
        self.map(|x| x + dx, |y| y + dy)
    }

    #[must_use]
    pub fn mirrored(&self, width: f64) -> Self {
        self.map(|x| width - x, |y| y)
    }

    #[must_use]
    pub fn d(&self) -> String {
        let n = |v: f64| js_number_text(v);
        self.0
            .iter()
            .map(|seg| match *seg {
                Seg::Move(x, y) => format!("M{} {}", n(x), n(y)),
                Seg::Line(x, y) => format!("L{} {}", n(x), n(y)),
                Seg::Curve(v) => format!("C{}", v.map(n).join(" ")),
                Seg::Close => "Z".to_owned(),
            })
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Shape {
    Rect {
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        opacity: f64,
    },
    Fill {
        path: Path,
        opacity: Option<f64>,
        clip: bool,
    },
    Stroke {
        path: Path,
        width: f64,
        clip: bool,
        inner: bool,
    },
    Inverted(Vec<Self>),
}

pub fn stroke_for(font_size: f64) -> f64 {
    1.0f64.max((font_size * UNDERLINE_EM).ceil())
}

const BOX_LINES: [(u32, &str); 105] = [
    (0x2500, "0101"),
    (0x2501, "0202"),
    (0x2502, "1010"),
    (0x2503, "2020"),
    (0x250c, "0110"),
    (0x250d, "0210"),
    (0x250e, "0120"),
    (0x250f, "0220"),
    (0x2510, "0011"),
    (0x2511, "0012"),
    (0x2512, "0021"),
    (0x2513, "0022"),
    (0x2514, "1100"),
    (0x2515, "1200"),
    (0x2516, "2100"),
    (0x2517, "2200"),
    (0x2518, "1001"),
    (0x2519, "1002"),
    (0x251a, "2001"),
    (0x251b, "2002"),
    (0x251c, "1110"),
    (0x251d, "1210"),
    (0x251e, "2110"),
    (0x251f, "1120"),
    (0x2520, "2120"),
    (0x2521, "2210"),
    (0x2522, "1220"),
    (0x2523, "2220"),
    (0x2524, "1011"),
    (0x2525, "1012"),
    (0x2526, "2011"),
    (0x2527, "1021"),
    (0x2528, "2021"),
    (0x2529, "2012"),
    (0x252a, "1022"),
    (0x252b, "2022"),
    (0x252c, "0111"),
    (0x252d, "0112"),
    (0x252e, "0211"),
    (0x252f, "0212"),
    (0x2530, "0121"),
    (0x2531, "0122"),
    (0x2532, "0221"),
    (0x2533, "0222"),
    (0x2534, "1101"),
    (0x2535, "1102"),
    (0x2536, "1201"),
    (0x2537, "1202"),
    (0x2538, "2101"),
    (0x2539, "2102"),
    (0x253a, "2201"),
    (0x253b, "2202"),
    (0x253c, "1111"),
    (0x253d, "1112"),
    (0x253e, "1211"),
    (0x253f, "1212"),
    (0x2540, "2111"),
    (0x2541, "1121"),
    (0x2542, "2121"),
    (0x2543, "2112"),
    (0x2544, "2211"),
    (0x2545, "1122"),
    (0x2546, "1221"),
    (0x2547, "2212"),
    (0x2548, "1222"),
    (0x2549, "2122"),
    (0x254a, "2221"),
    (0x254b, "2222"),
    (0x2550, "0303"),
    (0x2551, "3030"),
    (0x2552, "0310"),
    (0x2553, "0130"),
    (0x2554, "0330"),
    (0x2555, "0013"),
    (0x2556, "0031"),
    (0x2557, "0033"),
    (0x2558, "1300"),
    (0x2559, "3100"),
    (0x255a, "3300"),
    (0x255b, "1003"),
    (0x255c, "3001"),
    (0x255d, "3003"),
    (0x255e, "1310"),
    (0x255f, "3130"),
    (0x2560, "3330"),
    (0x2561, "1013"),
    (0x2562, "3031"),
    (0x2563, "3033"),
    (0x2564, "0313"),
    (0x2565, "0131"),
    (0x2566, "0333"),
    (0x2567, "1303"),
    (0x2568, "3101"),
    (0x2569, "3303"),
    (0x256a, "1313"),
    (0x256b, "3131"),
    (0x256c, "3333"),
    (0x2574, "0001"),
    (0x2575, "1000"),
    (0x2576, "0100"),
    (0x2577, "0010"),
    (0x2578, "0002"),
    (0x2579, "2000"),
    (0x257a, "0200"),
    (0x257b, "0020"),
];

const BOX_LINES_MORE: [(u32, &str); 4] = [
    (0x257c, "0201"),
    (0x257d, "1020"),
    (0x257e, "0102"),
    (0x257f, "2010"),
];

const MOSAICS: [(u32, &str); 44] = [
    (0x1fb3c, "...|...|#..|##."),
    (0x1fb3d, "...|...|#\\.|###"),
    (0x1fb3e, "...|#..|#\\.|##."),
    (0x1fb3f, "...|#..|##.|###"),
    (0x1fb40, "#..|#..|##.|##."),
    (0x1fb41, "/##|###|###|###"),
    (0x1fb42, "./#|###|###|###"),
    (0x1fb43, ".##|.##|###|###"),
    (0x1fb44, "..#|.##|###|###"),
    (0x1fb45, ".##|.##|.##|###"),
    (0x1fb46, "...|./#|###|###"),
    (0x1fb47, "...|...|..#|.##"),
    (0x1fb48, "...|...|./#|###"),
    (0x1fb49, "...|..#|./#|.##"),
    (0x1fb4a, "...|..#|.##|###"),
    (0x1fb4b, "..#|..#|.##|.##"),
    (0x1fb4c, "##\\|###|###|###"),
    (0x1fb4d, "#\\.|###|###|###"),
    (0x1fb4e, "##.|##.|###|###"),
    (0x1fb4f, "#..|##.|###|###"),
    (0x1fb50, "##.|##.|##.|###"),
    (0x1fb51, "...|#\\.|###|###"),
    (0x1fb52, "###|###|###|\\##"),
    (0x1fb53, "###|###|###|.\\#"),
    (0x1fb54, "###|###|.##|.##"),
    (0x1fb55, "###|###|.##|..#"),
    (0x1fb56, "###|.##|.##|.##"),
    (0x1fb57, "##.|#..|...|..."),
    (0x1fb58, "###|#/.|...|..."),
    (0x1fb59, "##.|#/.|#..|..."),
    (0x1fb5a, "###|##.|#..|..."),
    (0x1fb5b, "##.|##.|#..|#.."),
    (0x1fb5c, "###|###|#/.|..."),
    (0x1fb5d, "###|###|###|##/"),
    (0x1fb5e, "###|###|###|#/."),
    (0x1fb5f, "###|###|##.|##."),
    (0x1fb60, "###|###|##.|#.."),
    (0x1fb61, "###|##.|##.|##."),
    (0x1fb62, ".##|..#|...|..."),
    (0x1fb63, "###|.\\#|...|..."),
    (0x1fb64, ".##|.\\#|..#|..."),
    (0x1fb65, "###|.##|..#|..."),
    (0x1fb66, ".##|.##|..#|..#"),
    (0x1fb67, "###|###|.\\#|..."),
];

const OCTANTS: [u32; 230] = [
    4, 6, 7, 8, 9, 11, 12, 13, 14, 16, 17, 18, 19, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32,
    33, 34, 35, 36, 37, 38, 39, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57,
    58, 59, 60, 61, 62, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 81, 82, 83, 84,
    86, 87, 88, 89, 91, 92, 93, 94, 96, 97, 98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108,
    109, 110, 111, 112, 113, 114, 115, 116, 117, 118, 119, 120, 121, 122, 123, 124, 125, 126, 127,
    129, 130, 131, 132, 133, 134, 135, 136, 137, 138, 139, 140, 141, 142, 143, 144, 145, 146, 147,
    148, 149, 150, 151, 152, 153, 154, 155, 156, 157, 158, 159, 161, 162, 163, 164, 166, 167, 168,
    169, 171, 172, 173, 174, 176, 177, 178, 179, 180, 181, 182, 183, 184, 185, 186, 187, 188, 189,
    190, 191, 193, 194, 195, 196, 197, 198, 199, 200, 201, 202, 203, 204, 205, 206, 207, 208, 209,
    210, 211, 212, 213, 214, 215, 216, 217, 218, 219, 220, 221, 222, 223, 224, 225, 226, 227, 228,
    229, 230, 231, 232, 233, 234, 235, 236, 237, 238, 239, 241, 242, 243, 244, 246, 247, 248, 249,
    251, 253, 254,
];

fn rect(x0: f64, y0: f64, x1: f64, y1: f64, opacity: f64) -> Shape {
    Shape::Rect {
        x: x0,
        y: y0,
        width: 0.0f64.max(x1 - x0),
        height: 0.0f64.max(y1 - y0),
        opacity,
    }
}

fn solid(x0: f64, y0: f64, x1: f64, y1: f64) -> Shape {
    rect(x0, y0, x1, y1, 1.0)
}

fn min_at(fraction: f64, size: f64) -> f64 {
    size - js_round((1.0 - fraction) * size)
}

fn max_at(fraction: f64, size: f64) -> f64 {
    js_round(fraction * size)
}

fn fill(c: Cell, (x0, x1): (f64, f64), (y0, y1): (f64, f64)) -> Shape {
    solid(
        min_at(x0, c.width),
        min_at(y0, c.height),
        max_at(x1, c.width),
        max_at(y1, c.height),
    )
}

fn heavy(c: Cell) -> f64 {
    c.thickness * 2.0
}

const fn path_shape(path: Path) -> Shape {
    Shape::Fill {
        path,
        opacity: None,
        clip: false,
    }
}

const fn stroke(path: Path, width: f64) -> Shape {
    Shape::Stroke {
        path,
        width,
        clip: false,
        inner: false,
    }
}

fn polygon(points: &[(f64, f64)]) -> Path {
    let mut segs: Vec<Seg> = points
        .iter()
        .enumerate()
        .map(|(i, &(x, y))| {
            if i == 0 {
                Seg::Move(x, y)
            } else {
                Seg::Line(x, y)
            }
        })
        .collect();
    segs.push(Seg::Close);
    Path(segs)
}

#[derive(Debug, Clone, Copy)]
struct Sides {
    up: u8,
    right: u8,
    down: u8,
    left: u8,
}

fn sides_of(code: &str) -> Sides {
    let digits: Vec<u8> = code.bytes().map(|b| b - b'0').collect();
    let at = |i: usize| digits.get(i).copied().unwrap_or(0);
    Sides {
        up: at(0),
        right: at(1),
        down: at(2),
        left: at(3),
    }
}

const fn any_heavy(a: u8, b: u8) -> bool {
    a == HEAVY || b == HEAVY
}

const fn any_double(a: u8, b: u8) -> bool {
    a == DOUBLE || b == DOUBLE
}

#[derive(Debug, Clone, Copy)]
struct Band {
    light: (f64, f64),
    heavy: (f64, f64),
    double: (f64, f64),
}

fn band(size: f64, light: f64, strong: f64) -> Band {
    let light_low = ((size - light) / 2.0).floor();
    let heavy_low = ((size - strong) / 2.0).floor();
    Band {
        light: (light_low, light_low + light),
        heavy: (heavy_low, heavy_low + strong),
        double: (light_low - light, light_low + light + light),
    }
}

fn reach(b: Band, across: (u8, u8), along: (u8, u8), toward_low: bool) -> f64 {
    let (a, z) = across;
    let (near, far) = along;
    let pick = |low: f64, high: f64| if toward_low { low } else { high };
    if any_heavy(a, z) {
        pick(b.heavy.0, b.heavy.1)
    } else if a != z || near == far {
        if any_double(a, z) {
            pick(b.double.0, b.double.1)
        } else {
            pick(b.light.0, b.light.1)
        }
    } else if a == 0 && z == 0 {
        pick(b.light.0, b.light.1)
    } else {
        pick(b.light.1, b.light.0)
    }
}

fn lines(c: Cell, code: &str) -> Vec<Shape> {
    let s = sides_of(code);
    let h = band(c.height, c.thickness, heavy(c));
    let v = band(c.width, c.thickness, heavy(c));
    let up_bottom = reach(h, (s.left, s.right), (s.down, s.up), false);
    let down_top = reach(h, (s.left, s.right), (s.up, s.down), true);
    let left_right = reach(v, (s.up, s.down), (s.left, s.right), false);
    let right_left = reach(v, (s.up, s.down), (s.right, s.left), true);
    let (ht, hb) = h.light;
    let (vl, vr) = v.light;
    let mut out = Vec::new();
    match s.up {
        LIGHT => out.push(solid(vl, 0.0, vr, up_bottom)),
        HEAVY => out.push(solid(v.heavy.0, 0.0, v.heavy.1, up_bottom)),
        DOUBLE => {
            out.push(solid(
                v.double.0,
                0.0,
                vl,
                if s.left == DOUBLE { ht } else { up_bottom },
            ));
            out.push(solid(
                vr,
                0.0,
                v.double.1,
                if s.right == DOUBLE { ht } else { up_bottom },
            ));
        }
        _ => {}
    }
    match s.right {
        LIGHT => out.push(solid(right_left, ht, c.width, hb)),
        HEAVY => out.push(solid(right_left, h.heavy.0, c.width, h.heavy.1)),
        DOUBLE => {
            out.push(solid(
                if s.up == DOUBLE { vr } else { right_left },
                h.double.0,
                c.width,
                ht,
            ));
            out.push(solid(
                if s.down == DOUBLE { vr } else { right_left },
                hb,
                c.width,
                h.double.1,
            ));
        }
        _ => {}
    }
    match s.down {
        LIGHT => out.push(solid(vl, down_top, vr, c.height)),
        HEAVY => out.push(solid(v.heavy.0, down_top, v.heavy.1, c.height)),
        DOUBLE => {
            out.push(solid(
                v.double.0,
                if s.left == DOUBLE { hb } else { down_top },
                vl,
                c.height,
            ));
            out.push(solid(
                vr,
                if s.right == DOUBLE { hb } else { down_top },
                v.double.1,
                c.height,
            ));
        }
        _ => {}
    }
    match s.left {
        LIGHT => out.push(solid(0.0, ht, left_right, hb)),
        HEAVY => out.push(solid(0.0, h.heavy.0, left_right, h.heavy.1)),
        DOUBLE => {
            out.push(solid(
                0.0,
                h.double.0,
                if s.up == DOUBLE { vl } else { left_right },
                ht,
            ));
            out.push(solid(
                0.0,
                hb,
                if s.down == DOUBLE { vl } else { left_right },
                h.double.1,
            ));
        }
        _ => {}
    }
    out
}

fn dashes(c: Cell, count_whole: u32, thick: f64, gap: f64, across: bool) -> Vec<Shape> {
    let count = f64::from(count_whole);
    let length = ((if across { c.width } else { c.height }) + WHOLE_PIXEL).floor();
    if length < count * 2.0 {
        let y = ((c.height - c.thickness) / 2.0).floor();
        let x = ((c.width - c.thickness) / 2.0).floor();
        return vec![if across {
            solid(0.0, y, c.width, y + c.thickness)
        } else {
            solid(x, 0.0, x + c.thickness, c.height)
        }];
    }
    let gap_length = gap.min((length / (2.0 * count)).floor());
    let dash_total = length - count * gap_length;
    let dash = (dash_total / count).floor();
    let mut extra = dash_total - dash * count;
    let mut out = Vec::new();
    let mut at = if across {
        (gap_length / 2.0).floor()
    } else {
        0.0
    };
    for _ in 0..count_whole {
        let mut end = at + dash;
        if extra > 0.0 {
            extra -= 1.0;
            end += 1.0;
        }
        if across {
            let y = ((c.height - thick) / 2.0).floor();
            out.push(solid(at, y, end, y + thick));
        } else {
            let x = ((c.width - thick) / 2.0).floor();
            out.push(solid(x, at, x + thick, end));
        }
        at = end + gap_length;
    }
    out
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Corner {
    Tl,
    Tr,
    Bl,
    Br,
}

fn arc(c: Cell, corner: Corner) -> Shape {
    let t = c.thickness;
    let cx = ((c.width - t) / 2.0).floor() + t / 2.0;
    let cy = ((c.height - t) / 2.0).floor() + t / 2.0;
    let r = c.width.min(c.height) / 2.0;
    let s = ARC_TENSION;
    let top = matches!(corner, Corner::Tl | Corner::Tr);
    let leftward = matches!(corner, Corner::Tl | Corner::Bl);
    let edge_y = if top { 0.0 } else { c.height };
    let toward_y = if top { cy - r } else { cy + r };
    let handle_y = if top { cy - s * r } else { cy + s * r };
    let side_x = if leftward { cx - r } else { cx + r };
    let handle_x = if leftward { cx - s * r } else { cx + s * r };
    let end_x = if leftward { 0.0 } else { c.width };
    stroke(
        Path(vec![
            Seg::Move(cx, edge_y),
            Seg::Line(cx, toward_y),
            Seg::Curve([cx, handle_y, handle_x, cy, side_x, cy]),
            Seg::Line(end_x, cy),
        ]),
        t,
    )
}

fn diagonal(c: Cell, rising: bool) -> Shape {
    let sx = 1.0f64.min(c.width / c.height);
    let sy = 1.0f64.min(c.height / c.width);
    let path = if rising {
        Path(vec![
            Seg::Move(c.width + 0.5 * sx, -0.5 * sy),
            Seg::Line(-0.5 * sx, c.height + 0.5 * sy),
        ])
    } else {
        Path(vec![
            Seg::Move(-0.5 * sx, -0.5 * sy),
            Seg::Line(c.width + 0.5 * sx, c.height + 0.5 * sy),
        ])
    };
    stroke(path, c.thickness)
}

fn box_line_code(cp: u32) -> Option<&'static str> {
    BOX_LINES
        .iter()
        .chain(BOX_LINES_MORE.iter())
        .find(|(code, _)| *code == cp)
        .map(|(_, pattern)| *pattern)
}

fn box_drawing(cp: u32, c: Cell) -> Option<Vec<Shape>> {
    if let Some(code) = box_line_code(cp) {
        return Some(lines(c, code));
    }
    let gap_light = 4.0f64.max(c.thickness);
    let t = c.thickness;
    Some(match cp {
        0x2504 => dashes(c, 3, t, gap_light, true),
        0x2505 => dashes(c, 3, heavy(c), gap_light, true),
        0x2506 => dashes(c, 3, t, gap_light, false),
        0x2507 => dashes(c, 3, heavy(c), gap_light, false),
        0x2508 => dashes(c, 4, t, gap_light, true),
        0x2509 => dashes(c, 4, heavy(c), gap_light, true),
        0x250a => dashes(c, 4, t, gap_light, false),
        0x250b => dashes(c, 4, heavy(c), gap_light, false),
        0x254c => dashes(c, 2, t, t, true),
        0x254d => dashes(c, 2, heavy(c), heavy(c), true),
        0x254e => dashes(c, 2, t, heavy(c), false),
        0x254f => dashes(c, 2, heavy(c), heavy(c), false),
        0x256d => vec![arc(c, Corner::Br)],
        0x256e => vec![arc(c, Corner::Bl)],
        0x256f => vec![arc(c, Corner::Tl)],
        0x2570 => vec![arc(c, Corner::Tr)],
        0x2571 => vec![diagonal(c, true)],
        0x2572 => vec![diagonal(c, false)],
        0x2573 => vec![diagonal(c, true), diagonal(c, false)],
        _ => return None,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Align {
    Upper,
    Lower,
    Left,
    Right,
}

#[expect(
    clippy::many_single_char_names,
    reason = "the names are the parts of each box-drawing glyph, one term per stroke"
)]
fn block(c: Cell, align: Align, (width, height): (f64, f64), opacity: f64) -> Shape {
    let w = js_round(c.width * width);
    let h = js_round(c.height * height);
    let x = match align {
        Align::Right => c.width - w,
        Align::Left => 0.0,
        _ => ((c.width - w) / 2.0).floor(),
    };
    let y = match align {
        Align::Lower => c.height - h,
        Align::Upper => 0.0,
        _ => ((c.height - h) / 2.0).floor(),
    };
    rect(x, y, x + w, y + h, opacity)
}

fn eighth(n: usize) -> f64 {
    EIGHTHS.get(n).copied().unwrap_or(0.0)
}

fn index_from(cp: u32, from: u32) -> usize {
    usize::try_from(cp - from).unwrap_or(0)
}

fn quadrants(c: Cell, [tl, tr, bl, br]: [bool; 4]) -> Vec<Shape> {
    let mut out = Vec::new();
    if tl {
        out.push(fill(c, (0.0, 0.5), (0.0, 0.5)));
    }
    if tr {
        out.push(fill(c, (0.5, 1.0), (0.0, 0.5)));
    }
    if bl {
        out.push(fill(c, (0.0, 0.5), (0.5, 1.0)));
    }
    if br {
        out.push(fill(c, (0.5, 1.0), (0.5, 1.0)));
    }
    out
}

fn blocks(cp: u32, c: Cell) -> Option<Vec<Shape>> {
    let whole = |opacity: f64| vec![rect(0.0, 0.0, c.width, c.height, opacity)];
    Some(match cp {
        0x2580 => vec![block(c, Align::Upper, (1.0, 0.5), 1.0)],
        0x2581..=0x2587 => vec![block(
            c,
            Align::Lower,
            (1.0, eighth(index_from(cp, 0x2580))),
            1.0,
        )],
        0x2588 => whole(1.0),
        0x2589..=0x258f => vec![block(
            c,
            Align::Left,
            (eighth(index_from(0x2590, cp)), 1.0),
            1.0,
        )],
        0x2590 => vec![block(c, Align::Right, (0.5, 1.0), 1.0)],
        0x2591 => whole(SHADE_LIGHT),
        0x2592 => whole(SHADE_MEDIUM),
        0x2593 => whole(SHADE_DARK),
        0x2594 => vec![block(c, Align::Upper, (1.0, 0.125), 1.0)],
        0x2595 => vec![block(c, Align::Right, (0.125, 1.0), 1.0)],
        0x2596 => quadrants(c, [false, false, true, false]),
        0x2597 => quadrants(c, [false, false, false, true]),
        0x2598 => quadrants(c, [true, false, false, false]),
        0x2599 => quadrants(c, [true, false, true, true]),
        0x259a => quadrants(c, [true, false, false, true]),
        0x259b => quadrants(c, [true, true, true, false]),
        0x259c => quadrants(c, [true, true, false, true]),
        0x259d => quadrants(c, [false, true, false, false]),
        0x259e => quadrants(c, [false, true, true, false]),
        0x259f => quadrants(c, [false, true, true, true]),
        _ => return None,
    })
}

#[derive(Debug, Clone, Copy)]
struct BrailleGrid {
    dot: f64,
    xs: [f64; 2],
    ys: [f64; 4],
}

fn braille_grid(c: Cell) -> BrailleGrid {
    let width = c.width.floor();
    let height = c.height.floor();
    let mut w = (width / 4.0).floor().min((height / 8.0).floor());
    let mut x_spacing = (width / 4.0).floor();
    let mut y_spacing = (height / 8.0).floor();
    let mut x_margin = (x_spacing / 2.0).floor();
    let mut y_margin = (y_spacing / 2.0).floor();
    let mut x_left = width - 2.0 * x_margin - x_spacing - 2.0 * w;
    let mut y_left = height - 2.0 * y_margin - 3.0 * y_spacing - 4.0 * w;
    if x_left >= 2.0 && y_left >= 4.0 && w == 0.0 {
        w += 1.0;
        x_left -= 2.0;
        y_left -= 4.0;
    }
    if x_left >= 2.0 && x_margin == 0.0 {
        x_margin = 1.0;
        x_left -= 2.0;
    }
    if y_left >= 2.0 && y_margin == 0.0 {
        y_margin = 1.0;
        y_left -= 2.0;
    }
    if x_left >= 1.0 {
        x_spacing += 1.0;
        x_left -= 1.0;
    }
    if y_left >= 3.0 {
        y_spacing += 1.0;
        y_left -= 3.0;
    }
    if x_left >= 2.0 {
        x_margin += 1.0;
        x_left -= 2.0;
    }
    if y_left >= 2.0 {
        y_margin += 1.0;
        y_left -= 2.0;
    }
    if x_left >= 2.0 && y_left >= 4.0 {
        w += 1.0;
    }
    BrailleGrid {
        dot: w,
        xs: [x_margin, x_margin + w + x_spacing],
        ys: [0.0, 1.0, 2.0, 3.0].map(|i| y_margin + i * (w + y_spacing)),
    }
}

fn braille(cp: u32, c: Cell) -> Vec<Shape> {
    let grid = braille_grid(c);
    let bits = cp - 0x2800;
    let dots: [(usize, usize); 8] = [
        (0, 0),
        (0, 1),
        (0, 2),
        (1, 0),
        (1, 1),
        (1, 2),
        (0, 3),
        (1, 3),
    ];
    dots.iter()
        .enumerate()
        .filter(|(i, _)| bits & (1 << i) != 0)
        .map(|(_, &(col, row))| {
            let x = grid.xs[col];
            let y = grid.ys[row];
            solid(x, y, x + grid.dot, y + grid.dot)
        })
        .collect()
}

fn flip(shape: &Shape, width: f64) -> Shape {
    match shape {
        Shape::Rect {
            x,
            y,
            width: w,
            height,
            opacity,
        } => Shape::Rect {
            x: width - x - w,
            y: *y,
            width: *w,
            height: *height,
            opacity: *opacity,
        },
        Shape::Inverted(shapes) => Shape::Inverted(shapes.iter().map(|s| flip(s, width)).collect()),
        Shape::Fill {
            path,
            opacity,
            clip,
        } => Shape::Fill {
            path: path.mirrored(width),
            opacity: *opacity,
            clip: *clip,
        },
        Shape::Stroke {
            path,
            width: w,
            clip,
            inner,
        } => Shape::Stroke {
            path: path.mirrored(width),
            width: *w,
            clip: *clip,
            inner: *inner,
        },
    }
}

fn half_circle(c: Cell, outline: bool) -> Shape {
    let r = c.width.min(c.height / 2.0);
    let k = BEZIER_CIRCLE * r;
    let start = if outline { 1.0 } else { 0.0 };
    let mut segs = vec![Seg::Move(0.0, 0.0)];
    if outline {
        segs.push(Seg::Line(start, 0.0));
    }
    segs.push(Seg::Curve([k, 0.0, r, r - k, r, r]));
    segs.push(Seg::Line(r, c.height - r));
    segs.push(Seg::Curve([
        r,
        c.height - r + k,
        k,
        c.height,
        start,
        c.height,
    ]));
    segs.push(if outline {
        Seg::Line(0.0, c.height)
    } else {
        Seg::Close
    });
    if outline {
        Shape::Stroke {
            path: Path(segs),
            width: c.thickness,
            clip: false,
            inner: true,
        }
    } else {
        path_shape(Path(segs))
    }
}

fn open_path(points: &[(f64, f64)]) -> Path {
    Path(
        points
            .iter()
            .enumerate()
            .map(|(i, &(x, y))| {
                if i == 0 {
                    Seg::Move(x, y)
                } else {
                    Seg::Line(x, y)
                }
            })
            .collect(),
    )
}

fn powerline(cp: u32, c: Cell) -> Option<Vec<Shape>> {
    let (w, h) = (c.width, c.height);
    let triangle = |points: &[(f64, f64)]| vec![path_shape(polygon(points))];
    Some(match cp {
        0xe0b0 => triangle(&[(0.0, 0.0), (w, h / 2.0), (0.0, h)]),
        0xe0b2 => triangle(&[(w, 0.0), (0.0, h / 2.0), (w, h)]),
        0xe0b1 => vec![stroke(
            open_path(&[(0.0, 0.0), (w, h / 2.0), (0.0, h)]),
            c.thickness,
        )],
        0xe0b3 => vec![stroke(
            open_path(&[(w, 0.0), (0.0, h / 2.0), (w, h)]),
            c.thickness,
        )],
        0xe0b4 => vec![half_circle(c, false)],
        0xe0b5 => vec![half_circle(c, true)],
        0xe0b6 => vec![flip(&half_circle(c, false), w)],
        0xe0b7 => vec![flip(&half_circle(c, true), w)],
        0xe0b8 => triangle(&[(0.0, 0.0), (w, h), (0.0, h)]),
        0xe0ba => triangle(&[(w, 0.0), (w, h), (0.0, h)]),
        0xe0bc => triangle(&[(0.0, 0.0), (w, 0.0), (0.0, h)]),
        0xe0be => triangle(&[(0.0, 0.0), (w, 0.0), (w, h)]),
        0xe0b9 | 0xe0bf => vec![diagonal(c, false)],
        0xe0bb | 0xe0bd => vec![diagonal(c, true)],
        _ => return None,
    })
}

fn inner_outline(c: Cell, points: &[(f64, f64)]) -> Vec<Shape> {
    vec![Shape::Stroke {
        path: polygon(points),
        width: c.thickness,
        clip: false,
        inner: true,
    }]
}

fn geometric(cp: u32, c: Cell) -> Option<Vec<Shape>> {
    let (w, h) = (c.width, c.height);
    let triangle = |points: &[(f64, f64)]| vec![path_shape(polygon(points))];
    Some(match cp {
        0x25e2 => triangle(&[(w, 0.0), (w, h), (0.0, h)]),
        0x25e3 => triangle(&[(0.0, 0.0), (w, h), (0.0, h)]),
        0x25e4 => triangle(&[(0.0, 0.0), (w, 0.0), (0.0, h)]),
        0x25e5 => triangle(&[(0.0, 0.0), (w, 0.0), (w, h)]),
        0x25f8 => inner_outline(c, &[(0.0, 0.0), (w, 0.0), (0.0, h)]),
        0x25f9 => inner_outline(c, &[(0.0, 0.0), (w, 0.0), (w, h)]),
        0x25fa => inner_outline(c, &[(0.0, 0.0), (w, h), (0.0, h)]),
        0x25ff => inner_outline(c, &[(w, 0.0), (w, h), (0.0, h)]),
        _ => return None,
    })
}

fn parts_shown(c: Cell, bits: u32, parts: &[(f64, f64, f64, f64)]) -> Vec<Shape> {
    parts
        .iter()
        .enumerate()
        .filter(|(i, _)| bits & (1 << i) != 0)
        .map(|(_, &(x0, x1, y0, y1))| fill(c, (x0, x1), (y0, y1)))
        .collect()
}

fn sextants(cp: u32, c: Cell) -> Vec<Shape> {
    let index = cp - 0x1fb00;
    let bits = index + index / 0x14 + 1;
    parts_shown(
        c,
        bits,
        &[
            (0.0, 0.5, 0.0, ONE_THIRD),
            (0.5, 1.0, 0.0, ONE_THIRD),
            (0.0, 0.5, ONE_THIRD, TWO_THIRDS),
            (0.5, 1.0, ONE_THIRD, TWO_THIRDS),
            (0.0, 0.5, TWO_THIRDS, 1.0),
            (0.5, 1.0, TWO_THIRDS, 1.0),
        ],
    )
}

fn mosaic(pattern: &str, c: Cell) -> Shape {
    let bytes = pattern.as_bytes();
    let at = |i: usize| bytes.get(i) == Some(&b'#');
    let corners: [(bool, f64, f64); 10] = [
        (at(0), 0.0, 0.0),
        (at(4) && (!at(0) || !at(8)), 0.0, ONE_THIRD),
        (at(8) && (!at(4) || !at(12)), 0.0, TWO_THIRDS),
        (at(12), 0.0, 1.0),
        (at(13) && (!at(12) || !at(14)), 0.5, 1.0),
        (at(14), 1.0, 1.0),
        (at(10) && (!at(14) || !at(6)), 1.0, TWO_THIRDS),
        (at(6) && (!at(10) || !at(2)), 1.0, ONE_THIRD),
        (at(2), 1.0, 0.0),
        (at(1) && (!at(2) || !at(0)), 0.5, 0.0),
    ];
    let points: Vec<(f64, f64)> = corners
        .iter()
        .filter(|(on, _, _)| *on)
        .map(|&(_, fx, fy)| (fx * c.width, fy * c.height))
        .collect();
    path_shape(polygon(&points))
}

fn legacy(cp: u32, c: Cell) -> Option<Vec<Shape>> {
    if (0x1fb00..=0x1fb3b).contains(&cp) {
        return Some(sextants(cp, c));
    }
    if let Some((_, pattern)) = MOSAICS.iter().find(|(code, _)| *code == cp) {
        return Some(vec![mosaic(pattern, c)]);
    }
    if (0x1fb70..=0x1fb75).contains(&cp) {
        let n = index_from(cp + 1, 0x1fb70);
        return Some(vec![fill(
            c,
            (eighth(n), EIGHTHS.get(n + 1).copied().unwrap_or(1.0)),
            (0.0, 1.0),
        )]);
    }
    if (0x1fb76..=0x1fb7b).contains(&cp) {
        let n = index_from(cp + 1, 0x1fb76);
        return Some(vec![fill(
            c,
            (0.0, 1.0),
            (eighth(n), EIGHTHS.get(n + 1).copied().unwrap_or(1.0)),
        )]);
    }
    let top = match cp {
        0x1fb82 => Some(0.25),
        0x1fb83 => Some(0.375),
        0x1fb84 => Some(0.625),
        0x1fb85 => Some(0.75),
        0x1fb86 => Some(0.875),
        _ => None,
    };
    if let Some(share) = top {
        return Some(vec![block(c, Align::Upper, (1.0, share), 1.0)]);
    }
    let right = match cp {
        0x1fb87 => Some(0.25),
        0x1fb88 => Some(0.375),
        0x1fb89 => Some(0.625),
        0x1fb8a => Some(0.75),
        0x1fb8b => Some(0.875),
        _ => None,
    };
    if let Some(share) = right {
        return Some(vec![block(c, Align::Right, (share, 1.0), 1.0)]);
    }
    if (0x1cd00..=0x1cde5).contains(&cp) {
        let bits = OCTANTS.get(index_from(cp, 0x1cd00)).copied().unwrap_or(0);
        return Some(parts_shown(
            c,
            bits,
            &[
                (0.0, 0.5, 0.0, 0.25),
                (0.5, 1.0, 0.0, 0.25),
                (0.0, 0.5, 0.25, 0.5),
                (0.5, 1.0, 0.25, 0.5),
                (0.0, 0.5, 0.5, 0.75),
                (0.5, 1.0, 0.5, 0.75),
                (0.0, 0.5, 0.75, 1.0),
                (0.5, 1.0, 0.75, 1.0),
            ],
        ));
    }
    None
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Edge {
    Top,
    Right,
    Bottom,
    Left,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Spot {
    Left,
    Center,
    Right,
    Top,
    Middle,
    Bottom,
}

fn line(from: (f64, f64), to: (f64, f64), width: f64, clip: bool) -> Shape {
    Shape::Stroke {
        path: Path(vec![Seg::Move(from.0, from.1), Seg::Line(to.0, to.1)]),
        width,
        clip,
        inner: false,
    }
}

fn circle_path(cx: f64, cy: f64, r: f64) -> Path {
    let k = BEZIER_CIRCLE * r;
    Path(vec![
        Seg::Move(cx + r, cy),
        Seg::Curve([cx + r, cy + k, cx + k, cy + r, cx, cy + r]),
        Seg::Curve([cx - k, cy + r, cx - r, cy + k, cx - r, cy]),
        Seg::Curve([cx - r, cy - k, cx - k, cy - r, cx, cy - r]),
        Seg::Curve([cx + k, cy - r, cx + r, cy - k, cx + r, cy]),
        Seg::Close,
    ])
}

fn spot_x(c: Cell, spot: Spot) -> f64 {
    match spot {
        Spot::Left => 0.0,
        Spot::Right => c.width,
        _ => c.width / 2.0,
    }
}

fn spot_y(c: Cell, spot: Spot) -> f64 {
    match spot {
        Spot::Top => 0.0,
        Spot::Bottom => c.height,
        _ => c.height / 2.0,
    }
}

#[expect(
    clippy::many_single_char_names,
    reason = "the names are the parts of each box-drawing glyph, one term per stroke"
)]
fn circle(c: Cell, h: Spot, v: Spot, filled: bool) -> Shape {
    let x = spot_x(c, h);
    let y = spot_y(c, v);
    let r = 0.5 * c.width.min(c.height);
    if filled {
        Shape::Fill {
            path: circle_path(x, y, r),
            opacity: None,
            clip: true,
        }
    } else {
        Shape::Stroke {
            path: circle_path(x, y, r - c.thickness / 2.0),
            width: c.thickness,
            clip: true,
            inner: false,
        }
    }
}

fn edge_triangle(c: Cell, edge: Edge) -> Shape {
    let middle = js_round(c.height / 2.0);
    let center = js_round(c.width / 2.0);
    let (x0, y0, x1, y1) = match edge {
        Edge::Top => (c.width, 0.0, 0.0, 0.0),
        Edge::Left => (0.0, 0.0, 0.0, c.height),
        Edge::Bottom => (0.0, c.height, c.width, c.height),
        Edge::Right => (c.width, c.height, c.width, 0.0),
    };
    path_shape(polygon(&[(center, middle), (x0, y0), (x1, y1)]))
}

fn corner_triangle(c: Cell, corner: Corner, opacity: f64) -> Shape {
    let (w, h) = (c.width, c.height);
    let points: [(f64, f64); 3] = match corner {
        Corner::Tl => [(0.0, 0.0), (0.0, h), (w, 0.0)],
        Corner::Tr => [(0.0, 0.0), (w, h), (w, 0.0)],
        Corner::Bl => [(0.0, 0.0), (0.0, h), (w, h)],
        Corner::Br => [(0.0, h), (w, h), (w, 0.0)],
    };
    Shape::Fill {
        path: polygon(&points),
        opacity: Some(opacity),
        clip: false,
    }
}

fn corner_diagonals(c: Cell, corners: &[Corner]) -> Vec<Shape> {
    let cx = (c.width / 2.0).floor() + c.width.floor() % 2.0;
    let cy = (c.height / 2.0).floor() + c.height.floor() % 2.0;
    corners
        .iter()
        .map(|corner| {
            let (x0, y0, x1, y1) = match corner {
                Corner::Tl => (cx, 0.0, 0.0, cy),
                Corner::Tr => (cx, 0.0, c.width, cy),
                Corner::Bl => (cx, c.height, 0.0, cy),
                Corner::Br => (cx, c.height, c.width, cy),
            };
            line((x0, y0), (x1, y1), c.thickness, false)
        })
        .collect()
}

fn cell_diagonal(c: Cell, from: (Spot, Spot), to: (Spot, Spot)) -> Shape {
    line(
        (spot_x(c, from.0), spot_y(c, from.1)),
        (spot_x(c, to.0), spot_y(c, to.1)),
        c.thickness,
        false,
    )
}

fn checkerboard(c: Cell, parity: u32) -> Vec<Shape> {
    let width = c.width.floor();
    let height = c.height.floor();
    let ys = js_round(4.0 * (c.height / c.width));
    let mut out = Vec::new();
    for x in 0..4u32 {
        let fx = f64::from(x);
        for y in 0..whole(ys) {
            let fy = f64::from(y);
            if (x + y) % 2 == parity {
                out.push(solid(
                    (width * fx / 4.0).floor(),
                    (height * fy / ys).floor(),
                    (width * (fx + 1.0) / 4.0).floor(),
                    (height * (fy + 1.0) / ys).floor(),
                ));
            }
        }
    }
    out
}

fn diagonal_fill(c: Cell, falling: bool) -> Vec<Shape> {
    let count = 1.0f64.max((c.width.floor() / (2.0 * c.thickness)).floor());
    let stride = js_round(c.width / count);
    let mut out = Vec::new();
    let steps = i32::try_from(whole(count)).unwrap_or(0);
    for step in -steps..=steps {
        let near = f64::from(step) * stride;
        let far = c.width + near;
        out.push(if falling {
            line((near, 0.0), (far, c.height), c.thickness, true)
        } else {
            line((far, 0.0), (near, c.height), c.thickness, true)
        });
    }
    out
}

const EDGES: [Edge; 4] = [Edge::Left, Edge::Top, Edge::Right, Edge::Bottom];

const LINE_ENDS: [(Spot, Spot); 16] = [
    (Spot::Right, Spot::Middle),
    (Spot::Left, Spot::Bottom),
    (Spot::Right, Spot::Top),
    (Spot::Left, Spot::Middle),
    (Spot::Left, Spot::Top),
    (Spot::Right, Spot::Middle),
    (Spot::Left, Spot::Middle),
    (Spot::Right, Spot::Bottom),
    (Spot::Left, Spot::Top),
    (Spot::Center, Spot::Bottom),
    (Spot::Center, Spot::Top),
    (Spot::Right, Spot::Bottom),
    (Spot::Right, Spot::Top),
    (Spot::Center, Spot::Bottom),
    (Spot::Center, Spot::Top),
    (Spot::Left, Spot::Bottom),
];

fn legacy_blocks(cp: u32, c: Cell) -> Option<Vec<Shape>> {
    let medium = SHADE_MEDIUM;
    let whole = |opacity: f64| rect(0.0, 0.0, c.width, c.height, opacity);
    let edge_left = block(c, Align::Left, (ONE_EIGHTH, 1.0), 1.0);
    let edge_right = block(c, Align::Right, (ONE_EIGHTH, 1.0), 1.0);
    let edge_top = block(c, Align::Upper, (1.0, ONE_EIGHTH), 1.0);
    let edge_bottom = block(c, Align::Lower, (1.0, ONE_EIGHTH), 1.0);
    Some(match cp {
        0x1fb7c => vec![edge_left, edge_bottom],
        0x1fb7d => vec![edge_left, edge_top],
        0x1fb7e => vec![edge_right, edge_top],
        0x1fb7f => vec![edge_right, edge_bottom],
        0x1fb80 => vec![edge_top, edge_bottom],
        0x1fb81 => [0usize, 2, 4, 7]
            .iter()
            .map(|&n| {
                fill(
                    c,
                    (0.0, 1.0),
                    (eighth(n), EIGHTHS.get(n + 1).copied().unwrap_or(1.0)),
                )
            })
            .collect(),
        0x1fb8c => vec![block(c, Align::Left, (0.5, 1.0), medium)],
        0x1fb8d => vec![block(c, Align::Right, (0.5, 1.0), medium)],
        0x1fb8e => vec![block(c, Align::Upper, (1.0, 0.5), medium)],
        0x1fb8f => vec![block(c, Align::Lower, (1.0, 0.5), medium)],
        0x1fb90 => vec![whole(medium)],
        0x1fb91 => vec![whole(medium), block(c, Align::Upper, (1.0, 0.5), 1.0)],
        0x1fb92 => vec![whole(medium), block(c, Align::Lower, (1.0, 0.5), 1.0)],
        0x1fb93 => Vec::new(),
        0x1fb94 => vec![whole(medium), block(c, Align::Right, (0.5, 1.0), 1.0)],
        0x1fb95 => checkerboard(c, 0),
        0x1fb96 => checkerboard(c, 1),
        0x1fb97 => {
            let h = c.height.floor();
            vec![
                solid(0.0, (h / 4.0).floor(), c.width, (2.0 * h / 4.0).floor()),
                solid(0.0, (3.0 * h / 4.0).floor(), c.width, h),
            ]
        }
        0x1fb98 => diagonal_fill(c, true),
        0x1fb99 => diagonal_fill(c, false),
        0x1fb9a => vec![edge_triangle(c, Edge::Top), edge_triangle(c, Edge::Bottom)],
        0x1fb9b => vec![edge_triangle(c, Edge::Left), edge_triangle(c, Edge::Right)],
        0x1fb9c => vec![corner_triangle(c, Corner::Tl, medium)],
        0x1fb9d => vec![corner_triangle(c, Corner::Tr, medium)],
        0x1fb9e => vec![corner_triangle(c, Corner::Br, medium)],
        0x1fb9f => vec![corner_triangle(c, Corner::Bl, medium)],
        0x1fbaf => lines(c, "2121"),
        0x1fbbd => vec![Shape::Inverted(vec![diagonal(c, true), diagonal(c, false)])],
        0x1fbbe => vec![Shape::Inverted(corner_diagonals(c, &[Corner::Br]))],
        0x1fbbf => vec![Shape::Inverted(corner_diagonals(
            c,
            &[Corner::Tl, Corner::Tr, Corner::Bl, Corner::Br],
        ))],
        0x1fbce => vec![block(c, Align::Left, (TWO_THIRDS, 1.0), 1.0)],
        0x1fbcf => vec![block(c, Align::Left, (ONE_THIRD, 1.0), 1.0)],
        _ => return None,
    })
}

const fn diagonal_set(cp: u32) -> Option<&'static [Corner]> {
    use Corner::{Bl, Br, Tl, Tr};
    Some(match cp {
        0x1fba0 => &[Tl],
        0x1fba1 => &[Tr],
        0x1fba2 => &[Bl],
        0x1fba3 => &[Br],
        0x1fba4 => &[Tl, Bl],
        0x1fba5 => &[Tr, Br],
        0x1fba6 => &[Bl, Br],
        0x1fba7 => &[Tl, Tr],
        0x1fba8 => &[Tl, Br],
        0x1fba9 => &[Tr, Bl],
        0x1fbaa => &[Tr, Bl, Br],
        0x1fbab => &[Tl, Bl, Br],
        0x1fbac => &[Tl, Tr, Br],
        0x1fbad => &[Tl, Tr, Bl],
        0x1fbae => &[Tl, Tr, Bl, Br],
        _ => return None,
    })
}

type Ends = ((Spot, Spot), (Spot, Spot));

const fn diagonal_pairs(cp: u32) -> Option<[Ends; 2]> {
    use Spot::{Bottom, Center, Left, Middle, Right, Top};
    let middle = (Center, Middle);
    Some(match cp {
        0x1fbd8 => [((Left, Top), middle), (middle, (Right, Top))],
        0x1fbd9 => [((Right, Top), middle), (middle, (Right, Bottom))],
        0x1fbda => [((Left, Bottom), middle), (middle, (Right, Bottom))],
        0x1fbdb => [((Left, Top), middle), (middle, (Left, Bottom))],
        0x1fbdc => [
            ((Left, Top), (Center, Bottom)),
            ((Center, Bottom), (Right, Top)),
        ],
        0x1fbdd => [
            ((Right, Top), (Left, Middle)),
            ((Left, Middle), (Right, Bottom)),
        ],
        0x1fbde => [
            ((Left, Bottom), (Center, Top)),
            ((Center, Top), (Right, Bottom)),
        ],
        0x1fbdf => [
            ((Left, Top), (Right, Middle)),
            ((Right, Middle), (Left, Bottom)),
        ],
        _ => return None,
    })
}

const fn circle_spot(cp: u32) -> Option<(Spot, Spot, bool)> {
    use Spot::{Bottom, Center, Left, Middle, Right, Top};
    Some(match cp {
        0x1fbe0 => (Center, Top, false),
        0x1fbe1 => (Right, Middle, false),
        0x1fbe2 => (Center, Bottom, false),
        0x1fbe3 => (Left, Middle, false),
        0x1fbe8 => (Center, Top, true),
        0x1fbe9 => (Right, Middle, true),
        0x1fbea => (Center, Bottom, true),
        0x1fbeb => (Left, Middle, true),
        0x1fbec => (Right, Top, true),
        0x1fbed => (Left, Bottom, true),
        0x1fbee => (Right, Bottom, true),
        0x1fbef => (Left, Top, true),
        _ => return None,
    })
}

fn legacy_more(cp: u32, c: Cell) -> Option<Vec<Shape>> {
    if (0x1fb68..=0x1fb6b).contains(&cp) {
        let edge = EDGES
            .get(index_from(cp, 0x1fb68))
            .copied()
            .unwrap_or(Edge::Left);
        return Some(vec![Shape::Inverted(vec![edge_triangle(c, edge)])]);
    }
    if (0x1fb6c..=0x1fb6f).contains(&cp) {
        let edge = EDGES
            .get(index_from(cp, 0x1fb6c))
            .copied()
            .unwrap_or(Edge::Left);
        return Some(vec![edge_triangle(c, edge)]);
    }
    if let Some(shapes) = legacy_blocks(cp, c) {
        return Some(shapes);
    }
    if let Some(set) = diagonal_set(cp) {
        return Some(corner_diagonals(c, set));
    }
    if (0x1fbd0..=0x1fbd7).contains(&cp) {
        let i = index_from(cp, 0x1fbd0) * 2;
        let fallback = (Spot::Left, Spot::Top);
        return Some(vec![cell_diagonal(
            c,
            LINE_ENDS.get(i).copied().unwrap_or(fallback),
            LINE_ENDS.get(i + 1).copied().unwrap_or(fallback),
        )]);
    }
    if let Some(pairs) = diagonal_pairs(cp) {
        return Some(
            pairs
                .iter()
                .map(|&(from, to)| cell_diagonal(c, from, to))
                .collect(),
        );
    }
    if let Some((h, v, filled)) = circle_spot(cp) {
        return Some(vec![circle(c, h, v, filled)]);
    }
    let half = match cp {
        0x1fbe4 => Align::Upper,
        0x1fbe5 => Align::Lower,
        0x1fbe6 => Align::Left,
        0x1fbe7 => Align::Right,
        _ => return None,
    };
    Some(vec![block(c, half, (0.5, 0.5), 1.0)])
}

#[expect(
    clippy::many_single_char_names,
    reason = "the names are the parts of each box-drawing glyph, one term per stroke"
)]
fn separated(c: Cell, rows: u32, bits: u32) -> Vec<Shape> {
    let width = c.width.floor();
    let height = c.height.floor();
    let rows_f = f64::from(rows);
    let gap = 1.0f64.max((width / 12.0).floor());
    let mid_x = gap * 2.0 + width % 2.0;
    let w = ((width - gap * 2.0 - mid_x) / 2.0).floor();
    let mid_y = if rows == 2 {
        gap * 2.0 + height % 2.0
    } else {
        gap * 2.0 + ((height % 3.0) / 2.0).floor()
    };
    let h = ((height - gap * 2.0 - mid_y * (rows_f - 1.0)) / rows_f).floor();
    let middle_h = height - gap * 2.0 - mid_y * (rows_f - 1.0) - h * (rows_f - 1.0);
    let mut out = Vec::new();
    let mut y = gap;
    for row in 0..rows {
        let row_h = if rows == 3 && row == 1 { middle_h } else { h };
        for col in 0..2u32 {
            if bits & (1 << (row * 2 + col)) == 0 {
                continue;
            }
            let x = gap + f64::from(col) * (w + mid_x);
            out.push(solid(x, y, x + w, y + row_h));
        }
        y += row_h + mid_y;
    }
    out
}

#[derive(Debug, Clone, Copy)]
struct Piece {
    x: f64,
    y: f64,
    wf: f64,
    hf: f64,
    corner: Corner,
}

#[expect(
    clippy::many_single_char_names,
    reason = "the names are the parts of each box-drawing glyph, one term per stroke"
)]
fn circle_piece(c: Cell, p: Piece) -> Shape {
    let w = c.width * p.wf;
    let h = c.height * p.hf;
    let xp = c.width * p.x;
    let yp = c.height * p.y;
    let cw = BEZIER_CIRCLE * w;
    let ch = BEZIER_CIRCLE * h;
    let t = c.thickness / 2.0;
    let v: [f64; 8] = match p.corner {
        Corner::Tl => [
            w - xp,
            t - yp,
            w - cw - xp,
            t - yp,
            t - xp,
            h - ch - yp,
            t - xp,
            h - yp,
        ],
        Corner::Tr => [
            w - xp,
            t - yp,
            w + cw - xp,
            t - yp,
            w * 2.0 - t - xp,
            h - ch - yp,
            w * 2.0 - t - xp,
            h - yp,
        ],
        Corner::Bl => [
            t - xp,
            h - yp,
            t - xp,
            h + ch - yp,
            w - cw - xp,
            h * 2.0 - t - yp,
            w - xp,
            h * 2.0 - t - yp,
        ],
        Corner::Br => [
            w * 2.0 - t - xp,
            h - yp,
            w * 2.0 - t - xp,
            h + ch - yp,
            w + cw - xp,
            h * 2.0 - t - yp,
            w - xp,
            h * 2.0 - t - yp,
        ],
    };
    Shape::Stroke {
        path: Path(vec![
            Seg::Move(v[0], v[1]),
            Seg::Curve([v[2], v[3], v[4], v[5], v[6], v[7]]),
        ]),
        width: c.thickness,
        clip: true,
        inner: false,
    }
}

fn piece_of(cp: u32) -> Option<Piece> {
    let at = |x: f64, y: f64, wf: f64, hf: f64, corner: Corner| Piece {
        x,
        y,
        wf,
        hf,
        corner,
    };
    Some(match cp {
        0x1cc30 => at(0.0, 0.0, 2.0, 2.0, Corner::Tl),
        0x1cc31 => at(1.0, 0.0, 2.0, 2.0, Corner::Tl),
        0x1cc32 => at(2.0, 0.0, 2.0, 2.0, Corner::Tr),
        0x1cc33 => at(3.0, 0.0, 2.0, 2.0, Corner::Tr),
        0x1cc34 => at(0.0, 1.0, 2.0, 2.0, Corner::Tl),
        0x1cc35 => at(0.0, 0.0, 1.0, 1.0, Corner::Tl),
        0x1cc36 => at(1.0, 0.0, 1.0, 1.0, Corner::Tr),
        0x1cc37 => at(3.0, 1.0, 2.0, 2.0, Corner::Tr),
        0x1cc38 => at(0.0, 2.0, 2.0, 2.0, Corner::Bl),
        0x1cc39 => at(0.0, 1.0, 1.0, 1.0, Corner::Bl),
        0x1cc3a => at(1.0, 1.0, 1.0, 1.0, Corner::Br),
        0x1cc3b => at(3.0, 2.0, 2.0, 2.0, Corner::Br),
        0x1cc3c => at(0.0, 3.0, 2.0, 2.0, Corner::Bl),
        0x1cc3d => at(1.0, 3.0, 2.0, 2.0, Corner::Bl),
        0x1cc3e => at(2.0, 3.0, 2.0, 2.0, Corner::Br),
        0x1cc3f => at(3.0, 3.0, 2.0, 2.0, Corner::Br),
        _ => return None,
    })
}

const fn sixteenth(cp: u32) -> Option<[usize; 4]> {
    Some(match cp {
        0x1cea0 => [2, 4, 3, 4],
        0x1cea1 => [1, 4, 3, 4],
        0x1cea2 => [0, 3, 3, 4],
        0x1cea3 => [0, 2, 3, 4],
        0x1cea4 => [0, 1, 2, 4],
        0x1cea5 => [0, 1, 1, 4],
        0x1cea6 => [0, 1, 0, 3],
        0x1cea7 => [0, 1, 0, 2],
        0x1cea8 => [0, 2, 0, 1],
        0x1cea9 => [0, 3, 0, 1],
        0x1ceaa => [1, 4, 0, 1],
        0x1ceab => [2, 4, 0, 1],
        0x1ceac => [3, 4, 0, 2],
        0x1cead => [3, 4, 0, 3],
        0x1ceae => [3, 4, 1, 4],
        0x1ceaf => [3, 4, 2, 4],
        _ => return None,
    })
}

fn quarter(i: usize) -> f64 {
    QUARTERS.get(i).copied().unwrap_or(0.0)
}

#[expect(
    clippy::many_single_char_names,
    reason = "the names are the parts of each box-drawing glyph, one term per stroke"
)]
fn supplement(cp: u32, c: Cell) -> Option<Vec<Shape>> {
    let t = c.thickness;
    let w = c.width.floor();
    let h = c.height.floor();
    if (0x1cc21..=0x1cc2f).contains(&cp) {
        return Some(separated(c, 2, cp - 0x1cc20));
    }
    if (0x1ce51..=0x1ce8f).contains(&cp) {
        return Some(separated(c, 3, cp - 0x1ce50));
    }
    if let Some(piece) = piece_of(cp) {
        return Some(vec![circle_piece(c, piece)]);
    }
    if (0x1ce90..=0x1ce9f).contains(&cp) {
        let i = index_from(cp, 0x1ce90);
        let (col, row) = (i % 4, i / 4);
        return Some(vec![fill(
            c,
            (quarter(col), QUARTERS.get(col + 1).copied().unwrap_or(1.0)),
            (quarter(row), QUARTERS.get(row + 1).copied().unwrap_or(1.0)),
        )]);
    }
    if let Some([x0, x1, y0, y1]) = sixteenth(cp) {
        return Some(vec![fill(
            c,
            (quarter(x0), quarter(x1)),
            (quarter(y0), quarter(y1)),
        )]);
    }
    let half = (h / 2.0).floor();
    let half_w = (w / 2.0).floor();
    let with = |mut base: Vec<Shape>, extra: Shape| {
        base.push(extra);
        base
    };
    let piece = |corner: Corner, x: f64| Piece {
        x,
        y: 0.0,
        wf: 1.0,
        hf: 0.5,
        corner,
    };
    Some(match cp {
        0x1cc1b => with(lines(c, "0101"), solid(w - t, 0.0, w, half)),
        0x1cc1c => with(lines(c, "0101"), solid(w - t, half, w, h)),
        0x1cc1d => vec![solid(0.0, 0.0, w, t), solid(0.0, 0.0, t, half)],
        0x1cc1e => vec![solid(0.0, h - t, w, h), solid(0.0, half, t, h)],
        0x1ce00 => vec![
            circle(c, Spot::Left, Spot::Middle, false),
            circle(c, Spot::Right, Spot::Middle, false),
        ],
        0x1ce01 => vec![
            circle(c, Spot::Center, Spot::Top, false),
            circle(c, Spot::Center, Spot::Bottom, false),
        ],
        0x1ce0b => vec![
            circle_piece(c, piece(Corner::Tl, 0.0)),
            circle_piece(c, piece(Corner::Bl, 0.0)),
        ],
        0x1ce0c => vec![
            circle_piece(c, piece(Corner::Tr, 1.0)),
            circle_piece(c, piece(Corner::Br, 1.0)),
        ],
        0x1ce16 => with(lines(c, "1010"), solid(half_w, 0.0, w, t)),
        0x1ce17 => with(lines(c, "1010"), solid(half_w, h - t, w, h)),
        0x1ce18 => with(lines(c, "1010"), solid(0.0, 0.0, half_w, t)),
        0x1ce19 => with(lines(c, "1010"), solid(0.0, h - t, half_w, h)),
        _ => return None,
    })
}

fn fading_line(c: Cell, to: Edge) -> Vec<Shape> {
    let t = c.thickness;
    let width = c.width.floor();
    let height = c.height.floor();
    let top = ((height - t) / 2.0).floor();
    let left = ((width - t) / 2.0).floor();
    let vertical = matches!(to, Edge::Top | Edge::Bottom);
    let length = if vertical { height } else { width };
    let rising = matches!(to, Edge::Top | Edge::Left);
    let mut out = Vec::new();
    for step in 0..whole(length) {
        let i = f64::from(step);
        let alpha = js_round(if rising {
            FULL_ALPHA * i / length
        } else {
            FULL_ALPHA - FULL_ALPHA * i / length
        });
        if alpha > 0.0 {
            let opacity = alpha / FULL_ALPHA;
            out.push(if vertical {
                rect(left, i, left + t, i + 1.0, opacity)
            } else {
                rect(i, top, i + 1.0, top + t, opacity)
            });
        }
    }
    out
}

fn branch_node(c: Cell, sides: &str, filled: bool) -> Vec<Shape> {
    let t = c.thickness;
    let width = c.width.floor();
    let height = c.height.floor();
    let h_top = ((height - t) / 2.0).floor();
    let h_bottom = h_top + t;
    let v_left = ((width - t) / 2.0).floor();
    let v_right = v_left + t;
    let cx = v_left + t / 2.0;
    let cy = h_top + t / 2.0;
    let r = cx.min(cy).min(c.width - cx).min(c.height - cy);
    let mut out = Vec::new();
    if sides.contains('u') {
        out.push(solid(v_left, 0.0, v_right, (cy - r + t / 2.0).ceil()));
    }
    if sides.contains('r') {
        out.push(solid((cx + r - t / 2.0).floor(), h_top, width, h_bottom));
    }
    if sides.contains('d') {
        out.push(solid(v_left, (cy + r - t / 2.0).floor(), v_right, height));
    }
    if sides.contains('l') {
        out.push(solid(0.0, h_top, (cx - r + t / 2.0).ceil(), h_bottom));
    }
    out.push(if filled {
        path_shape(circle_path(cx, cy, r))
    } else {
        stroke(circle_path(cx, cy, r - t / 2.0), t)
    });
    out
}

const fn node_sides(cp: u32) -> Option<&'static str> {
    Some(match cp {
        0xf5ee => "",
        0xf5f0 => "r",
        0xf5f2 => "l",
        0xf5f4 => "lr",
        0xf5f6 => "d",
        0xf5f8 => "u",
        0xf5fa => "ud",
        0xf5fc => "rd",
        0xf5fe => "ld",
        0xf600 => "ur",
        0xf602 => "ul",
        0xf604 => "udr",
        0xf606 => "udl",
        0xf608 => "dlr",
        0xf60a => "ulr",
        0xf60c => "udlr",
        _ => return None,
    })
}

fn branch(cp: u32, c: Cell) -> Option<Vec<Shape>> {
    use Corner::{Bl, Br, Tl, Tr};
    let across = || lines(c, "0101");
    let along = || lines(c, "1010");
    let arcs = |corners: &[Corner]| {
        corners
            .iter()
            .map(|&corner| arc(c, corner))
            .collect::<Vec<_>>()
    };
    let joined = |mut a: Vec<Shape>, b: Vec<Shape>| {
        a.extend(b);
        a
    };
    let shapes = match cp {
        0xf5d0 => Some(across()),
        0xf5d1 => Some(along()),
        0xf5d2 => Some(fading_line(c, Edge::Right)),
        0xf5d3 => Some(fading_line(c, Edge::Left)),
        0xf5d4 => Some(fading_line(c, Edge::Bottom)),
        0xf5d5 => Some(fading_line(c, Edge::Top)),
        0xf5d6 => Some(arcs(&[Br])),
        0xf5d7 => Some(arcs(&[Bl])),
        0xf5d8 => Some(arcs(&[Tr])),
        0xf5d9 => Some(arcs(&[Tl])),
        0xf5da => Some(joined(along(), arcs(&[Tr]))),
        0xf5db => Some(joined(along(), arcs(&[Br]))),
        0xf5dc => Some(arcs(&[Tr, Br])),
        0xf5dd => Some(joined(along(), arcs(&[Tl]))),
        0xf5de => Some(joined(along(), arcs(&[Bl]))),
        0xf5df => Some(arcs(&[Tl, Bl])),
        0xf5e0 => Some(joined(arcs(&[Bl]), across())),
        0xf5e1 => Some(joined(arcs(&[Br]), across())),
        0xf5e2 => Some(arcs(&[Br, Bl])),
        0xf5e3 => Some(joined(arcs(&[Tl]), across())),
        0xf5e4 => Some(joined(arcs(&[Tr]), across())),
        0xf5e5 => Some(arcs(&[Tr, Tl])),
        0xf5e6 => Some(joined(along(), arcs(&[Tl, Tr]))),
        0xf5e7 => Some(joined(along(), arcs(&[Bl, Br]))),
        0xf5e8 => Some(joined(across(), arcs(&[Bl, Tl]))),
        0xf5e9 => Some(joined(across(), arcs(&[Tr, Br]))),
        0xf5ea => Some(joined(along(), arcs(&[Tl, Br]))),
        0xf5eb => Some(joined(along(), arcs(&[Tr, Bl]))),
        0xf5ec => Some(joined(across(), arcs(&[Tl, Br]))),
        0xf5ed => Some(joined(across(), arcs(&[Tr, Bl]))),
        _ => None,
    };
    if shapes.is_some() {
        return shapes;
    }
    let filled = node_sides(cp).is_some();
    let sides = node_sides(cp).or_else(|| node_sides(cp - 1))?;
    Some(branch_node(c, sides, filled))
}

fn powerline_more(cp: u32, c: Cell) -> Option<Vec<Shape>> {
    if cp != 0xe0d2 && cp != 0xe0d4 {
        return None;
    }
    let (w, h, t) = (c.width, c.height, c.thickness);
    let shapes = [
        path_shape(polygon(&[
            (0.0, 0.0),
            (w, 0.0),
            (w / 2.0, h / 2.0 - t / 2.0),
            (0.0, h / 2.0 - t / 2.0),
        ])),
        path_shape(polygon(&[
            (0.0, h),
            (w, h),
            (w / 2.0, h / 2.0 + t / 2.0),
            (0.0, h / 2.0 + t / 2.0),
        ])),
    ];
    Some(if cp == 0xe0d4 {
        shapes.iter().map(|s| flip(s, w)).collect()
    } else {
        shapes.to_vec()
    })
}

pub const fn is_sprite(cp: u32) -> bool {
    matches!(
        cp,
        0x2500..=0x259f
            | 0x2800..=0x28ff
            | 0xe0b0..=0xe0bf
            | 0x25e2..=0x25e5
            | 0x25f8..=0x25fa
            | 0x25ff
            | 0xe0d2
            | 0xe0d4
            | 0xf5d0..=0xf60d
            | 0x1fb00..=0x1fbaf
            | 0x1fbbd..=0x1fbbf
            | 0x1fbce..=0x1fbef
            | 0x1cc1b..=0x1cc1e
            | 0x1cc21..=0x1cc3f
            | 0x1cd00..=0x1cde5
            | 0x1ce00
            | 0x1ce01
            | 0x1ce0b
            | 0x1ce0c
            | 0x1ce16..=0x1ce19
            | 0x1ce51..=0x1ceaf
    )
}

pub fn sprite_for(cp: u32, cell: Cell) -> Option<Vec<Shape>> {
    if !is_sprite(cp) {
        return None;
    }
    if (0x2800..=0x28ff).contains(&cp) {
        return Some(braille(cp, cell));
    }
    box_drawing(cp, cell)
        .or_else(|| blocks(cp, cell))
        .or_else(|| powerline(cp, cell))
        .or_else(|| powerline_more(cp, cell))
        .or_else(|| geometric(cp, cell))
        .or_else(|| legacy(cp, cell))
        .or_else(|| legacy_more(cp, cell))
        .or_else(|| supplement(cp, cell))
        .or_else(|| branch(cp, cell))
}

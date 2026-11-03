export type SpriteShape =
  | { kind: "rect"; x: number; y: number; width: number; height: number; opacity: number }
  | { kind: "fill"; d: string }
  | { kind: "stroke"; d: string; width: number };

export type SpriteCell = { width: number; height: number; thickness: number };

const LIGHT = 1;
const HEAVY = 2;
const DOUBLE = 3;
const SHADES = { light: 0.25, medium: 0.5, dark: 0.75 } as const;
const ARC_TENSION = 0.25;
const BEZIER_CIRCLE = ((Math.SQRT2 - 1) * 4) / 3;
const UNDERLINE_EM = 0.05;

export function strokeFor(fontSize: number): number {
  return Math.max(1, Math.ceil(fontSize * UNDERLINE_EM));
}

const BOX_LINES = new Map<number, string>([
  [0x2500, "0101"],
  [0x2501, "0202"],
  [0x2502, "1010"],
  [0x2503, "2020"],
  [0x250c, "0110"],
  [0x250d, "0210"],
  [0x250e, "0120"],
  [0x250f, "0220"],
  [0x2510, "0011"],
  [0x2511, "0012"],
  [0x2512, "0021"],
  [0x2513, "0022"],
  [0x2514, "1100"],
  [0x2515, "1200"],
  [0x2516, "2100"],
  [0x2517, "2200"],
  [0x2518, "1001"],
  [0x2519, "1002"],
  [0x251a, "2001"],
  [0x251b, "2002"],
  [0x251c, "1110"],
  [0x251d, "1210"],
  [0x251e, "2110"],
  [0x251f, "1120"],
  [0x2520, "2120"],
  [0x2521, "2210"],
  [0x2522, "1220"],
  [0x2523, "2220"],
  [0x2524, "1011"],
  [0x2525, "1012"],
  [0x2526, "2011"],
  [0x2527, "1021"],
  [0x2528, "2021"],
  [0x2529, "2012"],
  [0x252a, "1022"],
  [0x252b, "2022"],
  [0x252c, "0111"],
  [0x252d, "0112"],
  [0x252e, "0211"],
  [0x252f, "0212"],
  [0x2530, "0121"],
  [0x2531, "0122"],
  [0x2532, "0221"],
  [0x2533, "0222"],
  [0x2534, "1101"],
  [0x2535, "1102"],
  [0x2536, "1201"],
  [0x2537, "1202"],
  [0x2538, "2101"],
  [0x2539, "2102"],
  [0x253a, "2201"],
  [0x253b, "2202"],
  [0x253c, "1111"],
  [0x253d, "1112"],
  [0x253e, "1211"],
  [0x253f, "1212"],
  [0x2540, "2111"],
  [0x2541, "1121"],
  [0x2542, "2121"],
  [0x2543, "2112"],
  [0x2544, "2211"],
  [0x2545, "1122"],
  [0x2546, "1221"],
  [0x2547, "2212"],
  [0x2548, "1222"],
  [0x2549, "2122"],
  [0x254a, "2221"],
  [0x254b, "2222"],
  [0x2550, "0303"],
  [0x2551, "3030"],
  [0x2552, "0310"],
  [0x2553, "0130"],
  [0x2554, "0330"],
  [0x2555, "0013"],
  [0x2556, "0031"],
  [0x2557, "0033"],
  [0x2558, "1300"],
  [0x2559, "3100"],
  [0x255a, "3300"],
  [0x255b, "1003"],
  [0x255c, "3001"],
  [0x255d, "3003"],
  [0x255e, "1310"],
  [0x255f, "3130"],
  [0x2560, "3330"],
  [0x2561, "1013"],
  [0x2562, "3031"],
  [0x2563, "3033"],
  [0x2564, "0313"],
  [0x2565, "0131"],
  [0x2566, "0333"],
  [0x2567, "1303"],
  [0x2568, "3101"],
  [0x2569, "3303"],
  [0x256a, "1313"],
  [0x256b, "3131"],
  [0x256c, "3333"],
  [0x2574, "0001"],
  [0x2575, "1000"],
  [0x2576, "0100"],
  [0x2577, "0010"],
  [0x2578, "0002"],
  [0x2579, "2000"],
  [0x257a, "0200"],
  [0x257b, "0020"],
  [0x257c, "0201"],
  [0x257d, "1020"],
  [0x257e, "0102"],
  [0x257f, "2010"],
]);

const MOSAICS = new Map<number, string>([
  [0x1fb3c, "...|...|#..|##."],
  [0x1fb3d, "...|...|#\\.|###"],
  [0x1fb3e, "...|#..|#\\.|##."],
  [0x1fb3f, "...|#..|##.|###"],
  [0x1fb40, "#..|#..|##.|##."],
  [0x1fb41, "/##|###|###|###"],
  [0x1fb42, "./#|###|###|###"],
  [0x1fb43, ".##|.##|###|###"],
  [0x1fb44, "..#|.##|###|###"],
  [0x1fb45, ".##|.##|.##|###"],
  [0x1fb46, "...|./#|###|###"],
  [0x1fb47, "...|...|..#|.##"],
  [0x1fb48, "...|...|./#|###"],
  [0x1fb49, "...|..#|./#|.##"],
  [0x1fb4a, "...|..#|.##|###"],
  [0x1fb4b, "..#|..#|.##|.##"],
  [0x1fb4c, "##\\|###|###|###"],
  [0x1fb4d, "#\\.|###|###|###"],
  [0x1fb4e, "##.|##.|###|###"],
  [0x1fb4f, "#..|##.|###|###"],
  [0x1fb50, "##.|##.|##.|###"],
  [0x1fb51, "...|#\\.|###|###"],
  [0x1fb52, "###|###|###|\\##"],
  [0x1fb53, "###|###|###|.\\#"],
  [0x1fb54, "###|###|.##|.##"],
  [0x1fb55, "###|###|.##|..#"],
  [0x1fb56, "###|.##|.##|.##"],
  [0x1fb57, "##.|#..|...|..."],
  [0x1fb58, "###|#/.|...|..."],
  [0x1fb59, "##.|#/.|#..|..."],
  [0x1fb5a, "###|##.|#..|..."],
  [0x1fb5b, "##.|##.|#..|#.."],
  [0x1fb5c, "###|###|#/.|..."],
  [0x1fb5d, "###|###|###|##/"],
  [0x1fb5e, "###|###|###|#/."],
  [0x1fb5f, "###|###|##.|##."],
  [0x1fb60, "###|###|##.|#.."],
  [0x1fb61, "###|##.|##.|##."],
  [0x1fb62, ".##|..#|...|..."],
  [0x1fb63, "###|.\\#|...|..."],
  [0x1fb64, ".##|.\\#|..#|..."],
  [0x1fb65, "###|.##|..#|..."],
  [0x1fb66, ".##|.##|..#|..#"],
  [0x1fb67, "###|###|.\\#|..."],
]);

const OCTANTS = [
  4, 6, 7, 8, 9, 11, 12, 13, 14, 16, 17, 18, 19, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33,
  34, 35, 36, 37, 38, 39, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58,
  59, 60, 61, 62, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 81, 82, 83, 84, 86,
  87, 88, 89, 91, 92, 93, 94, 96, 97, 98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110,
  111, 112, 113, 114, 115, 116, 117, 118, 119, 120, 121, 122, 123, 124, 125, 126, 127, 129, 130,
  131, 132, 133, 134, 135, 136, 137, 138, 139, 140, 141, 142, 143, 144, 145, 146, 147, 148, 149,
  150, 151, 152, 153, 154, 155, 156, 157, 158, 159, 161, 162, 163, 164, 166, 167, 168, 169, 171,
  172, 173, 174, 176, 177, 178, 179, 180, 181, 182, 183, 184, 185, 186, 187, 188, 189, 190, 191,
  193, 194, 195, 196, 197, 198, 199, 200, 201, 202, 203, 204, 205, 206, 207, 208, 209, 210, 211,
  212, 213, 214, 215, 216, 217, 218, 219, 220, 221, 222, 223, 224, 225, 226, 227, 228, 229, 230,
  231, 232, 233, 234, 235, 236, 237, 238, 239, 241, 242, 243, 244, 246, 247, 248, 249, 251, 253,
  254,
];

function rect(x0: number, y0: number, x1: number, y1: number, opacity = 1): SpriteShape {
  return {
    kind: "rect",
    x: x0,
    y: y0,
    width: Math.max(0, x1 - x0),
    height: Math.max(0, y1 - y0),
    opacity,
  };
}

function minAt(fraction: number, size: number): number {
  return size - Math.round((1 - fraction) * size);
}

function maxAt(fraction: number, size: number): number {
  return Math.round(fraction * size);
}

function fill(c: SpriteCell, x0: number, x1: number, y0: number, y1: number): SpriteShape {
  return rect(minAt(x0, c.width), minAt(y0, c.height), maxAt(x1, c.width), maxAt(y1, c.height));
}

function heavy(c: SpriteCell): number {
  return c.thickness * 2;
}

function lines(c: SpriteCell, code: string): SpriteShape[] {
  const [up, right, down, left] = [...code].map(Number);
  const light = c.thickness;
  const strong = heavy(c);
  const hLightTop = Math.floor((c.height - light) / 2);
  const hLightBottom = hLightTop + light;
  const hHeavyTop = Math.floor((c.height - strong) / 2);
  const hHeavyBottom = hHeavyTop + strong;
  const hDoubleTop = hLightTop - light;
  const hDoubleBottom = hLightBottom + light;
  const vLightLeft = Math.floor((c.width - light) / 2);
  const vLightRight = vLightLeft + light;
  const vHeavyLeft = Math.floor((c.width - strong) / 2);
  const vHeavyRight = vHeavyLeft + strong;
  const vDoubleLeft = vLightLeft - light;
  const vDoubleRight = vLightRight + light;
  function anyHeavy(a: number, b: number) {
    return a === HEAVY || b === HEAVY;
  }
  function anyDouble(a: number, b: number) {
    return a === DOUBLE || b === DOUBLE;
  }
  const upBottom = anyHeavy(left, right)
    ? hHeavyBottom
    : left !== right || down === up
      ? anyDouble(left, right)
        ? hDoubleBottom
        : hLightBottom
      : left === 0 && right === 0
        ? hLightBottom
        : hLightTop;
  const downTop = anyHeavy(left, right)
    ? hHeavyTop
    : left !== right || up === down
      ? anyDouble(left, right)
        ? hDoubleTop
        : hLightTop
      : left === 0 && right === 0
        ? hLightTop
        : hLightBottom;
  const leftRight = anyHeavy(up, down)
    ? vHeavyRight
    : up !== down || left === right
      ? anyDouble(up, down)
        ? vDoubleRight
        : vLightRight
      : up === 0 && down === 0
        ? vLightRight
        : vLightLeft;
  const rightLeft = anyHeavy(up, down)
    ? vHeavyLeft
    : up !== down || right === left
      ? anyDouble(up, down)
        ? vDoubleLeft
        : vLightLeft
      : up === 0 && down === 0
        ? vLightLeft
        : vLightRight;
  const out: SpriteShape[] = [];
  if (up === LIGHT) out.push(rect(vLightLeft, 0, vLightRight, upBottom));
  if (up === HEAVY) out.push(rect(vHeavyLeft, 0, vHeavyRight, upBottom));
  if (up === DOUBLE) {
    out.push(rect(vDoubleLeft, 0, vLightLeft, left === DOUBLE ? hLightTop : upBottom));
    out.push(rect(vLightRight, 0, vDoubleRight, right === DOUBLE ? hLightTop : upBottom));
  }
  if (right === LIGHT) out.push(rect(rightLeft, hLightTop, c.width, hLightBottom));
  if (right === HEAVY) out.push(rect(rightLeft, hHeavyTop, c.width, hHeavyBottom));
  if (right === DOUBLE) {
    out.push(rect(up === DOUBLE ? vLightRight : rightLeft, hDoubleTop, c.width, hLightTop));
    out.push(rect(down === DOUBLE ? vLightRight : rightLeft, hLightBottom, c.width, hDoubleBottom));
  }
  if (down === LIGHT) out.push(rect(vLightLeft, downTop, vLightRight, c.height));
  if (down === HEAVY) out.push(rect(vHeavyLeft, downTop, vHeavyRight, c.height));
  if (down === DOUBLE) {
    out.push(rect(vDoubleLeft, left === DOUBLE ? hLightBottom : downTop, vLightLeft, c.height));
    out.push(rect(vLightRight, right === DOUBLE ? hLightBottom : downTop, vDoubleRight, c.height));
  }
  if (left === LIGHT) out.push(rect(0, hLightTop, leftRight, hLightBottom));
  if (left === HEAVY) out.push(rect(0, hHeavyTop, leftRight, hHeavyBottom));
  if (left === DOUBLE) {
    out.push(rect(0, hDoubleTop, up === DOUBLE ? vLightLeft : leftRight, hLightTop));
    out.push(rect(0, hLightBottom, down === DOUBLE ? vLightLeft : leftRight, hDoubleBottom));
  }
  return out;
}

function dashes(
  c: SpriteCell,
  count: number,
  thick: number,
  gap: number,
  across: boolean,
): SpriteShape[] {
  const length = across ? c.width : c.height;
  if (length < count * 2) {
    return across
      ? [
          rect(
            0,
            Math.floor((c.height - c.thickness) / 2),
            c.width,
            Math.floor((c.height - c.thickness) / 2) + c.thickness,
          ),
        ]
      : [
          rect(
            Math.floor((c.width - c.thickness) / 2),
            0,
            Math.floor((c.width - c.thickness) / 2) + c.thickness,
            c.height,
          ),
        ];
  }
  const gapLength = Math.min(gap, Math.floor(length / (2 * count)));
  const dashTotal = length - count * gapLength;
  const dash = Math.floor(dashTotal / count);
  let extra = dashTotal - dash * count;
  const out: SpriteShape[] = [];
  let at = across ? Math.floor(gapLength / 2) : 0;
  for (let i = 0; i < count; i++) {
    let end = at + dash;
    if (extra > 0) {
      extra -= 1;
      end += 1;
    }
    if (across) {
      const y = Math.floor((c.height - thick) / 2);
      out.push(rect(at, y, end, y + thick));
    } else {
      const x = Math.floor((c.width - thick) / 2);
      out.push(rect(x, at, x + thick, end));
    }
    at = end + gapLength;
  }
  return out;
}

function arc(c: SpriteCell, corner: "tl" | "tr" | "bl" | "br"): SpriteShape {
  const t = c.thickness;
  const cx = Math.floor((c.width - t) / 2) + t / 2;
  const cy = Math.floor((c.height - t) / 2) + t / 2;
  const r = Math.min(c.width, c.height) / 2;
  const s = ARC_TENSION;
  const top = corner === "tl" || corner === "tr";
  const leftward = corner === "tl" || corner === "bl";
  const edgeY = top ? 0 : c.height;
  const towardY = top ? cy - r : cy + r;
  const handleY = top ? cy - s * r : cy + s * r;
  const sideX = leftward ? cx - r : cx + r;
  const handleX = leftward ? cx - s * r : cx + s * r;
  const endX = leftward ? 0 : c.width;
  return {
    kind: "stroke",
    width: t,
    d: `M${cx} ${edgeY}L${cx} ${towardY}C${cx} ${handleY} ${handleX} ${cy} ${sideX} ${cy}L${endX} ${cy}`,
  };
}

function diagonal(c: SpriteCell, rising: boolean): SpriteShape {
  const sx = Math.min(1, c.width / c.height);
  const sy = Math.min(1, c.height / c.width);
  const d = rising
    ? `M${c.width + 0.5 * sx} ${-0.5 * sy}L${-0.5 * sx} ${c.height + 0.5 * sy}`
    : `M${-0.5 * sx} ${-0.5 * sy}L${c.width + 0.5 * sx} ${c.height + 0.5 * sy}`;
  return { kind: "stroke", width: c.thickness, d };
}

function box(cp: number, c: SpriteCell): SpriteShape[] | null {
  const code = BOX_LINES.get(cp);
  if (code) return lines(c, code);
  const gapLight = Math.max(4, c.thickness);
  switch (cp) {
    case 0x2504:
      return dashes(c, 3, c.thickness, gapLight, true);
    case 0x2505:
      return dashes(c, 3, heavy(c), gapLight, true);
    case 0x2506:
      return dashes(c, 3, c.thickness, gapLight, false);
    case 0x2507:
      return dashes(c, 3, heavy(c), gapLight, false);
    case 0x2508:
      return dashes(c, 4, c.thickness, gapLight, true);
    case 0x2509:
      return dashes(c, 4, heavy(c), gapLight, true);
    case 0x250a:
      return dashes(c, 4, c.thickness, gapLight, false);
    case 0x250b:
      return dashes(c, 4, heavy(c), gapLight, false);
    case 0x254c:
      return dashes(c, 2, c.thickness, c.thickness, true);
    case 0x254d:
      return dashes(c, 2, heavy(c), heavy(c), true);
    case 0x254e:
      return dashes(c, 2, c.thickness, heavy(c), false);
    case 0x254f:
      return dashes(c, 2, heavy(c), heavy(c), false);
    case 0x256d:
      return [arc(c, "br")];
    case 0x256e:
      return [arc(c, "bl")];
    case 0x256f:
      return [arc(c, "tl")];
    case 0x2570:
      return [arc(c, "tr")];
    case 0x2571:
      return [diagonal(c, true)];
    case 0x2572:
      return [diagonal(c, false)];
    case 0x2573:
      return [diagonal(c, true), diagonal(c, false)];
    default:
      return null;
  }
}

type Align = "upper" | "lower" | "left" | "right";

function block(
  c: SpriteCell,
  align: Align,
  width: number,
  height: number,
  opacity = 1,
): SpriteShape {
  const w = Math.round(c.width * width);
  const h = Math.round(c.height * height);
  const x = align === "right" ? c.width - w : align === "left" ? 0 : Math.floor((c.width - w) / 2);
  const y =
    align === "lower" ? c.height - h : align === "upper" ? 0 : Math.floor((c.height - h) / 2);
  return rect(x, y, x + w, y + h, opacity);
}

function quadrants(
  c: SpriteCell,
  tl: boolean,
  tr: boolean,
  bl: boolean,
  br: boolean,
): SpriteShape[] {
  const out: SpriteShape[] = [];
  if (tl) out.push(fill(c, 0, 0.5, 0, 0.5));
  if (tr) out.push(fill(c, 0.5, 1, 0, 0.5));
  if (bl) out.push(fill(c, 0, 0.5, 0.5, 1));
  if (br) out.push(fill(c, 0.5, 1, 0.5, 1));
  return out;
}

const EIGHTHS = [0, 0.125, 0.25, 0.375, 0.5, 0.625, 0.75, 0.875, 1];

function blocks(cp: number, c: SpriteCell): SpriteShape[] | null {
  if (cp === 0x2580) return [block(c, "upper", 1, 0.5)];
  if (cp >= 0x2581 && cp <= 0x2587) return [block(c, "lower", 1, EIGHTHS[cp - 0x2580] ?? 0)];
  if (cp === 0x2588) return [rect(0, 0, c.width, c.height)];
  if (cp >= 0x2589 && cp <= 0x258f) return [block(c, "left", EIGHTHS[0x2590 - cp] ?? 0, 1)];
  if (cp === 0x2590) return [block(c, "right", 0.5, 1)];
  if (cp === 0x2591) return [rect(0, 0, c.width, c.height, SHADES.light)];
  if (cp === 0x2592) return [rect(0, 0, c.width, c.height, SHADES.medium)];
  if (cp === 0x2593) return [rect(0, 0, c.width, c.height, SHADES.dark)];
  if (cp === 0x2594) return [block(c, "upper", 1, 0.125)];
  if (cp === 0x2595) return [block(c, "right", 0.125, 1)];
  const quads: Record<number, [boolean, boolean, boolean, boolean]> = {
    0x2596: [false, false, true, false],
    0x2597: [false, false, false, true],
    0x2598: [true, false, false, false],
    0x2599: [true, false, true, true],
    0x259a: [true, false, false, true],
    0x259b: [true, true, true, false],
    0x259c: [true, true, false, true],
    0x259d: [false, true, false, false],
    0x259e: [false, true, true, false],
    0x259f: [false, true, true, true],
  };
  const q = quads[cp];
  return q ? quadrants(c, ...q) : null;
}

function braille(cp: number, c: SpriteCell): SpriteShape[] {
  const width = Math.floor(c.width);
  const height = Math.floor(c.height);
  let w = Math.min(Math.floor(width / 4), Math.floor(height / 8));
  let xSpacing = Math.floor(width / 4);
  let ySpacing = Math.floor(height / 8);
  let xMargin = Math.floor(xSpacing / 2);
  let yMargin = Math.floor(ySpacing / 2);
  let xLeft = width - 2 * xMargin - xSpacing - 2 * w;
  let yLeft = height - 2 * yMargin - 3 * ySpacing - 4 * w;
  if (xLeft >= 2 && yLeft >= 4 && w === 0) {
    w += 1;
    xLeft -= 2;
    yLeft -= 4;
  }
  if (xLeft >= 2 && xMargin === 0) {
    xMargin = 1;
    xLeft -= 2;
  }
  if (yLeft >= 2 && yMargin === 0) {
    yMargin = 1;
    yLeft -= 2;
  }
  if (xLeft >= 1) {
    xSpacing += 1;
    xLeft -= 1;
  }
  if (yLeft >= 3) {
    ySpacing += 1;
    yLeft -= 3;
  }
  if (xLeft >= 2) {
    xMargin += 1;
    xLeft -= 2;
  }
  if (yLeft >= 2) {
    yMargin += 1;
    yLeft -= 2;
  }
  if (xLeft >= 2 && yLeft >= 4) w += 1;
  const xs = [xMargin, xMargin + w + xSpacing];
  const ys = [0, 1, 2, 3].map((i) => yMargin + i * (w + ySpacing));
  const bits = cp - 0x2800;
  const dots: Array<[number, number]> = [
    [0, 0],
    [0, 1],
    [0, 2],
    [1, 0],
    [1, 1],
    [1, 2],
    [0, 3],
    [1, 3],
  ];
  return dots.flatMap(([col, row], i) => {
    if (!(bits & (1 << i))) return [];
    const x = xs[col] ?? 0;
    const y = ys[row] ?? 0;
    return [rect(x, y, x + w, y + w)];
  });
}

function polygon(points: Array<[number, number]>): SpriteShape {
  return { kind: "fill", d: `M${points.map(([x, y]) => `${x} ${y}`).join("L")}Z` };
}

function flip(shape: SpriteShape, width: number): SpriteShape {
  function mirror(d: string) {
    return d.replace(/([MLC])([^MLCZ]+)/g, (_, cmd: string, args: string) => {
      const n = args
        .trim()
        .split(/[\s,]+/)
        .map(Number);
      const out = n.map((v, i) => (i % 2 === 0 ? width - v : v));
      return `${cmd}${out.join(" ")}`;
    });
  }
  return shape.kind === "rect"
    ? { ...shape, x: width - shape.x - shape.width }
    : { ...shape, d: mirror(shape.d) };
}

function halfCircle(c: SpriteCell, outline: boolean): SpriteShape {
  const r = Math.min(c.width, c.height / 2);
  const k = BEZIER_CIRCLE * r;
  const start = outline ? 1 : 0;
  const d =
    `M0 0${outline ? `L${start} 0` : ""}C${k} 0 ${r} ${r - k} ${r} ${r}` +
    `L${r} ${c.height - r}C${r} ${c.height - r + k} ${k} ${c.height} ${start} ${c.height}` +
    (outline ? `L0 ${c.height}` : "Z");
  return outline ? { kind: "stroke", width: c.thickness, d } : { kind: "fill", d };
}

function powerline(cp: number, c: SpriteCell): SpriteShape[] | null {
  const { width: w, height: h } = c;
  switch (cp) {
    case 0xe0b0:
      return [
        polygon([
          [0, 0],
          [w, h / 2],
          [0, h],
        ]),
      ];
    case 0xe0b2:
      return [
        polygon([
          [w, 0],
          [0, h / 2],
          [w, h],
        ]),
      ];
    case 0xe0b1:
      return [{ kind: "stroke", width: c.thickness, d: `M0 0L${w} ${h / 2}L0 ${h}` }];
    case 0xe0b3:
      return [{ kind: "stroke", width: c.thickness, d: `M${w} 0L0 ${h / 2}L${w} ${h}` }];
    case 0xe0b4:
      return [halfCircle(c, false)];
    case 0xe0b5:
      return [halfCircle(c, true)];
    case 0xe0b6:
      return [flip(halfCircle(c, false), w)];
    case 0xe0b7:
      return [flip(halfCircle(c, true), w)];
    case 0xe0b8:
      return [
        polygon([
          [0, 0],
          [w, h],
          [0, h],
        ]),
      ];
    case 0xe0ba:
      return [
        polygon([
          [w, 0],
          [w, h],
          [0, h],
        ]),
      ];
    case 0xe0bc:
      return [
        polygon([
          [0, 0],
          [w, 0],
          [0, h],
        ]),
      ];
    case 0xe0be:
      return [
        polygon([
          [0, 0],
          [w, 0],
          [w, h],
        ]),
      ];
    case 0xe0b9:
    case 0xe0bf:
      return [diagonal(c, false)];
    case 0xe0bb:
    case 0xe0bd:
      return [diagonal(c, true)];
    default:
      return null;
  }
}

function geometric(cp: number, c: SpriteCell): SpriteShape[] | null {
  const { width: w, height: h } = c;
  switch (cp) {
    case 0x25e2:
      return [
        polygon([
          [w, 0],
          [w, h],
          [0, h],
        ]),
      ];
    case 0x25e3:
      return [
        polygon([
          [0, 0],
          [w, h],
          [0, h],
        ]),
      ];
    case 0x25e4:
      return [
        polygon([
          [0, 0],
          [w, 0],
          [0, h],
        ]),
      ];
    case 0x25e5:
      return [
        polygon([
          [0, 0],
          [w, 0],
          [w, h],
        ]),
      ];
    case 0x25f8:
      return [{ kind: "stroke", width: c.thickness, d: `M0 0L${w} 0L0 ${h}Z` }];
    case 0x25f9:
      return [{ kind: "stroke", width: c.thickness, d: `M0 0L${w} 0L${w} ${h}Z` }];
    case 0x25fa:
      return [{ kind: "stroke", width: c.thickness, d: `M0 0L${w} ${h}L0 ${h}Z` }];
    case 0x25ff:
      return [{ kind: "stroke", width: c.thickness, d: `M${w} 0L${w} ${h}L0 ${h}Z` }];
    default:
      return null;
  }
}

function sextants(cp: number, c: SpriteCell): SpriteShape[] {
  const index = cp - 0x1fb00;
  const bits = index + Math.floor(index / 0x14) + 1;
  const parts: Array<[number, number, number, number]> = [
    [0, 0.5, 0, 1 / 3],
    [0.5, 1, 0, 1 / 3],
    [0, 0.5, 1 / 3, 2 / 3],
    [0.5, 1, 1 / 3, 2 / 3],
    [0, 0.5, 2 / 3, 1],
    [0.5, 1, 2 / 3, 1],
  ];
  return parts.flatMap(([x0, x1, y0, y1], i) => (bits & (1 << i) ? [fill(c, x0, x1, y0, y1)] : []));
}

function mosaic(pattern: string, c: SpriteCell): SpriteShape {
  function at(i: number) {
    return pattern[i] === "#";
  }
  const corners: Array<[boolean, number, number]> = [
    [at(0), 0, 0],
    [at(4) && (!at(0) || !at(8)), 0, 1 / 3],
    [at(8) && (!at(4) || !at(12)), 0, 2 / 3],
    [at(12), 0, 1],
    [at(13) && (!at(12) || !at(14)), 0.5, 1],
    [at(14), 1, 1],
    [at(10) && (!at(14) || !at(6)), 1, 2 / 3],
    [at(6) && (!at(10) || !at(2)), 1, 1 / 3],
    [at(2), 1, 0],
    [at(1) && (!at(2) || !at(0)), 0.5, 0],
  ];
  return polygon(corners.filter(([on]) => on).map(([, fx, fy]) => [fx * c.width, fy * c.height]));
}

function legacy(cp: number, c: SpriteCell): SpriteShape[] | null {
  if (cp >= 0x1fb00 && cp <= 0x1fb3b) return sextants(cp, c);
  const pattern = MOSAICS.get(cp);
  if (pattern) return [mosaic(pattern, c)];
  if (cp >= 0x1fb70 && cp <= 0x1fb75) {
    const n = cp + 1 - 0x1fb70;
    return [fill(c, EIGHTHS[n] ?? 0, EIGHTHS[n + 1] ?? 1, 0, 1)];
  }
  if (cp >= 0x1fb76 && cp <= 0x1fb7b) {
    const n = cp + 1 - 0x1fb76;
    return [fill(c, 0, 1, EIGHTHS[n] ?? 0, EIGHTHS[n + 1] ?? 1)];
  }
  const tops: Record<number, number> = {
    0x1fb82: 0.25,
    0x1fb83: 0.375,
    0x1fb84: 0.625,
    0x1fb85: 0.75,
    0x1fb86: 0.875,
  };
  const rights: Record<number, number> = {
    0x1fb87: 0.25,
    0x1fb88: 0.375,
    0x1fb89: 0.625,
    0x1fb8a: 0.75,
    0x1fb8b: 0.875,
  };
  if (tops[cp] !== undefined) return [block(c, "upper", 1, tops[cp])];
  if (rights[cp] !== undefined) return [block(c, "right", rights[cp], 1)];
  if (cp >= 0x1cd00 && cp <= 0x1cde5) {
    const bits = OCTANTS[cp - 0x1cd00] ?? 0;
    const parts: Array<[number, number, number, number]> = [
      [0, 0.5, 0, 0.25],
      [0.5, 1, 0, 0.25],
      [0, 0.5, 0.25, 0.5],
      [0.5, 1, 0.25, 0.5],
      [0, 0.5, 0.5, 0.75],
      [0.5, 1, 0.5, 0.75],
      [0, 0.5, 0.75, 1],
      [0.5, 1, 0.75, 1],
    ];
    return parts.flatMap(([x0, x1, y0, y1], i) =>
      bits & (1 << i) ? [fill(c, x0, x1, y0, y1)] : [],
    );
  }
  return null;
}

export function isSprite(cp: number): boolean {
  return (
    (cp >= 0x2500 && cp <= 0x259f) ||
    (cp >= 0x2800 && cp <= 0x28ff) ||
    (cp >= 0xe0b0 && cp <= 0xe0bf) ||
    [0x25e2, 0x25e3, 0x25e4, 0x25e5, 0x25f8, 0x25f9, 0x25fa, 0x25ff].includes(cp) ||
    (cp >= 0x1fb00 && cp <= 0x1fb67) ||
    (cp >= 0x1fb70 && cp <= 0x1fb7b) ||
    (cp >= 0x1fb82 && cp <= 0x1fb8b) ||
    (cp >= 0x1cd00 && cp <= 0x1cde5)
  );
}

export function spriteFor(cp: number, cell: SpriteCell): SpriteShape[] | null {
  if (!isSprite(cp)) return null;
  if (cp >= 0x2800 && cp <= 0x28ff) return braille(cp, cell);
  return (
    box(cp, cell) ??
    blocks(cp, cell) ??
    powerline(cp, cell) ??
    geometric(cp, cell) ??
    legacy(cp, cell)
  );
}

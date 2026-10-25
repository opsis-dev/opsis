import path from "node:path";
import { hueAndSaturationOf } from "@opsis/core/color";
import { round3 } from "@opsis/core/math";
import {
  autoFixNode,
  colourNode,
  DEFAULT_TINT,
  lensNode,
  lookFileNode,
  monoNode,
  textureNode,
  toneOffsets,
} from "@opsis/core/nodes/grade";
import { colorOr } from "@opsis/core/nodes/params";
import type { StyleEntry } from "@opsis/core/schema/values";

export type GradeSize = { width: number; height: number; still: boolean };

type Piece = {
  filters: string[];
  strength?: number;
  lookImage?: string;
};

type GradeEntry = Pick<StyleEntry, "type" | "params">;

const DESIGN_WIDTH = 1920;
const NEUTRAL_KELVIN = 6500;
const KELVIN_SWING = 3500;
const TINT_LIGHTNESS = 0.5;
const EVEN_SIZE = "scale=trunc(iw/2)*2:trunc(ih/2)*2";

const fadeOf = (strength: number): number | undefined => (strength < 1 ? strength : undefined);

const num = (value: number): string => String(round3(value));

const optionValue = (text: string): string => text.replace(/[\\':]/g, "\\$&");

const graphText = (text: string): string => text.replace(/[\\',;[\]]/g, "\\$&");

export const filterPath = (file: string): string => graphText(optionValue(file));

export const lookKindOf = (file: string): "table" | "image" | null => {
  const ext = path.extname(file).toLowerCase();
  if (ext === ".cube" || ext === ".3dl") return "table";
  return ext === ".png" ? "image" : null;
};

const options = (name: string, values: Array<[string, string | number | false]>): string[] => {
  const used = values.flatMap(([key, value]) => (value === false ? [] : [`${key}=${value}`]));
  return used.length ? [`${name}=${used.join(":")}`] : [];
};

type Colour = ReturnType<typeof colourNode.params.parse>;

function colourFilters(p: Colour): string[] {
  const shadows = toneOffsets(p.shadows, p.shadowsAmount);
  const midtones = toneOffsets(p.midtones, p.midtonesAmount);
  const highlights = toneOffsets(p.highlights, p.highlightsAmount);
  const balance = (tones: [number, number, number] | null, keys: [string, string, string]) =>
    keys.map((key, i): [string, number | false] => [key, tones ? tones[i] : false]);
  return [
    ...options("exposure", [["exposure", p.exposure !== 0 && num(p.exposure)]]),
    ...options("eq", [
      ["brightness", p.brightness !== 0 && num(p.brightness)],
      ["contrast", p.contrast !== 1 && num(p.contrast)],
      ["saturation", p.saturation !== 1 && num(p.saturation)],
      ["gamma", p.gamma !== 1 && num(p.gamma)],
    ]),
    ...options("colortemperature", [
      ["temperature", p.warmth !== 0 && Math.round(NEUTRAL_KELVIN - p.warmth * KELVIN_SWING)],
      ["mix", p.warmth !== 0 && 1],
    ]),
    ...options("vibrance", [["intensity", p.vibrance !== 0 && num(p.vibrance)]]),
    ...options("hue", [["h", p.hue !== 0 && num(p.hue)]]),
    ...options("colorbalance", [
      ...balance(shadows, ["rs", "gs", "bs"]),
      ...balance(midtones, ["rm", "gm", "bm"]),
      ...balance(highlights, ["rh", "gh", "bh"]),
    ]),
    ...options("curves", [["preset", p.curve !== "none" && p.curve]]),
  ];
}

const SEPIA = "colorchannelmixer=.393:.769:.189:0:.349:.686:.168:0:.272:.534:.131";

function monoPiece(p: ReturnType<typeof monoNode.params.parse>): Piece | null {
  if (p.strength === 0) return null;
  if (p.kind === "tint") {
    const tint = hueAndSaturationOf(colorOr(p.tint, DEFAULT_TINT));
    const filters = options("colorize", [
      ["hue", num(tint?.hue ?? 0)],
      ["saturation", num(tint?.saturation ?? 0.5)],
      ["lightness", TINT_LIGHTNESS],
      ["mix", 1],
    ]);
    return { filters, strength: fadeOf(p.strength) };
  }
  const filters = p.kind === "sepia" ? [SEPIA] : ["monochrome=cb=0:cr=0:size=1"];
  return { filters, strength: fadeOf(p.strength) };
}

function textureFilters(p: ReturnType<typeof textureNode.params.parse>, size: GradeSize): string[] {
  return [
    ...options("cas", [["strength", p.sharpen > 0 && num(p.sharpen)]]),
    ...options("gblur", [
      ["sigma", p.soften > 0 && num(Math.max(0.01, p.soften * 8 * (size.width / DESIGN_WIDTH)))],
    ]),
    ...options("vignette", [
      ["angle", p.vignette > 0 && num((p.vignette * Math.PI) / 2)],
      ["x0", p.vignette > 0 && `w*${num(p.vignetteX)}`],
      ["y0", p.vignette > 0 && `h*${num(p.vignetteY)}`],
    ]),
    ...(size.still
      ? []
      : options("noise", [
          ["alls", p.grain > 0 && Math.max(1, Math.round(p.grain * 40))],
          ["allf", p.grain > 0 && "t+u"],
        ])),
  ];
}

function lensFilters(p: ReturnType<typeof lensNode.params.parse>, size: GradeSize): string[] {
  const shift = Math.max(1, Math.round(p.fringe * 8 * (size.width / DESIGN_WIDTH)));
  return [
    ...options("rgbashift", [
      ["rh", p.fringe > 0 && shift],
      ["bh", p.fringe > 0 && -shift],
    ]),
    ...options("lenscorrection", [
      ["k1", p.bulge !== 0 && num(p.bulge * 0.5)],
      ["k2", p.bulge !== 0 && 0],
      ["i", p.bulge !== 0 && "bilinear"],
    ]),
    ...(size.still
      ? []
      : options("lagfun", [["decay", p.trails > 0 && num(0.5 + p.trails * 0.48)]])),
  ];
}

function autoFixFilters(p: ReturnType<typeof autoFixNode.params.parse>, size: GradeSize): string[] {
  return [
    ...(p.whiteBalance
      ? ["format=gbrpf32le", "grayworld", `format=${size.still ? "rgb24" : "yuv420p"}`]
      : []),
    ...(p.levels && p.levelsStrength > 0
      ? options("normalize", [
          ["smoothing", p.levelsSmoothing],
          ["independence", 0],
          ["strength", num(p.levelsStrength)],
        ])
      : []),
    ...(p.deband ? ["deband"] : []),
  ];
}

function lookPiece(
  p: ReturnType<typeof lookFileNode.params.parse>,
  resolve: (file: string) => string,
): Piece | null {
  if (!p.file || p.strength === 0) return null;
  const kind = lookKindOf(p.file);
  if (!kind) throw new Error("A look file ends in .cube, .3dl or .png.");
  const strength = fadeOf(p.strength);
  if (kind === "image") return { filters: [], lookImage: resolve(p.file), strength };
  return {
    filters: options("lut3d", [
      ["file", filterPath(resolve(p.file))],
      ["interp", "tetrahedral"],
    ]),
    strength,
  };
}

function pieceOf(
  entry: GradeEntry,
  size: GradeSize,
  resolve: (file: string) => string,
): Piece | null {
  switch (entry.type) {
    case colourNode.type:
      return { filters: colourFilters(colourNode.params.parse(entry.params)) };
    case lookFileNode.type:
      return lookPiece(lookFileNode.params.parse(entry.params), resolve);
    case monoNode.type:
      return monoPiece(monoNode.params.parse(entry.params));
    case textureNode.type:
      return { filters: textureFilters(textureNode.params.parse(entry.params), size) };
    case lensNode.type:
      return { filters: lensFilters(lensNode.params.parse(entry.params), size) };
    case autoFixNode.type:
      return { filters: autoFixFilters(autoFixNode.params.parse(entry.params), size) };
    default:
      throw new Error(`"${entry.type}" isn't a grade step.`);
  }
}

export function gradeFilters(
  entries: readonly GradeEntry[],
  size: GradeSize,
  resolve: (file: string) => string,
): { chain: string; extraInputs: string[] } {
  const statements: string[] = [];
  const extraInputs: string[] = [];
  let current = "0:v";
  let pending: string[] = [];
  let labels = 0;
  const label = () => `g${labels++}`;

  const flush = () => {
    if (!pending.length) return;
    const out = label();
    statements.push(`[${current}]${pending.join(",")}[${out}]`);
    current = out;
    pending = [];
  };

  for (const entry of entries) {
    const piece = pieceOf(entry, size, resolve);
    if (!piece || (!piece.filters.length && !piece.lookImage)) continue;
    if (piece.strength === undefined && !piece.lookImage) {
      pending.push(...piece.filters);
      continue;
    }
    flush();
    let branch = current;
    let original = "";
    if (piece.strength !== undefined) {
      original = label();
      branch = label();
      statements.push(`[${current}]split[${original}][${branch}]`);
    }
    const graded = label();
    if (piece.lookImage) {
      extraInputs.push(piece.lookImage);
      statements.push(`[${branch}][${extraInputs.length}:v]haldclut=repeatlast=1[${graded}]`);
    } else statements.push(`[${branch}]${piece.filters.join(",")}[${graded}]`);
    if (piece.strength === undefined) current = graded;
    else {
      const out = label();
      statements.push(
        `[${graded}][${original}]blend=all_mode=normal:all_opacity=${num(piece.strength)}[${out}]`,
      );
      current = out;
    }
  }

  const last = [...pending, ...(size.still ? [] : [EVEN_SIZE])];
  statements.push(`[${current}]${last.length ? last.join(",") : "null"}[out]`);
  return { chain: statements.join(";"), extraInputs };
}

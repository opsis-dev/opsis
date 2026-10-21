import { create, type Font as FontFile, type FontCollection } from "fontkit";
import type { FontData } from "three/addons/loaders/FontLoader.js";
import decompress from "woff2-encoder/decompress";

const LATIN_RANGES: Array<[number, number]> = [
  [0x20, 0x7e],
  [0xa0, 0x24f],
  [0x2010, 0x2027],
  [0x2030, 0x203a],
  [0x20ac, 0x20ac],
  [0x2122, 0x2122],
];

const WOFF2_SIGNATURE = "wOF2";
const DECIMALS = 100;

export const FONT_UNREADABLE = "Couldn't prepare this font for 3D text.";

const openFont = create as (bytes: Uint8Array) => FontFile | FontCollection;

const isCollection = (font: FontFile | FontCollection): font is FontCollection =>
  "fonts" in font && Array.isArray(font.fonts);

const signatureOf = (bytes: Uint8Array) => String.fromCharCode(...bytes.subarray(0, 4));

export async function readFont(bytes: Uint8Array): Promise<FontFile> {
  const sfnt = signatureOf(bytes) === WOFF2_SIGNATURE ? await decompress(bytes) : bytes;
  const opened = openFont(sfnt);
  const font = isCollection(opened) ? opened.fonts[0] : opened;
  if (!font) throw new Error(FONT_UNREADABLE);
  return font;
}

export function atWeight(font: FontFile, weight: number): FontFile {
  const axis = font.variationAxes.wght;
  if (!axis) return font;
  return font.getVariation({ wght: Math.min(axis.max, Math.max(axis.min, weight)) });
}

const n = (value: number) => Math.round(value * DECIMALS) / DECIMALS;

type PathCommand = { command: string; args: number[] };

export function outlineOf(commands: readonly PathCommand[]): string {
  const parts: string[] = [];
  for (const { command, args } of commands) {
    const a = args.map(n);
    switch (command) {
      case "moveTo":
        parts.push(`m ${a[0]} ${a[1]}`);
        break;
      case "lineTo":
        parts.push(`l ${a[0]} ${a[1]}`);
        break;
      case "quadraticCurveTo":
        parts.push(`q ${a[2]} ${a[3]} ${a[0]} ${a[1]}`);
        break;
      case "bezierCurveTo":
        parts.push(`b ${a[4]} ${a[5]} ${a[0]} ${a[1]} ${a[2]} ${a[3]}`);
        break;
    }
  }
  return parts.join(" ");
}

export function typefaceOf(font: FontFile): FontData {
  const glyphs: FontData["glyphs"] = {};
  for (const [from, to] of LATIN_RANGES)
    for (let code = from; code <= to; code++) {
      if (!font.hasGlyphForCodePoint(code)) continue;
      const glyph = font.glyphForCodePoint(code);
      glyphs[String.fromCodePoint(code)] = {
        ha: n(glyph.advanceWidth),
        x_min: n(glyph.bbox.minX),
        x_max: n(glyph.bbox.maxX),
        o: outlineOf(glyph.path.commands),
      };
    }
  const { bbox } = font;
  return {
    glyphs,
    familyName: font.familyName,
    ascender: font.ascent,
    descender: font.descent,
    underlinePosition: font.underlinePosition,
    underlineThickness: font.underlineThickness,
    boundingBox: { xMin: bbox.minX, yMin: bbox.minY, xMax: bbox.maxX, yMax: bbox.maxY },
    resolution: font.unitsPerEm,
    original_font_information: {},
  };
}

const loaded = new Map<string, Promise<FontData>>();

async function fetchOutlines(url: string, weight: number): Promise<FontData> {
  const res = await fetch(url);
  if (!res.ok) throw new Error(FONT_UNREADABLE);
  const font = await readFont(new Uint8Array(await res.arrayBuffer()));
  return typefaceOf(atWeight(font, weight));
}

export function loadOutlines(url: string, weight: number): Promise<FontData> {
  const key = `${url}|${weight}`;
  let pending = loaded.get(key);
  if (!pending) {
    pending = fetchOutlines(url, weight).catch((error: unknown) => {
      loaded.delete(key);
      throw error instanceof Error && error.message === FONT_UNREADABLE
        ? error
        : new Error(FONT_UNREADABLE, { cause: error });
    });
    loaded.set(key, pending);
  }
  return pending;
}

export function wrapLines(
  text: string,
  maxWidth: number,
  widthOf: (s: string) => number,
): string[] {
  const lines: string[] = [];
  for (const paragraph of text.split("\n")) {
    let line = "";
    for (const word of paragraph.split(/\s+/).filter(Boolean)) {
      const next = line ? `${line} ${word}` : word;
      if (line && widthOf(next) > maxWidth) {
        lines.push(line);
        line = word;
      } else line = next;
    }
    lines.push(line);
  }
  return lines;
}

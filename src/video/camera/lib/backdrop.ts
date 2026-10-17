import { DEFAULT_BACKDROP, type BackdropName } from "../../../shared/backdrops";

const LIGHT_BACKDROPS: ReadonlySet<string> = new Set<BackdropName>([
  "canyon",
  "mist",
  "chalk",
  "aurora",
]);

const LIGHT_LUMINANCE = 0.6;
const HEX_COLOUR = /^#([0-9a-f]{3}|[0-9a-f]{6})$/i;
const RGB_COLOUR = /^rgba?\(\s*(\d+(?:\.\d+)?)[\s,]+(\d+(?:\.\d+)?)[\s,]+(\d+(?:\.\d+)?)[\s,/)]/i;
const CSS_PAINT = /^(#|rgb|hsl|oklch|linear-gradient|radial-gradient|conic-gradient)/;
const IMAGE_URL = /^(https?:|demoflow-file:|data:|blob:)/;

function colourChannels(colour: string): number[] | null {
  const text = colour.trim();
  const hex = HEX_COLOUR.exec(text)?.[1];
  if (hex) {
    const pairs =
      hex.length === 3
        ? [...hex].map((d) => d + d)
        : [hex.slice(0, 2), hex.slice(2, 4), hex.slice(4)];
    return pairs.map((c) => parseInt(c, 16) / 255);
  }
  const rgb = RGB_COLOUR.exec(text);
  return rgb ? rgb.slice(1, 4).map((c) => Number(c) / 255) : null;
}

function luminance(colour: string): number | null {
  const channels = colourChannels(colour);
  if (!channels) return null;
  const [r, g, b] = channels;
  return 0.2126 * r + 0.7152 * g + 0.0722 * b;
}

export const isLightBackdrop = (backdrop: string): boolean =>
  LIGHT_BACKDROPS.has(backdrop) || (luminance(backdrop) ?? 0) > LIGHT_LUMINANCE;

export function backdropFile(name?: string): string {
  const file = name?.trim() || DEFAULT_BACKDROP;
  return file.includes(".") ? `backdrops/${file}` : `backdrops/${file}.jpg`;
}

type BackdropSource =
  | { kind: "paint"; css: string }
  | { kind: "image"; url: string }
  | { kind: "file"; path: string };

export function backdropSource(backdrop?: string): BackdropSource {
  if (backdrop && CSS_PAINT.test(backdrop.trim())) return { kind: "paint", css: backdrop };
  if (backdrop && IMAGE_URL.test(backdrop)) return { kind: "image", url: backdrop };
  return { kind: "file", path: backdropFile(backdrop) };
}

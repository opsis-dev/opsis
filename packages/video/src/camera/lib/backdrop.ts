import { isExternalRef } from "@opsis/core/asset-ref";
import { BACKDROP_TONES, DEFAULT_BACKDROP, type BackdropName } from "@opsis/core/backdrops";
import { opaqueHex, parseColor } from "@opsis/core/color";
import { isKeyOf, nonBlankString } from "@opsis/core/guards";

const LIGHT_LUMA = 0.6;
const HEX_COLOUR = /^#([0-9a-f]{3}|[0-9a-f]{6})$/i;
const RGB_COLOUR = /^rgba?\(\s*(\d+(?:\.\d+)?)[\s,]+(\d+(?:\.\d+)?)[\s,]+(\d+(?:\.\d+)?)[\s,/)]/i;
const CSS_PAINT = /^(#|rgb|hsl|oklch|linear-gradient|radial-gradient|conic-gradient)/;

function colourChannels(colour: string): number[] | null {
  const hex = HEX_COLOUR.exec(colour)?.[1];
  if (hex) {
    const pairs =
      hex.length === 3
        ? [...hex].map((d) => d + d)
        : [hex.slice(0, 2), hex.slice(2, 4), hex.slice(4)];
    return pairs.map((c) => parseInt(c, 16) / 255);
  }
  const rgb = RGB_COLOUR.exec(colour);
  return rgb ? rgb.slice(1, 4).map((c) => Number(c) / 255) : null;
}

function luma(colour: string): number | null {
  const parsed = parseColor(colour);
  const channels = colourChannels(colour) ?? (parsed ? colourChannels(opaqueHex(parsed)) : null);
  if (!channels) return null;
  const [r, g, b] = channels;
  return 0.2126 * r + 0.7152 * g + 0.0722 * b;
}

const isBackdropName = (value: string): value is BackdropName => isKeyOf(BACKDROP_TONES, value);

export function isLightBackdrop(backdrop?: string): boolean {
  const value = nonBlankString(backdrop) ?? "";
  return isBackdropName(value)
    ? BACKDROP_TONES[value] === "light"
    : (luma(value) ?? 0) > LIGHT_LUMA;
}

function backdropFile(name?: string): string {
  const file = nonBlankString(name) ?? DEFAULT_BACKDROP;
  return file.includes(".") ? `backdrops/${file}` : `backdrops/${file}.jpg`;
}

type BackdropSource =
  | { kind: "paint"; css: string }
  | { kind: "image"; url: string }
  | { kind: "file"; path: string };

export function backdropSource(backdrop?: string): BackdropSource {
  const value = nonBlankString(backdrop) ?? "";
  if (CSS_PAINT.test(value) || (!isBackdropName(value) && parseColor(value)))
    return { kind: "paint", css: value };
  if (isExternalRef(value)) return { kind: "image", url: value };
  return { kind: "file", path: backdropFile(value) };
}

const FONTS: Record<string, string> = {
  Inter: '"Inter"',
  Geist: '"Geist"',
  "Space Grotesk": '"Space Grotesk"',
  Outfit: '"Outfit"',
  Montserrat: '"Montserrat"',
  "Instrument Serif": '"Instrument Serif"',
};

const DEFAULT_FONT = "Default";
export function resolveFont(name?: string): string | undefined {
  return !name || name === DEFAULT_FONT ? undefined : (FONTS[name] ?? name);
}

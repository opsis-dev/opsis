export const FONTS: Record<string, string> = {
  Inter: '"Inter"',
  Geist: '"Geist"',
  "Space Grotesk": '"Space Grotesk"',
  Outfit: '"Outfit"',
  Montserrat: '"Montserrat"',
  "Instrument Serif": '"Instrument Serif"',
};

export const DEFAULT_FONT = "Default";
export const FONT_NAMES = [DEFAULT_FONT, ...Object.keys(FONTS)];

export const resolveFont = (name?: string): string | undefined =>
  !name || name === DEFAULT_FONT ? undefined : (FONTS[name] ?? name);

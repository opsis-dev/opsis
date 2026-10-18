export const BACKDROPS = [
  "glaze",
  "ink",
  "cobalt",
  "graphite",
  "ember",
  "flare",
  "prism",
  "bloom",
  "halo",
  "dusk",
  "studio",
  "moonrise",
  "canyon",
  "mist",
  "chalk",
  "aurora",
] as const;

export type BackdropName = (typeof BACKDROPS)[number];

export const BACKDROP_TONES: Record<BackdropName, "light" | "dark"> = {
  glaze: "dark",
  ink: "dark",
  cobalt: "dark",
  graphite: "dark",
  ember: "dark",
  flare: "dark",
  prism: "dark",
  bloom: "dark",
  halo: "dark",
  dusk: "dark",
  studio: "dark",
  moonrise: "dark",
  canyon: "light",
  mist: "light",
  chalk: "light",
  aurora: "light",
};

export const DEFAULT_BACKDROP: BackdropName = "glaze";

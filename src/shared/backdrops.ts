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

export const MOTIONS = [
  "spin",
  "tilt-in",
  "float",
  "orbit",
  "swing",
  "still",
  "keyframes",
] as const;
export type Motion = (typeof MOTIONS)[number];

export const MOTION_LABELS: Record<Motion, string> = {
  spin: "Spin",
  "tilt-in": "Tilt in",
  float: "Float",
  orbit: "Orbit",
  swing: "Swing",
  still: "Still",
  keyframes: "Keyframes",
};

export const LIGHTINGS = ["studio", "soft", "dramatic"] as const;
export type Lighting = (typeof LIGHTINGS)[number];

export const LIGHTING_LABELS: Record<Lighting, string> = {
  studio: "Studio",
  soft: "Soft",
  dramatic: "Dramatic",
};

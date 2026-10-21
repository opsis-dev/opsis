export const DEVICES = ["laptop", "monitor", "phone", "floating"] as const;
export type Device = (typeof DEVICES)[number];

export const DEVICE_LABELS: Record<Device, string> = {
  laptop: "Laptop",
  monitor: "Monitor",
  phone: "Phone",
  floating: "Floating screen",
};

export const SCREEN_MOVES = ["push-in", "orbit", "tilt-reveal", "rise", "keyframes"] as const;
export type ScreenMove = (typeof SCREEN_MOVES)[number];

export const SCREEN_MOVE_LABELS: Record<ScreenMove, string> = {
  "push-in": "Push in",
  orbit: "Orbit",
  "tilt-reveal": "Tilt reveal",
  rise: "Rise",
  keyframes: "Keyframes",
};

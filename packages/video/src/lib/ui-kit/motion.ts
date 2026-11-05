export const easings = {
  linear: (t: number): number => t,
  out: (t: number): number => 1 - (1 - t) ** 3,
  in: (t: number): number => t * t * t,
  inOut: (t: number): number => (t < 0.5 ? 4 * t * t * t : 1 - (-2 * t + 2) ** 3 / 2),
} as const;

export type EasingName = keyof typeof easings;

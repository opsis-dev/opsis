import { clamp01 } from "#video/common/math.ts";
import {
  prepareMorph,
  MORPH_SHAPES,
  type MorphPair,
  type MorphShapeName,
} from "#video/elements/transition/transition-morph-shape/path-morph.ts";

type DisplacementProfile = "peak" | "ramp-in" | "ramp-out";

export type DisplacementCoreConfig = {
  scale?: number;
  scaleRatio?: number;
  frequency?: number;
  octaves?: number;
  seed?: number;
  turbulenceType?: "fractalNoise" | "turbulence";
  churn?: number;
  blur?: number;
  profile?: DisplacementProfile;
};

export const DISPLACEMENT_DEFAULTS = {
  scale: 90,
  frequency: 0.008,
  octaves: 2,
  seed: 7,
  turbulenceType: "fractalNoise",
  churn: 6,
  blur: 0,
  profile: "peak",
} as const satisfies Omit<Required<DisplacementCoreConfig>, "scaleRatio">;

function displacementIntensity(
  displace: number,
  profile: DisplacementProfile = DISPLACEMENT_DEFAULTS.profile,
): number {
  const d = clamp01(displace);
  if (profile === "ramp-in") {
    return d;
  }
  if (profile === "ramp-out") {
    return 1 - d;
  }
  return 4 * d * (1 - d);
}

export type DisplacementFrameState = {
  scale: number;
  blur: number;
  seed: number;
  intensity: number;
  overscan: number;
};

export function displacementFrame(
  displace: number,
  frameSize: number,
  config: DisplacementCoreConfig = {},
): DisplacementFrameState {
  const peakScale =
    config.scaleRatio !== undefined
      ? config.scaleRatio * frameSize
      : (config.scale ?? DISPLACEMENT_DEFAULTS.scale);
  const peakBlur = config.blur ?? DISPLACEMENT_DEFAULTS.blur;
  const seedBase = config.seed ?? DISPLACEMENT_DEFAULTS.seed;
  const churn = config.churn ?? DISPLACEMENT_DEFAULTS.churn;
  const profile = config.profile ?? DISPLACEMENT_DEFAULTS.profile;

  const intensity = displacementIntensity(displace, profile);
  const scale = peakScale * intensity;
  const blur = peakBlur * intensity;

  return {
    scale,
    blur,
    seed: Math.round(seedBase + clamp01(displace) * churn),
    intensity,
    overscan: overscanFor(scale, blur, frameSize),
  };
}

function overscanFor(scale: number, blur: number, frameSize: number): number {
  const drag = scale / 2 + blur * 3;
  if (drag <= 0 || frameSize <= 0) return 1;
  const clean = frameSize - 2 * (drag + 2);
  if (clean <= 0) return MAX_OVERSCAN;
  return Math.min(MAX_OVERSCAN, frameSize / clean);
}

const MAX_OVERSCAN = 3;

export const DISPLACEMENT_EPSILON = 0.5;

export type DisplacementMaskConfig = {
  from?: MorphShapeName | string;
  to?: MorphShapeName | string;
  overshoot?: number;
  originX?: number;
  originY?: number;
};

const MASK_DEFAULTS = {
  from: "circle",
  to: "circle",
  overshoot: 1.06,
  originX: 0.5,
  originY: 0.5,
} as const satisfies Required<DisplacementMaskConfig>;

function resolveShape(shape: MorphShapeName | string): string {
  return shape in MORPH_SHAPES ? MORPH_SHAPES[shape as MorphShapeName] : shape;
}

export function prepareMaskMorph(config: DisplacementMaskConfig = {}): MorphPair {
  const from = resolveShape(config.from ?? MASK_DEFAULTS.from);
  const to = resolveShape(config.to ?? MASK_DEFAULTS.to);
  return prepareMorph(from, to, { x: 0, y: 0, width: 1, height: 1 });
}

export type MaskFrameState = {
  coverage: number;
  size: number;
  centerX: number;
  centerY: number;
  isComplete: boolean;
};

export function maskFrame(
  displace: number,
  frame: { width: number; height: number },
  config: DisplacementMaskConfig = {},
): MaskFrameState {
  const overshoot = config.overshoot ?? MASK_DEFAULTS.overshoot;
  const originX = config.originX ?? MASK_DEFAULTS.originX;
  const originY = config.originY ?? MASK_DEFAULTS.originY;

  const coverage = 1 - clamp01(displace);
  const diagonal = Math.hypot(frame.width, frame.height);

  return {
    coverage,
    size: diagonal * overshoot * coverage,
    centerX: frame.width * originX,
    centerY: frame.height * originY,
    isComplete: coverage >= 1,
  };
}

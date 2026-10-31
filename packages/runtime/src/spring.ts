export type SpringConfig = {
  mass: number;
  damping: number;
  stiffness: number;
  overshootClamping: boolean;
};

export type SpringOptions = {
  frame: number;
  fps: number;
  config?: Partial<SpringConfig>;
  durationInFrames?: number;
  delay?: number;
};

export const DEFAULT_SPRING: SpringConfig = {
  mass: 1,
  damping: 10,
  stiffness: 100,
  overshootClamping: false,
};

export const REST_THRESHOLD = 0.005;
const LONGEST_SECONDS = 600;

export function springConfig(config: Partial<SpringConfig> = {}): SpringConfig {
  const resolved = { ...DEFAULT_SPRING, ...config };
  if (!(resolved.damping > 0) || !(resolved.mass > 0) || !(resolved.stiffness > 0))
    throw new Error("A spring's mass, damping and stiffness have to be above 0.");
  return resolved;
}

type Motion = { omega: number; zeta: number };

function motionOf({ mass, damping, stiffness }: SpringConfig): Motion {
  return {
    omega: Math.sqrt(stiffness / mass),
    zeta: damping / (2 * Math.sqrt(stiffness * mass)),
  };
}

function distanceLeft({ omega, zeta }: Motion, seconds: number): number {
  if (zeta < 1) {
    const damped = omega * Math.sqrt(1 - zeta * zeta);
    return (
      Math.exp(-zeta * omega * seconds) *
      (Math.cos(damped * seconds) + ((zeta * omega) / damped) * Math.sin(damped * seconds))
    );
  }
  if (zeta === 1) return Math.exp(-omega * seconds) * (1 + omega * seconds);
  const root = Math.sqrt(zeta * zeta - 1);
  const fast = -omega * (zeta + root);
  const slow = -omega * (zeta - root);
  return (fast * Math.exp(slow * seconds) - slow * Math.exp(fast * seconds)) / (fast - slow);
}

function firstArrival({ omega, zeta }: Motion): number {
  if (zeta >= 1) return Infinity;
  const damped = omega * Math.sqrt(1 - zeta * zeta);
  return (Math.PI - Math.atan(damped / (zeta * omega))) / damped;
}

function envelope({ omega, zeta }: Motion, seconds: number): number {
  if (zeta < 1) {
    const damped = omega * Math.sqrt(1 - zeta * zeta);
    return Math.exp(-zeta * omega * seconds) * Math.hypot(1, (zeta * omega) / damped);
  }
  return Math.abs(distanceLeft({ omega, zeta }, seconds));
}

export function springProgress(config: SpringConfig, seconds: number): number {
  if (seconds <= 0) return 0;
  const motion = motionOf(config);
  if (config.overshootClamping && seconds >= firstArrival(motion)) return 1;
  return 1 - distanceLeft(motion, seconds);
}

export function measureSpring({
  fps,
  config,
  threshold = REST_THRESHOLD,
}: {
  fps: number;
  config?: Partial<SpringConfig>;
  threshold?: number;
}): number {
  const resolved = springConfig(config);
  const motion = motionOf(resolved);
  const longest = Math.ceil(LONGEST_SECONDS * fps);
  let lastAway = 0;
  for (let frame = 0; frame <= longest; frame++) {
    const seconds = frame / fps;
    if (Math.abs(1 - springProgress(resolved, seconds)) >= threshold) lastAway = frame;
    else if (envelope(motion, seconds) < threshold) return lastAway + 1;
  }
  return longest;
}

export function spring({ frame, fps, config, durationInFrames, delay = 0 }: SpringOptions): number {
  const resolved = springConfig(config);
  const elapsed = frame - delay;
  if (elapsed <= 0) return 0;
  const stretch =
    durationInFrames === undefined ? 1 : measureSpring({ fps, config }) / durationInFrames;
  return springProgress(resolved, (elapsed * stretch) / fps);
}

import { interpolate } from "@opsis/runtime/interpolate.ts";
import { AbsoluteFill } from "@opsis/runtime/parts.tsx";
import { random } from "@opsis/runtime/random.ts";
import { CLAMPED } from "#video/core/anim.ts";
import { useAuthoredConfig, useAuthoredFrame } from "#video/core/authored-clock.ts";
import type { ElementSettings } from "#video/generated/spec.ts";

export interface Particle {
  angle: number;
  speed: number;
  color: string;
  size: number;
  rot0: number;
  spin: number;
  drift: number;
}

const DEG = Math.PI / 180;

function makeParticles({
  count,
  seed,
  colors,
  spread,
  direction,
  spin,
}: {
  count: number;
  seed: number;
  colors: string[];
  spread: number;
  direction: number;
  spin: number;
}): Particle[] {
  let draws = 0;
  function next(): number {
    draws += 1;
    return random(`${seed}:${draws}`);
  }
  const out: Particle[] = [];
  for (let i = 0; i < count; i++) {
    const angle = (direction - spread / 2 + next() * spread) * DEG;
    const speed = 0.35 + 0.65 * next();
    out.push({
      angle,
      speed,
      color: colors[Math.floor(next() * colors.length)],
      size: 0.7 + 0.6 * next(),
      rot0: next() * Math.PI * 2,
      spin: (next() - 0.5) * 0.6 * spin,
      drift: (next() - 0.5) * 2,
    });
  }
  return out;
}

function particleOpacity(tt: number, lifetime: number): number {
  if (tt < 0 || tt > lifetime) return 0;
  const fadeIn = interpolate(tt, [0, 4], [0, 1], CLAMPED);
  const fadeOut = interpolate(tt, [lifetime * 0.7, lifetime], [1, 0], CLAMPED);
  return Math.min(fadeIn, fadeOut);
}

export function Confetti({
  particleCount,
  colors,
  originX,
  originY,
  lifetime,
  power,
  gravity,
  size,
  seed,
  spread,
  direction,
  spin,
  pieceShape,
}: ElementSettings["confetti"]) {
  const frame = useAuthoredFrame();
  const { width, height } = useAuthoredConfig();

  const tt = frame;
  if (tt < 0 || tt > lifetime) return null;

  const sc = height / 720;
  const ox = width * originX;
  const oy = height * originY;
  const particles = makeParticles({
    count: particleCount,
    seed,
    colors,
    spread,
    direction,
    spin,
  });

  return (
    <AbsoluteFill style={{ pointerEvents: "none" }}>
      {particles.map((p, i) => {
        const opacity = particleOpacity(tt, lifetime);
        if (opacity <= 0) return null;

        const vx = Math.cos(p.angle) * p.speed * power + p.drift;
        const vy = Math.sin(p.angle) * p.speed * power;
        const x = ox + vx * tt * sc;
        const y = oy + (vy * tt + 0.5 * gravity * tt * tt) * sc;
        const rot = p.rot0 + p.spin * tt;
        const flutter = Math.cos(p.rot0 + p.spin * tt * 1.6);
        const w = size * p.size * sc;

        return (
          <div
            key={i}
            style={{
              position: "absolute",
              left: x,
              top: y,
              width: w,
              height: w * pieceShape,
              borderRadius: 2,
              background: p.color,
              opacity,
              translate: "-50% -50%",
              rotate: `${rot}rad`,
              scale: `${flutter} 1`,
            }}
          />
        );
      })}
    </AbsoluteFill>
  );
}

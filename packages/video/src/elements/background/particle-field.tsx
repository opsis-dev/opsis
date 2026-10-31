import { useFilm, useFrame } from "@opsis/runtime/clock.tsx";
import { AbsoluteFill } from "@opsis/runtime/parts.tsx";
import { random } from "@opsis/runtime/random.ts";
import { useMemo } from "react";
import type { ElementSettings } from "#video/generated/spec.ts";

const TRACK = 140;
const CROSSING_SECONDS = 8;

type Particle = {
  depth: number;
  offset: number;
  lane: number;
  driftAmount: number;
  driftPeriod: number;
  driftPhase: number;
  twinklePeriod: number;
  twinklePhase: number;
};

function buildParticles(count: number, seed: number): Particle[] {
  return Array.from({ length: count }, (_, index) => ({
    depth: random(`${seed}:${index}:1`) ** 2,
    offset: random(`${seed}:${index}:2`),
    lane: random(`${seed}:${index}:3`),
    driftAmount: 0.4 + random(`${seed}:${index}:4`) * 1.6,
    driftPeriod: 1.1 + random(`${seed}:${index}:5`) * 1.3,
    driftPhase: random(`${seed}:${index}:6`) * Math.PI * 2,
    twinklePeriod: 0.9 + random(`${seed}:${index}:7`) * 1.2,
    twinklePhase: random(`${seed}:${index}:8`) * Math.PI * 2,
  }));
}

export function ParticleField({
  backgroundColor,
  color,
  count,
  angle,
  speed,
  size,
  minSize,
  depthBlur,
  drift,
  intensity,
  glow,
  seed,
  twinkle: twinkleDepth,
}: ElementSettings["particle-field"]) {
  const frame = useFrame();
  const { fps, width } = useFilm();
  const pixel = width / 1920;
  const time = (frame / fps) * speed;

  const particles = useMemo(
    () => buildParticles(Math.max(1, Math.round(count)), seed),
    [count, seed],
  );
  const radians = (angle * Math.PI) / 180;
  const dirX = Math.sin(radians);
  const dirY = -Math.cos(radians);
  const perpX = Math.cos(radians);
  const perpY = Math.sin(radians);

  return (
    <AbsoluteFill style={{ background: backgroundColor, overflow: "hidden" }}>
      {particles.map((particle, index) => {
        const rate = (0.35 + particle.depth * 0.95) / CROSSING_SECONDS;
        const along = (particle.offset + time * rate) % 1;
        const wander =
          Math.sin((time / particle.driftPeriod) * Math.PI * 2 + particle.driftPhase) *
          drift *
          particle.driftAmount;
        const across = particle.lane * TRACK - TRACK / 2 + wander;
        const forward = along * TRACK - TRACK / 2;

        const x = 50 + dirX * forward + perpX * across;
        const y = 50 + dirY * forward + perpY * across;
        const diameter = (minSize + (size - minSize) * particle.depth) * pixel;
        const twinkle =
          1 -
          twinkleDepth +
          twinkleDepth *
            Math.sin((time / particle.twinklePeriod) * Math.PI * 2 + particle.twinklePhase);

        return (
          <div
            key={index}
            style={{
              position: "absolute",
              left: `${x.toFixed(3)}%`,
              top: `${y.toFixed(3)}%`,
              width: diameter,
              height: diameter,
              marginLeft: -diameter / 2,
              marginTop: -diameter / 2,
              borderRadius: "50%",
              background:
                glow > 0
                  ? `radial-gradient(circle, ${color} 0%, ${color} 34%, transparent 70%)`
                  : color,
              transform: glow > 0 ? `scale(${1 + glow})` : undefined,
              filter:
                depthBlur > 0
                  ? `blur(${((1 - particle.depth) * depthBlur * pixel).toFixed(2)}px)`
                  : undefined,
              opacity: (0.18 + particle.depth * 0.7) * twinkle * intensity,
              mixBlendMode: "screen",
            }}
          />
        );
      })}
    </AbsoluteFill>
  );
}

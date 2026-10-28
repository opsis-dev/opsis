import { useMemo } from "react";
import { AbsoluteFill, useCurrentFrame, useVideoConfig } from "remotion";

export type ParticleFieldProps = {
  backgroundColor?: string;
  color?: string;
  count?: number;
  angle?: number;
  speed?: number;
  size?: number;
  minSize?: number;
  depthBlur?: number;
  drift?: number;
  intensity?: number;
  glow?: number;
  seed?: number;
};

function hash(index: number, salt: number, seed: number): number {
  const value = Math.sin(index * 53.17 + salt * 79.3 + seed * 27.1) * 43758.5453;
  return value - Math.floor(value);
}

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
    depth: hash(index, 1, seed) ** 2,
    offset: hash(index, 2, seed),
    lane: hash(index, 3, seed),
    driftAmount: 0.4 + hash(index, 4, seed) * 1.6,
    driftPeriod: 1.1 + hash(index, 5, seed) * 1.3,
    driftPhase: hash(index, 6, seed) * Math.PI * 2,
    twinklePeriod: 0.9 + hash(index, 7, seed) * 1.2,
    twinklePhase: hash(index, 8, seed) * Math.PI * 2,
  }));
}

export const ParticleField: React.FC<ParticleFieldProps> = ({
  backgroundColor = "#05070f",
  color = "#e8b86d",
  count = 70,
  angle = 8,
  speed = 1,
  size = 14,
  minSize = 2,
  depthBlur = 3,
  drift = 4,
  intensity = 1,
  glow = 1.6,
  seed = 1,
}) => {
  const frame = useCurrentFrame();
  const { fps, width } = useVideoConfig();
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
          0.7 +
          0.3 * Math.sin((time / particle.twinklePeriod) * Math.PI * 2 + particle.twinklePhase);

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
};

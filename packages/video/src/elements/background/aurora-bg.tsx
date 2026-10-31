import { useFilm, useFrame } from "@opsis/runtime/clock.tsx";
import { AbsoluteFill } from "@opsis/runtime/parts.tsx";
import { random } from "@opsis/runtime/random.ts";
import { useId } from "react";
import type { ElementSettings } from "#video/generated/spec.ts";

const SAMPLES = 30;
const OVERSCAN = 12;

const SLOW_PERIOD = 2.3;
const FAST_PERIOD = 1.1;

function rayMask(strength: number): string {
  const dip = Math.max(0, 1 - strength).toFixed(3);
  return `linear-gradient(90deg, rgba(0,0,0,1) 0%, rgba(0,0,0,${dip}) 38%, rgba(0,0,0,0.82) 64%, rgba(0,0,0,1) 100%)`;
}

type Ribbon = {
  color: string;
  band: number;
  slowRate: number;
  fastRate: number;
  slowPhase: number;
  fastPhase: number;
  waves: number;
  drift: number;
  scale: number;
  alpha: number;
};

function buildRibbons(
  count: number,
  colors: string[],
  seed: number,
  folds: number,
  sidewaysDrift: number,
): Ribbon[] {
  return Array.from({ length: count }, (_, index) => {
    const t = count === 1 ? 0.5 : index / (count - 1);
    return {
      color: colors[index % colors.length],
      band: t - 0.5,
      slowRate: 0.8 + random(`${seed}:${index}:1`) * 0.5,
      fastRate: 0.75 + random(`${seed}:${index}:2`) * 0.6,
      slowPhase: random(`${seed}:${index}:3`) * Math.PI * 2,
      fastPhase: random(`${seed}:${index}:4`) * Math.PI * 2,
      waves: folds * (0.65 + random(`${seed}:${index}:5`) * 0.7),
      drift: (random(`${seed}:${index}:6`) - 0.5) * 2 * sidewaysDrift,
      scale: 0.65 + random(`${seed}:${index}:7`) * 0.6,
      alpha: 0.5 + random(`${seed}:${index}:8`) * 0.5,
    };
  });
}

function ribbonPath(
  ribbon: Ribbon,
  time: number,
  amplitude: number,
  thickness: number,
  centerY: number,
  spread: number,
): string {
  const k = (ribbon.waves * Math.PI * 2) / (100 + OVERSCAN * 2);
  const slow = (time / SLOW_PERIOD) * ribbon.slowRate * Math.PI * 2;
  const fast = (time / FAST_PERIOD) * ribbon.fastRate * Math.PI * 2;
  const base =
    centerY + ribbon.band * spread + Math.sin(slow * 0.5 + ribbon.slowPhase) * amplitude * 0.35;

  const top: number[] = [];
  const bottom: number[] = [];

  for (let i = 0; i <= SAMPLES; i += 1) {
    const u = i / SAMPLES;
    const x = -OVERSCAN + u * (100 + OVERSCAN * 2);
    const fold =
      Math.sin(k * (x + ribbon.drift * time) + slow + ribbon.slowPhase) * amplitude +
      Math.sin(k * 2.3 * x + fast + ribbon.fastPhase) * amplitude * 0.32;
    const taper = Math.sin(Math.PI * u) ** 1.4;
    const height = thickness * ribbon.scale * (0.3 + 0.7 * taper);
    top.push(base + fold - height);
    bottom.push(base + fold + height * 0.25);
  }

  const forward = top
    .map((y, i) => {
      const x = -OVERSCAN + (i / SAMPLES) * (100 + OVERSCAN * 2);
      return `${i === 0 ? "M" : "L"}${x.toFixed(2)},${y.toFixed(2)}`;
    })
    .join(" ");
  const back = bottom
    .map((_, i) => {
      const index = SAMPLES - i;
      const x = -OVERSCAN + (index / SAMPLES) * (100 + OVERSCAN * 2);
      return `L${x.toFixed(2)},${bottom[index].toFixed(2)}`;
    })
    .join(" ");

  return `${forward} ${back} Z`;
}

export function AuroraBg({
  backgroundColor,
  colors,
  ribbonCount,
  amplitude,
  thickness,
  centerY,
  spread,
  speed,
  blur,
  intensity,
  horizonGlow,
  striation,
  seed,
  folds,
  sidewaysDrift,
  raySpacing,
  vignette,
  horizonGlowColor,
}: ElementSettings["aurora-bg"]) {
  const frame = useFrame();
  const { fps } = useFilm();
  const uid = useId().replace(/:/g, "");
  const time = (frame / fps) * speed;

  const ribbons = buildRibbons(
    Math.max(1, Math.round(ribbonCount)),
    colors,
    seed,
    folds,
    sidewaysDrift,
  );

  return (
    <AbsoluteFill style={{ background: backgroundColor, overflow: "hidden" }}>
      {horizonGlow > 0 ? (
        <AbsoluteFill
          style={{
            background: `radial-gradient(120% 60% at 50% ${centerY + spread * 0.5}%, ${horizonGlowColor || colors[0]} 0%, transparent 70%)`,
            opacity:
              0.16 *
              horizonGlow *
              intensity *
              (0.85 + 0.15 * Math.sin((time / SLOW_PERIOD) * Math.PI)),
            mixBlendMode: "screen",
          }}
        />
      ) : null}

      <AbsoluteFill
        style={{
          filter: `blur(${blur}px)`,
          ...(striation > 0
            ? {
                maskImage: rayMask(striation),
                WebkitMaskImage: rayMask(striation),
                maskSize: `${raySpacing}px 100%`,
                WebkitMaskSize: `${raySpacing}px 100%`,
                maskPosition: `${((time * 9) % raySpacing).toFixed(2)}px 0`,
                WebkitMaskPosition: `${((time * 9) % raySpacing).toFixed(2)}px 0`,
              }
            : {}),
        }}
      >
        <svg
          viewBox="0 0 100 100"
          preserveAspectRatio="none"
          style={{ width: "100%", height: "100%" }}
        >
          <defs>
            {ribbons.map((ribbon, index) => (
              <linearGradient key={index} id={`aurora-${uid}-${index}`} x1="0" y1="0" x2="0" y2="1">
                <stop offset="0%" stopColor={ribbon.color} stopOpacity="0" />
                <stop offset="55%" stopColor={ribbon.color} stopOpacity="0.55" />
                <stop offset="100%" stopColor={ribbon.color} stopOpacity="1" />
              </linearGradient>
            ))}
          </defs>
          {ribbons.map((ribbon, index) => (
            <path
              key={index}
              d={ribbonPath(ribbon, time, amplitude, thickness, centerY, spread)}
              fill={`url(#aurora-${uid}-${index})`}
              opacity={ribbon.alpha * 0.72 * intensity}
              style={{ mixBlendMode: "screen" }}
            />
          ))}
        </svg>
      </AbsoluteFill>

      <AbsoluteFill
        style={{
          background: `radial-gradient(circle at 50% 45%, transparent 40%, rgba(0,0,0,${vignette}) 100%)`,
          pointerEvents: "none",
        }}
      />
    </AbsoluteFill>
  );
}

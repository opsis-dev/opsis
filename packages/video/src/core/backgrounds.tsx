import { LightLeak } from "@remotion/light-leaks";
import { noise3D } from "@remotion/noise";
import React from "react";
import {
  AbsoluteFill,
  getRemotionEnvironment,
  Img,
  random,
  staticFile,
  useCurrentFrame,
} from "remotion";
import { backdropSource } from "#video/camera/lib/backdrop.ts";

const visibleBackground = (background?: string): string | undefined =>
  background && background !== "transparent" ? background : undefined;

const Grain: React.FC<{ amount: number; seed?: number }> = ({ amount, seed = 3 }) =>
  amount > 0 ? (
    <AbsoluteFill style={{ opacity: amount, mixBlendMode: "overlay" }}>
      <svg width="100%" height="100%">
        <filter id={`grain-${seed}`}>
          <feTurbulence
            type="fractalNoise"
            baseFrequency="0.9"
            numOctaves={2}
            seed={seed}
            stitchTiles="stitch"
          />
          <feColorMatrix type="saturate" values="0" />
        </filter>
        <rect width="100%" height="100%" filter={`url(#grain-${seed})`} />
      </svg>
    </AbsoluteFill>
  ) : null;

export const Backgrounded: React.FC<{ background?: string; children?: React.ReactNode }> = ({
  background,
  children,
}) => <AbsoluteFill style={{ background: visibleBackground(background) }}>{children}</AbsoluteFill>;

export const Solid: React.FC<{ color: string }> = ({ color }) => (
  <AbsoluteFill style={{ background: color }} />
);

const DEFAULT_GRAIN = 0.08;

export const Gradient: React.FC<{ from: string; to: string; angle?: number; grain?: number }> = ({
  from,
  to,
  angle = 135,
  grain = DEFAULT_GRAIN,
}) => (
  <AbsoluteFill style={{ background: `linear-gradient(${angle}deg, ${from}, ${to})` }}>
    <Grain amount={grain} />
  </AbsoluteFill>
);

const BLOBS = 5;
const BLOB_DRIFT_PCT = 40;
const BLOB_RADIUS_PCT = 30;
const BLOB_RADIUS_SPREAD_PCT = 25;
const BLOB_NOISE_SPACING = 1000;
const BLOB_OPACITY = 0.55;
const BLOB_BLUR_PX = 40;

export const NoiseField: React.FC<{
  color1: string;
  color2: string;
  scale?: number;
  speed?: number;
}> = ({ color1, color2, scale = 0.004, speed = 0.004 }) => {
  const frame = useCurrentFrame();
  const blobs = Array.from({ length: BLOBS }, (_, i) => {
    const offset = i * BLOB_NOISE_SPACING * scale;
    const x = 50 + noise3D(`x${i}`, offset, 0, frame * speed) * BLOB_DRIFT_PCT;
    const y = 50 + noise3D(`y${i}`, 0, offset, frame * speed) * BLOB_DRIFT_PCT;
    const r = BLOB_RADIUS_PCT + random(`r${i}`) * BLOB_RADIUS_SPREAD_PCT;
    return `radial-gradient(circle at ${x}% ${y}%, ${color2} 0%, transparent ${r}%)`;
  });
  return (
    <AbsoluteFill style={{ background: color1 }}>
      <AbsoluteFill
        style={{
          background: blobs.join(","),
          opacity: BLOB_OPACITY,
          filter: `blur(${BLOB_BLUR_PX}px)`,
        }}
      />
      <Grain amount={DEFAULT_GRAIN} seed={7} />
    </AbsoluteFill>
  );
};

const DEFAULT_RAYS = 24;
const DEFAULT_ROTATION_SPEED = 0.15;
const STARBURST_OVERSCAN = "-50%";
const STARBURST_VIGNETTE_PCT = 75;

export const Starburst: React.FC<{
  color1: string;
  color2: string;
  rays?: number;
  rotationSpeed?: number;
}> = ({ color1, color2, rays = DEFAULT_RAYS, rotationSpeed = DEFAULT_ROTATION_SPEED }) => {
  const frame = useCurrentFrame();
  const step = 360 / Math.max(2, rays);
  return (
    <AbsoluteFill style={{ overflow: "hidden", background: color1 }}>
      <div
        style={{
          position: "absolute",
          inset: STARBURST_OVERSCAN,
          background: `repeating-conic-gradient(from ${frame * rotationSpeed}deg, ${color1} 0deg ${step / 2}deg, ${color2} ${step / 2}deg ${step}deg)`,
        }}
      />
      <AbsoluteFill
        style={{
          background: `radial-gradient(circle, transparent 0%, ${color1} ${STARBURST_VIGNETTE_PCT}%)`,
        }}
      />
    </AbsoluteFill>
  );
};

const PAPER_GRAIN = 0.18;

export const Paper: React.FC<{ color: string; grain?: number }> = ({
  color,
  grain = PAPER_GRAIN,
}) => (
  <AbsoluteFill style={{ background: color }}>
    <Grain amount={grain} seed={11} />
  </AbsoluteFill>
);

const BackdropImage: React.FC<{ src: string }> = ({ src }) => {
  const style = { width: "100%", height: "100%", objectFit: "cover" } as const;
  return getRemotionEnvironment().isRendering ? (
    <Img src={src} style={style} />
  ) : (
    <img src={src} alt="" decoding="async" style={style} />
  );
};

export const Backdrop: React.FC<{ name?: string }> = ({ name }) => {
  const source = backdropSource(name);
  if (source.kind === "paint") return <AbsoluteFill style={{ background: source.css }} />;
  return (
    <AbsoluteFill>
      <BackdropImage src={source.kind === "image" ? source.url : staticFile(source.path)} />
    </AbsoluteFill>
  );
};

export const LightLeakElement: React.FC<{ seed: number; hueShift: number }> = (props) => (
  <LightLeak {...props} />
);

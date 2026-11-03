import type { ReactNode } from "react";
import type { TerminalEffects } from "#video/generated/spec.ts";
import { hexColour } from "#video/terminal/frames.ts";

const DESIGN_WIDTH = 1920;
const SHIFT_PER_FRINGE = 8;
const BLUR_PER_SOFTEN = 8;
const VIGNETTE_DEPTH = 0.9;
const GRAIN_OPACITY = 0.35;
const GRAIN_FREQUENCY = 0.85;
const BULGE_SHARE = 0.18;
const MAP_SIZE = 128;
const MAP_MIDDLE = 128;
const MAP_SPAN = 127;
const TRAIL_FRAMES = 3;
const TRAIL_DECAY_FROM = 0.5;
const TRAIL_DECAY_SPAN = 0.48;

type Size = { width: number; height: number };

const maps = new Map<string, string>();

function bulgeMap(x: number, y: number): string {
  const key = `${x}|${y}`;
  const known = maps.get(key);
  if (known !== undefined) return known;
  if (typeof document === "undefined") return "";
  const canvas = document.createElement("canvas");
  canvas.width = MAP_SIZE;
  canvas.height = MAP_SIZE;
  const context = canvas.getContext("2d");
  if (!context) return "";
  const image = context.createImageData(MAP_SIZE, MAP_SIZE);
  for (let j = 0; j < MAP_SIZE; j++)
    for (let i = 0; i < MAP_SIZE; i++) {
      const dx = (i + 0.5) / MAP_SIZE - x;
      const dy = (j + 0.5) / MAP_SIZE - y;
      const pull = Math.max(0, 1 - (dx * dx + dy * dy) * 2);
      const at = (j * MAP_SIZE + i) * 4;
      image.data[at] = MAP_MIDDLE - dx * pull * MAP_SPAN;
      image.data[at + 1] = MAP_MIDDLE - dy * pull * MAP_SPAN;
      image.data[at + 2] = MAP_MIDDLE;
      image.data[at + 3] = 255;
    }
  context.putImageData(image, 0, 0);
  const url = canvas.toDataURL("image/png");
  maps.set(key, url);
  return url;
}

export function trailWeights(effects: TerminalEffects | undefined): number[] {
  if (!effects || effects.trails <= 0) return [];
  const decay = effects.trails * TRAIL_DECAY_SPAN + TRAIL_DECAY_FROM;
  return Array.from({ length: TRAIL_FRAMES }, (_, k) => decay ** (k + 1));
}

export function softenPx(effects: TerminalEffects | undefined, size: Size): number {
  return effects ? effects.soften * BLUR_PER_SOFTEN * (size.width / DESIGN_WIDTH) : 0;
}

function lensFilter(id: string, effects: TerminalEffects, size: Size): ReactNode {
  const shift = Math.max(
    1,
    Math.round(effects.fringe * SHIFT_PER_FRINGE * (size.width / DESIGN_WIDTH)),
  );
  const angle = (effects.fringeAngle * Math.PI) / 180;
  const dx = effects.fringe > 0 ? Math.round(shift * Math.cos(angle)) : 0;
  const dy = effects.fringe > 0 ? Math.round(shift * Math.sin(angle)) : 0;
  const bulge = effects.bulge !== 0;
  return (
    <filter
      id={id}
      x={0}
      y={0}
      width={size.width}
      height={size.height}
      filterUnits="userSpaceOnUse"
      colorInterpolationFilters="sRGB"
    >
      {bulge ? (
        <>
          <feImage
            href={bulgeMap(effects.bulgeX, effects.bulgeY)}
            x={0}
            y={0}
            width={size.width}
            height={size.height}
            preserveAspectRatio="none"
            result="map"
          />
          <feDisplacementMap
            in="SourceGraphic"
            in2="map"
            scale={effects.bulge * BULGE_SHARE * Math.max(size.width, size.height)}
            xChannelSelector="R"
            yChannelSelector="G"
            result="bent"
          />
        </>
      ) : (
        <feOffset in="SourceGraphic" dx={0} dy={0} result="bent" />
      )}
      <feColorMatrix in="bent" values="1 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 1 0" result="red" />
      <feOffset in="red" dx={dx} dy={dy} result="redShifted" />
      <feColorMatrix in="bent" values="0 0 0 0 0 0 1 0 0 0 0 0 0 0 0 0 0 0 1 0" result="green" />
      <feColorMatrix in="bent" values="0 0 0 0 0 0 0 0 0 0 0 0 1 0 0 0 0 0 1 0" result="blue" />
      <feOffset in="blue" dx={-dx} dy={-dy} result="blueShifted" />
      <feBlend in="redShifted" in2="green" mode="screen" result="redGreen" />
      <feBlend in="redGreen" in2="blueShifted" mode="screen" />
    </filter>
  );
}

export function hasLens(effects: TerminalEffects | undefined): boolean {
  return !!effects && (effects.fringe > 0 || effects.bulge !== 0);
}

export function EffectDefs({
  id,
  effects,
  size,
  frame,
}: {
  id: string;
  effects: TerminalEffects;
  size: Size;
  frame: number;
}) {
  const black = hexColour(0);
  const radius = effects.vignetteFollowsFrame ? 0.75 : Math.hypot(size.width, size.height) * 0.5;
  return (
    <defs>
      {hasLens(effects) ? lensFilter(`${id}-lens`, effects, size) : null}
      <radialGradient
        id={`${id}-vignette`}
        cx={effects.vignetteFollowsFrame ? effects.vignetteX : effects.vignetteX * size.width}
        cy={effects.vignetteFollowsFrame ? effects.vignetteY : effects.vignetteY * size.height}
        r={radius}
        gradientUnits={effects.vignetteFollowsFrame ? "objectBoundingBox" : "userSpaceOnUse"}
      >
        <stop offset="0.45" stopColor={black} stopOpacity={0} />
        <stop offset="1" stopColor={black} stopOpacity={effects.vignette * VIGNETTE_DEPTH} />
      </radialGradient>
      <filter id={`${id}-grain`} x={0} y={0} width="100%" height="100%">
        <feTurbulence
          type="fractalNoise"
          baseFrequency={GRAIN_FREQUENCY}
          numOctaves={1}
          seed={frame}
          stitchTiles="stitch"
        />
        {effects.colourGrain ? null : <feColorMatrix type="saturate" values="0" />}
      </filter>
    </defs>
  );
}

export function EffectLayers({
  id,
  effects,
  size,
}: {
  id: string;
  effects: TerminalEffects;
  size: Size;
}) {
  return (
    <>
      {effects.vignette > 0 ? (
        <rect x={0} y={0} width={size.width} height={size.height} fill={`url(#${id}-vignette)`} />
      ) : null}
      {effects.grain > 0 ? (
        <rect
          x={0}
          y={0}
          width={size.width}
          height={size.height}
          filter={`url(#${id}-grain)`}
          opacity={effects.grain * GRAIN_OPACITY}
          style={{ mixBlendMode: "overlay" }}
        />
      ) : null}
    </>
  );
}

import { DEFAULT_CURSOR_LOOK } from "@opsis/core/compile/recording";
import { percent } from "@opsis/core/format";
import type { ArrowCursorLook, CursorLook, RippleLook } from "@opsis/core/schema/recording";
import { resolveCursor } from "@remotion/mac-cursors";
import React from "react";
import { Img } from "remotion";
import { isFadedOut } from "../core/anim";
import type { ClickLog } from "./lib/click-log";
import { cursorAt } from "./lib/cursor";
import { useDesignScale } from "./lib/design-scale";
import { faded, ink } from "./lib/style";

const CURSOR_SCALE_EXP = 0.58;
const HOT_X = 1;
const HOT_Y = 1;
const ARROW_W = 34;
const ARROW_H = 45;
const ARROW_VIEW_W = 24;
const ARROW_VIEW_H = 32;
const RIPPLE_D = 22;
const RIPPLE_RING = "55%";
const RIPPLE_FILL = "10%";
const RIPPLE_FROM = 0.3;
const RIPPLE_GROWTH = 1.05;
const RIPPLE_OPACITY = 0.85;
const ARROW_PATH = "M1 1 L1 23 L6.5 17.8 L10.2 26.6 L13.7 25 L9.95 16.5 L17 16.3 Z";
const ARROW_STROKE = 1.85;
const MAC_CURSOR_PX = 64;
const MAC_ARTWORK_PX = 32;

const Ripple: React.FC<{ progress: number; k: number; look: RippleLook }> = ({
  progress,
  k,
  look,
}) => {
  const size = RIPPLE_D * look.size * k;
  return (
    <div
      style={{
        position: "absolute",
        left: 0,
        top: 0,
        width: size,
        height: size,
        marginLeft: -size / 2,
        marginTop: -size / 2,
        borderRadius: "50%",
        border: `${look.width * k}px solid ${faded(look.color, RIPPLE_RING)}`,
        background: faded(look.color, RIPPLE_FILL),
        transform: `scale(${RIPPLE_FROM + progress * RIPPLE_GROWTH})`,
        opacity: RIPPLE_OPACITY * (1 - progress),
      }}
    />
  );
};

const Arrow: React.FC<{ squash: number; k: number; look: ArrowCursorLook }> = ({
  squash,
  k,
  look,
}) => {
  const width = ARROW_W * k;
  const height = ARROW_H * k;
  return (
    <svg
      width={width}
      height={height}
      viewBox={`0 0 ${ARROW_VIEW_W} ${ARROW_VIEW_H}`}
      xmlns="http://www.w3.org/2000/svg"
      style={{
        position: "absolute",
        left: (-HOT_X / ARROW_VIEW_W) * width,
        top: (-HOT_Y / ARROW_VIEW_H) * height,
        transform: `scale(${squash})`,
        transformOrigin: `${percent(HOT_X / ARROW_VIEW_W)} ${percent(HOT_Y / ARROW_VIEW_H)}`,
        filter: `drop-shadow(0 ${1.5 * k}px ${2.5 * k}px ${ink(0.34)})`,
      }}
    >
      <path
        d={ARROW_PATH}
        fill={look.fill}
        stroke={look.edge}
        strokeWidth={ARROW_STROKE}
        strokeLinejoin="round"
        paintOrder="stroke"
      />
    </svg>
  );
};

const MacCursor: React.FC<{ shape: string; squash: number; k: number }> = ({
  shape,
  squash,
  k,
}) => {
  const resolved = resolveCursor(shape);
  if (!resolved) return null;
  const size = MAC_CURSOR_PX * k;
  const { x, y } = resolved.hotspot;
  const scale = size / (resolved.width ?? MAC_ARTWORK_PX);
  return (
    <Img
      src={resolved.src}
      width={size}
      height={size}
      style={{
        position: "absolute",
        left: -x * scale,
        top: -y * scale,
        width: size,
        height: size,
        maxWidth: "none",
        transform: `scale(${squash})`,
        transformOrigin: `${x * scale}px ${y * scale}px`,
      }}
    />
  );
};

export type CursorOptions = {
  log: ClickLog;
  cursor: boolean;
  ripple: RippleLook | null;
  cursorScale?: number;
  cursorLook?: CursorLook;
};

export const Cursor: React.FC<{ clip: CursorOptions; timeS: number; cameraScale: number }> = ({
  clip,
  timeS,
  cameraScale,
}) => {
  const k = useDesignScale();
  const { log, ripple, cursorScale: scale = 1, cursorLook = DEFAULT_CURSOR_LOOK } = clip;
  const state = clip.cursor ? cursorAt(log.cursorTrack ?? [], log.clicks, timeS) : null;
  if (!state || isFadedOut(state.opacity)) return null;

  const { width, height } = log.viewport;
  const counter = cameraScale ** (CURSOR_SCALE_EXP - 1) * scale;

  return (
    <div
      style={{
        position: "absolute",
        left: percent(state.x / width),
        top: percent(state.y / height),
        width: 0,
        height: 0,
        pointerEvents: "none",
        opacity: state.opacity,
        transform: `scale(${counter})`,
        transformOrigin: "0 0",
      }}
    >
      {ripple && state.ripple !== null ? (
        <Ripple progress={state.ripple} k={k} look={ripple} />
      ) : null}
      {cursorLook.shape === "mac" ? (
        <MacCursor shape={state.shape} squash={state.squash} k={k} />
      ) : (
        <Arrow squash={state.squash} k={k} look={cursorLook} />
      )}
    </div>
  );
};

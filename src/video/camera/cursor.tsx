import React from "react";
import { percent } from "../../shared/format";
import { isFadedOut } from "../core/anim";
import type { ClickLog } from "./lib/click-log";
import { cursorAt } from "./lib/cursor";
import { useDesignScale } from "./lib/design-scale";

const CURSOR_SCALE_EXP = 0.58;
const HOT_X = 1;
const HOT_Y = 1;
const ARROW_W = 34;
const ARROW_H = 45;
const ARROW_VIEW_W = 24;
const ARROW_VIEW_H = 32;
const RIPPLE_D = 22;
const RIPPLE_STROKE = 2;
const RIPPLE_FROM = 0.3;
const RIPPLE_GROWTH = 1.05;
const RIPPLE_OPACITY = 0.85;
const INK = "15, 23, 42";

const Ripple: React.FC<{ progress: number; k: number }> = ({ progress, k }) => {
  const size = RIPPLE_D * k;
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
        border: `${RIPPLE_STROKE * k}px solid rgba(${INK}, 0.55)`,
        background: `rgba(${INK}, 0.10)`,
        transform: `scale(${RIPPLE_FROM + progress * RIPPLE_GROWTH})`,
        opacity: RIPPLE_OPACITY * (1 - progress),
      }}
    />
  );
};

const Arrow: React.FC<{ squash: number; k: number }> = ({ squash, k }) => {
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
        filter: `drop-shadow(0 ${1.5 * k}px ${2.5 * k}px rgba(${INK}, 0.34))`,
      }}
    >
      <path
        d="M1 1 L1 23 L6.5 17.8 L10.2 26.6 L13.7 25 L9.95 16.5 L17 16.3 Z"
        fill="#111418"
        stroke="#ffffff"
        strokeWidth={1.85}
        strokeLinejoin="round"
        paintOrder="stroke"
      />
    </svg>
  );
};

export const Cursor: React.FC<{
  log: ClickLog;
  timeS: number;
  cameraScale: number;
  ripple?: boolean;
  scale?: number;
}> = ({ log, timeS, cameraScale, ripple = true, scale = 1 }) => {
  const k = useDesignScale();

  const state = cursorAt(log.cursorTrack ?? [], log.clicks, timeS);
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
      {ripple && state.ripple !== null ? <Ripple progress={state.ripple} k={k} /> : null}
      <Arrow squash={state.squash} k={k} />
    </div>
  );
};

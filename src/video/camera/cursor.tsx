import React from "react";
import { CURSOR_SCALE_EXP, type ClickLog } from "./lib/click-log";
import { cursorAt } from "./lib/cursor";
import { useDesignScale } from "./lib/design-scale";

const HOT_X = 1;
const HOT_Y = 1;
const ARROW_W = 34;
const ARROW_H = 45;
const RIPPLE_D = 22;
const RIPPLE_STROKE = 2;
const RIPPLE_FROM = 0.3;
const RIPPLE_GROWTH = 1.05;
const RIPPLE_OPACITY = 0.85;
const INK = "15, 23, 42";

export const Cursor: React.FC<{
  log: ClickLog;
  timeS: number;
  cameraScale: number;
  ripple?: boolean;
  scale?: number;
}> = ({ log, timeS, cameraScale, ripple = true, scale = 1 }) => {
  const k = useDesignScale();

  const state = cursorAt(log.cursorTrack ?? [], log.clicks, timeS);
  if (!state || state.opacity <= 0.001) return null;

  const { width, height } = log.viewport;
  const counter = Math.pow(cameraScale, CURSOR_SCALE_EXP - 1) * scale;
  const arrowW = ARROW_W * k;
  const arrowH = ARROW_H * k;
  const rippleD = RIPPLE_D * k;

  return (
    <div
      style={{
        position: "absolute",
        left: `${(state.x / width) * 100}%`,
        top: `${(state.y / height) * 100}%`,
        width: 0,
        height: 0,
        pointerEvents: "none",
        opacity: state.opacity,
        transform: `scale(${counter})`,
        transformOrigin: "0 0",
      }}
    >
      {ripple && state.ripple !== null ? (
        <div
          style={{
            position: "absolute",
            left: 0,
            top: 0,
            width: rippleD,
            height: rippleD,
            marginLeft: -rippleD / 2,
            marginTop: -rippleD / 2,
            borderRadius: "50%",
            border: `${RIPPLE_STROKE * k}px solid rgba(${INK}, 0.55)`,
            background: `rgba(${INK}, 0.10)`,
            transform: `scale(${RIPPLE_FROM + state.ripple * RIPPLE_GROWTH})`,
            opacity: RIPPLE_OPACITY * (1 - state.ripple),
          }}
        />
      ) : null}

      <svg
        width={arrowW}
        height={arrowH}
        viewBox="0 0 24 32"
        xmlns="http://www.w3.org/2000/svg"
        style={{
          position: "absolute",
          left: (-HOT_X / 24) * arrowW,
          top: (-HOT_Y / 32) * arrowH,
          transform: `scale(${state.squash})`,
          transformOrigin: `${(HOT_X / 24) * 100}% ${(HOT_Y / 32) * 100}%`,
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
    </div>
  );
};

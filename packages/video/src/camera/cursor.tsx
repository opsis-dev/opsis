import { Img } from "@opsis/runtime/parts.tsx";
import type { ClickLog } from "#video/camera/lib/click-log.ts";
import { cursorAt, type Ring } from "#video/camera/lib/cursor.ts";
import { useDesignScale } from "#video/camera/lib/design-scale.ts";
import {
  MAC_CURSOR_BOX,
  MAC_CURSOR_KEYWORDS,
  MAC_CURSORS,
} from "#video/camera/lib/mac-cursors.generated.ts";
import { faded, ink } from "#video/camera/lib/style.ts";
import { percent } from "#video/common/format.ts";
import { DEFAULT_CURSOR_FEEL, type ArrowCursorLook } from "#video/common/recording.ts";
import { isFadedOut } from "#video/core/anim.ts";
import type { CursorFeel, PointerProps, RippleLook } from "#video/generated/spec.ts";

const HOT_X = 1;
const HOT_Y = 1;
const ARROW_W = 34;
const ARROW_H = 45;
const ARROW_VIEW_W = 24;
const ARROW_VIEW_H = 32;
const RIPPLE_D = 22;
const RIPPLE_RING = 0.47;
const RIPPLE_FILL = 0.085;
const RIPPLE_S = 0.35;
const RIPPLE_FROM = 0.3;
const RIPPLE_GROWTH = 1.05;
const ARROW_PATH = "M1 1 L1 23 L6.5 17.8 L10.2 26.6 L13.7 25 L9.95 16.5 L17 16.3 Z";
const MAC_CURSOR_PX = 44;

function Ripple({
  ring: { progress, hollow },
  k,
  look,
}: {
  ring: Ring;
  k: number;
  look: RippleLook;
}) {
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
        border: `${look.width * k}px solid ${faded(look.color, percent(look.ringOpacity ?? RIPPLE_RING))}`,
        background: hollow
          ? "transparent"
          : faded(look.color, percent(look.fillOpacity ?? RIPPLE_FILL)),
        transform: `scale(${RIPPLE_FROM + progress * RIPPLE_GROWTH})`,
        opacity: 1 - progress,
      }}
    />
  );
}

function Arrow({
  squash,
  k,
  look,
  feel,
}: {
  squash: number;
  k: number;
  look: ArrowCursorLook;
  feel: CursorFeel;
}) {
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
        filter:
          feel.shadow > 0
            ? `drop-shadow(0 ${1.5 * k}px ${2.5 * k}px ${ink(feel.shadow)})`
            : undefined,
      }}
    >
      <path
        d={ARROW_PATH}
        fill={look.fill}
        stroke={look.edge}
        strokeWidth={feel.outlineWidth}
        strokeLinejoin="round"
        paintOrder="stroke"
      />
    </svg>
  );
}

function macCursor(keyword: string) {
  if (keyword === "none") return null;
  return MAC_CURSORS[MAC_CURSOR_KEYWORDS[keyword] ?? MAC_CURSOR_KEYWORDS.default ?? ""] ?? null;
}

function MacCursor({ shape, squash, k }: { shape: string; squash: number; k: number }) {
  const resolved = macCursor(shape);
  if (!resolved) return null;
  const size = MAC_CURSOR_PX * k;
  const [x, y] = resolved.hotspot;
  const scale = size / MAC_CURSOR_BOX;
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
}

export type CursorOptions = {
  log: ClickLog;
  pointer: PointerProps;
};

export function Cursor({
  clip,
  timeS,
  cameraScale,
}: {
  clip: CursorOptions;
  timeS: number;
  cameraScale: number;
}) {
  const k = useDesignScale();
  const { log } = clip;
  const { ripple, scale, look: cursorLook, feel = DEFAULT_CURSOR_FEEL } = clip.pointer;
  const motion = {
    pressSquash: feel.pressSquash,
    hideWhileTyping: feel.hideWhileTyping,
    rippleS: ripple?.durationS ?? RIPPLE_S,
  };
  const state = clip.pointer.shown
    ? cursorAt(log.cursorTrack ?? [], log.clicks, timeS, motion)
    : null;
  if (!state || isFadedOut(state.opacity)) return null;

  const { width, height } = log.viewport;
  const counter = cameraScale ** (feel.growWithZoom - 1) * scale;

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
      {ripple
        ? state.rings.map((ring, i) => <Ripple key={i} ring={ring} k={k} look={ripple} />)
        : null}
      {cursorLook.shape === "mac" ? (
        <MacCursor shape={state.shape} squash={state.squash} k={k} />
      ) : (
        <Arrow squash={state.squash} k={k} look={cursorLook} feel={feel} />
      )}
    </div>
  );
}

import { useFilm, useFrame } from "@opsis/runtime/clock.tsx";
import { interpolate } from "@opsis/runtime/interpolate.ts";
import { AbsoluteFill } from "@opsis/runtime/parts.tsx";
import React, { useMemo } from "react";
import { assetUrl, useReframe } from "#video/assets.tsx";
import {
  resolveAnnotations,
  toClickLog,
  TYPED_LINGER_S,
  typedBeats,
} from "#video/camera/action-log.ts";
import { DemoClip, ScreenClip, useClipTime } from "#video/camera/demo-clip.tsx";
import { frameAddress, terminalTitle } from "#video/camera/frame-address.ts";
import { KeycapHUD } from "#video/camera/keycap-hud.tsx";
import { DesignWidthContext } from "#video/camera/lib/design-scale.ts";
import { KEYSTROKE_EDGE, shownKeys } from "#video/camera/lib/keycap.ts";
import { faded, SHOT_STYLES } from "#video/camera/lib/style.ts";
import { poseAt } from "#video/camera/lib/zoom.ts";
import { STAGE_COLOR } from "#video/common/element-colors.ts";
import { CODE_FONT } from "#video/common/fonts.ts";
import { clamp } from "#video/common/math.ts";
import { CLAMPED, fadeWindow } from "#video/core/anim.ts";
import {
  JUSTIFY_AT,
  overlayPadding,
  reframeLayout,
  useOverlayFrame,
  verticalInsets,
  type Reframe,
} from "#video/core/frame.ts";
import { hasScreen } from "#video/core/screen.tsx";
import type { Size } from "#video/core/size.ts";
import type { KeystrokeLook, RecordingProps } from "#video/generated/spec.ts";

const TYPED_FADE_IN_S = 0.15;
const TYPED_FADE_OUT_S = 0.2;
const CARET_BLINKS_PER_S = 2.5;

const TYPED_CHIP = {
  padY: 14,
  padX: 26,
  fontSize: 30,
  shadowY: 8,
  shadowBlur: 24,
  maxWidth: "70%",
  keycapShare: "82%",
  shadow: 0.35,
} as const;

export function RecordingClip(props: RecordingProps) {
  const { width, height, fps } = useFilm();
  const frame = useFrame();
  const reframe = useReframe();
  const timeS = useClipTime(props.speed);
  const { zoom, tuning } = props.camera;

  const log = useMemo(
    () => toClickLog(props.log, props.name, props.extraBeats, { zoomActions: zoom }),
    [props.log, props.name, props.extraBeats, zoom],
  );
  const annotations = useMemo(
    () => resolveAnnotations(props.log, props.annotations),
    [props.log, props.annotations],
  );
  const timeMs = timeS * 1000;

  if (!hasScreen(props.screen)) return null;

  const shot = { ...props.shot, backdrop: assetUrl(props.shot.backdrop) };
  const base = {
    screen: props.screen,
    log,
    speed: props.speed,
    shot,
    window: props.window,
    pointer: props.pointer,
    annotations,
  };

  if (props.shot.inDevice) return <ScreenClip {...base} shot={{ ...shot, motionBlur: false }} />;

  const layout = reframeLayout(
    props.log.viewport,
    { width, height },
    reframe,
    SHOT_STYLES[props.shot.style].framing !== "window",
  );
  const contain = layout.mode === "contain" ? layout : undefined;
  const narrow = contain?.narrow;
  const clip = (
    <DemoClip
      {...base}
      box={contain?.box}
      title={
        props.screen.kind === "terminal"
          ? terminalTitle(props.log, timeMs, props.window.text)
          : frameAddress(props.log, timeMs, props.window.text, props.window.mode)
      }
      tuning={narrow ? { ...tuning, narrow } : tuning}
    />
  );

  const { keystrokes } = props;
  const chipKeys = keystrokes?.chords ? shownKeys(log.keys ?? [], keystrokes.leftOut ?? []) : [];
  function followX() {
    return poseAt(frame, fps, log, { speed: props.speed, tuning }).cx;
  }
  return (
    <AbsoluteFill>
      <LayoutFrame layout={layout} followX={followX}>
        {clip}
      </LayoutFrame>
      <DesignWidthContext.Provider value={Math.max(width, height)}>
        {keystrokes && chipKeys.length ? (
          <KeycapHUD keys={chipKeys} timeS={timeS} look={keystrokes} />
        ) : null}
        {keystrokes?.typed ? (
          <TypedChip
            beats={typedBeats(props.log, keystrokes.typedHoldS ?? TYPED_LINGER_S)}
            timeS={timeS}
            look={keystrokes}
          />
        ) : null}
      </DesignWidthContext.Provider>
    </AbsoluteFill>
  );
}

function LayoutFrame({
  layout,
  followX,
  children,
}: {
  layout: Reframe;
  followX: () => number;
  children: React.ReactNode;
}) {
  switch (layout.mode) {
    case "same":
      return <>{children}</>;
    case "contain":
      return (
        <DesignWidthContext.Provider value={layout.box.width}>
          {children}
        </DesignWidthContext.Provider>
      );
    case "fit":
      return <Letterboxed height={layout.height}>{children}</Letterboxed>;
    case "cover":
      return (
        <Cropped box={layout.box} cx={layout.follow ? followX() : 0.5}>
          {children}
        </Cropped>
      );
  }
}

function Letterboxed({ height, children }: { height: number; children: React.ReactNode }) {
  const { width } = useFilm();
  return (
    <AbsoluteFill
      style={{ alignItems: "center", justifyContent: "center", background: STAGE_COLOR }}
    >
      <DesignWidthContext.Provider value={width}>
        <div style={{ position: "relative", width, height }}>{children}</div>
      </DesignWidthContext.Provider>
    </AbsoluteFill>
  );
}

function Cropped({ box, cx, children }: { box: Size; cx: number; children: React.ReactNode }) {
  const { width, height } = useFilm();
  return (
    <AbsoluteFill style={{ overflow: "hidden" }}>
      <DesignWidthContext.Provider value={box.width}>
        <div
          style={{
            position: "absolute",
            left: clamp(width / 2 - cx * box.width, width - box.width, 0),
            top: (height - box.height) / 2,
            width: box.width,
            height: box.height,
          }}
        >
          {children}
        </div>
      </DesignWidthContext.Provider>
    </AbsoluteFill>
  );
}

function TypedChip({
  beats,
  timeS,
  look,
}: {
  beats: Array<{ text: string; fromS: number; toS: number }>;
  timeS: number;
  look: KeystrokeLook;
}) {
  const { position } = look;
  const { k: frameScale, insets } = useOverlayFrame();
  const k = frameScale * look.size;
  const beat = beats.find((b) => timeS >= b.fromS && timeS <= b.toS);
  if (!beat) return null;
  const typingEnd = beat.toS - (look.typedHoldS ?? TYPED_LINGER_S);
  const shown = Math.round(
    interpolate(timeS, [beat.fromS, typingEnd], [0, beat.text.length], CLAMPED),
  );
  const opacity = fadeWindow(timeS, beat.fromS, beat.toS, {
    in: TYPED_FADE_IN_S,
    out: TYPED_FADE_OUT_S,
  });
  return (
    <AbsoluteFill
      style={{
        justifyContent: JUSTIFY_AT[position],
        alignItems: "center",
        ...overlayPadding((look.edge ?? KEYSTROKE_EDGE) * frameScale, verticalInsets(insets)),
        pointerEvents: "none",
      }}
    >
      <div
        style={{
          opacity,
          maxWidth: TYPED_CHIP.maxWidth,
          padding: `${TYPED_CHIP.padY * k}px ${TYPED_CHIP.padX * k}px`,
          borderRadius: 999,
          background: faded(look.keycap, TYPED_CHIP.keycapShare),
          color: look.ink,
          fontFamily: look.font || CODE_FONT,
          fontSize: TYPED_CHIP.fontSize * k,
          whiteSpace: "nowrap",
          overflow: "hidden",
          textOverflow: "ellipsis",
          boxShadow: `0 ${TYPED_CHIP.shadowY * k}px ${TYPED_CHIP.shadowBlur * k}px rgba(0,0,0,${look.shadow ?? TYPED_CHIP.shadow})`,
        }}
      >
        {beat.text.slice(0, shown)}
        {look.caret === false ? null : (
          <span style={{ opacity: Math.floor(timeS * CARET_BLINKS_PER_S) % 2 ? 0 : 1 }}>▍</span>
        )}
      </div>
    </AbsoluteFill>
  );
}

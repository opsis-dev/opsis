import React, { useMemo } from "react";
import { AbsoluteFill, interpolate, useCurrentFrame, useVideoConfig } from "remotion";
import { assetUrl, useQuality, useReframe } from "#video/assets.tsx";
import {
  resolveAnnotations,
  toClickLog,
  TYPED_LINGER_S,
  typedBeats,
} from "#video/camera/action-log.ts";
import { DemoClip, ScreenClip, useClipTime } from "#video/camera/demo-clip.tsx";
import { frameAddress } from "#video/camera/frame-address.ts";
import { KeycapHUD } from "#video/camera/keycap-hud.tsx";
import { blurEffects } from "#video/camera/lib/blur-areas.ts";
import { DesignWidthContext } from "#video/camera/lib/design-scale.ts";
import { faded, SHOT_STYLES } from "#video/camera/lib/style.ts";
import { poseAt } from "#video/camera/lib/zoom.ts";
import { STAGE_COLOR } from "#video/common/element-colors.ts";
import { CODE_FONT } from "#video/common/fonts.ts";
import { clamp } from "#video/common/math.ts";
import { pick } from "#video/common/objects.ts";
import { PREVIEW_FILE_WIDTH } from "#video/common/recording.ts";
import { CLAMPED, fadeWindow } from "#video/core/anim.ts";
import {
  JUSTIFY_AT,
  overlayPadding,
  reframeLayout,
  useOverlayFrame,
  verticalInsets,
  type Reframe,
} from "#video/core/frame.ts";
import { lookEffects } from "#video/core/picture-look.ts";
import type { Size } from "#video/core/still-layout.ts";
import type { KeystrokeLook, RecordingProps } from "#video/generated/spec.ts";

const TYPED_FADE_IN_S = 0.15;
const TYPED_FADE_OUT_S = 0.2;
const CARET_BLINKS_PER_S = 2.5;

const TYPED_CHIP = {
  margin: 56,
  padY: 14,
  padX: 26,
  fontSize: 30,
  shadowY: 8,
  shadowBlur: 24,
  maxWidth: "70%",
  keycapShare: "82%",
  shadow: "rgba(0,0,0,0.35)",
} as const;

const CLIP_KEYS = [
  "speed",
  "style",
  "chrome",
  "windowStyle",
  "frameTheme",
  "addressBar",
  "backdrop",
  "backdropChosen",
  "cursor",
  "cursorScale",
  "cursorLook",
  "ripple",
] as const satisfies ReadonlyArray<keyof RecordingProps>;

export const RecordingClip: React.FC<RecordingProps> = (props) => {
  const fast = useQuality() === "fast";
  const proxy = fast && !!props.proxySrc;
  const shown = proxy && props.proxySrc ? props.proxySrc : props.src;
  const src = shown ? assetUrl(shown) : null;
  const backdrop = props.backdrop ? assetUrl(props.backdrop) : props.backdrop;
  const { width, height, fps } = useVideoConfig();
  const frame = useCurrentFrame();
  const reframe = useReframe();
  const timeS = useClipTime(props.speed);

  const log = useMemo(
    () => toClickLog(props.log, props.name, props.extraBeats, { zoomActions: props.zoom }),
    [props.log, props.name, props.extraBeats, props.zoom],
  );
  const annotations = useMemo(
    () => resolveAnnotations(props.log, props.annotations),
    [props.log, props.annotations],
  );
  const fileWidth = proxy ? PREVIEW_FILE_WIDTH : props.log.viewport.width;
  const timeMs = timeS * 1000;
  const effects = useMemo(
    () => [
      ...blurEffects(props.blurAreas ?? [], timeMs, fileWidth),
      ...lookEffects(props.look, frame),
    ],
    [props.blurAreas, props.look, timeMs, fileWidth, frame],
  );

  if (!src) return null;

  if (props.screen)
    return (
      <ScreenClip
        {...pick(props, CLIP_KEYS)}
        backdrop={backdrop}
        src={src}
        log={log}
        motionBlur={false}
        annotations={annotations}
        effects={effects}
      />
    );

  const layout = reframeLayout(
    props.log.viewport,
    { width, height },
    reframe,
    SHOT_STYLES[props.style].framing !== "window",
  );
  const contain = layout.mode === "contain" ? layout : undefined;
  const narrow = contain?.narrow;
  const clip = (
    <DemoClip
      {...pick(props, CLIP_KEYS)}
      backdrop={backdrop}
      box={contain?.box}
      src={src}
      log={log}
      address={frameAddress(props.log, timeMs, props.address, props.addressMode)}
      motionBlur={props.motionBlur && !fast}
      tuning={narrow ? { ...props.tuning, narrow } : props.tuning}
      annotations={annotations}
      effects={effects}
    />
  );

  const { keystrokes } = props;
  const followX = () => poseAt(frame, fps, log, { speed: props.speed, tuning: props.tuning }).cx;
  return (
    <AbsoluteFill>
      <LayoutFrame layout={layout} followX={followX}>
        {clip}
      </LayoutFrame>
      <DesignWidthContext.Provider value={Math.max(width, height)}>
        {keystrokes?.chords && log.keys?.length ? (
          <KeycapHUD keys={log.keys} timeS={timeS} look={keystrokes} />
        ) : null}
        {keystrokes?.typed ? (
          <TypedChip beats={typedBeats(props.log)} timeS={timeS} look={keystrokes} />
        ) : null}
      </DesignWidthContext.Provider>
    </AbsoluteFill>
  );
};

const LayoutFrame: React.FC<{
  layout: Reframe;
  followX: () => number;
  children: React.ReactNode;
}> = ({ layout, followX, children }) => {
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
};

const Letterboxed: React.FC<{ height: number; children: React.ReactNode }> = ({
  height,
  children,
}) => {
  const { width } = useVideoConfig();
  return (
    <AbsoluteFill
      style={{ alignItems: "center", justifyContent: "center", background: STAGE_COLOR }}
    >
      <DesignWidthContext.Provider value={width}>
        <div style={{ position: "relative", width, height }}>{children}</div>
      </DesignWidthContext.Provider>
    </AbsoluteFill>
  );
};

const Cropped: React.FC<{ box: Size; cx: number; children: React.ReactNode }> = ({
  box,
  cx,
  children,
}) => {
  const { width, height } = useVideoConfig();
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
};

const TypedChip: React.FC<{
  beats: Array<{ text: string; fromS: number; toS: number }>;
  timeS: number;
  look: KeystrokeLook;
}> = ({ beats, timeS, look }) => {
  const { position } = look;
  const { k: frameScale, insets } = useOverlayFrame();
  const k = frameScale * look.size;
  const beat = beats.find((b) => timeS >= b.fromS && timeS <= b.toS);
  if (!beat) return null;
  const typingEnd = beat.toS - TYPED_LINGER_S;
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
        ...overlayPadding(TYPED_CHIP.margin * frameScale, verticalInsets(insets)),
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
          fontFamily: CODE_FONT,
          fontSize: TYPED_CHIP.fontSize * k,
          whiteSpace: "nowrap",
          overflow: "hidden",
          textOverflow: "ellipsis",
          boxShadow: `0 ${TYPED_CHIP.shadowY * k}px ${TYPED_CHIP.shadowBlur * k}px ${TYPED_CHIP.shadow}`,
        }}
      >
        {beat.text.slice(0, shown)}
        <span style={{ opacity: Math.floor(timeS * CARET_BLINKS_PER_S) % 2 ? 0 : 1 }}>▍</span>
      </div>
    </AbsoluteFill>
  );
};

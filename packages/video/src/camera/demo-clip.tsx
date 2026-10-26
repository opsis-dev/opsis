import { Video } from "@remotion/media";
import { CameraMotionBlur } from "@remotion/motion-blur";
import React from "react";
import { AbsoluteFill, useCurrentFrame, useVideoConfig, type EffectDescriptor } from "remotion";
import type { Annotation } from "#video/camera/action-log.ts";
import { Annotations } from "#video/camera/annotations.tsx";
import { Cursor, type CursorOptions } from "#video/camera/cursor.tsx";
import { isLightBackdrop } from "#video/camera/lib/backdrop.ts";
import { isNarrow, NO_TUNING, type CameraTuning } from "#video/camera/lib/camera-track.ts";
import { BASE_POSE, poseToCss, type ZoomState } from "#video/camera/lib/camera.ts";
import type { ClickLog } from "#video/camera/lib/click-log.ts";
import { useDesignScale } from "#video/camera/lib/design-scale.ts";
import { SHOT_STYLES, type WindowShot } from "#video/camera/lib/style.ts";
import { logTimeS, zoomAt } from "#video/camera/lib/zoom.ts";
import { RimLight, Stage } from "#video/camera/stage.tsx";
import { CHROME_H, WindowFrame } from "#video/camera/window-frame.tsx";
import { percent } from "#video/common/format.ts";
import { Backdrop } from "#video/core/backgrounds.tsx";
import type { Size } from "#video/core/still-layout.ts";
import type {
  CursorLook,
  RecordingProps,
  RippleLook,
  FrameTheme,
  WindowStyle,
} from "#video/generated/spec.ts";

type DemoClipProps = {
  src: string;
  log: ClickLog;
  style: RecordingProps["style"];
  speed: number;
  chrome: boolean;
  motionBlur: boolean;
  cursor: boolean;
  ripple: RippleLook | null;
  cursorScale?: number;
  cursorLook?: CursorLook;
  tuning?: CameraTuning;
  annotations?: Annotation[];
  backdrop?: string;
  backdropChosen?: boolean;
  box?: Size;
  windowStyle?: WindowStyle;
  frameTheme?: FrameTheme;
  addressBar?: boolean;
  address?: string;
  effects?: EffectDescriptor<unknown>[];
};

const FILL = { width: "100%", height: "100%" } as const;
const PAGE_BG = "#fcfcfc";
const BLEED_BG = "#0b0b10";
const BLEED_BLUR = {
  filter: "blur(48px) brightness(0.55) saturate(1.2)",
  transform: "scale(1.15)",
};
const BLEED_SHADOW = "0 30px 80px rgba(0,0,0,0.45)";
const BLUR_SHUTTER_ANGLE = 180;
const BLUR_SAMPLES = 8;

export const useClipTime = (speed: number): number => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  return logTimeS(frame, fps, speed);
};

const Footage: React.FC<{
  src: string;
  speed: number;
  fit: "cover" | "fill";
  muted?: boolean;
  effects?: EffectDescriptor<unknown>[];
}> = ({ src, speed, fit, muted, effects = [] }) => (
  <Video
    src={src}
    playbackRate={speed}
    objectFit={fit}
    muted={muted}
    effects={effects}
    style={FILL}
  />
);

const cameraTransform = (z: ZoomState) =>
  `translate(${percent(z.translateX)}, ${percent(z.translateY)}) scale(${z.scale})`;

type Framing = { chrome: boolean; fit: number; zoom: boolean };

type CameraClip = Pick<DemoClipProps, "log" | "speed" | "tuning" | "box" | "chrome">;

function useCamera(clip: CameraClip, framing: Framing) {
  const frame = useCurrentFrame();
  const { fps, height } = useVideoConfig();
  const k = useDesignScale();
  const { log, speed, tuning, box } = clip;
  const { chrome, fit, zoom } = framing;
  const chromeH = chrome ? CHROME_H * k : 0;
  const chromeFrac = chromeH / ((box?.height ?? height) + chromeH);
  const z = zoom
    ? zoomAt(frame, fps, log, { chromeFrac, speed, fit, tuning })
    : poseToCss(BASE_POSE, chromeFrac, fit);

  const style: React.CSSProperties = {
    width: box ? box.width : "100%",
    height: box ? box.height + chromeH : `calc(100% + ${chromeH}px)`,
    flexShrink: 0,
    transform: cameraTransform(z),
    position: "relative",
    willChange: "transform",
  };
  return { z, style };
}

const windowFraming = (clip: CameraClip, shot: WindowShot): Framing => ({
  chrome: clip.chrome,
  fit: shot.windowFit,
  zoom: shot.zoom,
});

const ShadowGroup: React.FC<{ clip: CameraClip; shot: WindowShot; light: boolean }> = ({
  clip,
  shot,
  light,
}) => {
  const { style } = useCamera(clip, windowFraming(clip, shot));
  return (
    <Stage>
      <div style={style}>
        <RimLight light={light} />
      </div>
    </Stage>
  );
};

type FootageClip = Pick<DemoClipProps, "src" | "speed" | "annotations" | "effects"> & CursorOptions;

const Marks: React.FC<{ clip: FootageClip; timeS: number; cameraScale: number }> = ({
  clip,
  timeS,
  cameraScale,
}) =>
  clip.annotations?.length ? (
    <Annotations
      annotations={clip.annotations}
      viewport={clip.log.viewport}
      timeS={timeS}
      cameraScale={cameraScale}
    />
  ) : null;

const WindowGroup: React.FC<{ clip: DemoClipProps; shot: WindowShot }> = ({ clip, shot }) => {
  const { z, style } = useCamera(clip, windowFraming(clip, shot));
  const timeS = useClipTime(clip.speed);

  return (
    <Stage>
      <div style={style}>
        <WindowFrame
          chrome={clip.chrome}
          address={clip.address}
          windowStyle={clip.windowStyle}
          theme={clip.frameTheme}
          addressBar={clip.addressBar}
        >
          <Footage src={clip.src} speed={clip.speed} fit="fill" effects={clip.effects} />
          <Marks clip={clip} timeS={timeS} cameraScale={z.scale} />
          <Cursor clip={clip} timeS={timeS} cameraScale={z.scale} />
        </WindowFrame>
      </div>
    </Stage>
  );
};

const FullBleedClip: React.FC<{ clip: FootageClip }> = ({ clip }) => {
  const timeS = useClipTime(clip.speed);
  return (
    <AbsoluteFill style={{ overflow: "hidden", backgroundColor: PAGE_BG }}>
      <Footage src={clip.src} speed={clip.speed} fit="cover" effects={clip.effects} />
      <Marks clip={clip} timeS={timeS} cameraScale={1} />
      <Cursor clip={clip} timeS={timeS} cameraScale={1} />
    </AbsoluteFill>
  );
};

const NarrowBleedClip: React.FC<{ clip: FootageClip & CameraClip }> = ({ clip }) => {
  const { z, style } = useCamera(clip, {
    chrome: false,
    fit: 1,
    zoom: isNarrow(clip.tuning ?? NO_TUNING),
  });
  const timeS = useClipTime(clip.speed);
  return (
    <AbsoluteFill style={{ overflow: "hidden", backgroundColor: BLEED_BG }}>
      <AbsoluteFill style={BLEED_BLUR}>
        <Footage src={clip.src} speed={clip.speed} fit="cover" muted effects={clip.effects} />
      </AbsoluteFill>
      <Stage>
        <div style={{ ...style, boxShadow: BLEED_SHADOW }}>
          <Footage src={clip.src} speed={clip.speed} fit="fill" muted effects={clip.effects} />
          <Marks clip={clip} timeS={timeS} cameraScale={z.scale} />
          <Cursor clip={clip} timeS={timeS} cameraScale={z.scale} />
        </div>
      </Stage>
    </AbsoluteFill>
  );
};

export const ScreenClip: React.FC<DemoClipProps> = (clip) => {
  const shot = SHOT_STYLES[clip.style];
  const zoom = shot.framing === "window" && shot.zoom;
  const { z, style } = useCamera(clip, { chrome: false, fit: 1, zoom });
  const timeS = useClipTime(clip.speed);
  const { width, height } = clip.log.viewport;
  const aspect = width / height;
  return (
    <AbsoluteFill style={{ overflow: "hidden", containerType: "size", backgroundColor: PAGE_BG }}>
      <div
        style={{
          position: "absolute",
          left: "50%",
          top: "50%",
          width: `max(100cqw, calc(100cqh * ${aspect}))`,
          height: `max(100cqh, calc(100cqw / ${aspect}))`,
          transform: "translate(-50%, -50%)",
        }}
      >
        <div style={style}>
          <Footage src={clip.src} speed={clip.speed} fit="fill" effects={clip.effects} />
          <Marks clip={clip} timeS={timeS} cameraScale={z.scale} />
          <Cursor clip={clip} timeS={timeS} cameraScale={z.scale} />
        </div>
      </div>
    </AbsoluteFill>
  );
};

export const DemoClip: React.FC<DemoClipProps> = (props) => {
  const shot = SHOT_STYLES[props.style];
  if (shot.framing !== "window")
    return props.box ? <NarrowBleedClip clip={props} /> : <FullBleedClip clip={props} />;

  const backdrop = props.backdropChosen ? props.backdrop : (shot.ground ?? props.backdrop);
  const windowGroup = <WindowGroup clip={props} shot={shot} />;
  return (
    <AbsoluteFill>
      <Backdrop name={backdrop} />
      <ShadowGroup clip={props} shot={shot} light={isLightBackdrop(backdrop)} />
      {shot.zoom && props.motionBlur ? (
        <CameraMotionBlur shutterAngle={BLUR_SHUTTER_ANGLE} samples={BLUR_SAMPLES}>
          {windowGroup}
        </CameraMotionBlur>
      ) : (
        windowGroup
      )}
    </AbsoluteFill>
  );
};

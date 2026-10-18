import React from "react";
import { AbsoluteFill, useCurrentFrame, useVideoConfig } from "remotion";
import { Video } from "@remotion/media";
import { CameraMotionBlur } from "@remotion/motion-blur";
import type { ClickLog } from "./lib/click-log";
import { isNarrow, logTimeS, zoomAt, type CameraTuning } from "./lib/zoom";
import { BASE_POSE, poseToCss, type ZoomState } from "./lib/camera";
import { isLightBackdrop } from "./lib/backdrop";
import { useDesignScale } from "./lib/design-scale";
import { SHOT_STYLES, type WindowShot } from "./lib/style";
import { Cursor } from "./cursor";
import {
  CHROME_H,
  RimLight,
  Stage,
  WindowFrame,
  type FrameTheme,
  type WindowStyle,
} from "./window-frame";
import { Annotations } from "./annotations";
import type { Annotation } from "./action-log";
import { Backdrop } from "../core/backgrounds";
import type { Size } from "../core/still-layout";
import type { RecordingProps } from "../../shared/compile/recording";
import { percent } from "../../shared/format";

type DemoClipProps = {
  src: string;
  log: ClickLog;
  style: RecordingProps["style"];
  speed: number;
  chrome: boolean;
  motionBlur: boolean;
  cursor: boolean;
  ripple: boolean;
  cursorScale?: number;
  tuning?: CameraTuning;
  annotations?: Annotation[];
  backdrop?: string;
  box?: Size;
  windowStyle?: WindowStyle;
  frameTheme?: FrameTheme;
  addressBar?: boolean;
  address?: string;
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

const useClipTime = (speed: number): number => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  return logTimeS(frame, fps, speed);
};

const Footage: React.FC<{ src: string; speed: number; fit: "cover" | "fill"; muted?: boolean }> = ({
  src,
  speed,
  fit,
  muted,
}) => <Video src={src} playbackRate={speed} objectFit={fit} muted={muted} style={FILL} />;

const cameraTransform = (z: ZoomState) =>
  `translate(${percent(z.translateX)}, ${percent(z.translateY)}) scale(${z.scale})`;

type Framing = { chrome: boolean; fit: number; zoom: boolean };

function useCamera(
  clip: Pick<DemoClipProps, "log" | "speed" | "tuning" | "box">,
  framing: Framing,
) {
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

const windowFraming = (clip: DemoClipProps, shot: WindowShot): Framing => ({
  chrome: clip.chrome,
  fit: shot.windowFit,
  zoom: shot.zoom,
});

const ShadowGroup: React.FC<{ clip: DemoClipProps; shot: WindowShot; light: boolean }> = ({
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

type CursorOptions = Pick<DemoClipProps, "log" | "cursor" | "ripple" | "cursorScale">;

const CursorLayer: React.FC<{ clip: CursorOptions; timeS: number; cameraScale: number }> = ({
  clip,
  timeS,
  cameraScale,
}) =>
  clip.cursor ? (
    <Cursor
      log={clip.log}
      timeS={timeS}
      cameraScale={cameraScale}
      ripple={clip.ripple}
      scale={clip.cursorScale}
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
          hostLabel={clip.address}
          windowStyle={clip.windowStyle}
          theme={clip.frameTheme}
          addressBar={clip.addressBar}
        >
          <Footage src={clip.src} speed={clip.speed} fit="fill" />
          {clip.annotations?.length ? (
            <Annotations
              annotations={clip.annotations}
              viewport={clip.log.viewport}
              timeS={timeS}
              cameraScale={z.scale}
            />
          ) : null}
          <CursorLayer clip={clip} timeS={timeS} cameraScale={z.scale} />
        </WindowFrame>
      </div>
    </Stage>
  );
};

const FullBleedClip: React.FC<DemoClipProps> = (clip) => {
  const timeS = useClipTime(clip.speed);
  return (
    <AbsoluteFill style={{ overflow: "hidden", backgroundColor: PAGE_BG }}>
      <Footage src={clip.src} speed={clip.speed} fit="cover" />
      <CursorLayer clip={clip} timeS={timeS} cameraScale={1} />
    </AbsoluteFill>
  );
};

const NarrowBleedClip: React.FC<DemoClipProps> = (clip) => {
  const { z, style } = useCamera(clip, {
    chrome: false,
    fit: 1,
    zoom: isNarrow(clip.tuning),
  });
  const timeS = useClipTime(clip.speed);
  return (
    <AbsoluteFill style={{ overflow: "hidden", backgroundColor: BLEED_BG }}>
      <AbsoluteFill style={BLEED_BLUR}>
        <Footage src={clip.src} speed={clip.speed} fit="cover" muted />
      </AbsoluteFill>
      <Stage>
        <div style={{ ...style, boxShadow: BLEED_SHADOW }}>
          <Footage src={clip.src} speed={clip.speed} fit="fill" muted />
          <CursorLayer clip={clip} timeS={timeS} cameraScale={z.scale} />
        </div>
      </Stage>
    </AbsoluteFill>
  );
};

export const DemoClip: React.FC<DemoClipProps> = (props) => {
  const shot = SHOT_STYLES[props.style];
  if (shot.framing !== "window")
    return props.box ? <NarrowBleedClip {...props} /> : <FullBleedClip {...props} />;

  const backdrop = shot.ground ?? props.backdrop;
  const windowGroup = <WindowGroup clip={props} shot={shot} />;
  return (
    <AbsoluteFill>
      <Backdrop name={backdrop} />
      <ShadowGroup clip={props} shot={shot} light={isLightBackdrop(backdrop ?? "")} />
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

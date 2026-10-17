import React from "react";
import {
  AbsoluteFill,
  getRemotionEnvironment,
  Img,
  staticFile,
  useCurrentFrame,
  useVideoConfig,
} from "remotion";
import { Video } from "@remotion/media";
import { CameraMotionBlur } from "@remotion/motion-blur";
import type { ClickLog } from "./lib/click-log";
import { zoomAt, type CameraTuning } from "./lib/zoom";
import { BASE_POSE, poseToCss } from "./lib/camera";
import { backdropSource, isLightBackdrop } from "./lib/backdrop";
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
import type { Size } from "../core/still-layout";
import type { RecordingProps } from "../../shared/compile/recording";

export type DemoClipProps = {
  name: string;
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

const BackdropImage: React.FC<{ src: string }> = ({ src }) => {
  const style = { ...FILL, objectFit: "cover" } as const;
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

const useClipTime = (speed: number): number => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  return (frame / fps) * speed;
};

const Footage: React.FC<{ src: string; speed: number; fit: "cover" | "fill"; muted?: boolean }> = ({
  src,
  speed,
  fit,
  muted,
}) => <Video src={src} playbackRate={speed} objectFit={fit} muted={muted} style={FILL} />;

const cameraTransform = (z: { scale: number; translateX: number; translateY: number }) =>
  `translate(${z.translateX * 100}%, ${z.translateY * 100}%) scale(${z.scale})`;

type CameraGroup = Pick<DemoClipProps, "log" | "speed" | "chrome" | "tuning" | "box"> & {
  shot: WindowShot;
};

function useCameraGroup({ log, speed, chrome, tuning, box, shot }: CameraGroup) {
  const frame = useCurrentFrame();
  const { fps, height } = useVideoConfig();
  const k = useDesignScale();
  const chromeH = chrome ? CHROME_H * k : 0;
  const chromeFrac = chromeH / ((box?.height ?? height) + chromeH);
  const fit = shot.windowFit;
  const z = shot.zoom
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

const ShadowGroup: React.FC<{ group: CameraGroup; light: boolean }> = ({ group, light }) => {
  const { style } = useCameraGroup(group);
  return (
    <Stage>
      <div style={style}>
        <RimLight light={light} />
      </div>
    </Stage>
  );
};

const ClipCursor: React.FC<{ clip: DemoClipProps; timeS: number; cameraScale: number }> = ({
  clip,
  timeS,
  cameraScale,
}) => (
  <Cursor
    log={clip.log}
    timeS={timeS}
    cameraScale={cameraScale}
    ripple={clip.ripple}
    scale={clip.cursorScale}
  />
);

const WindowGroup: React.FC<{ clip: DemoClipProps; group: CameraGroup }> = ({ clip, group }) => {
  const { z, style } = useCameraGroup(group);
  const timeS = useClipTime(clip.speed);

  return (
    <Stage>
      <div style={style}>
        <WindowFrame
          chrome={clip.chrome}
          hostLabel={clip.address ?? clip.name.replace(/-/g, ".")}
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
          {clip.cursor ? <ClipCursor clip={clip} timeS={timeS} cameraScale={z.scale} /> : null}
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
      {clip.cursor ? (
        <AbsoluteFill>
          <ClipCursor clip={clip} timeS={timeS} cameraScale={1} />
        </AbsoluteFill>
      ) : null}
    </AbsoluteFill>
  );
};

const NarrowBleedClip: React.FC<DemoClipProps & { box: Size }> = (clip) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const timeS = useClipTime(clip.speed);
  const z =
    (clip.tuning?.narrow ?? 1) > 1
      ? zoomAt(frame, fps, clip.log, { speed: clip.speed, fit: 1, tuning: clip.tuning })
      : { scale: 1, translateX: 0, translateY: 0 };
  return (
    <AbsoluteFill style={{ overflow: "hidden", backgroundColor: BLEED_BG }}>
      <AbsoluteFill style={BLEED_BLUR}>
        <Footage src={clip.src} speed={clip.speed} fit="cover" muted />
      </AbsoluteFill>
      <Stage>
        <div
          style={{
            position: "relative",
            flexShrink: 0,
            width: clip.box.width,
            height: clip.box.height,
            transform: cameraTransform(z),
            willChange: "transform",
            boxShadow: BLEED_SHADOW,
          }}
        >
          <Footage src={clip.src} speed={clip.speed} fit="fill" muted />
          {clip.cursor ? (
            <AbsoluteFill>
              <ClipCursor clip={clip} timeS={timeS} cameraScale={z.scale} />
            </AbsoluteFill>
          ) : null}
        </div>
      </Stage>
    </AbsoluteFill>
  );
};

export const DemoClip: React.FC<DemoClipProps> = (props) => {
  const shot = SHOT_STYLES[props.style];
  if (shot.framing !== "window")
    return props.box ? (
      <NarrowBleedClip {...props} box={props.box} />
    ) : (
      <FullBleedClip {...props} />
    );

  const backdrop = shot.ground ?? props.backdrop;
  const group: CameraGroup = { ...props, shot };
  const windowGroup = <WindowGroup clip={props} group={group} />;
  return (
    <AbsoluteFill>
      <Backdrop name={backdrop} />
      <ShadowGroup group={group} light={isLightBackdrop(backdrop ?? "")} />
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

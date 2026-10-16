import React from "react";
import {
  AbsoluteFill,
  Freeze,
  getRemotionEnvironment,
  Img,
  staticFile,
  useCurrentFrame,
  useVideoConfig,
} from "remotion";
import { Video } from "@remotion/media";
import { CameraMotionBlur } from "@remotion/motion-blur";
import { zoomAt, type ClickLog } from "./lib/zoom";
import { BASE_POSE, poseToCss } from "./lib/camera";
import { CHROME_H, WINDOW_FIT } from "./lib/window";
import { backdropFile, isLightBackdrop, isLightGround } from "./lib/backdrop";
import { Cursor } from "./cursor";
import {
  RimLight,
  Stage,
  useDesignScale,
  WindowFrame,
  type FrameTheme,
  type WindowStyle,
} from "./window-frame";
import { pushEnvelope, pushToCss, type PushSpec } from "./lib/push";
import { cropClipPath, resolveCrop, type CropSpec, type ResolvedCrop } from "./lib/crop";
import { KeycapHUD } from "./keycap-hud";
import type { ReelLook } from "./lib/look";
import { resolvePreset, type ReelStyle } from "./lib/style";
import type { CameraTuning } from "./lib/zoom";
import { Annotations, type Annotation } from "./annotations";

const round = (n: number) => Math.round(n);

export type DemoClipProps = {
  name: string;
  src: string;
  log: ClickLog;
  tuning?: CameraTuning;
  motionBlur?: boolean;
  cursorScale?: number;
  annotations?: Annotation[];
  chrome?: boolean;
  speed?: number;
  drift?: number;
  backdrop?: string;
  look?: ReelLook;
  style?: ReelStyle;
  crop?: CropSpec;
  push?: PushSpec;
  range?: { first: number; last: number };
  pageBg?: string;
  cursor?: boolean;
  ripple?: boolean;
  freeze?: boolean;
  box?: { width: number; height: number };
  boost?: number;
  windowStyle?: WindowStyle;
  frameTheme?: FrameTheme;
  addressBar?: boolean;
  address?: string;
};

const BackdropImage: React.FC<{ src: string }> = ({ src }) => {
  const style = { width: "100%", height: "100%", objectFit: "cover" } as const;
  return getRemotionEnvironment().isRendering ? (
    <Img src={src} style={style} />
  ) : (
    <img src={src} alt="" decoding="async" style={style} />
  );
};

export const Backdrop: React.FC<{ name?: string; ground?: string | null }> = ({ name, ground }) =>
  ground ? (
    <AbsoluteFill style={{ backgroundColor: ground }} />
  ) : name &&
    /^(#|rgb|hsl|oklch|linear-gradient|radial-gradient|conic-gradient)/.test(name.trim()) ? (
    <AbsoluteFill style={{ background: name }} />
  ) : name && /^(https?:|demoflow-file:|data:|blob:)/.test(name) ? (
    <AbsoluteFill>
      <BackdropImage src={name} />
    </AbsoluteFill>
  ) : (
    <AbsoluteFill>
      <BackdropImage src={staticFile(backdropFile(name))} />
    </AbsoluteFill>
  );

function useCameraGroup(
  chrome: boolean,
  log: ClickLog,
  speed: number,
  drift: number | undefined,
  shot: { windowFit: number | null; zoom: boolean },
  tuning?: CameraTuning,
  box?: { width: number; height: number },
  boost?: number,
) {
  const frame = useCurrentFrame();
  const { fps, height } = useVideoConfig();
  const chromeH = CHROME_H * useDesignScale();
  const windowH = box?.height ?? height;
  const chromeFrac = chrome ? chromeH / (windowH + chromeH) : 0;
  const fit = shot.windowFit ?? WINDOW_FIT;
  const z = shot.zoom
    ? zoomAt(frame, fps, log, { chromeFrac, speed, fit, drift, tuning, boost })
    : poseToCss(BASE_POSE, chromeFrac, fit);

  const style: React.CSSProperties = {
    width: box ? box.width : "100%",
    height: box ? box.height + (chrome ? chromeH : 0) : `calc(100% + ${chrome ? chromeH : 0}px)`,
    flexShrink: 0,
    transform: `translate(${z.translateX * 100}%, ${z.translateY * 100}%) scale(${z.scale})`,
    position: "relative",
    willChange: "transform",
  };
  return { z, frame, fps, style };
}

const ShadowGroup: React.FC<DemoClipProps> = (props) => {
  const { log, chrome = false, speed = 1, drift, backdrop } = props;
  const { style } = useCameraGroup(
    chrome,
    log,
    speed,
    drift,
    resolvePreset(props).shot,
    props.tuning,
    props.box,
    props.boost,
  );
  const ground = resolvePreset(props).shot.ground;
  const light = ground ? isLightGround(ground) : isLightBackdrop(backdrop ?? "");
  return (
    <Stage>
      <div style={style}>
        <RimLight light={light} />
      </div>
    </Stage>
  );
};

const WindowGroup: React.FC<DemoClipProps> = (props) => {
  const { name, src, log, chrome = false, speed = 1, drift, ripple } = props;
  const { shot } = resolvePreset(props);
  const { z, frame, fps, style } = useCameraGroup(
    chrome,
    log,
    speed,
    drift,
    shot,
    props.tuning,
    props.box,
    props.boost,
  );
  const trimBefore = log.trimBeforeMs ? round((log.trimBeforeMs / 1000) * fps) : undefined;
  const hostLabel = props.address ?? name.replace(/-/g, ".");

  return (
    <Stage>
      <div style={style}>
        <WindowFrame
          chrome={chrome}
          hostLabel={hostLabel}
          windowStyle={props.windowStyle}
          theme={props.frameTheme}
          addressBar={props.addressBar}
        >
          <Video
            src={src}
            trimBefore={trimBefore}
            playbackRate={speed}
            objectFit="fill"
            style={{ width: "100%", height: "100%" }}
          />
          {props.annotations?.length ? (
            <Annotations
              annotations={props.annotations}
              viewport={log.viewport}
              timeS={(frame / fps) * speed - (log.offsetMs ?? 0) / 1000}
              cameraScale={z.scale}
            />
          ) : null}
          {props.cursor === false ? null : (
            <Cursor
              log={log}
              timeS={(frame / fps) * speed - (log.offsetMs ?? 0) / 1000}
              cameraScale={z.scale}
              ripple={ripple ?? true}
              scale={props.cursorScale}
            />
          )}
        </WindowFrame>
      </div>
    </Stage>
  );
};

const cropToCss = (crop: ResolvedCrop | undefined): React.CSSProperties | undefined =>
  crop
    ? {
        transform: `translate(${crop.dx * 100}%, ${crop.dy * 100}%) scale(${crop.k})`,
        transformOrigin: `${crop.cx * 100}% ${crop.cy * 100}%`,
      }
    : undefined;

const FullBleedClip: React.FC<DemoClipProps> = ({
  src,
  log,
  speed = 1,
  crop,
  push,
  range,
  pageBg = "#fcfcfc",
  cursor = false,
  ripple,
  freeze = false,
  cursorScale,
}) => {
  const frame = useCurrentFrame();
  const { fps, durationInFrames } = useVideoConfig();
  const designScale = useDesignScale();

  const first = range?.first ?? 0;
  const shotFrames = range ? range.last - range.first + 1 : durationInFrames;
  const envelope = pushEnvelope(frame - first, shotFrames, push);

  const trimBefore = log.trimBeforeMs ? round((log.trimBeforeMs / 1000) * fps) : undefined;
  const timeS = (frame / fps) * speed - (log.offsetMs ?? 0) / 1000;
  const resolved = resolveCrop(crop);
  const clipPath = cropClipPath(crop);
  const cropStyle = { ...cropToCss(resolved), ...(clipPath ? { clipPath } : {}) };

  const footage = (
    <Video
      src={src}
      trimBefore={trimBefore}
      playbackRate={speed}
      objectFit="cover"
      style={{ width: "100%", height: "100%" }}
    />
  );

  return (
    <AbsoluteFill style={{ overflow: "hidden", backgroundColor: pageBg }}>
      <AbsoluteFill
        style={{
          transform: pushToCss(envelope, designScale),
          willChange: "transform",
        }}
      >
        <AbsoluteFill style={cropStyle}>
          {freeze && range ? <Freeze frame={range.last}>{footage}</Freeze> : footage}
        </AbsoluteFill>
      </AbsoluteFill>
      {cursor && !freeze ? (
        <AbsoluteFill style={cropStyle}>
          <Cursor
            log={log}
            timeS={timeS}
            cameraScale={resolved?.k ?? 1}
            ripple={ripple ?? false}
            scale={cursorScale}
          />
        </AbsoluteFill>
      ) : null}
      {log.keys?.length ? <KeycapHUD keys={log.keys} timeS={timeS} /> : null}
    </AbsoluteFill>
  );
};

const NarrowBleedClip: React.FC<DemoClipProps & { box: { width: number; height: number } }> = ({
  src,
  log,
  speed = 1,
  box,
  boost,
  tuning,
  cursor = false,
  ripple,
  cursorScale,
}) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const trimBefore = log.trimBeforeMs ? round((log.trimBeforeMs / 1000) * fps) : undefined;
  const timeS = (frame / fps) * speed - (log.offsetMs ?? 0) / 1000;
  const z =
    boost && boost > 1
      ? zoomAt(frame, fps, log, { speed, fit: 1, tuning, boost })
      : { scale: 1, translateX: 0, translateY: 0 };
  const video = (fit: "cover" | "fill") => (
    <Video
      src={src}
      trimBefore={trimBefore}
      playbackRate={speed}
      objectFit={fit}
      muted
      style={{ width: "100%", height: "100%" }}
    />
  );
  return (
    <AbsoluteFill style={{ overflow: "hidden", backgroundColor: "#0b0b10" }}>
      <AbsoluteFill
        style={{ filter: "blur(48px) brightness(0.55) saturate(1.2)", transform: "scale(1.15)" }}
      >
        {video("cover")}
      </AbsoluteFill>
      <Stage>
        <div
          style={{
            position: "relative",
            flexShrink: 0,
            width: box.width,
            height: box.height,
            transform: `translate(${z.translateX * 100}%, ${z.translateY * 100}%) scale(${z.scale})`,
            willChange: "transform",
            boxShadow: "0 30px 80px rgba(0,0,0,0.45)",
          }}
        >
          {video("fill")}
          {cursor ? (
            <AbsoluteFill>
              <Cursor
                log={log}
                timeS={timeS}
                cameraScale={z.scale}
                ripple={ripple ?? false}
                scale={cursorScale}
              />
            </AbsoluteFill>
          ) : null}
        </div>
      </Stage>
      {log.keys?.length ? <KeycapHUD keys={log.keys} timeS={timeS} /> : null}
    </AbsoluteFill>
  );
};

export const DemoClip: React.FC<DemoClipProps> = (props) => {
  const chrome = props.chrome === true;

  const { shot } = resolvePreset(props);
  if (shot.framing !== "window")
    return props.box ? (
      <NarrowBleedClip {...props} box={props.box} />
    ) : (
      <FullBleedClip {...props} />
    );

  return (
    <AbsoluteFill>
      <Backdrop name={props.backdrop} ground={resolvePreset(props).shot.ground} />
      <FramedPush {...props}>
        <ShadowGroup {...props} chrome={chrome} />
        {shot.zoom && props.motionBlur !== false ? (
          <CameraMotionBlur shutterAngle={180} samples={8}>
            <WindowGroup {...props} chrome={chrome} />
          </CameraMotionBlur>
        ) : (
          <WindowGroup {...props} chrome={chrome} />
        )}
      </FramedPush>
    </AbsoluteFill>
  );
};

const FramedPush: React.FC<DemoClipProps & { children: React.ReactNode }> = ({
  push,
  range,
  children,
}) => {
  const frame = useCurrentFrame();
  const { durationInFrames } = useVideoConfig();
  const designScale = useDesignScale();
  if (!push) return <>{children}</>;
  const first = range?.first ?? 0;
  const shotFrames = range ? range.last - range.first + 1 : durationInFrames;
  const envelope = pushEnvelope(frame - first, shotFrames, push);
  return (
    <AbsoluteFill style={{ transform: pushToCss(envelope, designScale) }}>{children}</AbsoluteFill>
  );
};

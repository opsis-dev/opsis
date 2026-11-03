import { useFilm, useFrame } from "@opsis/runtime/clock.tsx";
import { AbsoluteFill } from "@opsis/runtime/parts.tsx";
import React from "react";
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
import { MotionBlur } from "#video/camera/motion-blur.tsx";
import { RimLight, Stage } from "#video/camera/stage.tsx";
import { CHROME_H, WindowFrame } from "#video/camera/window-frame.tsx";
import { percent } from "#video/common/format.ts";
import { Screen } from "#video/core/screen.tsx";
import type { Size } from "#video/core/still-layout.ts";
import { Backdrop } from "#video/elements/background/backdrop.tsx";
import type { PointerProps, ScreenProps, ShotProps, WindowProps } from "#video/generated/spec.ts";

export type DemoClipProps = {
  screen: ScreenProps;
  log: ClickLog;
  speed: number;
  shot: ShotProps;
  window: WindowProps;
  title?: string;
  pointer: PointerProps;
  tuning?: CameraTuning;
  annotations?: Annotation[];
  box?: Size;
};

const PAGE_BG = "#fcfcfc";
const BLEED_BG = "#0b0b10";
function bleedBlur(shot: ShotProps): React.CSSProperties {
  return {
    filter: `blur(${shot.bleedBlur ?? 48}px) brightness(${shot.bleedDim ?? 0.55}) saturate(1.2)`,
    transform: "scale(1.15)",
  };
}
const BLEED_SHADOW = "0 30px 80px rgba(0,0,0,0.45)";
const BLUR_SHUTTER_ANGLE = 180;
const BLUR_SAMPLES = 8;

export function useClipTime(speed: number): number {
  const frame = useFrame();
  const { fps } = useFilm();
  return logTimeS(frame, fps, speed);
}

function ClipScreen({ clip, fit }: { clip: FootageClip; fit: "cover" | "fill" }) {
  return <Screen screen={clip.screen} timeS={useClipTime(clip.speed)} fit={fit} />;
}

function cameraTransform(z: ZoomState) {
  return `translate(${percent(z.translateX)}, ${percent(z.translateY)}) scale(${z.scale})`;
}

type Framing = { chrome: boolean; fit: number; zoom: boolean };

type CameraClip = Pick<DemoClipProps, "log" | "speed" | "tuning" | "box" | "window" | "shot">;

function useCamera(clip: CameraClip, framing: Framing) {
  const frame = useFrame();
  const { fps, height } = useFilm();
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

function windowFraming(clip: CameraClip, shot: WindowShot): Framing {
  return {
    chrome: clip.window.chrome,
    fit: clip.shot.windowFit ?? shot.windowFit,
    zoom: shot.zoom,
  };
}

function ShadowGroup({
  clip,
  shot,
  light,
}: {
  clip: CameraClip;
  shot: WindowShot;
  light: boolean;
}) {
  const { style } = useCamera(clip, windowFraming(clip, shot));
  return (
    <Stage>
      <div style={style}>
        <RimLight light={light} strength={clip.shot.rimGlow ?? 1} />
      </div>
    </Stage>
  );
}

type FootageClip = Pick<DemoClipProps, "screen" | "speed" | "annotations"> & CursorOptions;

function Marks({
  clip,
  timeS,
  cameraScale,
}: {
  clip: FootageClip;
  timeS: number;
  cameraScale: number;
}) {
  return clip.annotations?.length ? (
    <Annotations
      annotations={clip.annotations}
      viewport={clip.log.viewport}
      timeS={timeS}
      cameraScale={cameraScale}
    />
  ) : null;
}

function WindowGroup({ clip, shot }: { clip: DemoClipProps; shot: WindowShot }) {
  const { z, style } = useCamera(clip, windowFraming(clip, shot));
  const timeS = useClipTime(clip.speed);

  return (
    <Stage>
      <div style={style}>
        <WindowFrame
          chrome={clip.window.chrome}
          address={clip.title}
          windowStyle={clip.window.style}
          theme={clip.window.theme}
          addressBar={clip.window.bar}
          radius={clip.shot.radius}
          lift={clip.shot.liftColours ?? true}
          font={clip.window.font}
        >
          <ClipScreen clip={clip} fit="fill" />
          <Marks clip={clip} timeS={timeS} cameraScale={z.scale} />
          <Cursor clip={clip} timeS={timeS} cameraScale={z.scale} />
        </WindowFrame>
      </div>
    </Stage>
  );
}

function FullBleedClip({ clip }: { clip: FootageClip }) {
  const timeS = useClipTime(clip.speed);
  return (
    <AbsoluteFill style={{ overflow: "hidden", backgroundColor: PAGE_BG }}>
      <ClipScreen clip={clip} fit="cover" />
      <Marks clip={clip} timeS={timeS} cameraScale={1} />
      <Cursor clip={clip} timeS={timeS} cameraScale={1} />
    </AbsoluteFill>
  );
}

function NarrowBleedClip({ clip }: { clip: DemoClipProps }) {
  const { z, style } = useCamera(clip, {
    chrome: false,
    fit: 1,
    zoom: isNarrow(clip.tuning ?? NO_TUNING),
  });
  const timeS = useClipTime(clip.speed);
  return (
    <AbsoluteFill style={{ overflow: "hidden", backgroundColor: BLEED_BG }}>
      <AbsoluteFill style={bleedBlur(clip.shot)}>
        <ClipScreen clip={clip} fit="cover" />
      </AbsoluteFill>
      <Stage>
        <div style={{ ...style, boxShadow: BLEED_SHADOW }}>
          <ClipScreen clip={clip} fit="fill" />
          <Marks clip={clip} timeS={timeS} cameraScale={z.scale} />
          <Cursor clip={clip} timeS={timeS} cameraScale={z.scale} />
        </div>
      </Stage>
    </AbsoluteFill>
  );
}

export function ScreenClip(clip: DemoClipProps) {
  const shot = SHOT_STYLES[clip.shot.style];
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
          <ClipScreen clip={clip} fit="fill" />
          <Marks clip={clip} timeS={timeS} cameraScale={z.scale} />
          <Cursor clip={clip} timeS={timeS} cameraScale={z.scale} />
        </div>
      </div>
    </AbsoluteFill>
  );
}

export function DemoClip(props: DemoClipProps) {
  const shot = SHOT_STYLES[props.shot.style];
  if (shot.framing !== "window")
    return props.box ? <NarrowBleedClip clip={props} /> : <FullBleedClip clip={props} />;

  const ground = shot.ground && (props.shot.ground ?? shot.ground);
  const backdrop = props.shot.backdropChosen ? props.shot.backdrop : ground || props.shot.backdrop;
  return (
    <AbsoluteFill>
      <Backdrop name={backdrop} />
      <ShadowGroup clip={props} shot={shot} light={isLightBackdrop(backdrop)} />
      {shot.zoom && props.shot.motionBlur ? (
        <MotionBlur
          shutterAngle={props.shot.shutterAngle ?? BLUR_SHUTTER_ANGLE}
          samples={BLUR_SAMPLES}
        >
          <WindowGroup clip={props} shot={shot} />
        </MotionBlur>
      ) : (
        <WindowGroup clip={props} shot={shot} />
      )}
    </AbsoluteFill>
  );
}

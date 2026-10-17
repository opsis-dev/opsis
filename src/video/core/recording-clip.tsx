import React, { useMemo } from "react";
import { AbsoluteFill, interpolate, useCurrentFrame, useVideoConfig } from "remotion";
import type { RecordingProps } from "../../shared/compile/recording";
import { roleStack } from "../../shared/fonts";
import { displayAddress } from "../../shared/format";
import { clamp } from "../../shared/math";
import { useAsset, useQuality, useReframe } from "../assets";
import { resolveAnnotations, toClickLog, TYPED_LINGER_S, typedBeats } from "../camera/action-log";
import { DemoClip } from "../camera/demo-clip";
import { KeycapHUD } from "../camera/keycap-hud";
import { DesignWidthContext } from "../camera/lib/design-scale";
import { SHOT_STYLES } from "../camera/lib/style";
import { poseAt } from "../camera/lib/zoom";
import { CLAMPED } from "./anim";
import { reframeLayout, useOverlayFrame } from "./frame";
import type { Size } from "./still-layout";

const CODE_FONT = roleStack("code");
const TYPED_FADE_IN_S = 0.15;
const TYPED_FADE_OUT_S = 0.2;
const CARET_BLINKS_PER_S = 2.5;

const passedToClip = ({
  name,
  speed,
  style,
  chrome,
  windowStyle,
  frameTheme,
  addressBar,
  backdrop,
  cursor,
  cursorScale,
  ripple,
}: RecordingProps) => ({
  name,
  speed,
  style,
  chrome,
  windowStyle,
  frameTheme,
  addressBar,
  backdrop,
  cursor,
  cursorScale,
  ripple,
});

export const RecordingClip: React.FC<RecordingProps> = (props) => {
  const fast = useQuality() === "fast";
  const src = useAsset(fast && props.proxySrc ? props.proxySrc : props.src);
  const { width, height, fps } = useVideoConfig();
  const frame = useCurrentFrame();
  const reframe = useReframe();

  const log = useMemo(() => {
    const l = toClickLog(props.log, props.name, props.extraBeats);
    return props.zoom ? l : { ...l, clicks: l.clicks.map((c) => ({ ...c, zoom: false })) };
  }, [props.log, props.name, props.extraBeats, props.zoom]);
  const annotations = useMemo(
    () => resolveAnnotations(props.log, props.annotations),
    [props.log, props.annotations],
  );
  const typed = useMemo(() => typedBeats(props.log), [props.log]);
  const address = useMemo(() => pageAddress(props.log), [props.log]);

  if (!src) return null;

  const layout = reframeLayout(
    props.log.viewport,
    { width, height },
    reframe,
    SHOT_STYLES[props.style].framing !== "window",
  );
  const narrow = layout.mode === "contain" ? layout.narrow : undefined;
  const clip = (
    <DemoClip
      {...passedToClip(props)}
      box={layout.mode === "contain" ? layout.box : undefined}
      src={src}
      log={log}
      address={address}
      motionBlur={props.motionBlur && !fast}
      tuning={narrow ? { ...props.tuning, narrow } : props.tuning}
      annotations={annotations}
    />
  );

  const timeS = (frame / fps) * props.speed;
  const ks = props.keystrokes;
  return (
    <AbsoluteFill>
      {layout.mode === "same" ? clip : null}
      {layout.mode === "contain" ? (
        <DesignWidthContext.Provider value={layout.box.width}>{clip}</DesignWidthContext.Provider>
      ) : null}
      {layout.mode === "fit" ? <Letterboxed height={layout.height}>{clip}</Letterboxed> : null}
      {layout.mode === "cover" ? (
        <Cropped
          box={layout.box}
          cx={layout.follow ? poseAt(frame, fps, log, props.speed, props.tuning).cx : 0.5}
        >
          {clip}
        </Cropped>
      ) : null}
      <DesignWidthContext.Provider value={Math.max(width, height)}>
        {ks?.chords && log.keys?.length ? (
          <KeycapHUD keys={log.keys} timeS={timeS} position={ks.position} />
        ) : null}
        {ks?.typed ? <TypedChip beats={typed} timeS={timeS} position={ks.position} /> : null}
      </DesignWidthContext.Provider>
    </AbsoluteFill>
  );
};

const Letterboxed: React.FC<{ height: number; children: React.ReactNode }> = ({
  height,
  children,
}) => {
  const { width } = useVideoConfig();
  return (
    <AbsoluteFill style={{ alignItems: "center", justifyContent: "center", background: "#000" }}>
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
  position: "top" | "bottom";
}> = ({ beats, timeS, position }) => {
  const { k, insets } = useOverlayFrame();
  const beat = beats.find((b) => timeS >= b.fromS && timeS <= b.toS);
  if (!beat) return null;
  const typingEnd = beat.toS - TYPED_LINGER_S;
  const shown = Math.round(
    interpolate(timeS, [beat.fromS, typingEnd], [0, beat.text.length], CLAMPED),
  );
  const o = Math.min(
    interpolate(timeS, [beat.fromS, beat.fromS + TYPED_FADE_IN_S], [0, 1], CLAMPED),
    interpolate(timeS, [beat.toS - TYPED_FADE_OUT_S, beat.toS], [1, 0], CLAMPED),
  );
  return (
    <AbsoluteFill
      style={{
        justifyContent: position === "top" ? "flex-start" : "flex-end",
        alignItems: "center",
        padding: 56 * k,
        paddingTop: 56 * k + insets.top,
        paddingBottom: 56 * k + insets.bottom,
        pointerEvents: "none",
      }}
    >
      <div
        style={{
          opacity: o,
          maxWidth: "70%",
          padding: `${14 * k}px ${26 * k}px`,
          borderRadius: 999,
          background: "rgba(10,10,12,0.82)",
          color: "#fafafa",
          fontFamily: CODE_FONT,
          fontSize: 30 * k,
          whiteSpace: "nowrap",
          overflow: "hidden",
          textOverflow: "ellipsis",
          boxShadow: `0 ${8 * k}px ${24 * k}px rgba(0,0,0,0.35)`,
        }}
      >
        {beat.text.slice(0, shown)}
        <span style={{ opacity: Math.floor(timeS * CARET_BLINKS_PER_S) % 2 ? 0 : 1 }}>▍</span>
      </div>
    </AbsoluteFill>
  );
};

function pageAddress(log: RecordingProps["log"]): string | undefined {
  const url = log.actions.find((a) => a.kind === "navigate")?.label;
  return url ? displayAddress(url) : undefined;
}

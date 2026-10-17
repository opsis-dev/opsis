import React, { useMemo } from "react";
import { AbsoluteFill, interpolate, useCurrentFrame, useVideoConfig } from "remotion";
import type { RecordingProps } from "../../shared/compile/recording";
import { useAsset, useQuality, useReframe } from "../assets";
import { resolveAnnotations, toClickLog, TYPED_LINGER_S, typedBeats } from "../camera/action-log";
import { DemoClip } from "../camera/demo-clip";
import { KeycapHUD } from "../camera/keycap-hud";
import { clamp as clampN } from "../../shared/math";
import { SHOT_STYLES } from "../camera/lib/style";
import { poseAt } from "../camera/lib/zoom";
import { DesignWidthContext } from "../camera/lib/design-scale";
import { clamp } from "./anim";
import { useOverlayFrame, zoomBoost } from "./frame";
import { containBox } from "./still-layout";
import { displayAddress } from "../../shared/format";

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
    () => resolveAnnotations(props.log, props.annotations ?? []),
    [props.log, props.annotations],
  );
  const typed = useMemo(() => typedBeats(props.log), [props.log]);
  const address = useMemo(() => pageAddress(props.log), [props.log]);

  if (!src) return null;

  const sourceAspect = props.log.viewport.width / props.log.viewport.height;
  const outAspect = width / height;
  const reshaped = Math.abs(sourceAspect - outAspect) > 0.02;
  const narrower = outAspect < sourceAspect;
  const fullPage = SHOT_STYLES[props.style].framing !== "window";
  const fitted = reshaped && reframe === "follow" && (narrower || fullPage);
  const box = fitted ? containBox({ width, height }, sourceAspect) : undefined;

  const clip = (
    <DemoClip
      box={box}
      name={props.name}
      src={src}
      log={log}
      speed={props.speed}
      chrome={props.chrome}
      windowStyle={props.windowStyle}
      frameTheme={props.frameTheme}
      addressBar={props.addressBar}
      address={address}
      backdrop={props.backdrop}
      style={props.style}
      motionBlur={props.motionBlur && !fast}
      tuning={
        fitted && narrower
          ? { ...props.tuning, narrow: zoomBoost(sourceAspect, outAspect) }
          : props.tuning
      }
      cursor={props.cursor}
      cursorScale={props.cursorScale}
      ripple={props.ripple}
      annotations={annotations}
    />
  );

  let body: React.ReactNode = box ? (
    <DesignWidthContext.Provider value={box.width}>{clip}</DesignWidthContext.Provider>
  ) : (
    clip
  );
  if (reshaped && !box) {
    if (reframe === "fit") {
      const boxH = width / sourceAspect;
      body = (
        <AbsoluteFill
          style={{ alignItems: "center", justifyContent: "center", background: "#000" }}
        >
          <DesignWidthContext.Provider value={width}>
            <div style={{ position: "relative", width, height: boxH }}>{clip}</div>
          </DesignWidthContext.Provider>
        </AbsoluteFill>
      );
    } else {
      const boxW = Math.max(width, height * sourceAspect);
      const boxH = boxW / sourceAspect;
      const cx = reframe === "follow" ? poseAt(frame, fps, log, props.speed, props.tuning).cx : 0.5;
      const x = clampN(width / 2 - cx * boxW, width - boxW, 0);
      body = (
        <AbsoluteFill style={{ overflow: "hidden" }}>
          <DesignWidthContext.Provider value={boxW}>
            <div
              style={{
                position: "absolute",
                left: x,
                top: (height - boxH) / 2,
                width: boxW,
                height: boxH,
              }}
            >
              {clip}
            </div>
          </DesignWidthContext.Provider>
        </AbsoluteFill>
      );
    }
  }

  const timeS = (frame / fps) * props.speed;
  const ks = props.keystrokes;
  return (
    <AbsoluteFill>
      {body}
      <DesignWidthContext.Provider value={Math.max(width, height)}>
        {ks?.chords && log.keys?.length ? (
          <KeycapHUD keys={log.keys} timeS={timeS} position={ks.position} />
        ) : null}
        {ks?.typed ? <TypedChip beats={typed} timeS={timeS} position={ks.position} /> : null}
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
    interpolate(timeS, [beat.fromS, typingEnd], [0, beat.text.length], clamp),
  );
  const o = Math.min(
    interpolate(timeS, [beat.fromS, beat.fromS + 0.15], [0, 1], clamp),
    interpolate(timeS, [beat.toS - 0.2, beat.toS], [1, 0], clamp),
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
          fontFamily: "ui-monospace, SFMono-Regular, Menlo, monospace",
          fontSize: 30 * k,
          whiteSpace: "nowrap",
          overflow: "hidden",
          textOverflow: "ellipsis",
          boxShadow: `0 ${8 * k}px ${24 * k}px rgba(0,0,0,0.35)`,
        }}
      >
        {beat.text.slice(0, shown)}
        <span style={{ opacity: Math.floor(timeS * 2.5) % 2 ? 0 : 1 }}>▍</span>
      </div>
    </AbsoluteFill>
  );
};

function pageAddress(log: RecordingProps["log"]): string | undefined {
  const url = log.actions.find((a) => a.kind === "navigate")?.label;
  return url ? displayAddress(url) : undefined;
}

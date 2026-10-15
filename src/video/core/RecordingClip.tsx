import React, { useMemo } from "react";
import { AbsoluteFill, interpolate, useCurrentFrame, useVideoConfig } from "remotion";
import type { RecordingProps } from "../../shared/compile/recording";
import { useAsset, useQuality, useReframe } from "../assets";
import { resolveAnnotations, toClickLog, typedBeats } from "../camera/action-log";
import { DemoClip } from "../camera/DemoClip";
import { KeycapHUD } from "../camera/KeycapHUD";
import { clamp as clampN } from "../camera/lib/camera";
import { poseAt } from "../camera/lib/zoom";
import { DesignWidthContext } from "../camera/WindowFrame";
import { clamp } from "./anim";

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

  if (!src) return null;

  const clip = (
    <DemoClip
      name={props.name}
      src={src}
      log={log}
      speed={props.speed}
      chrome={props.chrome}
      backdrop={props.backdrop}
      style={props.style}
      motionBlur={props.motionBlur && !fast}
      tuning={props.tuning}
      cursor={props.cursor}
      cursorScale={props.cursorScale}
      ripple={props.ripple}
      annotations={annotations}
    />
  );

  const sourceAspect = props.log.viewport.width / props.log.viewport.height;
  const outAspect = width / height;
  let body: React.ReactNode = clip;
  if (Math.abs(sourceAspect - outAspect) > 0.02) {
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
      {ks?.chords && log.keys?.length ? <KeycapHUD keys={log.keys} timeS={timeS} /> : null}
      {ks?.typed ? <TypedChip beats={typed} timeS={timeS} position={ks.position} /> : null}
    </AbsoluteFill>
  );
};

const TypedChip: React.FC<{
  beats: Array<{ text: string; fromS: number; toS: number }>;
  timeS: number;
  position: "top" | "bottom";
}> = ({ beats, timeS, position }) => {
  const { width } = useVideoConfig();
  const k = width / 1920;
  const beat = beats.find((b) => timeS >= b.fromS && timeS <= b.toS);
  if (!beat) return null;
  const typingEnd = beat.toS - 0.6;
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

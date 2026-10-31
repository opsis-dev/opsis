import { Easing } from "@opsis/runtime/easing.ts";
import { interpolate } from "@opsis/runtime/interpolate.ts";
import React from "react";
import type { Annotation } from "#video/camera/action-log.ts";
import { useDesignScale } from "#video/camera/lib/design-scale.ts";
import { faded } from "#video/camera/lib/style.ts";
import { MAIN_FONT } from "#video/common/fonts.ts";
import { percent } from "#video/common/format.ts";
import { rectCenter } from "#video/common/geometry.ts";
import type { HandDrawnLook } from "#video/common/recording.ts";
import { drawOn, fadeWindow, isFadedOut } from "#video/core/anim.ts";
import { HandDrawnMark } from "#video/core/hand-drawn.tsx";
import type { Viewport, AnnotationKind } from "#video/generated/spec.ts";

const FADE_S = 0.18;

function fadeOf(seconds: number) {
  return {
    in: seconds,
    out: seconds,
    easeIn: Easing.out(Easing.cubic),
    easeOut: Easing.in(Easing.cubic),
  };
}

const POP_SCALE = 1.2;
const SPOTLIGHT = { radius: 12, dimSpread: 9999 } as const;
const HIGHLIGHT = { radius: 10, border: 3, fill: 0.12, growFrom: 0.96 } as const;
const MARKER = { opacity: 0.5, blend: "multiply" } as const;
const TOOLTIP = {
  rise: 8,
  padShare: 8 / 14,
  shadowY: 6,
  shadowBlur: 18,
  shadow: "rgba(0,0,0,0.25)",
} as const;
const CALLOUT = {
  width: 140,
  height: 110,
  line: "M130 10 C 90 20, 40 40, 12 96",
  head: "M4 82 L12 98 L28 88",
  headShownFrom: 0.9,
  textLeft: 120,
  textBottom: 96,
} as const;

type MarkProps = {
  annotation: Annotation;
  opacity: number;
  designScale: number;
  counter: number;
  viewport: Viewport;
  timeS: number;
};

function boxOf({ annotation, viewport }: MarkProps) {
  return {
    left: percent((annotation.rect.x - annotation.padding) / viewport.width),
    top: percent((annotation.rect.y - annotation.padding) / viewport.height),
    width: percent((annotation.rect.w + annotation.padding * 2) / viewport.width),
    height: percent((annotation.rect.h + annotation.padding * 2) / viewport.height),
  };
}

function Spotlight(props: MarkProps) {
  const look = props.annotation.markLook;
  const k = props.designScale;
  return (
    <div
      style={{
        position: "absolute",
        ...boxOf(props),
        borderRadius: (look?.radius ?? SPOTLIGHT.radius) * k,
        boxShadow: `0 0 ${(look?.softness ?? 0) * k}px ${SPOTLIGHT.dimSpread}px ${props.annotation.color}`,
        opacity: props.opacity,
      }}
    />
  );
}

function percentOf(fraction: number) {
  return fraction * 100;
}

function SketchedHighlight({ look, ...props }: MarkProps & { look: HandDrawnLook }) {
  const { annotation, opacity, designScale: k, counter, viewport, timeS } = props;
  const { left, top } = boxOf(props);
  const marker = look === "marker";
  function padded(size: number, of: number) {
    return (size + annotation.padding * 2) / of;
  }
  return (
    <div
      style={{
        position: "absolute",
        left,
        top,
        opacity: marker
          ? opacity * (annotation.markLook?.markerOpacity ?? MARKER.opacity)
          : opacity,
        mixBlendMode: marker ? MARKER.blend : undefined,
      }}
    >
      <HandDrawnMark
        look={look}
        progress={drawOn(timeS, annotation.fromS, annotation.markLook?.drawS ?? undefined)}
        color={annotation.color}
        strokeWidth={(annotation.markLook?.lineWidth ?? HIGHLIGHT.border) * k * counter}
        seed={annotation.seed}
        roughness={annotation.roughness}
        strokes={annotation.markLook?.strokes ?? undefined}
      >
        <div
          style={{
            width: `${percentOf(padded(annotation.rect.w, viewport.width))}cqw`,
            height: `${percentOf(padded(annotation.rect.h, viewport.height))}cqh`,
          }}
        />
      </HandDrawnMark>
    </div>
  );
}

function Highlight(props: MarkProps) {
  const { annotation, opacity, designScale: k, counter } = props;
  const { look, markLook } = annotation;
  if (look !== "clean") return <SketchedHighlight {...props} look={look} />;
  return (
    <div
      style={{
        position: "absolute",
        ...boxOf(props),
        borderRadius: (markLook?.radius ?? HIGHLIGHT.radius) * k,
        border: `${(markLook?.lineWidth ?? HIGHLIGHT.border) * k * counter}px solid ${annotation.color}`,
        background: faded(annotation.color, percent(markLook?.fillOpacity ?? HIGHLIGHT.fill)),
        opacity,
        transform: `scale(${interpolate(opacity, [0, 1], [HIGHLIGHT.growFrom, 1])})`,
      }}
    />
  );
}

function Tooltip(props: MarkProps) {
  const { annotation, opacity, designScale: k, counter, viewport } = props;
  const { labelLook: look, fontSize, textColor } = annotation.label;
  if (!look || fontSize === undefined) return null;
  return (
    <div
      style={{
        position: "absolute",
        left: percent(rectCenter(annotation.rect).x / viewport.width),
        top: percent((annotation.rect.y - annotation.padding) / viewport.height),
        transform: `translate(-50%, -100%) translateY(${(1 - opacity) * TOOLTIP.rise * k}px) scale(${counter * POP_SCALE})`,
        transformOrigin: "50% 100%",
        opacity,
      }}
    >
      <div
        style={{
          background: annotation.color,
          color: textColor,
          fontFamily: annotation.label.font ?? MAIN_FONT,
          fontWeight: look.fontWeight,
          fontSize: fontSize * k,
          padding: `${look.boxPadding * TOOLTIP.padShare * k}px ${look.boxPadding * k}px`,
          borderRadius: look.radius * k,
          whiteSpace: "nowrap",
          boxShadow: look.shadow
            ? `0 ${TOOLTIP.shadowY * k}px ${TOOLTIP.shadowBlur * k}px ${TOOLTIP.shadow}`
            : undefined,
        }}
      >
        {annotation.text}
      </div>
      <div
        style={{
          width: 0,
          height: 0,
          margin: "0 auto",
          borderLeft: `${look.pointerSize * k}px solid transparent`,
          borderRight: `${look.pointerSize * k}px solid transparent`,
          borderTop: `${look.pointerSize * k}px solid ${annotation.color}`,
        }}
      />
    </div>
  );
}

function Callout(props: MarkProps) {
  const { annotation, opacity, designScale: k, counter, viewport } = props;
  const { labelLook: look, fontSize } = annotation.label;
  if (!look || fontSize === undefined) return null;
  const size = look.arrowSize * k;
  return (
    <div
      style={{
        position: "absolute",
        left: percent((annotation.rect.x + annotation.rect.w) / viewport.width),
        top: percent(annotation.rect.y / viewport.height),
        transform: `scale(${counter * POP_SCALE})`,
        transformOrigin: "0 100%",
        opacity,
      }}
    >
      <svg
        width={CALLOUT.width * size}
        height={CALLOUT.height * size}
        viewBox={`0 0 ${CALLOUT.width} ${CALLOUT.height}`}
        style={{ position: "absolute", left: 0, bottom: 0, overflow: "visible" }}
      >
        <path
          d={CALLOUT.line}
          fill="none"
          stroke={annotation.color}
          strokeWidth={look.lineWidth}
          strokeLinecap="round"
          pathLength={1}
          strokeDasharray={1}
          strokeDashoffset={1 - opacity}
        />
        <path
          d={CALLOUT.head}
          fill="none"
          stroke={annotation.color}
          strokeWidth={look.lineWidth}
          strokeLinecap="round"
          strokeLinejoin="round"
          opacity={opacity > CALLOUT.headShownFrom ? 1 : 0}
        />
      </svg>
      <div
        style={{
          position: "absolute",
          left: CALLOUT.textLeft * size,
          bottom: CALLOUT.textBottom * size,
          color: annotation.color,
          fontFamily: annotation.label.font ?? MAIN_FONT,
          fontWeight: look.fontWeight,
          fontSize: fontSize * k,
          whiteSpace: "nowrap",
          transform: `rotate(${look.tilt}deg)`,
        }}
      >
        {annotation.text}
      </div>
    </div>
  );
}

const MARKS: Record<AnnotationKind, React.FC<MarkProps>> = {
  spotlight: Spotlight,
  highlight: Highlight,
  tooltip: Tooltip,
  callout: Callout,
};

export function Annotations({
  annotations,
  viewport,
  timeS,
  cameraScale,
}: {
  annotations: Annotation[];
  viewport: Viewport;
  timeS: number;
  cameraScale: number;
}) {
  const k = useDesignScale();
  const counter = 1 / Math.max(1, cameraScale);

  return (
    <div style={{ position: "absolute", inset: 0, pointerEvents: "none", containerType: "size" }}>
      {annotations.map((annotation) => {
        const fade = fadeOf(annotation.markLook?.fadeS ?? FADE_S);
        const opacity = fadeWindow(timeS, annotation.fromS, annotation.toS, fade);
        if (isFadedOut(opacity)) return null;
        const Mark = MARKS[annotation.kind];
        return (
          <Mark
            key={annotation.id}
            annotation={annotation}
            opacity={opacity}
            designScale={k}
            counter={counter}
            viewport={viewport}
            timeS={timeS}
          />
        );
      })}
    </div>
  );
}

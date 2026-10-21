import React from "react";
import { Easing, interpolate } from "remotion";
import type { AnnotationKind, HandDrawnLook } from "../../shared/compile/recording";
import { TEXT_COLOR } from "../../shared/elements/colors";
import { MAIN_FONT } from "../../shared/fonts";
import { percent } from "../../shared/format";
import { rectCenter, type Viewport } from "../../shared/schema/actions";
import { drawOn, fadeWindow, isFadedOut } from "../core/anim";
import { HandDrawnMark } from "../core/hand-drawn";
import type { Annotation } from "./action-log";
import { useDesignScale } from "./lib/design-scale";
import { faded } from "./lib/style";

const FADE = {
  in: 0.18,
  out: 0.18,
  easeIn: Easing.out(Easing.cubic),
  easeOut: Easing.in(Easing.cubic),
};

const POP_SCALE = 1.2;
const SPOTLIGHT = { radius: 12, dimSpread: 9999 } as const;
const HIGHLIGHT = { radius: 10, border: 3, fill: "12%", growFrom: 0.96 } as const;
const MARKER = { opacity: 0.5, blend: "multiply" } as const;
const TOOLTIP = {
  rise: 8,
  fontSize: 18,
  fontWeight: 600,
  padY: 8,
  padX: 14,
  radius: 10,
  arrow: 8,
  shadowY: 6,
  shadowBlur: 18,
  shadow: "rgba(0,0,0,0.25)",
} as const;
const CALLOUT = {
  width: 140,
  height: 110,
  stroke: 5,
  line: "M130 10 C 90 20, 40 40, 12 96",
  head: "M4 82 L12 98 L28 88",
  headShownFrom: 0.9,
  textLeft: 120,
  textBottom: 96,
  fontSize: 30,
  fontWeight: 800,
  tilt: "rotate(-4deg)",
} as const;

type MarkProps = {
  annotation: Annotation;
  opacity: number;
  designScale: number;
  counter: number;
  viewport: Viewport;
  timeS: number;
};

const boxOf = ({ annotation, viewport }: MarkProps) => ({
  left: percent((annotation.rect.x - annotation.padding) / viewport.width),
  top: percent((annotation.rect.y - annotation.padding) / viewport.height),
  width: percent((annotation.rect.w + annotation.padding * 2) / viewport.width),
  height: percent((annotation.rect.h + annotation.padding * 2) / viewport.height),
});

const Spotlight: React.FC<MarkProps> = (props) => (
  <div
    style={{
      position: "absolute",
      ...boxOf(props),
      borderRadius: SPOTLIGHT.radius * props.designScale,
      boxShadow: `0 0 0 ${SPOTLIGHT.dimSpread}px ${props.annotation.color}`,
      opacity: props.opacity,
    }}
  />
);

const percentOf = (fraction: number) => fraction * 100;

const SketchedHighlight: React.FC<MarkProps & { look: HandDrawnLook }> = ({ look, ...props }) => {
  const { annotation, opacity, designScale: k, counter, viewport, timeS } = props;
  const { left, top } = boxOf(props);
  const marker = look === "marker";
  const padded = (size: number, of: number) => (size + annotation.padding * 2) / of;
  return (
    <div
      style={{
        position: "absolute",
        left,
        top,
        opacity: marker ? opacity * MARKER.opacity : opacity,
        mixBlendMode: marker ? MARKER.blend : undefined,
      }}
    >
      <HandDrawnMark
        look={look}
        progress={drawOn(timeS, annotation.fromS)}
        color={annotation.color}
        strokeWidth={HIGHLIGHT.border * k * counter}
        seed={annotation.seed}
        roughness={annotation.roughness}
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
};

const Highlight: React.FC<MarkProps> = (props) => {
  const { annotation, opacity, designScale: k, counter } = props;
  const { look } = annotation;
  if (look !== "clean") return <SketchedHighlight {...props} look={look} />;
  return (
    <div
      style={{
        position: "absolute",
        ...boxOf(props),
        borderRadius: HIGHLIGHT.radius * k,
        border: `${HIGHLIGHT.border * k * counter}px solid ${annotation.color}`,
        background: faded(annotation.color, HIGHLIGHT.fill),
        opacity,
        transform: `scale(${interpolate(opacity, [0, 1], [HIGHLIGHT.growFrom, 1])})`,
      }}
    />
  );
};

const Tooltip: React.FC<MarkProps> = (props) => {
  const { annotation, opacity, designScale: k, counter, viewport } = props;
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
          color: annotation.label.textColor ?? TEXT_COLOR,
          fontFamily: annotation.label.font ?? MAIN_FONT,
          fontWeight: TOOLTIP.fontWeight,
          fontSize: (annotation.label.fontSize ?? TOOLTIP.fontSize) * k,
          padding: `${TOOLTIP.padY * k}px ${TOOLTIP.padX * k}px`,
          borderRadius: TOOLTIP.radius * k,
          whiteSpace: "nowrap",
          boxShadow: `0 ${TOOLTIP.shadowY * k}px ${TOOLTIP.shadowBlur * k}px ${TOOLTIP.shadow}`,
        }}
      >
        {annotation.text}
      </div>
      <div
        style={{
          width: 0,
          height: 0,
          margin: "0 auto",
          borderLeft: `${TOOLTIP.arrow * k}px solid transparent`,
          borderRight: `${TOOLTIP.arrow * k}px solid transparent`,
          borderTop: `${TOOLTIP.arrow * k}px solid ${annotation.color}`,
        }}
      />
    </div>
  );
};

const Callout: React.FC<MarkProps> = (props) => {
  const { annotation, opacity, designScale: k, counter, viewport } = props;
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
        width={CALLOUT.width * k}
        height={CALLOUT.height * k}
        viewBox={`0 0 ${CALLOUT.width} ${CALLOUT.height}`}
        style={{ position: "absolute", left: 0, bottom: 0, overflow: "visible" }}
      >
        <path
          d={CALLOUT.line}
          fill="none"
          stroke={annotation.color}
          strokeWidth={CALLOUT.stroke}
          strokeLinecap="round"
          pathLength={1}
          strokeDasharray={1}
          strokeDashoffset={1 - opacity}
        />
        <path
          d={CALLOUT.head}
          fill="none"
          stroke={annotation.color}
          strokeWidth={CALLOUT.stroke}
          strokeLinecap="round"
          strokeLinejoin="round"
          opacity={opacity > CALLOUT.headShownFrom ? 1 : 0}
        />
      </svg>
      <div
        style={{
          position: "absolute",
          left: CALLOUT.textLeft * k,
          bottom: CALLOUT.textBottom * k,
          color: annotation.color,
          fontFamily: annotation.label.font ?? MAIN_FONT,
          fontWeight: CALLOUT.fontWeight,
          fontSize: (annotation.label.fontSize ?? CALLOUT.fontSize) * k,
          whiteSpace: "nowrap",
          transform: CALLOUT.tilt,
        }}
      >
        {annotation.text}
      </div>
    </div>
  );
};

const MARKS: Record<AnnotationKind, React.FC<MarkProps>> = {
  spotlight: Spotlight,
  highlight: Highlight,
  tooltip: Tooltip,
  callout: Callout,
};

export const Annotations: React.FC<{
  annotations: Annotation[];
  viewport: Viewport;
  timeS: number;
  cameraScale: number;
}> = ({ annotations, viewport, timeS, cameraScale }) => {
  const k = useDesignScale();
  const counter = 1 / Math.max(1, cameraScale);

  return (
    <div style={{ position: "absolute", inset: 0, pointerEvents: "none", containerType: "size" }}>
      {annotations.map((annotation) => {
        const opacity = fadeWindow(timeS, annotation.fromS, annotation.toS, FADE);
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
};

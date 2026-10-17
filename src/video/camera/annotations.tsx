import React from "react";
import { Easing, interpolate } from "remotion";
import type { AnnotationKind } from "../../shared/compile/recording";
import { TEXT_COLOR } from "../../shared/elements/colors";
import { MAIN_FONT } from "../../shared/fonts";
import { percent } from "../../shared/format";
import { rectCentre } from "../../shared/schema/actions";
import type { Annotation } from "./action-log";
import { useDesignScale } from "./lib/design-scale";
import { fadeWindow } from "../core/anim";

const FADE = {
  in: 0.18,
  out: 0.18,
  easeIn: Easing.out(Easing.cubic),
  easeOut: Easing.in(Easing.cubic),
};

type MarkProps = {
  a: Annotation;
  o: number;
  k: number;
  counter: number;
  viewport: { width: number; height: number };
};

const pct = (v: number, of: number): string => percent(v / of);

const boxOf = ({ a, viewport }: MarkProps) => ({
  left: pct(a.rect.x - a.padding, viewport.width),
  top: pct(a.rect.y - a.padding, viewport.height),
  width: pct(a.rect.w + a.padding * 2, viewport.width),
  height: pct(a.rect.h + a.padding * 2, viewport.height),
});

const Spotlight: React.FC<MarkProps> = (props) => (
  <div
    style={{
      position: "absolute",
      ...boxOf(props),
      borderRadius: 12 * props.k,
      boxShadow: `0 0 0 9999px ${props.a.color}`,
      opacity: props.o,
    }}
  />
);

const Highlight: React.FC<MarkProps> = (props) => {
  const { a, o, k, counter } = props;
  return (
    <div
      style={{
        position: "absolute",
        ...boxOf(props),
        borderRadius: 10 * k,
        border: `${3 * k * counter}px solid ${a.color}`,
        background: `color-mix(in srgb, ${a.color} 12%, transparent)`,
        opacity: o,
        transform: `scale(${interpolate(o, [0, 1], [0.96, 1])})`,
      }}
    />
  );
};

const Tooltip: React.FC<MarkProps> = ({ a, o, k, counter, viewport }) => (
  <div
    style={{
      position: "absolute",
      left: pct(rectCentre(a.rect).x, viewport.width),
      top: pct(a.rect.y - a.padding, viewport.height),
      transform: `translate(-50%, -100%) translateY(${(1 - o) * 8 * k}px) scale(${counter * 1.2})`,
      transformOrigin: "50% 100%",
      opacity: o,
    }}
  >
    <div
      style={{
        background: a.color,
        color: TEXT_COLOR,
        fontFamily: MAIN_FONT,
        fontWeight: 600,
        fontSize: 18 * k,
        padding: `${8 * k}px ${14 * k}px`,
        borderRadius: 10 * k,
        whiteSpace: "nowrap",
        boxShadow: `0 ${6 * k}px ${18 * k}px rgba(0,0,0,0.25)`,
      }}
    >
      {a.text}
    </div>
    <div
      style={{
        width: 0,
        height: 0,
        margin: "0 auto",
        borderLeft: `${8 * k}px solid transparent`,
        borderRight: `${8 * k}px solid transparent`,
        borderTop: `${8 * k}px solid ${a.color}`,
      }}
    />
  </div>
);

const Callout: React.FC<MarkProps> = ({ a, o, k, counter, viewport }) => (
  <div
    style={{
      position: "absolute",
      left: pct(a.rect.x + a.rect.w, viewport.width),
      top: pct(a.rect.y, viewport.height),
      transform: `scale(${counter * 1.2})`,
      transformOrigin: "0 100%",
      opacity: o,
    }}
  >
    <svg
      width={140 * k}
      height={110 * k}
      viewBox="0 0 140 110"
      style={{ position: "absolute", left: 0, bottom: 0, overflow: "visible" }}
    >
      <path
        d="M130 10 C 90 20, 40 40, 12 96"
        fill="none"
        stroke={a.color}
        strokeWidth={5}
        strokeLinecap="round"
        pathLength={1}
        strokeDasharray={1}
        strokeDashoffset={1 - o}
      />
      <path
        d="M4 82 L12 98 L28 88"
        fill="none"
        stroke={a.color}
        strokeWidth={5}
        strokeLinecap="round"
        strokeLinejoin="round"
        opacity={o > 0.9 ? 1 : 0}
      />
    </svg>
    <div
      style={{
        position: "absolute",
        left: 120 * k,
        bottom: 96 * k,
        color: a.color,
        fontFamily: MAIN_FONT,
        fontWeight: 800,
        fontSize: 30 * k,
        whiteSpace: "nowrap",
        transform: "rotate(-4deg)",
      }}
    >
      {a.text}
    </div>
  </div>
);

const MARKS: Record<AnnotationKind, React.FC<MarkProps>> = {
  spotlight: Spotlight,
  highlight: Highlight,
  tooltip: Tooltip,
  callout: Callout,
};

export const Annotations: React.FC<{
  annotations: Annotation[];
  viewport: { width: number; height: number };
  timeS: number;
  cameraScale: number;
}> = ({ annotations, viewport, timeS, cameraScale }) => {
  const k = useDesignScale();
  const counter = 1 / Math.max(1, cameraScale);

  return (
    <div style={{ position: "absolute", inset: 0, pointerEvents: "none" }}>
      {annotations.map((a, i) => {
        const o = fadeWindow(timeS, a.fromS, a.toS, FADE);
        if (o <= 0.001) return null;
        const Mark = MARKS[a.kind];
        return <Mark key={i} a={a} o={o} k={k} counter={counter} viewport={viewport} />;
      })}
    </div>
  );
};

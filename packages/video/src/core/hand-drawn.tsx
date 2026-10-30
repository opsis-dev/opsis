import React, { useEffectEvent, useLayoutEffect, useRef } from "react";
import { type MarkLook, markPaths, strokeShown } from "#video/core/hand-drawn-marks.ts";

const STROKES = 2;
const SVG_NS = "http://www.w3.org/2000/svg";

type HandDrawnMarkProps = {
  look: MarkLook;
  progress: number;
  color: string;
  strokeWidth: number;
  seed: number;
  roughness?: number;
  strokes?: number;
  children: React.ReactNode;
};

type Stroke = { d: string; shown: number; color: string; width: number };

function strokePath({ d, shown, color, width }: Stroke): SVGPathElement {
  const path = document.createElementNS(SVG_NS, "path");
  path.setAttribute("d", d);
  path.setAttribute("fill", "none");
  path.setAttribute("stroke", color);
  path.setAttribute("stroke-width", String(width));
  if (shown < 1) {
    path.setAttribute("pathLength", "1");
    path.setAttribute("stroke-dasharray", "1 1");
    path.setAttribute("stroke-dashoffset", String(1 - shown));
  }
  return path;
}

type Drawing = Omit<HandDrawnMarkProps, "children" | "strokes"> & { strokes: number };

function drawMark(svg: SVGSVGElement, tracker: HTMLElement, mark: Drawing) {
  const box = {
    x: tracker.offsetLeft,
    y: tracker.offsetTop,
    w: tracker.offsetWidth,
    h: tracker.offsetHeight,
  };
  const { paths, strokeWidth = mark.strokeWidth } = markPaths(
    {
      look: mark.look,
      seed: Math.max(1, mark.seed),
      strokes: mark.strokes,
      roughness: mark.roughness,
    },
    box,
  );
  svg.replaceChildren(
    ...paths.map((d, i) =>
      strokePath({
        d,
        shown: strokeShown(mark.progress, i, paths.length),
        color: mark.color,
        width: strokeWidth,
      }),
    ),
  );
}

const MARK_SVG = {
  position: "absolute",
  top: 0,
  left: 0,
  overflow: "visible",
  width: 100,
  height: 100,
} as const;

export function HandDrawnMark({ strokes = STROKES, children, ...mark }: HandDrawnMarkProps) {
  const svg = useRef<SVGSVGElement>(null);
  const tracker = useRef<HTMLSpanElement>(null);
  const redraw = useEffectEvent(() => {
    if (svg.current && tracker.current)
      drawMark(svg.current, tracker.current, { ...mark, strokes });
  });
  useLayoutEffect(() => redraw());
  useLayoutEffect(() => {
    const target = tracker.current;
    if (!target) return;
    const observer = new ResizeObserver(() => redraw());
    observer.observe(target);
    return () => observer.disconnect();
  }, []);
  const drawn = <svg ref={svg} style={MARK_SVG} />;
  const behind = mark.look === "marker";
  return (
    <span style={{ display: "inline-block", position: "relative" }}>
      {behind ? drawn : null}
      <span
        ref={tracker}
        style={{ display: "inline-block", position: "relative", whiteSpace: "pre" }}
      >
        {children}
      </span>
      {behind ? null : drawn}
    </span>
  );
}

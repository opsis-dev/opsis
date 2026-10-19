import { MorphIcon } from "morphicons/react";
import { useReducedMotion } from "motion/react";
import { createElement, useEffect, useRef, useState, type RefObject } from "react";
import { cn } from "@/renderer/lib/utils";
import { FALLBACK_LOOK, nodeLook, type NodeGlyph as Glyph } from "@/shared/nodes/looks";
import { NODE_GLYPHS } from "./node-glyphs.generated";
import { playNodeMotion } from "./node-motions";

const GLYPH_HOST_ATTRIBUTE = "data-glyph-host";

export const GLYPH_HOST = { [GLYPH_HOST_ATTRIBUTE]: "" };

const glyphNode = (glyph: Glyph) => NODE_GLYPHS[glyph] ?? NODE_GLYPHS[FALLBACK_LOOK.glyph];

function useHostHover(ref: RefObject<HTMLElement | null>): boolean {
  const [hovered, setHovered] = useState(false);
  useEffect(() => {
    const own = ref.current;
    const host = own?.closest(`[${GLYPH_HOST_ATTRIBUTE}]`) ?? own;
    if (!host) return;
    const enter = () => setHovered(true);
    const leave = () => setHovered(false);
    host.addEventListener("pointerenter", enter);
    host.addEventListener("pointerleave", leave);
    return () => {
      host.removeEventListener("pointerenter", enter);
      host.removeEventListener("pointerleave", leave);
    };
  }, [ref]);
  return hovered;
}

const SVG_PROPS = {
  viewBox: "0 0 24 24",
  fill: "none",
  stroke: "currentColor",
  strokeWidth: 2,
  strokeLinecap: "round",
  strokeLinejoin: "round",
  "aria-hidden": true,
} as const;

export function NodeGlyph({ type, className }: { type: string; className?: string }) {
  const look = nodeLook(type);
  const holder = useRef<HTMLSpanElement>(null);
  const svg = useRef<SVGSVGElement>(null);
  const hovered = useHostHover(holder);
  const reduced = useReducedMotion();
  const playing = useRef(false);

  useEffect(() => {
    if (!hovered || reduced || !look.motion || playing.current || !svg.current || !holder.current)
      return;
    playing.current = true;
    void playNodeMotion(look.motion, svg.current, holder.current).finally(() => {
      playing.current = false;
    });
  }, [hovered, reduced, look.motion]);

  return (
    <span ref={holder} className={cn("relative grid shrink-0 place-items-center", className)}>
      {look.morph ? (
        <MorphIcon
          icon={glyphNode(hovered ? look.morph : look.glyph)}
          className="size-full"
          strokeWidth={2}
          spring="snappy"
          reducedMotion="user"
        />
      ) : (
        <svg ref={svg} {...SVG_PROPS} className="size-full overflow-visible">
          {glyphNode(look.glyph).map(([tag, attrs], i) => createElement(tag, { key: i, ...attrs }))}
        </svg>
      )}
    </span>
  );
}

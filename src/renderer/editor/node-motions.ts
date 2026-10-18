import { animate, steps, type Easing } from "motion/react";
import { isKeyOf } from "@/shared/guards";
import type { NodeMotion } from "@/shared/nodes/looks";

const MOTION_PROPS = {
  transform: true,
  opacity: true,
  filter: true,
  clipPath: true,
  strokeDashoffset: true,
  maskPosition: true,
  "--sweep": true,
} as const;

type MotionProp = keyof typeof MOTION_PROPS;

export type MotionFrame = Partial<Record<MotionProp, string | number>> & {
  offset?: number;
  ease?: Easing;
};

type Part = "svg" | "shapes" | "ring" | "flare" | "march";

type MotionSpec = {
  frames: MotionFrame[];
  ms: number;
  origin?: string;
  part?: Part;
  mask?: { image: string; size?: string };
  clip?: boolean;
};

const EASE: Easing = [0.45, 0, 0.25, 1];
const OUT: Easing = [0.2, 0.8, 0.2, 1];
const BACK: Easing = [0.34, 1.56, 0.64, 1];
const LINEAR: Easing = "linear";
const STEP = steps(1, "end");

const scale = (s: number | string) => ({ transform: `scale(${s})` });
const blurred = (px: number, rest: Record<string, string | number> = {}) => ({
  filter: `blur(${px}px)`,
  ...rest,
});
const outAndBack = (
  prop: MotionProp,
  rest: string,
  mid: string,
  ms: number,
  ease = EASE,
): MotionSpec => ({
  frames: [{ [prop]: rest, ease }, { [prop]: mid, ease }, { [prop]: rest }],
  ms,
});
const away = (
  prop: MotionProp,
  rest: string,
  gone: string,
  back: string,
  ms: number,
  ease = EASE,
) => ({
  frames: [
    { [prop]: rest, opacity: 1, ease },
    { [prop]: gone, opacity: 0, offset: 0.45, ease: STEP },
    { [prop]: back, opacity: 0, offset: 0.46, ease },
    { [prop]: rest, opacity: 1 },
  ],
  ms,
  clip: true,
});
const split = (dx: number, dy: number) => `translate(${dx}%, ${dy}%)`;
const rgb = (px: number) =>
  `drop-shadow(${-px}px 0px 0px #ff2d55) drop-shadow(${px}px 0px 0px #00e5ff)`;
const ghost = (x: number, alpha: number) =>
  `drop-shadow(${x}px 0px 0px color-mix(in srgb, currentColor ${alpha}%, transparent))`;
const smear = (on: boolean) =>
  [ghost(on ? -3 : 0, 45), ghost(on ? -6 : 0, 25), ghost(on ? -9 : 0, 12)].join(" ");
const orbit = (i: number) =>
  `translate(${(Math.sin((i * Math.PI) / 4) * 12).toFixed(2)}%, ${(12 - Math.cos((i * Math.PI) / 4) * 12).toFixed(2)}%)`;

export const NODE_MOTION_SPECS: Record<NodeMotion, MotionSpec> = {
  press: outAndBack("transform", "scale(1)", "scale(0.8)", 320, OUT),
  bob: outAndBack("transform", "translateY(0%)", "translateY(-12%)", 700),
  "nudge-up": outAndBack("transform", "translateY(0%)", "translateY(-16%)", 520),
  "nudge-down": outAndBack("transform", "translateY(0%)", "translateY(16%)", 520),
  "nudge-right": outAndBack("transform", "translateX(0%)", "translateX(16%)", 520),
  pan: {
    frames: [
      { transform: "translateX(0%)", ease: EASE },
      { transform: "translateX(-16%)", ease: EASE },
      { transform: "translateX(16%)", ease: EASE },
      { transform: "translateX(0%)" },
    ],
    ms: 1300,
  },
  track: {
    frames: [0, 1, 2, 3, 4, 5, 6, 7, 8].map((i) => ({ transform: orbit(i % 8), ease: LINEAR })),
    ms: 1100,
  },
  turn: {
    frames: [{ transform: "rotate(0deg)", ease: EASE }, { transform: "rotate(360deg)" }],
    ms: 800,
  },
  "turn-over": {
    frames: [{ transform: "rotate(0deg)", ease: EASE }, { transform: "rotate(180deg)" }],
    ms: 700,
  },
  twist: outAndBack("transform", "rotate(0deg)", "rotate(35deg)", 600),
  stretch: outAndBack("transform", "scaleX(1)", "scaleX(1.3)", 600),
  grow: {
    frames: [
      { transform: "scaleY(1)", ease: EASE },
      { transform: "scaleY(0.15)", ease: BACK },
      { transform: "scaleY(1)" },
    ],
    ms: 700,
    origin: "50% 100%",
  },
  open: {
    frames: [
      { transform: "perspective(60px) rotateX(0deg)", ease: EASE },
      { transform: "perspective(60px) rotateX(55deg)", ease: EASE },
      { transform: "perspective(60px) rotateX(0deg)" },
    ],
    ms: 800,
    origin: "50% 100%",
  },
  flip: {
    frames: [
      { transform: "perspective(80px) rotateY(0deg)", ease: EASE },
      { transform: "perspective(80px) rotateY(180deg)", ease: EASE },
      { transform: "perspective(80px) rotateY(360deg)" },
    ],
    ms: 800,
  },
  shutter: { ...outAndBack("transform", "scaleY(1)", "scaleY(0)", 600), origin: "50% 0%" },
  collapse: {
    frames: [
      { transform: "scale(1, 1)", opacity: 1, ease: EASE },
      { transform: "scale(1, 0.06)", opacity: 1, offset: 0.3, ease: EASE },
      { transform: "scale(0.06, 0.06)", opacity: 1, offset: 0.55, ease: EASE },
      { transform: "scale(0.06, 0.06)", opacity: 0, offset: 0.7, ease: EASE },
      { transform: "scale(1, 1)", opacity: 1 },
    ],
    ms: 1100,
  },
  "slide-through": away("transform", "translateX(0%)", "translateX(70%)", "translateX(-70%)", 800),
  push: away("transform", "translateX(0%)", "translateX(-60%)", "translateX(60%)", 800, OUT),
  roll: away("transform", "translateY(0%)", "translateY(-80%)", "translateY(80%)", 700),
  "zoom-through": away("transform", "scale(1)", "scale(1.6)", "scale(0.6)", 800),
  whip: {
    frames: [
      { transform: "translateX(0%)", ...blurred(0), opacity: 1, ease: EASE },
      { transform: "translateX(60%)", ...blurred(3), opacity: 0, offset: 0.4, ease: STEP },
      { transform: "translateX(-60%)", ...blurred(3), opacity: 0, offset: 0.41, ease: EASE },
      { transform: "translateX(0%)", ...blurred(0), opacity: 1 },
    ],
    ms: 520,
    clip: true,
  },
  "zoom-blur": {
    frames: [
      { ...scale(1), ...blurred(0), ease: EASE },
      { ...scale(1.45), ...blurred(3), ease: EASE },
      { ...scale(1), ...blurred(0) },
    ],
    ms: 700,
  },
  smear: {
    frames: [
      { transform: "translateX(0%)", filter: smear(false), ease: EASE },
      { transform: "translateX(18%)", filter: smear(true), ease: EASE },
      { transform: "translateX(0%)", filter: smear(false) },
    ],
    ms: 650,
  },
  draw: {
    frames: [
      { strokeDashoffset: 0, ease: EASE },
      { strokeDashoffset: 1, offset: 0.3, ease: EASE },
      { strokeDashoffset: 0 },
    ],
    ms: 1100,
    part: "shapes",
  },
  march: {
    frames: [{ strokeDashoffset: 0, ease: LINEAR }, { strokeDashoffset: -15 }],
    ms: 1500,
    part: "march",
  },
  caret: {
    frames: [1, 0, 1, 0, 1, 1].map((opacity, i) => ({ opacity, offset: i / 5, ease: STEP })),
    ms: 1100,
  },
  "wipe-reveal": {
    frames: [
      { maskPosition: "0% 0%", ease: EASE },
      { maskPosition: "100% 0%", offset: 0.35, ease: EASE },
      { maskPosition: "0% 0%" },
    ],
    ms: 900,
    mask: { image: "linear-gradient(90deg, #000 45%, transparent 55%)", size: "300% 100%" },
  },
  "mask-up": {
    frames: [
      { clipPath: "inset(0% 0% 0% 0%)", opacity: 1, ease: OUT },
      { clipPath: "inset(0% 0% 0% 0%)", opacity: 0, offset: 0.2, ease: STEP },
      { clipPath: "inset(100% 0% 0% 0%)", opacity: 1, offset: 0.21, ease: OUT },
      { clipPath: "inset(0% 0% 0% 0%)", opacity: 1 },
    ],
    ms: 800,
  },
  "clock-wipe": {
    frames: [
      { "--sweep": "360deg", ease: EASE },
      { "--sweep": "0deg", ease: EASE },
      { "--sweep": "360deg" },
    ],
    ms: 1000,
    mask: { image: "conic-gradient(#000 var(--sweep), transparent var(--sweep))" },
  },
  iris: outAndBack("clipPath", "circle(75% at 50% 50%)", "circle(0% at 50% 50%)", 800),
  shimmer: {
    frames: [{ maskPosition: "100% 0%", ease: LINEAR }, { maskPosition: "0% 0%" }],
    ms: 1000,
    mask: {
      image: "linear-gradient(110deg, rgba(0,0,0,0.3) 40%, #000 50%, rgba(0,0,0,0.3) 60%)",
      size: "350% 100%",
    },
  },
  "blur-swap": {
    frames: [
      { ...blurred(0), opacity: 1, ...scale(1), ease: EASE },
      { ...blurred(5), opacity: 0, ...scale(0.85), offset: 0.45, ease: EASE },
      { ...blurred(5), opacity: 0, ...scale(0.85), offset: 0.55, ease: EASE },
      { ...blurred(0), opacity: 1, ...scale(1) },
    ],
    ms: 750,
  },
  "rack-focus": {
    frames: [{ ...blurred(0), ease: EASE }, { ...blurred(3), ease: EASE }, { ...blurred(0) }],
    ms: 800,
  },
  rise: {
    frames: [
      { transform: "translateY(0%)", opacity: 1, ease: OUT },
      { transform: "translateY(0%)", opacity: 0, offset: 0.2, ease: STEP },
      { transform: "translateY(35%)", opacity: 0, offset: 0.21, ease: OUT },
      { transform: "translateY(0%)", opacity: 1 },
    ],
    ms: 900,
  },
  tracking: {
    frames: [
      { transform: "scaleX(1)", ...blurred(0), opacity: 1, ease: EASE },
      { transform: "scaleX(1.6)", ...blurred(3), opacity: 0.3, offset: 0.35, ease: BACK },
      { transform: "scaleX(1)", ...blurred(0), opacity: 1 },
    ],
    ms: 900,
  },
  snap: {
    frames: [
      { ...scale(1), ...blurred(0), ease: EASE },
      { ...scale(1.5), ...blurred(3), offset: 0.5, ease: STEP },
      { ...scale(1), ...blurred(0), offset: 0.51, ease: STEP },
      { ...scale(1), ...blurred(0) },
    ],
    ms: 800,
  },
  "fade-through": outAndBack("opacity", "1", "0.1", 800),
  dissolve: {
    frames: [
      { opacity: 1, ...blurred(0), ...scale(1), ease: EASE },
      { opacity: 0, ...blurred(4), ...scale(1.2), ease: EASE },
      { opacity: 1, ...blurred(0), ...scale(1) },
    ],
    ms: 900,
  },
  glitch: {
    frames: [
      { transform: split(0, 0), filter: rgb(0), ease: STEP },
      { transform: split(3, -2), filter: rgb(2), ease: STEP },
      { transform: split(-4, 1), filter: rgb(-2), ease: STEP },
      { transform: split(2, 3), filter: rgb(2), ease: STEP },
      { transform: split(-3, -1), filter: rgb(-2), ease: STEP },
      { transform: split(4, 2), filter: rgb(2), ease: STEP },
      { transform: split(0, 0), filter: rgb(0) },
    ],
    ms: 520,
  },
  flicker: {
    frames: [
      { opacity: 1, ease: STEP },
      { opacity: 0.1, offset: 0.08, ease: STEP },
      { opacity: 1, offset: 0.16, ease: STEP },
      { opacity: 0.15, offset: 0.3, ease: STEP },
      { opacity: 0.9, offset: 0.38, ease: STEP },
      { opacity: 0.05, offset: 0.5, ease: STEP },
      { opacity: 1, offset: 0.62, ease: STEP },
      { opacity: 1 },
    ],
    ms: 1000,
  },
  "hard-cut": {
    frames: [1, 1.35, 0.75, 1.15, 1].map((s, i) => ({ ...scale(s), offset: i / 4, ease: STEP })),
    ms: 800,
  },
  ripple: {
    frames: [
      { ...scale(0.4), opacity: 0.9, ease: OUT },
      { ...scale(1.6), opacity: 0 },
    ],
    ms: 800,
    part: "ring",
  },
  flare: {
    frames: [
      { transform: "translateX(-90%)", opacity: 0, ease: EASE },
      { transform: "translateX(0%)", opacity: 1, ease: EASE },
      { transform: "translateX(90%)", opacity: 0 },
    ],
    ms: 1000,
    part: "flare",
  },
};

export function motionTimeline(frames: MotionFrame[]) {
  const known = frames.map(
    (f, i) => f.offset ?? (i === 0 ? 0 : i === frames.length - 1 ? 1 : undefined),
  );
  const times = known.map((t, i) => {
    if (t !== undefined) return t;
    const prev = known.slice(0, i).findLastIndex((v) => v !== undefined);
    const next = i + known.slice(i).findIndex((v) => v !== undefined);
    const from = known[prev] ?? 0;
    const to = known[next] ?? 1;
    return from + ((to - from) * (i - prev)) / (next - prev);
  });
  const props = Object.keys(frames[0]).filter((k) => isKeyOf(MOTION_PROPS, k));
  const values = Object.fromEntries(props.map((p) => [p, frames.map((f) => f[p])]));
  const ease = frames.slice(0, -1).map((f) => f.ease ?? EASE);
  return { values, times, ease };
}

const SVG_NS = "http://www.w3.org/2000/svg";

type Prepared = { targets: Element[]; restore: () => void };

const svgPieces = (svg: SVGSVGElement): SVGElement[] =>
  [...svg.children].filter((el): el is SVGElement => el instanceof SVGElement);

function marchingOutline(svg: SVGSVGElement): Prepared {
  const pieces = svgPieces(svg);
  const ring = document.createElementNS(SVG_NS, "rect");
  for (const [k, v] of Object.entries({
    x: 3,
    y: 3,
    width: 18,
    height: 18,
    rx: 2,
    "stroke-dasharray": "2 3",
  }))
    ring.setAttribute(k, String(v));
  svg.appendChild(ring);
  for (const p of pieces) p.style.opacity = "0";
  return {
    targets: [ring],
    restore: () => {
      ring.remove();
      for (const p of pieces) p.style.opacity = "";
    },
  };
}

function drawable(svg: SVGSVGElement): Prepared {
  const shapes = svgPieces(svg);
  for (const s of shapes) {
    s.setAttribute("pathLength", "1");
    s.style.strokeDasharray = "1";
  }
  return {
    targets: shapes,
    restore: () => {
      for (const s of shapes) {
        s.removeAttribute("pathLength");
        s.style.strokeDasharray = "";
        s.style.strokeDashoffset = "";
      }
    },
  };
}

function overlay(holder: HTMLElement, className: string): Prepared {
  const el = document.createElement("span");
  el.className = className;
  holder.appendChild(el);
  return { targets: [el], restore: () => el.remove() };
}

const RING_CLASS =
  "pointer-events-none absolute inset-[-10%] rounded-full border-2 border-current opacity-0";
const FLARE_CLASS =
  "pointer-events-none absolute inset-[-60%] opacity-0 mix-blend-screen bg-[radial-gradient(circle,rgba(255,244,214,0.95)_0%,rgba(255,190,110,0.6)_22%,rgba(255,140,60,0.25)_40%,transparent_62%)]";

const PARTS: Record<Part, (svg: SVGSVGElement, holder: HTMLElement) => Prepared> = {
  svg: (svg) => ({ targets: [svg], restore: () => undefined }),
  shapes: drawable,
  ring: (_svg, holder) => overlay(holder, RING_CLASS),
  flare: (_svg, holder) => overlay(holder, FLARE_CLASS),
  march: marchingOutline,
};

function styleFor(spec: MotionSpec, svg: SVGSVGElement, holder: HTMLElement): () => void {
  if (spec.origin) svg.style.transformOrigin = spec.origin;
  if (spec.clip) holder.style.overflow = "hidden";
  if (spec.mask) {
    svg.style.maskImage = spec.mask.image;
    svg.style.maskSize = spec.mask.size ?? "";
    svg.style.maskRepeat = "no-repeat";
  }
  return () => {
    if (spec.origin) svg.style.transformOrigin = "";
    if (spec.clip) holder.style.overflow = "";
    if (spec.mask) {
      svg.style.maskImage = "";
      svg.style.maskSize = "";
      svg.style.maskRepeat = "";
    }
  };
}

export async function playNodeMotion(
  motion: NodeMotion,
  svg: SVGSVGElement,
  holder: HTMLElement,
): Promise<void> {
  const spec = NODE_MOTION_SPECS[motion];
  const { values, times, ease } = motionTimeline(spec.frames);
  const options = { duration: spec.ms / 1000, times, ease };
  const { targets, restore } = PARTS[spec.part ?? "svg"](svg, holder);
  const restoreStyle = styleFor(spec, svg, holder);
  try {
    await Promise.all(targets.map((t) => animate(t, values, options).finished));
  } finally {
    restore();
    restoreStyle();
  }
}

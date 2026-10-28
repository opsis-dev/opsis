import type { CSSProperties, ReactNode } from "react";
import { useEffect, useRef, useState } from "react";
import { AbsoluteFill, continueRender, delayRender, getRemotionEnvironment } from "remotion";
import { useAuthoredConfig, useAuthoredFrame } from "#video/core/authored-clock.ts";
import { resolveFont, WarmUIProvider, useWarmTheme } from "#video/lib/warm-ui/index.ts";

const SANS = '"Google Sans"';

export interface AnnounceTitleProps {
  fontFamily?: string;
  eyebrow?: string;
  title?: string;
  tagline?: string;
  voidColor?: string;
  fieldColor?: string;
  paperColor?: string;
  titleColor?: string;
  nightColor?: string;
  glowColor?: string;
  glowStrength?: number;
  taglineColor?: string;
  inkColor?: string;
  symbolPath?: string;
  symbolColors?: string[] | string;
  symbolScale?: number;
  speed?: number;
}

interface SymbolStyle {
  path: string;
  colors: string[];
  scale: number;
}

const PLANE_OUT = 18;
const FIELD_OUT = 41;
const PAPER_OUT = 80;
const MACRO_OUT = 110;

const REF_W = 802;
const REF_H = 450;

function settle(t: number, span: number, decay: number): number {
  if (t <= 0) return 0;
  if (t >= span) return 1;
  return (1 - decay ** t) / (1 - decay ** span);
}

function clamp01(v: number): number {
  return v < 0 ? 0 : v > 1 ? 1 : v;
}

function sample(table: readonly (readonly [number, number])[], p: number) {
  if (p <= table[0][0]) return table[0][1];
  const last = table[table.length - 1];
  if (p >= last[0]) return last[1];
  for (let i = 1; i < table.length; i++) {
    const [x1, y1] = table[i];
    if (p <= x1) {
      const [x0, y0] = table[i - 1];
      return y0 + ((y1 - y0) * (p - x0)) / (x1 - x0);
    }
  }
  return last[1];
}

function Shutter({
  frame,
  samples,
  open,
  children,
}: {
  frame: number;
  samples: number;
  open: number;
  children: (subFrame: number) => ReactNode;
}) {
  const layers: ReactNode[] = [];
  const n = Math.max(2, Math.round(samples));
  for (let i = 0; i < n; i++) {
    layers.push(
      <AbsoluteFill key={i} style={{ opacity: 1 / (i + 1) }}>
        {children(frame - open + (open * i) / (n - 1))}
      </AbsoluteFill>,
    );
  }
  return <>{layers}</>;
}

function lineStyle(fontSize: number, tracking: number, face: string): CSSProperties {
  return {
    fontFamily: face,
    fontSize,
    fontWeight: 400,
    lineHeight: 1,
    letterSpacing: `${tracking}em`,
    marginRight: `${-tracking}em`,
    whiteSpace: "pre",
    textRendering: "geometricPrecision",
    ...(getRemotionEnvironment().isRendering ? null : { willChange: "transform" as const }),
  };
}

const EYEBROW_SIZE = 41.6 / REF_W;
const EYEBROW_TRACK = -0.0666;
const TITLE_SIZE = 41.3 / REF_W;
const TITLE_TRACK = -0.0563;
const TAGLINE_SIZE = 35 / REF_W;
const TAGLINE_TRACK = -0.015;

const BASELINE_DROP = 0.335;

const EYEBROW_BASELINE = 236.87 / REF_H;
const TITLE_BASELINE = 240.4 / REF_H;
const TAGLINE_BASELINE = 233.5 / REF_H;

function Line({
  children,
  fontSize,
  tracking,
  baseline,
  height,
  style,
}: {
  children: ReactNode;
  fontSize: number;
  tracking: number;
  baseline: number;
  height: number;
  style?: CSSProperties;
}) {
  const face = useWarmTheme().fontFamily ?? SANS;
  return (
    <Stage offset={(baseline - 0.5) * height - BASELINE_DROP * fontSize}>
      <div style={{ ...lineStyle(fontSize, tracking, face), ...style }}>{children}</div>
    </Stage>
  );
}

function Stage({
  offset,
  children,
  align = "center",
}: {
  offset: number;
  children: ReactNode;
  align?: "center" | "flex-start";
}) {
  return (
    <AbsoluteFill style={{ alignItems: align, justifyContent: "center" }}>
      <div style={{ transform: `translateY(${offset}px)` }}>{children}</div>
    </AbsoluteFill>
  );
}

interface LineFit {
  text: string;
  fontSize: number;
  tracking: number;
  maxWidth: number;
}

function useFitScales(lines: LineFit[]): {
  probes: ReactNode;
  scales: number[];
} {
  const face = useWarmTheme().fontFamily ?? SANS;
  const refs = useRef<(HTMLSpanElement | null)[]>([]);
  const [handle] = useState(() => delayRender("announce-title: measure lines"));
  const [measured, setMeasured] = useState<{
    key: string;
    widths: number[];
  } | null>(null);
  const count = lines.length;
  const key = lines.map((l) => `${l.text}@${Math.round(l.fontSize)}`).join("|");

  useEffect(() => {
    setMeasured({
      key,
      widths: Array.from({ length: count }, (_, i) => refs.current[i]?.offsetWidth ?? 0),
    });
  }, [key, count]);

  useEffect(() => {
    if (measured) continueRender(handle);
  }, [measured, handle]);

  const probes = (
    <div
      style={{
        position: "absolute",
        top: 0,
        left: 0,
        visibility: "hidden",
        pointerEvents: "none",
      }}
    >
      {lines.map((line, i) => (
        <span
          key={i}
          ref={(el) => {
            refs.current[i] = el;
          }}
          style={lineStyle(line.fontSize, line.tracking, face)}
        >
          {line.text}
        </span>
      ))}
    </div>
  );

  const scales = lines.map((line, i) => {
    const w = measured?.key === key ? measured.widths[i] : undefined;
    return w && w > line.maxWidth ? line.maxWidth / w : 1;
  });

  return { probes, scales };
}

export const MARK_PATH =
  "M15.757 15.459c-3.324 0.816 -6.07 2.966 -7.563 5.911 -1.194 2.388 -1.174 1.672 -1.174 24.66 0 19.982 0.02 21.077 0.378 22.132 0.955 2.926 2.946 4.498 6.588 5.215 1.99 0.398 2.528 0.657 3.702 1.732 1.055 0.975 1.473 2.169 1.473 4.219 0.02 2.408 0.836 3.901 2.647 4.856l1.035 0.537 19.505 0.06c17.574 0.06 19.624 0.02 20.739 -0.259 1.493 -0.418 2.647 -1.333 3.403 -2.766l0.537 -1.015 0.06 -6.269c0.04 -3.443 0.04 -6.807 0 -7.444l-0.06 -1.194 -0.617 0.995c-0.717 1.174 -2.01 2.289 -3.383 2.906l-0.975 0.458 -16.121 0.06c-11.763 0.04 -16.42 -0 -17.216 -0.159 -2.548 -0.557 -5.135 -2.408 -6.468 -4.657 -1.294 -2.189 -1.314 -2.548 -1.254 -16.44l0.06 -12.439 0.478 -1.154c0.876 -2.209 2.926 -4.18 5.374 -5.155l1.115 -0.458 16.519 -0.06c16.101 -0.04 16.539 -0.04 17.813 0.358 1.592 0.498 2.966 1.473 3.941 2.826l0.736 1.035 0.06 -6.648c0.06 -7.304 -0.06 -8.319 -1.115 -9.832 -0.597 -0.876 -1.95 -1.811 -3.025 -2.11 -0.438 -0.119 -9.096 -0.199 -23.386 -0.179 -18.37 0.02 -22.908 0.06 -23.804 0.279zM79.665 31.819c-6.508 3.901 -11.902 7.185 -11.981 7.304 -0.08 0.119 -0.139 5.055 -0.1 10.947l0.04 10.748 2.886 1.811c1.592 0.995 4.14 2.587 5.672 3.523 1.533 0.935 5.632 3.463 9.096 5.613 3.463 2.15 6.488 3.901 6.707 3.901 0.219 -0 0.537 -0.139 0.697 -0.318 0.279 -0.279 0.318 -2.806 0.318 -24.958 0 -15.544 -0.08 -24.839 -0.199 -25.157 -0.139 -0.398 -0.318 -0.517 -0.736 -0.517 -0.358 0.02 -4.617 2.448 -12.399 7.105z";

export const SPARK_RAMP: readonly (readonly [number, string])[] = [
  [0, "#0365f3"],
  [0.17, "#1c5cf1"],
  [0.31, "#3a54ec"],
  [0.41, "#554ce5"],
  [0.52, "#7041cb"],
  [0.62, "#9b3894"],
  [0.72, "#ca3068"],
  [0.83, "#e2284c"],
  [0.93, "#e71d32"],
  [1, "#e91927"],
];

const BLOOM_PASSES = [0.0825, 0.0315, 0.009, 0.009];

function symbolColorList(colors?: string[] | string): string[] {
  const list = typeof colors === "string" ? colors.split(",") : (colors ?? []);
  return list.map((c) => c.trim()).filter(Boolean);
}

function symbolStops(colors: string[]): readonly (readonly [number, string])[] {
  if (colors.length === 0) return SPARK_RAMP;
  if (colors.length === 1)
    return [
      [0, colors[0]],
      [1, colors[0]],
    ];
  return colors.map((c, i) => [i / (colors.length - 1), c] as const);
}

function Mark({
  path,
  colors,
  size,
  width,
  rotation,
  spread = 1,
  id,
}: {
  path: string;
  colors: string[];
  size: number;
  width: number;
  rotation: number;
  spread?: number;
  id: string;
}) {
  if (!path) return null;
  const stops = symbolStops(colors);
  const edge = 0.5 + spread / 2;
  const blur = (f: number) => (f * width * 100) / size;
  return (
    <svg
      viewBox="0 0 100 100"
      width={size}
      height={size}
      style={{
        display: "block",
        overflow: "visible",
        transform: `rotate(${rotation}deg)`,
      }}
    >
      <title>symbol</title>
      <defs>
        <linearGradient id={id} x1="0.5" y1={edge} x2="0.5" y2={1 - edge}>
          {stops.map(([at, color]) => (
            <stop key={at} offset={`${at * 100}%`} stopColor={color} />
          ))}
        </linearGradient>
        {[...new Set(BLOOM_PASSES)].map((r) => (
          <filter key={r} id={`${id}-${r}`} x="-300%" y="-300%" width="700%" height="700%">
            <feGaussianBlur stdDeviation={blur(r)} />
          </filter>
        ))}
      </defs>
      {BLOOM_PASSES.map((r, i) => (
        <path
          key={i}
          d={path}
          fill={`url(#${id})`}
          filter={`url(#${id}-${r})`}
          style={{ mixBlendMode: "screen" }}
        />
      ))}
      <path d={path} fill={`url(#${id})`} />
    </svg>
  );
}

const PLANE_MAG: readonly (readonly [number, number])[] = [
  [2, 52.6],
  [3, 43.7],
  [4, 36.3],
  [5, 30.2],
  [6, 25.1],
  [7, 19.8],
  [8, 16.1],
  [9, 13.6],
  [10, 11.75],
  [11, 10.4],
  [12, 9.3],
  [13, 8.45],
  [14, 7.5],
  [15, 6.35],
  [16, 5.2],
  [17, 4.0],
];
const PLANE_BASE = 6;
const PLANE_PERSPECTIVE = 0.36;
const PLANE_RAKE: readonly (readonly [number, number])[] = [
  [5, 81],
  [7, 68],
  [9, 55],
  [11, 41],
  [13, 31],
  [15, 20],
  [17, 14],
];

function RecedingPlane({
  text,
  frame,
  width,
  height,
  fontSize,
  ink,
}: {
  text: string;
  frame: number;
  width: number;
  height: number;
  fontSize: number;
  ink: string;
}) {
  const face = useWarmTheme().fontFamily ?? SANS;
  const mag = sample(PLANE_MAG, frame);
  const rake = sample(PLANE_RAKE, frame);
  return (
    <Stage offset={(EYEBROW_BASELINE - 0.5) * height - BASELINE_DROP * fontSize}>
      <div style={{ transform: `scale(${mag / PLANE_BASE})` }}>
        <div
          style={{
            perspective: PLANE_PERSPECTIVE * width,
            transformStyle: "preserve-3d",
          }}
        >
          <div
            style={{
              ...lineStyle(fontSize * PLANE_BASE, EYEBROW_TRACK, face),
              color: ink,
              transform: `rotateX(${rake}deg)`,
            }}
          >
            {text}
          </div>
        </div>
      </div>
    </Stage>
  );
}

const EYEBROW_ENTER = 0.53;
const EYEBROW_DECAY = 0.7605;
const EYEBROW_SPAN = 18;
const PIVOT_LIFT = 23.87 / 41.6;

const EXIT_FROM = 35;
const EXIT_BASE = 3.2;
const EXIT_STEP = 2.1 / REF_W;
const EXIT_LETTER_DELAY = 0.215;

function eyebrowExit(frame: number, letter: number, width: number): number {
  const t = frame - EXIT_FROM - letter * EXIT_LETTER_DELAY;
  if (t < 0) return 0;
  return -EXIT_STEP * width * EXIT_BASE ** t;
}

function EyebrowLine({
  text,
  frame,
  width,
  height,
  fontSize,
  ink,
}: {
  text: string;
  frame: number;
  width: number;
  height: number;
  fontSize: number;
  ink: string;
}) {
  const scale = 1 + EYEBROW_ENTER * (1 - settle(frame - PLANE_OUT, EYEBROW_SPAN, EYEBROW_DECAY));
  const letters = [...text];
  return (
    <Line
      fontSize={fontSize}
      tracking={EYEBROW_TRACK}
      baseline={EYEBROW_BASELINE}
      height={height}
      style={{
        color: ink,
        display: "flex",
        transformOrigin: "50% 100%",
        transform: `translateY(${PIVOT_LIFT * fontSize * (scale - 1)}px) scale(${scale})`,
      }}
    >
      {letters.map((ch, i) => (
        <span key={i} style={{ transform: `translateX(${eyebrowExit(frame, i, width)}px)` }}>
          {ch}
        </span>
      ))}
    </Line>
  );
}

const TITLE_DECAY = 0.82;
const TITLE_SPAN = 30;
const TITLE_ALPHA_DECAY = 0.79;
const TITLE_ENTRY: readonly (readonly [number, number, number])[] = [
  [41, 143, 0.39],
  [48, 87, 0.46],
  [54, 88, 0.29],
];

function TitleLine({
  words,
  frame,
  width,
  height,
  fontSize,
  color,
}: {
  words: string[];
  frame: number;
  width: number;
  height: number;
  fontSize: number;
  color: string;
}) {
  return (
    <Line
      fontSize={fontSize}
      tracking={TITLE_TRACK}
      baseline={TITLE_BASELINE}
      height={height}
      style={{ display: "flex", color }}
    >
      {words.map((word, i) => {
        const extra = Math.max(0, i - (TITLE_ENTRY.length - 1));
        const [at, travel, pop] = TITLE_ENTRY[i - extra];
        const t = frame - at - extra * 6;
        if (t < 0) return null;
        const x = ((travel * width) / REF_W) * (1 - settle(t, TITLE_SPAN, TITLE_DECAY));
        return (
          <span
            key={i}
            style={{
              opacity: 1 - (1 - pop) * TITLE_ALPHA_DECAY ** t,
              transform: `translateX(${x}px)`,
            }}
          >
            {i === 0 ? word : ` ${word}`}
          </span>
        );
      })}
    </Line>
  );
}

function nightGround(glow: string, strength: number): string {
  const [r, g, b] = hexToRgb(glow);
  return `radial-gradient(circle ${((500 / REF_W) * 100).toFixed(
    1,
  )}% at 50% 68%, rgba(${r},${g},${b},${strength}), transparent)`;
}

const MACRO_SIZE = 487 / REF_W;
const MACRO_BASELINE = 347 / REF_H;
const MACRO_HOLD = 90;
function macroPan(frame: number): number {
  const t = frame - MACRO_HOLD;
  return -(78.8 * (1 - 0.81 ** t) + 6.03 * (1.255 ** t - 1));
}
const MACRO_ANCHOR = 0.505;
const MACRO_STAR_AT_HOLD = 565 / REF_W;
const MACRO_STAR_SIZE = 1.255;
const MACRO_STAR_SPIN = 2.9;
const MACRO_STAR_Y = 60 / REF_H;
const MACRO_STAR_SPREAD = 0.55;

function MacroShot({
  tagline,
  symbol,
  frame,
  width,
  height,
  ink,
}: {
  tagline: string;
  symbol: SymbolStyle;
  frame: number;
  width: number;
  height: number;
  ink: string;
}) {
  const face = useWarmTheme().fontFamily ?? SANS;
  const pan = (macroPan(frame) * width) / REF_W;
  const fontSize = MACRO_SIZE * width;
  return (
    <>
      <Stage align="flex-start" offset={(MACRO_BASELINE - 0.5) * height - BASELINE_DROP * fontSize}>
        <div
          style={{
            ...lineStyle(fontSize, TAGLINE_TRACK, face),
            color: ink,
            transform: `translateX(${width / 2 + pan}px) translateX(${-MACRO_ANCHOR * 100}%)`,
          }}
        >
          {tagline}
        </div>
      </Stage>
      <AbsoluteFill style={{ alignItems: "center", justifyContent: "center" }}>
        <div
          style={{
            transform: `translate(${(MACRO_STAR_AT_HOLD - 0.5) * width - pan}px, ${MACRO_STAR_Y * height}px)`,
          }}
        >
          <Mark
            path={symbol.path}
            colors={symbol.colors}
            size={MACRO_STAR_SIZE * symbol.scale * width}
            width={width}
            rotation={MACRO_STAR_SPIN * (frame - MACRO_HOLD) - 50}
            spread={MACRO_STAR_SPREAD}
            id="announce-title-macro-symbol"
          />
        </div>
      </AbsoluteFill>
    </>
  );
}

const TAGLINE_TRAVEL = 417 / REF_W;
const TAGLINE_DECAY = 0.792;
const TAGLINE_SPAN = 30;
const TAGLINE_FIRST = 111.5;
const TAGLINE_STAGGER = 2.833;
const TAGLINE_INK = 693;
const WIPE_START = 124.6;
const WIPE_SPEED = 31.7 / TAGLINE_INK;
const WIPE_EDGE = 165 / TAGLINE_INK;

const TINT_RAMP: readonly (readonly [number, string])[] = [
  [0, "#8a2447"],
  [0.24, "#842647"],
  [0.33, "#722e57"],
  [0.5, "#58406d"],
  [0.66, "#434e90"],
  [0.76, "#3654ab"],
  [0.85, "#2f5ed1"],
  [1, "#3266e5"],
];

function hexToRgb(hex: string): [number, number, number] {
  const n = Number.parseInt(hex.slice(1), 16);
  return [(n >> 16) & 255, (n >> 8) & 255, n & 255];
}

function taglineFill(front: number, rest: string): string {
  const [rr, rg, rb] = hexToRgb(rest);
  const stops: string[] = [];
  const n = 14;
  for (let i = 0; i <= n; i++) {
    const p = i / n;
    let below = TINT_RAMP[0];
    let above = TINT_RAMP[TINT_RAMP.length - 1];
    for (let j = 1; j < TINT_RAMP.length; j++) {
      if (p <= TINT_RAMP[j][0]) {
        below = TINT_RAMP[j - 1];
        above = TINT_RAMP[j];
        break;
      }
    }
    const span = above[0] - below[0];
    const local = span > 0 ? (p - below[0]) / span : 0;
    const [r0, g0, b0] = hexToRgb(below[1]);
    const [r1, g1, b1] = hexToRgb(above[1]);
    const drained = clamp01((front - p) / WIPE_EDGE + 0.5);
    const mix = (a: number, b: number, target: number) =>
      Math.round((a + (b - a) * local) * (1 - drained) + target * drained);
    stops.push(
      `rgb(${mix(r0, r1, rr)},${mix(g0, g1, rg)},${mix(b0, b1, rb)}) ${(p * 100).toFixed(1)}%`,
    );
  }
  return `linear-gradient(90deg, ${stops.join(", ")})`;
}

const STAR_REST_X = 781.4 / REF_W;
const STAR_REST_Y = 227 / REF_H;
const STAR_SIZE = 60 / REF_W;
const STAR_TURN = 30;
const STAR_EXIT = 134;
const STAR_GONE = 139;

function ClosingShot({
  words,
  symbol,
  frame,
  width,
  height,
  fontSize,
  rest,
}: {
  words: string[];
  symbol: SymbolStyle;
  frame: number;
  width: number;
  height: number;
  fontSize: number;
  rest: string;
}) {
  const slide =
    -TAGLINE_TRAVEL * width * (1 - settle(frame - MACRO_OUT, TAGLINE_SPAN, TAGLINE_DECAY));
  const front = WIPE_SPEED * (frame - WIPE_START);
  const starLife = 1 - clamp01((frame - STAR_EXIT) / (STAR_GONE - STAR_EXIT));
  return (
    <>
      <Line
        fontSize={fontSize}
        tracking={TAGLINE_TRACK}
        baseline={TAGLINE_BASELINE}
        height={height}
        style={{
          display: "flex",
          transform: `translateX(${slide}px)`,
          backgroundImage: taglineFill(front, rest),
          backgroundClip: "text",
          WebkitBackgroundClip: "text",
          color: "transparent",
        }}
      >
        {words.map((word, i) => (
          <span
            key={i}
            style={{
              visibility:
                frame >= TAGLINE_FIRST + (words.length - 1 - i) * TAGLINE_STAGGER
                  ? "visible"
                  : "hidden",
            }}
          >
            {i === 0 ? word : ` ${word}`}
          </span>
        ))}
      </Line>
      {starLife > 0 && (
        <Stage offset={(STAR_REST_Y - 0.5) * height}>
          <div
            style={{
              transform: `translateX(${(STAR_REST_X - 0.5) * width + slide}px) scale(${starLife})`,
              opacity: starLife,
            }}
          >
            <Mark
              path={symbol.path}
              colors={symbol.colors}
              size={STAR_SIZE * symbol.scale * width}
              width={width}
              rotation={-STAR_TURN * (1 - settle(frame - MACRO_OUT, TAGLINE_SPAN, TAGLINE_DECAY))}
              id="announce-title-symbol"
            />
          </div>
        </Stage>
      )}
    </>
  );
}

export function AnnounceTitle({
  eyebrow = "Introducing",
  title = "Acme 2.0",
  tagline = "Ready-made scenes for Remotion, in one command.",
  voidColor = "#100022",
  fieldColor = "#5600f5",
  paperColor = "#fcfcfa",
  titleColor = "#4800c9",
  nightColor = "#000028",
  glowColor = "#08ff4b",
  glowStrength = 0.139,
  taglineColor = "#f2f8ff",
  inkColor = "#ffffff",
  symbolPath = MARK_PATH,
  symbolColors,
  symbolScale = 1,
  fontFamily,
  speed = 1,
}: AnnounceTitleProps) {
  const face = resolveFont(fontFamily) ?? SANS;
  const frame = useAuthoredFrame() * speed;
  const { width, height } = useAuthoredConfig();

  const symbol: SymbolStyle = {
    path: symbolPath,
    colors: symbolColorList(symbolColors),
    scale: symbolScale,
  };
  const titleWords = title.split(/\s+/).filter(Boolean);
  const taglineWords = tagline.split(/\s+/).filter(Boolean);

  const { probes, scales } = useFitScales([
    {
      text: eyebrow,
      fontSize: EYEBROW_SIZE * width,
      tracking: EYEBROW_TRACK,
      maxWidth: 0.55 * width,
    },
    {
      text: title,
      fontSize: TITLE_SIZE * width,
      tracking: TITLE_TRACK,
      maxWidth: 0.8 * width,
    },
    {
      text: tagline,
      fontSize: TAGLINE_SIZE * width,
      tracking: TAGLINE_TRACK,
      maxWidth: 0.88 * width,
    },
  ]);
  const eyebrowSize = EYEBROW_SIZE * width * scales[0];
  const titleSize = TITLE_SIZE * width * scales[1];
  const taglineSize = TAGLINE_SIZE * width * scales[2];

  const shot = () => {
    if (frame < PLANE_OUT) {
      return (
        <AbsoluteFill style={{ backgroundColor: voidColor }}>
          <Shutter frame={frame} samples={20} open={1}>
            {(f) => (
              <RecedingPlane
                text={eyebrow}
                frame={f}
                width={width}
                height={height}
                fontSize={eyebrowSize}
                ink={inkColor}
              />
            )}
          </Shutter>
        </AbsoluteFill>
      );
    }

    if (frame < FIELD_OUT) {
      const travel = Math.abs(eyebrowExit(frame, 0, width));
      return (
        <AbsoluteFill style={{ backgroundColor: fieldColor }}>
          <Shutter
            frame={frame}
            samples={Math.min(44, Math.max(6, travel / 4))}
            open={travel > 0 ? 1 : 0.5}
          >
            {(f) => (
              <EyebrowLine
                text={eyebrow}
                frame={f}
                width={width}
                height={height}
                fontSize={eyebrowSize}
                ink={inkColor}
              />
            )}
          </Shutter>
        </AbsoluteFill>
      );
    }

    if (frame < PAPER_OUT) {
      return (
        <AbsoluteFill style={{ backgroundColor: paperColor }}>
          <TitleLine
            words={titleWords}
            frame={frame}
            width={width}
            height={height}
            fontSize={titleSize}
            color={titleColor}
          />
        </AbsoluteFill>
      );
    }

    return (
      <AbsoluteFill
        style={{
          backgroundColor: nightColor,
          backgroundImage: nightGround(glowColor, glowStrength),
        }}
      >
        {frame < MACRO_OUT ? (
          <MacroShot
            tagline={tagline}
            symbol={symbol}
            frame={frame}
            width={width}
            height={height}
            ink={inkColor}
          />
        ) : (
          <ClosingShot
            words={taglineWords}
            symbol={symbol}
            frame={frame}
            width={width}
            height={height}
            fontSize={taglineSize}
            rest={taglineColor}
          />
        )}
      </AbsoluteFill>
    );
  };

  return (
    <WarmUIProvider theme={{ fontFamily: face }}>
      <AbsoluteFill>
        {probes}
        {shot()}
      </AbsoluteFill>
    </WarmUIProvider>
  );
}

import { isRendering } from "@opsis/runtime/environment.ts";
import { holdRender } from "@opsis/runtime/holds.ts";
import { AbsoluteFill } from "@opsis/runtime/parts.tsx";
import type { CSSProperties, ReactNode } from "react";
import { useEffect, useRef, useState } from "react";
import { fontOr, MAIN_FONT } from "#video/common/fonts.ts";
import { clamp01 } from "#video/common/math.ts";
import { useAuthoredConfig, useAuthoredFrame } from "#video/core/authored-clock.ts";
import type { ElementSettings } from "#video/generated/spec.ts";
import { WarmUIProvider, useWarmTheme } from "#video/lib/warm-ui/index.ts";

interface SymbolStyle {
  path: string;
  colors: string[];
  scale: number;
  glow: number;
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
    fontWeight: "inherit",
    lineHeight: 1,
    letterSpacing: `${tracking}em`,
    marginRight: `${-tracking}em`,
    whiteSpace: "pre",
    textRendering: "geometricPrecision",
    ...(isRendering() ? null : { willChange: "transform" as const }),
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
  const face = useWarmTheme().fontFamily ?? MAIN_FONT;
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
  const face = useWarmTheme().fontFamily ?? MAIN_FONT;
  const refs = useRef<(HTMLSpanElement | null)[]>([]);
  const [release] = useState(() => holdRender("announce-title: measure lines"));
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
    if (measured) release();
  }, [measured, release]);

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

const SPARK_RAMP: readonly (readonly [number, string])[] = [
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
  glow,
  id,
}: {
  path: string;
  colors: string[];
  size: number;
  width: number;
  rotation: number;
  spread?: number;
  glow: number;
  id: string;
}) {
  if (!path) return null;
  const stops = symbolStops(colors);
  const edge = 0.5 + spread / 2;
  function blur(f: number) {
    return (f * glow * width * 100) / size;
  }
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
  const face = useWarmTheme().fontFamily ?? MAIN_FONT;
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
  textScale,
}: {
  tagline: string;
  symbol: SymbolStyle;
  frame: number;
  width: number;
  height: number;
  ink: string;
  textScale: number;
}) {
  const face = useWarmTheme().fontFamily ?? MAIN_FONT;
  const pan = (macroPan(frame) * width) / REF_W;
  const fontSize = MACRO_SIZE * textScale * width;
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
            glow={symbol.glow}
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

function hexToRgb(hex: string): [number, number, number] {
  const n = Number.parseInt(hex.slice(1), 16);
  return [(n >> 16) & 255, (n >> 8) & 255, n & 255];
}

function taglineFill(front: number, rest: string, tint: string[]): string {
  const ramp = symbolStops(tint);
  const [rr, rg, rb] = hexToRgb(rest);
  const stops: string[] = [];
  const n = 14;
  for (let i = 0; i <= n; i++) {
    const p = i / n;
    let below = ramp[0];
    let above = ramp[ramp.length - 1];
    for (let j = 1; j < ramp.length; j++) {
      if (p <= ramp[j][0]) {
        below = ramp[j - 1];
        above = ramp[j];
        break;
      }
    }
    const span = above[0] - below[0];
    const local = span > 0 ? (p - below[0]) / span : 0;
    const [r0, g0, b0] = hexToRgb(below[1]);
    const [r1, g1, b1] = hexToRgb(above[1]);
    const drained = clamp01((front - p) / WIPE_EDGE + 0.5);
    function mix(a: number, b: number, target: number) {
      return Math.round((a + (b - a) * local) * (1 - drained) + target * drained);
    }
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
  tint,
}: {
  words: string[];
  symbol: SymbolStyle;
  frame: number;
  width: number;
  height: number;
  fontSize: number;
  rest: string;
  tint: string[];
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
          backgroundImage: taglineFill(front, rest, tint),
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
              glow={symbol.glow}
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
  eyebrow,
  title,
  tagline,
  voidColor,
  fieldColor,
  paperColor,
  titleColor,
  nightColor,
  glowColor,
  glowStrength,
  taglineColor,
  inkColor,
  symbolPath,
  symbolColors,
  symbolScale,
  fontFamily,
  speed,
  fontWeight,
  textScale,
  symbolGlow,
  taglineTint,
}: ElementSettings["announce-title"]) {
  const face = fontOr(fontFamily, "main");
  const frame = useAuthoredFrame() * speed;
  const { width, height } = useAuthoredConfig();

  const symbol: SymbolStyle = {
    path: symbolPath,
    colors: symbolColors,
    scale: symbolScale,
    glow: symbolGlow,
  };
  const titleWords = title.split(/\s+/).filter(Boolean);
  const taglineWords = tagline.split(/\s+/).filter(Boolean);

  const { probes, scales } = useFitScales([
    {
      text: eyebrow,
      fontSize: EYEBROW_SIZE * textScale * width,
      tracking: EYEBROW_TRACK,
      maxWidth: 0.55 * width,
    },
    {
      text: title,
      fontSize: TITLE_SIZE * textScale * width,
      tracking: TITLE_TRACK,
      maxWidth: 0.8 * width,
    },
    {
      text: tagline,
      fontSize: TAGLINE_SIZE * textScale * width,
      tracking: TAGLINE_TRACK,
      maxWidth: 0.88 * width,
    },
  ]);
  const eyebrowSize = EYEBROW_SIZE * textScale * width * scales[0];
  const titleSize = TITLE_SIZE * textScale * width * scales[1];
  const taglineSize = TAGLINE_SIZE * textScale * width * scales[2];

  function shot() {
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
            textScale={textScale}
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
            tint={taglineTint}
          />
        )}
      </AbsoluteFill>
    );
  }

  return (
    <WarmUIProvider theme={{ fontFamily: face }}>
      <AbsoluteFill style={{ fontWeight }}>
        {probes}
        {shot()}
      </AbsoluteFill>
    </WarmUIProvider>
  );
}

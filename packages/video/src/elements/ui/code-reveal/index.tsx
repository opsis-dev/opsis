import { interpolate, spring, useCurrentFrame, useVideoConfig } from "remotion";
import {
  CODE_THEMES,
  CodeLine,
  lineLength,
  MONO_ADVANCE,
  tokenizeCode,
} from "#video/lib/code/code-syntax.tsx";
import { EASING } from "#video/lib/motion/motion-tokens.ts";
import { Caret, stageScale } from "#video/lib/text/ai-composer-utils.tsx";

const fontFamily = '"JetBrains Mono"';

const REF_WIDTH = 1280;
const REF_HEIGHT = 720;

const WINDOW_WIDTH = 940;
const HEADER_HEIGHT = 46;
const BODY_PADDING = 24;
const ROW_HEIGHT = 34;
const FONT_SIZE = 21;
const GUTTER_WIDTH = 62;
const CODE_INSET = 20;
const MAX_VISIBLE_ROWS = 10;

const WRITE_CPS = 55;
const NEWLINE_PAUSE = 0.07;

const DEFAULT_CODE = `import { CodeReveal } from "#video/elements/ui/code-reveal/index.tsx";

export const Explainer = () => (
  <CodeReveal
    title="pipeline.ts"
    highlightedLines={[4, 5]}
  />
);`;

export type CodeRevealProps = {
  code?: string;
  highlightedLines?: number[];
  title?: string;
  language?: string;
  startLine?: number;
  showLineNumbers?: boolean;
  accentColor?: string;
  backgroundColor?: string;
  theme?: "dark" | "light";
  speed?: number;
};

const CHROME_LIGHTS = ["#FF5F57", "#FEBC2E", "#28C840"] as const;

export const CodeReveal: React.FC<CodeRevealProps> = ({
  code = DEFAULT_CODE,
  highlightedLines = [],
  title = "explainer.tsx",
  language = "tsx",
  startLine = 1,
  showLineNumbers = true,
  accentColor = "#E8B86D",
  backgroundColor,
  theme = "dark",
  speed = 1,
}) => {
  const rawFrame = useCurrentFrame();
  const { width, height, fps } = useVideoConfig();
  const palette = CODE_THEMES[theme];
  const page = backgroundColor ?? palette.page;
  const scale = stageScale(width, height, REF_WIDTH, REF_HEIGHT);
  const frame = rawFrame * speed;

  const seconds = (value: number) => value * fps;
  const clamp = {
    extrapolateLeft: "clamp" as const,
    extrapolateRight: "clamp" as const,
  };

  const lines = code
    .replace(/\s+$/, "")
    .replace(/^(\r?\n)+/, "")
    .split(/\r?\n/);
  const longest = Math.max(0, ...lines.map((line) => line.length));
  const windowWidth = Math.max(
    WINDOW_WIDTH,
    Math.ceil(
      (showLineNumbers ? GUTTER_WIDTH + CODE_INSET : BODY_PADDING) +
        longest * FONT_SIZE * MONO_ADVANCE +
        BODY_PADDING * 2,
    ),
  );
  const fit = Math.min(1, (REF_WIDTH * 0.94) / windowWidth);
  const tokenLines = tokenizeCode(lines);
  const focused = new Set(highlightedLines);
  const hasFocus = highlightedLines.length > 0;

  const writeStart = seconds(0.4);
  const plan: Array<{
    tokens: (typeof tokenLines)[number];
    indent: number;
    cost: number;
    startsAt: number;
    endsAt: number;
  }> = [];
  let cursorFrame = writeStart;
  for (const [index, tokens] of tokenLines.entries()) {
    const raw = lines[index];
    const indent = raw.length - raw.trimStart().length;
    const cost = Math.max(0, lineLength(tokens) - indent);
    const startsAt = cursorFrame;
    const endsAt = startsAt + (cost / WRITE_CPS) * fps;
    cursorFrame = endsAt + seconds(NEWLINE_PAUSE);
    plan.push({ tokens, indent, cost, startsAt, endsAt });
  }

  const writeEnd = plan[plan.length - 1]?.endsAt ?? writeStart;
  const focusAt = writeEnd + seconds(0.34);

  const writtenChars = (row: (typeof plan)[number]) =>
    row.indent +
    Math.round(
      interpolate(
        frame,
        [row.startsAt, Math.max(row.endsAt, row.startsAt + 1)],
        [0, row.cost],
        clamp,
      ),
    );

  const enter = spring({
    fps,
    frame,
    config: { damping: 18, stiffness: 120, mass: 0.8 },
  });
  const chromeIn = interpolate(frame, [seconds(0.1), seconds(0.4)], [0, 1], {
    easing: EASING.enter,
    ...clamp,
  });
  const focusProgress = hasFocus
    ? interpolate(frame, [focusAt, focusAt + seconds(0.5)], [0, 1], {
        easing: EASING.enter,
        ...clamp,
      })
    : 0;

  const charWidth = FONT_SIZE * MONO_ADVANCE;
  const writing = plan.findIndex((row) => frame < row.endsAt);
  const caretRow = writing === -1 ? plan.length - 1 : writing;
  const caretPlan = plan[caretRow];
  const caretChars = caretPlan ? writtenChars(caretPlan) : 0;
  const caretResting = frame >= writeEnd;

  const visibleRows = Math.min(plan.length, MAX_VISIBLE_ROWS);
  const bodyHeight = visibleRows * ROW_HEIGHT + BODY_PADDING * 2;
  const openRows = plan.reduce(
    (total, row) =>
      total +
      interpolate(frame, [row.startsAt, row.startsAt + 6], [0, 1], {
        easing: EASING.enter,
        ...clamp,
      }),
    0,
  );
  const scrollY = Math.max(0, openRows - visibleRows) * ROW_HEIGHT;
  const codeInset = showLineNumbers ? GUTTER_WIDTH + CODE_INSET : BODY_PADDING;

  return (
    <div
      style={{
        width,
        height,
        background: page,
        fontFamily,
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        overflow: "hidden",
        position: "relative",
      }}
    >
      <div
        style={{
          position: "absolute",
          inset: 0,
          background: `radial-gradient(ellipse 70% 50% at 50% 38%, color-mix(in srgb, ${accentColor} 11%, transparent), transparent 70%)`,
          opacity: chromeIn * (0.55 + focusProgress * 0.45),
        }}
      />
      <div
        style={{
          position: "absolute",
          inset: 0,
          background:
            theme === "dark"
              ? "radial-gradient(ellipse 120% 90% at 50% 50%, transparent 45%, rgba(0,0,0,0.55) 100%)"
              : "radial-gradient(ellipse 120% 90% at 50% 50%, transparent 55%, rgba(15,18,25,0.07) 100%)",
        }}
      />

      <div
        style={{
          width: windowWidth,
          transform: `scale(${scale * fit * interpolate(enter, [0, 1], [0.965, 1])}) translateY(${interpolate(enter, [0, 1], [26, 0])}px)`,
          opacity: enter,
          borderRadius: 16,
          background: palette.window,
          border: `1px solid ${palette.border}`,
          boxShadow: `inset 0 1px 0 ${palette.highlight}, 0 34px 90px ${palette.shadow}`,
          overflow: "hidden",
          position: "relative",
        }}
      >
        <div
          style={{
            height: HEADER_HEIGHT,
            display: "flex",
            alignItems: "center",
            padding: "0 18px",
            borderBottom: `1px solid ${palette.border}`,
            background: palette.header,
            opacity: chromeIn,
            position: "relative",
          }}
        >
          <div style={{ display: "flex", gap: 8 }}>
            {CHROME_LIGHTS.map((color) => (
              <div
                key={color}
                style={{
                  width: 11,
                  height: 11,
                  borderRadius: "50%",
                  background: color,
                }}
              />
            ))}
          </div>
          <div
            style={{
              position: "absolute",
              left: 0,
              right: 0,
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              gap: 9,
              pointerEvents: "none",
            }}
          >
            <div
              style={{
                width: 7,
                height: 7,
                borderRadius: "50%",
                background: accentColor,
                opacity: interpolate(frame, [writeEnd, writeEnd + seconds(0.4)], [1, 0.2], clamp),
              }}
            />
            <span
              style={{
                color: palette.dim,
                fontSize: 15,
                fontWeight: 500,
                letterSpacing: 0.2,
              }}
            >
              {title}
            </span>
          </div>
          <span
            style={{
              marginLeft: "auto",
              color: palette.faint,
              fontSize: 13,
              letterSpacing: 0.6,
              textTransform: "uppercase",
            }}
          >
            {language}
          </span>
        </div>

        <div
          style={{
            height: bodyHeight,
            overflow: "hidden",
            position: "relative",
          }}
        >
          {showLineNumbers ? (
            <div
              style={{
                position: "absolute",
                top: 0,
                bottom: 0,
                left: 0,
                width: GUTTER_WIDTH,
                background: palette.gutter,
                borderRight: `1px solid ${palette.border}`,
              }}
            />
          ) : null}

          <div
            style={{
              position: "absolute",
              inset: 0,
              paddingTop: BODY_PADDING,
              paddingBottom: BODY_PADDING,
            }}
          >
            <div
              style={{
                transform: `translateY(${-scrollY}px)`,
                fontSize: FONT_SIZE,
                lineHeight: `${ROW_HEIGHT}px`,
                position: "relative",
              }}
            >
              {plan.map((row, index) => {
                const started = frame >= row.startsAt;
                const written = started ? writtenChars(row) : 0;
                const complete = written >= row.indent + row.cost;
                const isFocused = focused.has(index + startLine);
                const recede = hasFocus && !isFocused ? focusProgress : 0;

                return (
                  <div
                    key={`${index}-${lines[index]}`}
                    style={{
                      height: ROW_HEIGHT,
                      display: "flex",
                      alignItems: "center",
                      position: "relative",
                      opacity: started ? interpolate(recede, [0, 1], [1, 0.34]) : 0,
                    }}
                  >
                    {isFocused ? (
                      <>
                        <div
                          style={{
                            position: "absolute",
                            inset: 0,
                            background: `color-mix(in srgb, ${accentColor} 8%, transparent)`,
                            opacity: focusProgress,
                          }}
                        />
                        <div
                          style={{
                            position: "absolute",
                            left: 0,
                            top: 3,
                            bottom: 3,
                            width: 3,
                            borderRadius: 3,
                            background: accentColor,
                            opacity: focusProgress,
                            transform: `scaleY(${0.35 + focusProgress * 0.65})`,
                          }}
                        />
                      </>
                    ) : null}
                    {showLineNumbers ? (
                      <span
                        style={{
                          width: GUTTER_WIDTH,
                          flexShrink: 0,
                          paddingRight: 16,
                          textAlign: "right",
                          color: palette.faint,
                          fontVariantNumeric: "tabular-nums",
                          fontSize: FONT_SIZE * 0.82,
                          position: "relative",
                        }}
                      >
                        {index + startLine}
                        {isFocused ? (
                          <span
                            style={{
                              position: "absolute",
                              inset: 0,
                              paddingRight: 16,
                              color: accentColor,
                              opacity: focusProgress,
                            }}
                          >
                            {index + startLine}
                          </span>
                        ) : null}
                      </span>
                    ) : null}
                    <span
                      style={{
                        paddingLeft: showLineNumbers ? CODE_INSET : BODY_PADDING,
                        position: "relative",
                      }}
                    >
                      <CodeLine
                        tokens={row.tokens}
                        theme={palette}
                        reveal={complete ? undefined : written}
                      />
                    </span>
                  </div>
                );
              })}

              {frame >= writeStart && caretPlan ? (
                <div
                  style={{
                    position: "absolute",
                    top: caretRow * ROW_HEIGHT,
                    left: codeInset + caretChars * charWidth,
                    height: ROW_HEIGHT,
                    display: "flex",
                    alignItems: "center",
                  }}
                >
                  <Caret
                    color={accentColor}
                    width={charWidth * 0.9}
                    height={FONT_SIZE * 1.15}
                    radius={2}
                    blink={caretResting}
                    blinkPerSecond={1.6}
                    speed={speed}
                  />
                </div>
              ) : null}
            </div>
          </div>

          <div
            style={{
              position: "absolute",
              top: 0,
              left: 0,
              right: 0,
              height: BODY_PADDING,
              background: `linear-gradient(${palette.window}, ${palette.window}00)`,
              opacity: Math.min(1, scrollY / 10),
              pointerEvents: "none",
            }}
          />
          <div
            style={{
              position: "absolute",
              bottom: 0,
              left: 0,
              right: 0,
              height: BODY_PADDING,
              background: `linear-gradient(${palette.window}00, ${palette.window})`,
              pointerEvents: "none",
            }}
          />
        </div>
      </div>
    </div>
  );
};

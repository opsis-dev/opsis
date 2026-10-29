import { interpolate, spring, useCurrentFrame, useVideoConfig } from "remotion";
import { fontOr } from "#video/common/fonts.ts";
import type { ElementSettings } from "#video/generated/spec.ts";
import { CodeLine, codePalette, MONO_ADVANCE, tokenizeCode } from "#video/lib/code/code-syntax.tsx";
import { EASING } from "#video/lib/motion/motion-tokens.ts";
import { stageScale } from "#video/lib/text/ai-composer-utils.tsx";

const REF_WIDTH = 1280;
const REF_HEIGHT = 720;

const HEADER_HEIGHT = 46;
const BODY_PADDING = 22;
const GUTTER_WIDTH = 46;
const CODE_INSET = 18;

type DiffKind = "context" | "remove" | "add";

type DiffRow = {
  kind: DiffKind;
  text: string;
};

function diffLines(before: string[], after: string[]): DiffRow[] {
  const table: number[][] = Array.from({ length: before.length + 1 }, () =>
    Array.from({ length: after.length + 1 }, () => 0),
  );

  for (let i = before.length - 1; i >= 0; i -= 1) {
    for (let j = after.length - 1; j >= 0; j -= 1) {
      table[i][j] =
        before[i] === after[j]
          ? table[i + 1][j + 1] + 1
          : Math.max(table[i + 1][j], table[i][j + 1]);
    }
  }

  const rows: DiffRow[] = [];
  let i = 0;
  let j = 0;

  while (i < before.length && j < after.length) {
    if (before[i] === after[j]) {
      rows.push({ kind: "context", text: before[i] });
      i += 1;
      j += 1;
    } else if (table[i + 1][j] >= table[i][j + 1]) {
      rows.push({ kind: "remove", text: before[i] });
      i += 1;
    } else {
      rows.push({ kind: "add", text: after[j] });
      j += 1;
    }
  }

  while (i < before.length) {
    rows.push({ kind: "remove", text: before[i] });
    i += 1;
  }
  while (j < after.length) {
    rows.push({ kind: "add", text: after[j] });
    j += 1;
  }

  return rows;
}

const CHROME_LIGHTS = ["#FF5F57", "#FEBC2E", "#28C840"] as const;

function Plus({ size, color }: { size: number; color: string }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none">
      <path d="M12 5.5v13M5.5 12h13" stroke={color} strokeWidth={2.6} strokeLinecap="round" />
    </svg>
  );
}

function Minus({ size, color }: { size: number; color: string }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none">
      <path d="M5.5 12h13" stroke={color} strokeWidth={2.6} strokeLinecap="round" />
    </svg>
  );
}

export function CodeDiffWipe({
  before,
  after,
  title,
  wipeSeconds,
  accentColor,
  backgroundColor,
  theme,
  speed,
  fontSize: FONT_SIZE,
  windowWidth: WINDOW_WIDTH,
  lineHeight,
  startAt,
  collapseRemoved,
  showCounts,
  dimUnchanged,
  strikeRemoved,
  glow,
  vignette,
  cornerRadius,
  windowDots,
  font,
}: ElementSettings["code-diff-wipe"]) {
  const ROW_HEIGHT = FONT_SIZE * lineHeight;
  const fontFamily = fontOr(font, "code");
  const rawFrame = useCurrentFrame();
  const { width, height, fps } = useVideoConfig();
  const palette = codePalette(theme);
  const dark = theme.mode === "dark";
  const page = backgroundColor || palette.page;
  const scale = stageScale(width, height, REF_WIDTH, REF_HEIGHT);
  const frame = rawFrame * speed;

  const seconds = (value: number) => value * fps;
  const clamp = {
    extrapolateLeft: "clamp" as const,
    extrapolateRight: "clamp" as const,
  };

  const rows = diffLines(
    before
      .replace(/\s+$/, "")
      .replace(/^(\r?\n)+/, "")
      .split(/\r?\n/),
    after
      .replace(/\s+$/, "")
      .replace(/^(\r?\n)+/, "")
      .split(/\r?\n/),
  );
  const tokenRows = tokenizeCode(rows.map((row) => row.text));
  const longest = Math.max(0, ...rows.map((row) => row.text.length));
  const windowWidth = Math.max(
    WINDOW_WIDTH,
    Math.ceil(GUTTER_WIDTH + CODE_INSET + longest * FONT_SIZE * MONO_ADVANCE + BODY_PADDING * 2),
  );
  const tallest = HEADER_HEIGHT + rows.length * ROW_HEIGHT + BODY_PADDING * 2;
  const fit = Math.min(1, (REF_WIDTH * 0.94) / windowWidth, (REF_HEIGHT * 0.92) / tallest);

  const added = rows.filter((row) => row.kind === "add").length;
  const removed = rows.filter((row) => row.kind === "remove").length;

  const addColor = palette.added;
  const removeColor = palette.removed;

  const wipeStart = seconds(startAt);
  const wipeFrames = seconds(wipeSeconds);
  const rowTrigger = (index: number) =>
    wipeStart + (rows.length <= 1 ? 0 : (index / rows.length) * wipeFrames);
  const wipeEnd = wipeStart + wipeFrames;

  const rowProgress = (index: number) =>
    interpolate(frame, [rowTrigger(index), rowTrigger(index) + seconds(0.3)], [0, 1], {
      easing: EASING.enter,
      ...clamp,
    });

  const enter = spring({
    fps,
    frame,
    config: { damping: 18, stiffness: 120, mass: 0.8 },
  });
  const chromeIn = interpolate(frame, [seconds(0.1), seconds(0.4)], [0, 1], {
    easing: EASING.enter,
    ...clamp,
  });
  const settle = interpolate(frame, [wipeEnd + seconds(0.7), wipeEnd + seconds(1.4)], [0, 1], {
    easing: EASING.enter,
    ...clamp,
  });

  const dropOld = interpolate(frame, [wipeEnd + seconds(0.3), wipeEnd + seconds(0.75)], [0, 1], {
    easing: EASING.enter,
    ...clamp,
  });

  const heights = rows.map((row, index) => {
    const progress = rowProgress(index);
    if (row.kind === "remove") return collapseRemoved ? ROW_HEIGHT * (1 - dropOld) : ROW_HEIGHT;
    if (row.kind === "add") return ROW_HEIGHT * progress;
    return ROW_HEIGHT;
  });

  const frontIndex = interpolate(frame, [wipeStart, wipeEnd], [0, rows.length], {
    ...clamp,
  });
  const frontY = heights
    .slice(0, Math.floor(frontIndex))
    .reduce((total, value) => total + value, 0);
  const frontVisible = interpolate(
    frame,
    [wipeStart - seconds(0.15), wipeStart, wipeEnd, wipeEnd + seconds(0.3)],
    [0, 1, 1, 0],
    clamp,
  );

  const bodyHeight = heights.reduce((total, value) => total + value, 0) + BODY_PADDING * 2;
  const glyphSize = FONT_SIZE * 0.8;

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
          background: `radial-gradient(ellipse 70% 50% at 50% 38%, color-mix(in srgb, ${accentColor} ${glow ? 11 : 0}%, transparent), transparent 70%)`,
          opacity: chromeIn,
        }}
      />
      <div
        style={{
          position: "absolute",
          inset: 0,
          background: !vignette
            ? undefined
            : dark
              ? "radial-gradient(ellipse 120% 90% at 50% 50%, transparent 45%, rgba(0,0,0,0.55) 100%)"
              : "radial-gradient(ellipse 120% 90% at 50% 50%, transparent 55%, rgba(15,18,25,0.07) 100%)",
        }}
      />

      <div
        style={{
          width: windowWidth,
          transform: `scale(${scale * fit * interpolate(enter, [0, 1], [0.965, 1])}) translateY(${interpolate(enter, [0, 1], [26, 0])}px)`,
          opacity: enter,
          borderRadius: cornerRadius,
          background: palette.window,
          border: `1px solid ${palette.border}`,
          boxShadow: `inset 0 1px 0 ${palette.highlight}, 0 34px 90px ${palette.shadow}`,
          overflow: "hidden",
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
          <div style={{ display: windowDots ? "flex" : "none", gap: 8 }}>
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
          <span
            style={{
              position: "absolute",
              left: 0,
              right: 0,
              textAlign: "center",
              color: palette.dim,
              fontSize: 15,
              fontWeight: 500,
              letterSpacing: 0.2,
              pointerEvents: "none",
            }}
          >
            {title}
          </span>
          <span
            style={{
              marginLeft: "auto",
              display: showCounts ? "flex" : "none",
              gap: 12,
              fontSize: 14,
              fontVariantNumeric: "tabular-nums",
              opacity: interpolate(
                frame,
                [wipeStart - seconds(0.2), wipeStart + seconds(0.2)],
                [0, 1],
                clamp,
              ),
            }}
          >
            <span style={{ color: addColor }}>
              +{Math.round(interpolate(frame, [wipeStart, wipeEnd], [0, added], clamp))}
            </span>
            <span style={{ color: removeColor }}>
              −{Math.round(interpolate(frame, [wipeStart, wipeEnd], [0, removed], clamp))}
            </span>
          </span>
        </div>

        <div
          style={{
            height: bodyHeight,
            paddingTop: BODY_PADDING,
            paddingBottom: BODY_PADDING,
            position: "relative",
            overflow: "hidden",
          }}
        >
          <div style={{ position: "relative" }}>
            {rows.map((row, index) => {
              const progress = rowProgress(index);
              const isAdd = row.kind === "add";
              const isRemove = row.kind === "remove";
              const tint = isRemove ? progress : (1 - settle) * progress;
              const color = isAdd ? addColor : isRemove ? removeColor : palette.faint;
              const revealed = isAdd ? Math.round(progress * row.text.length) : row.text.length;

              return (
                <div
                  key={`${index}-${row.text}`}
                  style={{
                    height: heights[index],
                    overflow: "hidden",
                    display: "flex",
                    alignItems: "flex-start",
                    fontSize: FONT_SIZE,
                    lineHeight: `${ROW_HEIGHT}px`,
                    background: isAdd || isRemove ? `${color}${tintAlpha(tint)}` : "none",
                    opacity: isRemove ? 1 - progress * 0.3 - dropOld * 0.5 : 1,
                  }}
                >
                  <span
                    style={{
                      width: GUTTER_WIDTH,
                      height: ROW_HEIGHT,
                      flexShrink: 0,
                      display: "flex",
                      alignItems: "center",
                      justifyContent: "center",
                    }}
                  >
                    {isAdd ? (
                      <Plus size={glyphSize} color={addColor} />
                    ) : isRemove ? (
                      <Minus size={glyphSize} color={removeColor} />
                    ) : null}
                  </span>
                  <span
                    style={{
                      paddingLeft: CODE_INSET,
                      height: ROW_HEIGHT,
                      display: "flex",
                      alignItems: "center",
                      textDecorationLine:
                        strikeRemoved && isRemove && progress > 0.15 ? "line-through" : "none",
                      textDecorationColor: removeColor,
                    }}
                  >
                    <CodeLine
                      tokens={tokenRows[index]}
                      theme={palette}
                      reveal={revealed >= row.text.length ? undefined : revealed}
                      muted={dimUnchanged && row.kind === "context"}
                    />
                  </span>
                </div>
              );
            })}

            <div
              style={{
                position: "absolute",
                left: 0,
                right: 0,
                top: frontY,
                height: 2,
                background: accentColor,
                opacity: frontVisible * 0.9,
                boxShadow: `0 0 18px 2px color-mix(in srgb, ${accentColor} 40%, transparent)`,
              }}
            />
          </div>
        </div>
      </div>
    </div>
  );
}

function tintAlpha(value: number): string {
  const alpha = Math.round(Math.max(0, Math.min(1, value)) * 30);
  return alpha.toString(16).padStart(2, "0");
}
